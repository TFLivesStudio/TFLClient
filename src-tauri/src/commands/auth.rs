use crate::core::http_client::get_json_retrying;
use crate::services::SettingsManager;
use base64::Engine;
use launchwerk::auth::MinecraftUser;
use serde::Serialize;
use tauri::command;
use tracing::info;

#[command]
pub fn microsoft_auth_config() -> Result<crate::core::microsoft_identity::MicrosoftIdentity, String>
{
    crate::core::microsoft_identity::configured()
}

#[command]
pub fn start_microsoft_auth(
    app: tauri::AppHandle,
    method: String,
    expected_uuid: Option<String>,
) -> Result<crate::services::microsoft_login::LoginSnapshot, String> {
    crate::services::microsoft_login::start(app, &method, expected_uuid)
}
#[command]
pub fn microsoft_auth_status(
    id: String,
) -> Result<crate::services::microsoft_login::LoginSnapshot, String> {
    crate::services::microsoft_login::status(&id)
}
#[command]
pub fn cancel_microsoft_auth(
    app: tauri::AppHandle,
    id: String,
) -> Result<crate::services::microsoft_login::LoginSnapshot, String> {
    crate::services::microsoft_login::cancel(&app, &id)
}
#[command]
pub async fn microsoft_auth_memory_session(
    id: String,
) -> Result<crate::services::microsoft_login::LoginSnapshot, String> {
    crate::services::microsoft_login::accept_memory(&id).await
}
#[command]
pub async fn microsoft_auth_retry_storage(
    id: String,
) -> Result<crate::services::microsoft_login::LoginSnapshot, String> {
    crate::services::microsoft_login::retry_storage(&id).await
}
#[command]
pub fn microsoft_auth_open_device(app: tauri::AppHandle, id: String) -> Result<(), String> {
    crate::services::microsoft_login::open_device(&app, &id)
}

#[derive(Serialize)]
pub struct AccountSessionStatus {
    pub uuid: String,
    pub state: String,
}
#[command]
pub async fn account_session_statuses() -> Result<Vec<AccountSessionStatus>, String> {
    let _account_guard = crate::services::mojang_auth::ACCOUNT_SESSION_LOCK
        .lock()
        .await;
    let users = SettingsManager::read().user.clone();
    let identity = crate::core::microsoft_identity::configured()?;
    tokio::task::spawn_blocking(move || {
        users
            .into_iter()
            .map(|mut user| {
                let state = if user.user_type == launchwerk::auth::AccountType::Cracked {
                    "offline"
                } else if user.session_only {
                    if user.access_token.is_empty() {
                        "reauthenticate"
                    } else {
                        "memory"
                    }
                } else if user.load_tokens().is_err() {
                    "storage_or_reauthentication"
                } else if user.user_type == launchwerk::auth::AccountType::Microsoft
                    && user.microsoft_client_id.as_deref() != Some(identity.client_id.as_str())
                {
                    "reauthenticate"
                } else {
                    "protected"
                };
                AccountSessionStatus {
                    uuid: user.uuid,
                    state: state.into(),
                }
            })
            .collect()
    })
    .await
    .map_err(|_| "LOCAL_STORAGE_ERROR".into())
}

#[command]
pub async fn add_offline_account(username: String) -> Result<MinecraftUser, String> {
    let _account_guard = crate::services::mojang_auth::ACCOUNT_SESSION_LOCK
        .lock()
        .await;
    if username.trim().is_empty() {
        return Err("El nombre de usuario no puede estar vacío".into());
    }
    let user = MinecraftUser::cracked(username);
    SettingsManager::write(|s| {
        s.add_user(user.clone());
        s.onboarded = true;
    })
    .map_err(|e| e.to_string())?;
    SettingsManager::save().await?;
    Ok(user)
}

#[command]
pub fn get_current_user() -> MinecraftUser {
    let _perf = crate::core::perf::span("accounts.load");
    SettingsManager::read().get_user()
}

