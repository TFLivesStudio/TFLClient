//! Reversible retirement of a byte-identical resource previously installed by TFL.
//! Unknown files, even with the historical filename, are never moved or removed.
use sha2::{Digest, Sha256};
use std::path::Path;

const RETIRED_HASH: &str = "104214dc662691ae7516debbccb9d24974378a8bc24f3fb35c775ca1492bb270";

pub async fn quarantine_retired_mod(game_dir: &Path) -> Result<(), String> {
    let mods = game_dir.join("mods");
    match tokio::fs::symlink_metadata(&mods).await {
        Ok(meta) if meta.is_dir() && !redirected(&meta) => {}
        Ok(_) => return Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("No se pudo revisar la carpeta mods: {e}")),
    }
    let candidate = mods.join("tfl-mp-guard.jar");
    let metadata = match tokio::fs::symlink_metadata(&candidate).await {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("No se pudo revisar el recurso retirado: {e}")),
    };
    if !metadata.is_file() || redirected(&metadata) || metadata.len() > 1_048_576 {
        return Ok(());
    }
    let bytes = tokio::fs::read(&candidate)
        .await
        .map_err(|e| e.to_string())?;
    if hex::encode(Sha256::digest(&bytes)) != RETIRED_HASH {
        return Ok(());
    }
    let destination = game_dir
        .join(".tfl-retired")
        .join(format!("{RETIRED_HASH}.jar"));
    let parent = destination
        .parent()
        .ok_or("Destino de migración inválido")?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|e| e.to_string())?;
    // Never follow a user-created redirection outside the game directory.
    if redirected(
        &tokio::fs::symlink_metadata(parent)
            .await
            .map_err(|e| e.to_string())?,
    ) {
        return Err("La carpeta de recursos retirados es un enlace; revisa la instancia".into());
    }
    if destination.exists() {
        return Err("Ya existe un respaldo del recurso retirado; revisa .tfl-retired".into());
    }
    tokio::fs::rename(candidate, destination)
        .await
        .map_err(|e| format!("No se pudo respaldar el recurso retirado: {e}"))
}

fn redirected(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    metadata.file_type().is_symlink()
}
