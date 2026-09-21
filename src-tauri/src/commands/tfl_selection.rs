//! "TFL Selection" — modpacks listados para que el usuario los instale en
//! cualquier instancia compatible con un solo click. Dos fuentes:
//!
//! 1. Curados a mano (Modrinth) — placeholder real y funcional hasta que
//!    TFLives entregue su lista final (spec §26).
//! 2. Comunitarios (GitHub) — descubiertos en vivo cada vez que se abre
//!    el panel: se buscan repos públicos con el topic `tfl-modpack` en la
//!    org de TFLives, y de cada uno se lee `tfl-modpack.json` (si existe
//!    y es válido) para armar la entrada. Un repo con el topic pero sin
//!    manifest válido nunca aparece — el topic solo filtra candidatos,
//!    el manifest es lo que realmente decide.
use crate::core::http_client::{HTTP, get_json_retrying};
use base64::Engine;
use serde::Serialize;
use tauri::command;

const GITHUB_ORG: &str = "TFLivesStudio";
const TOPIC: &str = "tfl-modpack";
const MANIFEST_FILENAME: &str = "tfl-modpack.json";
const VALID_LOADERS: [&str; 5] = ["vanilla", "fabric", "forge", "neoforge", "quilt"];

#[derive(Debug, Serialize, Clone)]
pub struct ModpackVersion {
    pub mc_version: String,
    pub loader: String,
    pub mrpack_url: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TflSelectionEntry {
    /// Clave estable para la UI: project_id de Modrinth, o "owner/repo"
    /// para los comunitarios.
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub source: String, // "modrinth" | "community"
    /// Solo tiene sentido para source == "modrinth" — el resto de los
    /// campos ya alcanza para instalar un comunitario.
    pub project_id: Option<String>,
    /// Vacío para "modrinth" (ahí se resuelve la mejor versión en el
    /// momento de instalar, según la instancia elegida). Para
    /// "community" es la lista real de builds disponibles — puede tener
    /// una sola entrada (pack de una versión) o varias (compatible con
    /// varias versiones de Minecraft a la vez).
    pub versions: Vec<ModpackVersion>,
}

#[command]
pub async fn get_tfl_selection() -> Vec<TflSelectionEntry> {
    let mut entries = vec![
        // TODO(TFL): reemplazar por la lista real de modpacks curados de TFLives.
        TflSelectionEntry {
            id: "1KVo5zza".into(),
            title: "Fabulously Optimized".into(),
            description: "Rendimiento y calidad de vida para Fabric, sin cambiar el juego base."
                .into(),
            icon_url: None,
            source: "modrinth".into(),
            project_id: Some("1KVo5zza".into()),
            versions: Vec::new(),
        },
    ];

    match discover_community_modpacks().await {
        Ok(mut community) => entries.append(&mut community),
        Err(e) => tracing::warn!("TFL Selection: no se pudo consultar GitHub ({e}), se omite"),
    }

    entries
}

#[derive(Debug, serde::Deserialize)]
struct GhSearchResponse {
    items: Vec<GhRepoItem>,
}

#[derive(Debug, serde::Deserialize)]
struct GhRepoItem {
    full_name: String,
}

#[derive(Debug, serde::Deserialize)]
struct GhContentResponse {
    content: String,
    encoding: String,
}

async fn discover_community_modpacks() -> Result<Vec<TflSelectionEntry>, String> {
    let search_url = format!(
        "https://api.github.com/search/repositories?q=org:{GITHUB_ORG}+topic:{TOPIC}"
    );
    let search: GhSearchResponse = get_json_retrying(&search_url).await?;

    let mut out = Vec::new();
    for repo in search.items {
        match fetch_manifest(&repo.full_name).await {
            Ok(Some(entry)) => out.push(entry),
            Ok(None) => {
                tracing::debug!("TFL Selection: {} sin manifest válido, se omite", repo.full_name);
            }
            Err(e) => {
                tracing::debug!("TFL Selection: {} falló al leer manifest: {e}", repo.full_name);
            }
        }
    }
    Ok(out)
}

/// Lee y valida `tfl-modpack.json` de un repo puntual. `Ok(None)` (no
/// `Err`) para cualquier manifest ausente/inválido — un repo mal armado
/// no debe tirar abajo el resto de la lista, solo se lo salta.
pub(crate) async fn fetch_manifest(full_name: &str) -> Result<Option<TflSelectionEntry>, String> {
    let url = format!("https://api.github.com/repos/{full_name}/contents/{MANIFEST_FILENAME}");
    let res = HTTP.get(&url).send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Ok(None); // 404 = no tiene el archivo, caso normal
    }
    let content_res: GhContentResponse = res.json().await.map_err(|e| e.to_string())?;
    if content_res.encoding != "base64" {
        return Ok(None);
    }
    let cleaned: String = content_res.content.chars().filter(|c| !c.is_whitespace()).collect();
    let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(&cleaned) else {
        return Ok(None);
    };
    let Ok(raw) = serde_json::from_slice::<serde_json::Value>(&decoded) else {
        return Ok(None);
    };

    let Some(name) = raw.get("name").and_then(|v| v.as_str()) else {
        return Ok(None);
    };
    let description = raw
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let icon_url = raw
        .get("icon_url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);

    let versions = extract_versions(&raw);
    if versions.is_empty() {
        return Ok(None);
    }

    Ok(Some(TflSelectionEntry {
        id: full_name.to_string(),
        title: name.to_string(),
        description,
        icon_url,
        source: "community".into(),
        project_id: None,
        versions,
    }))
}

/// Soporta las dos formas de manifest que se le pasaron al cliente como
/// plantilla: el esquema plano de una sola versión (mc_version/loader/
/// mrpack_url al nivel raíz) y el esquema con array `versions` para
/// packs compatibles con varias versiones de Minecraft a la vez.
fn extract_versions(raw: &serde_json::Value) -> Vec<ModpackVersion> {
    let parse_one = |v: &serde_json::Value| -> Option<ModpackVersion> {
        let mc_version = v.get("mc_version")?.as_str()?.trim();
        let loader = v.get("loader")?.as_str()?.trim().to_lowercase();
        let mrpack_url = v.get("mrpack_url")?.as_str()?.trim();
        if mc_version.is_empty() || mrpack_url.is_empty() || !VALID_LOADERS.contains(&loader.as_str()) {
            return None;
        }
        Some(ModpackVersion {
            mc_version: mc_version.to_string(),
            loader,
            mrpack_url: mrpack_url.to_string(),
        })
    };

    if let Some(arr) = raw.get("versions").and_then(|v| v.as_array()) {
        return arr.iter().filter_map(parse_one).collect();
    }
    parse_one(raw).into_iter().collect()
}
