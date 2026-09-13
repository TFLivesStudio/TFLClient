use crate::services::java_manager::{self, JavaStatus};
use crate::services::{QualityProfile, SettingsManager};
use serde::Serialize;
use sysinfo::{MemoryRefreshKind, RefreshKind, System};
use tauri::command;
use tracing::info;

#[derive(Serialize, Copy, Clone)]
pub struct RecommendedRam {
    pub total_mb: u32,
    pub recommended_min_mb: u32,
    pub recommended_max_mb: u32,
}

#[command]
pub fn get_recommended_ram() -> RecommendedRam {
    let sys = System::new_with_specifics(
        RefreshKind::new().with_memory(MemoryRefreshKind::new().with_ram()),
    );
    let total_mb = (sys.total_memory() / 1024 / 1024).max(1) as u32;
    let max_mb = if total_mb <= 4096 {
        2048
    } else if total_mb <= 8192 {
        3072
    } else {
        (total_mb / 4).clamp(3072, 8192)
    };
    RecommendedRam {
        total_mb,
        recommended_min_mb: (max_mb / 2).max(512),
        recommended_max_mb: max_mb,
    }
}

#[command]
pub fn get_settings() -> SettingsManager {
    SettingsManager::snapshot()
}

#[command]
pub async fn update_settings(mut new_settings: SettingsManager) -> Result<(), String> {
    if new_settings.min_memory == 0 {
        new_settings.min_memory = 1024;
    }
    if new_settings.max_memory < new_settings.min_memory {
        new_settings.max_memory = new_settings.min_memory * 2;
    }
    SettingsManager::write(|s| *s = new_settings).map_err(|e| e.to_string())?;
    SettingsManager::save().await?;
    Ok(())
}

#[command]
pub fn get_java_status() -> Vec<JavaStatus> {
    java_manager::status()
}

#[command]
pub async fn set_quality_profile(profile: QualityProfile) -> Result<SettingsManager, String> {
    info!("Aplicando perfil de calidad: {:?}", profile);
    SettingsManager::apply_quality_profile(profile).map_err(|e| e.to_string())?;
    SettingsManager::save().await?;
    Ok(SettingsManager::snapshot())
}
