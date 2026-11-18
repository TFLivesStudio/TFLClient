use std::collections::HashSet;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use super::pack_format::{MrpackMetadata, PackFormat};
use crate::utils::path::safe_join;

#[derive(Debug, thiserror::Error)]
pub enum MrpackError {
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Download error: {0}")]
    Download(String),
    #[error("Invalid mrpack: {0}")]
    Invalid(String),
    #[error("Installation was cancelled")]
    Cancelled,
}

const MAX_ENTRIES: usize = 50_000;
const MAX_EXPANDED: u64 = 32 * 1024 * 1024 * 1024;
const MAX_INDEX: u64 = 16 * 1024 * 1024;
const MAX_OVERRIDE: u64 = 128 * 1024 * 1024;

fn invalid(message: &str) -> MrpackError {
    MrpackError::Invalid(message.to_string())
}

fn portable_path(raw: &str) -> bool {
    !raw.is_empty()
        && raw.len() <= 4096
        && raw.split('/').all(|part| {
            !part.is_empty()
                && part.len() <= 240
                && !part.ends_with(['.', ' '])
                && !part
                    .chars()
                    .any(|c| c.is_control() || "\\:*?\"<>|".contains(c))
                && !matches!(
                    part.split('.')
                        .next()
                        .unwrap_or("")
                        .to_ascii_uppercase()
                        .as_str(),
                    "CON"
                        | "PRN"
                        | "AUX"
                        | "NUL"
                        | "COM1"
                        | "COM2"
                        | "COM3"
                        | "COM4"
                        | "COM5"
                        | "COM6"
                        | "COM7"
                        | "COM8"
                        | "COM9"
                        | "LPT1"
                        | "LPT2"
                        | "LPT3"
                        | "LPT4"
                        | "LPT5"
                        | "LPT6"
                        | "LPT7"
                        | "LPT8"
                        | "LPT9"
                )
        })
}

fn pack_destination(raw: &str) -> bool {
    portable_path(raw)
        && raw.split('/').all(|part| {
            let part = part.to_ascii_lowercase();
            !part.starts_with(".tfl")
                && !matches!(
                    part.as_str(),
                    "instance.tfl.json"
                        | "accounts.json"
                        | "launcher_accounts.json"
                        | "launcher_accounts_microsoft_store.json"
                        | "auth.json"
                        | "credentials.json"
                        | "msa-cache.json"
                        | ".env"
                        | "session.lock"
                )
        })
}

fn trusted_download(raw: &str) -> bool {
    let Some((host, path)) = raw
        .strip_prefix("https://")
        .and_then(|url| url.split_once('/'))
    else {
        return false;
    };
    matches!(
        host,
        "cdn.modrinth.com" | "github.com" | "raw.githubusercontent.com" | "gitlab.com"
    ) && !path.is_empty()
        && !raw
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
}

