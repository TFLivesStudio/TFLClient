//! Búsqueda e instalación de mods y shaders desde Modrinth, con resolución
//! automática de dependencias requeridas (el usuario nunca instala una
//! dependencia a mano). Mods y shaders comparten la misma lógica de
//! búsqueda/descarga — solo cambia el `project_type` de Modrinth y la
//! carpeta de destino dentro de la instancia (`mods/` vs `shaderpacks/`).
//! CurseForge queda para una próxima iteración — necesita una API key
//! propia que todavía no existe (spec §26).
use crate::core::{AppEvent, PathManager, emit, get_bytes_retrying, get_json_retrying, post_json_retrying};
use crate::services::instance_manager;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::{AppHandle, command};
use tauri_plugin_dialog::DialogExt;

const MODRINTH_API: &str = "https://api.modrinth.com/v2";
const MAX_DEPENDENCY_DEPTH: u8 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentKind {
    Mod,
    Shader,
    ResourcePack,
}

impl ContentKind {
    fn project_type(self) -> &'static str {
        match self {
            ContentKind::Mod => "mod",
            ContentKind::Shader => "shader",
            ContentKind::ResourcePack => "resourcepack",
        }
    }

    /// Carpeta de la instancia donde va el archivo instalado.
    fn subdir(self) -> &'static str {
        match self {
            ContentKind::Mod => "mods",
            ContentKind::Shader => "shaderpacks",
            ContentKind::ResourcePack => "resourcepacks",
        }
    }

    /// Los shaders y resource packs no se filtran por mod loader
    /// (Fabric/Forge/…) en Modrinth — son compatibles con vanilla o vía un
    /// mod aparte (Iris/OptiFine para shaders), no por loader propio.
    /// Filtrar por loader ahí no tendría sentido y dejaría la búsqueda
    /// vacía.
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
    categories: &[String],
    kind: ContentKind,
) -> Result<Vec<ModSearchHit>, String> {
    let mut facet_groups = vec![format!(r#"["project_type:{}"]"#, kind.project_type())];
    if kind.filters_by_loader() {
        facet_groups.push(format!(r#"["categories:{}"]"#, loader.to_lowercase()));
    }
    // Categorías elegidas por el usuario en el filtro de la pestaña
    // Descargar — se combinan entre sí con OR (cualquiera de las elegidas
    // sirve) y con AND contra el resto de los facets ya armados.
    if !categories.is_empty() {
        let cats = categories
            .iter()
            .map(|c| format!(r#""categories:{c}""#))
            .collect::<Vec<_>>()
            .join(",");
        facet_groups.push(format!("[{cats}]"));
    }
    facet_groups.push(format!(r#"["versions:{mc_version}"]"#));
    let facets = format!("[{}]", facet_groups.join(","));
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
    categories: Vec<String>,
) -> Result<Vec<ModSearchHit>, String> {
    search_content(&query, &mc_version, &loader, &categories, ContentKind::Mod).await
}

#[command]
pub async fn search_shaders(
    query: String,
    mc_version: String,
    categories: Vec<String>,
) -> Result<Vec<ModSearchHit>, String> {
    search_content(&query, &mc_version, "", &categories, ContentKind::Shader).await
}

#[command]
pub async fn search_resourcepacks(
    query: String,
    mc_version: String,
    categories: Vec<String>,
) -> Result<Vec<ModSearchHit>, String> {
    search_content(&query, &mc_version, "", &categories, ContentKind::ResourcePack).await
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ModrinthVersion {
    pub(crate) id: String,
    pub(crate) project_id: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) version_number: String,
    #[serde(default)]
    pub(crate) date_published: String,
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
    #[serde(default)]
    pub(crate) size: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ModrinthFileHashes {
    pub(crate) sha1: Option<String>,
    #[serde(default)]
    pub(crate) sha512: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ModrinthDependency {
    project_id: Option<String>,
    version_id: Option<String>,
    dependency_type: String,
}

pub(crate) async fn list_versions(
    project_id: &str,
    mc_version: &str,
    loader: &str,
    filters_by_loader: bool,
) -> Result<Vec<ModrinthVersion>, String> {
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
    get_json_retrying(&url).await
}

pub(crate) async fn best_version(
    project_id: &str,
    mc_version: &str,
    loader: &str,
    filters_by_loader: bool,
) -> Result<Option<ModrinthVersion>, String> {
    // Modrinth ya devuelve las versiones ordenadas por fecha de publicación
    // (más nueva primero) — la primera es la que se usa como "instalar
    // directo" cuando el usuario no eligió una versión puntual del dropdown.
    let versions = list_versions(project_id, mc_version, loader, filters_by_loader).await?;
    Ok(versions.into_iter().next())
}

/// Versión liviana de `ModrinthVersion` para el dropdown de "ver versiones"
/// de la pestaña Descargar — no trae `files`/`dependencies`, que no hacen
/// falta ahí y solo abultan la respuesta.
#[derive(Debug, Serialize)]
pub struct ModrinthVersionSummary {
    pub id: String,
    pub name: String,
    pub version_number: String,
    pub date_published: String,
}

async fn list_version_summaries(
    project_id: &str,
    mc_version: &str,
    loader: &str,
    filters_by_loader: bool,
) -> Result<Vec<ModrinthVersionSummary>, String> {
    let versions = list_versions(project_id, mc_version, loader, filters_by_loader).await?;
    Ok(versions
        .into_iter()
        .map(|v| ModrinthVersionSummary {
            id: v.id,
            name: v.name,
            version_number: v.version_number,
            date_published: v.date_published,
        })
        .collect())
}

#[command]
pub async fn get_mod_versions(
    project_id: String,
    mc_version: String,
    loader: String,
) -> Result<Vec<ModrinthVersionSummary>, String> {
    list_version_summaries(&project_id, &mc_version, &loader, true).await
}

#[command]
pub async fn get_shader_versions(
    project_id: String,
    mc_version: String,
) -> Result<Vec<ModrinthVersionSummary>, String> {
    list_version_summaries(&project_id, &mc_version, "", false).await
}

#[command]
pub async fn get_resourcepack_versions(
    project_id: String,
    mc_version: String,
) -> Result<Vec<ModrinthVersionSummary>, String> {
    list_version_summaries(&project_id, &mc_version, "", false).await
}

pub(crate) async fn version_by_id(version_id: &str) -> Result<ModrinthVersion, String> {
    get_json_retrying(&format!("{MODRINTH_API}/version/{version_id}")).await
}

#[derive(Debug, Deserialize)]
struct ProjectDependenciesResponse {
    projects: Vec<RawDependencyProject>,
}

#[derive(Debug, Deserialize)]
struct RawDependencyProject {
    id: String,
    client_side: String,
}

/// IDs de proyecto requeridos del lado cliente, según Modrinth — a
/// diferencia de `version.dependencies` (que es por VERSIÓN puntual y
/// puede venir vacío si el autor no lo completó en un build reciente,
/// aunque el mod sí dependa de otro en la práctica), este endpoint agrega
/// las dependencias declaradas en CUALQUIER versión del proyecto. Se usa
/// para complementar, no reemplazar, la lista de `version.dependencies`.
async fn required_dependency_project_ids(project_id: &str) -> Result<Vec<String>, String> {
    let url = format!("{MODRINTH_API}/project/{project_id}/dependencies");
    let resp: ProjectDependenciesResponse = get_json_retrying(&url).await?;
    Ok(resp
        .projects
        .into_iter()
        .filter(|p| p.client_side == "required")
        .map(|p| p.id)
        .collect())
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

    let mut had_required_dep = false;
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
        had_required_dep = true;
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

    // Complemento SOLO cuando la versión puntual no declaró ninguna
    // dependencia required utilizable (el caso real que motivó esto:
    // Create Crafts & Additions con `dependencies` vacío en su build más
    // reciente, aunque el mod sí depende de Create en la práctica). Si la
    // versión SÍ trae dependencias, se confía en eso y no se toca este
    // complemento — `required_dependency_project_ids` junta TODO lo que
    // el proyecto alguna vez declaró en CUALQUIER versión, sin contexto de
    // a cuál build aplica cada una. Corriéndolo siempre (como era antes)
    // causó un caso real: un mod que en el pasado soportaba Canvas y migró
    // a Sodium instalaba los DOS —Sodium y Canvas son motores de
    // renderizado mutuamente excluyentes— dejando la instancia rota con
    // "Incompatible mods found!" aunque el usuario nunca pidió Canvas.
    if !had_required_dep {
        if let Ok(extra_deps) = required_dependency_project_ids(project_id).await {
            for dep_project in extra_deps {
                Box::pin(install_recursive(
                    instance_name,
                    mc_version,
                    loader,
                    &dep_project,
                    None,
                    depth + 1,
                    seen,
                    kind,
                ))
                .await?;
            }
        }
    }

    Ok(())
}

// Grupos de mods mutuamente excluyentes conocidos — Modrinth no expone esta
// relación como metadata estructurada (ni Sodium ni Canvas declaran
// "incompatible" entre sí en la versión ni a nivel de proyecto, se
// verificó contra la API real), así que se mantiene a mano. Cada entrada
// es (id corto canónico, slug) de Modrinth, para no depender de cuál de
// los dos venga en `project_id` según el contexto.
const KNOWN_CONFLICT_GROUPS: &[&[(&str, &str)]] = &[
    // Motores de renderizado alternativos — cada uno reemplaza el
    // renderer de Minecraft entero, no pueden coexistir. Iris queda
    // afuera del grupo a propósito: es compatible con Sodium (de hecho lo
    // requiere), solo Canvas es el que choca.
    &[("AANobbMI", "sodium"), ("VOYxIjFI", "canvas")],
];

fn conflicts_with(project_id: &str, installed_project_id: &str) -> bool {
    KNOWN_CONFLICT_GROUPS.iter().any(|group| {
        let has = |pid: &str| group.iter().any(|(id, slug)| *id == pid || *slug == pid);
        has(project_id) && has(installed_project_id) && project_id != installed_project_id
    })
}

#[command]
pub async fn install_mod(
    instance_name: String,
    project_id: String,
    mc_version: String,
    loader: String,
    version_id: Option<String>,
) -> Result<(), String> {
    // Chequeo de conflictos conocidos ANTES de instalar — antes esto lo
    // descubría recién Fabric Loader al lanzar el juego ("Incompatible
    // mods found!"), con el usuario ya instalados los dos a mano sin
    // ningún aviso previo del launcher.
    let already_installed = get_installed_content_info(&instance_name, "mods", "jar").await?;
    if let Some(conflict) = already_installed.iter().find(|i| {
        i.project_id
            .as_deref()
            .is_some_and(|pid| conflicts_with(&project_id, pid))
    }) {
        let conflict_name = conflict.title.clone().unwrap_or_else(|| conflict.filename.clone());
        return Err(format!(
            "Este mod no es compatible con \"{conflict_name}\", que ya tenés instalado — son motores de renderizado alternativos, no pueden convivir. Sacá uno de los dos antes de instalar el otro."
        ));
    }

    let mut seen = HashSet::new();
    install_recursive(
        &instance_name,
        &mc_version,
        &loader,
        &project_id,
        version_id.as_deref(),
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
    version_id: Option<String>,
) -> Result<(), String> {
    let mut seen = HashSet::new();
    install_recursive(
        &instance_name,
        &mc_version,
        "",
        &project_id,
        version_id.as_deref(),
        0,
        &mut seen,
        ContentKind::Shader,
    )
    .await
}

#[command]
pub async fn install_resourcepack(
    instance_name: String,
    project_id: String,
    mc_version: String,
    version_id: Option<String>,
) -> Result<(), String> {
    let mut seen = HashSet::new();
    install_recursive(
        &instance_name,
        &mc_version,
        "",
        &project_id,
        version_id.as_deref(),
        0,
        &mut seen,
        ContentKind::ResourcePack,
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

#[command]
pub async fn get_instance_resourcepacks(instance_name: String) -> Result<Vec<String>, String> {
    list_dir_names(&instance_name, "resourcepacks").await
}

#[command]
pub async fn remove_resourcepack(instance_name: String, filename: String) -> Result<(), String> {
    remove_file_in(&instance_name, "resourcepacks", &filename).await
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

/// (filename, sha1) de cada archivo con la extensión dada en `dir` — no
/// recursivo, mods/shaderpacks/resourcepacks viven todos sueltos ahí, no en
/// subcarpetas.
async fn hash_installed_files(dir: &std::path::Path, ext: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return out;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some(ext) {
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
    pub icon_url: Option<String>,
    pub categories: Vec<String>,
}

/// Resuelve cada archivo instalado (mod/shader/resourcepack) contra
/// Modrinth por hash — funciona con cualquier build publicada ahí, se haya
/// instalado desde este launcher o no. Los que no resuelven (de otro lado,
/// builds custom) quedan con los campos en None, no rompe nada. Usada por
/// mods (.jar en mods/), shaders (.zip en shaderpacks/) y resourcepacks
/// (.zip en resourcepacks/) — es lo que permite marcar con check, en la
/// pestaña Descargar, los resultados que ya están instalados.
#[derive(Debug, Deserialize)]
struct RawProjectInfo {
    id: String,
    title: String,
    icon_url: Option<String>,
    #[serde(default)]
    categories: Vec<String>,
}

/// Nombre e ícono reales del MOD (ej. "Canvas Renderer"), no de la build
/// puntual instalada — algunos autores le ponen a sus versiones un `name`
/// técnico que no identifica el mod en absoluto (ej. Canvas Renderer
/// nombra sus builds "fabric-20.0.2625", que a simple vista parece ser el
/// propio Fabric Loader instalándose solo), y `version_files` no trae
/// ícono en absoluto. Un solo request en bulk (`/v2/projects`) para todos
/// los project_id resueltos a la vez, en vez de uno por mod.
async fn fetch_project_info(
    project_ids: &[String],
) -> std::collections::HashMap<String, RawProjectInfo> {
    if project_ids.is_empty() {
        return std::collections::HashMap::new();
    }
    let ids_json = serde_json::to_string(project_ids).unwrap_or_default();
    let url = format!(
        "{MODRINTH_API}/projects?ids={}",
        urlencoding::encode(&ids_json)
    );
    let projects: Vec<RawProjectInfo> = get_json_retrying(&url).await.unwrap_or_default();
    projects.into_iter().map(|p| (p.id.clone(), p)).collect()
}

async fn get_installed_content_info(
    instance_name: &str,
    subdir: &str,
    ext: &str,
) -> Result<Vec<InstalledModInfo>, String> {
    let instance = instance_manager::get_instance(instance_name).await?;
    let hashed = hash_installed_files(&instance.dir().join(subdir), ext).await;
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

    let project_ids: Vec<String> = resolved
        .values()
        .map(|v| v.project_id.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let project_info = fetch_project_info(&project_ids).await;

    Ok(hashed
        .into_iter()
        .map(|(filename, hash)| match resolved.get(&hash) {
            Some(v) => {
                let info = project_info.get(&v.project_id);
                InstalledModInfo {
                    filename,
                    project_id: Some(v.project_id.clone()),
                    // Preferí el título del proyecto (el nombre real del
                    // mod) sobre el `name` de la versión puntual — si el
                    // bulk lookup falla (red, Modrinth caído) se cae al
                    // de la versión como antes, mejor eso que nada.
                    title: Some(
                        info.map(|i| i.title.clone())
                            .unwrap_or_else(|| v.name.clone()),
                    ),
                    version_id: Some(v.id.clone()),
                    icon_url: info.and_then(|i| i.icon_url.clone()),
                    categories: info.map(|i| i.categories.clone()).unwrap_or_default(),
                }
            }
            None => InstalledModInfo {
                filename,
                project_id: None,
                title: None,
                version_id: None,
                categories: Vec::new(),
                icon_url: None,
            },
        })
        .collect())
}

#[command]
pub async fn get_installed_mods_info(instance_name: String) -> Result<Vec<InstalledModInfo>, String> {
    get_installed_content_info(&instance_name, "mods", "jar").await
}

#[command]
pub async fn get_installed_shaders_info(instance_name: String) -> Result<Vec<InstalledModInfo>, String> {
    get_installed_content_info(&instance_name, "shaderpacks", "zip").await
}

#[command]
pub async fn get_installed_resourcepacks_info(
    instance_name: String,
) -> Result<Vec<InstalledModInfo>, String> {
    get_installed_content_info(&instance_name, "resourcepacks", "zip").await
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
    let hashed = hash_installed_files(&instance.dir().join("mods"), "jar").await;
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
    if updates.is_empty() {
        return Ok(0);
    }
    // Backup del mundo antes de tocar mods — un mod nuevo puede romper un
    // mundo existente (cambios de formato de chunk, IDs de bloque
    // distintos, etc). Best-effort: si falla (sin carpeta saves/, error de
    // disco) no bloquea la actualización, solo no queda backup.
    if let Ok(instance) = instance_manager::get_instance(&instance_name).await {
        let _ = backup_world_dir(&instance.dir()).await;
    }
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

// ── Exportar instancia a .mrpack ────────────────────────────────────────────
//
// Los mods que resuelven contra Modrinth se referencian por URL (igual que
// hace cualquier .mrpack real) — el export sale liviano, no va un jar de 20MB
// adentro por cada mod. Todo lo que NO resuelve (jars de otro lado, config,
// resourcepacks, shaderpacks) se empaqueta de verdad adentro del zip, en
// overrides/ — así es como el formato .mrpack espera ese contenido.

#[derive(Serialize)]
struct MrpackIndexFile {
    path: String,
    hashes: std::collections::HashMap<String, String>,
    env: std::collections::HashMap<String, String>,
    downloads: Vec<String>,
    #[serde(rename = "fileSize")]
    file_size: u64,
}

#[derive(Serialize)]
struct MrpackIndex {
    game: String,
    #[serde(rename = "formatVersion")]
    format_version: u32,
    #[serde(rename = "versionId")]
    version_id: String,
    name: String,
    summary: Option<String>,
    files: Vec<MrpackIndexFile>,
    dependencies: std::collections::HashMap<String, String>,
}

fn loader_dependency_key(loader: instance_manager::LoaderKind) -> Option<&'static str> {
    match loader {
        instance_manager::LoaderKind::Fabric => Some("fabric-loader"),
        instance_manager::LoaderKind::Forge => Some("forge"),
        instance_manager::LoaderKind::NeoForge => Some("neoforge"),
        instance_manager::LoaderKind::Quilt => Some("quilt-loader"),
        instance_manager::LoaderKind::Vanilla => None,
    }
}

/// Agrega una carpeta entera al zip bajo `overrides/<nombre de la carpeta>`
/// — sync, corre dentro de spawn_blocking. Si la carpeta no existe (ej. la
/// instancia no tiene shaderpacks), no hace nada, no es un error.
fn add_override_dir(
    zip: &mut zip::ZipWriter<std::fs::File>,
    source: &std::path::Path,
    zip_prefix: &str,
) -> std::io::Result<()> {
    if !source.is_dir() {
        return Ok(());
    }
    for entry in walkdir_files(source) {
        let rel = entry.strip_prefix(source).unwrap_or(&entry);
        let zip_path = format!("{zip_prefix}/{}", rel.to_string_lossy().replace('\\', "/"));
        let bytes = std::fs::read(&entry)?;
        zip.start_file(zip_path, zip::write::SimpleFileOptions::default())?;
        std::io::Write::write_all(zip, &bytes)?;
    }
    Ok(())
}

/// Lista recursiva simple de archivos (no directorios) bajo `dir`.
fn walkdir_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walkdir_files(&path));
        } else {
            out.push(path);
        }
    }
    out
}

fn build_mrpack_zip(
    instance_dir: &std::path::Path,
    mods_dir: &std::path::Path,
    unresolved_mod_filenames: &std::collections::HashSet<String>,
    index: &MrpackIndex,
    export_path: &std::path::Path,
) -> std::io::Result<()> {
    if let Some(parent) = export_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(export_path)?;
    let mut zip = zip::ZipWriter::new(file);

    let index_json = serde_json::to_vec_pretty(index)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    zip.start_file("modrinth.index.json", zip::write::SimpleFileOptions::default())?;
    std::io::Write::write_all(&mut zip, &index_json)?;

    // Mods que no resolvieron contra Modrinth — van embebidos de verdad,
    // no hay URL pública de donde bajarlos después.
    for filename in unresolved_mod_filenames {
        let path = mods_dir.join(filename);
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        zip.start_file(
            format!("overrides/mods/{filename}"),
            zip::write::SimpleFileOptions::default(),
        )?;
        std::io::Write::write_all(&mut zip, &bytes)?;
    }

    for subdir in ["config", "resourcepacks", "shaderpacks"] {
        add_override_dir(&mut zip, &instance_dir.join(subdir), &format!("overrides/{subdir}"))?;
    }

    zip.finish()?;
    Ok(())
}

/// Exporta la instancia como un .mrpack real y portable — cualquier launcher
/// compatible con el formato (incluido este) puede instalarlo. Devuelve la
/// ruta del archivo generado.
#[command]
pub async fn export_instance_as_mrpack(instance_name: String) -> Result<String, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let instance_dir = instance.dir();
    let mods_dir = instance_dir.join("mods");

    let hashed = hash_installed_files(&mods_dir, "jar").await;
    let resolved: std::collections::HashMap<String, ModrinthVersion> = if hashed.is_empty() {
        Default::default()
    } else {
        let hashes: Vec<String> = hashed.iter().map(|(_, h)| h.clone()).collect();
        let body = HashLookupBody {
            hashes: &hashes,
            algorithm: "sha1",
        };
        post_json_retrying(&format!("{MODRINTH_API}/version_files"), &body)
            .await
            .unwrap_or_default()
    };

    let mut index_files = Vec::new();
    let mut unresolved = std::collections::HashSet::new();

    for (filename, hash) in &hashed {
        let Some(version) = resolved.get(hash) else {
            unresolved.insert(filename.clone());
            continue;
        };
        let Some(file) = version.files.iter().find(|f| {
            f.hashes.as_ref().and_then(|h| h.sha1.as_deref()) == Some(hash.as_str())
        }) else {
            unresolved.insert(filename.clone());
            continue;
        };
        let Some(sha1) = file.hashes.as_ref().and_then(|h| h.sha1.clone()) else {
            unresolved.insert(filename.clone());
            continue;
        };
        let mut hashes_map = std::collections::HashMap::new();
        hashes_map.insert("sha1".to_string(), sha1);
        if let Some(sha512) = file.hashes.as_ref().and_then(|h| h.sha512.clone()) {
            hashes_map.insert("sha512".to_string(), sha512);
        }
        index_files.push(MrpackIndexFile {
            path: format!("mods/{filename}"),
            hashes: hashes_map,
            env: std::collections::HashMap::from([
                ("client".to_string(), "required".to_string()),
                ("server".to_string(), "required".to_string()),
            ]),
            downloads: vec![file.url.clone()],
            file_size: file.size.unwrap_or(0),
        });
    }

    let mut dependencies = std::collections::HashMap::new();
    dependencies.insert("minecraft".to_string(), instance.mc_version.clone());
    if let (Some(loader_version), Some(key)) =
        (&instance.loader_version, loader_dependency_key(instance.loader))
    {
        dependencies.insert(key.to_string(), loader_version.clone());
    }

    let export_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let index = MrpackIndex {
        game: "minecraft".into(),
        format_version: 1,
        version_id: format!("tfl-export-{export_id}"),
        name: instance.name.clone(),
        summary: None,
        files: index_files,
        dependencies,
    };

    let safe_name: String = instance
        .name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let export_path = PathManager::get()
        .get_shared_dir()
        .join("exports")
        .join(format!("{safe_name}-{export_id}.mrpack"));

    let instance_dir_clone = instance_dir.clone();
    let mods_dir_clone = mods_dir.clone();
    let export_path_clone = export_path.clone();
    tokio::task::spawn_blocking(move || {
        build_mrpack_zip(&instance_dir_clone, &mods_dir_clone, &unresolved, &index, &export_path_clone)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(export_path.to_string_lossy().to_string())
}

// ── Backup de mundo + verificación de integridad ────────────────────────────

fn zip_dir(source: &std::path::Path, dest: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(dest)?;
    let mut zip = zip::ZipWriter::new(file);
    for entry in walkdir_files(source) {
        let rel = entry.strip_prefix(source).unwrap_or(&entry);
        let zip_path = rel.to_string_lossy().replace('\\', "/");
        let bytes = std::fs::read(&entry)?;
        zip.start_file(zip_path, zip::write::SimpleFileOptions::default())?;
        std::io::Write::write_all(&mut zip, &bytes)?;
    }
    zip.finish()?;
    Ok(())
}

/// Backup de saves/ antes de tocar mods — un mod nuevo puede romper un
/// mundo existente (formato de chunk, IDs de bloque distintos entre
/// versiones de un mod, etc). `None` si la instancia no tiene mundos
/// todavía (nada que respaldar, no es un error).
async fn backup_world_dir(instance_dir: &std::path::Path) -> Result<Option<String>, String> {
    let saves_dir = instance_dir.join("saves");
    if !saves_dir.is_dir() {
        return Ok(None);
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup_path = instance_dir.join(".tfl_backups").join(format!("saves-{ts}.zip"));
    let backup_path_clone = backup_path.clone();
    tokio::task::spawn_blocking(move || zip_dir(&saves_dir, &backup_path_clone))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    Ok(Some(backup_path.to_string_lossy().to_string()))
}

#[command]
pub async fn list_world_backups(instance_name: String) -> Result<Vec<String>, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let backups_dir = instance.dir().join(".tfl_backups");
    let Ok(mut entries) = tokio::fs::read_dir(&backups_dir).await else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Some(name) = entry.file_name().to_str() {
            out.push(name.to_string());
        }
    }
    out.sort();
    out.reverse(); // más reciente primero — el timestamp está en el nombre
    Ok(out)
}

// ── Agregar mods/shaders/resourcepacks por archivo local ───────────────────
//
// Usa el mismo diálogo nativo que `pick_image_file` (instance.rs) — por eso
// el frontend solo ofrece esto cuando `settings.native_dialog_mode ===
// 'manual'` (ver NativeDialogModePrompt.svelte y CHANGELOG_macos-dialog-
// crash-java26.txt para el porqué). Acá no hay bloqueo del lado Rust: el
// gate es una decisión de UX que vive en el frontend/Ajustes.

/// Diálogo nativo de "elegir archivos" con selección múltiple, filtrado a
/// una extensión — comando separado de la copia en sí para que el bloqueo
/// del diálogo (síncrono) no retenga nada del lado de la instancia.
#[command]
pub fn pick_content_files(app: AppHandle, extension: String, filter_label: String) -> Vec<String> {
    app.dialog()
        .file()
        .add_filter(&filter_label, &[extension.as_str()])
        .blocking_pick_files()
        .map(|files| {
            files
                .into_iter()
                .filter_map(|f| f.into_path().ok())
                .map(|p| p.to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Copia cada archivo elegido a `subdir/` dentro de la instancia, filtrando
/// por extensión y nombre de archivo seguro (mismo criterio que
/// `download_single_file`). Sigue de largo si un archivo puntual falla, no
/// aborta el resto — devuelve cuántos se copiaron de verdad.
async fn add_local_content_files(
    instance_name: &str,
    subdir: &str,
    ext: &str,
    paths: Vec<String>,
) -> Result<u32, String> {
    let instance = instance_manager::get_instance(instance_name).await?;
    let dest_dir = instance.dir().join(subdir);
    tokio::fs::create_dir_all(&dest_dir)
        .await
        .map_err(|e| e.to_string())?;

    let mut count = 0u32;
    for path_str in paths {
        let source = std::path::PathBuf::from(&path_str);
        let matches_ext = source
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case(ext))
            .unwrap_or(false);
        if !matches_ext {
            continue;
        }
        let Some(filename) = source.file_name().and_then(|f| f.to_str()) else {
            continue;
        };
        if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
            continue;
        }
        if tokio::fs::copy(&source, dest_dir.join(filename)).await.is_ok() {
            count += 1;
        }
    }
    Ok(count)
}

#[command]
pub async fn add_local_mod_files(instance_name: String, paths: Vec<String>) -> Result<u32, String> {
    add_local_content_files(&instance_name, "mods", "jar", paths).await
}

#[command]
pub async fn add_local_shader_files(instance_name: String, paths: Vec<String>) -> Result<u32, String> {
    add_local_content_files(&instance_name, "shaderpacks", "zip", paths).await
}

#[command]
pub async fn add_local_resourcepack_files(
    instance_name: String,
    paths: Vec<String>,
) -> Result<u32, String> {
    add_local_content_files(&instance_name, "resourcepacks", "zip", paths).await
}
