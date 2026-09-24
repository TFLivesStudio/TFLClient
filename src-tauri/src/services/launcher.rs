use crate::core::PathManager;
use crate::core::event_bus::{AppEvent, emit};
use crate::services::instance_manager::LoaderKind;
use crate::services::{SettingsManager, instance_manager, java_manager, progress};
use aqua::{DownloadManager, FabricBatch, ForgeBatch, NeoForgeBatch, QuiltBatch};
use launchwerk::Launchwerk;
use launchwerk::auth::AccountType;
use launchwerk::models::VersionManifest;
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use tracing::{error, info};

static LAUNCHWERK: LazyLock<Launchwerk> =
    LazyLock::new(|| Launchwerk::new(PathManager::get().get_shared_dir().to_path_buf()));

/// Instancia de Minecraft actualmente en ejecución (a lo sumo una — el
/// launcher no permite abrir dos al mismo tiempo). `id` queda en `None`
/// mientras se reserva el lugar (justo antes de empezar a lanzar) y se
/// completa una vez que el proceso realmente arrancó, para poder matarlo
/// después por su uuid en `launchwerk`.
struct RunningGame {
    name: String,
    id: Option<uuid::Uuid>,
    pid: Option<u32>,
}

static RUNNING: LazyLock<Mutex<Option<RunningGame>>> = LazyLock::new(|| Mutex::new(None));

/// Nombre de la instancia corriendo ahora mismo, si hay alguna.
pub fn running_instance_name() -> Option<String> {
    RUNNING.lock().unwrap().as_ref().map(|r| r.name.clone())
}

/// PID del proceso de Java corriendo ahora mismo, si hay alguna instancia
/// activa y ya arrancó de verdad (puede ser `None` un instante mientras se
/// resuelve, aunque `running_instance_name` ya no sea `None`).
pub fn running_instance_pid() -> Option<u32> {
    RUNNING.lock().unwrap().as_ref().and_then(|r| r.pid)
}

/// Mata el proceso de la instancia en ejecución (si hay alguna). La
/// limpieza real del estado (`RUNNING`, `LAUNCHWERK.remove`, evento
/// `InstanceExited`) la hace la misma tarea de fondo que espera la salida
/// normal del proceso — matar solo dispara esa salida.
pub async fn stop_running() -> Result<(), String> {
    let id = RUNNING.lock().unwrap().as_ref().and_then(|r| r.id);
    let Some(id) = id else {
        return Err("No hay ninguna instancia corriendo".into());
    };
    let Some(handle) = LAUNCHWERK.get(id) else {
        return Err("La instancia ya no está activa".into());
    };
    handle.kill().await.map_err(|e| e.to_string())
}

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

/// Punto de entrada público: reserva el "lugar" de instancia corriendo
/// (evita que se lancen dos al mismo tiempo) y delega en `launch_inner`.
/// Si algo falla antes de que el proceso llegue a arrancar, libera el
/// lugar reservado — si el lanzamiento tiene éxito, lo libera la tarea de
/// fondo que espera a que el proceso termine (ver el final de
/// `launch_inner`).
pub async fn launch(app: tauri::AppHandle, instance_name: String) -> Result<(), String> {
    {
        let mut guard = RUNNING.lock().unwrap();
        if let Some(running) = guard.as_ref() {
            return Err(format!(
                "Ya hay una instancia corriendo: \"{}\". Cerrala antes de iniciar otra.",
                running.name
            ));
        }
        *guard = Some(RunningGame {
            name: instance_name.clone(),
            id: None,
            pid: None,
        });
    }

    let result = launch_inner(app, instance_name).await;
    if result.is_err() {
        *RUNNING.lock().unwrap() = None;
    }
    result
}

/// Copia el jar de TFL MP Guard a `instance_dir/mods` si existe uno para
/// esta versión de Minecraft — enforcement real (no solo el aviso de
/// texto) para cuentas cracked en instancias con loader. Falla soft: si
/// no hay jar para esta combinación (versión no soportada todavía), no
/// bloquea el lanzamiento, solo loguea — la restricción JVM de siempre
/// (auth hosts falsos) sigue aplicando igual.
async fn inject_mp_guard(app: &tauri::AppHandle, mc_version: &str, instance_dir: &std::path::Path) {
    let Ok(base) = crate::core::bundled_resource_dir(app, "mp-guard") else {
        tracing::debug!("mp-guard: recurso empaquetado no encontrado, se omite");
        return;
    };
    let jar = base.join(mc_version).join("tfl-mp-guard.jar");
    if !jar.exists() {
        tracing::info!("mp-guard: sin jar para Minecraft {mc_version}, se omite (queda solo el bloqueo JVM)");
        return;
    }
    let mods_dir = instance_dir.join("mods");
    if let Err(e) = tokio::fs::create_dir_all(&mods_dir).await {
        error!("mp-guard: no se pudo crear la carpeta mods: {e}");
        return;
    }
    if let Err(e) = tokio::fs::copy(&jar, mods_dir.join("tfl-mp-guard.jar")).await {
        error!("mp-guard: no se pudo copiar el jar: {e}");
    } else {
        info!("mp-guard: jar instalado para Minecraft {mc_version}");
    }
}

