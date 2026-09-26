mod commands;
mod core;
mod services;

use tauri::Manager;

/// Busca `--launch-instance <nombre>` en los argumentos de línea de comandos
/// (los que arma el acceso directo de escritorio, ver `commands::shortcuts`).
/// Se comparte entre el arranque en frío y el callback de single-instance
/// (segundo click en un acceso directo con la app ya abierta).
fn extract_launch_instance(args: &[String]) -> Option<String> {
    args.iter()
        .position(|a| a == "--launch-instance")
        .and_then(|i| args.get(i + 1))
        .cloned()
}

/// Ícono de bandeja + menú (Mostrar/Salir) para el modo "acceso directo":
/// la ventana principal queda oculta, esto es lo único visible del launcher
/// mientras el juego corre.
fn build_background_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::TrayIconBuilder;

    let show_i = MenuItem::with_id(app, "show", "Mostrar TFL Client", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

    let mut builder = TrayIconBuilder::new().menu(&menu).on_menu_event(|app, event| match event.id.as_ref() {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "quit" => app.exit(0),
        _ => {}
    });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

/// Lanza `instance_name` sin mostrar la ventana principal (la deja oculta,
/// solo queda el ícono de bandeja) — el flujo completo que pidió el cliente
/// para los accesos directos: doble click, el launcher no "aparece", el
/// juego sí. Si el lanzamiento falla, se muestra la ventana igual para que
/// el error no quede invisible.
fn spawn_background_launch(app: tauri::AppHandle, instance_name: String) {
    if let Err(e) = build_background_tray(&app) {
        tracing::warn!("No se pudo crear el ícono de bandeja: {e}");
    }

    // Sin ventana ni ícono que lo indique, quedarse colgado en la bandeja
    // después de que el juego cierra no tiene sentido — se cierra sola.
    {
        use tauri::Listener;
        let app_for_exit = app.clone();
        let watched_name = instance_name.clone();
        app.listen("app-event", move |event| {
            if let Ok(core::AppEvent::InstanceExited { instance, .. }) =
                serde_json::from_str::<core::AppEvent>(event.payload())
            {
                if instance == watched_name {
                    app_for_exit.exit(0);
                }
            }
        });
    }

    tauri::async_runtime::spawn(async move {
        if let Err(e) = services::launcher::launch(app.clone(), instance_name.clone(), None).await {
            tracing::error!("No se pudo lanzar \"{instance_name}\" desde el acceso directo: {e}");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    core::PathManager::ensure_dirs().expect("No se pudieron crear los directorios de datos");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(name) = extract_launch_instance(&args) {
                spawn_background_launch(app.clone(), name);
            } else if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .setup(|app| {
            core::event_bus::init(app.handle().clone());
            #[cfg(desktop)]
            {
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
            }

            // La ventana arranca oculta (tauri.conf.json `visible: false`) para
            // poder controlar acá si se muestra normal o se queda en segundo
            // plano — un acceso directo de instancia (`--launch-instance`) no
            // debe mostrar el launcher, solo lanzar el juego.
            match extract_launch_instance(&std::env::args().collect::<Vec<_>>()) {
                Some(name) => spawn_background_launch(app.handle().clone(), name),
                None => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.show()?;
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::get_device_code,
            commands::auth::authenticate_with_device_code,
            commands::auth::add_offline_account,
            commands::auth::get_current_user,
            commands::auth::get_skin_texture_url,
            commands::auth::account_allows_multiplayer,
            commands::auth::logout,
            commands::auth::get_user_list,
            commands::auth::switch_user,
            commands::auth::remove_user,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::set_quality_profile,
            commands::settings::get_recommended_ram,
            commands::settings::get_recommended_ram_for_instance,
            commands::settings::clear_temp_cache,
            commands::settings::set_custom_wallpaper,
            commands::settings::get_custom_wallpaper_path,
            commands::settings::get_java_status,
            commands::instance::create_instance,
            commands::instance::get_instances,
            commands::instance::delete_instance,
            commands::instance::rename_instance,
            commands::instance::duplicate_instance,
            commands::instance::update_instance_memory,
            commands::instance::launch,
            commands::instance::stop_running_instance,
            commands::instance::get_running_instance,
            commands::instance::get_running_instance_stats,
            commands::instance::open_instance_folder,
            commands::instance::open_external_url,
            commands::instance::pick_image_file,
            commands::instance::set_instance_icon,
            commands::instance::get_instance_icon_path,
            commands::instance::get_instance_screenshots,
            commands::instance::delete_screenshot,
            commands::instance::open_screenshots_folder,
            commands::instance::get_crash_report,
            commands::instance::list_instance_icon_presets,
            commands::instance::set_instance_icon_from_preset,
            commands::versions::get_available_versions,
            commands::loaders::get_fabric_loader,
            commands::loaders::get_quilt_loader,
            commands::loaders::get_forge_version,
            commands::loaders::get_neoforge_version,
            commands::mods::search_mods,
            commands::mods::install_mod,
            commands::mods::get_instance_mods,
            commands::mods::remove_mod,
            commands::mods::search_shaders,
            commands::mods::install_shader,
            commands::mods::get_instance_shaders,
            commands::mods::remove_shader,
            commands::mods::search_resourcepacks,
            commands::mods::install_resourcepack,
            commands::mods::get_instance_resourcepacks,
            commands::mods::remove_resourcepack,
            commands::mods::get_installed_mods_info,
            commands::mods::check_mod_updates,
            commands::mods::update_all_mods,
            commands::mods::get_mod_version_changelog,
            commands::mods::find_duplicate_mods,
            commands::mods::export_instance_as_mrpack,
            commands::mods::list_world_backups,
            commands::mods::get_mod_versions,
            commands::mods::get_shader_versions,
            commands::mods::get_resourcepack_versions,
            commands::mods::get_installed_shaders_info,
            commands::mods::get_installed_resourcepacks_info,
            commands::mods::pick_content_files,
            commands::mods::add_local_mod_files,
            commands::mods::add_local_shader_files,
            commands::mods::add_local_resourcepack_files,
            commands::modpacks::verify_instance_integrity,
            commands::modpacks::search_modpacks,
            commands::modpacks::install_modpack,
            commands::modpacks::install_modpack_from_url,
            commands::modpacks::update_community_modpack,
            commands::modpacks::get_instance_modpacks,
            commands::modpacks::remove_modpack,
            commands::modpacks::check_modpack_updates,
            commands::tfl_selection::get_tfl_selection,
            commands::mods::search_plugins,
            commands::mods::install_plugin,
            commands::mods::get_instance_plugins,
            commands::mods::remove_plugin,
            commands::mods::get_installed_plugins_info,
            commands::mods::get_plugin_versions,
            commands::mods::add_local_plugin_files,
            commands::servers::get_server_versions,
            commands::servers::get_server_connection_info,
            commands::servers::create_server_instance,
            commands::servers::launch_server,
            commands::servers::stop_server,
            commands::servers::send_server_command,
            commands::servers::is_server_running,
            commands::servers::list_server_worlds,
            commands::servers::delete_server_world,
            commands::servers::list_instance_dir,
            commands::servers::read_instance_text_file,
            commands::servers::write_instance_text_file,
            commands::servers::delete_instance_path,
            commands::servers::create_instance_dir,
            commands::playit::playit_is_linked,
            commands::playit::playit_start_claim,
            commands::playit::playit_poll_claim,
            commands::playit::playit_unlink,
            commands::playit::playit_open_claim_url,
            commands::mojang_profile::get_mojang_profile,
            commands::mojang_profile::set_skin_from_file,
            commands::mojang_profile::reset_skin,
            commands::mojang_profile::set_active_cape,
            commands::mojang_profile::hide_cape,
            commands::shortcuts::create_instance_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
