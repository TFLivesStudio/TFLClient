//! Manifiesto v2 de una release de TFL Selection: parseo estricto y
//! validación de contenido ANTES de tocar el disco o la red.
//!
//! Nada de lo que viene en el manifiesto se usa como ruta o URL sin pasar
//! por un validador de lista blanca. El manifiesto está firmado, pero
//! una firma válida no excusa de validar: defensa en profundidad.

use super::catalog::{
    self, ChannelPointer, DISTRIBUTION_REPOSITORY, PACK_ID, Target, is_sha256_hex, launcher_meets,
};
use super::signatures;
use super::util::sha256_hex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const MAX_MANIFEST_BYTES: usize = 2 * 1024 * 1024;
const MAX_FILE_BYTES: u64 = 2_147_483_647;
/// Tope de lo que una release puede pedir descargar en total.
const MAX_TOTAL_BYTES: u64 = 4 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MinecraftSpec {
    pub version: String,
    pub loader: String,
    pub java_major: u32,
    pub loader_version: Option<String>,
    pub bootstrap: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Distribution {
    pub repository: String,
    pub tag: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AssetRef {
    pub asset_name: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Origin {
    pub project: String,
    pub project_version: String,
    pub source_repository: String,
    pub source_revision: String,
    pub license_spdx: String,
    pub license_evidence_url: String,
    pub redistribution_basis: String,
    pub permission_evidence_url: Option<String>,
    pub corresponding_source_asset: Option<AssetRef>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Download {
    pub kind: String,
    pub repository: String,
    pub tag: String,
    pub asset: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ManifestFile {
    pub id: String,
    pub kind: String,
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub asset_name: String,
    pub install_policy: String,
    pub platforms: Option<Vec<String>>,
    pub architectures: Option<Vec<String>>,
    pub origin: Origin,
    pub download: Option<Download>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: u32,
    pub pack_id: String,
    pub target_id: String,
    pub release: String,
    pub status: String,
    pub selection_mode: String,
    pub published_at: Option<String>,
    pub minimum_launcher_version: Option<String>,
    pub minecraft: MinecraftSpec,
    pub distribution: Option<Distribution>,
    pub notices_asset: Option<AssetRef>,
    pub files: Vec<ManifestFile>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ManifestError {
    HashMismatch,
    Signature(String),
    Malformed(String),
    Invalid(String),
    /// Hace falta un launcher más nuevo (versión mínima, loader, etc.).
    LauncherUpdateRequired(String),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HashMismatch => write!(
                f,
                "el manifiesto descargado no coincide con el hash anunciado por el catálogo firmado"
            ),
            Self::Signature(e) => write!(f, "firma del manifiesto inválida: {e}"),
            Self::Malformed(e) => write!(f, "manifiesto ilegible: {e}"),
            Self::Invalid(e) => write!(f, "manifiesto inválido: {e}"),
            Self::LauncherUpdateRequired(e) => {
                write!(
                    f,
                    "hace falta actualizar TFL Client para instalar esto: {e}"
                )
            }
        }
    }
}

impl std::error::Error for ManifestError {}

// ── Plataforma ──────────────────────────────────────────────────────────

pub fn current_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}

pub fn current_arch() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x64"
    }
}

// ── Validadores de lista blanca ─────────────────────────────────────────

fn safe_segment_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '+' | '(' | ')' | '@' | ' ')
}

/// Ruta relativa segura dentro de la instancia: sin absolutas, `..`,
/// separadores de Windows, ADS/unidades (`:`), segmentos vacíos ni nombres
/// que Windows trunca (terminan en punto o espacio).
pub fn safe_relative_path(path: &str) -> bool {
    if path.is_empty() || path.len() > 300 || path.starts_with('/') || path.contains('\\') {
        return false;
    }
    path.split('/').all(|seg| {
        !seg.is_empty()
            && seg != "."
            && seg != ".."
            && !seg.starts_with('.')
            && !seg.ends_with('.')
            && !seg.ends_with(' ')
            && !seg.starts_with(' ')
            && seg.chars().all(safe_segment_char)
    })
}

