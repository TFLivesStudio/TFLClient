use crate::services::instance_manager::{InstanceData, LoaderKind};
use crate::services::{instance_manager, launcher};
use aqua::{FabricBatch, QuiltBatch};
use tauri::command;

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
