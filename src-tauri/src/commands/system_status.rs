//! Machine hardware inventory, distinct from launcher/instance resource usage.
use crate::core::PathManager;
use crate::services::SettingsManager;
use serde::Serialize;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};

#[derive(Clone, Serialize)]
pub struct GpuAdapter {
    pub name: String,
    pub provider: String,
    pub vram_bytes: Option<u64>,
}
#[derive(Clone, Serialize)]
pub struct SystemStatus {
    pub cpu_model: Option<String>,
    pub gpu_adapters: Vec<GpuAdapter>,
    pub hardware_error: Option<String>,
    pub ram_assigned_mb: u32,
    pub ram_total_mb: u32,
    pub ram_available_mb: u32,
    pub disk_free_gb: Option<f32>,
    pub disk_total_gb: Option<f32>,
    pub instances_dir: String,
}
type Inventory = (Option<String>, Vec<GpuAdapter>, Option<String>);
static INVENTORY: LazyLock<Mutex<Option<(Instant, Inventory)>>> =
    LazyLock::new(|| Mutex::new(None));

fn disk_for_path<'a>(disks: &'a Disks, path: &std::path::Path) -> Option<&'a sysinfo::Disk> {
    disks
        .list()
        .iter()
        .filter(|d| path.starts_with(d.mount_point()))
        .max_by_key(|d| d.mount_point().as_os_str().len())
}

/// Fixed local commands only. Bounded startup, hidden windows, no shell input from IPC.
fn provider_output(program: &str, args: &[&str]) -> Result<String, String> {
    use std::process::{Command, Stdio};
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .spawn()
        .map_err(|_| "Proveedor de hardware no disponible".to_string())?;
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(Some(_)) | Err(_) => {
                return Err("El proveedor de hardware no respondió correctamente".into());
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("El proveedor de hardware agotó el tiempo de espera".into());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(25)),
        }
    }
    let output = child
        .wait_with_output()
        .map_err(|_| "No se pudo leer el hardware".to_string())?;
    if output.stdout.len() > 1_048_576 {
        return Err("Respuesta de hardware fuera de límite".into());
    }
    String::from_utf8(output.stdout).map_err(|_| "Respuesta de hardware inválida".into())
}

fn gpu_inventory() -> Result<Vec<GpuAdapter>, String> {
    #[cfg(target_os = "windows")]
    {
        let raw = provider_output(
            "powershell.exe",
            &[
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "[Console]::OutputEncoding=[Text.UTF8Encoding]::new(); @(Get-CimInstance Win32_VideoController | Select-Object Name) | ConvertTo-Json -Compress",
            ],
        )?;
        let value: serde_json::Value = serde_json::from_str(raw.trim_start_matches('\u{feff}'))
            .map_err(|_| "Inventario GPU inválido")?;
        let values = value.as_array().cloned().unwrap_or_else(|| vec![value]);
        Ok(values
            .iter()
            .filter_map(|v| v["Name"].as_str())
            .filter(|n| !n.trim().is_empty())
            .map(|n| GpuAdapter {
                name: n.to_string(),
                provider: "Windows CIM · Win32_VideoController".into(),
                vram_bytes: None,
            })
            .collect())
    }
    #[cfg(target_os = "macos")]
    {
        let raw = provider_output(
            "/usr/sbin/system_profiler",
            &["SPDisplaysDataType", "-json"],
        )?;
        let value: serde_json::Value =
            serde_json::from_str(&raw).map_err(|_| "Inventario GPU inválido")?;
        Ok(value["SPDisplaysDataType"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v["sppci_model"].as_str())
            .map(|n| GpuAdapter {
                name: n.into(),
                provider: "macOS system_profiler".into(),
                vram_bytes: None,
            })
            .collect())
    }
    #[cfg(target_os = "linux")]
    {
        let raw = provider_output("lspci", &["-mm"])?;
        Ok(raw
            .lines()
            .filter(|l| {
                l.contains("VGA compatible controller")
                    || l.contains("3D controller")
                    || l.contains("Display controller")
            })
            .filter_map(|l| {
                let fields: Vec<&str> = l.split('"').collect();
                (fields.len() >= 6).then(|| GpuAdapter {
                    name: format!("{} {}", fields[3], fields[5]),
                    provider: "Linux PCI · lspci".into(),
                    vram_bytes: None,
                })
            })
            .collect())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("No hay proveedor de GPU para esta plataforma".into())
    }
}

#[tauri::command]
pub async fn get_system_status() -> Result<SystemStatus, String> {
    tokio::task::spawn_blocking(|| {
        let sys = System::new_with_specifics(
            RefreshKind::new()
                .with_memory(MemoryRefreshKind::new().with_ram())
                .with_cpu(CpuRefreshKind::new()),
        );
        let mut cache = INVENTORY.lock().map_err(|_| "Inventario no disponible")?;
        if cache
            .as_ref()
            .is_none_or(|(at, _)| at.elapsed() >= Duration::from_secs(600))
        {
            let model = sys
                .cpus()
                .first()
                .map(|c| c.brand().trim().to_string())
                .filter(|c| !c.is_empty());
            let (gpus, error) = match gpu_inventory() {
                Ok(gpus) => (gpus, None),
                Err(e) => (vec![], Some(e)),
            };
            *cache = Some((Instant::now(), (model, gpus, error)));
        }
        let (cpu_model, gpu_adapters, hardware_error) =
            cache.as_ref().ok_or("Inventario no disponible")?.1.clone();
        drop(cache);
        let instances_dir = PathManager::get().get_instance_dir();
        let disks = Disks::new_with_refreshed_list();
        let disk = disk_for_path(&disks, instances_dir);
        Ok(SystemStatus {
            cpu_model,
            gpu_adapters,
            hardware_error,
            ram_assigned_mb: SettingsManager::read().max_memory,
            ram_total_mb: (sys.total_memory() / 1_048_576) as u32,
            ram_available_mb: (sys.available_memory() / 1_048_576) as u32,
            disk_free_gb: disk.map(|d| d.available_space() as f32 / 1_073_741_824.0),
            disk_total_gb: disk.map(|d| d.total_space() as f32 / 1_073_741_824.0),
            instances_dir: instances_dir.display().to_string(),
        })
    })
    .await
    .map_err(|_| "No se pudo consultar el hardware".to_string())?
}
