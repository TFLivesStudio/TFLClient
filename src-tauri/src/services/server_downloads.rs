//! Resolución del jar de servidor a descargar por tipo (Vanilla/Paper/
//! Purpur) — separado de `services/launcher.rs`, que es todo sobre el lado
//! *cliente* (auth, assets, libraries). Un servidor solo necesita un único
//! jar autocontenido; no hay nada más que preparar de antemano.
use crate::core::get_json_retrying;
use crate::services::instance_manager::ServerType;
use serde::Deserialize;

pub struct ServerDownloadInfo {
    pub url: String,
    pub filename: String,
    pub build_label: String,
}

#[derive(Deserialize)]
struct MojangManifestEntry {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    url: String,
}

#[derive(Deserialize)]
struct MojangManifest {
    versions: Vec<MojangManifestEntry>,
}

#[derive(Deserialize)]
struct MojangDownloadEntry {
    url: String,
}

#[derive(Deserialize)]
struct MojangDownloads {
    server: Option<MojangDownloadEntry>,
}

#[derive(Deserialize)]
struct MojangVersionDetail {
    downloads: MojangDownloads,
}

async fn resolve_vanilla(mc_version: &str) -> Result<ServerDownloadInfo, String> {
    let manifest: MojangManifest = get_json_retrying(aqua::MOJANG_MANIFEST_URL).await?;
    let entry = manifest
        .versions
        .iter()
        .find(|v| v.id == mc_version)
        .ok_or_else(|| format!("Versión de Minecraft \"{mc_version}\" no encontrada"))?;
    let detail: MojangVersionDetail = get_json_retrying(&entry.url).await?;
    let server = detail.downloads.server.ok_or_else(|| {
        format!("Mojang no publicó server.jar para la versión {mc_version} (pasa con versiones muy viejas)")
    })?;
    Ok(ServerDownloadInfo {
        url: server.url,
        filename: "server.jar".into(),
        build_label: mc_version.into(),
    })
}

#[derive(Deserialize)]
struct PaperDownloadEntry {
    name: String,
    url: String,
}

#[derive(Deserialize)]
struct PaperDownloads {
    #[serde(rename = "server:default")]
    server_default: Option<PaperDownloadEntry>,
}

#[derive(Deserialize)]
struct PaperBuild {
    id: u32,
    downloads: PaperDownloads,
}

async fn resolve_paper(mc_version: &str) -> Result<ServerDownloadInfo, String> {
    let url = format!("https://fill.papermc.io/v3/projects/paper/versions/{mc_version}/builds");
    let builds: Vec<PaperBuild> = get_json_retrying(&url)
        .await
        .map_err(|_| format!("No hay builds de Paper para la versión {mc_version}"))?;
    // Vienen del más nuevo al más viejo (confirmado contra la API real).
    let build = builds
        .first()
        .ok_or_else(|| format!("No hay builds de Paper para la versión {mc_version}"))?;
    let dl = build
        .downloads
        .server_default
        .as_ref()
        .ok_or_else(|| "Ese build de Paper no tiene jar de servidor publicado".to_string())?;
    Ok(ServerDownloadInfo {
        url: dl.url.clone(),
        filename: dl.name.clone(),
        build_label: format!("Paper build {}", build.id),
    })
}

#[derive(Deserialize)]
struct PurpurBuilds {
    latest: String,
}

#[derive(Deserialize)]
struct PurpurVersionInfo {
    builds: PurpurBuilds,
}

async fn resolve_purpur(mc_version: &str) -> Result<ServerDownloadInfo, String> {
    let info_url = format!("https://api.purpurmc.org/v2/purpur/{mc_version}");
    let info: PurpurVersionInfo = get_json_retrying(&info_url)
        .await
        .map_err(|_| format!("No hay builds de Purpur para la versión {mc_version}"))?;
    let build = info.builds.latest;
    Ok(ServerDownloadInfo {
        url: format!("https://api.purpurmc.org/v2/purpur/{mc_version}/{build}/download"),
        filename: format!("purpur-{mc_version}-{build}.jar"),
        build_label: format!("Purpur build {build}"),
    })
}

pub async fn resolve_server_download(
    server_type: ServerType,
    mc_version: &str,
) -> Result<ServerDownloadInfo, String> {
    match server_type {
        ServerType::Vanilla => resolve_vanilla(mc_version).await,
        ServerType::Paper => resolve_paper(mc_version).await,
        ServerType::Purpur => resolve_purpur(mc_version).await,
    }
}

#[derive(Deserialize)]
struct PaperProjectVersions {
    versions: std::collections::HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct PurpurProject {
    versions: Vec<String>,
}

/// Versiones de Minecraft con build disponible para ese software de
/// servidor — no todas las versiones vanilla tienen Paper/Purpur (se
/// tardan en salir, o directamente nunca para versiones muy nuevas/viejas).
pub async fn list_server_versions(server_type: ServerType) -> Result<Vec<String>, String> {
    match server_type {
        ServerType::Vanilla => {
            let manifest: MojangManifest = get_json_retrying(aqua::MOJANG_MANIFEST_URL).await?;
            Ok(manifest
                .versions
                .into_iter()
                .filter(|v| v.kind == "release")
                .map(|v| v.id)
                .collect())
        }
        ServerType::Paper => {
            let data: PaperProjectVersions =
                get_json_retrying("https://fill.papermc.io/v3/projects/paper").await?;
            // Cada value es [versión_completa, ...release_candidates] — solo
            // la primera es la release real, las demás son pre-releases/rc
            // de esa misma versión, no queremos listarlas como si fueran
            // otra versión de Minecraft distinta.
            let mut versions: Vec<String> = data
                .versions
                .into_values()
                .filter_map(|builds| builds.into_iter().find(|v| !v.contains('-')))
                .collect();
            versions.sort_by(|a, b| b.cmp(a));
            Ok(versions)
        }
        ServerType::Purpur => {
            let data: PurpurProject = get_json_retrying("https://api.purpurmc.org/v2/purpur").await?;
            let mut versions = data.versions;
            versions.reverse();
            Ok(versions)
        }
    }
}
