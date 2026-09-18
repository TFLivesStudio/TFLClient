use crate::core::http_client::get_json_retrying;
use crate::services::SettingsManager;
use base64::Engine;
use launchwerk::auth::MinecraftUser;
use launchwerk::auth::microsoft::MicrosoftAuth;
use serde::Serialize;
use tauri::command;
use tracing::info;

#[derive(Serialize)]
pub struct DeviceCode {
    pub user_code: String,
    pub device_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[command]
pub async fn get_device_code() -> Result<DeviceCode, String> {
    info!("Obteniendo código de dispositivo de Microsoft");
    let res = tokio::task::spawn_blocking(|| {
        MicrosoftAuth::default()
            .get_device_code()
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(DeviceCode {
        user_code: res.user_code,
        device_code: res.device_code,
        verification_uri: res.verification_uri,
        expires_in: res.expires_in,
        interval: res.interval,
    })
}

#[command]
pub async fn authenticate_with_device_code(
    device_code: String,
    interval: u64,
    expires_in: u64,
) -> Result<MinecraftUser, String> {
    let user = tokio::task::spawn_blocking(move || {
        MicrosoftAuth::default()
            .authenticate_with_device_code(&device_code, interval, expires_in)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    SettingsManager::write(|s| {
        s.add_user(user.clone());
        s.onboarded = true;
    })
    .map_err(|e| e.to_string())?;
    SettingsManager::save().await?;
    Ok(user)
}

#[command]
pub async fn add_offline_account(username: String) -> Result<MinecraftUser, String> {
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
        .map(String::from)
}

/// Cuentas offline/cracked solo pueden jugar en singleplayer (regla de
/// producto de TFL Client). Ver `crate::core::allows_multiplayer`.
#[command]
pub fn account_allows_multiplayer() -> bool {
    crate::core::allows_multiplayer(SettingsManager::read().get_user().user_type)
}

#[command]
pub async fn logout() -> Result<(), String> {
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
    SettingsManager::write(|s| {
        s.user.retain(|u| u.uuid != uuid);
        if s.user.is_empty() {
            s.user.push(MinecraftUser::cracked("Steve"));
        }
        if s.active_user_idx >= s.user.len() {
            s.active_user_idx = 0;
        }
    })
    .map_err(|e| e.to_string())?;
    SettingsManager::save().await?;
    Ok(())
}
