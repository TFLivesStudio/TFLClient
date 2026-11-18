//! Preview and transactional, cancelable import of explicitly selected sources.
//! Only format adapters select files. Launcher credentials/hooks are not copied.
use super::import_launchers::{self, COPY_ENTRIES, Detected, ImportCandidate};
use crate::{
    core::PathManager,
    services::{
        content,
        instance_manager::{self, InstanceData, LoaderKind, ServerType},
        server_ports, worlds,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, HashMap},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, LazyLock, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tauri::{AppHandle, command};

const MAX_BYTES: u64 = 32 * 1024 * 1024 * 1024;
const MAX_FILES: usize = 50_000;
const MAX_METADATA: u64 = 2 * 1024 * 1024;
static IMPORTS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> = LazyLock::new(Default::default);

pub fn any_critical_import() -> bool {
    !IMPORTS.lock().unwrap().is_empty()
}

pub(super) struct ActiveImport {
    _maintenance: tokio::sync::OwnedRwLockReadGuard<()>,
    id: String,
    pub(super) cancel: Arc<AtomicBool>,
}
impl ActiveImport {
    pub(super) fn register(request: Option<String>) -> Result<Self, String> {
        let maintenance = crate::services::maintenance::operation()?;
        let id = request.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        uuid::Uuid::parse_str(&id).map_err(|_| "Identificador de importación inválido.")?;
        let mut active = IMPORTS.lock().unwrap();
        if active.len() >= 8 || active.contains_key(&id) {
            return Err("Ya hay demasiadas importaciones o ese identificador está en uso.".into());
        }
        let cancel = Arc::new(AtomicBool::new(false));
        active.insert(id.clone(), cancel.clone());
        Ok(Self {
            id,
            cancel,
            _maintenance: maintenance,
        })
    }
}
impl Drop for ActiveImport {
    fn drop(&mut self) {
        IMPORTS.lock().unwrap().remove(&self.id);
    }
}

#[command]
pub fn cancel_instance_import(request_id: String) -> bool {
    if let Some(cancel) = IMPORTS.lock().unwrap().get(&request_id) {
        cancel.store(true, Ordering::Relaxed);
        true
    } else {
        false
    }
}
#[command]
pub async fn pick_import_directory(app: AppHandle) -> Result<Option<String>, String> {
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        return crate::core::file_dialog::pick_macos(true, false, &[])
            .await
            .map(|paths| {
                paths
                    .into_iter()
                    .next()
                    .map(|path| path.to_string_lossy().to_string())
            });
    }
    #[cfg(not(target_os = "macos"))]
    {
        use tauri_plugin_dialog::DialogExt;
        tokio::task::spawn_blocking(move || {
            app.dialog()
                .file()
                .blocking_pick_folder()
                .and_then(|folder| folder.into_path().ok())
                .map(|path| path.to_string_lossy().to_string())
        })
        .await
        .map_err(|_| "No se pudo abrir el selector de carpetas.".into())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportPreview {
    pub candidate: ImportCandidate,
    pub format: String,
    pub importable: bool,
    pub included: Vec<String>,
    pub omitted: Vec<String>,
    pub warnings: Vec<String>,
    pub file_count: usize,
    pub size_bytes: u64,
    pub fingerprint: String,
    pub server_type: Option<ServerType>,
}
#[derive(Clone)]
struct CopyItem {
    source: PathBuf,
    relative: PathBuf,
    zip_entry: Option<ZipEntry>,
    size: u64,
    modified: u128,
    directory: bool,
}
#[derive(Clone)]
struct ZipEntry {
    index: usize,
    name: String,
    crc: u32,
}
struct Plan {
    preview: ImportPreview,
    items: Vec<CopyItem>,
    source: PathBuf,
    game_dir: PathBuf,
    server: Option<ServerManifest>,
    min_memory: Option<u32>,
    max_memory: Option<u32>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServerManifest {
    format_version: u32,
    name: String,
    mc_version: String,
    server_type: ServerType,
    server_jar: String,
    #[serde(default)]
    server_build: Option<String>,
}

fn error(value: impl std::fmt::Display) -> String {
    value.to_string()
}
pub(super) fn canceled(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("Importación cancelada. El origen se conserva sin cambios.".into())
    } else {
        Ok(())
    }
}
fn small_file(path: &Path) -> Result<Vec<u8>, String> {
    worlds::no_link(path)?;
    if !fs::metadata(path).map_err(error)?.is_file() {
        return Err("Los metadatos deben ser un archivo normal.".into());
    }
    let file = File::open(path).map_err(error)?;
    if file.metadata().map_err(error)?.len() > MAX_METADATA {
        return Err("Los metadatos del origen superan 2 MiB.".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_METADATA + 1)
        .read_to_end(&mut bytes)
        .map_err(error)?;
    if bytes.len() as u64 > MAX_METADATA {
        return Err("Los metadatos cambiaron o superan el límite.".into());
    }
    Ok(bytes)
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-+".contains(&byte))
}
fn credentials(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "accounts.json"
            | "launcher_accounts.json"
            | "launcher_accounts_microsoft_store.json"
            | "auth.json"
            | "credentials.json"
            | "msa-cache.json"
            | ".env"
            | "session.lock"
    )
}
fn stamp(meta: &fs::Metadata) -> u128 {
    meta.modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|time| time.as_nanos())
        .unwrap_or(0)
}
fn remember_omitted(list: &mut Vec<String>, name: &str) {
    if list.len() < 40 && !list.iter().any(|item| item == name) {
        list.push(name.into());
    }
}
fn metadata_files(root: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut result = Vec::new();
    for name in [
        "mmc-pack.json",
        "instance.cfg",
        "minecraftinstance.json",
        "tfl-server.json",
        "manifest.json",
    ] {
        let path = root.join(name);
        if path.try_exists().map_err(error)? {
            result.push((name.into(), small_file(&path)?));
        }
    }
    Ok(result)
}
fn metadata_blocked(root: &Path) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();
    if root.join("mmc-pack.json").is_file() {
        let data: serde_json::Value =
            serde_json::from_slice(&small_file(&root.join("mmc-pack.json"))?)
                .map_err(|_| "mmc-pack.json no contiene JSON válido.")?;
        let components = data["components"]
            .as_array()
            .ok_or("El origen no declara componentes MultiMC válidos.")?;
        let mut loaders = 0;
        for component in components {
            let uid = component["uid"].as_str().unwrap_or("");
            match uid {
                "net.minecraft" | "org.lwjgl" | "org.lwjgl3" | "net.fabricmc.intermediary" => {}
                "net.fabricmc.fabric-loader"
                | "org.quiltmc.quilt-loader"
                | "net.neoforged"
                | "net.minecraftforge" => {
                    loaders += 1;
                }
                _ => warnings.push(format!("Componente sin adaptador comprobado: {uid}")),
            }
        }
        if loaders > 1 {
            warnings.push("El origen combina varios loaders; no se puede preservar esa composición con seguridad.".into());
        }
        if root.join("patches").exists() || root.join("jarmods").exists() {
            warnings.push("El origen incluye parches de versión o jarmods que este adaptador no puede reproducir.".into());
        }
    }
    Ok(warnings)
}
fn detect_directory(root: &Path) -> Result<(Detected, Option<ServerManifest>), String> {
    worlds::no_link(root)?;
    if root.join("tfl-server.json").is_file() {
        let server: ServerManifest =
            serde_json::from_slice(&small_file(&root.join("tfl-server.json"))?)
                .map_err(|_| "tfl-server.json no cumple el esquema portable de servidor.")?;
        if server.format_version != 1
            || !identifier(&server.mc_version)
            || worlds::valid_name(&server.server_jar).is_err()
            || !server.server_jar.ends_with(".jar")
            || server
                .server_build
                .as_ref()
                .is_some_and(|build| build.len() > 200 || build.chars().any(char::is_control))
        {
            return Err("El formato o los metadatos del servidor no están admitidos.".into());
        }
        let candidate = ImportCandidate {
            source: "tfl_server".into(),
            name: server.name.clone(),
            path: root.to_string_lossy().to_string(),
            mc_version: Some(server.mc_version.clone()),
            loader: "vanilla".into(),
            mod_count: 0,
            loader_version: None,
        };
        return Ok((
            Detected {
                candidate,
                game_dir: root.to_path_buf(),
            },
            Some(server),
        ));
    }
    if root.join("mmc-pack.json").is_file() {
        if root.join(".minecraft").is_dir() && root.join("minecraft").is_dir() {
            return Err("Se encontraron dos directorios de juego en la instancia. Conserva un único directorio en una copia del origen e inténtalo de nuevo.".into());
        }
        return import_launchers::detect_mmc_style("mmc_family", root).map(|detected| (detected, None)).ok_or_else(|| "La instancia MultiMC no contiene versión o directorio de juego válido (.minecraft o minecraft).".into());
    }
    if root.join("minecraftinstance.json").is_file() {
        return import_launchers::detect_curseforge(root)
            .map(|detected| (detected, None))
            .ok_or_else(|| {
                "Los metadatos de la instancia instalada de CurseForge son inválidos.".into()
            });
    }
    let canonical = fs::canonicalize(root).map_err(error)?;
    if let Some(found) = import_launchers::detect_all()
        .into_iter()
        .find(|found| fs::canonicalize(&found.game_dir).ok().as_ref() == Some(&canonical))
    {
        return Ok((found, None));
    }
    // A manually selected custom Modrinth app directory may contain its own DB.
    for parent in root.ancestors().take(3) {
        let db = parent.join("app.db");
        if db.is_file() {
            worlds::no_link(&db)?;
            let found = import_launchers::query_modrinth_app_db(parent, &db)
                .map_err(|_| "No se pudo leer el esquema de Modrinth en modo de solo lectura.")?;
            if let Some(found) = found
                .into_iter()
                .find(|found| fs::canonicalize(&found.game_dir).ok().as_ref() == Some(&canonical))
            {
                return Ok((found, None));
            }
        }
    }
    Err(if root.join("manifest.json").is_file() { "Se detectó un ZIP/directorio de exportación de CurseForge. Sus referencias de descarga no se importan: selecciona la instancia ya instalada en CurseForge." } else { "Formato desconocido. Selecciona una instancia instalada MultiMC/Prism/PolyMC/Freesm, CurseForge, Modrinth o un ZIP portable de servidor TFL." }.into())
}
fn allowed(relative: &Path, server: Option<&ServerManifest>) -> bool {
    let parts: Vec<_> = relative.iter().filter_map(|part| part.to_str()).collect();
    if parts.is_empty() || parts.iter().any(|part| credentials(part)) {
        return false;
    }
    if let Some(server) = server {
        parts.len() == 1 && (parts[0] == server.server_jar || parts[0] == "server.properties")
            || server.server_type != ServerType::Vanilla
                && parts[0] == "plugins"
                && (parts.len() == 1 || parts.len() == 2 && parts[1].ends_with(".jar"))
    } else {
        COPY_ENTRIES.contains(&parts[0])
    }
}
fn directory_items(
    game: &Path,
    server: Option<&ServerManifest>,
    omitted: &mut Vec<String>,
) -> Result<Vec<CopyItem>, String> {
    worlds::no_link(game)?;
    let mut items = Vec::new();
    let mut stack = vec![(game.to_path_buf(), 0)];
    let mut total = 0u64;
    let mut inspected = 0usize;
    let mut seen = BTreeSet::new();
    while let Some((dir, depth)) = stack.pop() {
        if depth > 64 {
            return Err("La instancia supera 64 niveles de carpetas.".into());
        }
        for entry in
            fs::read_dir(&dir).map_err(|e| format!("No se pudo inspeccionar el origen: {e}"))?
        {
            let entry = entry.map_err(error)?;
            inspected += 1;
            if inspected > MAX_FILES {
                return Err("El origen supera 50 000 entradas inspeccionables.".into());
            }
            let relative = entry
                .path()
                .strip_prefix(game)
                .map_err(error)?
                .to_path_buf();
            if !allowed(&relative, server) {
                remember_omitted(omitted, &relative.to_string_lossy());
                continue;
            }
            for component in &relative {
                worlds::valid_name(
                    component
                        .to_str()
                        .ok_or("El origen contiene un nombre no UTF-8.")?,
                )?;
            }
            if !seen.insert(relative.to_string_lossy().to_lowercase()) {
                return Err("El origen incluye rutas equivalentes en Windows.".into());
            }
            let path = entry.path();
            worlds::no_link(&path)?;
            let meta = fs::symlink_metadata(&path).map_err(error)?;
            if !meta.is_dir() && !meta.is_file() {
                return Err("El origen contiene archivos especiales no admitidos.".into());
            }
            let size = if meta.is_file() { meta.len() } else { 0 };
            total = total
                .checked_add(size)
                .filter(|total| *total <= MAX_BYTES)
                .ok_or("La importación supera 32 GiB.")?;
            items.push(CopyItem {
                source: path.clone(),
                relative,
                zip_entry: None,
                size,
                modified: stamp(&meta),
                directory: meta.is_dir(),
            });
            if meta.is_dir() {
                stack.push((path, depth + 1));
            }
        }
    }
    items.sort_by(|a, b| a.relative.cmp(&b.relative));
    Ok(items)
}

