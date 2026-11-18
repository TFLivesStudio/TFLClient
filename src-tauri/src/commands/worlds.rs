//! Per-world management, shared by client and server instances.
use crate::services::{content, instance_manager, worlds};
use serde::{Deserialize, Serialize};
use tauri::command;

async fn instance(name: &str) -> Result<instance_manager::InstanceData, String> {
    let instance = instance_manager::get_instance(name).await?;
    if matches!(
        instance.preparation.as_deref(),
        Some("preparing" | "downloading")
    ) {
        return Err("La instancia todavía se está preparando. Espera a que termine o cancela la tarea en Actividades.".into());
    }
    Ok(instance)
}

#[command]
pub async fn list_instance_worlds(instance_name: String) -> Result<Vec<worlds::WorldInfo>, String> {
    let data = instance(&instance_name).await?;
    tokio::task::spawn_blocking(move || {
        worlds::list_worlds(&data.dir(), data.server_type.is_some())
    })
    .await
    .map_err(|_| "No se pudo completar la lectura de mundos.")?
}

#[command]
pub async fn rename_instance_world(
    instance_name: String,
    world_name: String,
    display_name: String,
) -> Result<(), String> {
    let _maintenance = crate::services::maintenance::operation()?;
    let _lock = content::mutation_lock(&instance_name).await;
    worlds::ensure_stopped(&instance_name)?;
    let data = instance(&instance_name).await?;
    tokio::task::spawn_blocking(move || {
        worlds::rename_world(
            &data.dir(),
            data.server_type.is_some(),
            &world_name,
            &display_name,
        )
    })
    .await
    .map_err(|_| "No se pudo completar el cambio del nombre del mundo.")?
}

#[command]
pub async fn delete_instance_world(
    instance_name: String,
    world_name: String,
) -> Result<(), String> {
    let _maintenance = crate::services::maintenance::operation()?;
    let _lock = content::mutation_lock(&instance_name).await;
    worlds::ensure_stopped(&instance_name)?;
    let data = instance(&instance_name).await?;
    tokio::task::spawn_blocking(move || {
        worlds::delete_world(&data.dir(), data.server_type.is_some(), &world_name)
    })
    .await
    .map_err(|_| "No se pudo completar la eliminación del mundo.")?
}

#[command]
pub async fn backup_instance_world(
    instance_name: String,
    world_name: String,
) -> Result<worlds::WorldBackup, String> {
    let _maintenance = crate::services::maintenance::operation()?;
    let _lock = content::mutation_lock(&instance_name).await;
    worlds::ensure_stopped(&instance_name)?;
    let data = instance(&instance_name).await?;
    tokio::task::spawn_blocking(move || {
        worlds::backup_world(&data.dir(), data.server_type.is_some(), &world_name)
    })
    .await
    .map_err(|_| "No se pudo completar el respaldo del mundo.")?
}

#[command]
pub async fn list_instance_world_backups(
    instance_name: String,
    world_name: String,
) -> Result<Vec<worlds::WorldBackup>, String> {
    let data = instance(&instance_name).await?;
    tokio::task::spawn_blocking(move || worlds::list_backups(&data.dir(), &world_name))
        .await
        .map_err(|_| "No se pudieron consultar los respaldos del mundo.")?
}

#[command]
pub async fn restore_instance_world(
    instance_name: String,
    world_name: String,
    id: String,
) -> Result<(), String> {
    let _maintenance = crate::services::maintenance::operation()?;
    let _lock = content::mutation_lock(&instance_name).await;
    worlds::ensure_stopped(&instance_name)?;
    let data = instance(&instance_name).await?;
    tokio::task::spawn_blocking(move || {
        worlds::restore_world(&data.dir(), data.server_type.is_some(), &world_name, &id)
    })
    .await
    .map_err(|_| "No se pudo completar la restauración del mundo.")?
}

#[command]
pub async fn list_world_datapacks(
    instance_name: String,
    world_name: String,
) -> Result<Vec<worlds::Datapack>, String> {
    let data = instance(&instance_name).await?;
    tokio::task::spawn_blocking(move || {
        worlds::list_datapacks(&data.dir(), data.server_type.is_some(), &world_name)
    })
    .await
    .map_err(|_| "No se pudieron consultar los datapacks.")?
}

#[command]
pub async fn install_world_datapack(
    instance_name: String,
    world_name: String,
    path: String,
) -> Result<(), String> {
    let _maintenance = crate::services::maintenance::operation()?;
    let _lock = content::mutation_lock(&instance_name).await;
    worlds::ensure_stopped(&instance_name)?;
    let data = instance(&instance_name).await?;
    tokio::task::spawn_blocking(move || {
        worlds::install_datapack(
            &data.dir(),
            data.server_type.is_some(),
            &world_name,
            std::path::Path::new(&path),
        )
    })
    .await
    .map_err(|_| "No se pudo completar la instalación del datapack.")?
}

