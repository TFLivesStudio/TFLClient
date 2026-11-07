//! Catálogo firmado de TFL Selection (esquema v2): parseo estricto,
//! descubrimiento dinámico de targets, canales, compatibilidad con el
//! launcher y protección contra reemplazo por versiones anteriores
//! (`generation` monótona persistida).
//!
//! Nada acá está hardcodeado por versión de Minecraft: los targets salen
//! SIEMPRE del catálogo firmado.

use super::net::{FetchError, Transport};
use super::signatures;
use super::util::atomic_write;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const DISTRIBUTION_REPOSITORY: &str = "TFLivesStudio/TFLClientSelection";
pub const PACK_ID: &str = "tfl-selection";
pub const CATALOG_URL: &str =
    "https://raw.githubusercontent.com/TFLivesStudio/TFLClientSelection/main/catalog/current.json";
pub const CATALOG_SIG_URL: &str = "https://raw.githubusercontent.com/TFLivesStudio/TFLClientSelection/main/catalog/current.json.sig";

pub const MAX_CATALOG_BYTES: usize = 256 * 1024;
pub const MAX_SIGNATURE_BYTES: usize = 1024;
/// Cuánto se confía en la última verificación antes de volver a preguntar a
/// GitHub (evita llamadas innecesarias al abrir/cerrar la vista).
pub const FRESH_FOR: Duration = Duration::from_secs(15 * 60);

/// Loaders que ESTA versión del launcher sabe instalar y arrancar. Un target
/// con otro loader se muestra como "actualizá el launcher", nunca como
/// instalable.
pub const SUPPORTED_LOADERS: &[&str] = &["ornithe-gen2", "fabric"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Stable,
    Beta,
    Dev,
}

impl Channel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
            Self::Dev => "dev",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "stable" => Some(Self::Stable),
            "beta" => Some(Self::Beta),
            "dev" => Some(Self::Dev),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ChannelPointer {
    pub release: String,
    pub tag: String,
    pub manifest_sha256: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Channels {
    pub stable: Option<ChannelPointer>,
    pub beta: Option<ChannelPointer>,
    pub dev: Option<ChannelPointer>,
}

impl Channels {
    pub fn get(&self, channel: Channel) -> Option<&ChannelPointer> {
        match channel {
            Channel::Stable => self.stable.as_ref(),
            Channel::Beta => self.beta.as_ref(),
            Channel::Dev => self.dev.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Target {
    pub target_id: String,
    pub minecraft_version: String,
    pub loader: String,
    pub channels: Channels,
    pub display_name: Option<String>,
    pub summary: Option<String>,
    pub sort_order: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Catalog {
    pub schema_version: u32,
    pub pack_id: String,
    pub generation: u64,
    pub distribution_repository: String,
    pub targets: Vec<Target>,
}

// ── Validadores de identificadores (reflejan los patrones del schema) ───

pub fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

pub fn valid_target_id(id: &str) -> bool {
    // ^mc-[a-z0-9][a-z0-9.-]{2,79}$
    let Some(rest) = id.strip_prefix("mc-") else {
        return false;
    };
    let mut chars = rest.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return false;
    }
    let tail: Vec<char> = chars.collect();
    (2..=79).contains(&tail.len())
        && tail
            .iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '.' || *c == '-')
}

pub fn valid_release(release: &str) -> bool {
    semver::Version::parse(release).is_ok()
}

pub fn valid_tag(tag: &str) -> bool {
    // ^selection-mc-[a-z0-9.-]+-v[0-9A-Za-z.+-]+$
    tag.strip_prefix("selection-mc-").is_some_and(|rest| {
        rest.rfind("-v").is_some_and(|i| {
            let (a, b) = (&rest[..i], &rest[i + 2..]);
            !a.is_empty()
                && a.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
                && !b.is_empty()
                && b.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '-'))
        })
    })
}

/// El tag inmutable de un release sale de la fórmula publicada
/// (`selection-<targetId>-v<release>`). Exigirla impide que un puntero de
/// canal apunte a la release de OTRO target.
pub fn expected_tag(target_id: &str, release: &str) -> String {
    format!("selection-{target_id}-v{release}")
}

#[derive(Debug, PartialEq, Eq)]
pub enum CatalogError {
    Signature(String),
    Malformed(String),
    UnknownSchema(u64),
    Invalid(String),
    /// Catálogo con `generation` menor a la última verificada (replay).
    Replay {
        seen: u64,
        offered: u64,
    },
    /// Misma `generation` pero bytes distintos.
    Conflict(u64),
    Network(String),
    /// Sin red y sin catálogo verificado previo.
    Unavailable(String),
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Signature(e) => write!(f, "firma del catálogo inválida: {e}"),
            Self::Malformed(e) => write!(f, "catálogo ilegible: {e}"),
            Self::UnknownSchema(v) => write!(
                f,
                "el catálogo usa el esquema v{v}, que esta versión del launcher no entiende (actualizá TFL Client)"
            ),
            Self::Invalid(e) => write!(f, "catálogo inválido: {e}"),
            Self::Replay { seen, offered } => write!(
                f,
                "catálogo rechazado por ser anterior al ya verificado (generación {offered} < {seen})"
            ),
            Self::Conflict(g) => write!(
                f,
                "catálogo rechazado: la generación {g} ya se había verificado con otro contenido"
            ),
            Self::Network(e) => write!(f, "no se pudo consultar el catálogo: {e}"),
            Self::Unavailable(e) => write!(
                f,
                "sin conexión y sin catálogo verificado previo disponible ({e})"
            ),
        }
    }
}