fn valid_repo(repo: &str) -> bool {
    let mut it = repo.split('/');
    let (Some(owner), Some(name), None) = (it.next(), it.next(), it.next()) else {
        return false;
    };
    let ok = |s: &str| {
        !s.is_empty()
            && s != "."
            && s != ".."
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
    };
    ok(owner) && ok(name)
}

fn valid_download_tag(tag: &str) -> bool {
    // ^[A-Za-z0-9][A-Za-z0-9_.+-]{0,119}$
    let mut chars = tag.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphanumeric())
        && tag.len() <= 120
        && tag
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '+' | '-'))
}

fn valid_download_asset(asset: &str) -> bool {
    // ^[A-Za-z0-9][A-Za-z0-9_.+-]{0,199}$
    asset
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphanumeric())
        && asset.len() <= 200
        && asset
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '+' | '-'))
}

fn valid_notice_asset_name(n: &str) -> bool {
    // ^[a-zA-Z0-9][a-zA-Z0-9._-]{1,199}$
    n.len() >= 2
        && n.len() <= 200
        && n.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && n.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

fn https(url: &str) -> bool {
    url.starts_with("https://") && url.len() <= 500
}

fn valid_asset_ref(a: &AssetRef) -> bool {
    valid_notice_asset_name(&a.asset_name)
        && is_sha256_hex(&a.sha256)
        && (1..=MAX_FILE_BYTES).contains(&a.bytes)
}

// ── URLs ────────────────────────────────────────────────────────────────

pub fn manifest_url(repository: &str, tag: &str) -> String {
    format!("https://github.com/{repository}/releases/download/{tag}/manifest.json")
}

pub fn manifest_sig_url(repository: &str, tag: &str) -> String {
    format!("https://github.com/{repository}/releases/download/{tag}/manifest.json.sig")
}

/// URL de descarga de un archivo del manifiesto. Solo se arma con campos ya
/// validados; nunca se usa `latest` ni una URL arbitraria.
pub fn file_download_url(m: &Manifest, f: &ManifestFile) -> Result<String, ManifestError> {
    let dist = m
        .distribution
        .as_ref()
        .ok_or_else(|| ManifestError::Invalid("sin distribution".into()))?;
    Ok(match &f.download {
        Some(d) => format!(
            "https://github.com/{}/releases/download/{}/{}",
            d.repository, d.tag, d.asset
        ),
        None => format!(
            "https://github.com/{}/releases/download/{}/{}",
            dist.repository, dist.tag, f.asset_name
        ),
    })
}

// ── Carga y validación ──────────────────────────────────────────────────

/// Lo que el catálogo firmado afirma de la release que se va a instalar.
pub struct Expected<'a> {
    pub target: &'a Target,
    pub pointer: &'a ChannelPointer,
}

/// Verifica hash anunciado → firma → esquema → contenido, en ese orden.
pub fn load_manifest(
    raw: &[u8],
    sig: &[u8],
    expected: &Expected<'_>,
    launcher_version: &str,
    keys: &[signatures::TrustedKey],
) -> Result<Manifest, ManifestError> {
    if raw.len() > MAX_MANIFEST_BYTES {
        return Err(ManifestError::Invalid("manifiesto demasiado grande".into()));
    }
    if sha256_hex(raw) != expected.pointer.manifest_sha256 {
        return Err(ManifestError::HashMismatch);
    }
    signatures::verify_with_keys(raw, sig, keys)
        .map_err(|e| ManifestError::Signature(e.to_string()))?;
    let value: serde_json::Value =
        serde_json::from_slice(raw).map_err(|e| ManifestError::Malformed(e.to_string()))?;
    if value.get("schemaVersion").and_then(|v| v.as_u64()) != Some(2) {
        return Err(ManifestError::LauncherUpdateRequired(
            "el manifiesto usa un esquema que este launcher no entiende".into(),
        ));
    }
    let manifest: Manifest =
        serde_json::from_value(value).map_err(|e| ManifestError::Malformed(e.to_string()))?;
    validate_manifest(&manifest, expected, launcher_version)?;
    Ok(manifest)
}

