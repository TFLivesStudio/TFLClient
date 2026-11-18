//! macOS file panels execute in a separate OS process. A failure in AppKit
//! does not terminate the launcher, including when running an ad-hoc bundle.
#[cfg(target_os = "macos")]
use std::path::PathBuf;

pub async fn pick_files(
    app: tauri::AppHandle,
    label: &str,
    extensions: &[&str],
    multiple: bool,
) -> Result<Vec<String>, String> {
    if extensions.is_empty()
        || extensions.len() > 12
        || extensions
            .iter()
            .any(|s| s.is_empty() || s.len() > 16 || !s.bytes().all(|b| b.is_ascii_alphanumeric()))
    {
        return Err("Filtro de archivos inválido".into());
    }
    #[cfg(target_os = "macos")]
    {
        let _ = (app, label);
        Ok(pick_macos(false, multiple, extensions)
            .await?
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect())
    }
    #[cfg(not(target_os = "macos"))]
    {
        use tauri_plugin_dialog::DialogExt;
        let label = label.to_string();
        let extensions: Vec<String> = extensions.iter().map(|s| s.to_string()).collect();
        tokio::task::spawn_blocking(move || {
            let extensions: Vec<_> = extensions.iter().map(String::as_str).collect();
            let picker = app.dialog().file().add_filter(label, &extensions);
            let files = if multiple {
                picker.blocking_pick_files().unwrap_or_default()
            } else {
                picker.blocking_pick_file().into_iter().collect()
            };
            files
                .into_iter()
                .filter_map(|f| f.into_path().ok())
                .map(|p| p.to_string_lossy().into_owned())
                .collect()
        })
        .await
        .map_err(|_| "No se pudo abrir el selector de archivos".into())
    }
}

#[cfg(target_os = "macos")]
pub async fn pick_macos(
    folder: bool,
    multiple: bool,
    extensions: &[&str],
) -> Result<Vec<PathBuf>, String> {
    const PICKER: &str = r#"
function run(argv) {
    const app = Application.currentApplication();
    app.includeStandardAdditions = true;
    const options = {withPrompt: argv[0] === 'folder' ? 'Selecciona una carpeta' : 'Selecciona un archivo'};
    if (argv[1] === 'many') options.multipleSelectionsAllowed = true;
    if (argv.length > 2) options.ofType = argv.slice(2);
    try {
        const result = argv[0] === 'folder' ? app.chooseFolder(options) : app.chooseFile(options);
        return JSON.stringify((Array.isArray(result) ? result : [result]).map(p => p.toString()));
    } catch (error) {
        if (error.errorNumber === -128) return '[]';
        throw error;
    }
}
"#;
    let mut command = tokio::process::Command::new("/usr/bin/osascript");
    command.args([
        "-l",
        "JavaScript",
        "-e",
        PICKER,
        "--",
        if folder { "folder" } else { "file" },
        if multiple { "many" } else { "one" },
    ]);
    if !folder {
        command.args(
            extensions
                .iter()
                .copied()
                .filter(|s| s.len() <= 16 && s.bytes().all(|b| b.is_ascii_alphanumeric())),
        );
    }
    command.kill_on_drop(true);
    let selected = tokio::time::timeout(std::time::Duration::from_secs(300), command.output())
        .await
        .map_err(|_| "La selección de archivos agotó el tiempo de espera")?
        .map_err(|_| "No se pudo abrir el selector de archivos de macOS")?;
    if !selected.status.success() {
        return Err("El selector de archivos de macOS no respondió. Puedes arrastrar el archivo a la ventana cuando ese formato lo permita.".into());
    }
    let paths: Vec<PathBuf> = serde_json::from_slice(&selected.stdout)
        .map_err(|_| "El selector devolvió una ruta inválida")?;
    if paths.len() > 512 || paths.iter().any(|p| !p.is_absolute()) {
        return Err("La selección contiene rutas no compatibles".into());
    }
    Ok(paths)
}
