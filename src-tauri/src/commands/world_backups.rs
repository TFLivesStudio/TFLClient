//! Copias de seguridad de los mundos (`saves/`) de una instancia: crear a
//! mano, ver la lista, restaurar y borrar. Las copias automáticas que ya se
//! hacían antes de actualizar mods (`world_backup::backup_world_dir`) caen en la
//! misma carpeta, así que aparecen acá también.

use crate::services::{content, instance_manager, world_backup, worlds};
use serde::Serialize;
use std::path::Path;
use tauri::command;

#[derive(Debug, Serialize, Clone)]
pub struct WorldBackup {
    /// Nombre del archivo (`saves-<timestamp>.zip`) — es el id.
    pub id: String,
    pub created_at: i64,
    pub size_bytes: u64,
}

fn backups_dir(instance: &instance_manager::InstanceData) -> Result<std::path::PathBuf, String> {
    let root = instance.dir();
    worlds::no_link(&root)?;
    let dir = root.join(".tfl_backups");
    if dir
        .try_exists()
        .map_err(|e| format!("No se pudo consultar la carpeta de respaldos: {e}"))?
    {
        worlds::no_link(&dir)?;
    }
    Ok(dir)
}

/// Solo ids con la forma exacta que genera `backup_world_dir` — el id viene
/// del frontend y termina en una ruta de archivo.
fn parse_backup_id(id: &str) -> Option<i64> {
    let raw = id.strip_prefix("saves-")?.strip_suffix(".zip")?;
    let timestamp = if let Some((ts, suffix)) = raw.split_once('-') {
        uuid::Uuid::parse_str(suffix).ok()?;
        ts
    } else {
        raw
    };
    let timestamp = timestamp.parse::<i64>().ok()?;
    (timestamp >= 0).then_some(timestamp)
}

async fn describe(dir: &Path, id: &str) -> Option<WorldBackup> {
    let created_at = parse_backup_id(id)?;
    let path = dir.join(id);
    worlds::no_link(&path).ok()?;
    let meta = tokio::fs::metadata(path).await.ok()?;
    Some(WorldBackup {
        id: id.to_string(),
        created_at,
        size_bytes: meta.len(),
    })
}

#[command]
pub async fn get_world_backups(instance_name: String) -> Result<Vec<WorldBackup>, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let dir = backups_dir(&instance)?;
    let mut entries = match tokio::fs::read_dir(&dir).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("No se pudo leer la carpeta de respaldos: {error}")),
    };
    let mut out = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let Some(name) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        if let Some(b) = describe(&dir, &name).await {
            out.push(b);
        }
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(out)
}

/// `None` si la instancia todavía no tiene mundos (nada que respaldar).
/// Nombres de archivo de las copias, la más reciente primero.
#[command]
pub async fn list_world_backups(instance_name: String) -> Result<Vec<String>, String> {
    world_backup::list_world_backup_names(&instance_name).await
}

#[command]
pub async fn create_world_backup(instance_name: String) -> Result<Option<WorldBackup>, String> {
    create_backup_impl(instance_name, false).await
}
pub async fn create_world_backup_preserving(
    instance_name: String,
) -> Result<Option<WorldBackup>, String> {
    create_backup_impl(instance_name, true).await
}
async fn create_backup_impl(
    instance_name: String,
    preserve: bool,
) -> Result<Option<WorldBackup>, String> {
    let _mutation = content::mutation_lock(&instance_name).await;
    worlds::ensure_stopped(&instance_name)?;
    let instance = instance_manager::get_instance(&instance_name).await?;
    let path = if preserve {
        crate::services::world_backup::backup_world_dir_preserving(&instance.dir()).await?
    } else {
        crate::services::world_backup::backup_world_dir(&instance.dir()).await?
    };
    let Some(path) = path else {
        return Ok(None);
    };
    let id = Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .map(String::from)
        .ok_or("No se pudo leer el nombre de la copia")?;
    Ok(describe(&backups_dir(&instance)?, &id).await)
}

#[command]
pub async fn delete_world_backup(instance_name: String, id: String) -> Result<(), String> {
    parse_backup_id(&id).ok_or("Copia inválida")?;
    let instance = instance_manager::get_instance(&instance_name).await?;
    let path = backups_dir(&instance)?.join(&id);
    worlds::no_link(&path)?;
    tokio::fs::remove_file(path)
        .await
        .map_err(|e| e.to_string())
}

