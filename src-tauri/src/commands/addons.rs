//! Declarative packages are data, never executable content. No WebView, script or IPC is
//! created for an addon. Immutable generations + an atomic registry are the commit point.
use crate::core::PathManager;
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tauri::Emitter;
use tauri_plugin_dialog::DialogExt;

const MAX_PACKAGE: u64 = 12 * 1024 * 1024;
const MAX_EXPANDED: u64 = 16 * 1024 * 1024;
const MAX_RESOURCE: u64 = 4 * 1024 * 1024;
const MAX_MANIFEST: u64 = 64 * 1024;
const MAX_ENTRIES: usize = 25;
const MAX_ADDONS: usize = 20;
static MUTATION: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub api_version: String,
    pub launcher_version: String,
    pub permissions: Vec<String>,
    #[serde(default)]
    pub themes: Vec<Theme>,
    #[serde(default)]
    pub scenes: Vec<Scene>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub tokens: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub id: String,
    pub name: String,
    #[serde(default = "background")]
    pub background: String,
    pub layers: Vec<Layer>,
}
fn background() -> String {
    "#111827".into()
}
fn opacity() -> f64 {
    1.0
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub path: String,
    pub speed: f64,
    #[serde(default = "opacity")]
    pub opacity: f64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    digest: String,
    version: String,
    enabled: bool,
    #[serde(default)]
    previous: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Registry {
    #[serde(default)]
    safe_mode: bool,
    #[serde(default)]
    addons: BTreeMap<String, Record>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Installed {
    manifest: Manifest,
    digest: String,
    enabled: bool,
    rollback_available: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    safe_mode: bool,
    addons: Vec<Installed>,
    broken_addons: Vec<BrokenAddon>,
    errors: Vec<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokenAddon {
    id: String,
    error: String,
    rollback_available: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inspection {
    manifest: Manifest,
    digest: String,
    installed_version: Option<String>,
}
#[derive(Debug)]
struct Package {
    manifest: Manifest,
    digest: String,
    bytes: Vec<u8>,
    resources: BTreeMap<String, Vec<u8>>,
}
fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn directory() -> PathBuf {
    PathManager::get().get_settings_dir().join("addons")
}
fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}
fn slug(v: &str) -> bool {
    !v.is_empty()
        && v.len() <= 48
        && v.as_bytes()[0].is_ascii_lowercase()
        && v.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn addon_id(v: &str) -> bool {
    (3..=80).contains(&v.len()) && v.contains('.') && v.split('.').all(slug)
}
fn digest_id(v: &str) -> bool {
    v.len() == 64
        && v.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn color(v: &str) -> bool {
    v.len() == 7 && v.starts_with('#') && v[1..].bytes().all(|b| b.is_ascii_hexdigit())
}
fn resource_path(v: &str) -> bool {
    v.len() <= 160
        && v.starts_with("assets/")
        && v.ends_with(".png")
        && v.split('/').all(|s| {
            !s.is_empty()
                && s != "."
                && s != ".."
                && s.len() <= 64
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_.".contains(&b))
                && !s.ends_with('.')
                && ![
                    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6",
                    "com7", "com8", "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7",
                    "lpt8", "lpt9",
                ]
                .contains(&s.split('.').next().unwrap_or(""))
        })
}
const TOKENS: &[&str] = &[
    "--bg-main",
    "--bg-sidebar",
    "--bg-card",
    "--bg-input",
    "--bg-item-active",
    "--text-primary",
    "--text-secondary",
    "--text-muted",
    "--border",
    "--accent",
    "--accent-text",
];
fn validate_manifest(m: &Manifest) -> Result<BTreeSet<String>, String> {
    if m.schema_version != 1
        || !addon_id(&m.id)
        || !text(&m.name, 80)
        || !text(&m.author, 120)
        || !text(&m.description, 1200)
        || !m.permissions.is_empty()
    {
        return Err(
            "Manifest inválido; esta beta sólo admite contribuciones declarativas sin permisos"
                .into(),
        );
    }
    semver::Version::parse(&m.version).map_err(|_| "Versión del addon no es SemVer")?;
    let api = semver::VersionReq::parse(&m.api_version).map_err(|_| "Rango API inválido")?;
    let launcher =
        semver::VersionReq::parse(&m.launcher_version).map_err(|_| "Rango launcher inválido")?;
    if !api.matches(&semver::Version::new(1, 0, 0))
        || !launcher.matches(&semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(error)?)
    {
        return Err("Addon incompatible con esta versión del launcher/API".into());
    }
    if m.themes.len() > 8 || m.scenes.len() > 8 || m.themes.len() + m.scenes.len() == 0 {
        return Err("El addon necesita entre 1 y 8 contribuciones por clase".into());
    }
    let mut ids = BTreeSet::new();
    for t in &m.themes {
        if !slug(&t.id)
            || !text(&t.name, 80)
            || !ids.insert(format!("theme:{}", t.id))
            || t.tokens.is_empty()
            || t.tokens
                .iter()
                .any(|(k, v)| !TOKENS.contains(&k.as_str()) || !color(v))
        {
            return Err("Tema inválido: sólo tokens admitidos con colores #RRGGBB".into());
        }
    }
    let mut resources = BTreeSet::new();
    for s in &m.scenes {
        if !slug(&s.id)
            || !text(&s.name, 80)
            || !ids.insert(format!("scene:{}", s.id))
            || !color(&s.background)
            || s.layers.is_empty()
            || s.layers.len() > 6
        {
            return Err("Escena inválida".into());
        }
        for l in &s.layers {
            if !resource_path(&l.path)
                || !l.speed.is_finite()
                || !(0.0..=20.0).contains(&l.speed)
                || !l.opacity.is_finite()
                || !(0.0..=1.0).contains(&l.opacity)
            {
                return Err("Capa inválida".into());
            }
            resources.insert(l.path.clone());
        }
    }
    if resources.len() > MAX_ENTRIES - 1 {
        return Err("Demasiados recursos".into());
    }
    Ok(resources)
}
fn validate_png(bytes: &[u8]) -> Result<(), String> {
    // Bound dimensions BEFORE invoking the decoder. Decoder validates PNG structure/CRC;
    // metadata alone is not an image validation. APNG and trailing payloads are rejected.
    if bytes.len() < 33
        || !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        || bytes.get(12..16) != Some(b"IHDR")
    {
        return Err("Recurso no es PNG".into());
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().map_err(error)?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().map_err(error)?);
    if width == 0 || height == 0 || width > 2048 || height > 2048 {
        return Err("PNG supera 2048×2048".into());
    }
    let mut cursor = 8usize;
    let mut end = false;
    while cursor + 12 <= bytes.len() {
        let len = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().map_err(error)?) as usize;
        let next = cursor
            .checked_add(12)
            .and_then(|v| v.checked_add(len))
            .ok_or("PNG corrupto")?;
        if next > bytes.len() || &bytes[cursor + 4..cursor + 8] == b"acTL" {
            return Err("PNG corrupto o animado".into());
        }
        let mut crc = 0xffffffffu32;
        for &byte in &bytes[cursor + 4..next - 4] {
            crc ^= byte as u32;
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
            }
        }
        if (crc ^ 0xffffffff)
            != u32::from_be_bytes(bytes[next - 4..next].try_into().map_err(error)?)
        {
            return Err("CRC PNG inválido".into());
        }
        if &bytes[cursor + 4..cursor + 8] == b"IEND" {
            end = len == 0 && next == bytes.len();
            break;
        }
        cursor = next;
    }
    if !end {
        return Err("PNG incompleto o con contenido extra".into());
    }
    let decoded = tauri::image::Image::from_bytes(bytes).map_err(|_| "PNG no decodificable")?;
    if decoded.width() != width || decoded.height() != height {
        return Err("Dimensiones PNG inconsistentes".into());
    }
    Ok(())
}
fn bounded_read(path: &Path, max: u64) -> Result<Vec<u8>, String> {
    let meta = fs::symlink_metadata(path).map_err(error)?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > max {
        return Err("Archivo inválido o demasiado grande".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(error)?
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(error)?;
    if bytes.len() as u64 > max {
        return Err("Archivo demasiado grande".into());
    }
    Ok(bytes)
}
fn parse_package(bytes: Vec<u8>) -> Result<Package, String> {
    if bytes.len() as u64 > MAX_PACKAGE {
        return Err("Paquete supera 12 MiB".into());
    }
    // Reject ZIP64/multi-disk/comments and excessive central-directory counts BEFORE
    // ZipArchive allocates its index. The format deliberately supports a bounded ZIP subset.
    let eocd = bytes.len().checked_sub(22).ok_or("ZIP incompleto")?;
    let tail = &bytes[eocd..];
    if eocd >= 20 && &bytes[eocd - 20..eocd - 16] == b"PK\x06\x07" {
        return Err("ZIP64 no admitido".into());
    }
    if !tail.starts_with(b"PK\x05\x06") || tail[4..8] != [0, 0, 0, 0] || tail[20..22] != [0, 0] {
        return Err("ZIP64, multidisco o comentarios no admitidos".into());
    }
    let entries = u16::from_le_bytes([tail[10], tail[11]]) as usize;
    let local_entries = u16::from_le_bytes([tail[8], tail[9]]) as usize;
    let central_size = u32::from_le_bytes(tail[12..16].try_into().map_err(error)?) as usize;
    let central_offset = u32::from_le_bytes(tail[16..20].try_into().map_err(error)?) as usize;
    if entries == 0
        || entries > MAX_ENTRIES
        || entries != local_entries
        || central_offset.checked_add(central_size) != Some(eocd)
    {
        return Err("Índice ZIP fuera de límites".into());
    }
    let digest = hex::encode(Sha256::digest(&bytes));
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(&bytes)).map_err(|_| "ZIP inválido")?;
    if archive.len() != entries {
        return Err("ZIP supera el límite de entradas".into());
    }
    let mut names = BTreeSet::new();
    let mut resources = BTreeMap::new();
    let mut manifest = None;
    let mut total = 0u64;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|_| "Entrada ZIP inválida o cifrada")?;
        if !matches!(
            entry.compression(),
            zip::CompressionMethod::Stored | zip::CompressionMethod::Deflated
        ) {
            return Err("Compresión ZIP no admitida; usa Stored o Deflate".into());
        }
        let name = entry
            .name()
            .map_err(|_| "Nombre de entrada ZIP inválido")?
            .into_owned();
        let mode = entry.unix_mode().unwrap_or(0o100644) & 0o170000;
        if entry.is_dir()
            || (mode != 0 && mode != 0o100000)
            || !names.insert(name.to_ascii_lowercase())
            || (name != "manifest.json" && !resource_path(&name))
        {
            return Err("Ruta ZIP no permitida, enlace o colisión de nombres".into());
        }
        let limit = if name == "manifest.json" {
            MAX_MANIFEST
        } else {
            MAX_RESOURCE
        };
        total = total
            .checked_add(entry.size())
            .ok_or("Tamaño ZIP inválido")?;
        if entry.size() > limit
            || total > MAX_EXPANDED
            || entry.size()
                > entry
                    .compressed_size()
                    .saturating_mul(200)
                    .saturating_add(4096)
        {
            return Err("Límite de tamaño o ratio ZIP excedido".into());
        }
        let mut data = Vec::new();
        entry
            .by_ref()
            .take(limit + 1)
            .read_to_end(&mut data)
            .map_err(|_| "ZIP truncado o CRC inválido")?;
        if data.len() as u64 != entry.size() || data.len() as u64 > limit {
            return Err("Tamaño descomprimido inconsistente".into());
        }
        if name == "manifest.json" {
            manifest = Some(
                serde_json::from_slice::<Manifest>(&data).map_err(|_| "manifest.json inválido")?,
            );
        } else {
            validate_png(&data)?;
            resources.insert(name, data);
        }
    }
    let manifest = manifest.ok_or("Falta manifest.json")?;
    let declared = validate_manifest(&manifest)?;
    if declared != resources.keys().cloned().collect() {
        return Err("Recursos ausentes o archivos no declarados".into());
    }
    drop(archive);
    Ok(Package {
        manifest,
        digest,
        bytes,
        resources,
    })
}
fn package_at(path: &Path) -> Result<Package, String> {
    if path
        .extension()
        .and_then(|v| v.to_str())
        .is_none_or(|v| !v.eq_ignore_ascii_case("tfl"))
    {
        return Err("Selecciona un archivo .tfl".into());
    }
    parse_package(bounded_read(path, MAX_PACKAGE)?)
}
fn ensure_plain_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        let meta = fs::symlink_metadata(path).map_err(error)?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            return Err("Directorio de addons no seguro".into());
        }
    } else {
        fs::create_dir_all(path).map_err(error)?;
    }
    Ok(())
}
fn registry(root: &Path) -> Result<(Registry, Vec<String>), String> {
    if !root.exists() {
        return Ok((Registry::default(), vec![]));
    }
    ensure_plain_dir(root)?;
    let path = root.join("state.json");
    if !path.exists() {
        return Ok((Registry::default(), vec![]));
    }
    let read = |p: &Path| -> Result<Registry, String> {
        let s: Registry = serde_json::from_slice(&bounded_read(p, 128 * 1024)?).map_err(error)?;
        if s.addons.len() > MAX_ADDONS
            || s.addons.iter().any(|(id, r)| {
                !addon_id(id)
                    || !digest_id(&r.digest)
                    || semver::Version::parse(&r.version).is_err()
                    || r.previous.as_ref().is_some_and(|v| !digest_id(v))
            })
        {
            return Err("Registro inválido".into());
        }
        Ok(s)
    };
    match read(&path) {
        Ok(s) => Ok((s, vec![])),
        Err(_) => {
            let mut recovered = read(&root.join("state.previous.json")).unwrap_or_default();
            recovered.safe_mode = true;
            Ok((
                recovered,
                vec![
                    "Registro dañado: modo seguro activo; se recuperó la copia válida disponible"
                        .into(),
                ],
            ))
        }
    }
}
fn write_synced(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(error)?;
    file.write_all(bytes).map_err(error)?;
    file.sync_all().map_err(error)
}
fn commit(root: &Path, state: &Registry) -> Result<(), String> {
    ensure_plain_dir(root)?;
    let temporary = root.join(format!("state-{}.tmp", uuid::Uuid::new_v4()));
    let bytes = serde_json::to_vec(state).map_err(error)?;
    write_synced(&temporary, &bytes)?;
    // The backup is replaced atomically too; never copy unvalidated corrupted state.
    let (old, _) = registry(root)?;
    let backup = root.join(format!("backup-{}.tmp", uuid::Uuid::new_v4()));
    write_synced(&backup, &serde_json::to_vec(&old).map_err(error)?)?;
    fs::rename(&backup, root.join("state.previous.json")).map_err(error)?;
    if let Err(e) = fs::rename(&temporary, root.join("state.json")) {
        let _ = fs::remove_file(&temporary);
        return Err(error(e));
    }
    #[cfg(unix)]
    fs::File::open(root)
        .and_then(|f| f.sync_all())
        .map_err(error)?;
    Ok(())
}
fn installed_package(root: &Path, digest: &str) -> Result<Package, String> {
    if !digest_id(digest) {
        return Err("Identificador de generación inválido".into());
    }
    ensure_plain_dir(root)?;
    ensure_plain_dir(&root.join("packages"))?;
    let generation = root.join("packages").join(digest);
    let metadata = fs::symlink_metadata(&generation).map_err(error)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("Generación no segura".into());
    }
    let package = parse_package(bounded_read(&generation.join("package.tfl"), MAX_PACKAGE)?)?;
    if package.digest != digest {
        return Err("Paquete alterado: hash no coincide".into());
    }
    Ok(package)
}
fn catalog_at(root: &Path) -> Result<Catalog, String> {
    let (mut state, errors) = registry(root)?;
    let mut addons = Vec::new();
    let mut broken_addons = Vec::new();
    if std::env::var("TFL_ADDONS_SAFE_MODE").is_ok_and(|s| s == "1") {
        state.safe_mode = true;
    }
    for (id, r) in &state.addons {
        match installed_package(root, &r.digest) {
            Ok(p) if p.manifest.id == *id && p.manifest.version == r.version => {
                addons.push(Installed {
                    manifest: p.manifest,
                    digest: r.digest.clone(),
                    enabled: r.enabled && !state.safe_mode,
                    rollback_available: r.previous.is_some(),
                })
            }
            _ => broken_addons.push(BrokenAddon {
                id: id.clone(),
                error: "Paquete dañado o incompatible; restaura, actualiza o desinstala".into(),
                rollback_available: r.previous.is_some(),
            }),
        }
    }
    Ok(Catalog {
        safe_mode: state.safe_mode,
        addons,
        broken_addons,
        errors,
    })
}
fn remove_plain_tree(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(error)?;
    if metadata.file_type().is_symlink() {
        return Err("Enlace inesperado en cache de addons".into());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path).map_err(error)? {
            remove_plain_tree(&entry.map_err(error)?.path())?;
        }
        fs::remove_dir(path).map_err(error)
    } else if metadata.is_file() {
        fs::remove_file(path).map_err(error)
    } else {
        Err("Tipo de archivo inesperado".into())
    }
}
fn remove_owned_tree(path: &Path, boundary: &Path) -> Result<(), String> {
    // Resolve the final absolute target before recursive cleanup (including Windows
    // junctions). A package can never turn cleanup into deletion of the boundary
    // itself or a location outside its managed directory.
    let boundary = boundary.canonicalize().map_err(error)?;
    let target = path.canonicalize().map_err(error)?;
    if !target.is_absolute() || target == boundary || !target.starts_with(&boundary) {
        return Err("Destino de limpieza fuera del directorio administrado".into());
    }
    remove_plain_tree(path)
}
fn prune(root: &Path, state: &Registry) {
    let mut keep = BTreeSet::new();
    for r in state.addons.values() {
        keep.insert(r.digest.clone());
        if let Some(p) = &r.previous {
            keep.insert(p.clone());
        }
    }
    // Retain the registry backup's generations so crash recovery remains useful.
    if let Ok(bytes) = bounded_read(&root.join("state.previous.json"), 128 * 1024)
        && let Ok(backup) = serde_json::from_slice::<Registry>(&bytes)
    {
        for r in backup.addons.values() {
            keep.insert(r.digest.clone());
            if let Some(p) = &r.previous {
                keep.insert(p.clone());
            }
        }
    }
    let packages = root.join("packages");
    if ensure_plain_dir(&packages).is_err() {
        return;
    }
    if let Ok(entries) = fs::read_dir(&packages) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let staging = name
                .strip_prefix("staging-")
                .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok());
            if staging || (digest_id(&name) && !keep.contains(&name)) {
                let _ = remove_owned_tree(&entry.path(), &packages);
            }
        }
    }
}
fn install_at(root: &Path, source: &Path, digest: &str, replace: bool) -> Result<Catalog, String> {
    let p = package_at(source)?;
    if p.digest != digest {
        return Err("El archivo cambió desde la inspección; vuelve a revisarlo".into());
    }
    let (mut state, _) = registry(root)?;
    if let Some(old) = state.addons.get(&p.manifest.id) {
        if !replace {
            return Err("ID ya instalado: confirma expresamente la actualización".into());
        }
        if semver::Version::parse(&p.manifest.version).map_err(error)?
            <= semver::Version::parse(&old.version).map_err(error)?
        {
            return Err(
                "La actualización debe tener una versión superior; usa rollback para restaurar"
                    .into(),
            );
        }
    } else if state.addons.len() >= MAX_ADDONS {
        return Err("Máximo 20 addons instalados".into());
    }
    ensure_plain_dir(root)?;
    let packages = root.join("packages");
    ensure_plain_dir(&packages)?;
    // Every package is fully validated in memory before any resource reaches disk.
    let target = packages.join(&p.digest);
    if !target.exists() {
        let stage = packages.join(format!("staging-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&stage).map_err(error)?;
        let staged = (|| {
            write_synced(&stage.join("package.tfl"), &p.bytes)?;
            write_synced(
                &stage.join("manifest.json"),
                &serde_json::to_vec(&p.manifest).map_err(error)?,
            )?;
            for (name, bytes) in &p.resources {
                let target = stage.join(name);
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).map_err(error)?;
                }
                write_synced(&target, bytes)?;
            }
            fs::rename(&stage, &target).map_err(error)
        })();
        if staged.is_err() && stage.is_absolute() && stage.parent() == Some(packages.as_path()) {
            // The computed absolute target is constrained to this package staging root;
            // traversal refuses symlinks instead of following a replaced directory.
            let _ = remove_owned_tree(&stage, &packages);
        }
        staged?;
    } else {
        installed_package(root, &p.digest)?;
    }
    let old = state.addons.get(&p.manifest.id);
    let record = Record {
        digest: p.digest.clone(),
        version: p.manifest.version.clone(),
        enabled: old.is_some_and(|r| r.enabled),
        previous: old
            .filter(|r| installed_package(root, &r.digest).is_ok())
            .map(|r| r.digest.clone())
            .or_else(|| old.and_then(|r| r.previous.clone())),
    };
    state.addons.insert(p.manifest.id.clone(), record);
    commit(root, &state)?;
    prune(root, &state);
    catalog_at(root)
}
async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(error)?
}
#[tauri::command]
pub async fn addons_pick_file(app: tauri::AppHandle) -> Option<String> {
    if crate::services::SettingsManager::read().native_dialog_mode != "manual" {
        return None;
    }
    tokio::task::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("TFL Addon", &["tfl"])
            .blocking_pick_file()
            .and_then(|f| f.into_path().ok())
            .map(|p| p.to_string_lossy().into_owned())
    })
    .await
    .ok()
    .flatten()
}
#[tauri::command]
pub async fn addons_list() -> Result<Catalog, String> {
    let _guard = MUTATION.lock().await;
    blocking(|| catalog_at(&directory())).await
}
#[tauri::command]
pub async fn addons_inspect(path: String) -> Result<Inspection, String> {
    let _guard = MUTATION.lock().await;
    blocking(move || {
        let p = package_at(Path::new(&path))?;
        let root = directory();
        let (state, _) = registry(&root)?;
        let installed_version = state.addons.get(&p.manifest.id).map(|r| r.version.clone());
        Ok(Inspection {
            manifest: p.manifest,
            digest: p.digest,
            installed_version,
        })
    })
    .await
}
#[tauri::command]
pub async fn addons_install(
    app: tauri::AppHandle,
    path: String,
    digest: String,
    replace: bool,
) -> Result<Catalog, String> {
    let _guard = MUTATION.lock().await;
    let result =
        blocking(move || install_at(&directory(), Path::new(&path), &digest, replace)).await?;
    let _ = app.emit("addons-changed", ());
    Ok(result)
}
#[tauri::command]
pub async fn addons_set_enabled(
    app: tauri::AppHandle,
    id: String,
    enabled: bool,
) -> Result<Catalog, String> {
    let _guard = MUTATION.lock().await;
    blocking(move || {
        let root = directory();
        let (mut state, _) = registry(&root)?;
        let r = state.addons.get_mut(&id).ok_or("Addon no instalado")?;
        installed_package(&root, &r.digest)?;
        r.enabled = enabled;
        commit(&root, &state)?;
        prune(&root, &state);
        let _ = app.emit("addons-changed", ());
        catalog_at(&root)
    })
    .await
}
#[tauri::command]
pub async fn addons_safe_mode(app: tauri::AppHandle, enabled: bool) -> Result<Catalog, String> {
    let _guard = MUTATION.lock().await;
    blocking(move || {
        let root = directory();
        let (mut state, _) = registry(&root)?;
        state.safe_mode = enabled;
        commit(&root, &state)?;
        prune(&root, &state);
        let _ = app.emit("addons-changed", ());
        catalog_at(&root)
    })
    .await
}
#[tauri::command]
pub async fn addons_remove(app: tauri::AppHandle, id: String) -> Result<Catalog, String> {
    let _guard = MUTATION.lock().await;
    blocking(move || {
        let root = directory();
        let (mut state, _) = registry(&root)?;
        if state.addons.remove(&id).is_none() {
            return Err("Addon no instalado".into());
        }
        // Revoke first. Immutable package caches are retained for crash-safe rollback;
        // no caller-provided path is recursively removed, nor any addon-owned user data.
        commit(&root, &state)?;
        prune(&root, &state);
        let _ = app.emit("addons-changed", ());
        catalog_at(&root)
    })
    .await
}
#[tauri::command]
pub async fn addons_rollback(app: tauri::AppHandle, id: String) -> Result<Catalog, String> {
    let _guard = MUTATION.lock().await;
    blocking(move || {
        let root = directory();
        let (mut state, _) = registry(&root)?;
        let r = state.addons.get_mut(&id).ok_or("Addon no instalado")?;
        let previous = r.previous.clone().ok_or("No hay versión anterior")?;
        let p = installed_package(&root, &previous)?;
        if p.manifest.id != id {
            return Err("ID de rollback incorrecto".into());
        }
        r.version = p.manifest.version;
        r.previous = Some(std::mem::replace(&mut r.digest, previous));
        commit(&root, &state)?;
        prune(&root, &state);
        let _ = app.emit("addons-changed", ());
        catalog_at(&root)
    })
    .await
}
fn resource_package(root: &Path, id: &str) -> Result<Package, String> {
    let (state, _) = registry(root)?;
    if state.safe_mode || std::env::var("TFL_ADDONS_SAFE_MODE").is_ok_and(|s| s == "1") {
        return Err("Addons desactivados por modo seguro".into());
    }
    let r = state
        .addons
        .get(id)
        .filter(|r| r.enabled)
        .ok_or("Addon desactivado o no instalado")?;
    let p = installed_package(root, &r.digest)?;
    if p.manifest.id != id || p.manifest.version != r.version {
        return Err("ID o versión de registro incorrectos".into());
    }
    Ok(p)
}
fn data_url(bytes: &[u8]) -> String {
    format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}
