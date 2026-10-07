//! Agregar contenido por archivo local.
//!
//! Usa el mismo diálogo nativo que `pick_image_file` (instance.rs) — por eso
//! el frontend solo ofrece esto cuando `settings.native_dialog_mode ===
//! 'manual'` (ver NativeDialogModePrompt.svelte y CHANGELOG_macos-dialog-
//! crash-java26.txt para el porqué). Acá no hay bloqueo del lado Rust: el
//! gate es una decisión de UX que vive en el frontend/Ajustes.

use super::filesystem::is_unsafe_filename;
use super::types::ContentKind;
use crate::services::instance_manager;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

/// Diálogo nativo de "elegir archivos" con selección múltiple, filtrado a
/// una extensión — separado de la copia en sí para que el bloqueo del
/// diálogo (síncrono) no retenga nada del lado de la instancia.
pub fn pick_files(app: &AppHandle, extension: &str, filter_label: &str) -> Vec<String> {
    app.dialog()
        .file()
        .add_filter(filter_label, &[extension])
        .blocking_pick_files()
        .map(|files| {
            files
                .into_iter()
                .filter_map(|f| f.into_path().ok())
                .map(|p| p.to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Nombre de archivo a copiar si `source` tiene la extensión pedida y un
/// nombre seguro (mismo criterio que `download_single_file`).
fn accepted_filename<'a>(source: &'a std::path::Path, ext: &str) -> Option<&'a str> {
    let matches_ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case(ext))
        .unwrap_or(false);
    if !matches_ext {
        return None;
    }
    let filename = source.file_name().and_then(|f| f.to_str())?;
    if is_unsafe_filename(filename) {
        return None;
    }
    Some(filename)
}

/// Copia cada archivo elegido a la carpeta del tipo dentro de la instancia,
/// filtrando por extensión y nombre de archivo seguro. Sigue de largo si un
/// archivo puntual falla, no aborta el resto — devuelve cuántos se copiaron
/// de verdad.
pub async fn add_local_files(
    instance_name: &str,
    kind: ContentKind,
    paths: Vec<String>,
) -> Result<u32, String> {
    let instance = instance_manager::get_instance(instance_name).await?;
    let dest_dir = instance.dir().join(kind.subdir());
    tokio::fs::create_dir_all(&dest_dir)
        .await
        .map_err(|e| e.to_string())?;

    let mut count = 0u32;
    for path_str in paths {
        let source = std::path::PathBuf::from(&path_str);
        let Some(filename) = accepted_filename(&source, kind.extension()) else {
            continue;
        };
        if tokio::fs::copy(&source, dest_dir.join(filename))
            .await
            .is_ok()
        {
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn only_files_with_the_right_extension_are_accepted() {
        assert_eq!(
            accepted_filename(Path::new("/tmp/Mod.JAR"), "jar"),
            Some("Mod.JAR")
        );
        assert_eq!(accepted_filename(Path::new("/tmp/pack.zip"), "jar"), None);
        assert_eq!(accepted_filename(Path::new("/tmp/noext"), "jar"), None);
    }

    #[test]
    fn filenames_with_dot_dot_are_rejected() {
        assert_eq!(accepted_filename(Path::new("/tmp/a..b.jar"), "jar"), None);
    }
}
