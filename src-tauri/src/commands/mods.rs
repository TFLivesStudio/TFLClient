//! Búsqueda e instalación de mods y shaders desde Modrinth, con resolución
//! automática de dependencias requeridas (el usuario nunca instala una
//! dependencia a mano). Mods y shaders comparten la misma lógica de
//! búsqueda/descarga — solo cambia el `project_type` de Modrinth y la
//! carpeta de destino dentro de la instancia (`mods/` vs `shaderpacks/`).
//! CurseForge queda para una próxima iteración — necesita una API key
//! propia que todavía no existe (spec §26).
use crate::core::{AppEvent, emit, get_bytes_retrying, get_json_retrying, post_json_retrying};
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
    pub(crate) id: String,
    pub(crate) project_id: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) changelog: Option<String>,
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

async fn download_single_file(
    dest_dir: &std::path::Path,
    file: &ModrinthFile,
) -> Result<(), String> {
    if file.filename.contains('/') || file.filename.contains('\\') || file.filename.contains("..")
    {
        return Err(format!(
            "Nombre de archivo inválido recibido de Modrinth: {}",
            file.filename
        ));
    }

    let dest = dest_dir.join(&file.filename);
    if dest.exists() {
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
        return Ok(());
    }
    Err(format!(
        "No se pudo descargar {} de forma íntegra: {last_err}",
        file.filename
    ))
}

/// Baja TODOS los archivos que trae la versión, no solo el "primary" — la
/// mayoría de las versiones de Modrinth traen uno solo, pero algunas
/// empaquetan varios en la misma versión (el mod + una lib que necesita sí
/// o sí) y antes esto se perdía en silencio, solo se bajaba el primero.
async fn download_content_file(
    instance_name: &str,
    version: &ModrinthVersion,
    kind: ContentKind,
) -> Result<(), String> {
    if version.files.is_empty() {
        return Err("No tiene archivos para descargar".into());
    }

    let instance = instance_manager::get_instance(instance_name).await?;
    let dest_dir = instance.dir().join(kind.subdir());
    tokio::fs::create_dir_all(&dest_dir)
        .await
        .map_err(|e| e.to_string())?;

    for file in &version.files {
        download_single_file(&dest_dir, file).await?;
    }

    emit(AppEvent::DownloadFinished {
        task: format!("{}:{}", kind.subdir(), version.name),
    });
    Ok(())
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

// ── Resolución de mods instalados por hash, updates, duplicados ────────────
//
// get_instance_mods solo devolvía nombres de archivo crudos — no había
// forma de saber a qué mod de Modrinth correspondía cada .jar instalado (ni
// para mostrar el nombre real, ni para ofrecer actualizarlo). La API de
// Modrinth tiene justo un endpoint pensado para esto: resolver por hash
// SHA1 del archivo, sin importar si se instaló desde este launcher o no.

#[derive(Serialize)]
struct HashLookupBody<'a> {
    hashes: &'a [String],
    algorithm: &'a str,
}

#[derive(Serialize)]
struct HashUpdateBody<'a> {
    hashes: &'a [String],
    algorithm: &'a str,
    loaders: Vec<&'a str>,
    game_versions: Vec<&'a str>,
}

fn sha1_hex(bytes: &[u8]) -> String {
    use sha1::{Digest, Sha1};
    Sha1::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// (filename, sha1) de cada .jar en mods/ — no recursivo, los mods viven
/// todos sueltos ahí, no en subcarpetas.
async fn hash_installed_files(dir: &std::path::Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return out;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jar") {
            continue;
        }
        let Ok(bytes) = tokio::fs::read(&path).await else {
            continue;
        };
        let Some(filename) = path.file_name().and_then(|f| f.to_str()) else {
            continue;
        };
        out.push((filename.to_string(), sha1_hex(&bytes)));
    }
    out
}

