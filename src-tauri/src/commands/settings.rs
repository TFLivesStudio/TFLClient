use crate::core::PathManager;
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

/// Misma base que get_recommended_ram, ajustada por cantidad de mods
/// instalados — heurística simple, no una medición real de cuánta RAM
/// necesita cada mod en particular (eso no se puede saber sin perfilar
/// cada uno). Por cada mod más allá de 20 (modpacks chicos no necesitan
/// ajuste), suma 48MB al máximo recomendado — nunca por encima de
/// total_mb - 1024 (deja margen para el sistema operativo).
#[command]
pub async fn get_recommended_ram_for_instance(name: String) -> RecommendedRam {
    let base = get_recommended_ram();
    let Ok(instance) = crate::services::instance_manager::get_instance(&name).await else {
        return base;
    };
    let mods_dir = instance.dir().join("mods");
    let mod_count = match tokio::fs::read_dir(&mods_dir).await {
        Ok(mut entries) => {
            let mut count = 0u32;
            while let Ok(Some(entry)) = entries.next_entry().await {
                if entry.path().extension().and_then(|e| e.to_str()) == Some("jar") {
                    count += 1;
                }
            }
            count
        }
        Err(_) => 0,
    };
    if mod_count <= 20 {
        return base;
    }
    let bump_mb = (mod_count - 20) * 48;
    let ceiling = base.total_mb.saturating_sub(1024).max(base.recommended_max_mb);
    let recommended_max_mb = (base.recommended_max_mb + bump_mb).min(ceiling);
    RecommendedRam {
        total_mb: base.total_mb,
        recommended_min_mb: base.recommended_min_mb,
        recommended_max_mb,
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

fn dir_size_blocking(path: &std::path::Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    let mut total = 0u64;
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if meta.is_dir() {
            total += dir_size_blocking(&entry.path());
        } else {
            total += meta.len();
        }
    }
    total
}

/// Borra `shared_dir/temp` — SOLO scratch de descargas/instalaciones a
/// medio terminar (mrpacks, procesadores de Forge, etc), nunca las
/// librerías/assets/versions ya bajadas — eso obligaría a re-bajar todo
/// de nuevo, no es lo que pide "limpiar caché". Devuelve cuánto se
/// liberó para mostrarlo en la UI.
#[command]
pub async fn clear_temp_cache() -> Result<u64, String> {
    let temp_dir = PathManager::get().get_shared_dir().join("temp");
    let freed_bytes = tokio::task::spawn_blocking({
        let temp_dir = temp_dir.clone();
        move || dir_size_blocking(&temp_dir)
    })
    .await
    .map_err(|e| e.to_string())?;

    if temp_dir.exists() {
        tokio::fs::remove_dir_all(&temp_dir)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(freed_bytes)
}
