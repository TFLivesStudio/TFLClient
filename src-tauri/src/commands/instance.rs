use crate::services::instance_manager::{InstanceData, LoaderKind};
use crate::services::{instance_manager, launcher};
use aqua::{FabricBatch, QuiltBatch};
use base64::Engine;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, command};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

/// Label fijo: el launcher solo permite una instancia corriendo a la vez
/// (ver `launcher::launch`), así que alcanza con una sola ventana de log.
const LOG_WINDOW_LABEL: &str = "log-instance";

const ICON_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

/// Carpeta empaquetada con `bundle.resources` en tauri.conf.json — íconos
/// de instancia CC0 (Simplexity-Development/Entity-Icons +
/// hube12/mc_icons) para elegir al azar al crear una instancia y después
/// poder cambiar por otro del mismo set, sin depender de red.
fn icon_presets_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    crate::core::bundled_resource_dir(app, "instance-icons")
}

async fn resolve_loader(
    mc_version: &str,
    loader: LoaderKind,
) -> Result<(Option<String>, String), String> {
    match loader {
        LoaderKind::Vanilla => Ok((None, mc_version.to_string())),
        LoaderKind::Fabric => {
            let lv = FabricBatch::resolve_latest_loader(mc_version)
                .await
                .map_err(|e| e.to_string())?;
            let id = format!("fabric-loader-{lv}-{mc_version}");
            Ok((Some(lv), id))
        }
        LoaderKind::Quilt => {
            let lv = QuiltBatch::resolve_latest_loader(mc_version)
                .await
                .map_err(|e| e.to_string())?;
            let id = format!("quilt-loader-{lv}-{mc_version}");
            Ok((Some(lv), id))
        }
        LoaderKind::Forge => {
            let lv = crate::commands::loaders::get_forge_version(mc_version.to_string()).await?;
            let id = format!("{mc_version}-forge-{lv}");
            Ok((Some(lv), id))
        }
        LoaderKind::NeoForge => {
            let lv =
                crate::commands::loaders::get_neoforge_version(mc_version.to_string()).await?;
            let id = format!("{mc_version}-neoforge-{lv}");
            Ok((Some(lv), id))
        }
    }
}

#[command]
pub async fn create_instance(
    app: AppHandle,
    name: String,
    mc_version: String,
    loader: String,
) -> Result<InstanceData, String> {
    let loader_kind =
        LoaderKind::parse(&loader).ok_or_else(|| format!("Loader desconocido: {loader}"))?;
    let (loader_version, launch_version_id) = resolve_loader(&mc_version, loader_kind).await?;
    let data = instance_manager::create_instance(
        name,
        mc_version,
        loader_kind,
        loader_version,
        launch_version_id,
    )
    .await?;

    // Ícono al azar del set vendoreado — el usuario lo puede cambiar
    // después por otro del mismo set o subir el suyo (ver
    // set_instance_icon_from_preset / set_instance_icon). Si falla (SO
    // sin el recurso bundleado, por ejemplo en un dev build roto), no
    // bloquea la creación de la instancia — se queda con el avatar de
    // letra por defecto.
    if let Ok(dir) = icon_presets_dir(&app) {
        if let Ok(mut entries) = tokio::fs::read_dir(&dir).await {
            let mut files = Vec::new();
            while let Ok(Some(entry)) = entries.next_entry().await {
                if entry.path().extension().and_then(|e| e.to_str()) == Some("png") {
                    files.push(entry.path());
                }
            }
            if !files.is_empty() {
                let idx = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
                    % files.len() as u128) as usize;
                let chosen = &files[idx];
                let dest = data.dir().join("icon.png");
                let _ = tokio::fs::create_dir_all(data.dir()).await;
                let _ = tokio::fs::copy(chosen, &dest).await;
            }
        }
    }

    Ok(data)
}

#[command]
pub async fn get_instances() -> Vec<InstanceData> {
    instance_manager::list_instances().await
}

