//! Credential provenance is checked before refresh. No identity switching or raw errors.
use crate::{core::microsoft_identity, services::SettingsManager};
use launchwerk::auth::{AccountType, MinecraftUser, microsoft::MicrosoftAuth};
use std::sync::LazyLock;
pub(crate) static ACCOUNT_SESSION_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

pub async fn active_user_fresh() -> Result<MinecraftUser, String> {
    fresh_session()
        .await
        .map_err(|code| session_message(&code, SettingsManager::read().language.as_str() == "en"))
}
fn session_message(code: &str, english: bool) -> String {
    let pair = match code {
        "NETWORK_ERROR" | "AUTH_SERVICE_UNAVAILABLE" => (
            "No se pudo verificar la sesión por un problema de conexión. Reintenta cuando tengas conexión.",
            "Your session could not be verified because of a connection problem. Retry when online.",
        ),
        "SECURE_STORE_UNAVAILABLE"
        | "SESSION_KEY_MISSING"
        | "SESSION_KEY_INVALID"
        | "SESSION_DECRYPTION_FAILED" => (
            "No se pudo acceder a las credenciales. Desbloquea el almacén seguro del sistema o renueva la sesión en Ajustes → Cuentas.",
            "Credentials could not be accessed. Unlock the system secure store or sign in again in Settings → Accounts.",
        ),
        "MINECRAFT_APP_NOT_APPROVED" => (
            "Minecraft rechazó el registro de la aplicación (403 Invalid app registration). El responsable del launcher debe confirmar su aprobación.",
            "Minecraft rejected the application registration (403 Invalid app registration). The launcher maintainer must confirm its approval.",
        ),
        "MINECRAFT_PROFILE_REQUIRED" | "MINECRAFT_PROFILE_INVALID" => (
            "Esta cuenta no tiene un perfil de Minecraft Java válido. Comprueba su licencia y renueva la sesión.",
            "This account has no valid Minecraft Java profile. Check its license and sign in again.",
        ),
        "XBOX_LIVE_ERROR"
        | "XBOX_XSTS_ERROR"
        | "XBOX_PROFILE_REQUIRED"
        | "XBOX_FAMILY_RESTRICTION" => (
            "Xbox no pudo autorizar esta cuenta. Revisa su perfil y restricciones, y renueva la sesión en Ajustes → Cuentas.",
            "Xbox could not authorize this account. Review its profile and restrictions, then sign in again in Settings → Accounts.",
        ),
        "LOCAL_STORAGE_ERROR" => (
            "No se pudieron guardar los datos de la sesión. Comprueba los permisos y el espacio disponible.",
            "Session data could not be saved. Check permissions and free disk space.",
        ),
        _ => (
            "La sesión de Microsoft necesita atención. Vuelve a iniciar sesión en Ajustes → Cuentas.",
            "Your Microsoft session needs attention. Sign in again in Settings → Accounts.",
        ),
    };
    if english {
        pair.1.into()
    } else {
        pair.0.into()
    }
}
fn can_use_cached(user: &MinecraftUser, code: &str, now: u64) -> bool {
    matches!(code, "NETWORK_ERROR" | "AUTH_SERVICE_UNAVAILABLE")
        && user.cached_session_valid_at(now)
}
async fn fresh_session() -> Result<MinecraftUser, String> {
    let _guard = ACCOUNT_SESSION_LOCK.lock().await;
    let mut user = SettingsManager::read().get_user();
    if user.user_type == AccountType::Cracked {
        return Ok(user);
    }
    let copy = user.clone();
    user = tokio::task::spawn_blocking(move || {
        let mut user = copy;
        user.load_tokens().map(|_| user)
    })
    .await
    .map_err(|_| "LOCAL_STORAGE_ERROR".to_string())?
    .map_err(|e| crate::services::microsoft_login::safe_error(&e.to_string()))?;
    if user.user_type == AccountType::Microsoft {
        let identity = microsoft_identity::configured()?;
        if user.requires_reauthorization(&identity.client_id) {
            return Err("REAUTHENTICATION_REQUIRED".into());
        }
        let refresh = user
            .refresh_token
            .clone()
            .ok_or("REAUTHENTICATION_REQUIRED")?;
        let result = tokio::task::spawn_blocking(move || {
            MicrosoftAuth::new(identity.client_id).refresh_token(&refresh)
        })
        .await
        .map_err(|_| "LOCAL_STORAGE_ERROR".to_string())?
        .map_err(|e| crate::services::microsoft_login::safe_error(&e.to_string()));
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(u64::MAX);
        let mut fresh = match result {
            Ok(user) => user,
            Err(code) if can_use_cached(&user, &code, now) => {
                crate::core::emit(crate::core::AppEvent::AuthSessionNotice {
                    reason: "cached_valid_session".into(),
                });
                return Ok(user);
            }
            Err(code) => return Err(code),
        };
        if fresh.uuid != user.uuid {
            return Err("ACCOUNT_MISMATCH".into());
        }
        fresh.session_only = user.session_only;
        let copy = fresh.clone();
        tokio::task::spawn_blocking(move || copy.save_tokens())
            .await
            .map_err(|_| "LOCAL_STORAGE_ERROR".to_string())?
            .map_err(|_| "SECURE_STORE_UNAVAILABLE".to_string())?;
        user = fresh;
    }
    SettingsManager::write(|s| {
        if let Some(stored) = s.user.iter_mut().find(|u| u.uuid == user.uuid) {
            *stored = user.clone();
        }
    })
    .map_err(|_| "LOCAL_STORAGE_ERROR".to_string())?;
    SettingsManager::save()
        .await
        .map_err(|_| "LOCAL_STORAGE_ERROR".to_string())?;
    Ok(user)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_known_valid_cached_sessions_survive_transient_network_failure() {
        let mut user =
            MinecraftUser::premium("Fixture".into(), "id".into(), "not-real".into(), None);
        assert!(!can_use_cached(&user, "NETWORK_ERROR", 1000));
        user.access_token_expires_at = Some(2000);
        assert!(can_use_cached(&user, "NETWORK_ERROR", 1000));
        assert!(!can_use_cached(&user, "NETWORK_ERROR", 1950));
        assert!(!can_use_cached(&user, "REAUTHENTICATION_REQUIRED", 1000));
        assert!(!can_use_cached(&user, "MINECRAFT_APP_NOT_APPROVED", 1000));
        assert!(session_message("MINECRAFT_APP_NOT_APPROVED", false).contains("403"));
    }
}
