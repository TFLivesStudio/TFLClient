use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, warn};

use super::catalog::{self, CatalogStore, Channel};
use super::manifest::{self, Expected, Manifest, ManifestFile};
use super::net::{GithubTransport, Transport};
use super::signatures::TRUSTED_KEYS;
use super::util::{atomic_write, sha256_file};
use crate::core::PathManager;
use crate::core::event_bus::{AppEvent, emit};
use crate::services::instance_manager::{InstanceData, LoaderKind};
use crate::services::java_manager;

pub const LOCKFILE_NAME: &str = "selection.lock.json";
const CONCURRENT_DOWNLOADS: usize = 4;
#[cfg(test)]
#[path = "installer_tests.rs"]
mod presentation_tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionLockfile {
    pub schema_version: u32,
    pub pack_id: String,
    pub target_id: String,
    pub channel: String,
    pub release: String,
    pub manifest_sha256: String,
    pub installed_at: String,
    pub managed_files: Vec<String>,
    pub seeded_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub is_valid: bool,
    pub missing: Vec<String>,
    pub corrupted: Vec<String>,
    pub foreign_mods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionUpdateInfo {
    pub current_release: String,
    pub latest_release: String,
    pub channel: String,
    pub target_id: String,
}

pub fn selection_cache_dir() -> PathBuf {
    PathManager::get().get_shared_dir().join("selection_cache")
}

pub fn is_selection_instance(dir: &Path) -> bool {
    dir.join(LOCKFILE_NAME).exists()
        || std::fs::read(dir.join("instance.tfl.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .is_some_and(|data| {
                data.get("selection")
                    .is_some_and(serde_json::Value::is_object)
            })
}

/// Public presentation metadata from local installation provenance. This is not an authenticity
/// certificate: verify_integrity still checks the pinned signed manifest and file hashes.
pub fn identity_from_lock(
    dir: &Path,
) -> Option<crate::services::instance_manager::SelectionIdentity> {
    let lock = read_lockfile(dir).ok()?;
    if lock.schema_version != 1
        || lock.pack_id != catalog::PACK_ID
        || !catalog::valid_target_id(&lock.target_id)
        || !catalog::valid_release(&lock.release)
        || Channel::parse(&lock.channel).is_none()
        || !catalog::is_sha256_hex(&lock.manifest_sha256)
    {
        return None;
    }
    Some(crate::services::instance_manager::SelectionIdentity {
        shader_support: legacy_shader_support(&lock)
            || std::fs::read(dir.join("instance.tfl.json"))
                .ok()
                .and_then(|raw| serde_json::from_slice::<serde_json::Value>(&raw).ok())
                .is_some_and(|v| {
                    v["selection"]["manifest_sha256"].as_str()
                        == Some(lock.manifest_sha256.as_str())
                        && v["selection"]["shader_support"].as_bool() == Some(true)
                }),
        target_id: lock.target_id,
        channel: lock.channel,
        release: lock.release,
        manifest_sha256: lock.manifest_sha256,
        installed_at: lock.installed_at,
    })
}

fn legacy_shader_support(lock: &SelectionLockfile) -> bool {
    // This exact signed legacy release contains the documented Lucerna shader engine.
    lock.manifest_sha256 == super::util::sha256_hex(include_bytes!("testdata/manifest-1.8.9.json"))
}

pub async fn required_java(dir: &Path) -> Result<u8, String> {
    u8::try_from(pinned_manifest(dir).await?.minecraft.java_major)
        .map_err(|_| "Unsupported Selection Java version".into())
}

pub fn read_lockfile(dir: &Path) -> Result<SelectionLockfile, String> {
    let p = dir.join(LOCKFILE_NAME);
    if std::fs::metadata(&p).map_err(|e| e.to_string())?.len() > 2 * 1024 * 1024 {
        return Err("Selection lockfile exceeds size limit".into());
    }
    let bytes = std::fs::read(&p).map_err(|e| format!("No se pudo leer {}: {e}", p.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("Lockfile corrupto en {}: {e}", p.display()))
}

/// Immutable manifest metadata is reverified on every use, including offline cache hits.
async fn pinned_manifest(dir: &Path) -> Result<Manifest, String> {
    let lock = read_lockfile(dir)?;
    if identity_from_lock(dir).is_none() {
        return Err("Invalid Selection installation identity".into());
    }
    let raw_instance = std::fs::read(dir.join("instance.tfl.json")).map_err(|e| e.to_string())?;
    let instance: InstanceData =
        serde_json::from_slice(&raw_instance).map_err(|e| e.to_string())?;
    let target = catalog::Target {
        target_id: lock.target_id.clone(),
        minecraft_version: instance.mc_version,
        loader: instance.loader.as_str().into(),
        channels: Default::default(),
        display_name: None,
        summary: None,
        sort_order: None,
    };
    let pointer = catalog::ChannelPointer {
        release: lock.release.clone(),
        tag: catalog::expected_tag(&lock.target_id, &lock.release),
        manifest_sha256: lock.manifest_sha256.clone(),
    };
    let expected = Expected {
        target: &target,
        pointer: &pointer,
    };
    let cache = selection_cache_dir();
    let file = cache.join(format!("manifest-{}.json", lock.manifest_sha256));
    let signature = cache.join(format!("manifest-{}.sig", lock.manifest_sha256));
    if std::fs::metadata(&file).is_ok_and(|m| m.len() <= manifest::MAX_MANIFEST_BYTES as u64)
        && std::fs::metadata(&signature).is_ok_and(|m| m.len() <= 1024)
        && let (Ok(raw), Ok(sig)) = (std::fs::read(&file), std::fs::read(&signature))
    {
        if let Ok(m) = manifest::load_manifest(
            &raw,
            &sig,
            &expected,
            env!("CARGO_PKG_VERSION"),
            TRUSTED_KEYS,
        ) {
            return Ok(m);
        }
    }
    // Current legacy installations predate metadata caching. Preserve their offline launch
    // with the exact published signed bytes; never substitute a different hash or release.
    if let Ok(m) = manifest::load_manifest(
        include_bytes!("testdata/manifest-1.8.9.json"),
        include_bytes!("testdata/manifest-1.8.9.json.sig"),
        &expected,
        env!("CARGO_PKG_VERSION"),
        TRUSTED_KEYS,
    ) {
        return Ok(m);
    }
    let transport = GithubTransport;
    let raw = transport
        .get(
            &manifest::manifest_url(catalog::DISTRIBUTION_REPOSITORY, &pointer.tag),
            manifest::MAX_MANIFEST_BYTES,
            None,
        )
        .await
        .map_err(|e| e.to_string())?;
    let sig = transport
        .get(
            &manifest::manifest_sig_url(catalog::DISTRIBUTION_REPOSITORY, &pointer.tag),
            1024,
            None,
        )
        .await
        .map_err(|e| e.to_string())?;
    let m = manifest::load_manifest(
        &raw.body,
        &sig.body,
        &expected,
        env!("CARGO_PKG_VERSION"),
        TRUSTED_KEYS,
    )
    .map_err(|e| e.to_string())?;
    atomic_write(&file, &raw.body).map_err(|e| e.to_string())?;
    atomic_write(&signature, &sig.body).map_err(|e| e.to_string())?;
    Ok(m)
}

fn managed_path(dir: &Path, relative: &str) -> Result<PathBuf, String> {
    if !manifest::safe_relative_path(relative) {
        return Err("Invalid managed path".into());
    }
    let mut path = dir.to_path_buf();
    for part in relative.split('/') {
        path.push(part);
        if std::fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err("Selection managed paths cannot contain symlinks".into());
        }
    }
    Ok(path)
}

/// Icons can only come from signed immutable releases, never arbitrary instance paths/URLs.
pub async fn official_icon(dir: &Path) -> Option<PathBuf> {
    let manifest = pinned_manifest(dir).await.ok()?;
    let icon = manifest.icon_asset.as_ref()?;
    let file = selection_cache_dir().join(&icon.asset_name);
    if let Ok((hash, bytes)) = sha256_file(&file) {
        if hash == icon.sha256 && bytes == icon.bytes {
            return present_icon(&file, dir);
        }
    }
    let tag = manifest.distribution.as_ref()?.tag.as_str();
    let url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        catalog::DISTRIBUTION_REPOSITORY,
        tag,
        icon.asset_name
    );
    let bytes = GithubTransport
        .get(&url, icon.bytes as usize, None)
        .await
        .ok()?
        .body;
    if bytes.len() as u64 != icon.bytes
        || super::util::sha256_hex(&bytes) != icon.sha256
        || !valid_icon_png(&bytes)
    {
        return None;
    }
    atomic_write(&file, &bytes).ok()?;
    present_icon(&file, dir)
}