#[command]
pub async fn delete_instance(name: String) -> Result<(), String> {
    instance_manager::delete_instance(&name).await
}

#[command]
pub async fn rename_instance(old_name: String, new_name: String) -> Result<InstanceData, String> {
    instance_manager::rename_instance(&old_name, new_name).await
}

#[command]
pub async fn duplicate_instance(name: String) -> Result<InstanceData, String> {
    instance_manager::duplicate_instance(&name).await
}

#[command]
pub async fn update_instance_memory(
    name: String,
    min_memory: Option<u32>,
    max_memory: Option<u32>,
) -> Result<InstanceData, String> {
    instance_manager::update_instance_memory(&name, min_memory, max_memory).await
}

#[command]
pub async fn launch(app: AppHandle, instance_name: String) -> Result<(), String> {
    // launcher::launch() rechaza si ya hay otra instancia corriendo — se
    // valida ahí antes, así que acá nunca se abre una ventana de log
    // huérfana por un lanzamiento que ni siquiera arrancó.
    launcher::launch(app.clone(), instance_name.clone()).await?;
    open_log_window(&app, &instance_name)
}

/// Abre (o enfoca, si ya está abierta) la ventana emergente con el log en
/// vivo del proceso de Minecraft. Ventana con chrome nativo del SO (a
/// diferencia de la principal) — no necesita TitleBar propia.
///
/// El nombre de instancia se pasa por `initialization_script` (una
/// variable global inyectada ANTES de que cargue cualquier script de la
/// página), no por query string en la URL — `WebviewUrl::App` toma el
/// string entero como un path de archivo literal, no lo parsea como
/// URL+query, así que "index.html?window=log&instance=X" buscaba un
/// archivo con ese nombre exacto y tiraba 404 (visto en Windows, pero el
/// bug no es específico de esa plataforma).
fn open_log_window(app: &AppHandle, instance_name: &str) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window(LOG_WINDOW_LABEL) {
        let _ = existing.set_focus();
        return Ok(());
    }

    let instance_json = serde_json::to_string(instance_name).map_err(|e| e.to_string())?;
    let init_script = format!("window.__TFL_LOG_INSTANCE__ = {instance_json};");

    WebviewWindowBuilder::new(app, LOG_WINDOW_LABEL, WebviewUrl::App("index.html".into()))
        .title(format!("Log — {instance_name}"))
        .inner_size(760.0, 480.0)
        .min_inner_size(480.0, 320.0)
        .decorations(true)
        .initialization_script(&init_script)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub async fn stop_running_instance() -> Result<(), String> {
    launcher::stop_running().await
}

#[command]
pub fn get_running_instance() -> Option<String> {
    launcher::running_instance_name()
}

/// System persistente entre llamadas — sysinfo necesita eso para poder
/// calcular %CPU (compara contra la muestra anterior del mismo proceso).
/// Uno nuevo por llamada siempre daría 0%.
static PROCESS_MONITOR: std::sync::LazyLock<std::sync::Mutex<sysinfo::System>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(sysinfo::System::new()));

#[derive(serde::Serialize)]
pub struct ProcessStats {
    pub cpu_percent: f32,
    pub memory_mb: u64,
}

/// Uso de CPU/RAM del proceso de Java en vivo — `None` si no hay ninguna
/// instancia corriendo ahora mismo. Pensado para pollearse cada 1-2s desde
/// la ventana de log mientras el juego está abierto.
#[command]
pub fn get_running_instance_stats() -> Option<ProcessStats> {
    let pid = launcher::running_instance_pid()?;
    let sysinfo_pid = sysinfo::Pid::from_u32(pid);
    let mut sys = PROCESS_MONITOR.lock().unwrap();
    sys.refresh_process(sysinfo_pid);
    let process = sys.process(sysinfo_pid)?;
    Some(ProcessStats {
        cpu_percent: process.cpu_usage(),
        memory_mb: process.memory() / 1024 / 1024,
    })
}

