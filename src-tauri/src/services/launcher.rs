use crate::core::PathManager;
use crate::services::instance_manager::LoaderKind;
use crate::services::{SettingsManager, instance_manager, java_manager, progress};
use aqua::{DownloadManager, FabricBatch, ForgeBatch, NeoForgeBatch, QuiltBatch};
use launchwerk::Launchwerk;
use launchwerk::auth::AccountType;
use launchwerk::models::VersionManifest;
use std::path::Path;
use std::sync::LazyLock;
use tracing::{error, info};

static LAUNCHWERK: LazyLock<Launchwerk> =
    LazyLock::new(|| Launchwerk::new(PathManager::get().get_shared_dir().to_path_buf()));

fn version_json_exists(shared_dir: &Path, version_id: &str) -> bool {
    shared_dir
        .join("versions")
        .join(version_id)
        .join(format!("{version_id}.json"))
        .exists()
}

/// El `.json` de versión se escribe temprano (en `prepare()`), antes de que
/// se bajen los archivos de verdad — si la descarga se corta a mitad de
/// camino (un corte de red, como el visto con "error decoding response
/// body"), el `.json` queda ahí solo, y como `ensure_downloaded()` sólo
/// miraba ese archivo, todo lanzamiento posterior asumía "ya está
/// instalado" y jamás reintentaba: quedaba trabado para siempre con el jar
/// faltante ("Version JAR not found") hasta borrar el caché a mano. Este
/// chequeo verifica también el jar del cliente, así que una descarga
/// incompleta se detecta y se reintenta sola en el próximo "Jugar".
fn vanilla_fully_installed(shared_dir: &Path, version_id: &str) -> bool {
    let dir = shared_dir.join("versions").join(version_id);
    dir.join(format!("{version_id}.json")).exists() && dir.join(format!("{version_id}.jar")).exists()
}

/// Descarga lo que haga falta para poder lanzar `data`: siempre la base
/// Vanilla, y además el loader elegido (Fabric/Forge/NeoForge/Quilt) si
/// corresponde. Reporta progreso real vía `AppEvent::DownloadProgress`.
async fn ensure_downloaded(
    data: &instance_manager::InstanceData,
    java_path_hint: Option<std::path::PathBuf>,
) -> Result<(), String> {
    let shared_dir = PathManager::get().get_shared_dir().to_path_buf();

    if !vanilla_fully_installed(&shared_dir, &data.mc_version) {
        info!("Descargando base Vanilla {}", data.mc_version);
        let manager = DownloadManager::new(shared_dir.clone());
        let handle = manager.prepare(&data.mc_version).await.map_err(|e| {
            error!("No se pudo preparar la descarga de {}: {e}", data.mc_version);
            e.to_string()
        })?;
        let (tx, watcher) = progress::watch(&format!("minecraft:{}", data.mc_version));
        let result = handle.download_all(Some(tx)).await.map_err(|e| e.to_string());
        watcher.finish(result).await.inspect_err(|e| {
            error!("Falló la descarga de Vanilla {}: {e}", data.mc_version);
        })?;
        info!("Vanilla {} listo", data.mc_version);
    }

    if version_json_exists(&shared_dir, &data.launch_version_id) {
        return Ok(());
    }

    let loader_version = data
        .loader_version
        .as_deref()
        .ok_or("Falta la versión del loader guardada en la instancia")?;

    info!(
        "Instalando {:?} {} para Minecraft {}",
        data.loader, loader_version, data.mc_version
    );

    match data.loader {
        LoaderKind::Vanilla => {}
        LoaderKind::Fabric => {
            let batch = FabricBatch::new(&shared_dir, &data.mc_version, loader_version)
                .await
                .map_err(|e| {
                    error!("No se pudo resolver el perfil de Fabric: {e}");
                    e.to_string()
                })?;
            let manager = DownloadManager::new(shared_dir.clone());
            let handle = manager
                .prepare_batch(Box::new(batch))
                .await
                .map_err(|e| e.to_string())?;
            let (tx, watcher) = progress::watch(&format!("fabric:{}", data.mc_version));
            let result = handle.download_all(Some(tx)).await.map_err(|e| e.to_string());
            watcher
                .finish(result)
                .await
                .inspect_err(|e| error!("Falló la instalación de Fabric: {e}"))?;
        }
        LoaderKind::Quilt => {
            let batch = QuiltBatch::new(&shared_dir, &data.mc_version, loader_version)
                .await
                .map_err(|e| {
                    error!("No se pudo resolver el perfil de Quilt: {e}");
                    e.to_string()
                })?;
            let manager = DownloadManager::new(shared_dir.clone());
            let handle = manager
                .prepare_batch(Box::new(batch))
                .await
                .map_err(|e| e.to_string())?;
            let (tx, watcher) = progress::watch(&format!("quilt:{}", data.mc_version));
            let result = handle.download_all(Some(tx)).await.map_err(|e| e.to_string());
            watcher
                .finish(result)
                .await
                .inspect_err(|e| error!("Falló la instalación de Quilt: {e}"))?;
        }
        LoaderKind::Forge => {
            // ForgeBatch::install necesita Java (corre el instalador con
            // procesadores post-descarga) — se resuelve antes de entrar acá.
            let java_path = java_path_hint.ok_or("Forge necesita Java resuelto de antemano")?;
            let (tx, watcher) = progress::watch(&format!("forge:{}", data.mc_version));
            progress::mark_processing(&tx, "Instalando Forge (esto puede tardar)…");
            let result = ForgeBatch::install(
                &shared_dir,
                &data.mc_version,
                loader_version,
                Some(java_path),
            )
            .await
            .map(|_manifest| ())
            .map_err(|e| e.to_string());
            watcher
                .finish(result)
                .await
                .inspect_err(|e| error!("Falló la instalación de Forge: {e}"))?;
        }
        LoaderKind::NeoForge => {
            let java_path = java_path_hint.ok_or("NeoForge necesita Java resuelto de antemano")?;
            let (tx, watcher) = progress::watch(&format!("neoforge:{}", data.mc_version));
            progress::mark_processing(&tx, "Instalando NeoForge (esto puede tardar)…");
            let result =
                NeoForgeBatch::install(&shared_dir, &data.mc_version, loader_version, Some(java_path))
                    .await
                    .map(|_manifest| ())
                    .map_err(|e| e.to_string());
            watcher
                .finish(result)
                .await
                .inspect_err(|e| error!("Falló la instalación de NeoForge: {e}"))?;
        }
    }

    info!(
        "{:?} {} instalado para Minecraft {}",
        data.loader, loader_version, data.mc_version
    );
    Ok(())
}