/// URL de la textura de skin real de una cuenta premium (Microsoft o
/// Yggdrasil), directo del session server oficial de Mojang — sin pasar
/// por Crafatar/mc-heads.net/etc. Esos servicios de terceros gratuitos
/// son poco confiables (los dos que se probaron esta semana estuvieron
/// caídos en momentos distintos); esto es la fuente autoritativa, la
/// misma que usa el cliente oficial de Minecraft.
#[command]
pub async fn get_skin_texture_url(uuid: String) -> Option<String> {
    let clean_uuid: String = uuid.chars().filter(|c| *c != '-').collect();
    let url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{clean_uuid}");
    let profile: serde_json::Value = get_json_retrying(&url).await.ok()?;
    let textures_b64 = profile
        .get("properties")?
        .as_array()?
        .iter()
        .find(|p| p.get("name").and_then(|n| n.as_str()) == Some("textures"))?
        .get("value")?
        .as_str()?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(textures_b64)
        .ok()?;
    let parsed: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    parsed
        .get("textures")?
        .get("SKIN")?
        .get("url")?
        .as_str()
        .map(|url| {
            url.replacen(
                "http://textures.minecraft.net/",
                "https://textures.minecraft.net/",
                1,
            )
        })
}

/// Cuentas offline/cracked solo pueden jugar en singleplayer (regla de
/// producto de TFL Client). Ver `crate::core::allows_multiplayer`.
#[command]
pub fn account_allows_multiplayer() -> bool {
    crate::core::allows_multiplayer(SettingsManager::read().get_user().user_type)
}

#[command]
pub async fn logout() -> Result<(), String> {
    let _account_guard = crate::services::mojang_auth::ACCOUNT_SESSION_LOCK
        .lock()
        .await;
    info!("Cerrando sesión de usuario");
    SettingsManager::write(|s| {
        s.active_user_idx = 0;
        if s.user.is_empty() {
            s.user.push(MinecraftUser::cracked("Steve"));
        }
    })
    .map_err(|e| e.to_string())?;
    SettingsManager::save().await?;
    Ok(())
}

#[command]
pub fn get_user_list() -> Vec<MinecraftUser> {
    SettingsManager::read().user.clone()
}

#[command]
pub async fn switch_user(uuid: String) -> Result<MinecraftUser, String> {
    let _account_guard = crate::services::mojang_auth::ACCOUNT_SESSION_LOCK
        .lock()
        .await;
    let user = SettingsManager::write(|s| {
        if let Some(idx) = s.user.iter().position(|u| u.uuid == uuid) {
            s.active_user_idx = idx;
        }
    })
    .map_err(|e| e.to_string());
    user?;
    SettingsManager::save().await?;
    Ok(SettingsManager::read().get_user())
}

#[command]
pub async fn remove_user(uuid: String) -> Result<(), String> {
    let _account_guard = crate::services::mojang_auth::ACCOUNT_SESSION_LOCK
        .lock()
        .await;
    // Si la cuenta activa era otra (no la que se está borrando), reindexar
    // por posición numérica después de un `retain` la puede cambiar sola
    // sin que el usuario elija nada — ej. [A,B,C] activa=B(idx1), se borra
    // A(idx0) → queda [B,C] y el idx1 ahora apunta a C. Se recuerda el
    // uuid de la cuenta activa y se recalcula su posición real después.
    let active_uuid = {
        let s = SettingsManager::read();
        s.user.get(s.active_user_idx).map(|u| u.uuid.clone())
    };

    let removed = SettingsManager::read()
        .user
        .iter()
        .find(|u| u.uuid == uuid)
        .cloned();
    if let Some(removed) = removed {
        tokio::task::spawn_blocking(move || removed.delete_tokens())
            .await
            .map_err(|_| "LOCAL_STORAGE_ERROR".to_string())?
            .map_err(|_| "SECURE_STORE_UNAVAILABLE".to_string())?;
    }

    SettingsManager::write(|s| {
        s.user.retain(|u| u.uuid != uuid);
        if s.user.is_empty() {
            s.user.push(MinecraftUser::cracked("Steve"));
        }
        s.active_user_idx = active_uuid
            .as_deref()
            .and_then(|au| s.user.iter().position(|u| u.uuid == au))
            .unwrap_or(0);
    })
    .map_err(|e| e.to_string())?;
    SettingsManager::save().await?;
    Ok(())
}
