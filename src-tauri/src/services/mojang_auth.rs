//! Único punto que resuelve "el usuario activo, con su sesión de Microsoft
//! al día" — lo usan tanto el lanzamiento del juego (`launcher.rs`) como los
//! comandos de perfil de Mojang (skin/capa). Antes esta lógica de refresco
//! vivía solo en `launcher.rs`; cualquier otra llamada a la API de Mojang
//! (que exige un access_token vigente) hubiera repetido el mismo refresco a
//! mano o, peor, usado un token vencido.
use crate::services::SettingsManager;
use launchwerk::auth::AccountType;
use launchwerk::auth::MinecraftUser;
use tracing::warn;

/// Devuelve el usuario activo con el access_token de Microsoft refrescado
/// best-effort — si el refresco falla (sin internet, refresh_token vencido)
/// se sigue con el token guardado en vez de fallar, para no bloquear
/// singleplayer offline por un problema de red.
pub async fn active_user_fresh() -> MinecraftUser {
    let mut user = SettingsManager::read().get_user();
    if let Err(e) = user.load_tokens() {
        warn!("No se pudieron cargar los tokens del usuario: {e:?}");
    }

    if user.user_type == AccountType::Microsoft {
        if let Some(refresh) = user.refresh_token.clone() {
            match tokio::task::spawn_blocking(move || {
                launchwerk::auth::microsoft::MicrosoftAuth::default().refresh_token(&refresh)
            })
            .await
            {
                Ok(Ok(fresh)) => {
                    if let Err(e) = fresh.save_tokens() {
                        warn!("No se pudo guardar la sesión refrescada: {e}");
                    }
                    user = fresh;
                }
                Ok(Err(e)) => {
                    warn!("No se pudo refrescar la sesión de Microsoft, se sigue con la guardada: {e}");
                }
                Err(e) => {
                    warn!("Falló la tarea de refresco de sesión: {e}");
                }
            }
        }
    }

    user
}
