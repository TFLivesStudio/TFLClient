use crate::services::instance_manager::{InstanceData, LoaderKind};
use crate::services::{instance_manager, launcher};
use aqua::{FabricBatch, QuiltBatch};
use tauri::{AppHandle, command};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

const ICON_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

async fn resolve_loader(
    mc_version: &str,
    loader: LoaderKind,
) -> Result<(Option<String>, String), String> {
    match loader {
        LoaderKind::Vanilla => Ok((None, mc_version.to_string())),
        LoaderKind::Fabric => {
            let lv = FabricBatch::resolve_latest_loader(mc_version)
                .await
                .map_err(|e| e.to_string())?;
            let id = format!("fabric-loader-{lv}-{mc_version}");
            Ok((Some(lv), id))
        }
        LoaderKind::Quilt => {
            let lv = QuiltBatch::resolve_latest_loader(mc_version)
                .await
                .map_err(|e| e.to_string())?;
            let id = format!("quilt-loader-{lv}-{mc_version}");
            Ok((Some(lv), id))
        }
        LoaderKind::Forge => {
            let lv = crate::commands::loaders::get_forge_version(mc_version.to_string()).await?;
            let id = format!("{mc_version}-forge-{lv}");
            Ok((Some(lv), id))
        }
        LoaderKind::NeoForge => {
            let lv =
                crate::commands::loaders::get_neoforge_version(mc_version.to_string()).await?;
            let id = format!("{mc_version}-neoforge-{lv}");
            Ok((Some(lv), id))
        }
    }
}

#[command]
pub async fn create_instance(
    name: String,
    mc_version: String,
    loader: String,
) -> Result<InstanceData, String> {
    let loader_kind =
        LoaderKind::parse(&loader).ok_or_else(|| format!("Loader desconocido: {loader}"))?;
    let (loader_version, launch_version_id) = resolve_loader(&mc_version, loader_kind).await?;
    instance_manager::create_instance(name, mc_version, loader_kind, loader_version, launch_version_id)
        .await
}

#[command]
pub async fn get_instances() -> Vec<InstanceData> {
    instance_manager::list_instances().await
}

#[command]
pub async fn delete_instance(name: String) -> Result<(), String> {
    instance_manager::delete_instance(&name).await
}

#[command]
pub async fn rename_instance(old_name: String, new_name: String) -> Result<InstanceData, String> {
    instance_manager::rename_instance(&old_name, new_name).await
}

#[command]
pub async fn update_instance_memory(
    name: String,
    min_memory: Option<u32>,
    max_memory: Option<u32>,
) -> Result<InstanceData, String> {
    instance_manager::update_instance_memory(&name, min_memory, max_memory).await
}

#[command]
pub async fn launch(instance_name: String) -> Result<(), String> {
    launcher::launch(instance_name).await
}

#[command]
pub async fn open_instance_folder(app: AppHandle, name: String) -> Result<(), String> {
    let data = instance_manager::get_instance(&name).await?;
    let dir = data.dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;
    app.opener()
        .open_path(dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}

/// Abre el diálogo nativo de "elegir archivo" filtrado a imágenes. Comando
/// separado de `set_instance_icon` para que el bloqueo del diálogo (síncrono,
/// espera a que el usuario elija) no retenga nada del lado de la instancia.
#[command]
pub fn pick_image_file(app: AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .add_filter("Imagen", &ICON_EXTENSIONS)
        .blocking_pick_file()
        .and_then(|f| f.into_path().ok())
        .map(|p| p.to_string_lossy().to_string())
}

#[command]
pub async fn set_instance_icon(name: String, source_path: String) -> Result<String, String> {
    let source = std::path::PathBuf::from(&source_path);
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .filter(|e| ICON_EXTENSIONS.contains(&e.as_str()))
        .ok_or("Formato de imagen no soportado (usá PNG, JPG o WEBP)")?;

    let instance = instance_manager::get_instance(&name).await?;
    let dir = instance.dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;

    // Un ícono previo con otra extensión no debe quedar huérfano al lado
    // del nuevo — get_instance_icon_path() solo mira uno a la vez.
    for old_ext in ICON_EXTENSIONS {
        let _ = tokio::fs::remove_file(dir.join(format!("icon.{old_ext}"))).await;
    }

    let dest = dir.join(format!("icon.{ext}"));
    tokio::fs::copy(&source, &dest)
        .await
        .map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().to_string())
}

#[command]
pub async fn get_instance_icon_path(name: String) -> Option<String> {
    let instance = instance_manager::get_instance(&name).await.ok()?;
    let dir = instance.dir();
    for ext in ICON_EXTENSIONS {
        let p = dir.join(format!("icon.{ext}"));
        if p.exists() {
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
}