#[command]
pub async fn change_world_datapack(
    instance_name: String,
    world_name: String,
    id: String,
    enabled: bool,
    remove: bool,
) -> Result<(), String> {
    let _maintenance = crate::services::maintenance::operation()?;
    let _lock = content::mutation_lock(&instance_name).await;
    worlds::ensure_stopped(&instance_name)?;
    let data = instance(&instance_name).await?;
    tokio::task::spawn_blocking(move || {
        worlds::change_datapack(
            &data.dir(),
            data.server_type.is_some(),
            &world_name,
            &id,
            enabled,
            remove,
        )
    })
    .await
    .map_err(|_| "No se pudo completar el cambio del datapack.")?
}

#[derive(Deserialize, Serialize)]
pub struct DatapackHit {
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub author: String,
}
#[derive(Deserialize)]
struct SearchResult {
    hits: Vec<DatapackHit>,
}

async fn world_version(
    data: &instance_manager::InstanceData,
    name: String,
) -> Result<String, String> {
    let root = data.dir();
    let server = data.server_type.is_some();
    tokio::task::spawn_blocking(move || {
        let world = worlds::world_path(&root, server, &name)?;
        worlds::world_version(&world)?.ok_or_else(|| "El mundo no declara su versión. Instala un ZIP local y comprueba su compatibilidad antes de abrirlo.".into())
    }).await.map_err(|_| "No se pudo leer la versión del mundo.")?
}

/// The provider must declare BOTH the datapack loader and the world's exact
/// Minecraft version. There is no fallback to an unrelated client mod.
#[command]
pub async fn search_world_datapacks(
    instance_name: String,
    world_name: String,
    query: String,
) -> Result<Vec<DatapackHit>, String> {
    if query.chars().count() > 200 {
        return Err("La búsqueda es demasiado larga.".into());
    }
    let data = instance(&instance_name).await?;
    let version = world_version(&data, world_name).await?;
    let facets = serde_json::to_string(&vec![
        vec!["categories:datapack".to_string()],
        vec![format!("versions:{version}")],
    ])
    .map_err(|_| "No se pudo preparar la búsqueda.")?;
    let mut url = url::Url::parse("https://api.modrinth.com/v2/search")
        .map_err(|_| "Proveedor no válido.")?;
    url.query_pairs_mut()
        .append_pair("query", &query)
        .append_pair("facets", &facets)
        .append_pair("limit", "20");
    let result: SearchResult = crate::core::get_json_retrying(url.as_str()).await?;
    Ok(result.hits)
}

#[derive(Deserialize)]
struct PackVersion {
    game_versions: Vec<String>,
    loaders: Vec<String>,
    files: Vec<PackFile>,
}
#[derive(Deserialize)]
struct PackFile {
    url: String,
    filename: String,
    primary: bool,
    size: u64,
    hashes: PackHashes,
}
#[derive(Deserialize)]
struct PackHashes {
    sha1: String,
}

#[command]
pub async fn download_world_datapack(
    instance_name: String,
    world_name: String,
    project_id: String,
) -> Result<(), String> {
    let _maintenance = crate::services::maintenance::operation()?;
    if project_id.is_empty()
        || project_id.len() > 128
        || !project_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("Identificador de proyecto inválido.".into());
    }
    let _lock = content::mutation_lock(&instance_name).await;
    worlds::ensure_stopped(&instance_name)?;
    let data = instance(&instance_name).await?;
    let version = world_version(&data, world_name.clone()).await?;
    let mut url = url::Url::parse(&format!(
        "https://api.modrinth.com/v2/project/{project_id}/version"
    ))
    .map_err(|_| "Proveedor no válido.")?;
    url.query_pairs_mut()
        .append_pair("loaders", r#"["datapack"]"#)
        .append_pair(
            "game_versions",
            &serde_json::to_string(&vec![&version]).map_err(|_| "Versión no válida.")?,
        );
    let versions: Vec<PackVersion> = crate::core::get_json_retrying(url.as_str()).await?;
    let selected = versions
        .into_iter()
        .find(|v| {
            v.game_versions.contains(&version)
                && v.loaders.iter().any(|loader| loader == "datapack")
        })
        .ok_or("El proveedor no declara un datapack compatible con la versión de este mundo.")?;
    let file = selected
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| selected.files.first())
        .ok_or("La versión no contiene un archivo descargable.")?;
    if file.size > 128 * 1024 * 1024 || !file.filename.to_ascii_lowercase().ends_with(".zip") {
        return Err("El archivo del datapack no es un ZIP admitido de hasta 128 MiB.".into());
    }
    let staging = data
        .dir()
        .join(format!(".tfl-datapack-download-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir(&staging)
        .await
        .map_err(|_| "No se pudo preparar la descarga.")?;
    let result = async {
        content::download::download_single_file(
            &staging,
            &content::types::ContentFile {
                url: file.url.clone(),
                filename: file.filename.clone(),
                primary: true,
                sha1: Some(file.hashes.sha1.clone()),
                sha512: None,
                size: Some(file.size),
            },
        )
        .await?;
        let path = staging.join(&file.filename);
        tokio::task::spawn_blocking(move || {
            worlds::install_datapack(&data.dir(), data.server_type.is_some(), &world_name, &path)
        })
        .await
        .map_err(|_| "No se pudo completar la instalación del datapack.")?
    }
    .await;
    let _ = tokio::fs::remove_dir_all(staging).await;
    result
}
