//! Credential provenance is checked before refresh. No identity switching or raw errors.
use crate::{core::microsoft_identity, services::SettingsManager};
use launchwerk::auth::{AccountType, MinecraftUser, microsoft::MicrosoftAuth};
use std::sync::LazyLock;
static REFRESH_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

pub async fn active_user_fresh() -> Result<MinecraftUser, String> {
    let _guard = REFRESH_LOCK.lock().await;
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
        if user.microsoft_client_id.as_deref() != Some(identity.client_id.as_str()) {
            return Err("REAUTHENTICATION_REQUIRED".into());
        }
        let refresh = user
            .refresh_token
            .clone()
            .ok_or("REAUTHENTICATION_REQUIRED")?;
        let mut fresh = tokio::task::spawn_blocking(move || {
            MicrosoftAuth::new(identity.client_id).refresh_token(&refresh)
        })
        .await
        .map_err(|_| "LOCAL_STORAGE_ERROR".to_string())?
        .map_err(|e| crate::services::microsoft_login::safe_error(&e.to_string()))?;
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