impl std::error::Error for CatalogError {}

/// Parseo estricto + validaciones de contenido. NO verifica firma.
pub fn parse_catalog(raw: &[u8]) -> Result<Catalog, CatalogError> {
    if raw.len() > MAX_CATALOG_BYTES {
        return Err(CatalogError::Invalid("catálogo demasiado grande".into()));
    }
    let value: serde_json::Value =
        serde_json::from_slice(raw).map_err(|e| CatalogError::Malformed(e.to_string()))?;
    match value.get("schemaVersion").and_then(|v| v.as_u64()) {
        Some(2) => {}
        Some(other) => return Err(CatalogError::UnknownSchema(other)),
        None => return Err(CatalogError::Malformed("falta schemaVersion".into())),
    }
    let catalog: Catalog =
        serde_json::from_value(value).map_err(|e| CatalogError::Malformed(e.to_string()))?;
    validate_catalog(&catalog)?;
    Ok(catalog)
}

fn validate_catalog(c: &Catalog) -> Result<(), CatalogError> {
    let bad = |m: String| Err(CatalogError::Invalid(m));
    if c.pack_id != PACK_ID {
        return bad(format!("packId inesperado: {}", c.pack_id));
    }
    if c.distribution_repository != DISTRIBUTION_REPOSITORY {
        return bad(format!(
            "repositorio de distribución no autorizado: {}",
            c.distribution_repository
        ));
    }
    if c.generation == 0 {
        return bad("generation 0 es solo de la plantilla sin publicar".into());
    }
    if c.targets.is_empty() || c.targets.len() > 100 {
        return bad("cantidad de targets fuera de rango".into());
    }
    let mut seen = HashSet::new();
    for t in &c.targets {
        if !valid_target_id(&t.target_id) {
            return bad(format!("targetId inválido: {}", t.target_id));
        }
        if !seen.insert(t.target_id.as_str()) {
            return bad(format!("targetId duplicado: {}", t.target_id));
        }
        if t.minecraft_version.is_empty() || t.minecraft_version.len() > 40 {
            return bad(format!("minecraftVersion inválida en {}", t.target_id));
        }
        if t.loader.is_empty()
            || t.loader.len() > 40
            || !t
                .loader
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        {
            return bad(format!("loader inválido en {}", t.target_id));
        }
        if let Some(n) = &t.display_name
            && (n.is_empty() || n.chars().count() > 100)
        {
            return bad(format!("displayName fuera de rango en {}", t.target_id));
        }
        if let Some(s) = &t.summary
            && s.chars().count() > 350
        {
            return bad(format!("summary demasiado largo en {}", t.target_id));
        }
        if t.sort_order.is_some_and(|n| n > 9999) {
            return bad(format!("sortOrder fuera de rango en {}", t.target_id));
        }
        for ch in [Channel::Stable, Channel::Beta, Channel::Dev] {
            let Some(p) = t.channels.get(ch) else {
                continue;
            };
            if !valid_release(&p.release) {
                return bad(format!(
                    "release inválida en {}/{}",
                    t.target_id,
                    ch.as_str()
                ));
            }
            if !valid_tag(&p.tag) || p.tag != expected_tag(&t.target_id, &p.release) {
                return bad(format!(
                    "tag inválido o que no corresponde al target en {}/{}",
                    t.target_id,
                    ch.as_str()
                ));
            }
            if !is_sha256_hex(&p.manifest_sha256) {
                return bad(format!(
                    "manifestSha256 inválido en {}/{}",
                    t.target_id,
                    ch.as_str()
                ));
            }
        }
    }
    Ok(())
}

