//! Actualizaciones del launcher con dos canales.
//!
//! - **Estable** (default): el endpoint de `tauri.conf.json`
//!   (`releases/latest/download/latest.json`). GitHub no cuenta las releases
//!   marcadas como "prerelease" como "latest", así que una beta nunca le llega
//!   a quien no la pidió.
//! - **Beta** (`Settings.beta_updates`): la release MÁS NUEVA entre todas las
//!   publicadas (betas y estables), tomada de la API de GitHub. Si la más nueva
//!   es una estable, también se recibe (el canal beta incluye las estables).
//!   Si la API falla (sin red, límite de pedidos), se cae al endpoint estable:
//!   quien tiene betas activadas nunca se queda sin las actualizaciones normales.
//!
//! En los dos canales la firma del paquete se verifica contra la clave pública
//! de `tauri.conf.json`, venga de donde venga el `latest.json`.
//!
//! Antes esto lo hacía el plugin desde JS; para elegir el endpoint en tiempo de
//! ejecución hay que usar el builder del lado Rust (el JS no permite cambiar
//! los endpoints).

use crate::core::get_json_retrying;
use parking_lot::Mutex;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, command};
use tauri_plugin_updater::{Update, UpdaterExt};

const RELEASES_API: &str =
    "https://api.github.com/repos/TFLivesStudio/TFLClient/releases?per_page=30";
const RELEASE_DOWNLOAD_BASE: &str = "https://github.com/TFLivesStudio/TFLClient/releases/download";
const MANIFEST_ASSET: &str = "latest.json";
const PROGRESS_EVENT: &str = "update-download-progress";
const PROGRESS_MIN_INTERVAL: Duration = Duration::from_millis(150);

/// La actualización encontrada por el último `update_check` y, si ya se bajó,
/// sus bytes. El objeto `Update` no es serializable, vive acá.
struct Pending {
    update: Update,
    bytes: Option<Vec<u8>>,
}

static PENDING: LazyLock<Mutex<Option<Pending>>> = LazyLock::new(|| Mutex::new(None));

#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub notes: Option<String>,
    /// Versión con sufijo de prerelease (`0.11.0-beta.1`).
    pub is_beta: bool,
}

#[derive(Debug, Clone, Serialize)]
struct DownloadProgress {
    downloaded: u64,
    total: Option<u64>,
}

// ── Elección de la release del canal beta ──────────────────────────────────

#[derive(Debug, Deserialize)]
struct GhAsset {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    assets: Vec<GhAsset>,
}

/// `v0.11.0-beta.1` → `0.11.0-beta.1`. Solo caracteres de versión: el tag
/// termina dentro de una URL.
fn parse_tag(tag: &str) -> Option<Version> {
    let raw = tag.strip_prefix('v').unwrap_or(tag);
    if raw.is_empty()
        || !raw
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'))
    {
        return None;
    }
    Version::parse(raw).ok()
}

/// La release publicada (no borrador, con `latest.json`) de versión más alta.
/// Devuelve su tag. Ordena por semver, no por fecha: `0.11.0` > `0.11.0-beta.2`
/// > `0.11.0-beta.1`.
fn newest_release_tag(releases: &[GhRelease]) -> Option<String> {
    releases
        .iter()
        .filter(|r| !r.draft && r.assets.iter().any(|a| a.name == MANIFEST_ASSET))
        .filter_map(|r| parse_tag(&r.tag_name).map(|v| (v, r.tag_name.clone())))
        .max_by(|(a, _), (b, _)| a.cmp(b))
        .map(|(_, tag)| tag)
}

fn manifest_url_for_tag(tag: &str) -> Option<url::Url> {
    parse_tag(tag)?;
    url::Url::parse(&format!("{RELEASE_DOWNLOAD_BASE}/{tag}/{MANIFEST_ASSET}")).ok()
}

/// URL del `latest.json` de la release más nueva (canal beta), o `None` si no
/// se pudo determinar.
async fn beta_manifest_url() -> Option<url::Url> {
    let releases: Vec<GhRelease> = match get_json_retrying(RELEASES_API).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                "Canal beta: no se pudo leer la lista de releases ({e}); se usa el canal estable"
            );
            return None;
        }
    };
    let tag = newest_release_tag(&releases)?;
    manifest_url_for_tag(&tag)
}

