//! Backup de mundos (`saves/`) de una instancia: la copia automática que se
//! hace antes de tocar mods y el listado de copias. Los comandos de crear /
//! restaurar / borrar a mano viven en `commands::world_backups`.

use crate::services::instance_manager;
use crate::services::zip_util::zip_dir;

/// Cuántas copias de mundos se guardan por instancia — cada una es un zip
/// completo de saves/, sin tope se comerían el disco en silencio.
const MAX_WORLD_BACKUPS: usize = 10;

async fn prune_world_backups(dir: &std::path::Path) {
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return;
    };
    let mut names = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Some(name) = entry.file_name().to_str()
            && name.starts_with("saves-")
            && name.ends_with(".zip")
        {
            names.push(name.to_string());
        }
    }
    // El timestamp está en el nombre: ordenar por nombre numérico = por fecha.
    names.sort_by_key(|n| {
        n.trim_start_matches("saves-")
            .trim_end_matches(".zip")
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
    let saves_dir = instance_dir.join("saves");
    if !saves_dir.is_dir() {
        return Ok(None);
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup_path = instance_dir
        .join(".tfl_backups")
        .join(format!("saves-{ts}.zip"));
    let backup_path_clone = backup_path.clone();
    tokio::task::spawn_blocking(move || zip_dir(&saves_dir, &backup_path_clone))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    prune_world_backups(&instance_dir.join(".tfl_backups")).await;
    Ok(Some(backup_path.to_string_lossy().to_string()))
}

/// Nombres de archivo de las copias, la más reciente primero.
pub async fn list_world_backup_names(instance_name: &str) -> Result<Vec<String>, String> {
    let instance = instance_manager::get_instance(instance_name).await?;
    let backups_dir = instance.dir().join(".tfl_backups");
    let Ok(mut entries) = tokio::fs::read_dir(&backups_dir).await else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Some(name) = entry.file_name().to_str() {
            out.push(name.to_string());
        }
    }
    out.sort();
    out.reverse(); // más reciente primero — el timestamp está en el nombre
    Ok(out)
}
