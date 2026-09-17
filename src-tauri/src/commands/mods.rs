//! Búsqueda e instalación de mods y shaders desde Modrinth, con resolución
//! automática de dependencias requeridas (el usuario nunca instala una
//! dependencia a mano). Mods y shaders comparten la misma lógica de
//! búsqueda/descarga — solo cambia el `project_type` de Modrinth y la
//! carpeta de destino dentro de la instancia (`mods/` vs `shaderpacks/`).
//! CurseForge queda para una próxima iteración — necesita una API key
//! propia que todavía no existe (spec §26).
use crate::core::{AppEvent, emit, get_bytes_retrying, get_json_retrying};
use crate::services::instance_manager;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::command;

const MODRINTH_API: &str = "https://api.modrinth.com/v2";
const MAX_DEPENDENCY_DEPTH: u8 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentKind {
    Mod,
    Shader,
}

impl ContentKind {
    fn project_type(self) -> &'static str {
        match self {
            ContentKind::Mod => "mod",
            ContentKind::Shader => "shader",
        }
    }

    /// Carpeta de la instancia donde va el archivo instalado.
    fn subdir(self) -> &'static str {
        match self {
            ContentKind::Mod => "mods",
            ContentKind::Shader => "shaderpacks",
        }
    }

    /// Los shaders no se filtran por mod loader (Fabric/Forge/…) en
    /// Modrinth — son compatibles vía un mod aparte (Iris/OptiFine), no
    /// por sí mismos. Filtrar por loader ahí no tendría sentido y dejaría
    /// la búsqueda vacía.
    fn filters_by_loader(self) -> bool {
        matches!(self, ContentKind::Mod)
    }
}

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