struct MetadataStage(PathBuf);
impl MetadataStage {
    fn new() -> Result<Self, String> {
        let path =
            std::env::temp_dir().join(format!("tfl-import-metadata-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).map_err(error)?;
        Ok(Self(path))
    }
}
impl Drop for MetadataStage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn inspect(path: &Path) -> Result<Plan, String> {
    worlds::no_link(path)?;
    if !path.is_dir() && !path.is_file() {
        return Err("Selecciona un archivo o directorio normal.".into());
    }
    let mut omitted = vec!["Cuentas, tokens, credenciales, cachés, Java personalizado y comandos del launcher de origen.".into()];
    let mut hasher = Sha256::new();
    let mut format_warnings = Vec::new();
    let (mut detected, server, items, metadata, archive, cfg) = if path.is_dir() {
        let root = if matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some(".minecraft" | "minecraft")
        ) && path
            .parent()
            .is_some_and(|parent| parent.join("mmc-pack.json").is_file())
        {
            path.parent().unwrap()
        } else {
            path
        };
        let (detected, server) = detect_directory(root)?;
        if root.join("patches").exists() || root.join("jarmods").exists() {
            format_warnings.push(
                "El origen incluye parches de versión o jarmods sin adaptador comprobado.".into(),
            );
        }
        let metadata = metadata_files(root)?;
        let items = directory_items(&detected.game_dir, server.as_ref(), &mut omitted)?;
        let cfg = metadata
            .iter()
            .find(|(name, _)| name == "instance.cfg")
            .map(|(_, raw)| String::from_utf8_lossy(raw).to_string());
        (detected, server, items, metadata, false, cfg)
    } else {
        if !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
        {
            return Err("Selecciona un directorio o un ZIP de instancia. Los .mrpack tienen su propio importador.".into());
        }
        let mut zip = zip::ZipArchive::new(File::open(path).map_err(error)?)
            .map_err(|_| "El archivo no es un ZIP válido.")?;
        worlds::inspect_zip(&mut zip, MAX_BYTES)?;
        let stage = MetadataStage::new()?;
        let mut prefixes = BTreeSet::new();
        for index in 0..zip.len() {
            let entry = zip.by_index(index).map_err(error)?;
            let name = entry.name().map_err(error)?.to_string();
            let rel = worlds::archive_rel(&name)?;
            if matches!(
                rel.file_name().and_then(|name| name.to_str()),
                Some(
                    "mmc-pack.json"
                        | "minecraftinstance.json"
                        | "tfl-server.json"
                        | "manifest.json"
                )
            ) {
                let prefix = rel.parent().unwrap_or(Path::new(""));
                if prefix.components().count() <= 1 {
                    prefixes.insert(prefix.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        if prefixes.len() != 1 {
            return Err("El ZIP debe contener una única instancia reconocible en la raíz o dentro de una carpeta.".into());
        }
        let prefix = prefixes.into_iter().next().unwrap();
        let prefix = if prefix.is_empty() {
            String::new()
        } else {
            format!("{prefix}/")
        };
        let mut metadata = Vec::new();
        let mut game_variants = BTreeSet::new();
        for index in 0..zip.len() {
            let mut entry = zip.by_index(index).map_err(error)?;
            let name = entry.name().map_err(error)?.to_string();
            let Some(local) = name.strip_prefix(&prefix) else {
                remember_omitted(&mut omitted, &name);
                continue;
            };
            if local.starts_with("patches/") || local.starts_with("jarmods/") {
                remember_omitted(
                    &mut format_warnings,
                    "El origen incluye parches de versión o jarmods sin adaptador comprobado.",
                );
            }
            for game in [".minecraft/", "minecraft/"] {
                if local.starts_with(game) {
                    game_variants.insert(game.trim_end_matches('/'));
                }
            }
            if [
                "mmc-pack.json",
                "instance.cfg",
                "minecraftinstance.json",
                "tfl-server.json",
                "manifest.json",
            ]
            .contains(&local)
            {
                if entry.size() > MAX_METADATA {
                    return Err("Los metadatos del ZIP superan 2 MiB.".into());
                }
                let mut raw = Vec::new();
                (&mut entry)
                    .take(MAX_METADATA + 1)
                    .read_to_end(&mut raw)
                    .map_err(error)?;
                if raw.len() as u64 > MAX_METADATA {
                    return Err("Los metadatos del ZIP superan el límite.".into());
                }
                fs::write(stage.0.join(local), &raw).map_err(error)?;
                metadata.push((local.to_string(), raw));
            }
        }
        for game in game_variants {
            fs::create_dir(stage.0.join(game)).map_err(error)?;
        }
        let (mut detected, server) = detect_directory(&stage.0)?;
        if detected.candidate.name.starts_with("tfl-import-metadata-") {
            detected.candidate.name = path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("Instancia importada")
                .into();
        }
        let game_suffix = detected
            .game_dir
            .strip_prefix(&stage.0)
            .map_err(error)?
            .to_string_lossy()
            .replace('\\', "/");
        let game_prefix = format!(
            "{prefix}{}",
            if game_suffix.is_empty() {
                String::new()
            } else {
                format!("{game_suffix}/")
            }
        );
        let mut items = Vec::new();
        for index in 0..zip.len() {
            let entry = zip.by_index(index).map_err(error)?;
            let name = entry.name().map_err(error)?.to_string();
            let Some(relative) = name
                .strip_prefix(&game_prefix)
                .filter(|relative| !relative.is_empty())
            else {
                continue;
            };
            let relative = worlds::archive_rel(relative)?;
            if !allowed(&relative, server.as_ref()) {
                remember_omitted(&mut omitted, &relative.to_string_lossy());
                continue;
            }
            hasher.update(entry.crc32().to_le_bytes());
            items.push(CopyItem {
                source: path.to_path_buf(),
                relative,
                zip_entry: Some(ZipEntry {
                    index,
                    name,
                    crc: entry.crc32(),
                }),
                size: entry.size(),
                modified: 0,
                directory: entry.is_dir(),
            });
        }
        let cfg = metadata
            .iter()
            .find(|(name, _)| name == "instance.cfg")
            .map(|(_, raw)| String::from_utf8_lossy(raw).to_string());
        detected.game_dir = PathBuf::from(game_prefix);
        (detected, server, items, metadata, true, cfg)
    };
    let mut warnings = format_warnings;
    if let Some((_, raw)) = metadata.iter().find(|(name, _)| name == "mmc-pack.json") {
        let stage = MetadataStage::new()?;
        fs::write(stage.0.join("mmc-pack.json"), raw).map_err(error)?;
        warnings.extend(metadata_blocked(&stage.0)?);
    }
    let _archive = archive;
    let mc_version = detected.candidate.mc_version.as_deref();
    if !mc_version.is_some_and(identifier) {
        warnings.push("No se reconoce una versión de Minecraft válida en el origen.".into());
    }
    let loader = LoaderKind::parse(&detected.candidate.loader);
    if loader.is_none() {
        warnings.push("El loader del origen no tiene un adaptador compatible.".into());
    }
    if loader.is_some_and(|loader| loader != LoaderKind::Vanilla)
        && !detected
            .candidate
            .loader_version
            .as_deref()
            .is_some_and(identifier)
    {
        warnings.push("El origen no declara la versión exacta del loader; no se sustituirá silenciosamente por otra.".into());
    }
    if let Some(server) = &server {
        if !items
            .iter()
            .any(|item| !item.directory && item.relative == Path::new(&server.server_jar))
        {
            warnings.push("Falta el JAR de servidor declarado.".into());
        }
    }
    let mut included = BTreeSet::new();
    let mut bytes = 0u64;
    let mut file_count = 0usize;
    for item in &items {
        if let Some(top) = item.relative.iter().next() {
            included.insert(top.to_string_lossy().to_string());
        }
        hasher.update(
            item.relative
                .to_string_lossy()
                .replace('\\', "/")
                .as_bytes(),
        );
        hasher.update(item.size.to_le_bytes());
        hasher.update(item.modified.to_le_bytes());
        if !item.directory {
            bytes += item.size;
            file_count += 1;
        }
    }
    for (name, raw) in &metadata {
        hasher.update(name.as_bytes());
        hasher.update(raw);
    }
    let source_meta = fs::metadata(path).map_err(error)?;
    hasher.update(source_meta.len().to_le_bytes());
    hasher.update(stamp(&source_meta).to_le_bytes());
    hasher.update(
        detected
            .candidate
            .mc_version
            .as_deref()
            .unwrap_or("")
            .as_bytes(),
    );
    hasher.update(detected.candidate.loader.as_bytes());
    hasher.update(
        detected
            .candidate
            .loader_version
            .as_deref()
            .unwrap_or("")
            .as_bytes(),
    );
    detected.candidate.path = path.to_string_lossy().to_string();
    detected.candidate.mod_count = items
        .iter()
        .filter(|item| {
            item.relative.starts_with("mods")
                && item
                    .relative
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
        })
        .count() as u32;
    let memory = |key: &str| -> Option<u32> {
        cfg.as_ref()?
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}=")))?
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|value| (128..=131072).contains(value))
    };
    let overrides_memory = cfg.as_ref().is_some_and(|cfg| {
        cfg.lines()
            .any(|line| line.trim().eq_ignore_ascii_case("OverrideMemory=true"))
    });
    let (min_memory, max_memory) = if overrides_memory {
        match (memory("MinMemAlloc"), memory("MaxMemAlloc")) {
            (Some(min), Some(max)) if min <= max => (Some(min), Some(max)),
            _ => {
                remember_omitted(&mut omitted, "Asignación de memoria inválida o incompleta.");
                (None, None)
            }
        }
    } else {
        (None, None)
    };
    let importable = warnings.is_empty();
    let format = if server.is_some() {
        "tfl-server-v1"
    } else if detected.candidate.source == "curseforge" {
        "curseforge-installed"
    } else if detected.candidate.source == "modrinth_app" {
        "modrinth-sqlite"
    } else {
        "multimc-v1"
    };
    Ok(Plan {
        preview: ImportPreview {
            candidate: detected.candidate,
            format: format.into(),
            importable,
            included: included.into_iter().collect(),
            omitted,
            warnings,
            file_count,
            size_bytes: bytes,
            fingerprint: hex::encode(hasher.finalize()),
            server_type: server.as_ref().map(|server| server.server_type),
        },
        items,
        source: path.to_path_buf(),
        game_dir: detected.game_dir,
        server,
        min_memory,
        max_memory,
    })
}

#[command]
pub async fn inspect_instance_source(path: String) -> Result<ImportPreview, String> {
    tokio::task::spawn_blocking(move || inspect(Path::new(&path)).map(|plan| plan.preview))
        .await
        .map_err(|_| "No se pudo completar la inspección del origen.")?
}

fn copy_stream(
    input: &mut impl Read,
    output: &mut File,
    expected: u64,
    cancel: &AtomicBool,
) -> Result<(), String> {
    let mut bytes = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        canceled(cancel)?;
        let count = input.read(&mut buffer).map_err(error)?;
        if count == 0 {
            break;
        }
        bytes += count as u64;
        if bytes > expected || bytes > MAX_BYTES {
            return Err("Un archivo del origen cambió o supera el límite.".into());
        }
        output.write_all(&buffer[..count]).map_err(error)?;
    }
    if bytes != expected {
        return Err("Un archivo del origen quedó incompleto; no se creó la instancia.".into());
    }
    output.sync_all().map_err(error)
}
fn copy_plan(plan: &Plan, destination: &Path, cancel: &AtomicBool) -> Result<(), String> {
    let _sessions = if plan.source.is_dir() && plan.game_dir.join("saves").is_dir() {
        worlds::lock_world_sessions(&plan.game_dir.join("saves"))?
    } else {
        Vec::new()
    };
    let mut archive = if plan.source.is_file() {
        worlds::no_link(&plan.source)?;
        let mut archive =
            zip::ZipArchive::new(File::open(&plan.source).map_err(error)?).map_err(error)?;
        worlds::inspect_zip(&mut archive, MAX_BYTES)?;
        Some(archive)
    } else {
        None
    };
    for item in &plan.items {
        canceled(cancel)?;
        let target = destination.join(&item.relative);
        let mut zip_input = match &item.zip_entry {
            Some(expected) => {
                let entry = archive
                    .as_mut()
                    .ok_or("Falta el ZIP del origen.")?
                    .by_index(expected.index)
                    .map_err(error)?;
                if entry.name().map_err(error)? != expected.name
                    || entry.crc32() != expected.crc
                    || entry.size() != item.size
                    || entry.is_dir() != item.directory
                {
                    return Err(
                        "El ZIP cambió después de la previsualización. Vuelve a inspeccionarlo."
                            .into(),
                    );
                }
                Some(entry)
            }
            None => None,
        };
        if item.directory {
            fs::create_dir_all(target).map_err(error)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(error)?;
        }
        let mut output = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(target)
            .map_err(error)?;
        if let Some(ref mut entry) = zip_input {
            copy_stream(entry, &mut output, item.size, cancel)?;
        } else {
            worlds::no_link(&item.source)?;
            let before = fs::metadata(&item.source).map_err(error)?;
            if before.len() != item.size || stamp(&before) != item.modified {
                return Err(
                    "El origen cambió después de la previsualización. Vuelve a inspeccionarlo."
                        .into(),
                );
            }
            let mut input = File::open(&item.source).map_err(error)?;
            copy_stream(&mut input, &mut output, item.size, cancel)?;
            let after = fs::metadata(&item.source).map_err(error)?;
            if after.len() != item.size || stamp(&after) != item.modified {
                return Err(
                    "El origen cambió mientras se copiaba. No se creó una instancia parcial."
                        .into(),
                );
            }
        }
    }
    Ok(())
}

#[command]
pub async fn import_instance_source(
    app: AppHandle,
    path: String,
    new_name: String,
    preview_fingerprint: Option<String>,
    request_id: Option<String>,
) -> Result<InstanceData, String> {
    let data = commit_instance_source(path, new_name, preview_fingerprint, request_id).await?;
    super::instance::assign_random_icon(&app, &data).await;
    Ok(data)
}

async fn commit_instance_source(
    path: String,
    new_name: String,
    preview_fingerprint: Option<String>,
    request_id: Option<String>,
) -> Result<InstanceData, String> {
    let active = ActiveImport::register(request_id)?;
    instance_manager::valid_instance_name(&new_name)?;
    worlds::valid_name(&new_name)?;
    let parse_path = path.clone();
    let plan = tokio::task::spawn_blocking(move || inspect(Path::new(&parse_path)))
        .await
        .map_err(|_| "No se pudo completar la inspección.")??;
    if !plan.preview.importable {
        return Err(plan.preview.warnings.join(" "));
    }
    if preview_fingerprint.is_some_and(|fingerprint| fingerprint != plan.preview.fingerprint) {
        return Err(
            "El origen cambió desde la previsualización. Vuelve a revisarlo antes de importar."
                .into(),
        );
    }
    canceled(&active.cancel)?;
    let _mutation = content::mutation_lock(&new_name).await;
    let server_guard = if plan.server.is_some() {
        Some(server_ports::CREATION.lock().await)
    } else {
        None
    };
    let root = PathManager::get().get_instance_dir().to_path_buf();
    tokio::fs::create_dir_all(&root).await.map_err(error)?;
    worlds::no_link(&root)?;
    let destination = root.join(&new_name);
    let candidate = plan.preview.candidate.clone();
    let server = plan.server.clone();
    let loader = LoaderKind::parse(&candidate.loader).ok_or("Loader no compatible.")?;
    let mc_version = candidate
        .mc_version
        .clone()
        .ok_or("Falta la versión de Minecraft.")?;
    let loader_version = candidate.loader_version.clone().map(|version| {
        version
            .strip_prefix(&format!("{mc_version}-"))
            .unwrap_or(&version)
            .to_string()
    });
    let launch_id = if let Some(version) = &loader_version {
        instance_manager::launch_version_id(&mc_version, loader, version)
    } else {
        mc_version.clone()
    };
    let data: InstanceData = serde_json::from_value(serde_json::json!({
        "uuid": uuid::Uuid::new_v4().to_string(), "name": new_name, "mc_version": mc_version,
        "loader": loader, "loader_version": loader_version, "launch_version_id": launch_id,
        "min_memory": plan.min_memory, "max_memory": plan.max_memory,
        "server_type": server.as_ref().map(|server| server.server_type),
        "server_jar": server.as_ref().map(|server| &server.server_jar), "server_build": server.as_ref().and_then(|server| server.server_build.clone())
    })).map_err(|_| "No se pudo crear el manifiesto de la instancia.")?;
    tokio::fs::create_dir(&destination).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            "Ya existe una instancia con ese nombre. No se reemplazó ningún archivo.".into()
        } else {
            error(e)
        }
    })?;
    let cancel = active.cancel.clone();
    let copy_dest = destination.clone();
    let copied = tokio::task::spawn_blocking(move || copy_plan(&plan, &copy_dest, &cancel))
        .await
        .map_err(|_| "No se pudo completar la copia del origen.".to_string())
        .and_then(|result| result);
    let result = async {
        copied?;
        canceled(&active.cancel)?;
        if let Some(server) = &server {
            let jar = File::open(destination.join(&server.server_jar)).map_err(error)?;
            let jar = zip::ZipArchive::new(jar)
                .map_err(|_| "El JAR del servidor no contiene un archivo Java ZIP válido.")?;
            if jar.is_empty() {
                return Err("El JAR del servidor está vacío.".into());
            }
            let properties_path = destination.join("server.properties");
            let raw = if properties_path.exists() {
                let raw = small_file(&properties_path)?;
                String::from_utf8(raw).map_err(|_| "server.properties no es texto UTF-8 válido.")?
            } else {
                String::new()
            };
            let sanitized = crate::services::instance_archive::portable_properties(&raw)?;
            if properties_path.exists() {
                tokio::fs::remove_file(&properties_path)
                    .await
                    .map_err(error)?;
            }
            server_ports::assign(&destination, &data.name, None).await?;
            use tokio::io::AsyncWriteExt;
            let mut properties = tokio::fs::OpenOptions::new()
                .append(true)
                .open(&properties_path)
                .await
                .map_err(error)?;
            properties
                .write_all(sanitized.as_bytes())
                .await
                .map_err(error)?;
            properties.sync_all().await.map_err(error)?;
        }
        canceled(&active.cancel)?;
        // The manifest is the visibility/commit point. Before it, discovery
        // ignores the reserved directory, and cancellation can remove only ours.
        let bytes = serde_json::to_vec_pretty(&data).map_err(error)?;
        let manifest = destination.join("instance.tfl.json");
        worlds::atomic_replace_file(&manifest, &bytes)?;
        Ok(())
    }
    .await;
    if let Err(reason) = result {
        if !destination.join("instance.tfl.json").exists() {
            let _ = tokio::fs::remove_dir_all(&destination).await;
        }
        return Err(reason);
    }
    drop(server_guard);
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PACK: &[u8] = br#"{"formatVersion":1,"components":[{"uid":"net.minecraft","version":"1.20.1"},{"uid":"net.fabricmc.fabric-loader","version":"0.16.14"}]}"#;

    fn fixture(root: &Path, game: &str) -> PathBuf {
        let dir = root.join(game);
        fs::create_dir_all(dir.join("saves/new world – montaña")).unwrap();
        fs::create_dir_all(dir.join("config")).unwrap();
        fs::write(root.join("mmc-pack.json"), PACK).unwrap();
        fs::write(
            root.join("instance.cfg"),
            "InstanceType=OneSix\nname=Fabric de prueba\nOverrideMemory=true\nMinMemAlloc=512\nMaxMemAlloc=2048",
        )
        .unwrap();
        fs::write(dir.join("saves/new world – montaña/level.dat"), b"world").unwrap();
        fs::write(dir.join("config/settings.toml"), b"option = true").unwrap();
        fs::write(dir.join("config/accounts.json"), b"excluded").unwrap();
        fs::write(dir.join("servers.dat"), b"favorites").unwrap();
        dir
    }

    fn write_zip(path: &Path, files: &[(&str, &[u8])]) {
        let mut zip = zip::ZipWriter::new(File::create(path).unwrap());
        for (name, content) in files {
            zip.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(content).unwrap();
        }
        zip.finish().unwrap();
    }

    #[test]
    fn custom_multimc_family_copies_worlds_and_content_without_credentials() {
        for (family, game) in [("polymc", ".minecraft"), ("freesm", "minecraft")] {
            let temp = MetadataStage::new().unwrap();
            let source = temp.0.join(format!("{family} custom"));
            let game_dir = fixture(&source, game);
            let detected = import_launchers::detect_mmc_style(family, &source).unwrap();
            assert_eq!(detected.candidate.source, family);
            let plan = inspect(&game_dir).unwrap();
            assert!(plan.preview.importable, "{:?}", plan.preview.warnings);
            assert_eq!(plan.preview.format, "multimc-v1");
            assert_eq!(
                plan.preview.candidate.loader_version.as_deref(),
                Some("0.16.14")
            );
            assert_eq!((plan.min_memory, plan.max_memory), (Some(512), Some(2048)));
            assert_eq!(plan.preview.file_count, 3);
            let destination = temp.0.join("copy");
            fs::create_dir(&destination).unwrap();
            copy_plan(&plan, &destination, &AtomicBool::new(false)).unwrap();
            assert_eq!(
                fs::read(destination.join("saves/new world – montaña/level.dat")).unwrap(),
                b"world"
            );
            assert_eq!(
                fs::read(destination.join("servers.dat")).unwrap(),
                b"favorites"
            );
            assert!(!destination.join("config/accounts.json").exists());
            assert!(game_dir.join("config/accounts.json").exists());
        }
    }

    #[test]
    fn installed_curseforge_is_distinct_from_exports_and_unknown_loaders() {
        let temp = MetadataStage::new().unwrap();
        let source = temp.0.join("CurseForge instalado");
        fs::create_dir(&source).unwrap();
        let manifest = source.join("minecraftinstance.json");
        fs::write(
            &manifest,
            br#"{"name":"Pack","gameVersion":"1.20.1","baseModLoader":{"name":"forge-47.2.0"}}"#,
        )
        .unwrap();
        let plan = inspect(&source).unwrap();
        assert!(plan.preview.importable);
        assert_eq!(
            plan.preview.candidate.loader_version.as_deref(),
            Some("47.2.0")
        );
        assert_eq!(plan.preview.format, "curseforge-installed");
        fs::write(
            &manifest,
            br#"{"name":"Pack","gameVersion":"1.20.1","baseModLoader":{"name":"custom-1"}}"#,
        )
        .unwrap();
        assert!(!inspect(&source).unwrap().preview.importable);
        fs::remove_file(manifest).unwrap();
        fs::write(source.join("manifest.json"), b"{}").unwrap();
        assert!(
            inspect(&source)
                .err()
                .unwrap()
                .contains("exportación de CurseForge")
        );
    }

    #[test]
    fn portable_zip_inspection_copy_and_replacement_detection() {
        let temp = MetadataStage::new().unwrap();
        let archive = temp.0.join("Profile.zip");
        let files: &[(&str, &[u8])] = &[
            ("Perfil/mmc-pack.json", PACK),
            ("Perfil/.minecraft/options.txt", b"valid"),
            ("Perfil/.minecraft/accounts.json", b"excluded"),
        ];
        write_zip(&archive, files);
        let plan = inspect(&archive).unwrap();
        assert!(plan.preview.importable);
        assert_eq!(plan.preview.file_count, 1);
        let destination = temp.0.join("copy");
        fs::create_dir(&destination).unwrap();
        copy_plan(&plan, &destination, &AtomicBool::new(false)).unwrap();
        assert_eq!(fs::read(destination.join("options.txt")).unwrap(), b"valid");
        assert!(!destination.join("accounts.json").exists());

        write_zip(
            &archive,
            &[
                files[0],
                ("Perfil/.minecraft/options.txt", b"other"),
                files[2],
            ],
        );
        let changed = temp.0.join("changed");
        fs::create_dir(&changed).unwrap();
        let error = copy_plan(&plan, &changed, &AtomicBool::new(false)).unwrap_err();
        assert!(error.contains("ZIP cambió"), "{error}");
        assert!(!changed.join("options.txt").exists());
    }

    #[test]
    fn ambiguous_layout_missing_loader_and_cancelled_copy_leave_source_intact() {
        let temp = MetadataStage::new().unwrap();
        let source = temp.0.join("instance");
        let game_dir = fixture(&source, "minecraft");
        let destination = temp.0.join("copy");
        fs::create_dir(&destination).unwrap();
        let plan = inspect(&source).unwrap();
        assert!(copy_plan(&plan, &destination, &AtomicBool::new(true)).is_err());
        assert_eq!(fs::read_dir(&destination).unwrap().count(), 0);
        assert!(game_dir.join("config/accounts.json").exists());

        fs::create_dir(source.join(".minecraft")).unwrap();
        assert!(inspect(&source).err().unwrap().contains("dos directorios"));
        fs::remove_dir(source.join(".minecraft")).unwrap();
        fs::write(
            source.join("mmc-pack.json"),
            br#"{"components":[{"uid":"net.minecraft","version":"1.20.1"},{"uid":"net.fabricmc.fabric-loader"}]}"#,
        ).unwrap();
        assert!(!inspect(&source).unwrap().preview.importable);
    }

    #[test]
    fn supported_server_preview_requires_a_jar_and_excludes_private_data() {
        let temp = MetadataStage::new().unwrap();
        let archive = temp.0.join("portable.tflserver.zip");
        let server: &[u8] = br#"{"format_version":1,"name":"Paper","mc_version":"1.21.5","server_type":"paper","server_jar":"paper.jar","server_build":"1"}"#;
        write_zip(&archive, &[("tfl-server.json", server)]);
        assert!(!inspect(&archive).unwrap().preview.importable);
        write_zip(
            &archive,
            &[
                ("tfl-server.json", server),
                ("paper.jar", b"fixture"),
                ("server.properties", b"online-mode=true\n"),
                ("plugins/test.jar", b"plugin"),
                ("eula.txt", b"eula=true"),
                ("ops.json", b"[]"),
            ],
        );
        let plan = inspect(&archive).unwrap();
        assert!(plan.preview.importable);
        assert_eq!(plan.preview.server_type, Some(ServerType::Paper));
        assert_eq!(
            plan.preview.included,
            ["paper.jar", "plugins", "server.properties"]
        );
        assert!(plan.preview.omitted.contains(&"eula.txt".to_string()));
        assert!(plan.preview.omitted.contains(&"ops.json".to_string()));
    }
    #[tokio::test]
    #[ignore = "requires official checksum-verified Paper fixture supplied by CI"]
    async fn official_paper_export_import_preserves_jar_and_auth_policy() {
        use sha2::{Digest, Sha256};
        let profile = std::env::var_os("TFL_DATA_DIR")
            .map(PathBuf::from)
            .expect("isolated profile");
        assert!(profile.join("TEST_PROFILE").is_file());
        PathManager::ensure_dirs().unwrap();
        let jar_path = std::env::var_os("TFL_PAPER_TEST_JAR")
            .map(PathBuf::from)
            .expect("Paper fixture");
        let expected = std::env::var("TFL_PAPER_TEST_SHA256").expect("official SHA-256");
        let jar = fs::read(&jar_path).unwrap();
        assert_eq!(hex::encode(Sha256::digest(&jar)), expected);
        let nonce = uuid::Uuid::new_v4();
        let source_name = format!("Paper original {nonce}");
        let imported_name = format!("Paper importado {nonce}");
        let data = instance_manager::create_instance_full(
            source_name.clone(),
            "1.21.4".into(),
            LoaderKind::Vanilla,
            None,
            "1.21.4".into(),
            Some(ServerType::Paper),
            Some("paper.jar".into()),
            Some("232".into()),
        )
        .await
        .unwrap();
        fs::write(data.dir().join("paper.jar"), &jar).unwrap();
        fs::write(
            data.dir().join("server.properties"),
            "online-mode=true\nrcon.password=private\nserver-ip=192.0.2.1\ndifficulty=hard\n",
        )
        .unwrap();
        fs::write(data.dir().join("eula.txt"), "eula=false\n").unwrap();
        let exported = crate::services::instance_archive::create(source_name, true)
            .await
            .unwrap();
        let preview = inspect(Path::new(&exported)).unwrap().preview;
        assert!(preview.importable);
        let imported =
            commit_instance_source(exported, imported_name, Some(preview.fingerprint), None)
                .await
                .unwrap();
        let copied = fs::read(imported.dir().join("paper.jar")).unwrap();
        assert_eq!(hex::encode(Sha256::digest(&copied)), expected);
        assert_eq!(imported.server_type, Some(ServerType::Paper));
        assert!(imported.preparation.is_none());
        let properties = server_ports::read(&imported.dir()).await.unwrap();
        let parsed = server_ports::properties(&properties).unwrap();
        assert_eq!(parsed.get("online-mode").map(String::as_str), Some("true"));
        assert_eq!(parsed.get("difficulty").map(String::as_str), Some("hard"));
        assert!(!parsed.contains_key("rcon.password"));
        assert!(!parsed.contains_key("server-ip"));
        assert!(!imported.dir().join("eula.txt").exists());
        assert_eq!(
            fs::read_to_string(data.dir().join("eula.txt")).unwrap(),
            "eula=false\n"
        );
        println!(
            "Paper 1.21.4 build 232 retained SHA-256 {expected} through server export and import"
        );
    }
}
