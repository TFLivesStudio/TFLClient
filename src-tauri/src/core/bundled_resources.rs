use tauri::{AppHandle, Manager};

/// Resuelve una subcarpeta dentro de los recursos empaquetados
/// (`bundle.resources` en tauri.conf.json). `resource_dir()` resuelve a
/// `${exe_dir}/../Resources` en macOS — válido dentro de un .app/.exe
/// real, pero en `tauri dev` el binario corre suelto desde `target/debug/`
/// sin ningún bundle alrededor, esa carpeta no existe ahí. Sin el
/// fallback de desarrollo (a `CARGO_MANIFEST_DIR/resources/<subdir>`,
/// solo en debug_assertions), cualquier feature que dependa de un recurso
/// empaquetado aparenta estar rota en toda sesión de desarrollo aunque
/// funcione en un build real — ya pasó una vez con los íconos de
/// instancia, este helper evita repetir ese mismo bug en cada feature
/// nueva que necesite lo mismo (ej. los jars de mp-guard).
pub fn bundled_resource_dir(app: &AppHandle, subdir: &str) -> Result<std::path::PathBuf, String> {
    if let Ok(d) = app.path().resource_dir() {
        let candidate = d.join(subdir);
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }

    #[cfg(debug_assertions)]
    {
        let dev_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(subdir);
        if dev_path.is_dir() {
            return Ok(dev_path);
        }
    }

    Err(format!("No se encontró el recurso empaquetado: {subdir}"))
}