fn present_icon(cache: &Path, dir: &Path) -> Option<PathBuf> {
    let bytes = std::fs::read(cache).ok()?;
    if bytes.len() > 1024 * 1024 || !valid_icon_png(&bytes) {
        return None;
    }
    // Reuse the existing instance icon asset scope; do not broaden Tauri file permissions.
    let target = dir.join("icon.png");
    if std::fs::read(&target).ok().as_deref() != Some(bytes.as_slice()) {
        atomic_write(&target, &bytes).ok()?;
    }
    Some(target)
}

pub fn ensure_personal_path(dir: &Path, relative: &str) -> Result<(), String> {
    if !is_selection_instance(dir) {
        return Ok(());
    }
    if !manifest::safe_relative_path(relative) {
        return Err("Invalid Selection personal path".into());
    }
    let first = relative
        .split(['/', '\\'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if [
        "mods",
        "selection.lock.json",
        "instance.tfl.json",
        ".tfl_modpacks",
    ]
    .contains(&first.as_str())
        || first.is_empty()
    {
        return Err(
            "TFL Selection administra estos archivos. Usa reparar o actualizar el cliente.".into(),
        );
    }
    Ok(())
}

fn valid_icon_png(bytes: &[u8]) -> bool {
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" {
        return false;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    (1..=512).contains(&width) && (1..=512).contains(&height)
}

pub fn write_lockfile(dir: &Path, lock: &SelectionLockfile) -> Result<(), String> {
    let p = dir.join(LOCKFILE_NAME);
    let json = serde_json::to_string_pretty(lock)
        .map_err(|e| format!("Error serializando lockfile: {e}"))?;
    atomic_write(&p, json.as_bytes()).map_err(|e| format!("Error guardando lockfile: {e}"))
}

/// Descarga o recupera de la caché el archivo correspondiente a `f`,
/// garantizando que coincide exactamente en tamaño y SHA-256.
pub async fn ensure_cached_file(
    manifest: &Manifest,
    f: &ManifestFile,
    transport: &dyn Transport,
) -> Result<PathBuf, String> {
    let cache_dir = selection_cache_dir();
    tokio::fs::create_dir_all(&cache_dir)
        .await
        .map_err(|e| e.to_string())?;

    let blob_path = cache_dir.join(format!("sha256-{}", f.sha256));

    // Si ya existe en caché, verificamos tamaño y hash
    if blob_path.exists() {
        if let Ok(meta) = tokio::fs::metadata(&blob_path).await {
            if meta.len() == f.bytes {
                if let Ok((hash, _)) = sha256_file(&blob_path) {
                    if hash == f.sha256 {
                        return Ok(blob_path);
                    }
                }
            }
        }
        // Si el archivo en caché estaba corrupto o incompleto, lo eliminamos
        let _ = tokio::fs::remove_file(&blob_path).await;
    }

    let url = manifest::file_download_url(manifest, f)
        .map_err(|e| format!("URL inválida para {}: {:?}", f.id, e))?;

    let temp_name = format!("tmp-{}-{}", f.sha256, uuid::Uuid::new_v4());
    let temp_path = cache_dir.join(&temp_name);

    let res = transport
        .download(&url, &temp_path, 0, f.bytes, &|_| {})
        .await
        .map_err(|e| format!("Error descargando {}: {:?}", f.id, e))?;

    if res.total_len != f.bytes {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(format!(
            "Tamaño incorrecto para {}: esperado {}, recibido {}",
            f.id, f.bytes, res.total_len
        ));
    }

    let (digest, _) = sha256_file(&temp_path)
        .map_err(|e| format!("Error calculando SHA-256 para {}: {e}", f.id))?;

    if digest != f.sha256 {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(format!(
            "SHA-256 corrupto para {}: esperado {}, calculado {}",
            f.id, f.sha256, digest
        ));
    }

    // Atomic move a su lugar definitivo por hash
    tokio::fs::rename(&temp_path, &blob_path)
        .await
        .map_err(|e| format!("Error persistiendo blob en caché: {e}"))?;

    Ok(blob_path)
}

/// Mueve cualquier mod ajeno a la selección hacia `quarantine/`.
pub fn quarantine_foreign_mods(
    instance_dir: &Path,
    managed_rel_paths: &[String],
) -> Result<Vec<String>, String> {
    let mods_dir = managed_path(instance_dir, "mods")?;
    if !mods_dir.exists() {
        return Ok(Vec::new());
    }

    let quarantine_dir = managed_path(instance_dir, "quarantine")?;
    let mut quarantined = Vec::new();

    let allowed_filenames: std::collections::HashSet<String> = managed_rel_paths
        .iter()
        .filter_map(|p| {
            Path::new(p)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_ascii_lowercase())
        })
        .collect();

    let read_dir = std::fs::read_dir(&mods_dir).map_err(|e| e.to_string())?;
    for entry in read_dir {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if entry.file_type().map_err(|e| e.to_string())?.is_symlink() {
            return Err("Selection mods cannot contain symlinks".into());
        }
        if path.is_file()
            && let Some(ext) = path.extension().and_then(|e| e.to_str())
            && (ext.eq_ignore_ascii_case("jar") || ext.eq_ignore_ascii_case("disabled"))
        {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !allowed_filenames.contains(&file_name.to_ascii_lowercase()) {
                std::fs::create_dir_all(&quarantine_dir).map_err(|e| e.to_string())?;
                // Preserve any previously quarantined copy instead of replacing it.
                let dest = quarantine_dir.join(format!("{}-{file_name}", uuid::Uuid::new_v4()));
                std::fs::rename(&path, &dest).map_err(|e| e.to_string())?;
                warn!(
                    "Mod externo detectado en instancia TFLClient: {}. Movido a quarantine/ para proteger la integridad.",
                    file_name
                );
                quarantined.push(file_name);
            }
        }
    }

    Ok(quarantined)
}

/// Verifica la integridad de los archivos administrados y reporta discrepancias.
pub async fn verify_integrity(instance_dir: &Path) -> Result<IntegrityReport, String> {
    let manifest = pinned_manifest(instance_dir).await?;
    let mut missing = Vec::new();
    let mut corrupted = Vec::new();

    for file in manifest
        .files_for(manifest::current_platform(), manifest::current_arch())
        .filter(|f| f.is_managed())
    {
        let full_path = managed_path(instance_dir, &file.path)?;
        match sha256_file(&full_path) {
            Ok((hash, bytes)) if hash == file.sha256 && bytes == file.bytes => {}
            Ok(_) => corrupted.push(file.path.clone()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => missing.push(file.path.clone()),
            Err(e) => return Err(e.to_string()),
        }
    }

    let quarantine_dir = instance_dir.join("quarantine");
    let mut foreign_mods = Vec::new();
    if quarantine_dir.exists()
        && let Ok(entries) = std::fs::read_dir(&quarantine_dir)
    {
        for entry in entries.flatten() {
            foreign_mods.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    let is_valid = missing.is_empty() && corrupted.is_empty();
    Ok(IntegrityReport {
        is_valid,
        missing,
        corrupted,
        foreign_mods,
    })
}

/// Prepara la instancia antes de lanzar el juego:
/// 1. Cuarentena de mods foráneos.
/// 2. Verificación de integridad.
/// 3. Garantiza el Java declarado por el manifiesto firmado.
pub async fn prepare_for_launch(instance_data: &InstanceData) -> Result<(), String> {
    let dir = instance_data.dir();
    let manifest = pinned_manifest(&dir).await?;
    let managed: Vec<String> = manifest
        .files_for(manifest::current_platform(), manifest::current_arch())
        .filter(|f| f.is_managed())
        .map(|f| f.path.clone())
        .collect();

    // Cuarentena de cualquier archivo agregado manualmente
    quarantine_foreign_mods(&dir, &managed)?;

    // Verificación estricta de que todos los mods oficiales existen
    if !verify_integrity(&dir).await?.is_valid {
        return Err("El cliente necesita reparación en TFL Selection antes de iniciar.".into());
    }

    // Java declarado por el cliente oficial
    let major = u8::try_from(manifest.minecraft.java_major)
        .map_err(|_| "Unsupported Selection Java version")?;
    let _ = java_manager::ensure_java(major, &instance_data.mc_version).await?;

    Ok(())
}

/// Instalación completa y atómica de una versión de TFLClient de la biblioteca TFL Selection. Envuelve
/// `install_target_inner` para garantizar que el task de progreso
/// `selection:{target_id}` siempre cierra con `DownloadFinished` o
/// `DownloadFailed` — `install_target_inner` emite `DownloadProgress` pero
/// nunca su evento terminal, así que sin este wrapper el toast de descarga
/// quedaba en la UI para siempre aunque la instalación ya hubiera
/// terminado (bien o mal).
pub async fn install_target(
    instance_name: &str,
    target_id: &str,
    channel: Channel,
) -> Result<InstanceData, String> {
    let task = format!("selection:{target_id}");
    let _guard = crate::services::content::mutation_lock(instance_name).await;
    if crate::services::launcher::running_instances()
        .iter()
        .any(|(name, _)| name == instance_name)
    {
        let error = "Cierra Minecraft antes de actualizar o reparar este cliente.".to_string();
        emit(AppEvent::DownloadFailed {
            task,
            error: error.clone(),
        });
        return Err(error);
    }
    let result = install_target_inner(instance_name, target_id, channel).await;
    match &result {
        Ok(_) => emit(AppEvent::DownloadFinished { task }),
        Err(e) => emit(AppEvent::DownloadFailed {
            task,
            error: e.clone(),
        }),
    }
    result
}

async fn install_target_inner(
    instance_name: &str,
    target_id: &str,
    channel: Channel,
) -> Result<InstanceData, String> {
    crate::services::instance_manager::valid_instance_name(instance_name)?;
    let existing_dir = PathManager::get().get_instance_dir().join(instance_name);
    if existing_dir.exists() && !is_selection_instance(&existing_dir) {
        return Err("Elige otro nombre: esa instancia contiene una instalación personal.".into());
    }
    if let Some(identity) = identity_from_lock(&existing_dir) {
        if identity.target_id != target_id {
            return Err("Esa instancia pertenece a otro cliente de TFL Selection.".into());
        }
    }
    let transport = GithubTransport;
    let store = CatalogStore::new(
        PathManager::get()
            .get_shared_dir()
            .join("selection_catalog"),
    );
    let snapshot = catalog::refresh(&transport, &store, false)
        .await
        .map_err(|e| format!("No se pudo obtener el catálogo oficial: {e}"))?;
    let catalog = snapshot.catalog;

    let target = catalog
        .targets
        .iter()
        .find(|t| t.target_id == target_id)
        .ok_or_else(|| format!("Target \"{target_id}\" no encontrado en el catálogo"))?;

    let pointer = target
        .channels
        .get(channel)
        .ok_or_else(|| format!("Canal {:?} no publicado para {}", channel, target_id))?;

    // Descarga y verificación del manifiesto firmado
    let dist_repo = catalog::DISTRIBUTION_REPOSITORY;
    let m_url = manifest::manifest_url(dist_repo, &pointer.tag);
    let m_sig_url = manifest::manifest_sig_url(dist_repo, &pointer.tag);

    let m_bytes = transport
        .get(&m_url, manifest::MAX_MANIFEST_BYTES, None)
        .await
        .map_err(|e| format!("Error descargando manifiesto: {e}"))?;

    let m_sig_bytes = transport
        .get(&m_sig_url, 1024, None)
        .await
        .map_err(|e| format!("Error descargando firma del manifiesto: {e}"))?;

    let expected = Expected { target, pointer };
    let manifest = manifest::load_manifest(
        &m_bytes.body,
        &m_sig_bytes.body,
        &expected,
        env!("CARGO_PKG_VERSION"),
        TRUSTED_KEYS,
    )
    .map_err(|e| format!("Manifiesto no válido: {e:?}"))?;

    // Instancia en disco
    let instance_dir = PathManager::get().get_instance_dir().join(instance_name);
    tokio::fs::create_dir_all(&instance_dir)
        .await
        .map_err(|e| e.to_string())?;

    let platform = manifest::current_platform();
    let arch = manifest::current_arch();
    let applicable_files: Vec<&ManifestFile> = manifest.files_for(platform, arch).collect();

    // Descarga concurrente de todos los archivos al blob cache
    let sem = Arc::new(Semaphore::new(CONCURRENT_DOWNLOADS));
    let mut tasks = Vec::new();

    let total_items = applicable_files.len();
    let total_bytes: u64 = applicable_files.iter().map(|f| f.bytes).sum();
    let downloaded_items = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let downloaded_bytes = Arc::new(std::sync::atomic::AtomicU64::new(0));

    emit(AppEvent::DownloadProgress {
        task: format!("selection:{}", target.target_id),
        stage: "Descargando archivos oficiales".into(),
        item_current: 0,
        item_total: total_items,
        bytes_current: 0,
        bytes_total: total_bytes,
        current_item: None,
    });

    for f in applicable_files {
        let sem_clone = Arc::clone(&sem);
        let m_clone = manifest.clone();
        let file_clone = f.clone();
        let d_items = Arc::clone(&downloaded_items);
        let d_bytes = Arc::clone(&downloaded_bytes);
        let target_id_str = target.target_id.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            let res = ensure_cached_file(&m_clone, &file_clone, &GithubTransport).await;

            let cur_items = d_items.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            let cur_bytes = d_bytes
                .fetch_add(file_clone.bytes, std::sync::atomic::Ordering::Relaxed)
                + file_clone.bytes;

            emit(AppEvent::DownloadProgress {
                task: format!("selection:{}", target_id_str),
                stage: "Descargando archivos oficiales".into(),
                item_current: cur_items,
                item_total: total_items,
                bytes_current: cur_bytes,
                bytes_total: total_bytes,
                current_item: None,
            });

            res.map(|path| (file_clone, path))
        }));
    }

    let mut staged_files = Vec::new();
    for t in tasks {
        match t.await {
            Ok(Ok((file, blob_path))) => staged_files.push((file, blob_path)),
            Ok(Err(err)) => return Err(err),
            Err(e) => return Err(format!("Fallo en tarea de descarga: {e}")),
        }
    }

    // Instalación transaccional en la instancia
    let mut managed_list = Vec::new();
    let mut seeded_list = Vec::new();

    for (f, blob_path) in staged_files {
        let dest = managed_path(&instance_dir, &f.path)?;
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }

        if f.is_managed() {
            tokio::fs::copy(&blob_path, &dest)
                .await
                .map_err(|e| format!("Error copiando mod {} a la instancia: {e}", f.path))?;
            managed_list.push(f.path.clone());
        } else if f.install_policy == "seed-if-absent" {
            if !dest.exists() {
                tokio::fs::copy(&blob_path, &dest)
                    .await
                    .map_err(|e| format!("Error creando config {} en la instancia: {e}", f.path))?;
            }
            seeded_list.push(f.path.clone());
        }
    }

    // Cuarentena de archivos extraños
    quarantine_foreign_mods(&instance_dir, &managed_list)?;

    // Escribir lockfile
    let lock = SelectionLockfile {
        schema_version: 1,
        pack_id: manifest.pack_id.clone(),
        target_id: manifest.target_id.clone(),
        channel: channel.as_str().to_string(),
        release: manifest.release.clone(),
        manifest_sha256: pointer.manifest_sha256.clone(),
        installed_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string(),
        managed_files: managed_list,
        seeded_files: seeded_list,
    };
    write_lockfile(&instance_dir, &lock)?;
    atomic_write(
        &selection_cache_dir().join(format!("manifest-{}.json", lock.manifest_sha256)),
        &m_bytes.body,
    )
    .map_err(|e| e.to_string())?;
    atomic_write(
        &selection_cache_dir().join(format!("manifest-{}.sig", lock.manifest_sha256)),
        &m_sig_bytes.body,
    )
    .map_err(|e| e.to_string())?;

    // Guardar / Actualizar instancia
    let loader_ver = manifest
        .minecraft
        .loader_version
        .as_deref()
        .unwrap_or("0.20.1");
    let loader =
        LoaderKind::parse(&manifest.minecraft.loader).ok_or("Unsupported Selection loader")?;
    let launch_id = match loader {
        LoaderKind::OrnitheGen2 => format!(
            "fabric-loader-{}-{}-ornithe-gen2",
            loader_ver, manifest.minecraft.version
        ),
        LoaderKind::Fabric => format!(
            "fabric-loader-{}-{}",
            loader_ver, manifest.minecraft.version
        ),
        _ => return Err("Unsupported Selection bootstrap".into()),
    };
    let previous = crate::services::instance_manager::get_instance(instance_name)
        .await
        .ok();

    let mut identity = identity_from_lock(&instance_dir);
    if let Some(identity) = identity.as_mut() {
        identity.shader_support = manifest
            .capabilities
            .as_ref()
            .map_or_else(|| legacy_shader_support(&lock), |c| c.shaders);
    }
    let instance = InstanceData {
        uuid: previous
            .as_ref()
            .map(|i| i.uuid.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        name: instance_name.to_string(),
        mc_version: manifest.minecraft.version.clone(),
        loader,
        loader_version: Some(loader_ver.to_string()),
        launch_version_id: launch_id,
        last_played: previous.as_ref().map_or(0, |i| i.last_played),
        play_time_secs: previous.as_ref().map_or(0, |i| i.play_time_secs),
        min_memory: previous.as_ref().and_then(|i| i.min_memory).or(Some(2048)),
        max_memory: previous.as_ref().and_then(|i| i.max_memory).or(Some(4096)),
        server_type: None,
        server_jar: None,
        server_build: None,
        selection: identity,
    };

    instance.save().await.map_err(|e| e.to_string())?;

    info!(
        "TFLClient {} instalado exitosamente como \"{}\"",
        manifest.release, instance_name
    );
    Ok(instance)
}

/// Repara la instancia reinstalando los archivos oficiales faltantes o corruptos.
pub async fn repair_selection(instance_name: &str) -> Result<(), String> {
    let instance_dir = PathManager::get().get_instance_dir().join(instance_name);
    let lock = read_lockfile(&instance_dir)?;
    let channel = Channel::parse(&lock.channel).unwrap_or(Channel::Beta);
    install_target(instance_name, &lock.target_id, channel)
        .await
        .map(|_| ())
}

/// Comprueba si existe una actualización disponible en el catálogo para esta instancia.
pub async fn check_update(instance_name: &str) -> Result<Option<SelectionUpdateInfo>, String> {
    let instance_dir = PathManager::get().get_instance_dir().join(instance_name);
    let lock = read_lockfile(&instance_dir)?;
    let channel = Channel::parse(&lock.channel).unwrap_or(Channel::Beta);

    let transport = GithubTransport;
    let store = CatalogStore::new(
        PathManager::get()
            .get_shared_dir()
            .join("selection_catalog"),
    );
    let snapshot = catalog::refresh(&transport, &store, false)
        .await
        .map_err(|e| format!("Error consultando catálogo: {e}"))?;
    let catalog = snapshot.catalog;

    let target = catalog
        .targets
        .iter()
        .find(|t| t.target_id == lock.target_id)
        .ok_or_else(|| "Target no encontrado en el catálogo".to_string())?;

    if let Some(pointer) = target.channels.get(channel)
        && (pointer.release != lock.release || pointer.manifest_sha256 != lock.manifest_sha256)
    {
        return Ok(Some(SelectionUpdateInfo {
            current_release: lock.release,
            latest_release: pointer.release.clone(),
            channel: channel.as_str().to_string(),
            target_id: target.target_id.clone(),
        }));
    }

    Ok(None)
}