#[tauri::command]
pub async fn addons_resource(id: String, path: String) -> Result<String, String> {
    let _guard = MUTATION.lock().await;
    blocking(move || {
        let p = resource_package(&directory(), &id)?;
        Ok(data_url(
            p.resources.get(&path).ok_or("Recurso no declarado")?,
        ))
    })
    .await
}
#[tauri::command]
pub async fn addons_scene_resources(
    id: String,
    scene_id: String,
) -> Result<BTreeMap<String, String>, String> {
    let _guard = MUTATION.lock().await;
    blocking(move || {
        let p = resource_package(&directory(), &id)?;
        let scene = p
            .manifest
            .scenes
            .iter()
            .find(|s| s.id == scene_id)
            .ok_or("Escena no declarada")?;
        scene
            .layers
            .iter()
            .map(|l| {
                Ok((
                    l.path.clone(),
                    data_url(p.resources.get(&l.path).ok_or("Recurso no declarado")?),
                ))
            })
            .collect()
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use zip::write::SimpleFileOptions;
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!("tfl-addons-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            if self.0.is_absolute() && self.0.parent() == Some(std::env::temp_dir().as_path()) {
                let _ = remove_owned_tree(&self.0, &std::env::temp_dir());
            }
        }
    }
    fn manifest() -> Manifest {
        Manifest {
            schema_version: 1,
            id: "test.orange".into(),
            name: "Example".into(),
            author: "Declared author".into(),
            version: "1.0.0".into(),
            description: "A test theme".into(),
            api_version: "^1.0.0".into(),
            launcher_version: format!("={}", env!("CARGO_PKG_VERSION")),
            permissions: vec![],
            themes: vec![Theme {
                id: "orange".into(),
                name: "Orange".into(),
                tokens: BTreeMap::from([("--accent".into(), "#F97316".into())]),
            }],
            scenes: vec![],
        }
    }
    fn zip(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            writer
                .start_file(
                    *name,
                    SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored),
                )
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }
    fn bytes(m: &Manifest) -> Vec<u8> {
        zip(&[("manifest.json", serde_json::to_vec(m).unwrap())])
    }
    fn source(temp: &Temp, m: &Manifest) -> (PathBuf, String) {
        let p = temp.0.join(format!("{}.tfl", m.version));
        let b = bytes(m);
        let digest = hex::encode(Sha256::digest(&b));
        fs::write(&p, b).unwrap();
        (p, digest)
    }
    #[test]
    fn declarative_example_decodes_and_has_only_declared_images() {
        let p = parse_package(include_bytes!("../../../examples/addons/orange-dunes.tfl").to_vec())
            .unwrap();
        assert_eq!(p.manifest.id, "tfl.orange-dunes");
        assert_eq!(p.resources.len(), 3);
    }
    #[test]
    fn traversal_absolute_windows_names_and_undeclared_files_are_rejected() {
        for bad in [
            "../outside.png",
            "/tmp/test.png",
            "C:/a.png",
            "assets/../a.png",
            "assets\\a.png",
            "assets/con.png",
            "assets/a.png.",
            "assets/a.js",
            "evil.html",
        ] {
            let b = zip(&[
                ("manifest.json", serde_json::to_vec(&manifest()).unwrap()),
                (bad, vec![0; 16]),
            ]);
            assert!(parse_package(b).is_err(), "accepted {bad}");
        }
    }
    #[test]
    fn duplicate_and_case_colliding_entries_are_rejected() {
        let m = serde_json::to_vec(&manifest()).unwrap();
        // Writer rejects duplicate names itself, so mutate a same-length second name.
        let mut b = zip(&[("manifest.json", m.clone()), ("manifest.jsox", m)]);
        for i in 0..b.len() - 12 {
            if &b[i..i + 13] == b"manifest.jsox" {
                b[i + 12] = b'n';
            }
        }
        assert!(parse_package(b).is_err());
        let b = zip(&[
            ("manifest.json", serde_json::to_vec(&manifest()).unwrap()),
            ("MANIFEST.JSON", vec![0]),
        ]);
        assert!(parse_package(b).is_err());
    }
    #[test]
    fn symlink_and_non_regular_entry_are_rejected() {
        let mut b = bytes(&manifest());
        let i = b.windows(4).position(|w| w == b"PK\x01\x02").unwrap();
        b[i + 5] = 3; // made by UNIX
        b[i + 38..i + 42].copy_from_slice(&(0o120777u32 << 16).to_le_bytes());
        assert!(parse_package(b).is_err());
    }
    #[test]
    fn package_limits_and_compression_bombs_are_rejected() {
        assert!(parse_package(vec![0; (MAX_PACKAGE + 1) as usize]).is_err());
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        writer
            .start_file(
                "manifest.json",
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
            )
            .unwrap();
        writer
            .write_all(&vec![b' '; MAX_MANIFEST as usize])
            .unwrap();
        let b = writer.finish().unwrap().into_inner();
        assert!(parse_package(b).is_err());
    }
    #[test]
    fn encrypted_unsupported_compression_and_excess_index_counts_rejected() {
        let original = bytes(&manifest());
        let central = original
            .windows(4)
            .position(|w| w == b"PK\x01\x02")
            .unwrap();
        let mut encrypted = original.clone();
        encrypted[6] |= 1;
        encrypted[central + 8] |= 1;
        assert!(parse_package(encrypted).is_err());
        let mut method = original.clone();
        method[8..10].copy_from_slice(&99u16.to_le_bytes());
        method[central + 10..central + 12].copy_from_slice(&99u16.to_le_bytes());
        assert!(parse_package(method).is_err());
        let mut count = original;
        let tail = count.len() - 22;
        count[tail + 8..tail + 10].copy_from_slice(&5000u16.to_le_bytes());
        count[tail + 10..tail + 12].copy_from_slice(&5000u16.to_le_bytes());
        assert!(parse_package(count).unwrap_err().contains("Índice ZIP"));
    }
    #[test]
    fn zip64_locator_is_rejected_even_when_zip32_eocd_values_are_small() {
        let original = bytes(&manifest());
        let end = original.len() - 22;
        let mut eocd = original[end..].to_vec();
        let central_size = u32::from_le_bytes(eocd[12..16].try_into().unwrap());
        let central_offset = u32::from_le_bytes(eocd[16..20].try_into().unwrap());
        // A complete, small EOCD64 + locator. The ordinary EOCD still declares
        // one entry and its size engulfs both records, so old preflight passed.
        let mut zip64 = Vec::new();
        zip64.extend_from_slice(b"PK\x06\x06");
        zip64.extend_from_slice(&44u64.to_le_bytes());
        zip64.extend_from_slice(&45u16.to_le_bytes());
        zip64.extend_from_slice(&45u16.to_le_bytes());
        zip64.extend_from_slice(&0u32.to_le_bytes());
        zip64.extend_from_slice(&0u32.to_le_bytes());
        zip64.extend_from_slice(&1u64.to_le_bytes());
        zip64.extend_from_slice(&1u64.to_le_bytes());
        zip64.extend_from_slice(&u64::from(central_size).to_le_bytes());
        zip64.extend_from_slice(&u64::from(central_offset).to_le_bytes());
        zip64.extend_from_slice(b"PK\x06\x07");
        zip64.extend_from_slice(&0u32.to_le_bytes());
        zip64.extend_from_slice(&(end as u64).to_le_bytes());
        zip64.extend_from_slice(&1u32.to_le_bytes());
        assert_eq!(zip64.len(), 76);
        eocd[12..16].copy_from_slice(&(central_size + 76).to_le_bytes());
        let mut package = original[..end].to_vec();
        package.extend_from_slice(&zip64);
        package.extend_from_slice(&eocd);
        assert!(package.len() < 4096);
        // zip 9 ignores this locator because the ZIP32 fields have no sentinel.
        assert_eq!(
            zip::ZipArchive::new(std::io::Cursor::new(&package))
                .unwrap()
                .len(),
            1
        );
        assert_eq!(parse_package(package).unwrap_err(), "ZIP64 no admitido");
    }
    #[test]
    fn unknown_executable_capabilities_and_css_are_rejected() {
        let mut m = manifest();
        m.permissions.push("network".into());
        assert!(parse_package(bytes(&m)).is_err());
        let mut m = manifest();
        m.themes[0]
            .tokens
            .insert("--accent".into(), "url(https://evil.test/x)".into());
        assert!(parse_package(bytes(&m)).is_err());
        let mut m = manifest();
        m.themes[0]
            .tokens
            .insert("--arbitrary".into(), "#FFFFFF".into());
        assert!(parse_package(bytes(&m)).is_err());
        let mut v = serde_json::to_value(manifest()).unwrap();
        v["scripts"] = serde_json::json!(["entry.js"]);
        assert!(parse_package(zip(&[("manifest.json", serde_json::to_vec(&v).unwrap())])).is_err());
        let duplicate = serde_json::to_string(&manifest()).unwrap().replacen(
            "\"schemaVersion\":1",
            "\"schemaVersion\":1,\"schemaVersion\":1",
            1,
        );
        assert!(parse_package(zip(&[("manifest.json", duplicate.into_bytes())])).is_err());
    }
    #[test]
    fn incompatible_schema_api_versions_ids_and_nan_rejected() {
        let mut m = manifest();
        m.schema_version = 2;
        assert!(parse_package(bytes(&m)).is_err());
        let mut m = manifest();
        m.api_version = "^2.0.0".into();
        assert!(parse_package(bytes(&m)).is_err());
        let mut m = manifest();
        m.launcher_version = "<0.1.0".into();
        assert!(parse_package(bytes(&m)).is_err());
        for id in ["plain", "../../foo", "Upper.case", "author.__proto__"] {
            let mut m = manifest();
            m.id = id.into();
            assert!(parse_package(bytes(&m)).is_err());
        }
        let mut m = manifest();
        m.scenes.push(Scene {
            id: "bad".into(),
            name: "Bad".into(),
            background: background(),
            layers: vec![Layer {
                path: "assets/a.png".into(),
                speed: f64::NAN,
                opacity: 1.0,
            }],
        });
        assert!(validate_manifest(&m).is_err());
    }
    #[test]
    fn png_dimensions_structure_crc_and_trailing_data_checked() {
        let original = include_bytes!("../../../examples/addons/orange-dunes/assets/dunes.png");
        assert!(validate_png(original).is_ok());
        let mut b = original.to_vec();
        b[16..20].copy_from_slice(&4096u32.to_be_bytes());
        assert!(validate_png(&b).is_err());
        let mut b = original.to_vec();
        b.extend_from_slice(b"<script>");
        assert!(validate_png(&b).is_err());
        let mut b = original.to_vec();
        b[50] ^= 0x55;
        assert!(validate_png(&b).is_err());
        assert!(validate_png(&original[..30]).is_err());
    }
    #[test]
    fn install_is_disabled_duplicate_requires_consent_and_toctou_is_rejected() {
        let temp = Temp::new();
        let root = temp.0.join("addons");
        let (path, digest) = source(&temp, &manifest());
        assert!(install_at(&root, &path, "bad", false).is_err());
        assert!(!root.exists());
        let catalog = install_at(&root, &path, &digest, false).unwrap();
        assert!(!catalog.addons[0].enabled);
        assert!(install_at(&root, &path, &digest, false).is_err());
        assert!(install_at(&root, &path, &digest, true).is_err());
        fs::write(path, b"changed").unwrap();
        assert!(install_at(&root, &temp.0.join("1.0.0.tfl"), &digest, true).is_err());
        assert_eq!(catalog_at(&root).unwrap().addons[0].digest, digest);
    }
    #[test]
    fn update_preserves_enablement_backup_previous_and_failed_update_is_atomic() {
        let temp = Temp::new();
        let root = temp.0.join("addons");
        let mut m = manifest();
        let (p, d) = source(&temp, &m);
        install_at(&root, &p, &d, false).unwrap();
        let (mut s, _) = registry(&root).unwrap();
        s.addons.get_mut(&m.id).unwrap().enabled = true;
        commit(&root, &s).unwrap();
        m.version = "1.1.0".into();
        let (p2, d2) = source(&temp, &m);
        // Simulate a failed generation publication before the registry commit.
        let collision = root.join("packages").join(&d2);
        fs::write(&collision, b"not a directory").unwrap();
        assert!(install_at(&root, &p2, &d2, true).is_err());
        assert_eq!(registry(&root).unwrap().0.addons[&m.id].digest, d);
        fs::remove_file(collision).unwrap();
        let result = install_at(&root, &p2, &d2, true).unwrap();
        assert!(result.addons[0].enabled);
        assert!(result.addons[0].rollback_available);
        let (s, _) = registry(&root).unwrap();
        assert_eq!(s.addons[&m.id].previous.as_deref(), Some(d.as_str()));
        fs::write(&p2, b"bad zip").unwrap();
        assert!(install_at(&root, &p2, &d2, true).is_err());
        assert_eq!(registry(&root).unwrap().0.addons[&m.id].digest, d2);
        assert!(installed_package(&root, &d).is_ok());
    }
    #[test]
    fn corrupted_registry_recovers_safe_and_tampered_package_is_never_contributed() {
        let temp = Temp::new();
        let root = temp.0.join("addons");
        let (p, d) = source(&temp, &manifest());
        install_at(&root, &p, &d, false).unwrap();
        let (mut s, _) = registry(&root).unwrap();
        s.addons.get_mut("test.orange").unwrap().enabled = true;
        commit(&root, &s).unwrap();
        fs::write(root.join("state.json"), b"{broken").unwrap();
        let c = catalog_at(&root).unwrap();
        assert!(c.safe_mode);
        assert!(!c.errors.is_empty());
        fs::write(
            root.join("packages").join(&d).join("package.tfl"),
            b"tampered",
        )
        .unwrap();
        let c = catalog_at(&root).unwrap();
        assert!(c.addons.is_empty());
        assert_eq!(c.broken_addons[0].id, "test.orange");
    }
    #[test]
    fn disabled_safe_mode_and_undeclared_resources_fail_closed() {
        let temp = Temp::new();
        let root = temp.0.join("addons");
        let (p, d) = source(&temp, &manifest());
        install_at(&root, &p, &d, false).unwrap();
        assert!(resource_package(&root, "test.orange").is_err());
        let (mut s, _) = registry(&root).unwrap();
        s.addons.get_mut("test.orange").unwrap().enabled = true;
        s.safe_mode = true;
        commit(&root, &s).unwrap();
        assert!(resource_package(&root, "test.orange").is_err());
        s.safe_mode = false;
        commit(&root, &s).unwrap();
        let package = resource_package(&root, "test.orange").unwrap();
        assert!(!package.resources.contains_key("../../settings.json"));
    }
    #[test]
    fn corrupt_package_can_be_repaired_without_losing_current_registry() {
        let temp = Temp::new();
        let root = temp.0.join("addons");
        let mut m = manifest();
        let (p, d) = source(&temp, &m);
        install_at(&root, &p, &d, false).unwrap();
        fs::write(
            root.join("packages").join(&d).join("package.tfl"),
            b"corrupt",
        )
        .unwrap();
        m.version = "1.1.0".into();
        let (p2, d2) = source(&temp, &m);
        assert!(install_at(&root, &p2, &d2, false).is_err());
        let c = install_at(&root, &p2, &d2, true).unwrap();
        assert!(c.broken_addons.is_empty());
        assert_eq!(c.addons[0].manifest.version, "1.1.0");
    }
    #[test]
    fn abandoned_staging_does_not_activate_and_cleanup_preserves_foreign_files() {
        let temp = Temp::new();
        let root = temp.0.join("addons");
        let packages = root.join("packages");
        fs::create_dir_all(&packages).unwrap();
        let staging = packages.join(format!("staging-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&staging).unwrap();
        fs::write(staging.join("partial"), b"partial").unwrap();
        fs::write(packages.join("unrelated.txt"), b"keep").unwrap();
        assert!(catalog_at(&root).unwrap().addons.is_empty());
        prune(&root, &Registry::default());
        assert!(!staging.exists());
        assert!(packages.join("unrelated.txt").exists());
    }
    #[test]
    fn cleanup_requires_resolved_child_of_its_managed_boundary() {
        let managed = Temp::new();
        let foreign = Temp::new();
        fs::write(foreign.0.join("keep.txt"), b"keep").unwrap();
        assert!(remove_owned_tree(&managed.0, &managed.0).is_err());
        assert!(remove_owned_tree(&foreign.0, &managed.0).is_err());
        assert!(foreign.0.join("keep.txt").exists());
        let child = managed.0.join("staging-test");
        fs::create_dir(&child).unwrap();
        fs::write(child.join("partial"), b"partial").unwrap();
        remove_owned_tree(&child, &managed.0).unwrap();
        assert!(!child.exists());
    }
    #[cfg(unix)]
    #[test]
    fn installed_symlinks_fail_closed_and_cleanup_never_follows_them() {
        let temp = Temp::new();
        let root = temp.0.join("addons");
        let (p, d) = source(&temp, &manifest());
        install_at(&root, &p, &d, false).unwrap();
        let generation = root.join("packages").join(&d);
        fs::remove_file(generation.join("package.tfl")).unwrap();
        std::os::unix::fs::symlink(&p, generation.join("package.tfl")).unwrap();
        assert!(installed_package(&root, &d).is_err());
        assert!(remove_plain_tree(&generation).is_err());
        assert!(p.exists());
    }
}
