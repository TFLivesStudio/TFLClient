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
    let ceiling = base
        .total_mb
        .saturating_sub(1024)
        .max(base.recommended_max_mb);
    let recommended_max_mb = (base.recommended_max_mb + bump_mb).min(ceiling);
    RecommendedRam {
        total_mb: base.total_mb,
        recommended_min_mb: base.recommended_min_mb,
        recommended_max_mb,
    }
}

#[command]
pub fn get_settings() -> SettingsManager {
    let _perf = crate::core::perf::span("settings.load");
    SettingsManager::snapshot()
}

#[command]
pub async fn patch_settings(
    app: tauri::AppHandle,
    patch: serde_json::Value,
    expected_revision: u64,
) -> Result<SettingsManager, String> {
    use tauri::Emitter;
    let settings = SettingsManager::patch(patch, expected_revision).await?;
    let _ = app.emit("settings-changed", settings.settings_revision);
    Ok(settings)
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

// Only discard old TFL download scratch. Never remove the shared directory or
// unrelated content, and do not traverse links or junctions.
fn stale_cache_size(
    path: &std::path::Path,
    cutoff: std::time::SystemTime,
    budget: &mut usize,
    depth: usize,
) -> Option<u64> {
    if depth >= 64 {
        return None;
    }
    *budget = budget.checked_sub(1)?;
    let metadata = std::fs::symlink_metadata(path).ok()?;
    if metadata.file_type().is_symlink() || metadata.modified().ok()? >= cutoff {
        return None;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return None;
        }
    }
    if metadata.is_file() {
        return Some(metadata.len());
    }
    if !metadata.is_dir() {
        return None;
    }
    let mut bytes = 0u64;
    for entry in std::fs::read_dir(path).ok()? {
        bytes = bytes.checked_add(stale_cache_size(
            &entry.ok()?.path(),
            cutoff,
            budget,
            depth + 1,
        )?)?;
    }
    Some(bytes)
}

#[command]
pub async fn clear_temp_cache() -> Result<u64, String> {
    if crate::services::progress::has_active_downloads() {
        return Ok(0);
    }
    let root = PathManager::get().get_shared_dir().join("temp");
    tokio::task::spawn_blocking(move || {
        let entries = match std::fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(e.to_string()),
        };
        let root_meta = std::fs::symlink_metadata(&root).map_err(|e| e.to_string())?;
        if root_meta.file_type().is_symlink() {
            return Ok(0);
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if root_meta.file_attributes() & 0x400 != 0 {
                return Ok(0);
            }
        }
        let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(30 * 86_400);
        let mut freed = 0u64;
        let mut budget = 20_000;
        for entry in entries.take(512).flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !["forge-", "neoforge-", "mrpack-", "natives-"]
                .iter()
                .any(|p| name.starts_with(p))
            {
                continue;
            }
            let path = entry.path();
            let Some(bytes) = stale_cache_size(&path, cutoff, &mut budget, 0) else {
                continue;
            };
            if crate::services::progress::has_active_downloads() {
                break;
            }
            let result = if path.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            };
            if result.is_ok() {
                freed = freed.saturating_add(bytes);
            }
        }
        Ok(freed)
    })
    .await
    .map_err(|_| "No se pudo revisar el almacenamiento temporal".to_string())?
}

const WALLPAPER_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

fn wallpaper_dir() -> std::path::PathBuf {
    PathManager::get().get_shared_dir().join("appearance")
}

/// Copia la imagen elegida por el usuario (vía `pick_image_file`, ya
/// existente para íconos de instancia) a un archivo fijo propio —
/// mismo patrón que `set_instance_icon`: un solo archivo activo a la
/// vez, se borra cualquier extensión vieja antes de copiar la nueva.
#[command]
pub async fn set_custom_wallpaper(source_path: String) -> Result<String, String> {
    let source = std::path::PathBuf::from(&source_path);
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .filter(|e| WALLPAPER_EXTENSIONS.contains(&e.as_str()))
        .ok_or("Formato de imagen no soportado (usa PNG, JPG o WEBP)")?;

    let dir = wallpaper_dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;
    for old_ext in WALLPAPER_EXTENSIONS {
        let _ = tokio::fs::remove_file(dir.join(format!("custom-wallpaper.{old_ext}"))).await;
    }
    let dest = dir.join(format!("custom-wallpaper.{ext}"));
    tokio::fs::copy(&source, &dest)
        .await
        .map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().to_string())
}

#[command]
pub async fn get_custom_wallpaper_path() -> Option<String> {
    let dir = wallpaper_dir();
    for ext in WALLPAPER_EXTENSIONS {
        let p = dir.join(format!("custom-wallpaper.{ext}"));
        if p.exists() {
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
}