fn is_prerelease_version(version: &str) -> bool {
    Version::parse(version).is_ok_and(|v| !v.pre.is_empty())
}

// ── Comandos ────────────────────────────────────────────────────────────────

/// Un chequeo contra `endpoint` (o contra los de `tauri.conf.json` si es `None`).
async fn run_check(app: &AppHandle, endpoint: Option<url::Url>) -> Result<Option<Update>, String> {
    let mut builder = app.updater_builder();
    if let Some(url) = endpoint {
        builder = builder.endpoints(vec![url]).map_err(|e| e.to_string())?;
    }
    let updater = builder.build().map_err(|e| e.to_string())?;
    updater.check().await.map_err(|e| e.to_string())
}

/// Busca una actualización en el canal pedido. Si hay, la deja pendiente para
/// `update_download`/`update_install` y devuelve sus datos.
#[command]
pub async fn update_check(app: AppHandle, beta: bool) -> Result<Option<UpdateInfo>, String> {
    let beta_url = if beta {
        beta_manifest_url().await
    } else {
        None
    };
    let found = match beta_url {
        // Si el manifest de la release más nueva falla (p. ej. una beta cuyo
        // build para esta plataforma no terminó de publicarse), no se deja al
        // usuario sin las actualizaciones normales: se prueba el canal estable.
        Some(url) => match run_check(&app, Some(url)).await {
            Ok(found) => found,
            Err(e) => {
                tracing::warn!("Canal beta: falló el chequeo ({e}); se usa el canal estable");
                run_check(&app, None).await?
            }
        },
        None => run_check(&app, None).await?,
    };

    let mut pending = PENDING.lock();
    match found {
        Some(update) => {
            let info = UpdateInfo {
                is_beta: is_prerelease_version(&update.version),
                version: update.version.clone(),
                current_version: update.current_version.clone(),
                notes: update.body.clone(),
            };
            // Si el chequeo manual de Ajustes encuentra la misma versión que el
            // chequeo automático ya bajó, no se tira la descarga.
            let bytes = pending
                .take()
                .filter(|p| p.update.version == update.version)
                .and_then(|p| p.bytes);
            *pending = Some(Pending { update, bytes });
            Ok(Some(info))
        }
        None => {
            *pending = None;
            Ok(None)
        }
    }
}

/// Baja el paquete de la actualización pendiente, avisando el progreso con el
/// evento `update-download-progress`.
#[command]
pub async fn update_download(app: AppHandle) -> Result<(), String> {
    let update = PENDING
        .lock()
        .as_ref()
        .map(|p| p.update.clone())
        .ok_or("No hay ninguna actualización pendiente: buscala de nuevo")?;

    let mut downloaded: u64 = 0;
    let mut last_emit = Instant::now() - PROGRESS_MIN_INTERVAL;
    let progress_app = app.clone();
    let bytes = update
        .download(
            move |chunk, total| {
                downloaded += chunk as u64;
                // Cada chunk son unos KB: sin este tope serían miles de eventos.
                if last_emit.elapsed() >= PROGRESS_MIN_INTERVAL {
                    last_emit = Instant::now();
                    let _ =
                        progress_app.emit(PROGRESS_EVENT, DownloadProgress { downloaded, total });
                }
            },
            || {},
        )
        .await
        .map_err(|e| e.to_string())?;
    let _ = app.emit(
        PROGRESS_EVENT,
        DownloadProgress {
            downloaded: bytes.len() as u64,
            total: Some(bytes.len() as u64),
        },
    );

    if let Some(pending) = PENDING.lock().as_mut() {
        pending.bytes = Some(bytes);
    }
    Ok(())
}

