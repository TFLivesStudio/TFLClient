//! Búsqueda e instalación de mods desde Modrinth, con resolución
//! automática de dependencias requeridas (el usuario nunca instala una
//! dependencia a mano). CurseForge queda para una próxima iteración —
//! necesita una API key propia que todavía no existe (spec §26).
use crate::core::{AppEvent, emit, get_bytes_retrying, get_json_retrying};
use crate::services::instance_manager;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::command;

const MODRINTH_API: &str = "https://api.modrinth.com/v2";
const MAX_DEPENDENCY_DEPTH: u8 = 10;

#[derive(Debug, Deserialize, Serialize)]
pub struct ModSearchHit {
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub author: String,
}

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
pub async fn search_mods(
    query: String,
    mc_version: String,
    loader: String,
) -> Result<Vec<ModSearchHit>, String> {
    let facets = format!(
        r#"[["project_type:mod"],["categories:{}"],["versions:{}"]]"#,
        loader.to_lowercase(),
        mc_version
    );
    let url = format!(
        "{MODRINTH_API}/search?query={}&facets={}",
        urlencoding::encode(&query),
        urlencoding::encode(&facets)
    );
    let resp: SearchResponse = get_json_retrying(&url).await?;

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

#[derive(Debug, Deserialize, Clone)]
struct ModrinthVersion {
    id: String,
    project_id: String,
    name: String,
    files: Vec<ModrinthFile>,
    dependencies: Vec<ModrinthDependency>,
}

#[derive(Debug, Deserialize, Clone)]
struct ModrinthFile {
    url: String,
    filename: String,
    primary: bool,
    hashes: Option<ModrinthFileHashes>,
}

#[derive(Debug, Deserialize, Clone)]
struct ModrinthFileHashes {
    sha1: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct ModrinthDependency {
    project_id: Option<String>,
    version_id: Option<String>,
    dependency_type: String,
}

async fn best_version(
    project_id: &str,
    mc_version: &str,
    loader: &str,
) -> Result<Option<ModrinthVersion>, String> {
    let loaders = format!(r#"["{}"]"#, loader.to_lowercase());
    let game_versions = format!(r#"["{mc_version}"]"#);
    let url = format!(
        "{MODRINTH_API}/project/{project_id}/version?loaders={}&game_versions={}",
        urlencoding::encode(&loaders),
        urlencoding::encode(&game_versions)
    );
    let versions: Vec<ModrinthVersion> = get_json_retrying(&url).await?;

    Ok(versions.into_iter().next())
}

async fn version_by_id(version_id: &str) -> Result<ModrinthVersion, String> {
    get_json_retrying(&format!("{MODRINTH_API}/version/{version_id}")).await
}

async fn download_mod_file(instance_name: &str, version: &ModrinthVersion) -> Result<(), String> {
    let file = version
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| version.files.first())
        .ok_or("El mod no tiene archivos para descargar")?;

    if file.filename.contains('/') || file.filename.contains('\\') || file.filename.contains("..")
    {
        return Err(format!(
            "Nombre de archivo inválido recibido de Modrinth: {}",
            file.filename
        ));
    }

    let instance = instance_manager::get_instance(instance_name).await?;
    let mods_dir = instance.dir().join("mods");
    tokio::fs::create_dir_all(&mods_dir)
        .await
        .map_err(|e| e.to_string())?;
    let dest = mods_dir.join(&file.filename);
    if dest.exists() {
        return Ok(());
    }

    let expected_sha1 = file.hashes.as_ref().and_then(|h| h.sha1.clone());

    // El motor `aqua` reintenta y valida hash para todo lo demás (Vanilla,
    // Fabric, Forge…) — acá era una escritura ciega sin checksum, así que
    // un mod truncado por un corte de red quedaba instalado como si nada.
    let mut last_err = String::new();
    for attempt in 1..=3u8 {
        let bytes = get_bytes_retrying(&file.url).await?;
        if let Some(expected) = &expected_sha1 {
            use sha1::{Digest, Sha1};
            let actual = Sha1::digest(&bytes)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            if &actual != expected {
                last_err = format!("hash SHA1 no coincide (esperado {expected}, obtenido {actual})");
                tracing::warn!(
                    "Descarga de {} corrupta en intento {attempt}/3: {last_err}",
                    file.filename
                );
                continue;
            }
        }
        tokio::fs::write(&dest, &bytes)
            .await
            .map_err(|e| e.to_string())?;
        emit(AppEvent::DownloadFinished {
            task: format!("mod:{}", file.filename),
        });
        return Ok(());
    }
    Err(format!(
        "No se pudo descargar {} de forma íntegra: {last_err}",
        file.filename
    ))
}

/// Instala un mod y, recursivamente, sus dependencias *requeridas* — el
/// usuario nunca tiene que buscar e instalar una dependencia a mano.
async fn install_recursive(
    instance_name: &str,
    mc_version: &str,
    loader: &str,
    project_id: &str,
    explicit_version_id: Option<&str>,
    depth: u8,
    seen: &mut HashSet<String>,
) -> Result<(), String> {
    if depth > MAX_DEPENDENCY_DEPTH || !seen.insert(project_id.to_string()) {
        return Ok(());
    }

    let version = if let Some(vid) = explicit_version_id {
        version_by_id(vid).await?
    } else {
        match best_version(project_id, mc_version, loader).await? {
            Some(v) => v,
            None => {
                tracing::warn!(
                    "No hay versión de {project_id} compatible con {mc_version}/{loader}, se omite"
                );
                return Ok(());
            }
        }
    };

    emit(AppEvent::DownloadProgress {
        task: format!("mod:{}", version.name),
        stage: "downloading".into(),
        item_current: 0,
        item_total: 1,
        bytes_current: 0,
        bytes_total: 0,
        current_item: Some(version.name.clone()),
    });
    download_mod_file(instance_name, &version).await?;

    for dep in &version.dependencies {
        if dep.dependency_type != "required" {
            continue;
        }
        let dep_project = match (&dep.project_id, &dep.version_id) {
            (Some(p), _) => p.clone(),
            (None, Some(vid)) => {
                let v = version_by_id(vid).await?;
                v.project_id.clone()
            }
            (None, None) => continue,
        };
        Box::pin(install_recursive(
            instance_name,
            mc_version,
            loader,
            &dep_project,
            dep.version_id.as_deref(),
            depth + 1,
            seen,
        ))
        .await?;
    }

    Ok(())
}

#[command]
pub async fn install_mod(
    instance_name: String,
    project_id: String,
    mc_version: String,
    loader: String,
) -> Result<(), String> {
    let mut seen = HashSet::new();
    install_recursive(
        &instance_name,
        &mc_version,
        &loader,
        &project_id,
        None,
        0,
        &mut seen,
    )
    .await
}

#[command]
pub async fn get_instance_mods(instance_name: String) -> Result<Vec<String>, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let mods_dir = instance.dir().join("mods");
    let mut out = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(&mods_dir).await else {
        return Ok(out);
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Some(name) = entry.file_name().to_str() {
            out.push(name.to_string());
        }
    }
    Ok(out)
}

#[command]
pub async fn remove_mod(instance_name: String, filename: String) -> Result<(), String> {
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err("Nombre de archivo inválido".into());
    }
    let instance = instance_manager::get_instance(&instance_name).await?;
    let path = instance.dir().join("mods").join(&filename);
    tokio::fs::remove_file(&path)
        .await
        .map_err(|e| e.to_string())
}
