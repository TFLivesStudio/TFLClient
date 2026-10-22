use crate::services::instance_manager::InstanceData;
use crate::services::{SettingsManager, instance_manager, launcher};
use base64::Engine;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, command};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

/// Ahora se puede tener más de una instancia corriendo a la vez (ver
/// `launcher::launch`) — cada una necesita su propia ventana de log, así
/// que el label incluye el nombre (saneado: los labels de ventana de Tauri
/// no aceptan cualquier carácter).
fn log_window_label(instance_name: &str) -> String {
    let sanitized: String = instance_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    format!("log-{sanitized}")
}

const ICON_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];
const MANAGED_ICON_EXTENSIONS: [&str; 5] = ["svg", "png", "jpg", "jpeg", "webp"];

/// Carpeta empaquetada con `bundle.resources` en tauri.conf.json — íconos
/// de instancia CC0 (Simplexity-Development/Entity-Icons +
/// hube12/mc_icons) para elegir al azar al crear una instancia y después
/// poder cambiar por otro del mismo set, sin depender de red.
fn icon_presets_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    crate::core::bundled_resource_dir(app, "instance-icons")
}

#[command]
pub async fn create_instance(
    app: AppHandle,
    name: String,
    mc_version: String,
    loader: String,
) -> Result<InstanceData, String> {
    let data = instance_manager::create_resolved(name, mc_version, loader).await?;

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
    let _perf = crate::core::perf::span("instances.load");
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
pub async fn launch(
    app: AppHandle,
    instance_name: String,
    server_address: Option<String>,
) -> Result<(), String> {
    // launcher::launch() rechaza solo si ESTA MISMA instancia ya está
    // corriendo (doble click) — otra instancia distinta corriendo a la vez
    // ya se filtró antes en el frontend (confirmación si hacía falta), así
    // que acá nunca se abre una ventana de log huérfana por un lanzamiento
    // que ni siquiera arrancó.
    launcher::launch(app.clone(), instance_name.clone(), server_address).await?;
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
    let label = log_window_label(instance_name);
    if let Some(existing) = app.get_webview_window(&label) {
        let _ = existing.set_focus();
        return Ok(());
    }

    let instance_json = serde_json::to_string(instance_name).map_err(|e| e.to_string())?;
    let init_script = format!("window.__TFL_LOG_INSTANCE__ = {instance_json};");

    WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
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
pub async fn stop_running_instance(instance_name: String) -> Result<(), String> {
    launcher::stop_running(&instance_name).await
}

#[derive(serde::Serialize)]
pub struct RunningInstanceInfo {
    pub name: String,
    pub account_uuid: String,
}

/// Todas las instancias de cliente corriendo ahora mismo, con la cuenta
/// que se usó para lanzar cada una — el frontend lo usa para avisar antes
/// de abrir una instancia más si eso puede traer problemas (misma cuenta
/// en dos lados, o rendimiento).
#[command]
pub fn get_running_instances() -> Vec<RunningInstanceInfo> {
    launcher::running_instances()
        .into_iter()
        .map(|(name, account_uuid)| RunningInstanceInfo { name, account_uuid })
        .collect()
}

// One persistent sampler per PID. Coalesce simultaneous consumers so two windows
// cannot reset each other's CPU/disk baselines. First sample has no CPU baseline.
struct ProcessSampler {
    system: sysinfo::System,
    at: std::time::Instant,
    sample: ProcessStats,
}
static PROCESS_MONITOR: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<u32, ProcessSampler>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

#[derive(Clone, serde::Serialize)]
pub struct ProcessStats {
    /// Whole machine percentage, retained for existing consumers.
    pub cpu_percent: f32,
    /// 100% is one logical core. Multithreaded Java can exceed 100%.
    pub cpu_core_percent: f32,
    pub logical_cores: usize,
    /// Resident memory, not JVM heap usage.
    pub memory_mb: u64,
    pub disk_bytes_delta: u64,
    pub sampled_at_ms: u64,
    pub sample_interval_ms: u64,
    pub ready: bool,
}

fn process_stats(pid: u32) -> Option<ProcessStats> {
    let now = std::time::Instant::now();
    let mut monitors = PROCESS_MONITOR.lock().ok()?;
    monitors.retain(|_, m| now.duration_since(m.at).as_secs() < 180);
    if let Some(m) = monitors.get(&pid) {
        if now.duration_since(m.at).as_millis() < 500 {
            return Some(m.sample.clone());
        }
    }
    let mut system = monitors.remove(&pid);
    let interval = system
        .as_ref()
        .map(|m| now.duration_since(m.at).as_millis() as u64);
    let mut sys = system.take().map(|m| m.system).unwrap_or_default();
    let sys_pid = sysinfo::Pid::from_u32(pid);
    if !sys.refresh_process(sys_pid) {
        return None;
    }
    let process = sys.process(sys_pid)?;
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let raw = core_cpu_percent(process.cpu_usage());
    let disk = process.disk_usage();
    let sample = ProcessStats {
        cpu_percent: normalize_cpu_percent(raw, cores as f32),
        cpu_core_percent: raw,
        logical_cores: cores,
        memory_mb: process.memory() / 1024 / 1024,
        disk_bytes_delta: disk.read_bytes.saturating_add(disk.written_bytes),
        sampled_at_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_millis() as u64,
        sample_interval_ms: interval.unwrap_or(0),
        ready: interval.is_some(),
    };
    monitors.insert(
        pid,
        ProcessSampler {
            system: sys,
            at: now,
            sample: sample.clone(),
        },
    );
    Some(sample)
}

#[command]
pub fn get_running_instance_stats(instance_name: String) -> Option<ProcessStats> {
    process_stats(launcher::running_instance_pid(&instance_name)?)
}

#[command]
pub fn get_running_server_stats(instance_name: String) -> Option<ProcessStats> {
    process_stats(crate::services::server_process::running_server_pid(
        &instance_name,
    )?)
}

fn core_cpu_percent(raw: f32) -> f32 {
    if raw.is_finite() { raw.max(0.0) } else { 0.0 }
}
fn normalize_cpu_percent(raw: f32, cores: f32) -> f32 {
    (core_cpu_percent(raw) / cores.max(1.0)).min(100.0)
}

#[cfg(test)]
mod cpu_percent_tests {
    use super::{core_cpu_percent, normalize_cpu_percent};
    #[test]
    fn multiple_cores_are_preserved() {
        assert_eq!(core_cpu_percent(425.0), 425.0);
        assert_eq!(normalize_cpu_percent(400.0, 8.0), 50.0);
        assert_eq!(normalize_cpu_percent(101.0, 1.0), 100.0);
    }
    #[test]
    fn sampler_reads_a_real_process_and_coalesces_consumers() {
        let pid = std::process::id();
        let first = super::process_stats(pid).expect("test process exists");
        assert!(first.memory_mb > 0);
        let cached = super::process_stats(pid).unwrap();
        assert_eq!(first.sampled_at_ms, cached.sampled_at_ms);
        std::thread::sleep(std::time::Duration::from_millis(550));
        let next = super::process_stats(pid).unwrap();
        assert!(next.ready);
        assert!(next.sample_interval_ms >= 500);
        assert!(next.cpu_core_percent.is_finite());
        assert!(next.sampled_at_ms > first.sampled_at_ms);
        assert!(super::process_stats(u32::MAX).is_none());
    }
    #[test]
    fn invalid_cpu_cannot_poison_charts() {
        assert_eq!(core_cpu_percent(f32::NAN), 0.0);
        assert_eq!(core_cpu_percent(f32::INFINITY), 0.0);
        assert_eq!(core_cpu_percent(-1.0), 0.0);
    }
}

/// "Techo" del gauge de RAM de la ventana de log — no el cálculo completo
/// de lanzamiento (que además escala por cantidad de mods, ver
/// `services::launcher::launch`), solo una referencia visual de cuánta RAM
/// tiene asignada esta instancia. Se pide una sola vez al abrir la ventana,
/// no en cada poll.
#[command]
pub async fn get_instance_ram_budget_mb(instance_name: String) -> u32 {
    let global_max = SettingsManager::read().max_memory;
    instance_ram_budget_mb(
        instance_manager::get_instance(&instance_name)
            .await
            .ok()
            .and_then(|d| d.max_memory),
        global_max,
    )
}

/// Separado de `get_instance_ram_budget_mb` para poder probarlo sin tocar
/// disco ni el estado global de `SettingsManager`.
fn instance_ram_budget_mb(instance_override: Option<u32>, global_max: u32) -> u32 {
    instance_override.unwrap_or(global_max)
}

#[cfg(test)]
mod ram_budget_tests {
    use super::instance_ram_budget_mb;

    #[test]
    fn usa_el_override_de_la_instancia_si_lo_tiene() {
        assert_eq!(instance_ram_budget_mb(Some(6144), 2048), 6144);
    }

    #[test]
    fn cae_al_global_sin_override() {
        assert_eq!(instance_ram_budget_mb(None, 2048), 2048);
    }
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

/// Abre una URL en el navegador del sistema — restringido a
/// github.com/TFLivesStudio/TFLClient (el panel de "qué hay de nuevo" es lo
/// único que llama esto, para linkear al Release). No es un "abrir
/// cualquier URL" genérico a propósito, evita que se pueda usar para abrir
/// lo que sea.
#[command]
pub fn open_external_url(app: AppHandle, url: String) -> Result<(), String> {
    if !url.starts_with("https://github.com/TFLivesStudio/TFLClient/") {
        return Err("URL no permitida".into());
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

/// Abre el diálogo nativo de "elegir archivo" filtrado a imágenes. Comando
/// separado de `set_instance_icon` para que el bloqueo del diálogo (síncrono,
/// espera a que el usuario elija) no retenga nada del lado de la instancia.
///
/// `async` a propósito: los comandos síncronos de Tauri corren en el hilo
/// principal, y `blocking_pick_file` NO debe usarse desde ahí (lo indica la
/// documentación de tauri-plugin-dialog — el diálogo necesita ese mismo
/// hilo para mostrarse, así que queda esperándose a sí mismo en macOS).
#[command]
pub async fn pick_image_file(app: AppHandle) -> Option<String> {
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

    let dest = dir.join(format!("icon.{ext}"));
    let temp = dir.join("icon-upload.tmp");
    tokio::fs::copy(&source, &temp)
        .await
        .map_err(|e| e.to_string())?;
    tokio::fs::copy(&temp, &dest)
        .await
        .map_err(|e| e.to_string())?;
    let _ = tokio::fs::remove_file(temp).await;
    for old_ext in MANAGED_ICON_EXTENSIONS {
        if old_ext != ext {
            let _ = tokio::fs::remove_file(dir.join(format!("icon.{old_ext}"))).await;
        }
    }
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
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !["png", "svg"].contains(&ext) {
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
            data_url: format!(
                "data:image/{};base64,{b64}",
                if ext == "svg" { "svg+xml" } else { "png" }
            ),
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
    let presets = icon_presets_dir(&app)?;
    let source = ["svg", "png"]
        .into_iter()
        .map(|ext| presets.join(format!("{preset_id}.{ext}")))
        .find(|path| path.is_file())
        .ok_or("Ese ícono no existe en el set")?;
    if !source.exists() {
        return Err("Ese ícono no existe en el set".into());
    }

    let instance = instance_manager::get_instance(&name).await?;
    let dir = instance.dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .ok_or("Formato inválido")?;
    let dest = dir.join(format!("icon.{ext}"));
    let temp = dir.join("icon-preset.tmp");
    tokio::fs::copy(&source, &temp)
        .await
        .map_err(|e| e.to_string())?;
    // Copy succeeds before the previous icon is replaced (also works on Windows).
    tokio::fs::copy(&temp, &dest)
        .await
        .map_err(|e| e.to_string())?;
    let _ = tokio::fs::remove_file(&temp).await;
    for old_ext in MANAGED_ICON_EXTENSIONS {
        if old_ext != ext {
            let _ = tokio::fs::remove_file(dir.join(format!("icon.{old_ext}"))).await;
        }
    }
    Ok(dest.to_string_lossy().to_string())
}

#[command]
pub async fn get_instance_icon_path(name: String) -> Option<String> {
    let instance = instance_manager::get_instance(&name).await.ok()?;
    let dir = instance.dir();
    for ext in MANAGED_ICON_EXTENSIONS {
        let p = dir.join(format!("icon.{ext}"));
        if p.exists() {
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
}

#[derive(serde::Serialize)]
pub struct ScreenshotInfo {
    pub filename: String,
    pub path: String,
    pub modified_ms: u64,
}

/// Minecraft guarda las capturas en `<instancia>/screenshots/*.png` solo —
/// no hace falta ningún comando nuevo de "sacar captura", el juego ya lo
/// hace con F2. Esto es puramente el gestor: listar/borrar/abrir carpeta.
#[command]
pub async fn get_instance_screenshots(name: String) -> Result<Vec<ScreenshotInfo>, String> {
    let instance = instance_manager::get_instance(&name).await?;
    let dir = instance.dir().join("screenshots");
    let mut out = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
        return Ok(out); // sin capturas todavía, no es un error
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !["png", "svg"].contains(&ext) {
            continue;
        }
        let Some(filename) = path.file_name().and_then(|f| f.to_str()) else {
            continue;
        };
        let modified_ms = entry
            .metadata()
            .await
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        out.push(ScreenshotInfo {
            filename: filename.to_string(),
            path: path.to_string_lossy().to_string(),
            modified_ms,
        });
    }
    out.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms));
    Ok(out)
}

#[command]
pub async fn delete_screenshot(name: String, filename: String) -> Result<(), String> {
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err("Nombre de archivo inválido".into());
    }
    let instance = instance_manager::get_instance(&name).await?;
    let path = instance.dir().join("screenshots").join(filename);
    tokio::fs::remove_file(&path)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn open_screenshots_folder(app: AppHandle, name: String) -> Result<(), String> {
    let instance = instance_manager::get_instance(&name).await?;
    let dir = instance.dir().join("screenshots");
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;
    app.opener()
        .open_path(dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}

/// Cuando la JVM crashea de forma nativa (segfault, OOM nativo, etc — no un
/// error normal de Java) escribe un `hs_err_pidNNNN.log` en el directorio
/// de trabajo del proceso, que es justo el de la instancia (ver
/// `current_dir` en launchwerk). El log de Minecraft (stdout/stderr, lo que
/// ve InstanceLogWindow) no tiene ni idea de que ese archivo existe — antes
/// el usuario se quedaba sin ninguna pista de qué pasó en ese caso. Solo se
/// devuelve el resumen (las primeras ~40 líneas, donde está la causa: "SIGSEGV",
/// "Problematic frame", motivo de OOM nativo, etc) — el resto del archivo son
/// volcados de registros/stack de cada thread, miles de líneas que no
/// aportan nada a un jugador común.
#[command]
pub async fn get_crash_report(name: String) -> Result<Option<String>, String> {
    let instance = instance_manager::get_instance(&name).await?;
    let dir = instance.dir();

    let mut newest: Option<(std::path::PathBuf, std::time::SystemTime)> = None;
    let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
        return Ok(None);
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        if !name.starts_with("hs_err_pid") || !name.ends_with(".log") {
            continue;
        }
        let Ok(metadata) = entry.metadata().await else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if newest.as_ref().is_none_or(|(_, t)| modified > *t) {
            newest = Some((entry.path(), modified));
        }
    }

    let Some((path, _)) = newest else {
        return Ok(None);
    };
    use tokio::io::AsyncReadExt;
    let file = tokio::fs::File::open(&path)
        .await
        .map_err(|_| "Crash report unavailable")?;
    let mut bytes = Vec::new();
    file.take(65536)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| "Crash report read failed")?;
    let content = String::from_utf8_lossy(&bytes);
    let summary: String = content.lines().take(40).collect::<Vec<_>>().join("\n");
    Ok(Some(crate::core::session_privacy::redact(&summary)))
}