pub async fn launch(instance_name: String) -> Result<(), String> {
    info!("Lanzando instancia \"{instance_name}\"");
    let data = instance_manager::get_instance(&instance_name).await?;
    let shared_dir = PathManager::get().get_shared_dir().to_path_buf();

    // Java se resuelve una vez acá: lo necesitan tanto el lanzamiento final
    // como (para Forge/NeoForge) el propio instalador del loader.
    let java_major = aqua::infer_java_version(&data.mc_version);
    let java_path = java_manager::ensure_java(java_major).await.inspect_err(|e| {
        error!("No se pudo resolver Java {java_major} para \"{instance_name}\": {e}");
    })?;
    info!("Java {java_major}: {}", java_path.display());

    ensure_downloaded(&data, Some(java_path.clone())).await?;

    let manifest_path = shared_dir
        .join("versions")
        .join(&data.launch_version_id)
        .join(format!("{}.json", data.launch_version_id));
    let manifest = VersionManifest::from_file(&manifest_path).map_err(|e| {
        error!(
            "No se pudo leer el manifest en {}: {e}",
            manifest_path.display()
        );
        format!("Manifest inválido: {e}")
    })?;

    let mut user = { SettingsManager::read().get_user() };
    if let Err(e) = user.load_tokens() {
        tracing::warn!("No se pudieron cargar los tokens del usuario: {e:?}");
    }

    let (min_mem, max_mem) = {
        let settings = SettingsManager::read();
        (
            format!("{}M", data.min_memory.unwrap_or(settings.min_memory)),
            format!("{}M", data.max_memory.unwrap_or(settings.max_memory)),
        )
    };

    let mut builder = launchwerk::LaunchConfig::builder()
        .java_path(java_path)
        .username(user.username.clone())
        .ram(min_mem, max_mem)
        .cracked(user.user_type == AccountType::Cracked);

    builder = match user.user_type {
        AccountType::Microsoft | AccountType::Yggdrasil => builder
            .access_token(user.access_token.clone())
            .auth_uuid(user.uuid.clone())
            .user_type(if user.user_type == AccountType::Microsoft {
                "msa"
            } else {
                "mojang"
            }),
        AccountType::Cracked => builder,
    };

    let config = builder.build();
    let instance_dir = data.dir();
    tokio::fs::create_dir_all(&instance_dir)
        .await
        .map_err(|e| e.to_string())?;

    let handle = LAUNCHWERK.prepare(manifest, config, instance_dir);
    handle.launch().await.map_err(|e| {
        error!("No se pudo lanzar \"{instance_name}\": {e}");
        e.to_string()
    })?;
    info!("\"{instance_name}\" lanzada correctamente");
    instance_manager::mark_last_played(&instance_name).await?;
    Ok(())
}
