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

/// Instala un modpack desde una URL de .mrpack directa — usado por TFL
/// Selection para los modpacks comunitarios descubiertos en GitHub (no
/// tienen project_id de Modrinth, ya viene resuelta la URL exacta desde
/// el manifest del repo). Mismo motor que install_modpack (cubrinth),
/// solo cambia de dónde sale el archivo.
#[command]
pub async fn install_modpack_from_url(
    instance_name: String,
    source_id: String,
    mrpack_url: String,
) -> Result<InstalledModpack, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let shared_dir = PathManager::get().get_shared_dir().to_path_buf();
    let temp_dir = shared_dir
        .join("temp")
        .join(format!("mrpack-community-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(|e| e.to_string())?;
    let temp_file = temp_dir.join("pack.mrpack");

    let bytes = get_bytes_retrying(&mrpack_url).await?;
    tokio::fs::write(&temp_file, &bytes)
        .await
        .map_err(|e| e.to_string())?;

    let metadata = cubrinth::mrpack::install_mrpack(&temp_file, &instance.dir(), &shared_dir, None)
        .await
        .map_err(|e| e.to_string());

    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    let metadata = metadata?;

    let installed = InstalledModpack {
        project_id: source_id.clone(),
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
        "project_id": source_id,
        "version_id": metadata.version_id,
        "title": metadata.name,
        "installed_paths": metadata.installed_paths,
        // Guardado para poder comparar contra el manifest actual del repo
        // más adelante (check_modpack_updates) — instalado vía Modrinth
        // nunca tiene este campo, ahí no aplica.
        "mrpack_url": mrpack_url,
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

/// version_id de cada modpack instalado en la instancia que tiene una
/// versión más nueva disponible en su repo de origen — solo aplica a
/// modpacks comunitarios (los de Modrinth no guardan mrpack_url, project_id
/// ahí es un id corto sin "/"). Compara la mrpack_url guardada al instalar
/// contra la que tiene HOY el manifest del repo para la misma
/// versión de Minecraft + loader de la instancia — si cambió, hay update.
/// Falla soft en cualquier paso (repo borrado, sin red, manifest inválido):
/// ese modpack puntual simplemente no aparece como actualizable, no rompe
/// el resto de la lista.
#[derive(serde::Serialize)]
pub struct ModpackUpdateInfo {
    pub version_id: String,
    pub new_mrpack_url: String,
}

#[command]
pub async fn check_modpack_updates(instance_name: String) -> Vec<ModpackUpdateInfo> {
    let Ok(instance) = instance_manager::get_instance(&instance_name).await else {
        return Vec::new();
    };
    let instance_loader = serde_json::to_value(&instance.loader)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default();

    let manifest_dir = manifest_dir(&instance.dir());
    let Ok(mut entries) = tokio::fs::read_dir(&manifest_dir).await else {
        return Vec::new();
    };

    let mut updatable = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let Ok(raw) = tokio::fs::read_to_string(entry.path()).await else {
            continue;
        };
        let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&raw) else {
            continue;
        };
        let (Some(project_id), Some(version_id), Some(installed_url)) = (
            manifest.get("project_id").and_then(|v| v.as_str()),
            manifest.get("version_id").and_then(|v| v.as_str()),
            manifest.get("mrpack_url").and_then(|v| v.as_str()),
        ) else {
            continue;
        };
        if !project_id.contains('/') {
            continue; // Modrinth, no comunitario — no aplica acá
        }

        if let Ok(Some(current)) = super::tfl_selection::fetch_manifest(project_id).await {
            let matching_build = current
                .versions
                .iter()
                .find(|v| v.loader == instance_loader && v.mc_version == instance.mc_version);
            if let Some(build) = matching_build {
                if build.mrpack_url != installed_url {
                    updatable.push(ModpackUpdateInfo {
                        version_id: version_id.to_string(),
                        new_mrpack_url: build.mrpack_url.clone(),
                    });
                }
            }
        }
    }
    updatable
}

/// Chequea que los archivos que instaló cada modpack (según su manifest
/// local en .tfl_modpacks/) sigan existiendo — detecta si algo se
/// borró/movió por fuera del launcher después de instalar. Es honesto en lo
/// que puede detectar: archivos faltantes, no corrupción de contenido byte a
/// byte (no se guarda ningún hash de referencia para comparar eso).
#[command]
pub async fn verify_instance_integrity(instance_name: String) -> Result<Vec<String>, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let instance_dir = instance.dir();
    let manifests_dir = manifest_dir(&instance_dir);
    let Ok(mut entries) = tokio::fs::read_dir(&manifests_dir).await else {
        return Ok(Vec::new());
    };

    let mut missing = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let Ok(raw) = tokio::fs::read_to_string(entry.path()).await else {
            continue;
        };
        let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&raw) else {
            continue;
        };
        let Some(paths) = manifest.get("installed_paths").and_then(|v| v.as_array()) else {
            continue;
        };
        for p in paths {
            let Some(rel) = p.as_str() else { continue };
            let Ok(abs) = cubrinth::utils::path::safe_join(&instance_dir, rel) else {
                continue;
            };
            if !abs.exists() {
                missing.push(rel.to_string());
            }
        }
    }
    Ok(missing)
}