/// Reemplaza `saves/` por el contenido de la copia. Antes guarda una copia
/// de cómo estaban los mundos AHORA, para que restaurar nunca sea un paso
/// sin vuelta atrás. No se puede con el juego abierto: Minecraft tiene los
/// mundos en uso y se corromperían.
#[command]
pub async fn restore_world_backup(instance_name: String, id: String) -> Result<(), String> {
    parse_backup_id(&id).ok_or("Copia inválida")?;
    let _mutation = content::mutation_lock(&instance_name).await;
    worlds::ensure_stopped(&instance_name)?;
    let instance = instance_manager::get_instance(&instance_name).await?;
    let zip_path = backups_dir(&instance)?.join(&id);
    worlds::no_link(&zip_path)?;
    if !zip_path.is_file() {
        return Err("Esa copia ya no existe".into());
    }

    // Preserve the selected archive before creating/pruning the automatic safety copy.
    let selected = backups_dir(&instance)?.join(format!("restore-{}.zip", uuid::Uuid::new_v4()));
    tokio::fs::copy(&zip_path, &selected)
        .await
        .map_err(|_| "Cannot stage selected backup")?;
    let safety = crate::services::world_backup::backup_world_dir(&instance.dir()).await;
    if let Err(e) = safety {
        let _ = tokio::fs::remove_file(&selected).await;
        return Err(e);
    }
    let saves_dir = worlds::worlds_root(&instance.dir(), false)?;
    let source = selected.clone();
    let result = tokio::task::spawn_blocking(move || replace_saves_from_zip(&source, &saves_dir))
        .await
        .map_err(|e| e.to_string())?;
    let _ = tokio::fs::remove_file(&selected).await;
    result
}

/// Fully validate and stage before replacing the saves directory.
fn replace_saves_from_zip(zip_path: &Path, saves_dir: &Path) -> Result<(), String> {
    let _sessions = if saves_dir.exists() {
        worlds::lock_world_sessions(saves_dir)?
    } else {
        Vec::new()
    };
    worlds::restore_directory(zip_path, saves_dir)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_ids_accept_legacy_and_unique_but_not_paths() {
        assert_eq!(parse_backup_id("saves-123.zip"), Some(123));
        assert_eq!(
            parse_backup_id("saves-123-00000000000000000000000000000001.zip"),
            Some(123)
        );
        assert!(parse_backup_id("saves-123-../escape.zip").is_none());
    }
    #[test]
    fn invalid_zip_keeps_existing_worlds() {
        let base = temp_dir("invalid");
        let saves = base.join("saves");
        std::fs::create_dir_all(&saves).unwrap();
        std::fs::write(saves.join("keep.dat"), b"safe").unwrap();
        let zip = base.join("bad.zip");
        std::fs::write(&zip, b"invalid archive").unwrap();
        assert!(replace_saves_from_zip(&zip, &saves).is_err());
        assert_eq!(std::fs::read(saves.join("keep.dat")).unwrap(), b"safe");
        let _ = std::fs::remove_dir_all(base);
    }
    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("tfl-wb-test-{tag}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn backup_id_only_accepts_the_generated_shape() {
        assert_eq!(parse_backup_id("saves-1700000000.zip"), Some(1_700_000_000));
        assert_eq!(parse_backup_id("saves-../../x.zip"), None);
        assert_eq!(parse_backup_id("saves-12.zip.exe"), None);
        assert_eq!(parse_backup_id("other.zip"), None);
        assert_eq!(parse_backup_id("saves-.zip"), None);
    }

    #[test]
    fn restore_roundtrip_replaces_current_saves() {
        let base = temp_dir("roundtrip");
        let saves = base.join("saves");
        std::fs::create_dir_all(saves.join("World1")).unwrap();
        std::fs::write(saves.join("World1").join("level.dat"), b"original").unwrap();
        let zip_path = base.join("saves-1.zip");
        crate::services::zip_util::zip_dir(&saves, &zip_path).unwrap();

        // El estado actual cambia después de la copia: se tiene que pisar.
        std::fs::write(saves.join("World1").join("level.dat"), b"modificado").unwrap();
        std::fs::create_dir_all(saves.join("World2")).unwrap();

        replace_saves_from_zip(&zip_path, &saves).unwrap();
        assert_eq!(
            std::fs::read(saves.join("World1").join("level.dat")).unwrap(),
            b"original"
        );
        assert!(!saves.join("World2").exists());
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn restore_rejects_unsafe_archive_without_restoring_partial_worlds() {
        let base = temp_dir("zipslip");
        let zip_path = base.join("evil.zip");
        {
            let file = std::fs::File::create(&zip_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let opts = zip::write::SimpleFileOptions::default();
            zip.start_file("World1/level.dat", opts).unwrap();
            std::io::Write::write_all(&mut zip, b"ok").unwrap();
            zip.start_file("../escaped.txt", opts).unwrap();
            std::io::Write::write_all(&mut zip, b"malo").unwrap();
            zip.finish().unwrap();
        }
        let saves = base.join("saves");
        assert!(replace_saves_from_zip(&zip_path, &saves).is_err());
        assert!(!saves.exists());
        assert!(!base.join("escaped.txt").exists());
        let _ = std::fs::remove_dir_all(base);
    }
}
