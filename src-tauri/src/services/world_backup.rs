//! Backup de mundos (`saves/`) de una instancia: la copia automática que se
//! hace antes de tocar mods y el listado de copias. Los comandos de crear /
//! restaurar / borrar a mano viven en `commands::world_backups`.

use crate::services::instance_manager;
use crate::services::worlds;

/// Cuántas copias de mundos se guardan por instancia — cada una es un zip
/// completo de saves/, sin tope se comerían el disco en silencio.
const MAX_WORLD_BACKUPS: usize = 10;

fn is_managed_archive(name: &str) -> bool {
    let Some(raw) = name
        .strip_prefix("saves-")
        .and_then(|raw| raw.strip_suffix(".zip"))
    else {
        return false;
    };
    match raw.split_once('-') {
        Some((timestamp, suffix)) => {
            timestamp.parse::<u64>().is_ok() && uuid::Uuid::parse_str(suffix).is_ok()
        }
        None => raw.parse::<u64>().is_ok(),
    }
}

async fn prune_world_backups(dir: &std::path::Path) {
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return;
    };
    let mut names = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Some(name) = entry.file_name().to_str()
            && is_managed_archive(name)
        {
            names.push(name.to_string());
        }
    }
    // El timestamp está en el nombre: ordenar por nombre numérico = por fecha.
    names.sort_by_key(|n| {
        n.trim_start_matches("saves-")
            .trim_end_matches(".zip")
            .split('-')
            .next()
            .unwrap_or("")
            .parse::<u64>()
            .unwrap_or(0)
    });
    while names.len() > MAX_WORLD_BACKUPS {
        let oldest = names.remove(0);
        let _ = tokio::fs::remove_file(dir.join(oldest)).await;
    }
}

/// Backup de saves/ antes de tocar mods — un mod nuevo puede romper un
/// mundo existente (formato de chunk, IDs de bloque distintos entre
/// versiones de un mod, etc). `None` si la instancia no tiene mundos
/// todavía (nada que respaldar, no es un error).
pub(crate) async fn backup_world_dir(
    instance_dir: &std::path::Path,
) -> Result<Option<String>, String> {
    backup_world_dir_policy(instance_dir, true).await
}

/// MCP may create backups but must not delete older backups through automatic retention.
pub(crate) async fn backup_world_dir_preserving(
    instance_dir: &std::path::Path,
) -> Result<Option<String>, String> {
    backup_world_dir_policy(instance_dir, false).await
}
async fn backup_world_dir_policy(
    instance_dir: &std::path::Path,
    prune: bool,
) -> Result<Option<String>, String> {
    static BACKUPS: std::sync::LazyLock<tokio::sync::Mutex<()>> =
        std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));
    let _backup_lock = BACKUPS.lock().await;
    worlds::no_link(instance_dir)?;
    let managed_backups = instance_dir.join(".tfl_backups");
    if managed_backups
        .try_exists()
        .map_err(|e| format!("No se pudo consultar la carpeta de respaldos: {e}"))?
    {
        worlds::no_link(&managed_backups)?;
    }
    if !prune {
        let mut count = 0;
        if let Ok(mut entries) = tokio::fs::read_dir(instance_dir.join(".tfl_backups")).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if is_managed_archive(&name) {
                    count += 1;
                }
            }
        }
        if count >= MAX_WORLD_BACKUPS {
            return Err(
                "BACKUP_RETENTION_LIMIT: delete an exact backup with local approval first".into(),
            );
        }
    }
    let saves_dir = worlds::worlds_root(instance_dir, false)?;
    if !saves_dir
        .try_exists()
        .map_err(|e| format!("No se pudo consultar la carpeta de mundos: {e}"))?
    {
        return Ok(None);
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup_path = instance_dir
        .join(".tfl_backups")
        .join(format!("saves-{ts}-{}.zip", uuid::Uuid::new_v4().simple()));
    let backup_path_clone = backup_path.clone();
    tokio::task::spawn_blocking(move || {
        let _sessions = worlds::lock_world_sessions(&saves_dir)?;
        worlds::archive_directory(&saves_dir, &backup_path_clone)
    })
    .await
    .map_err(|e| e.to_string())??;
    if prune {
        prune_world_backups(&instance_dir.join(".tfl_backups")).await;
    }
    Ok(Some(backup_path.to_string_lossy().to_string()))
}

/// Nombres de archivo de las copias, la más reciente primero.
pub async fn list_world_backup_names(instance_name: &str) -> Result<Vec<String>, String> {
    let instance = instance_manager::get_instance(instance_name).await?;
    let backups_dir = instance.dir().join(".tfl_backups");
    if backups_dir.exists() {
        worlds::no_link(&backups_dir)?;
    }
    let mut entries = match tokio::fs::read_dir(&backups_dir).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("No se pudo leer la carpeta de respaldos: {error}")),
    };
    let mut out = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Some(name) = entry.file_name().to_str() {
            if is_managed_archive(name)
                && entry
                    .file_type()
                    .await
                    .map_err(|e| format!("No se pudo consultar el respaldo: {e}"))?
                    .is_file()
            {
                out.push(name.to_string());
            }
        }
    }
    out.sort();
    out.reverse(); // más reciente primero — el timestamp está en el nombre
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn mcp_backups_never_prune_existing_backups() {
        let root = std::env::temp_dir().join(format!("tfl-backup-cap-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("saves")).unwrap();
        std::fs::write(root.join("saves/level.dat"), b"fixture").unwrap();
        let mut paths = Vec::new();
        for _ in 0..MAX_WORLD_BACKUPS {
            paths.push(backup_world_dir_preserving(&root).await.unwrap().unwrap());
        }
        assert!(backup_world_dir_preserving(&root).await.is_err());
        assert!(paths.iter().all(|p| std::path::Path::new(p).is_file()));
        let _ = std::fs::remove_dir_all(root);
    }
    #[tokio::test]
    async fn repeated_backups_never_overwrite_selected_archive() {
        let root = std::env::temp_dir().join(format!("tfl-backup-{}", uuid::Uuid::new_v4()));
        let world = root.join("saves/World");
        std::fs::create_dir_all(&world).unwrap();
        std::fs::write(world.join("level.dat"), b"first").unwrap();
        let first = backup_world_dir(&root).await.unwrap().unwrap();
        let original = std::fs::read(&first).unwrap();
        std::fs::write(world.join("level.dat"), b"second").unwrap();
        let second = backup_world_dir(&root).await.unwrap().unwrap();
        assert_ne!(first, second);
        assert_eq!(std::fs::read(first).unwrap(), original);
        let _ = std::fs::remove_dir_all(root);
    }
}
