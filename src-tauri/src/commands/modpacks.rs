//! Búsqueda e instalación de modpacks (.mrpack) de Modrinth sobre una
//! instancia ya existente. Reusa `cubrinth::mrpack` (motor de
//! CubicLauncher, ya trae verificación de hash + extracción segura de
//! overrides) para hacer el trabajo pesado; acá solo se resuelve qué
//! versión de Modrinth bajar y se guarda un manifest para poder
//! desinstalar después sin adivinar qué archivos eran del modpack.
use crate::core::{PathManager, get_bytes_retrying};
use crate::services::instance_manager;
use serde::{Deserialize, Serialize};
use tauri::command;

use super::mods::{ModSearchHit, best_version};

const MODRINTH_API: &str = "https://api.modrinth.com/v2";

#[derive(Debug, Deserialize)]
struct SearchResponse {
    hits: Vec<RawHit>,
}

#[derive(Debug, Deserialize)]
struct RawHit {
    project_id: String,
    title: String,
    description: String,
    icon_url: Option<String>,
    downloads: u64,
    author: String,
}

#[command]
pub async fn search_modpacks(
    query: String,
    mc_version: String,
    loader: String,
) -> Result<Vec<ModSearchHit>, String> {
    let facets = format!(
        r#"[["project_type:modpack"],["categories:{}"],["versions:{}"]]"#,
        loader.to_lowercase(),
        mc_version
    );
    let url = format!(
        "{MODRINTH_API}/search?query={}&facets={}",
        urlencoding::encode(&query),
        urlencoding::encode(&facets)
    );
    let resp: SearchResponse = crate::core::get_json_retrying(&url).await?;

    Ok(resp
        .hits
        .into_iter()
        .map(|h| ModSearchHit {
            project_id: h.project_id,
            title: h.title,
            description: h.description,
            icon_url: h.icon_url,
            downloads: h.downloads,
            author: h.author,
        })
        .collect())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledModpack {
    pub project_id: String,
    pub version_id: String,
    pub title: String,
    pub file_count: usize,
}

fn manifest_dir(instance_dir: &std::path::Path) -> std::path::PathBuf {
    instance_dir.join(".tfl_modpacks")
}

#[command]
pub async fn install_modpack(
    instance_name: String,
    project_id: String,
    mc_version: String,
    loader: String,
) -> Result<InstalledModpack, String> {
    let version = best_version(&project_id, &mc_version, &loader, true)
        .await?
        .ok_or_else(|| {
            format!("No hay versión de este modpack compatible con {mc_version}/{loader}")
        })?;

    let file = version
        .files
        .iter()
        .find(|f| f.filename.ends_with(".mrpack"))
        .or_else(|| version.files.iter().find(|f| f.primary))
        .ok_or("El modpack no tiene un archivo .mrpack para descargar")?;

    let instance = instance_manager::get_instance(&instance_name).await?;
    let shared_dir = PathManager::get().get_shared_dir().to_path_buf();
    let temp_dir = shared_dir.join("temp").join(format!("mrpack-{}", version.id));
    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(|e| e.to_string())?;
    let temp_file = temp_dir.join(&file.filename);

    let bytes = get_bytes_retrying(&file.url).await?;
    tokio::fs::write(&temp_file, &bytes)
        .await
        .map_err(|e| e.to_string())?;

    let metadata = cubrinth::mrpack::install_mrpack(&temp_file, &instance.dir(), &shared_dir, None)
        .await
        .map_err(|e| e.to_string());

    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    let metadata = metadata?;

    let installed = InstalledModpack {
        project_id: project_id.clone(),
        version_id: metadata.version_id.clone(),
        title: metadata.name.clone(),
        file_count: metadata.installed_paths.len(),
    };

    let manifest_dir = manifest_dir(&instance.dir());
    tokio::fs::create_dir_all(&manifest_dir)
        .await
        .map_err(|e| e.to_string())?;
    let manifest_path = manifest_dir.join(format!("{}.json", metadata.version_id));
    let manifest = serde_json::json!({
        "project_id": project_id,
        "version_id": metadata.version_id,
        "title": metadata.name,
        "installed_paths": metadata.installed_paths,
    });
    tokio::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(installed)
}

#[command]
pub async fn get_instance_modpacks(instance_name: String) -> Result<Vec<InstalledModpack>, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let dir = manifest_dir(&instance.dir());
    let mut out = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
        return Ok(out);
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let Ok(raw) = tokio::fs::read_to_string(entry.path()).await else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
            continue;
        };
        let (Some(project_id), Some(version_id), Some(title)) = (
            v.get("project_id").and_then(|x| x.as_str()),
            v.get("version_id").and_then(|x| x.as_str()),
            v.get("title").and_then(|x| x.as_str()),
        ) else {
            continue;
        };
        let file_count = v
            .get("installed_paths")
            .and_then(|x| x.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        out.push(InstalledModpack {
            project_id: project_id.to_string(),
            version_id: version_id.to_string(),
            title: title.to_string(),
            file_count,
        });
    }
    Ok(out)
}

#[command]
pub async fn remove_modpack(instance_name: String, version_id: String) -> Result<(), String> {
    if version_id.contains('/') || version_id.contains('\\') || version_id.contains("..") {
        return Err("Id de versión inválido".into());
    }
    let instance = instance_manager::get_instance(&instance_name).await?;
    let instance_dir = instance.dir();
    let manifest_path = manifest_dir(&instance_dir).join(format!("{version_id}.json"));

    let raw = tokio::fs::read_to_string(&manifest_path)
        .await
        .map_err(|_| "No se encontró el modpack instalado".to_string())?;
    let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let paths = v
        .get("installed_paths")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();

    for p in paths {
        let Some(rel) = p.as_str() else { continue };
        let Ok(abs) = cubrinth::utils::path::safe_join(&instance_dir, rel) else {
            continue;
        };
        let _ = tokio::fs::remove_file(&abs).await;
    }

    tokio::fs::remove_file(&manifest_path)
        .await
        .map_err(|e| e.to_string())
}
