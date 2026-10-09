//! "Estado del sistema" de la home: Java/RAM/Disco. Solo números reales de
//! lo que el launcher ya puede ver — nada de heurísticas de "¿hay un
//! problema?" inventadas, eso lo decide quien mire los números.

use crate::core::PathManager;
use crate::services::{SettingsManager, java_manager};
use serde::Serialize;
use sysinfo::{Disks, MemoryRefreshKind, RefreshKind, System};
use tauri::command;

#[derive(Serialize)]
pub struct SystemStatus {
    /// El mayor major de Java instalado (de los que el launcher gestiona),
    /// o `None` si no hay ninguno todavía.
    pub java_major: Option<u8>,
    pub ram_assigned_mb: u32,
    pub ram_total_mb: u32,
    pub disk_free_gb: f32,
    pub disk_total_gb: f32,
    pub instances_dir: String,
}

/// El disco cuyo punto de montaje es el prefijo más largo de `path` — el
/// algoritmo estándar para "¿qué partición contiene esta carpeta?" (más de
/// un disco puede tener un punto de montaje que sea prefijo; el más largo
/// es el más específico, ej. "/" y "/home" si `path` vive en "/home").
fn disk_for_path<'a>(disks: &'a Disks, path: &std::path::Path) -> Option<&'a sysinfo::Disk> {
    disks
        .list()
        .iter()
        .filter(|d| path.starts_with(d.mount_point()))
        .max_by_key(|d| d.mount_point().as_os_str().len())
}

#[command]
pub fn get_system_status() -> SystemStatus {
    let java_major = java_manager::status()
        .into_iter()
        .filter(|s| s.installed)
        .map(|s| s.major)
        .max();

    let sys = System::new_with_specifics(
        RefreshKind::new().with_memory(MemoryRefreshKind::new().with_ram()),
    );
    let ram_total_mb = (sys.total_memory() / 1024 / 1024).max(1) as u32;
    let ram_assigned_mb = SettingsManager::read().max_memory;

    let instances_dir = PathManager::get().get_instance_dir();
    let disks = Disks::new_with_refreshed_list();
    let (disk_free_gb, disk_total_gb) = match disk_for_path(&disks, instances_dir) {
        Some(d) => (
            d.available_space() as f32 / 1_073_741_824.0,
            d.total_space() as f32 / 1_073_741_824.0,
        ),
        None => (0.0, 0.0),
    };

    SystemStatus {
        java_major,
        ram_assigned_mb,
        ram_total_mb,
        disk_free_gb,
        disk_total_gb,
        instances_dir: instances_dir.display().to_string(),
    }
}
