//! Acceso directo de escritorio por instancia — doble click abre TFL Client
//! en segundo plano (sin ventana, queda en la bandeja del sistema, ver
//! `spawn_background_launch` en `lib.rs`) y lanza esa instancia sola, sin
//! pasar por la ventana principal. El binario se relanza a sí mismo con
//! `--launch-instance "<nombre>"`; `tauri-plugin-single-instance` se encarga
//! de que esto reuse el proceso ya abierto en vez de duplicarlo.
use crate::services::instance_manager;
use tauri::command;

fn desktop_dir() -> Result<std::path::PathBuf, String> {
    directories::UserDirs::new()
        .and_then(|d| d.desktop_dir().map(|p| p.to_path_buf()))
        .ok_or_else(|| "No se encontró la carpeta Escritorio".to_string())
}

fn sanitize_filename(name: &str) -> String {
    name.chars().map(|c| if r#"\/:*?"<>|"#.contains(c) { '_' } else { c }).collect()
}

#[cfg(target_os = "windows")]
fn write_shortcut(instance_name: &str, exe: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    let mut link = mslnk::ShellLink::new(exe).map_err(|e| e.to_string())?;
    link.set_arguments(Some(format!("--launch-instance \"{instance_name}\"")));
    link.set_icon_location(Some(exe.to_string_lossy().to_string()));
    link.create_lnk(dest).map_err(|e| e.to_string())
}

#[cfg(target_os = "macos")]
fn write_shortcut(instance_name: &str, exe: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let script = format!("#!/bin/bash\n\"{}\" --launch-instance \"{instance_name}\" &\n", exe.display());
    std::fs::write(dest, script).map_err(|e| e.to_string())?;
    let mut perms = std::fs::metadata(dest).map_err(|e| e.to_string())?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(dest, perms).map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
fn write_shortcut(instance_name: &str, exe: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let content = format!(
        "[Desktop Entry]\nType=Application\nName=TFL Client - {instance_name}\nExec=\"{}\" --launch-instance \"{instance_name}\"\nIcon=tflclient\nTerminal=false\n",
        exe.display()
    );
    std::fs::write(dest, content).map_err(|e| e.to_string())?;
    let mut perms = std::fs::metadata(dest).map_err(|e| e.to_string())?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(dest, perms).map_err(|e| e.to_string())
}

/// Devuelve la ruta del acceso directo creado — la UI la muestra para que
/// el usuario sepa dónde quedó (no siempre es obvio que fue al Escritorio).
#[command]
pub async fn create_instance_shortcut(instance_name: String) -> Result<String, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    // `--launch-instance` corre `launcher::launch` (cliente) — una instancia
    // de servidor necesita `server_process::launch_server`, otro camino
    // entero. En vez de duplicar esa lógica para un caso de uso que el
    // cliente no pidió, se bloquea acá con un mensaje claro.
    if instance.server_type.is_some() {
        return Err("Los accesos directos son para instancias de juego — para el servidor usá el botón Iniciar.".into());
    }

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = desktop_dir()?;
    let filename = sanitize_filename(&instance_name);

    #[cfg(target_os = "windows")]
    let dest = dir.join(format!("{filename}.lnk"));
    #[cfg(target_os = "macos")]
    let dest = dir.join(format!("{filename}.command"));
    #[cfg(target_os = "linux")]
    let dest = dir.join(format!("{filename}.desktop"));

    write_shortcut(&instance_name, &exe, &dest)?;
    Ok(dest.to_string_lossy().to_string())
}
