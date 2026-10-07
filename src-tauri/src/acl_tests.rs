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
        "get_perf_report",
        "perf_record",
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
        "get_perf_report",
        "perf_record",
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

// ── Guardas de configuración de producción ─────────────────────────────────

#[test]
fn release_builds_do_not_ship_devtools() {
    let cargo =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let tauri_line = cargo
        .lines()
        .find(|l| l.trim_start().starts_with("tauri = {"))
        .expect("no se encontró la dependencia de tauri en Cargo.toml");
    assert!(
        !tauri_line.contains("devtools"),
        "la feature `devtools` de tauri expone el inspector en builds de release: {tauri_line}"
    );
}

#[test]
fn production_csp_has_no_unsafe_inline_scripts_or_localhost() {
    let raw =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json"))
            .unwrap();
    let conf: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let csp = &conf["app"]["security"]["csp"];
    assert!(
        csp.is_object(),
        "la CSP de producción tiene que estar definida"
    );
    let script_src = csp["script-src"].as_str().unwrap_or_default();
    assert!(
        !script_src.contains("unsafe-inline"),
        "script-src: {script_src}"
    );
    assert!(
        !script_src.contains("unsafe-eval"),
        "script-src: {script_src}"
    );
    for (directive, value) in csp.as_object().unwrap() {
        let v = value.as_str().unwrap_or_default();
        assert!(
            !v.contains("localhost:*"),
            "{directive} permite localhost: {v}"
        );
        assert!(
            !v.split_whitespace().any(|t| t == "https:" || t == "*"),
            "{directive} es demasiado abierta: {v}"
        );
    }
    assert!(
        conf["app"]["security"]["dangerousDisableAssetCspModification"].is_null(),
        "no desactivar la inyección de hashes/nonces de CSP de Tauri"
    );
    // El asset protocol no puede exponer la carpeta de ajustes (tokens, secrets).
    let scope = &conf["app"]["security"]["assetProtocol"]["scope"];
    let allow = serde_json::to_string(&scope["allow"]).unwrap();
    assert!(
        !allow.contains("**"),
        "scope de assets demasiado amplio: {allow}"
    );
    assert!(
        !allow.contains("settings"),
        "scope de assets incluye settings: {allow}"
    );
    let deny = serde_json::to_string(&scope["deny"]).unwrap();
    assert!(
        deny.contains(".tflclient/settings"),
        "falta denegar settings: {deny}"
    );
}

#[test]
fn asset_protocol_only_serves_icons_screenshots_and_wallpaper() {
    use tauri::Manager;
    let app = app();
    let scope = app.asset_protocol_scope();
    let home = std::env::var("HOME").expect("HOME no definido");
    let p = |rel: &str| Path::new(&home).join(".tflclient").join(rel);

    // Lo que la interfaz muestra de verdad.
    for ok in [
        "instances/Mi Pack/icon.png",
        "instances/Mi Pack/icon.webp",
        "instances/Mi Pack/screenshots/2026-10-07_20.00.00.png",
        "shared/appearance/custom-wallpaper.jpg",
    ] {
        assert!(scope.is_allowed(p(ok)), "debería poder servirse: {ok}");
    }

    // Todo lo demás de ~/.tflclient, en especial tokens y secrets.
    for no in [
        "settings/settings.tfl",
        "settings/playit.tfl",
        "settings/settings.cub",
        "instances/Mi Pack/instance.tfl.json",
        "instances/Mi Pack/saves/Mundo/level.dat",
        "instances/Mi Pack/options.txt",
        "instances/Mi Pack/mods/algo.jar",
        "shared/other/archivo.bin",
        "skins/algo.png",
    ] {
        assert!(!scope.is_allowed(p(no)), "NO debería poder servirse: {no}");
    }
    assert!(!scope.is_allowed(Path::new(&home).join(".ssh").join("id_rsa")));
}