fn read_pack(archive: &mut zip::ZipArchive<std::fs::File>) -> Result<PackFormat, MrpackError> {
    if archive.len() > MAX_ENTRIES {
        return Err(invalid("El archivo .mrpack supera el límite de entradas."));
    }
    let mut expanded = 0u64;
    let mut archive_paths = HashSet::new();
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        let name = entry.name()?;
        let name = name.strip_suffix('/').unwrap_or(&name);
        if !portable_path(name) || !archive_paths.insert(name.to_lowercase()) {
            return Err(invalid("El .mrpack contiene rutas inválidas o duplicadas."));
        }
        if ["overrides/", "client-overrides/"].iter().any(|prefix| {
            name.strip_prefix(*prefix)
                .is_some_and(|relative| !pack_destination(relative))
        }) {
            return Err(invalid(
                "El .mrpack intenta copiar archivos de cuentas o del launcher.",
            ));
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| !matches!(mode & 0o170000, 0 | 0o100000 | 0o040000))
        {
            return Err(invalid("El .mrpack contiene un enlace o archivo especial."));
        }
        if entry.size() > MAX_OVERRIDE {
            return Err(invalid(
                "Un archivo del .mrpack supera el límite de tamaño.",
            ));
        }
        expanded = expanded
            .checked_add(entry.size())
            .filter(|n| *n <= MAX_EXPANDED)
            .ok_or_else(|| invalid("El .mrpack supera el límite de contenido descomprimido."))?;
    }
    let manifest_idx = archive
        .file_names()
        .position(|name| matches!(name.as_deref(), Ok("modrinth.index.json")))
        .ok_or_else(|| {
            MrpackError::Invalid("No modrinth.index.json found in mrpack".to_string())
        })?;

    let mut content = String::new();
    archive
        .by_index(manifest_idx)?
        .take(MAX_INDEX + 1)
        .read_to_string(&mut content)?;
    if content.len() as u64 > MAX_INDEX {
        return Err(invalid("El manifiesto .mrpack supera el límite de tamaño."));
    }

    let pack: PackFormat = serde_json::from_str(&content)?;

    if pack.game != "minecraft" {
        return Err(MrpackError::Invalid(format!(
            "Pack is for '{}', not 'minecraft'",
            pack.game
        )));
    }

    if pack.format_version != 1 || pack.files.len() + archive.len() > MAX_ENTRIES {
        return Err(invalid(
            "La versión o el número de archivos de este .mrpack no son compatibles.",
        ));
    }
    let mut download_paths = HashSet::new();
    for file in &pack.files {
        let valid_hash = |kind: &str, length: usize| {
            file.hashes.get(kind).is_some_and(|hash| {
                hash.len() == length && hash.bytes().all(|c| c.is_ascii_hexdigit())
            })
        };
        if !pack_destination(&file.path) || !download_paths.insert(file.path.to_lowercase()) {
            return Err(invalid(
                "El manifiesto incluye rutas inválidas, reservadas o duplicadas.",
            ));
        }
        if !file.downloads.iter().any(|url| trusted_download(url))
            || !valid_hash("sha1", 40)
            || !valid_hash("sha512", 128)
        {
            return Err(invalid(
                "Una descarga no incluye una URL HTTPS admitida y hashes válidos.",
            ));
        }
        expanded = expanded
            .checked_add(u64::from(file.file_size))
            .filter(|n| *n <= MAX_EXPANDED)
            .ok_or_else(|| invalid("El .mrpack supera el límite total de contenido."))?;
    }
    Ok(pack)
}

pub fn parse_mrpack(path: &Path) -> Result<MrpackMetadata, MrpackError> {
    let mut archive = zip::ZipArchive::new(std::fs::File::open(path)?)?;
    Ok(read_pack(&mut archive)?.extract_metadata())
}

pub async fn install_mrpack(
    path: &Path,
    instance_dir: &Path,
    shared_dir: &Path,
    progress: Option<aqua::progress::ProgressSender>,
) -> Result<MrpackMetadata, MrpackError> {
    install_mrpack_cancellable(path, instance_dir, shared_dir, progress, None).await
}

fn check_cancel(cancel: Option<&AtomicBool>) -> Result<(), MrpackError> {
    if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
        return Err(MrpackError::Cancelled);
    }
    Ok(())
}

