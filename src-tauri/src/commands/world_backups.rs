//! Copias de seguridad de los mundos (`saves/`) de una instancia: crear a
//! mano, ver la lista, restaurar y borrar. Las copias automáticas que ya se
//! hacían antes de actualizar mods (`mods::backup_world_dir`) caen en la
//! misma carpeta, así que aparecen acá también.

use crate::services::{instance_manager, launcher};
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

fn backups_dir(instance: &instance_manager::InstanceData) -> std::path::PathBuf {
    instance.dir().join(".tfl_backups")
}

/// Solo ids con la forma exacta que genera `backup_world_dir` — el id viene
/// del frontend y termina en una ruta de archivo.
fn parse_backup_id(id: &str) -> Option<i64> {
    id.strip_prefix("saves-")?
        .strip_suffix(".zip")?
        .parse::<i64>()
        .ok()
}

async fn describe(dir: &Path, id: &str) -> Option<WorldBackup> {
    let created_at = parse_backup_id(id)?;
    let meta = tokio::fs::metadata(dir.join(id)).await.ok()?;
    Some(WorldBackup {
        id: id.to_string(),
        created_at,
        size_bytes: meta.len(),
    })
}

#[command]
pub async fn get_world_backups(instance_name: String) -> Result<Vec<WorldBackup>, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let dir = backups_dir(&instance);
    let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
        return Ok(Vec::new());
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
#[command]
pub async fn create_world_backup(instance_name: String) -> Result<Option<WorldBackup>, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let Some(path) = crate::commands::mods::backup_world_dir(&instance.dir()).await? else {
        return Ok(None);
    };
    let id = Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .map(String::from)
        .ok_or("No se pudo leer el nombre de la copia")?;
    Ok(describe(&backups_dir(&instance), &id).await)
}

#[command]
pub async fn delete_world_backup(instance_name: String, id: String) -> Result<(), String> {
    parse_backup_id(&id).ok_or("Copia inválida")?;
    let instance = instance_manager::get_instance(&instance_name).await?;
    tokio::fs::remove_file(backups_dir(&instance).join(&id))
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
    if launcher::running_instances()
        .iter()
        .any(|(name, _)| name == &instance_name)
    {
        return Err("Cerrá el juego antes de restaurar una copia de los mundos".into());
    }
    let instance = instance_manager::get_instance(&instance_name).await?;
    let zip_path = backups_dir(&instance).join(&id);
    if !zip_path.is_file() {
        return Err("Esa copia ya no existe".into());
    }

    crate::commands::mods::backup_world_dir(&instance.dir()).await?;

    let saves_dir = instance.dir().join("saves");
    tokio::task::spawn_blocking(move || replace_saves_from_zip(&zip_path, &saves_dir))
        .await
        .map_err(|e| e.to_string())?
}

/// Vacía `saves_dir` y extrae el zip adentro. Descarta cualquier entrada cuya
/// ruta se escape de la carpeta (`..`, rutas absolutas): el zip es un archivo
/// del disco del usuario, pero no por eso se confía en sus rutas.
fn replace_saves_from_zip(zip_path: &Path, saves_dir: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    if saves_dir.exists() {
        std::fs::remove_dir_all(saves_dir).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(saves_dir).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let Some(rel) = entry.enclosed_name() else {
            continue;
        };
        let out_path = saves_dir.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut out = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        crate::commands::mods::zip_dir(&saves, &zip_path).unwrap();

        // El estado actual cambia después de la copia: se tiene que pisar.
        std::fs::write(saves.join("World1").join("level.dat"), b"modificado").unwrap();
        std::fs::create_dir_all(saves.join("World2")).unwrap();

        replace_saves_from_zip(&zip_path, &saves).unwrap();
        assert_eq!(std::fs::read(saves.join("World1").join("level.dat")).unwrap(), b"original");
        assert!(!saves.join("World2").exists());
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn restore_ignores_entries_that_escape_the_folder() {
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
        replace_saves_from_zip(&zip_path, &saves).unwrap();
        assert!(saves.join("World1").join("level.dat").exists());
        assert!(!base.join("escaped.txt").exists());
        let _ = std::fs::remove_dir_all(base);
    }
}
