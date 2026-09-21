mod commands;
mod core;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    core::PathManager::ensure_dirs().expect("No se pudieron crear los directorios de datos");

    tauri::Builder::default()
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
            commands::instance::open_instance_folder,
            commands::instance::pick_image_file,
            commands::instance::set_instance_icon,
            commands::instance::get_instance_icon_path,
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
            commands::modpacks::search_modpacks,
            commands::modpacks::install_modpack,
            commands::modpacks::install_modpack_from_url,
            commands::modpacks::get_instance_modpacks,
            commands::modpacks::remove_modpack,
            commands::modpacks::check_modpack_updates,
            commands::tfl_selection::get_tfl_selection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