async fn launch_inner(app: tauri::AppHandle, instance_name: String) -> Result<(), String> {
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

    // Si la instancia no tiene override explícito de RAM, antes se caía
    // directo al default global fijo (1024/2048 MB) sin importar cuántos
    // mods tenga — un modpack grande (80+ mods, frameworks pesados como
    // Silk/Kotlin) se queda corto ahí y termina en OOM silencioso, sin
    // ningún error visible más que el proceso terminando de golpe. Se usa
    // el recomendado según cantidad de mods como piso, sin bajar nunca por
    // debajo de lo que el usuario ya configuró a mano en Ajustes.
    let (global_min, global_max) = {
        let settings = SettingsManager::read();
        (settings.min_memory, settings.max_memory)
    };
    let (min_mem, max_mem) = if data.min_memory.is_none() || data.max_memory.is_none() {
        let recommended =
            crate::commands::settings::get_recommended_ram_for_instance(instance_name.to_string())
                .await;
        (
            data.min_memory
                .unwrap_or_else(|| recommended.recommended_min_mb.max(global_min)),
            data.max_memory
                .unwrap_or_else(|| recommended.recommended_max_mb.max(global_max)),
        )
    } else {
        (data.min_memory.unwrap(), data.max_memory.unwrap())
    };
    let (min_mem, max_mem) = (format!("{min_mem}M"), format!("{max_mem}M"));

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
        // El flag lo lee TFL MP Guard (mod inyectado abajo) para saber si
        // tiene que bloquear multijugador — sin este flag el mod no hace
        // nada, ni siquiera si el jar está instalado.
        AccountType::Cracked => builder.extra_jvm_args(vec!["-Dtflclient.cracked=true".into()]),
    };

    let config = builder.build();
    let instance_dir = data.dir();
    tokio::fs::create_dir_all(&instance_dir)
        .await
        .map_err(|e| e.to_string())?;

    if user.user_type == AccountType::Cracked && data.loader != LoaderKind::Vanilla {
        inject_mp_guard(&app, &data.mc_version, &instance_dir).await;
    }

    let handle = LAUNCHWERK.prepare(manifest, config, instance_dir);
    handle.launch().await.map_err(|e| {
        error!("No se pudo lanzar \"{instance_name}\": {e}");
        e.to_string()
    })?;
    info!("\"{instance_name}\" lanzada correctamente");
    instance_manager::mark_last_played(&instance_name).await?;

    let id = handle.id();
    let pid = handle.pid().await;
    if let Some(running) = RUNNING.lock().unwrap().as_mut() {
        running.id = Some(id);
        running.pid = pid;
    }
    emit(AppEvent::InstanceStatusChanged {
        name: instance_name.clone(),
        status: "running".into(),
    });

    // Reenvía stdout/stderr del proceso al frontend en tiempo real (para la
    // ventana emergente de log) — cada línea que ya viene de `launchwerk`
    // por sus canales broadcast se re-emite como AppEvent, que Tauri manda
    // a TODAS las ventanas abiertas (incluida la de log).
    let mut out_rx = handle.subscribe_stdout();
    let name_out = instance_name.clone();
    tokio::spawn(async move {
        while let Ok(line) = out_rx.recv().await {
            emit(AppEvent::InstanceLogLine {
                instance: name_out.clone(),
                stream: "stdout".into(),
                line,
            });
        }
    });

    let mut err_rx = handle.subscribe_stderr();
    let name_err = instance_name.clone();
    tokio::spawn(async move {
        while let Ok(line) = err_rx.recv().await {
            emit(AppEvent::InstanceLogLine {
                instance: name_err.clone(),
                stream: "stderr".into(),
                line,
            });
        }
    });

    // Dueña final del handle: espera a que el proceso termine (solo o por
    // stop_running()) y recién ahí libera el "lugar" de instancia corriendo
    // — así una segunda instancia solo puede lanzarse una vez que esta de
    // verdad cerró, no apenas se disparó el proceso.
    tokio::spawn(async move {
        let code = handle.wait().await;
        LAUNCHWERK.remove(id);
        *RUNNING.lock().unwrap() = None;
        emit(AppEvent::InstanceExited {
            instance: instance_name.clone(),
            code,
        });
        emit(AppEvent::InstanceStatusChanged {
            name: instance_name,
            status: "stopped".into(),
        });
    });

    Ok(())
}