fn instance_loader_str(instance: &instance_manager::InstanceData) -> String {
    serde_json::to_value(&instance.loader)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

#[derive(Debug, Serialize, Clone)]
pub struct InstalledModInfo {
    pub filename: String,
    pub project_id: Option<String>,
    pub title: Option<String>,
    pub version_id: Option<String>,
}

/// Resuelve cada mod instalado contra Modrinth por hash — funciona con
/// cualquier .jar que sea una build publicada ahí, se haya instalado desde
/// este launcher o no. Los que no resuelven (mods de otro lado, builds
/// custom) quedan con los campos en None, no rompe nada.
#[command]
pub async fn get_installed_mods_info(instance_name: String) -> Result<Vec<InstalledModInfo>, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let hashed = hash_installed_files(&instance.dir().join("mods")).await;
    if hashed.is_empty() {
        return Ok(Vec::new());
    }
    let hashes: Vec<String> = hashed.iter().map(|(_, h)| h.clone()).collect();
    let body = HashLookupBody {
        hashes: &hashes,
        algorithm: "sha1",
    };
    let resolved: std::collections::HashMap<String, ModrinthVersion> =
        post_json_retrying(&format!("{MODRINTH_API}/version_files"), &body)
            .await
            .unwrap_or_default();

    Ok(hashed
        .into_iter()
        .map(|(filename, hash)| match resolved.get(&hash) {
            Some(v) => InstalledModInfo {
                filename,
                project_id: Some(v.project_id.clone()),
                title: Some(v.name.clone()),
                version_id: Some(v.id.clone()),
            },
            None => InstalledModInfo {
                filename,
                project_id: None,
                title: None,
                version_id: None,
            },
        })
        .collect())
}

#[derive(Debug, Serialize, Clone)]
pub struct ModUpdateAvailable {
    pub filename: String,
    pub project_id: String,
    pub title: String,
    pub new_version_id: String,
}

/// Mods instalados con una versión más nueva disponible para la versión de
/// Minecraft + loader de ESTA instancia puntual — usa el endpoint de
/// Modrinth pensado justo para esto (resuelve por hash y devuelve la
/// última build compatible, sin tener que buscar mod por mod).
#[command]
pub async fn check_mod_updates(instance_name: String) -> Result<Vec<ModUpdateAvailable>, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let hashed = hash_installed_files(&instance.dir().join("mods")).await;
    if hashed.is_empty() {
        return Ok(Vec::new());
    }
    let hashes: Vec<String> = hashed.iter().map(|(_, h)| h.clone()).collect();
    let loader_str = instance_loader_str(&instance);
    let body = HashUpdateBody {
        hashes: &hashes,
        algorithm: "sha1",
        loaders: vec![loader_str.as_str()],
        game_versions: vec![instance.mc_version.as_str()],
    };
    let latest: std::collections::HashMap<String, ModrinthVersion> =
        post_json_retrying(&format!("{MODRINTH_API}/version_files/update"), &body)
            .await
            .unwrap_or_default();

    let mut out = Vec::new();
    for (filename, hash) in hashed {
        let Some(newest) = latest.get(&hash) else {
            continue;
        };
        // Modrinth devuelve la versión más nueva pase lo que pase — si
        // coincide con el hash ya instalado, en realidad no hay update.
        let already_current = newest.files.iter().any(|f| {
            f.hashes.as_ref().and_then(|h| h.sha1.as_deref()) == Some(hash.as_str())
        });
        if !already_current {
            out.push(ModUpdateAvailable {
                filename,
                project_id: newest.project_id.clone(),
                title: newest.name.clone(),
                new_version_id: newest.id.clone(),
            });
        }
    }
    Ok(out)
}

/// Actualiza todos los mods con versión nueva disponible de una — sigue de
/// largo si un mod puntual falla (red, archivo corrupto, etc), no aborta
/// el resto del lote. Devuelve cuántos se actualizaron de verdad.
#[command]
pub async fn update_all_mods(instance_name: String) -> Result<u32, String> {
    let updates = check_mod_updates(instance_name.clone()).await?;
    let mut count = 0u32;
    for update in updates {
        let Ok(version) = version_by_id(&update.new_version_id).await else {
            continue;
        };
        if download_content_file(&instance_name, &version, ContentKind::Mod)
            .await
            .is_err()
        {
            continue;
        }
        if let Ok(instance) = instance_manager::get_instance(&instance_name).await {
            let _ = tokio::fs::remove_file(instance.dir().join("mods").join(&update.filename)).await;
        }
        count += 1;
    }
    Ok(count)
}

#[command]
pub async fn get_mod_version_changelog(version_id: String) -> Result<Option<String>, String> {
    let version = version_by_id(&version_id).await?;
    Ok(version.changelog)
}

/// Grupos de archivos que resuelven al MISMO mod de Modrinth (dos builds
/// distintas del mismo mod instaladas a la vez) — el caso de "mods
/// duplicados/incompatibles" más común y bien definido; detectar choques
/// entre mods NO relacionados es un problema mucho más especulativo, fuera
/// de alcance acá.
#[command]
pub async fn find_duplicate_mods(instance_name: String) -> Result<Vec<Vec<String>>, String> {
    let infos = get_installed_mods_info(instance_name).await?;
    let mut by_project: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for info in infos {
        if let Some(pid) = info.project_id {
            by_project.entry(pid).or_default().push(info.filename);
        }
    }
    Ok(by_project.into_values().filter(|v| v.len() > 1).collect())
}