/// Verifica firma sobre los bytes crudos y parsea.
pub fn verify_and_parse(raw: &[u8], sig: &[u8]) -> Result<Catalog, CatalogError> {
    verify_and_parse_with(signatures::TRUSTED_KEYS, raw, sig)
}

pub fn verify_and_parse_with(
    keys: &[signatures::TrustedKey],
    raw: &[u8],
    sig: &[u8],
) -> Result<Catalog, CatalogError> {
    signatures::verify_with_keys(raw, sig, keys)
        .map_err(|e| CatalogError::Signature(e.to_string()))?;
    parse_catalog(raw)
}

/// Política anti-replay. `seen` = mayor generación verificada alguna vez.
pub fn check_generation(
    seen: Option<u64>,
    offered: u64,
    same_bytes_as_seen: bool,
) -> Result<(), CatalogError> {
    match seen {
        Some(s) if offered < s => Err(CatalogError::Replay { seen: s, offered }),
        Some(s) if offered == s && !same_bytes_as_seen => Err(CatalogError::Conflict(s)),
        _ => Ok(()),
    }
}

// ── Compatibilidad ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum Support {
    Supported,
    /// El target necesita algo que este launcher no trae.
    LauncherUpdateRequired {
        reason: String,
    },
}

pub fn target_support(t: &Target) -> Support {
    if !SUPPORTED_LOADERS.contains(&t.loader.as_str()) {
        return Support::LauncherUpdateRequired {
            reason: format!("loader «{}» no soportado por esta versión", t.loader),
        };
    }
    Support::Supported
}

/// ¿El launcher actual alcanza para un manifiesto que pide `minimum`?
pub fn launcher_meets(minimum: &str, current: &str) -> bool {
    match (
        semver::Version::parse(minimum),
        semver::Version::parse(current),
    ) {
        (Ok(min), Ok(cur)) => {
            // Una pre-release del launcher (0.15.3-beta.1) cuenta como su
            // versión base a efectos de compatibilidad mínima.
            let cur_base = semver::Version::new(cur.major, cur.minor, cur.patch);
            let min_base = semver::Version::new(min.major, min.minor, min.patch);
            cur_base >= min_base
        }
        _ => false,
    }
}

// ── Vista para la UI ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelView {
    pub channel: Channel,
    pub release: String,
    pub tag: String,
    pub manifest_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetView {
    pub target_id: String,
    pub minecraft_version: String,
    pub loader: String,
    pub display_name: String,
    pub summary: String,
    pub sort_order: u32,
    pub channels: Vec<ChannelView>,
    pub support: Support,
}

/// Solo targets con al menos una release publicada en un canal visible —
/// nunca se muestra como instalable algo sin Release. Orden: `sortOrder`,
/// luego versión de Minecraft y `targetId`.
pub fn visible_targets(catalog: &Catalog, include_dev: bool) -> Vec<TargetView> {
    let mut out: Vec<TargetView> = catalog
        .targets
        .iter()
        .filter_map(|t| {
            let mut channels = Vec::new();
            for ch in [Channel::Stable, Channel::Beta, Channel::Dev] {
                if ch == Channel::Dev && !include_dev {
                    continue;
                }
                if let Some(p) = t.channels.get(ch) {
                    channels.push(ChannelView {
                        channel: ch,
                        release: p.release.clone(),
                        tag: p.tag.clone(),
                        manifest_sha256: p.manifest_sha256.clone(),
                    });
                }
            }
            if channels.is_empty() {
                return None;
            }
            Some(TargetView {
                target_id: t.target_id.clone(),
                minecraft_version: t.minecraft_version.clone(),
                loader: t.loader.clone(),
                display_name: t
                    .display_name
                    .clone()
                    .unwrap_or_else(|| format!("TFL Selection {}", t.minecraft_version)),
                summary: t.summary.clone().unwrap_or_default(),
                sort_order: t.sort_order.unwrap_or(u32::MAX),
                channels,
                support: target_support(t),
            })
        })
        .collect();
    out.sort_by(|a, b| {
        (a.sort_order, &a.minecraft_version, &a.target_id).cmp(&(
            b.sort_order,
            &b.minecraft_version,
            &b.target_id,
        ))
    });
    out
}