/// Instala el paquete ya descargado. En Windows el instalador puede cerrar el
/// launcher por su cuenta; en el resto devuelve OK y el frontend reinicia.
#[command]
pub async fn update_install() -> Result<(), String> {
    let (update, bytes) = {
        let mut guard = PENDING.lock();
        let pending = guard
            .as_mut()
            .ok_or("No hay ninguna actualización pendiente")?;
        let bytes = pending
            .bytes
            .take()
            .ok_or("La actualización todavía no se descargó")?;
        (pending.update.clone(), bytes)
    };
    tauri::async_runtime::spawn_blocking(move || update.install(bytes))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str, draft: bool, assets: &[&str]) -> GhRelease {
        GhRelease {
            tag_name: tag.into(),
            draft,
            assets: assets
                .iter()
                .map(|n| GhAsset { name: (*n).into() })
                .collect(),
        }
    }

    #[test]
    fn picks_the_highest_version_not_the_newest_by_date() {
        // La API lista por fecha de creación: una estable vieja puede venir
        // después de una beta de una versión más alta.
        let releases = vec![
            release("v0.11.0-beta.1", false, &["latest.json"]),
            release("v0.10.17", false, &["latest.json"]),
            release("v0.10.16", false, &["latest.json"]),
        ];
        assert_eq!(
            newest_release_tag(&releases).as_deref(),
            Some("v0.11.0-beta.1")
        );
    }

    #[test]
    fn a_stable_release_beats_its_own_betas() {
        let releases = vec![
            release("v0.11.0-beta.2", false, &["latest.json"]),
            release("v0.11.0", false, &["latest.json"]),
            release("v0.11.0-beta.1", false, &["latest.json"]),
        ];
        assert_eq!(newest_release_tag(&releases).as_deref(), Some("v0.11.0"));
    }

    #[test]
    fn later_betas_win_over_earlier_ones() {
        let releases = vec![
            release("v0.11.0-beta.1", false, &["latest.json"]),
            release("v0.11.0-beta.10", false, &["latest.json"]),
            release("v0.11.0-beta.2", false, &["latest.json"]),
        ];
        assert_eq!(
            newest_release_tag(&releases).as_deref(),
            Some("v0.11.0-beta.10")
        );
    }

    #[test]
    fn drafts_and_releases_without_manifest_are_ignored() {
        let releases = vec![
            release("v0.12.0", true, &["latest.json"]),
            release("v0.11.5", false, &["tflclient.dmg"]),
            release("v0.11.0", false, &["latest.json", "x.dmg"]),
        ];
        assert_eq!(newest_release_tag(&releases).as_deref(), Some("v0.11.0"));
        assert_eq!(newest_release_tag(&[]), None);
    }

    #[test]
    fn weird_tags_are_skipped_and_never_reach_a_url() {
        let releases = vec![
            release("nightly", false, &["latest.json"]),
            release("v1.0.0/../../evil", false, &["latest.json"]),
            release("v0.10.17", false, &["latest.json"]),
        ];
        assert_eq!(newest_release_tag(&releases).as_deref(), Some("v0.10.17"));
        assert!(manifest_url_for_tag("v1.0.0/../../evil").is_none());
        assert!(manifest_url_for_tag("../x").is_none());
    }

    #[test]
    fn manifest_url_points_at_our_repo_release() {
        let url = manifest_url_for_tag("v0.11.0-beta.1").unwrap();
        assert_eq!(
            url.as_str(),
            "https://github.com/TFLivesStudio/TFLClient/releases/download/v0.11.0-beta.1/latest.json"
        );
    }

    #[test]
    fn prerelease_detection() {
        assert!(is_prerelease_version("0.11.0-beta.1"));
        assert!(!is_prerelease_version("0.10.17"));
        assert!(!is_prerelease_version("garbage"));
    }

    #[test]
    fn github_release_json_parses() {
        let json = r#"[{"tag_name":"v0.11.0-beta.1","draft":false,"prerelease":true,
            "assets":[{"name":"latest.json","size":1},{"name":"a.dmg"}],"other":1}]"#;
        let releases: Vec<GhRelease> = serde_json::from_str(json).unwrap();
        assert_eq!(
            newest_release_tag(&releases).as_deref(),
            Some("v0.11.0-beta.1")
        );
    }
}