pub fn validate_manifest(
    m: &Manifest,
    expected: &Expected<'_>,
    launcher_version: &str,
) -> Result<(), ManifestError> {
    let bad = |msg: String| Err(ManifestError::Invalid(msg));
    if m.pack_id != PACK_ID {
        return bad(format!("packId inesperado: {}", m.pack_id));
    }
    if m.status != "published" {
        return bad("la release no está publicada (draft)".into());
    }
    if m.selection_mode != "locked" {
        return bad("selectionMode debe ser «locked»".into());
    }
    // Identidad: lo que dice el manifiesto debe coincidir con lo que el
    // catálogo firmado anunció (no se acepta un manifiesto de OTRO target
    // o release aunque esté bien firmado).
    if m.target_id != expected.target.target_id {
        return bad("el manifiesto es de otro target".into());
    }
    if m.release != expected.pointer.release {
        return bad("el manifiesto es de otra release".into());
    }
    if m.minecraft.version != expected.target.minecraft_version
        || m.minecraft.loader != expected.target.loader
    {
        return bad("Minecraft/loader del manifiesto no coinciden con el catálogo".into());
    }
    let Some(dist) = &m.distribution else {
        return bad("falta distribution".into());
    };
    if dist.repository != DISTRIBUTION_REPOSITORY || dist.tag != expected.pointer.tag {
        return bad("distribution no coincide con el repositorio/tag autorizados".into());
    }

    // Capacidades del launcher.
    match &m.minimum_launcher_version {
        Some(min) if catalog::valid_release(min) => {
            if !launcher_meets(min, launcher_version) {
                return Err(ManifestError::LauncherUpdateRequired(format!(
                    "requiere TFL Client {min} o superior (tenés {launcher_version})"
                )));
            }
        }
        _ => return bad("falta minimumLauncherVersion válido".into()),
    }
    if !catalog::SUPPORTED_LOADERS.contains(&m.minecraft.loader.as_str()) {
        return Err(ManifestError::LauncherUpdateRequired(format!(
            "loader «{}» no soportado",
            m.minecraft.loader
        )));
    }
    let bootstrap_ok = matches!(
        (
            m.minecraft.loader.as_str(),
            m.minecraft.bootstrap.as_deref()
        ),
        ("ornithe-gen2", Some("launcher-pinned-ornithe"))
            | ("fabric", Some("launcher-pinned-fabric"))
    );
    if !bootstrap_ok {
        return Err(ManifestError::LauncherUpdateRequired(
            "estrategia de bootstrap desconocida para este launcher".into(),
        ));
    }
    match &m.minecraft.loader_version {
        Some(v)
            if !v.is_empty()
                && v.len() <= 100
                && v.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+' | '_')) => {}
        _ => {
            return bad(
                "loaderVersion ausente o inválido (se instala EXACTA, nunca la última)".into(),
            );
        }
    }
    if !(8..=30).contains(&m.minecraft.java_major) {
        return bad("javaMajor fuera de rango".into());
    }
    if m.minecraft.version.is_empty() || m.minecraft.version.len() > 40 {
        return bad("versión de Minecraft inválida".into());
    }

    match &m.notices_asset {
        Some(n) if valid_asset_ref(n) => {}
        _ => return bad("falta noticesAsset válido".into()),
    }

    if m.files.is_empty() || m.files.len() > 5000 {
        return bad("cantidad de archivos fuera de rango".into());
    }
    let mut ids = HashSet::new();
    let mut paths = HashSet::new();
    let mut total = 0u64;
    for f in &m.files {
        let id_ok = f.id.len() >= 2
            && f.id.len() <= 120
            && f.id
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            && f.id.chars().all(|c| {
                c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-')
            });
        if !id_ok {
            return bad(format!("id de archivo inválido: {:?}", f.id));
        }
        if !ids.insert(f.id.as_str()) {
            return bad(format!("id de archivo duplicado: {}", f.id));
        }
        if !safe_relative_path(&f.path) {
            return bad(format!("ruta insegura en el manifiesto: {:?}", f.path));
        }
        // Dos rutas que difieren solo en mayúsculas colisionan en macOS/Windows.
        if !paths.insert(f.path.to_ascii_lowercase()) {
            return bad(format!(
                "ruta duplicada (o que solo difiere en mayúsculas): {}",
                f.path
            ));
        }
        // Kinds que esta versión sabe instalar, y dónde pueden caer.
        let top = f.path.split('/').next().unwrap_or("");
        let segs = f.path.split('/').count();
        match f.kind.as_str() {
            "mod" => {
                if top != "mods" || segs != 2 || !f.path.to_ascii_lowercase().ends_with(".jar") {
                    return bad(format!("un mod debe ir en mods/<archivo>.jar: {}", f.path));
                }
                if f.install_policy != "managed" {
                    return bad(format!("los mods siempre son «managed»: {}", f.id));
                }
            }
            "config" => {
                if top != "config" || segs < 2 {
                    return bad(format!("una config debe ir dentro de config/: {}", f.path));
                }
            }
            other => {
                return Err(ManifestError::LauncherUpdateRequired(format!(
                    "tipo de archivo «{other}» no soportado por esta versión"
                )));
            }
        }
        if !matches!(f.install_policy.as_str(), "managed" | "seed-if-absent") {
            return bad(format!("installPolicy desconocida: {}", f.install_policy));
        }
        if !is_sha256_hex(&f.sha256) {
            return bad(format!("sha256 inválido en {}", f.id));
        }
        if f.asset_name != format!("sha256-{}.bin", f.sha256) {
            return bad(format!("assetName no corresponde al sha256 en {}", f.id));
        }
        if !(1..=MAX_FILE_BYTES).contains(&f.bytes) {
            return bad(format!("tamaño inválido en {}", f.id));
        }
        total = total.saturating_add(f.bytes);
        if let Some(d) = &f.download {
            if d.kind != "github-release"
                || !valid_repo(&d.repository)
                || !valid_download_tag(&d.tag)
                || !valid_download_asset(&d.asset)
            {
                return bad(format!("download no permitido en {}", f.id));
            }
        }
        for list in [&f.platforms, &f.architectures].into_iter().flatten() {
            if list.is_empty() {
                return bad(format!("lista de plataformas vacía en {}", f.id));
            }
        }
        if let Some(p) = &f.platforms
            && p.iter()
                .any(|x| !matches!(x.as_str(), "windows" | "linux" | "macos"))
        {
            return bad(format!("plataforma desconocida en {}", f.id));
        }
        if let Some(a) = &f.architectures
            && a.iter().any(|x| !matches!(x.as_str(), "x64" | "arm64"))
        {
            return bad(format!("arquitectura desconocida en {}", f.id));
        }
        let o = &f.origin;
        if !https(&o.source_repository) || !https(&o.license_evidence_url) {
            return bad(format!("origen sin URLs https en {}", f.id));
        }
        if !matches!(
            o.redistribution_basis.as_str(),
            "owned" | "open-source-license" | "explicit-permission" | "upstream-direct"
        ) {
            return bad(format!("redistributionBasis desconocido en {}", f.id));
        }
        if let Some(src) = &o.corresponding_source_asset
            && !valid_asset_ref(src)
        {
            return bad(format!("correspondingSourceAsset inválido en {}", f.id));
        }
    }
    if total > MAX_TOTAL_BYTES {
        return bad("la release pide descargar demasiado".into());
    }
    Ok(())
}