/// Canal por defecto de un target: el más estable disponible.
pub fn default_channel(t: &TargetView) -> Option<Channel> {
    t.channels.iter().map(|c| c.channel).min()
}

// ── Persistencia + refresco ─────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CatalogState {
    /// Mayor generación verificada alguna vez (anti-replay).
    highest_generation: Option<u64>,
    etag: Option<String>,
    /// Segundos Unix de la última consulta exitosa (200 o 304).
    checked_at: Option<u64>,
}

pub struct CatalogStore {
    dir: PathBuf,
    keys: &'static [signatures::TrustedKey],
}

impl CatalogStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self::with_keys(dir, signatures::TRUSTED_KEYS)
    }
    /// Solo para pruebas: permite firmar catálogos sintéticos con una clave
    /// de test. En producción siempre `new` (claves embebidas).
    pub fn with_keys(dir: impl Into<PathBuf>, keys: &'static [signatures::TrustedKey]) -> Self {
        Self {
            dir: dir.into(),
            keys,
        }
    }
    fn state_path(&self) -> PathBuf {
        self.dir.join("catalog-state.json")
    }
    fn raw_path(&self) -> PathBuf {
        self.dir.join("catalog.json")
    }
    fn sig_path(&self) -> PathBuf {
        self.dir.join("catalog.json.sig")
    }

    fn load_state(&self) -> CatalogState {
        std::fs::read(self.state_path())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    fn save_state(&self, s: &CatalogState) -> std::io::Result<()> {
        atomic_write(&self.state_path(), &serde_json::to_vec_pretty(s).unwrap())
    }

    /// Snapshot local. SIEMPRE se vuelve a verificar firma y esquema: lo que
    /// hay en disco no es de confianza por estar ahí.
    fn load_verified_snapshot(&self) -> Option<(Vec<u8>, Vec<u8>, Catalog)> {
        let raw = std::fs::read(self.raw_path()).ok()?;
        let sig = std::fs::read(self.sig_path()).ok()?;
        let catalog = verify_and_parse_with(self.keys, &raw, &sig).ok()?;
        Some((raw, sig, catalog))
    }

    fn save_snapshot(&self, raw: &[u8], sig: &[u8]) -> std::io::Result<()> {
        atomic_write(&self.raw_path(), raw)?;
        atomic_write(&self.sig_path(), sig)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Origin {
    /// Consultado a GitHub en esta llamada (o dentro de la ventana fresca).
    Online,
    /// Snapshot verificado previo: sin red o catálogo en línea rechazado.
    Cached,
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub catalog: Catalog,
    pub origin: Origin,
    /// Por qué se usó el snapshot local, si aplica.
    pub degraded_reason: Option<String>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Refresca el catálogo. `force` ignora la ventana fresca (acción manual).
pub async fn refresh(
    transport: &dyn Transport,
    store: &CatalogStore,
    force: bool,
) -> Result<Snapshot, CatalogError> {
    let _ = std::fs::create_dir_all(&store.dir);
    let mut state = store.load_state();
    let cached = store.load_verified_snapshot();

    // Un snapshot válido y reciente evita tocar la red.
    if !force
        && let (Some((_, _, cat)), Some(at)) = (&cached, state.checked_at)
        && now_secs().saturating_sub(at) < FRESH_FOR.as_secs()
    {
        return Ok(Snapshot {
            catalog: cat.clone(),
            origin: Origin::Online,
            degraded_reason: None,
        });
    }

    // Solo mandar If-None-Match si el snapshot que respalda ese ETag existe.
    let etag = cached.as_ref().and(state.etag.as_deref());
    let degrade = |reason: String, cached: Option<(Vec<u8>, Vec<u8>, Catalog)>| match cached {
        Some((_, _, catalog)) => Ok(Snapshot {
            catalog,
            origin: Origin::Cached,
            degraded_reason: Some(reason),
        }),
        None => Err(CatalogError::Unavailable(reason)),
    };

    let fetched = match transport.get(CATALOG_URL, MAX_CATALOG_BYTES, etag).await {
        Ok(f) => f,
        Err(e) => return degrade(e.to_string(), cached),
    };

    if fetched.status == 304 {
        if let Some((_, _, cat)) = cached {
            state.checked_at = Some(now_secs());
            let _ = store.save_state(&state);
            return Ok(Snapshot {
                catalog: cat,
                origin: Origin::Online,
                degraded_reason: None,
            });
        }
        // 304 sin snapshot propio no debería pasar: caer a descarga plena.
        state.etag = None;
        return Box::pin(refresh_without_etag(transport, store, state)).await;
    }
    if fetched.status != 200 {
        return degrade(FetchError::Status(fetched.status).to_string(), cached);
    }

    let sig = match transport
        .get(CATALOG_SIG_URL, MAX_SIGNATURE_BYTES, None)
        .await
    {
        Ok(s) if s.status == 200 => s.body,
        Ok(s) => {
            return degrade(FetchError::Status(s.status).to_string(), cached);
        }
        Err(e) => return degrade(e.to_string(), cached),
    };

    accept_online(store, state, cached, fetched.body, sig, fetched.etag)
}

async fn refresh_without_etag(
    transport: &dyn Transport,
    store: &CatalogStore,
    state: CatalogState,
) -> Result<Snapshot, CatalogError> {
    let fetched = transport
        .get(CATALOG_URL, MAX_CATALOG_BYTES, None)
        .await
        .map_err(|e| CatalogError::Unavailable(e.to_string()))?;
    if fetched.status != 200 {
        return Err(CatalogError::Network(
            FetchError::Status(fetched.status).to_string(),
        ));
    }
    let sig = transport
        .get(CATALOG_SIG_URL, MAX_SIGNATURE_BYTES, None)
        .await
        .map_err(|e| CatalogError::Network(e.to_string()))?;
    accept_online(store, state, None, fetched.body, sig.body, fetched.etag)
}

/// Verifica un catálogo recibido y lo promueve a snapshot SOLO si pasa todo.
/// Si algo falla, el snapshot previo queda intacto y se usa como respaldo.
fn accept_online(
    store: &CatalogStore,
    mut state: CatalogState,
    cached: Option<(Vec<u8>, Vec<u8>, Catalog)>,
    raw: Vec<u8>,
    sig: Vec<u8>,
    etag: Option<String>,
) -> Result<Snapshot, CatalogError> {
    let verdict = verify_and_parse_with(store.keys, &raw, &sig).and_then(|catalog| {
        let same = cached.as_ref().is_some_and(|(r, _, _)| *r == raw);
        check_generation(state.highest_generation, catalog.generation, same)?;
        Ok(catalog)
    });
    match verdict {
        Ok(catalog) => {
            let _ = store.save_snapshot(&raw, &sig);
            state.highest_generation = Some(
                state
                    .highest_generation
                    .map_or(catalog.generation, |g| g.max(catalog.generation)),
            );
            state.etag = etag;
            state.checked_at = Some(now_secs());
            let _ = store.save_state(&state);
            Ok(Snapshot {
                catalog,
                origin: Origin::Online,
                degraded_reason: None,
            })
        }
        Err(e) => match cached {
            // El catálogo en línea es malo (firma, replay, esquema…): NUNCA
            // se acepta, pero lo verificado antes sigue sirviendo.
            Some((_, _, catalog)) => Ok(Snapshot {
                catalog,
                origin: Origin::Cached,
                degraded_reason: Some(e.to_string()),
            }),
            None => Err(e),
        },
    }
}

pub fn selection_state_dir(settings_dir: &Path) -> PathBuf {
    settings_dir.join("selection")
}