#[command]
pub async fn open_instance_folder(app: AppHandle, name: String) -> Result<(), String> {
    let data = instance_manager::get_instance(&name).await?;
    let dir = data.dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;
    app.opener()
        .open_path(dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}

/// Abre el diálogo nativo de "elegir archivo" filtrado a imágenes. Comando
/// separado de `set_instance_icon` para que el bloqueo del diálogo (síncrono,
/// espera a que el usuario elija) no retenga nada del lado de la instancia.
#[command]
pub fn pick_image_file(app: AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .add_filter("Imagen", &ICON_EXTENSIONS)
        .blocking_pick_file()
        .and_then(|f| f.into_path().ok())
        .map(|p| p.to_string_lossy().to_string())
}

#[command]
pub async fn set_instance_icon(name: String, source_path: String) -> Result<String, String> {
    let source = std::path::PathBuf::from(&source_path);
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .filter(|e| ICON_EXTENSIONS.contains(&e.as_str()))
        .ok_or("Formato de imagen no soportado (usá PNG, JPG o WEBP)")?;

    let instance = instance_manager::get_instance(&name).await?;
    let dir = instance.dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;

    // Un ícono previo con otra extensión no debe quedar huérfano al lado
    // del nuevo — get_instance_icon_path() solo mira uno a la vez.
    for old_ext in ICON_EXTENSIONS {
        let _ = tokio::fs::remove_file(dir.join(format!("icon.{old_ext}"))).await;
    }

    let dest = dir.join(format!("icon.{ext}"));
    tokio::fs::copy(&source, &dest)
        .await
        .map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().to_string())
}

#[derive(serde::Serialize)]
pub struct IconPreset {
    pub id: String,
    pub data_url: String,
}

/// Devuelve TODO el set de íconos vendoreados de una — 67 archivos, ~1MB
/// en base64, un único viaje de IPC en vez de uno por ícono. El picker
/// del frontend los renderiza directo como <img src="data:...">, sin
/// necesitar que el scope del asset-protocol cubra la carpeta de recursos
/// empaquetados (que además varía de ubicación real según plataforma).
#[command]
pub async fn list_instance_icon_presets(app: AppHandle) -> Result<Vec<IconPreset>, String> {
    let dir = icon_presets_dir(&app)?;
    let mut entries = tokio::fs::read_dir(&dir)
        .await
        .map_err(|e| format!("No se encontraron los íconos empaquetados: {e}"))?;
    let mut out = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("png") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Ok(bytes) = tokio::fs::read(&path).await else {
            continue;
        };
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        out.push(IconPreset {
            id: stem.to_string(),
            data_url: format!("data:image/png;base64,{b64}"),
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

#[command]
pub async fn set_instance_icon_from_preset(
    app: AppHandle,
    name: String,
    preset_id: String,
) -> Result<String, String> {
    if preset_id.contains('/') || preset_id.contains('\\') || preset_id.contains("..") {
        return Err("Id de ícono inválido".into());
    }
    let source = icon_presets_dir(&app)?.join(format!("{preset_id}.png"));
    if !source.exists() {
        return Err("Ese ícono no existe en el set".into());
    }

    let instance = instance_manager::get_instance(&name).await?;
    let dir = instance.dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;
    for old_ext in ICON_EXTENSIONS {
        let _ = tokio::fs::remove_file(dir.join(format!("icon.{old_ext}"))).await;
    }
    let dest = dir.join("icon.png");
    tokio::fs::copy(&source, &dest)
        .await
        .map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().to_string())
}

#[command]
pub async fn get_instance_icon_path(name: String) -> Option<String> {
    let instance = instance_manager::get_instance(&name).await.ok()?;
    let dir = instance.dir();
    for ext in ICON_EXTENSIONS {
        let p = dir.join(format!("icon.{ext}"));
        if p.exists() {
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
}