impl ManifestFile {
    /// ¿Aplica a esta plataforma/arquitectura?
    pub fn applies_to(&self, platform: &str, arch: &str) -> bool {
        self.platforms
            .as_ref()
            .is_none_or(|p| p.iter().any(|x| x == platform))
            && self
                .architectures
                .as_ref()
                .is_none_or(|a| a.iter().any(|x| x == arch))
    }
    pub fn is_managed(&self) -> bool {
        self.install_policy == "managed"
    }
}

impl Manifest {
    /// Archivos que aplican al equipo actual.
    pub fn files_for<'a>(
        &'a self,
        platform: &'a str,
        arch: &'a str,
    ) -> impl Iterator<Item = &'a ManifestFile> {
        self.files
            .iter()
            .filter(move |f| f.applies_to(platform, arch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::selection::catalog::{Catalog, Channel, parse_catalog};
    use crate::services::selection::signatures::TRUSTED_KEYS;

    const MANIFEST_BYTES: &[u8] = include_bytes!("testdata/manifest-1.8.9.json");
    const MANIFEST_SIG: &[u8] = include_bytes!("testdata/manifest-1.8.9.json.sig");
    const CATALOG_BYTES: &[u8] = include_bytes!("testdata/catalog.json");

    fn real_expected() -> (Catalog, Target, ChannelPointer) {
        let catalog = parse_catalog(CATALOG_BYTES).unwrap();
        let target = catalog
            .targets
            .iter()
            .find(|t| t.target_id == "mc-1.8.9-ornithe")
            .unwrap()
            .clone();
        let pointer = target.channels.get(Channel::Beta).unwrap().clone();
        (catalog, target, pointer)
    }

    #[test]
    fn real_published_manifest_verifies_and_parses() {
        let (_cat, target, pointer) = real_expected();
        let expected = Expected {
            target: &target,
            pointer: &pointer,
        };
        let m = load_manifest(
            MANIFEST_BYTES,
            MANIFEST_SIG,
            &expected,
            "0.15.2",
            TRUSTED_KEYS,
        )
        .unwrap();
        assert_eq!(m.pack_id, "tfl-selection");
        assert_eq!(m.target_id, "mc-1.8.9-ornithe");
        assert_eq!(m.release, "0.1.0-preview.1");
        assert_eq!(m.minecraft.version, "1.8.9");
        assert_eq!(m.minecraft.loader, "ornithe-gen2");
        assert_eq!(m.minecraft.java_major, 25);
        assert_eq!(
            m.minecraft.bootstrap.as_deref(),
            Some("launcher-pinned-ornithe")
        );
        // Debe tener 11 mods y 6 configs (17 archivos en total)
        let mods: Vec<_> = m.files.iter().filter(|f| f.kind == "mod").collect();
        let configs: Vec<_> = m.files.iter().filter(|f| f.kind == "config").collect();
        assert_eq!(mods.len(), 11, "debe tener exactamente 11 mods");
        assert_eq!(configs.len(), 6, "debe tener 6 archivos de config");
        assert_eq!(m.files.len(), 17);
    }

    #[test]
    fn tampered_manifest_fails_signature() {
        let (_cat, target, pointer) = real_expected();
        let expected = Expected {
            target: &target,
            pointer: &pointer,
        };
        let mut tampered = MANIFEST_BYTES.to_vec();
        tampered.push(b' ');
        let err = load_manifest(&tampered, MANIFEST_SIG, &expected, "0.15.2", TRUSTED_KEYS);
        assert!(matches!(
            err,
            Err(ManifestError::HashMismatch | ManifestError::Signature(_))
        ));
    }

    #[test]
    fn manifest_hash_mismatch_is_rejected() {
        let (_cat, target, mut pointer) = real_expected();
        pointer.manifest_sha256 = "0".repeat(64);
        let expected = Expected {
            target: &target,
            pointer: &pointer,
        };
        let err = load_manifest(
            MANIFEST_BYTES,
            MANIFEST_SIG,
            &expected,
            "0.15.2",
            TRUSTED_KEYS,
        );
        assert!(matches!(err, Err(ManifestError::HashMismatch)));
    }

    #[test]
    fn higher_launcher_version_is_flagged() {
        let (_cat, target, pointer) = real_expected();
        let expected = Expected {
            target: &target,
            pointer: &pointer,
        };
        // 0.1.0-preview.1 requiere minimumLauncherVersion = 0.1.0
        let err = load_manifest(
            MANIFEST_BYTES,
            MANIFEST_SIG,
            &expected,
            "0.0.9",
            TRUSTED_KEYS,
        );
        assert!(matches!(err, Err(ManifestError::LauncherUpdateRequired(_))));
    }

    #[test]
    fn path_traversal_attempts_are_blocked() {
        assert!(!safe_relative_path("../escape.jar"));
        assert!(!safe_relative_path("mods/../../escape.jar"));
        assert!(!safe_relative_path("/absolute/path.jar"));
        assert!(!safe_relative_path("C:\\windows\\win.ini"));
        assert!(!safe_relative_path("mods\\slash.jar"));
        assert!(!safe_relative_path("mods/foo/../bar.jar"));
        assert!(!safe_relative_path(".hidden"));
        assert!(!safe_relative_path(""));
        assert!(safe_relative_path("mods/sodium-1.8.9.jar"));
        assert!(safe_relative_path("config/tfl/settings.json"));
    }
}
