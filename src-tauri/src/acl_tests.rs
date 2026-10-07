//! Prueba el ACL REAL de Tauri (las capabilities y el app manifest que
//! `build.rs` compila dentro de la app) con un runtime simulado: qué comandos
//! puede invocar la ventana principal y cuáles la ventana de log.
//!
//! El handler es de mentira (resuelve todo): lo que se ejercita es la
//! decisión de permiso, que Tauri toma ANTES de llegar al handler.

use std::path::{Path, PathBuf};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;
use tauri::{WebviewUrl, WebviewWindow, WebviewWindowBuilder};

fn app() -> tauri::App<MockRuntime> {
    mock_builder()
        .invoke_handler(|invoke| {
            invoke.resolver.resolve(());
            true
        })
        .build(crate::tauri_context::<MockRuntime>())
        .expect("no se pudo armar la app de prueba")
}

fn window(app: &tauri::App<MockRuntime>, label: &str) -> WebviewWindow<MockRuntime> {
    WebviewWindowBuilder::new(app, label, WebviewUrl::default())
        .build()
        .expect("no se pudo crear la ventana de prueba")
}

/// Origen "local" del WebView: cambia según la plataforma.
fn local_url() -> &'static str {
    if cfg!(windows) {
        "http://tauri.localhost"
    } else {
        "tauri://localhost"
    }
}

fn allowed(w: &WebviewWindow<MockRuntime>, cmd: &str) -> bool {
    let r = get_ipc_response(
        w,
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: local_url().parse().unwrap(),
            body: InvokeBody::default(),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    );
    // Un error de ARGUMENTOS (los comandos de plugin reales validan lo que
    // reciben) significa que el ACL ya dejó pasar la llamada: solo cuenta
    // como denegado el rechazo explícito por permisos.
    match r {
        Ok(_) => true,
        Err(e) => !e.to_string().contains("not allowed"),
    }
}

#[test]
fn main_window_can_use_every_app_command_and_log_window_only_its_own() {
    let app = app();
    let main = window(&app, "main");
    let log = window(&app, "log-prueba");

    // La principal: comandos propios de todo tipo.
    for cmd in [
        "get_settings",
        "update_settings",
        "launch",
        "install_mod",
        "update_all_mods",
        "restore_world_backup",
        "create_instance_shortcut",
    ] {
        assert!(
            allowed(&main, cmd),
            "la ventana principal debería poder `{cmd}`"
        );
    }

    // La de log: solo lo de la pantalla de log.
    for cmd in [
        "get_running_instance_stats",
        "stop_running_instance",
        "get_crash_report",
        "diagnose_crash",
        "set_mod_enabled",
        "rollback_mod_update",
    ] {
        assert!(
            allowed(&log, cmd),
            "la ventana de log debería poder `{cmd}`"
        );
    }
    for cmd in [
        "get_settings",
        "update_settings",
        "launch",
        "install_mod",
        "update_all_mods",
        "restore_world_backup",
        "create_instance_shortcut",
        "remove_mod",
        "import_external_instance",
        "playit_unlink",
    ] {
        assert!(
            !allowed(&log, cmd),
            "la ventana de log NO debería poder `{cmd}`"
        );
    }
}

