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
}

pub fn read_lockfile(dir: &Path) -> Result<SelectionLockfile, String> {
    let p = dir.join(LOCKFILE_NAME);
    let bytes = std::fs::read(&p).map_err(|e| format!("No se pudo leer {}: {e}", p.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("Lockfile corrupto en {}: {e}", p.display()))
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
    let mods_dir = instance_dir.join("mods");
    if !mods_dir.exists() {
        return Ok(Vec::new());
    }

    let quarantine_dir = instance_dir.join("quarantine");
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
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_file()
            && let Some(ext) = path.extension().and_then(|e| e.to_str())
            && (ext.eq_ignore_ascii_case("jar") || ext.eq_ignore_ascii_case("disabled"))
        {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !allowed_filenames.contains(&file_name.to_ascii_lowercase()) {
                let _ = std::fs::create_dir_all(&quarantine_dir);
                let dest = quarantine_dir.join(&file_name);
                if let Ok(()) = std::fs::rename(&path, &dest) {
                    warn!(
                        "Mod externo detectado en instancia TFL Selection: {}. Movido a quarantine/ para proteger la integridad.",
                        file_name
                    );
                    quarantined.push(file_name);
                }
            }
        }
    }

    Ok(quarantined)
}

/// Verifica la integridad de los archivos administrados y reporta discrepancias.
pub fn verify_integrity(instance_dir: &Path) -> Result<IntegrityReport, String> {
    let lock = read_lockfile(instance_dir)?;
    let mut missing = Vec::new();
    let corrupted = Vec::new();

    for rel_path in &lock.managed_files {
        let full_path = instance_dir.join(rel_path);
        if !full_path.exists() {
            missing.push(rel_path.clone());
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
/// 3. Garantiza Java 25.
pub async fn prepare_for_launch(instance_data: &InstanceData) -> Result<(), String> {
    let dir = instance_data.dir();
    let lock = read_lockfile(&dir)?;

    // Cuarentena de cualquier archivo agregado manualmente
    let _ = quarantine_foreign_mods(&dir, &lock.managed_files);

    // Verificación estricta de que todos los mods oficiales existen
    for rel_path in &lock.managed_files {
        let full = dir.join(rel_path);
        if !full.exists() {
            return Err(format!(
                "Archivo oficial faltante: {rel_path}. Realiza una reparación en TFL Selection antes de iniciar."
            ));
        }
    }

    // Java 25 garantizado
    let _ = java_manager::ensure_java(25, &instance_data.mc_version).await?;

    Ok(())
}

/// Instalación completa y atómica de una versión de TFL Selection. Envuelve
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
                current_item: Some(file_clone.path.clone()),
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
        let dest = instance_dir.join(&f.path);
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
    let _ = quarantine_foreign_mods(&instance_dir, &managed_list);

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

    // Guardar / Actualizar instancia
    let loader_ver = manifest
        .minecraft
        .loader_version
        .as_deref()
        .unwrap_or("0.20.1");
    let launch_id = format!(
        "fabric-loader-{}-{}-ornithe-gen2",
        loader_ver, manifest.minecraft.version
    );

    let instance = InstanceData {
        uuid: uuid::Uuid::new_v4().to_string(),
        name: instance_name.to_string(),
        mc_version: manifest.minecraft.version.clone(),
        loader: LoaderKind::OrnitheGen2,
        loader_version: Some(loader_ver.to_string()),
        launch_version_id: launch_id,
        last_played: 0,
        play_time_secs: 0,
        min_memory: Some(2048),
        max_memory: Some(4096),
        server_type: None,
        server_jar: None,
        server_build: None,
    };

    instance.save().await.map_err(|e| e.to_string())?;

    info!(
        "TFL Selection {} instalada exitosamente como \"{}\"",
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