pub async fn install_mrpack_cancellable(
    path: &Path,
    instance_dir: &Path,
    shared_dir: &Path,
    progress: Option<aqua::progress::ProgressSender>,
    cancel: Option<&AtomicBool>,
) -> Result<MrpackMetadata, MrpackError> {
    check_cancel(cancel)?;
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let pack = read_pack(&mut archive)?;

    let mut metadata = pack.extract_metadata();

    let applicable_files: Vec<&super::pack_format::PackFile> = pack
        .files
        .iter()
        .filter(|f| {
            f.env
                .as_ref()
                .and_then(|env| env.get("client"))
                .is_none_or(|v| v != "unsupported")
        })
        .collect();

    let items = applicable_files
        .iter()
        .map(|file| {
            let url = file
                .downloads
                .iter()
                .find(|url| trusted_download(url))
                .ok_or_else(|| invalid("No hay una URL admitida para uno de los archivos."))?;
            let destination = safe_join(instance_dir, &file.path).map_err(MrpackError::Invalid)?;
            let hash = file
                .hashes
                .get("sha1")
                .ok_or_else(|| invalid("Falta el hash SHA-1."))?;
            Ok(
                aqua::DownloadItemSpec::new(url.clone(), destination, &file.path)
                    .with_hash(hash.to_ascii_lowercase())
                    .with_size(u64::from(file.file_size)),
            )
        })
        .collect::<Result<Vec<_>, MrpackError>>()?;

    if !items.is_empty() {
        let batch = aqua::GenericBatch::new(format!("mrpack-{}", metadata.version_id), items);

        let dm = aqua::DownloadManager::new(shared_dir.to_path_buf());
        let handle = dm
            .prepare_batch(Box::new(batch))
            .await
            .map_err(|e| MrpackError::Download(e.to_string()))?;

        let download = handle.download_all(progress);
        tokio::pin!(download);
        loop {
            tokio::select! {
                result = &mut download => {
                    check_cancel(cancel)?;
                    result.map_err(|e| MrpackError::Download(e.to_string()))?;
                    break;
                }
                _ = tokio::time::sleep(Duration::from_millis(150)), if cancel.is_some() => {
                    if check_cancel(cancel).is_err() {
                        handle.cancel();
                        let _ = download.await;
                        return Err(MrpackError::Cancelled);
                    }
                }
            }
        }
    }

    metadata.installed_paths = applicable_files.iter().map(|f| f.path.clone()).collect();
    metadata
        .installed_paths
        .extend(extract_overrides(&mut archive, instance_dir, cancel).await?);
    check_cancel(cancel)?;
    extract_icon(&mut archive, instance_dir).await?;

    Ok(metadata)
}

async fn extract_overrides(
    archive: &mut zip::ZipArchive<std::fs::File>,
    instance_dir: &Path,
    cancel: Option<&AtomicBool>,
) -> Result<Vec<String>, MrpackError> {
    let mut written = Vec::new();
    for i in 0..archive.len() {
        check_cancel(cancel)?;
        let entry = archive.by_index(i)?;
        let entry_name = entry.name()?.to_string();
        let is_dir = entry.is_dir();

        // Reject paths that escape the archive root (e.g. `../` or absolute paths).
        let Some(enclosed) = entry.enclosed_name().and_then(|p| {
            if p.is_absolute() {
                None
            } else {
                Some(p.to_path_buf())
            }
        }) else {
            tracing::warn!("Mrpack override with unsafe path ignored: {}", entry_name);
            drop(entry);
            continue;
        };
        drop(entry);

        if is_dir {
            continue;
        }

        let relative_path = if let Ok(stripped) = enclosed.strip_prefix("overrides/") {
            stripped
        } else if let Ok(stripped) = enclosed.strip_prefix("client-overrides/") {
            stripped
        } else {
            continue;
        };

        if relative_path.as_os_str().is_empty() {
            continue;
        }

        let dest = safe_join(instance_dir, relative_path.to_string_lossy().as_ref())
            .map_err(MrpackError::Invalid)?;

        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tracing::info!("Extracting override {} -> {:?}", entry_name, dest);

        let mut buffer = Vec::new();
        archive.by_index(i)?.read_to_end(&mut buffer)?;
        tokio::fs::write(&dest, &buffer).await?;
        written.push(relative_path.to_string_lossy().replace('\\', "/"));
    }
    Ok(written)
}

async fn extract_icon(
    archive: &mut zip::ZipArchive<std::fs::File>,
    instance_dir: &Path,
) -> Result<(), MrpackError> {
    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let is_dir = entry.is_dir();
        let name = entry.name()?.to_string();
        drop(entry);

        if is_dir || name != "icon.png" {
            continue;
        }

        let icon_dest = instance_dir.join("icon.png");
        let mut buffer = Vec::new();
        archive.by_index(i)?.read_to_end(&mut buffer)?;
        tokio::fs::write(&icon_dest, &buffer).await?;
        break;
    }
    Ok(())
}