#[test]
fn plugins_are_limited_to_what_the_ui_uses() {
    let app = app();
    let main = window(&app, "main");
    let log = window(&app, "log-prueba");

    // Eventos: escuchar sí, emitir no (la UI nunca emite).
    assert!(allowed(&main, "plugin:event|listen"));
    assert!(allowed(&log, "plugin:event|listen"));
    assert!(!allowed(&main, "plugin:event|emit"));
    assert!(!allowed(&log, "plugin:event|emit"));

    // Ventana: la principal controla su barra de título, el log solo se cierra.
    assert!(allowed(&main, "plugin:window|minimize"));
    assert!(allowed(&main, "plugin:window|close"));
    assert!(!allowed(&main, "plugin:window|set_always_on_top"));
    assert!(allowed(&log, "plugin:window|close"));
    assert!(!allowed(&log, "plugin:window|minimize"));

    // Lo que la interfaz principal sí usa: portapapeles, updater, reinicio,
    // lectura de la captura para copiarla.
    for cmd in [
        "plugin:clipboard-manager|write_text",
        "plugin:clipboard-manager|write_image",
        "plugin:updater|check",
        "plugin:process|restart",
        "plugin:image|from_path",
        "plugin:resources|close",
    ] {
        assert!(allowed(&main, cmd), "la principal debería poder `{cmd}`");
        assert!(!allowed(&log, cmd), "el log NO debería poder `{cmd}`");
    }
    assert!(!allowed(&main, "plugin:clipboard-manager|read_text"));

    // Fuera de lo que usa la interfaz: ni DevTools ni diálogos ni salir de la app.
    for cmd in [
        "plugin:webview|internal_toggle_devtools",
        "plugin:dialog|open",
        "plugin:process|exit",
        "plugin:path|resolve_directory",
        "plugin:app|app_hide",
    ] {
        assert!(
            !allowed(&main, cmd),
            "la principal NO debería poder `{cmd}`"
        );
        assert!(!allowed(&log, cmd), "el log NO debería poder `{cmd}`");
    }
}

#[test]
fn an_unknown_window_label_gets_nothing() {
    let app = app();
    let other = window(&app, "otra-ventana");
    assert!(!allowed(&other, "get_settings"));
    assert!(!allowed(&other, "plugin:event|listen"));
}

// ── El log solo puede llamar lo que su pantalla realmente usa ──────────────

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file(&path, name) {
                return Some(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(path);
        }
    }
    None
}

fn read_all_ts(dir: &Path, out: &mut String) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                read_all_ts(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("ts") {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    out.push_str(&text);
                    out.push('\n');
                }
            }
        }
    }
}

/// Comandos que usa el componente de la ventana de log, tanto directos
/// (`invoke('x')`) como a través de funciones de `src/lib/api`.
fn commands_used_by_log_window() -> Vec<String> {
    let src = repo_root().join("src");
    let component = find_file(&src, "InstanceLogWindow.svelte")
        .expect("no se encontró InstanceLogWindow.svelte");
    let text = std::fs::read_to_string(component).unwrap();

    let mut commands = Vec::new();
    let direct = regex::Regex::new(r#"invoke(?:<[^>]*>)?\(\s*['"]([a-z_]+)['"]"#).unwrap();
    for c in direct.captures_iter(&text) {
        commands.push(c[1].to_string());
    }

    // Mapa función de la capa API -> comando de Rust.
    let mut api_text = String::new();
    read_all_ts(&src.join("lib").join("api"), &mut api_text);
    let api_fn = regex::Regex::new(
        r#"export (?:const|function) (\w+)\b[^;]*?invoke(?:<[^;]*?>)?\(\s*['"]([a-z_]+)['"]"#,
    )
    .unwrap();
    let mut fn_to_cmd = std::collections::HashMap::new();
    for c in api_fn.captures_iter(&api_text) {
        fn_to_cmd.insert(c[1].to_string(), c[2].to_string());
    }
    let import =
        regex::Regex::new(r#"import\s*\{([^}]*)\}\s*from\s*['"]\$lib/api[^'"]*['"]"#).unwrap();
    for c in import.captures_iter(&text) {
        for name in c[1].split(',') {
            let name = name.trim();
            if let Some(cmd) = fn_to_cmd.get(name) {
                commands.push(cmd.clone());
            }
        }
    }
    commands.sort();
    commands.dedup();
    commands
}

#[test]
fn log_window_capability_covers_exactly_what_its_screen_uses() {
    let raw = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("capabilities")
            .join("log-window.json"),
    )
    .unwrap();
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let mut granted: Vec<String> = json["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p.as_str())
        .filter_map(|p| p.strip_prefix("allow-"))
        .map(|p| p.replace('-', "_"))
        .collect();
    granted.sort();

    let used = commands_used_by_log_window();
    assert!(
        used.len() >= 5,
        "el escáner de InstanceLogWindow.svelte encontró muy pocos comandos: {used:?}"
    );
    assert_eq!(
        used, granted,
        "InstanceLogWindow.svelte usa comandos distintos a los de capabilities/log-window.json \
         (y a LOG_WINDOW_COMMANDS en build.rs): si cambió la pantalla, actualizá ambas listas"
    );
}