async fn search_content(
    query: &str,
    mc_version: &str,
    loader: &str,
    kind: ContentKind,
) -> Result<Vec<ModSearchHit>, String> {
    let facets = if kind.filters_by_loader() {
        format!(
            r#"[["project_type:{}"],["categories:{}"],["versions:{}"]]"#,
            kind.project_type(),
            loader.to_lowercase(),
            mc_version
        )
    } else {
        format!(
            r#"[["project_type:{}"],["versions:{}"]]"#,
            kind.project_type(),
            mc_version
        )
    };
    let url = format!(
        "{MODRINTH_API}/search?query={}&facets={}",
        urlencoding::encode(query),
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

#[command]
pub async fn search_mods(
    query: String,
    mc_version: String,
    loader: String,
) -> Result<Vec<ModSearchHit>, String> {
    search_content(&query, &mc_version, &loader, ContentKind::Mod).await
}

#[command]
pub async fn search_shaders(
    query: String,
    mc_version: String,
) -> Result<Vec<ModSearchHit>, String> {
    search_content(&query, &mc_version, "", ContentKind::Shader).await
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ModrinthVersion {
    #[allow(dead_code)]
    pub(crate) id: String,
    pub(crate) project_id: String,
    pub(crate) name: String,
    pub(crate) files: Vec<ModrinthFile>,
    pub(crate) dependencies: Vec<ModrinthDependency>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ModrinthFile {
    pub(crate) url: String,
    pub(crate) filename: String,
    pub(crate) primary: bool,
    pub(crate) hashes: Option<ModrinthFileHashes>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ModrinthFileHashes {
    pub(crate) sha1: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ModrinthDependency {
    project_id: Option<String>,
    version_id: Option<String>,
    dependency_type: String,
}

pub(crate) async fn best_version(
    project_id: &str,
    mc_version: &str,
    loader: &str,
    filters_by_loader: bool,
) -> Result<Option<ModrinthVersion>, String> {
    let game_versions = format!(r#"["{mc_version}"]"#);
    let url = if filters_by_loader {
        let loaders = format!(r#"["{}"]"#, loader.to_lowercase());
        format!(
            "{MODRINTH_API}/project/{project_id}/version?loaders={}&game_versions={}",
            urlencoding::encode(&loaders),
            urlencoding::encode(&game_versions)
        )
    } else {
        format!(
            "{MODRINTH_API}/project/{project_id}/version?game_versions={}",
            urlencoding::encode(&game_versions)
        )
    };
    let versions: Vec<ModrinthVersion> = get_json_retrying(&url).await?;

    Ok(versions.into_iter().next())
}

pub(crate) async fn version_by_id(version_id: &str) -> Result<ModrinthVersion, String> {
    get_json_retrying(&format!("{MODRINTH_API}/version/{version_id}")).await
}

async fn download_content_file(
    instance_name: &str,
    version: &ModrinthVersion,
    kind: ContentKind,
) -> Result<(), String> {
    let file = version
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| version.files.first())
        .ok_or("No tiene archivos para descargar")?;

    if file.filename.contains('/') || file.filename.contains('\\') || file.filename.contains("..")
    {
        return Err(format!(
            "Nombre de archivo inválido recibido de Modrinth: {}",
            file.filename
        ));
    }

    let instance = instance_manager::get_instance(instance_name).await?;
    let dest_dir = instance.dir().join(kind.subdir());
    tokio::fs::create_dir_all(&dest_dir)
        .await
        .map_err(|e| e.to_string())?;
    let dest = dest_dir.join(&file.filename);
    if dest.exists() {
        emit(AppEvent::DownloadFinished {
            task: format!("{}:{}", kind.subdir(), version.name),
        });
        return Ok(());
    }

    let expected_sha1 = file.hashes.as_ref().and_then(|h| h.sha1.clone());

    // El motor `aqua` reintenta y valida hash para todo lo demás (Vanilla,
    // Fabric, Forge…) — acá era una escritura ciega sin checksum, así que
    // una descarga truncada por un corte de red quedaba instalada como si
    // nada.
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
            task: format!("{}:{}", kind.subdir(), version.name),
        });
        return Ok(());
    }
    Err(format!(
        "No se pudo descargar {} de forma íntegra: {last_err}",
        file.filename
    ))
}

/// Instala un mod/shader y, recursivamente, sus dependencias *requeridas*
/// — el usuario nunca tiene que buscar e instalar una dependencia a mano.
#[allow(clippy::too_many_arguments)]
async fn install_recursive(
    instance_name: &str,
    mc_version: &str,
    loader: &str,
    project_id: &str,
    explicit_version_id: Option<&str>,
    depth: u8,
    seen: &mut HashSet<String>,
    kind: ContentKind,
) -> Result<(), String> {
    if depth > MAX_DEPENDENCY_DEPTH || !seen.insert(project_id.to_string()) {
        return Ok(());
    }

    let version = if let Some(vid) = explicit_version_id {
        version_by_id(vid).await?
    } else {
        match best_version(project_id, mc_version, loader, kind.filters_by_loader()).await? {
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
        task: format!("{}:{}", kind.subdir(), version.name),
        stage: "downloading".into(),
        item_current: 0,
        item_total: 1,
        bytes_current: 0,
        bytes_total: 0,
        current_item: Some(version.name.clone()),
    });
    download_content_file(instance_name, &version, kind).await?;

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
            kind,
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
        ContentKind::Mod,
    )
    .await
}

#[command]
pub async fn install_shader(
    instance_name: String,
    project_id: String,
    mc_version: String,
) -> Result<(), String> {
    let mut seen = HashSet::new();
    install_recursive(
        &instance_name,
        &mc_version,
        "",
        &project_id,
        None,
        0,
        &mut seen,
        ContentKind::Shader,
    )
    .await
}

async fn list_dir_names(instance_name: &str, subdir: &str) -> Result<Vec<String>, String> {
    let instance = instance_manager::get_instance(instance_name).await?;
    let dir = instance.dir().join(subdir);
    let mut out = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
        return Ok(out);
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Some(name) = entry.file_name().to_str() {
            out.push(name.to_string());
        }
    }
    Ok(out)
}

async fn remove_file_in(instance_name: &str, subdir: &str, filename: &str) -> Result<(), String> {
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err("Nombre de archivo inválido".into());
    }
    let instance = instance_manager::get_instance(instance_name).await?;
    let path = instance.dir().join(subdir).join(filename);
    tokio::fs::remove_file(&path)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_instance_mods(instance_name: String) -> Result<Vec<String>, String> {
    list_dir_names(&instance_name, "mods").await
}

#[command]
pub async fn remove_mod(instance_name: String, filename: String) -> Result<(), String> {
    remove_file_in(&instance_name, "mods", &filename).await
}

#[command]
pub async fn get_instance_shaders(instance_name: String) -> Result<Vec<String>, String> {
    list_dir_names(&instance_name, "shaderpacks").await
}

#[command]
pub async fn remove_shader(instance_name: String, filename: String) -> Result<(), String> {
    remove_file_in(&instance_name, "shaderpacks", &filename).await
}
