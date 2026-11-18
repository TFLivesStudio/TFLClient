//! World operations share the launch/content mutation lock at the command boundary.
//! Paths are resolved from the instance game directory, never by a disk-wide search.
use fastnbt::Value;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

const MAX_LEVEL: u64 = 16 * 1024 * 1024;
const MAX_ARCHIVE: u64 = 32 * 1024 * 1024 * 1024;
const MAX_PACK: u64 = 128 * 1024 * 1024;
const MAX_ENTRIES: usize = 50_000;

#[derive(Debug, Clone, Serialize)]
pub struct WorldInfo {
    /// Folder identity; the visible NBT LevelName is independent.
    pub name: String,
    pub display_name: String,
    pub version: Option<String>,
    pub data_version: Option<i32>,
    pub size_bytes: Option<u64>,
    pub metadata_error: Option<String>,
    pub dimensions: Vec<String>,
    pub server_primary: bool,
    pub missing: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldBackup {
    pub id: String,
    pub created_at: u64,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Datapack {
    pub id: String,
    pub enabled: bool,
    pub pack_format: Option<u64>,
    pub min_format: Option<serde_json::Value>,
    pub max_format: Option<serde_json::Value>,
    pub world_version: Option<String>,
    pub compatibility: String,
    pub error: Option<String>,
}

pub fn ensure_stopped(name: &str) -> Result<(), String> {
    if crate::services::launcher::running_instances()
        .iter()
        .any(|(instance, _)| instance == name)
        || crate::services::server_process::is_server_running(name)
    {
        return Err("Detén esta instancia antes de modificar o respaldar sus mundos. Espera a que termine de guardar.".into());
    }
    Ok(())
}

fn io_error(action: &str, error: std::io::Error) -> String {
    format!("{action}: {error}")
}

pub(crate) fn no_link(path: &Path) -> Result<(), String> {
    let meta = fs::symlink_metadata(path).map_err(|e| io_error("No se pudo leer la ruta", e))?;
    if meta.file_type().is_symlink() {
        return Err(
            "No se admiten enlaces simbólicos ni rutas redirigidas en esta operación.".into(),
        );
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err(
                "No se admiten junctions ni puntos de reanálisis en esta operación.".into(),
            );
        }
    }
    Ok(())
}

pub fn valid_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 240
        || name == "."
        || name == ".."
        || name.ends_with(['.', ' '])
        || name
            .chars()
            .any(|c| c.is_control() || "/\\:*?\"<>|".contains(c))
        || name.split('.').next().is_some_and(|part| {
            let part = part.to_ascii_uppercase();
            ["CON", "PRN", "AUX", "NUL"].contains(&part.as_str())
                || (part.len() == 4
                    && (part.starts_with("COM") || part.starts_with("LPT"))
                    && matches!(part.as_bytes()[3], b'1'..=b'9'))
        })
    {
        return Err("El nombre de carpeta o archivo no es válido.".into());
    }
    Ok(())
}

/// Current imports copy their game files into root. These two bounded legacy
/// locations cover older MultiMC-family imports; ambiguity is an error.
pub fn resolve_game_dir(root: &Path, server: bool) -> Result<PathBuf, String> {
    no_link(root)?;
    if server
        || root
            .join("saves")
            .try_exists()
            .map_err(|e| io_error("No se pudo consultar saves", e))?
    {
        return Ok(root.to_path_buf());
    }
    let mut candidates = Vec::new();
    for name in [".minecraft", "minecraft"] {
        let candidate = root.join(name);
        if candidate
            .try_exists()
            .map_err(|e| io_error("No se pudo consultar el directorio de juego", e))?
        {
            no_link(&candidate)?;
            if candidate
                .join("saves")
                .try_exists()
                .map_err(|e| io_error("No se pudo consultar saves", e))?
            {
                candidates.push(candidate);
            }
        }
    }
    match candidates.len() {
        0 => Ok(root.to_path_buf()),
        1 => Ok(candidates.remove(0)),
        _ => Err("Hay varios directorios de juego importados. Revisa la estructura antes de administrar los mundos.".into()),
    }
}

pub fn worlds_root(root: &Path, server: bool) -> Result<PathBuf, String> {
    let game = resolve_game_dir(root, server)?;
    let result = if server { game } else { game.join("saves") };
    if result
        .try_exists()
        .map_err(|e| io_error("No se pudo consultar la carpeta de mundos", e))?
    {
        no_link(&result)?;
    }
    Ok(result)
}

fn find_level(world: &Path) -> Result<Option<PathBuf>, String> {
    // Windows is case-insensitive; also inspect imported case variants on Unix.
    let mut found = None;
    for entry in fs::read_dir(world).map_err(|e| io_error("No se pudo leer el mundo", e))? {
        let entry = entry.map_err(|e| io_error("No se pudo leer un archivo del mundo", e))?;
        if entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case("level.dat")
        {
            if found.is_some() {
                return Err("Hay varios archivos level.dat con distinta capitalización.".into());
            }
            no_link(&entry.path())?;
            found = Some(entry.path());
        }
    }
    Ok(found)
}

fn level_path(world: &Path) -> Result<PathBuf, String> {
    find_level(world)?.ok_or_else(|| "Esta carpeta no contiene level.dat.".into())
}

pub fn world_path(root: &Path, server: bool, name: &str) -> Result<PathBuf, String> {
    validate_world_id(name, server)?;
    let parent = worlds_root(root, server)?;
    let mut ancestor = parent.clone();
    for component in name.split('/') {
        ancestor.push(component);
        no_link(&ancestor)?;
    }
    let world = parent.join(name);
    no_link(&world)?;
    if !world.is_dir() {
        return Err("La ruta del mundo no es una carpeta.".into());
    }
    level_path(&world)?;
    Ok(world)
}

fn validate_world_id(name: &str, server: bool) -> Result<(), String> {
    if !server {
        return valid_name(name);
    }
    if name.len() > 1024 || name.split('/').count() > 16 {
        return Err("La ruta del mundo es demasiado larga.".into());
    }
    for component in name.split('/') {
        valid_name(component)?;
    }
    Ok(())
}

fn read_level(world: &Path) -> Result<(Value, Vec<u8>, bool), String> {
    let path = level_path(world)?;
    let file = File::open(path).map_err(|e| io_error("No se pudo abrir level.dat", e))?;
    if file
        .metadata()
        .map_err(|e| io_error("No se pudo consultar level.dat", e))?
        .len()
        > MAX_LEVEL
    {
        return Err("level.dat supera el límite de 16 MiB.".into());
    }
    let mut stored = Vec::new();
    file.take(MAX_LEVEL + 1)
        .read_to_end(&mut stored)
        .map_err(|e| io_error("No se pudo leer level.dat", e))?;
    let gzip = stored.starts_with(&[0x1f, 0x8b]);
    let raw = if gzip {
        let mut raw = Vec::new();
        GzDecoder::new(&stored[..])
            .take(MAX_LEVEL + 1)
            .read_to_end(&mut raw)
            .map_err(|_| "level.dat contiene un GZIP inválido.".to_string())?;
        raw
    } else {
        stored
    };
    if raw.len() as u64 > MAX_LEVEL || raw.len() < 3 || raw[0] != 10 {
        return Err("level.dat no es un compuesto NBT Java válido o supera el límite.".into());
    }
    let prefix_len = 3 + u16::from_be_bytes([raw[1], raw[2]]) as usize;
    let prefix = raw
        .get(..prefix_len)
        .ok_or("Nombre raíz NBT truncado.")?
        .to_vec();
    // Bound nesting and sequence allocation before asking Serde to own the
    // document. A corrupt level.dat must not exhaust the worker stack/heap.
    validate_nbt_shape(&raw, prefix_len)?;
    let value =
        fastnbt::from_bytes_with_opts::<Value>(&raw, fastnbt::DeOpts::new().max_seq_len(1_000_000))
            .map_err(|_| "No se pudieron leer los metadatos NBT de level.dat.")?;
    Ok((value, prefix, gzip))
}

fn validate_nbt_shape(raw: &[u8], prefix_len: usize) -> Result<(), String> {
    struct Cursor<'a> {
        raw: &'a [u8],
        pos: usize,
        nodes: usize,
    }
    impl<'a> Cursor<'a> {
        fn take(&mut self, size: usize) -> Result<&'a [u8], String> {
            let end = self.pos.checked_add(size).ok_or("NBT demasiado grande.")?;
            let result = self.raw.get(self.pos..end).ok_or("NBT truncado.")?;
            self.pos = end;
            Ok(result)
        }
        fn name(&mut self) -> Result<&'a [u8], String> {
            let length = self.take(2)?;
            let length = u16::from_be_bytes([length[0], length[1]]) as usize;
            self.take(length)
        }
        fn sequence_len(&mut self) -> Result<usize, String> {
            let length = self.take(4)?;
            let length = i32::from_be_bytes([length[0], length[1], length[2], length[3]]);
            if !(0..=1_000_000).contains(&length) {
                return Err("La secuencia NBT supera el límite seguro.".into());
            }
            Ok(length as usize)
        }
        fn payload(&mut self, tag: u8, depth: usize) -> Result<(), String> {
            self.nodes += 1;
            if depth > 96 || self.nodes > 1_000_000 {
                return Err("La estructura NBT supera el límite seguro.".into());
            }
            match tag {
                1 => {
                    self.take(1)?;
                }
                2 => {
                    self.take(2)?;
                }
                3 | 5 => {
                    self.take(4)?;
                }
                4 | 6 => {
                    self.take(8)?;
                }
                7 | 11 | 12 => {
                    let length = self.sequence_len()?;
                    let width = match tag {
                        7 => 1,
                        11 => 4,
                        _ => 8,
                    };
                    self.take(length * width)?;
                }
                8 => {
                    self.name()?;
                }
                9 => {
                    let item_tag = self.take(1)?[0];
                    let length = self.sequence_len()?;
                    if item_tag > 12 || (item_tag == 0 && length != 0) {
                        return Err("Tipo de lista NBT inválido.".into());
                    }
                    for _ in 0..length {
                        self.payload(item_tag, depth + 1)?;
                    }
                }
                10 => {
                    let mut names = HashSet::new();
                    loop {
                        let item_tag = self.take(1)?[0];
                        if item_tag == 0 {
                            break;
                        }
                        if !names.insert(self.name()?) {
                            return Err("El compuesto NBT contiene claves duplicadas.".into());
                        }
                        self.payload(item_tag, depth + 1)?;
                    }
                }
                _ => return Err("Tipo de etiqueta NBT inválido.".into()),
            }
            Ok(())
        }
    }
    let mut cursor = Cursor {
        raw,
        pos: prefix_len,
        nodes: 0,
    };
    cursor.payload(10, 0)?;
    if cursor.pos != raw.len() {
        return Err("level.dat contiene bytes adicionales no reconocidos.".into());
    }
    Ok(())
}

fn compound(value: &Value) -> Option<&std::collections::HashMap<String, Value>> {
    match value {
        Value::Compound(map) => Some(map),
        _ => None,
    }
}
fn nbt_string(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(text)) => Some(text.clone()),
        _ => None,
    }
}

pub fn world_version(world: &Path) -> Result<Option<String>, String> {
    let (value, _, _) = read_level(world)?;
    Ok(version_from_level(&value))
}

fn version_from_level(value: &Value) -> Option<String> {
    compound(value)
        .and_then(|root| root.get("Data"))
        .and_then(compound)
        .and_then(|data| data.get("Version"))
        .and_then(compound)
        .and_then(|version| nbt_string(version.get("Name")))
}

fn tree_files(root: &Path, max_bytes: u64) -> Result<Vec<(PathBuf, u64)>, String> {
    no_link(root)?;
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    let mut total = 0u64;
    let mut seen = 0usize;
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|e| io_error("No se pudo leer el contenido", e))? {
            let entry = entry.map_err(|e| io_error("No se pudo leer una entrada", e))?;
            seen += 1;
            if seen > MAX_ENTRIES {
                return Err("El contenido supera 50 000 entradas.".into());
            }
            let path = entry.path();
            no_link(&path)?;
            let meta = fs::symlink_metadata(&path)
                .map_err(|e| io_error("No se pudo consultar el contenido", e))?;
            if meta.is_dir() {
                stack.push(path.clone());
                files.push((path, 0));
            } else if meta.is_file() {
                total = total
                    .checked_add(meta.len())
                    .filter(|n| *n <= max_bytes)
                    .ok_or("El contenido supera el límite de tamaño.")?;
                files.push((path, meta.len()));
            } else {
                return Err("El contenido incluye un archivo especial no admitido.".into());
            }
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

pub fn list_worlds(root: &Path, server: bool) -> Result<Vec<WorldInfo>, String> {
    let dir = worlds_root(root, server)?;
    if !dir
        .try_exists()
        .map_err(|e| io_error("No se pudo consultar la carpeta de mundos", e))?
    {
        return Ok(Vec::new());
    }
    let primary = if server {
        let path = root.join("server.properties");
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_file() && meta.len() <= 1024 * 1024 => {
                no_link(&path)?;
                let mut raw = String::new();
                File::open(&path)
                    .and_then(|file| file.take(1024 * 1024 + 1).read_to_string(&mut raw))
                    .map_err(|e| io_error("No se pudo leer server.properties", e))?;
                if raw.len() > 1024 * 1024 {
                    return Err("server.properties supera el límite de 1 MiB.".into());
                }
                super::server_ports::properties(&raw)?
                    .remove("level-name")
                    .unwrap_or_else(|| "world".into())
            }
            Ok(_) => return Err("server.properties no es un archivo normal de hasta 1 MiB.".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => "world".into(),
            Err(e) => return Err(io_error("No se pudo leer server.properties", e)),
        }
    } else {
        String::new()
    };
    let mut candidates = std::collections::BTreeMap::new();
    for entry in
        fs::read_dir(&dir).map_err(|e| io_error("No se pudo leer la carpeta de mundos", e))?
    {
        let entry = entry.map_err(|e| io_error("No se pudo leer una carpeta de mundo", e))?;
        let path = entry.path();
        let meta = fs::symlink_metadata(&path)
            .map_err(|e| io_error("No se pudo consultar una carpeta de mundo", e))?;
        if !meta.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        no_link(&path)?;
        candidates.insert(name, path);
    }
    if server && primary.contains('/') {
        validate_world_id(&primary, true)?;
        for suffix in ["", "_nether", "_the_end"] {
            let id = format!("{primary}{suffix}");
            let path = dir.join(&id);
            if path
                .try_exists()
                .map_err(|e| io_error("No se pudo consultar level-name", e))?
            {
                // Validate each ancestor, not just the final nested directory.
                world_path(root, true, &id)?;
                candidates.insert(id, path);
            }
        }
    }
    let mut result = Vec::new();
    for (name, path) in candidates {
        if find_level(&path)?.is_none() {
            continue;
        }
        no_link(&path)?;
        let mut info = WorldInfo {
            name: name.clone(),
            display_name: name.clone(),
            version: None,
            data_version: None,
            size_bytes: None,
            metadata_error: None,
            dimensions: Vec::new(),
            server_primary: server && name == primary,
            missing: false,
        };
        match read_level(&path) {
            Ok((value, _, _)) => {
                if let Some(data) = compound(&value)
                    .and_then(|root| root.get("Data"))
                    .and_then(compound)
                {
                    info.display_name = nbt_string(data.get("LevelName"))
                        .filter(|name| !name.is_empty())
                        .unwrap_or(name);
                    info.version = data
                        .get("Version")
                        .and_then(compound)
                        .and_then(|version| nbt_string(version.get("Name")));
                    info.data_version = match data.get("DataVersion") {
                        Some(Value::Int(version)) => Some(*version),
                        _ => None,
                    };
                } else {
                    info.metadata_error = Some("level.dat no contiene el compuesto Data.".into());
                }
            }
            Err(error) => info.metadata_error = Some(error),
        }
        match tree_files(&path, MAX_ARCHIVE) {
            Ok(files) => info.size_bytes = Some(files.iter().map(|(_, size)| size).sum()),
            Err(error) => info.metadata_error = Some(error),
        }
        for dimension in ["DIM-1", "DIM1", "dimensions"] {
            if path.join(dimension).is_dir() {
                info.dimensions.push(dimension.into());
            }
        }
        result.push(info);
    }
    append_recoverable_worlds(root, server, &mut result)?;
    result.sort_by(|a, b| {
        b.server_primary
            .cmp(&a.server_primary)
            .then(
                a.display_name
                    .to_lowercase()
                    .cmp(&b.display_name.to_lowercase()),
            )
            .then(a.name.cmp(&b.name))
    });
    Ok(result)
}

/// Held throughout an operation. A locked session.lock is an extra native
/// guard; TFL's process registry and shared mutation lock remain authoritative.
fn session_guard(world: &Path) -> Result<Option<File>, String> {
    let path = world.join("session.lock");
    if !path
        .try_exists()
        .map_err(|e| io_error("No se pudo consultar session.lock", e))?
    {
        return Ok(None);
    }
    no_link(&path)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|e| io_error("No se pudo comprobar si el mundo está en uso", e))?;
    #[cfg(windows)]
    file.try_lock().map_err(|_| "El mundo está bloqueado por otro proceso. Cierra Minecraft o el servidor antes de continuar.")?;
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        // Use Java FileChannel's POSIX record-lock mechanism. flock is a
        // separate lock on Linux and can conflict with our own record lock on
        // macOS, so taking both would reject even a closed world.
        // SAFETY: libc::flock consists solely of integer fields; zero is a
        // valid initial value. fcntl receives a live fd and a valid pointer.
        let mut lock: libc::flock = unsafe { std::mem::zeroed() };
        lock.l_type = libc::F_WRLCK as _;
        lock.l_whence = libc::SEEK_SET as _;
        lock.l_start = 0;
        lock.l_len = 0;
        if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETLK, &lock) } == -1 {
            return Err(
                "El mundo está bloqueado por Java u otro proceso. Ciérralo antes de continuar."
                    .into(),
            );
        }
    }
    Ok(Some(file))
}

pub(crate) fn lock_world_sessions(saves: &Path) -> Result<Vec<File>, String> {
    let mut guards = Vec::new();
    for entry in fs::read_dir(saves)
        .map_err(|e| io_error("No se pudieron comprobar los mundos en uso", e))?
    {
        let entry = entry.map_err(|e| io_error("No se pudo comprobar un mundo en uso", e))?;
        if entry.path().is_dir() {
            no_link(&entry.path())?;
            if let Some(file) = session_guard(&entry.path())? {
                guards.push(file);
            }
        }
    }
    Ok(guards)
}

pub fn rename_world(
    root: &Path,
    server: bool,
    folder: &str,
    display_name: &str,
) -> Result<(), String> {
    let display_name = display_name.trim();
    if display_name.is_empty()
        || display_name.chars().count() > 128
        || display_name.chars().any(char::is_control)
    {
        return Err(
            "Escribe un nombre de mundo de 1 a 128 caracteres, sin caracteres de control.".into(),
        );
    }
    let world = world_path(root, server, folder)?;
    let _session = session_guard(&world)?;
    let (mut value, prefix, gzip) = read_level(&world)?;
    let Value::Compound(ref mut map) = value else {
        return Err("NBT raíz inválido.".into());
    };
    let Some(Value::Compound(data)) = map.get_mut("Data") else {
        return Err("El mundo no contiene metadatos Data válidos.".into());
    };
    data.insert("LevelName".into(), Value::String(display_name.into()));
    write_level(&world, &value, prefix, gzip)
}

fn write_level(world: &Path, value: &Value, prefix: Vec<u8>, gzip: bool) -> Result<(), String> {
    let encoded =
        fastnbt::to_bytes(value).map_err(|_| "No se pudieron escribir los metadatos NBT.")?;
    let mut raw = prefix;
    raw.extend_from_slice(encoded.get(3..).ok_or("NBT serializado inválido.")?);
    let bytes = if gzip {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&raw)
            .map_err(|e| io_error("No se pudo comprimir level.dat", e))?;
        encoder
            .finish()
            .map_err(|e| io_error("No se pudo terminar level.dat", e))?
    } else {
        raw
    };
    let level = level_path(world)?;
    atomic_replace_file(&level, &bytes)
}

/// Keep the old file in place until the complete replacement is durable.
pub fn atomic_replace_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("Ruta sin directorio padre.")?;
    no_link(parent)?;
    match fs::symlink_metadata(path) {
        Ok(_) => no_link(path)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        Err(error) => return Err(io_error("No se pudo consultar el destino", error)),
    }
    let staging = parent.join(format!(".tfl-stage-{}", uuid::Uuid::new_v4()));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&staging)
        .map_err(|e| io_error("No se pudo preparar el archivo", e))?;
    if let Err(error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        let _ = fs::remove_file(&staging);
        return Err(io_error("No se pudo guardar el archivo completo", error));
    }
    drop(file);
    if let Err(error) = fs::rename(&staging, path) {
        let _ = fs::remove_file(&staging);
        return Err(io_error("No se pudo reemplazar el archivo", error));
    }
    Ok(())
}

pub(crate) fn archive_rel(raw: &str) -> Result<PathBuf, String> {
    if raw.is_empty() || raw.contains('\\') || raw.starts_with('/') || raw.contains(':') {
        return Err("El ZIP contiene una ruta no segura.".into());
    }
    let path = Path::new(raw);
    for part in path.components() {
        match part {
            Component::Normal(name) => valid_name(name.to_str().ok_or("Nombre ZIP no válido.")?)?,
            _ => return Err("El ZIP contiene una ruta no segura.".into()),
        }
    }
    // Path::components normalizes internal '.', reject them explicitly.
    if raw.split('/').any(|part| part == "." || part == "..") {
        return Err("El ZIP contiene una ruta relativa no segura.".into());
    }
    Ok(path.to_path_buf())
}

pub(crate) fn inspect_zip(archive: &mut zip::ZipArchive<File>, max: u64) -> Result<(), String> {
    if archive.len() > MAX_ENTRIES {
        return Err("El ZIP supera 50 000 entradas.".into());
    }
    let mut bytes = 0u64;
    let mut names = HashSet::new();
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|_| "No se pudo leer el índice ZIP.")?;
        let name = entry
            .name()
            .map_err(|_| "El ZIP contiene un nombre de archivo inválido.")?;
        let rel = archive_rel(&name)?;
        let key = rel.to_string_lossy().replace('\\', "/").to_lowercase();
        if !names.insert(key) {
            return Err("El ZIP contiene rutas duplicadas o equivalentes en Windows.".into());
        }
        if entry.unix_mode().is_some_and(|mode| {
            let kind = mode & 0o170000;
            kind != 0 && kind != 0o100000 && kind != 0o040000
        }) {
            return Err("El ZIP contiene enlaces o archivos especiales.".into());
        }
        bytes = bytes
            .checked_add(entry.size())
            .filter(|size| *size <= max)
            .ok_or("El ZIP supera el límite descomprimido.")?;
    }
    Ok(())
}

/// Verify every CRC before publishing the archive. Reject unsafe entries rather
/// than silently restoring a partial world from a malicious ZIP.
pub fn verify_archive(path: &Path, max: u64) -> Result<(), String> {
    no_link(path)?;
    let file = File::open(path).map_err(|e| io_error("No se pudo abrir el respaldo", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| "El respaldo no es un ZIP válido.")?;
    inspect_zip(&mut archive, max)?;
    let mut total = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|_| "No se pudo leer una entrada ZIP.")?;
        let count = std::io::copy(
            &mut (&mut entry).take(max - total + 1),
            &mut std::io::sink(),
        )
        .map_err(|_| "El ZIP está truncado o no supera la verificación CRC.")?;
        total += count;
        if total > max {
            return Err("El ZIP supera el límite descomprimido.".into());
        }
    }
    Ok(())
}

fn copy_planned_file(path: &Path, output: &mut impl Write, planned: u64) -> Result<(), String> {
    no_link(path)?;
    let mut file =
        File::open(path).map_err(|e| io_error("No se pudo abrir un archivo del mundo", e))?;
    let metadata = file
        .metadata()
        .map_err(|e| io_error("No se pudo consultar un archivo del mundo", e))?;
    if !metadata.is_file() || metadata.len() != planned {
        return Err("Un archivo del mundo cambió de tamaño. Vuelve a crear el respaldo.".into());
    }
    let copied = std::io::copy(&mut (&mut file).take(planned + 1), output)
        .map_err(|e| io_error("No se pudo completar el respaldo", e))?;
    if copied != planned
        || file
            .metadata()
            .map_err(|e| io_error("No se pudo verificar el archivo respaldado", e))?
            .len()
            != planned
    {
        return Err("Un archivo del mundo cambió de tamaño. Vuelve a crear el respaldo.".into());
    }
    Ok(())
}

pub fn archive_directory(source: &Path, dest: &Path) -> Result<(), String> {
    let files = tree_files(source, MAX_ARCHIVE)?;
    let parent = dest.parent().ok_or("Destino de respaldo inválido.")?;
    fs::create_dir_all(parent)
        .map_err(|e| io_error("No se pudo crear la carpeta de respaldos", e))?;
    no_link(parent)?;
    let staging = parent.join(format!(".tfl-backup-{}.part", uuid::Uuid::new_v4()));
    let result = (|| {
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&staging)
            .map_err(|e| io_error("No se pudo preparar el respaldo", e))?;
        let mut zip = zip::ZipWriter::new(file);
        for (path, planned) in files {
            // session.lock describes a live OS lock, never durable world data.
            if path.file_name().is_some_and(|name| name == "session.lock") {
                continue;
            }
            let rel = path
                .strip_prefix(source)
                .map_err(|_| "Ruta de respaldo inválida.")?
                .to_str()
                .ok_or("Ruta de respaldo no UTF-8.")?
                .replace('\\', "/");
            archive_rel(&rel)?;
            no_link(&path)?;
            if path.is_dir() {
                zip.add_directory(format!("{rel}/"), zip::write::SimpleFileOptions::default())
                    .map_err(|_| "No se pudo conservar una carpeta vacía del mundo.")?;
                continue;
            }
            zip.start_file(
                rel,
                zip::write::SimpleFileOptions::default().large_file(planned >= u64::from(u32::MAX)),
            )
            .map_err(|_| "No se pudo crear una entrada del respaldo.")?;
            copy_planned_file(&path, &mut zip, planned)?;
        }
        zip.finish()
            .map_err(|_| "No se pudo cerrar el respaldo.")?
            .sync_all()
            .map_err(|e| io_error("No se pudo confirmar el respaldo en disco", e))?;
        verify_archive(&staging, MAX_ARCHIVE)?;
        if dest
            .try_exists()
            .map_err(|e| io_error("No se pudo consultar el destino del respaldo", e))?
        {
            return Err("Ya existe ese respaldo; no se reemplazó.".into());
        }
        fs::rename(&staging, dest).map_err(|e| io_error("No se pudo publicar el respaldo", e))
    })();
    if result.is_err() {
        let _ = fs::remove_file(staging);
    }
    result
}

pub fn restore_directory(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let parent = destination
        .parent()
        .ok_or("Destino de restauración inválido.")?;
    no_link(parent)?;
    if destination.exists() {
        no_link(destination)?;
    }
    let mut archive = zip::ZipArchive::new(
        File::open(archive_path).map_err(|e| io_error("No se pudo abrir el respaldo", e))?,
    )
    .map_err(|_| "El respaldo no es un ZIP válido.")?;
    inspect_zip(&mut archive, MAX_ARCHIVE)?;
    let staging = parent.join(format!(".tfl-restore-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&staging).map_err(|e| io_error("No se pudo preparar la restauración", e))?;
    let extraction = (|| {
        let mut total = 0u64;
        for index in 0..archive.len() {
            let mut entry = archive
                .by_index(index)
                .map_err(|_| "No se pudo leer el respaldo.")?;
            let name = entry
                .name()
                .map_err(|_| "El ZIP contiene un nombre de archivo inválido.")?;
            let path = staging.join(archive_rel(&name)?);
            if entry.is_dir() {
                fs::create_dir_all(path)
                    .map_err(|e| io_error("No se pudo crear una carpeta restaurada", e))?;
                continue;
            }
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| io_error("No se pudo crear una carpeta restaurada", e))?;
            }
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(path)
                .map_err(|e| io_error("No se pudo crear un archivo restaurado", e))?;
            let count = std::io::copy(&mut (&mut entry).take(MAX_ARCHIVE - total + 1), &mut file)
                .map_err(|_| "Falló la verificación CRC del respaldo.")?;
            total += count;
            if total > MAX_ARCHIVE {
                return Err("El respaldo supera 32 GiB descomprimidos.".into());
            }
            file.sync_all()
                .map_err(|e| io_error("No se pudo confirmar un archivo restaurado", e))?;
        }
        Ok(())
    })();
    if let Err(error) = extraction {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    let old = parent.join(format!(".tfl-replaced-{}", uuid::Uuid::new_v4()));
    let existed = destination.exists();
    if existed {
        if let Err(error) = fs::rename(destination, &old) {
            let _ = fs::remove_dir_all(&staging);
            return Err(io_error("No se pudo conservar el mundo actual", error));
        }
    }
    if let Err(error) = fs::rename(&staging, destination) {
        if existed {
            fs::rename(&old, destination).map_err(|_| {
                format!(
                    "Se conserva el original para recuperación en {}",
                    old.display()
                )
            })?;
        }
        let _ = fs::remove_dir_all(staging);
        return Err(io_error("No se pudo activar la restauración", error));
    }
    if existed {
        let _ = fs::remove_dir_all(old);
    }
    Ok(())
}

fn backup_dir(root: &Path, folder: &str) -> Result<PathBuf, String> {
    validate_world_id(folder, true)?;
    let mut result = root.to_path_buf();
    for part in [
        ".tfl_backups".to_string(),
        "worlds".into(),
        hex::encode(Sha256::digest(folder.as_bytes())),
    ] {
        result.push(part);
        if result.exists() {
            no_link(&result)?;
        }
    }
    Ok(result)
}

fn append_recoverable_worlds(
    root: &Path,
    server: bool,
    result: &mut Vec<WorldInfo>,
) -> Result<(), String> {
    let catalogue = root.join(".tfl_backups").join("worlds");
    if !catalogue.exists() {
        return Ok(());
    }
    no_link(&root.join(".tfl_backups"))?;
    no_link(&catalogue)?;
    let mut count = 0;
    for entry in fs::read_dir(&catalogue)
        .map_err(|e| io_error("No se pudo leer el catálogo de respaldos", e))?
    {
        count += 1;
        if count > 5000 {
            return Err("El catálogo de mundos supera el límite de 5000 carpetas.".into());
        }
        let entry = entry.map_err(|e| io_error("No se pudo leer una entrada del catálogo", e))?;
        no_link(&entry.path())?;
        let metadata = entry.path().join("world.json");
        if !metadata.is_file() {
            continue;
        }
        no_link(&metadata)?;
        let file = File::open(&metadata)
            .map_err(|e| io_error("No se pudo abrir el catálogo del mundo", e))?;
        if file
            .metadata()
            .map_err(|e| io_error("No se pudo consultar el catálogo", e))?
            .len()
            > 16384
        {
            return Err("La entrada del catálogo del mundo supera el límite.".into());
        }
        let data: serde_json::Value = serde_json::from_reader(file.take(16385))
            .map_err(|_| "El catálogo de respaldos contiene una entrada inválida.")?;
        let name = data["folder"]
            .as_str()
            .ok_or("El catálogo no identifica la carpeta de mundo.")?;
        validate_world_id(name, server)?;
        if data["server"].as_bool() != Some(server)
            || entry.file_name().to_string_lossy() != hex::encode(Sha256::digest(name.as_bytes()))
        {
            continue;
        }
        if !result.iter().any(|world| world.name == name) && !list_backups(root, name)?.is_empty() {
            result.push(WorldInfo {
                name: name.into(),
                display_name: data["display_name"].as_str().unwrap_or(name).into(),
                version: data["version"].as_str().map(String::from),
                data_version: None,
                size_bytes: None,
                metadata_error: None,
                dimensions: Vec::new(),
                server_primary: false,
                missing: true,
            });
        }
    }
    Ok(())
}
fn backup_timestamp(id: &str) -> Option<u64> {
    let raw = id.strip_prefix("world-")?.strip_suffix(".zip")?;
    let (timestamp, uuid) = raw.split_once('-')?;
    uuid::Uuid::parse_str(uuid).ok()?;
    timestamp.parse().ok()
}
fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| io_error("No se pudo verificar el respaldo", e))?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|e| io_error("No se pudo verificar el respaldo", e))?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hex::encode(hash.finalize()))
}
fn describe_backup(path: &Path, id: &str) -> Result<WorldBackup, String> {
    no_link(path)?;
    Ok(WorldBackup {
        id: id.into(),
        created_at: backup_timestamp(id).ok_or("Identificador de respaldo inválido.")?,
        size_bytes: fs::metadata(path)
            .map_err(|e| io_error("No se pudo consultar el respaldo", e))?
            .len(),
        sha256: hash_file(path)?,
    })
}
pub fn backup_world(root: &Path, server: bool, folder: &str) -> Result<WorldBackup, String> {
    let world = world_path(root, server, folder)?;
    let _session = session_guard(&world)?;
    let dir = backup_dir(root, folder)?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| "La hora del sistema no es válida.")?
        .as_secs();
    let id = format!("world-{timestamp}-{}.zip", uuid::Uuid::new_v4().simple());
    archive_directory(&world, &dir.join(&id))?;
    let info = describe_backup(&dir.join(&id), &id)?;
    atomic_replace_file(&dir.join(format!("{id}.sha256")), info.sha256.as_bytes())?;
    let display_name = read_level(&world)
        .ok()
        .and_then(|(value, _, _)| {
            compound(&value)
                .and_then(|root| root.get("Data"))
                .and_then(compound)
                .and_then(|data| nbt_string(data.get("LevelName")))
        })
        .unwrap_or_else(|| folder.into());
    let metadata = serde_json::to_vec(&serde_json::json!({"folder": folder, "server": server, "display_name": display_name, "version": world_version(&world).ok().flatten()})).map_err(|_| "No se pudo preparar el catálogo del respaldo.")?;
    atomic_replace_file(&dir.join("world.json"), &metadata)?;
    Ok(info)
}
pub fn list_backups(root: &Path, folder: &str) -> Result<Vec<WorldBackup>, String> {
    let dir = backup_dir(root, folder)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    for entry in
        fs::read_dir(&dir).map_err(|e| io_error("No se pudo leer la carpeta de respaldos", e))?
    {
        let entry = entry.map_err(|e| io_error("No se pudo leer un respaldo", e))?;
        let id = entry.file_name().to_string_lossy().to_string();
        if backup_timestamp(&id).is_some() {
            result.push(describe_backup(&entry.path(), &id)?);
        }
    }
    result.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)));
    Ok(result)
}
pub fn restore_world(root: &Path, server: bool, folder: &str, id: &str) -> Result<(), String> {
    backup_timestamp(id).ok_or("Identificador de respaldo inválido.")?;
    validate_world_id(folder, server)?;
    let parent = worlds_root(root, server)?;
    let world = parent.join(folder);
    let existed = world
        .try_exists()
        .map_err(|e| io_error("No se pudo consultar el mundo", e))?;
    if existed {
        world_path(root, server, folder)?;
    } else {
        let mut ancestor = parent;
        no_link(&ancestor)?;
        let components: Vec<_> = folder.split('/').collect();
        for component in &components[..components.len() - 1] {
            ancestor.push(component);
            no_link(&ancestor)?;
        }
    }
    let dir = backup_dir(root, folder)?;
    let archive = dir.join(id);
    no_link(&archive)?;
    let checksum_path = dir.join(format!("{id}.sha256"));
    no_link(&checksum_path)?;
    let expected = fs::read_to_string(checksum_path)
        .map_err(|e| io_error("Falta el hash de integridad del respaldo", e))?;
    if expected.trim() != hash_file(&archive)? {
        return Err("El respaldo cambió o está dañado; no se modificó el mundo.".into());
    }
    verify_archive(&archive, MAX_ARCHIVE)?;
    if existed {
        backup_world(root, server, folder)?;
    }
    let _session = if existed {
        session_guard(&world)?
    } else {
        None
    };
    restore_directory(&archive, &world)
}
pub fn delete_world(root: &Path, server: bool, folder: &str) -> Result<(), String> {
    let world = world_path(root, server, folder)?;
    backup_world(root, server, folder)?;
    let _session = session_guard(&world)?;
    tree_files(&world, MAX_ARCHIVE)?;
    fs::remove_dir_all(world)
        .map_err(|e| io_error("No se pudo eliminar el mundo; se conserva su respaldo", e))
}

fn datapack_root(world: &Path, enabled: bool) -> Result<PathBuf, String> {
    let dir = world.join(if enabled {
        "datapacks"
    } else {
        ".tfl-disabled-datapacks"
    });
    if dir.exists() {
        no_link(&dir)?;
    }
    Ok(dir)
}
fn pack_meta(path: &Path) -> Result<serde_json::Value, String> {
    let raw = if path.is_dir() {
        tree_files(path, MAX_PACK)?;
        let file = path.join("pack.mcmeta");
        no_link(&file)?;
        let file = File::open(file).map_err(|e| io_error("No se pudo abrir pack.mcmeta", e))?;
        if file
            .metadata()
            .map_err(|e| io_error("No se pudo consultar pack.mcmeta", e))?
            .len()
            > 1024 * 1024
        {
            return Err("pack.mcmeta supera 1 MiB.".into());
        }
        let mut bytes = Vec::new();
        file.take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| io_error("No se pudo leer pack.mcmeta", e))?;
        bytes
    } else {
        let mut zip = zip::ZipArchive::new(
            File::open(path).map_err(|e| io_error("No se pudo abrir el datapack", e))?,
        )
        .map_err(|_| "El datapack no es un ZIP válido.")?;
        inspect_zip(&mut zip, MAX_PACK)?;
        let mut meta = zip
            .by_name("pack.mcmeta")
            .map_err(|_| "El ZIP debe contener pack.mcmeta en su raíz.")?;
        let mut bytes = Vec::new();
        (&mut meta)
            .take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "pack.mcmeta está dañado.")?;
        if bytes.len() > 1024 * 1024 {
            return Err("pack.mcmeta supera 1 MiB.".into());
        }
        bytes
    };
    let value: serde_json::Value =
        serde_json::from_slice(&raw).map_err(|_| "pack.mcmeta no contiene JSON válido.")?;
    let pack = value
        .get("pack")
        .filter(|v| v.is_object())
        .ok_or("pack.mcmeta no declara un objeto pack.")?;
    let format_tuple = |value: &serde_json::Value| -> Option<(u64, u64)> {
        if let Some(major) = value.as_u64() {
            return Some((major, 0));
        }
        let pair = value.as_array()?;
        (pair.len() == 2)
            .then(|| pair[0].as_u64().zip(pair[1].as_u64()))
            .flatten()
    };
    if pack.get("min_format").is_some() || pack.get("max_format").is_some() {
        let range = pack
            .get("min_format")
            .and_then(format_tuple)
            .zip(pack.get("max_format").and_then(format_tuple));
        if !range.is_some_and(|(min, max)| min <= max) {
            return Err("pack.mcmeta declara un rango min_format/max_format inválido.".into());
        }
    }
    if pack
        .get("pack_format")
        .and_then(|value| value.as_u64())
        .is_none()
        && (pack.get("min_format").is_none() || pack.get("max_format").is_none())
    {
        return Err("pack.mcmeta no declara un formato de datapack válido.".into());
    }
    Ok(value)
}

/// Exact versions covered by Mojang release notes. Unknown versions stay
/// unknown rather than inheriting the pack format of a nearby release.
fn expected_pack_format(version: &str) -> Option<u64> {
    match version {
        "1.21" | "1.21.1" => Some(48),
        "1.21.4" => Some(61),
        "1.21.5" => Some(71),
        _ => None,
    }
}
fn pack_compatibility(meta: &serde_json::Value, version: Option<&str>) -> &'static str {
    let Some(expected) = version.and_then(expected_pack_format) else {
        return "unknown";
    };
    let pack = &meta["pack"];
    let supported = if let Some(range) = pack.get("supported_formats") {
        if let Some(format) = range.as_u64() {
            Some(expected == format)
        } else if let Some(range) = range.as_array() {
            range
                .first()
                .and_then(|v| v.as_u64())
                .zip(range.get(1).and_then(|v| v.as_u64()))
                .map(|(min, max)| expected >= min && expected <= max)
        } else {
            range
                .get("min_inclusive")
                .and_then(|v| v.as_u64())
                .zip(range.get("max_inclusive").and_then(|v| v.as_u64()))
                .map(|(min, max)| expected >= min && expected <= max)
        }
    } else {
        pack.get("pack_format")
            .and_then(|v| v.as_u64())
            .map(|format| format == expected)
    };
    match supported {
        Some(true) => "compatible",
        Some(false) => "incompatible",
        None => "unknown",
    }
}
fn datapack_lists(root: &Value) -> Option<&std::collections::HashMap<String, Value>> {
    let data = compound(root)?.get("Data").and_then(compound)?;
    data.get("DataPacks").and_then(compound)
}

fn disabled_datapack(root: &Value, id: &str) -> bool {
    let name = Value::String(format!("file/{id}"));
    matches!(
        datapack_lists(root).and_then(|data| data.get("Disabled")),
        Some(Value::List(disabled)) if disabled.contains(&name)
    )
}

fn update_datapack_status(root: &mut Value, id: &str, enable: bool) -> Result<(), String> {
    let Value::Compound(root) = root else {
        return Err("NBT raíz inválido.".into());
    };
    let Some(Value::Compound(data)) = root.get_mut("Data") else {
        return Err("El mundo no contiene metadatos Data válidos.".into());
    };
    let packs = data.entry("DataPacks".into()).or_insert_with(|| {
        Value::Compound(std::collections::HashMap::from([
            (
                "Enabled".into(),
                Value::List(vec![Value::String("vanilla".into())]),
            ),
            ("Disabled".into(), Value::List(Vec::new())),
        ]))
    });
    let Value::Compound(packs) = packs else {
        return Err("El mundo no contiene una lista válida de datapacks.".into());
    };
    let name = Value::String(format!("file/{id}"));
    for (field, include) in [("Enabled", enable), ("Disabled", !enable)] {
        let Value::List(entries) = packs
            .entry(field.into())
            .or_insert_with(|| Value::List(Vec::new()))
        else {
            return Err("El mundo no contiene una lista válida de datapacks.".into());
        };
        if include {
            if !entries.contains(&name) {
                entries.push(name.clone());
            }
        } else {
            entries.retain(|entry| entry != &name);
        }
    }
    Ok(())
}

pub fn list_datapacks(root: &Path, server: bool, folder: &str) -> Result<Vec<Datapack>, String> {
    validate_world_id(folder, server)?;
    if !worlds_root(root, server)?.join(folder).exists() {
        return Ok(Vec::new());
    }
    let world = world_path(root, server, folder)?;
    let level = read_level(&world).ok().map(|(value, _, _)| value);
    let version = level.as_ref().and_then(version_from_level);
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for in_datapacks in [true, false] {
        let dir = datapack_root(&world, in_datapacks)?;
        if !dir.exists() {
            continue;
        }
        for entry in
            fs::read_dir(dir).map_err(|e| io_error("No se pudieron listar los datapacks", e))?
        {
            let entry = entry.map_err(|e| io_error("No se pudo leer un datapack", e))?;
            let id = entry.file_name().to_string_lossy().to_string();
            if id.starts_with('.') {
                continue;
            }
            if !seen.insert(id.to_lowercase()) {
                return Err("Hay dos archivos con el mismo nombre de datapack.".into());
            }
            let path = entry.path();
            no_link(&path)?;
            let enabled = in_datapacks
                && !level
                    .as_ref()
                    .is_some_and(|value| disabled_datapack(value, &id));
            let mut item = Datapack {
                id,
                enabled,
                pack_format: None,
                min_format: None,
                max_format: None,
                world_version: version.clone(),
                compatibility: "unknown".into(),
                error: None,
            };
            match pack_meta(&path) {
                Ok(meta) => {
                    let pack = &meta["pack"];
                    item.pack_format = pack["pack_format"].as_u64();
                    item.min_format = pack.get("min_format").cloned();
                    item.max_format = pack.get("max_format").cloned();
                    item.compatibility = pack_compatibility(&meta, version.as_deref()).into();
                }
                Err(error) => item.error = Some(error),
            }
            result.push(item);
        }
    }
    result.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(result)
}
pub fn install_datapack(
    root: &Path,
    server: bool,
    folder: &str,
    source: &Path,
) -> Result<(), String> {
    let world = world_path(root, server, folder)?;
    let _session = session_guard(&world)?;
    no_link(source)?;
    if !source.is_file()
        || !source
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("zip"))
    {
        return Err("Selecciona un datapack ZIP.".into());
    }
    let id = source
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Nombre de datapack inválido.")?;
    valid_name(id)?;
    pack_meta(source)?;
    verify_archive(source, MAX_PACK)?;
    let dir = datapack_root(&world, true)?;
    fs::create_dir_all(&dir)
        .map_err(|e| io_error("No se pudo crear la carpeta de datapacks", e))?;
    if dir.join(id).exists() || datapack_root(&world, false)?.join(id).exists() {
        return Err("Ya existe un datapack con ese nombre. Elimínalo de forma explícita antes de reemplazarlo.".into());
    }
    let tmp = dir.join(format!(".tfl-install-{}", uuid::Uuid::new_v4()));
    fs::copy(source, &tmp).map_err(|e| io_error("No se pudo copiar el datapack", e))?;
    let result = verify_archive(&tmp, MAX_PACK).and_then(|_| {
        fs::rename(&tmp, dir.join(id)).map_err(|e| io_error("No se pudo instalar el datapack", e))
    });
    if result.is_err() {
        let _ = fs::remove_file(tmp);
    }
    result
}
pub fn change_datapack(
    root: &Path,
    server: bool,
    folder: &str,
    id: &str,
    enabled: bool,
    remove: bool,
) -> Result<(), String> {
    valid_name(id)?;
    let world = world_path(root, server, folder)?;
    let _session = session_guard(&world)?;
    let active = datapack_root(&world, true)?.join(id);
    let parked = datapack_root(&world, false)?.join(id);
    let active_exists = active
        .try_exists()
        .map_err(|e| io_error("No se pudo consultar el datapack", e))?;
    let parked_exists = parked
        .try_exists()
        .map_err(|e| io_error("No se pudo consultar el datapack deshabilitado", e))?;
    if active_exists && parked_exists {
        return Err("Hay dos archivos con el mismo nombre de datapack.".into());
    }
    let source = if active_exists { &active } else { &parked };
    no_link(source)?;
    // Refuse an unbounded traversal before recursive removal of a directory pack.
    if source.is_dir() {
        tree_files(source, MAX_PACK)?;
    }
    if remove {
        let (mut level, prefix, gzip) = read_level(&world)?;
        if let Value::Compound(root) = &mut level
            && let Some(Value::Compound(data)) = root.get_mut("Data")
            && let Some(Value::Compound(packs)) = data.get_mut("DataPacks")
        {
            let name = Value::String(format!("file/{id}"));
            for field in ["Enabled", "Disabled"] {
                match packs.get_mut(field) {
                    Some(Value::List(entries)) => entries.retain(|entry| entry != &name),
                    None => {}
                    _ => return Err("El mundo no contiene una lista válida de datapacks.".into()),
                }
            }
        }
        let recovery = world.join(format!(".tfl-removed-datapack-{}", uuid::Uuid::new_v4()));
        fs::rename(source, &recovery)
            .map_err(|e| io_error("No se pudo preparar la eliminación del datapack", e))?;
        if let Err(error) = write_level(&world, &level, prefix, gzip) {
            fs::rename(&recovery, source).map_err(|e| {
                io_error("No se pudo recuperar el datapack de su copia temporal", e)
            })?;
            return Err(error);
        }
        let removed = if recovery.is_dir() {
            fs::remove_dir_all(&recovery)
        } else {
            fs::remove_file(&recovery)
        };
        removed.map_err(|e| {
            format!(
                "El datapack se retiró del mundo. Su copia temporal se conservó en {}: {e}",
                recovery.display()
            )
        })
    } else {
        let (mut level, prefix, gzip) = read_level(&world)?;
        let current = active_exists && !disabled_datapack(&level, id);
        if current != enabled {
            return Err("El estado del datapack cambió. Actualiza la lista del mundo.".into());
        }
        update_datapack_status(&mut level, id, !current)?;
        if parked_exists {
            let folder = active.parent().ok_or("Ruta de datapacks inválida.")?;
            fs::create_dir_all(folder)
                .map_err(|e| io_error("No se pudo preparar la carpeta de datapacks", e))?;
            fs::rename(&parked, &active)
                .map_err(|e| io_error("No se pudo recuperar el datapack deshabilitado", e))?;
        }
        let saved = write_level(&world, &level, prefix, gzip);
        if saved.is_err() && parked_exists {
            fs::rename(&active, &parked)
                .map_err(|e| io_error("No se pudo revertir el movimiento del datapack", e))?;
        }
        saved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("tfl-worlds-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
        fn world(&self, relative: &str) -> PathBuf {
            let world = self.0.join(relative);
            fs::create_dir_all(world.join("region")).unwrap();
            write_fixture_level(&world, "Un mundo — 世界", "1.21.4");
            fs::write(world.join("region/r.0.0.mca"), b"fixture chunk data").unwrap();
            world
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn write_fixture_level(world: &Path, name: &str, version: &str) {
        let version = Value::Compound(HashMap::from([
            ("Name".into(), Value::String(version.into())),
            ("Id".into(), Value::Int(4189)),
        ]));
        let data = Value::Compound(HashMap::from([
            ("LevelName".into(), Value::String(name.into())),
            ("Version".into(), version),
            ("DataVersion".into(), Value::Int(4189)),
            ("RandomSeed".into(), Value::Long(9223372036854775000)),
            ("SpawnX".into(), Value::Int(-12345)),
            (
                "UnknownModMetadata".into(),
                Value::ByteArray(fastnbt::ByteArray::new(vec![-1, 0, 127])),
            ),
        ]));
        let root = Value::Compound(HashMap::from([("Data".into(), data)]));
        let raw =
            fastnbt::to_bytes_with_opts(&root, fastnbt::SerOpts::new().root_name("fixture-root"))
                .unwrap();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&raw).unwrap();
        fs::write(world.join("level.dat"), encoder.finish().unwrap()).unwrap();
    }
    fn zip_entries(path: &Path, entries: &[(&str, &[u8])]) {
        let mut writer = zip::ZipWriter::new(File::create(path).unwrap());
        for (name, bytes) in entries {
            writer
                .start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap();
    }
    fn pack(path: &Path, format: u64) {
        let meta = serde_json::to_vec(
            &serde_json::json!({"pack":{"pack_format":format,"description":"test datapack"}}),
        )
        .unwrap();
        zip_entries(
            path,
            &[
                ("pack.mcmeta", &meta),
                ("data/test/function/test.mcfunction", b"say test"),
            ],
        );
    }

    #[test]
    fn spaces_unicode_and_windows_game_layout_are_detected_without_global_search() {
        let fixture = Fixture::new();
        fixture.world("saves/new world");
        fixture.world("saves/世界 con espacios");
        let worlds = list_worlds(&fixture.0, false).unwrap();
        assert_eq!(worlds.len(), 2);
        assert!(worlds.iter().any(|world| world.name == "new world"));
        assert!(worlds.iter().any(|world| world.name == "世界 con espacios"));
        assert!(worlds.iter().all(
            |world| world.version.as_deref() == Some("1.21.4") && world.size_bytes.unwrap() > 0
        ));
    }

    #[test]
    fn vanilla_save_is_discovered_renamed_and_restored_with_its_original_region() {
        // A vanilla 26.1.2 save from Apoptosis. See the vendored provenance.
        let level = include_bytes!("../../tests/fixtures/vanilla-26.1.2/level_26_1_2.dat");
        let region = include_bytes!("../../tests/fixtures/vanilla-26.1.2/region_26_1_2.mca");
        let fixture = Fixture::new();
        let id = "new world – montaña";
        let world = fixture.0.join(".minecraft/saves").join(id);
        let region_relative = "dimensions/minecraft/overworld/region/r.-2.-1.mca";
        let region_path = world.join(region_relative);
        fs::create_dir_all(region_path.parent().unwrap()).unwrap();
        fs::write(world.join("level.dat"), level).unwrap();
        fs::write(&region_path, region).unwrap();

        let detected = list_worlds(&fixture.0, false).unwrap();
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].name, id);
        assert_eq!(detected[0].data_version, Some(4790));
        assert_eq!(detected[0].version.as_deref(), Some("26.1.2"));
        assert_eq!(detected[0].metadata_error, None);

        let backup = backup_world(&fixture.0, false, id).unwrap();
        rename_world(&fixture.0, false, id, "Montaña «26.1.2»").unwrap();
        assert_eq!(
            list_worlds(&fixture.0, false).unwrap()[0].display_name,
            "Montaña «26.1.2»"
        );
        assert_eq!(fs::read(&region_path).unwrap(), region.as_slice());
        restore_world(&fixture.0, false, id, &backup.id).unwrap();
        assert_eq!(fs::read(world.join("level.dat")).unwrap(), level.as_slice());
        assert_eq!(fs::read(region_path).unwrap(), region.as_slice());
    }

    #[cfg(windows)]
    #[test]
    fn unreadable_windows_saves_reports_permission_error() {
        use std::process::Command;

        let fixture = Fixture::new();
        fixture.world("saves/new world");
        let saves = fixture.0.join("saves");
        let identity = Command::new("whoami.exe")
            .args(["/user", "/fo", "csv", "/nh"])
            .output()
            .unwrap();
        assert!(identity.status.success());
        let output = String::from_utf8_lossy(&identity.stdout);
        let sid = output
            .split('"')
            .find(|part| part.starts_with("S-1-"))
            .unwrap();
        let principal = format!("*{sid}");
        let deny_read = format!("{principal}:(RD,RA,REA)");
        let deny = Command::new("icacls.exe")
            .arg(&saves)
            .args(["/deny", &deny_read])
            .output()
            .unwrap();
        assert!(
            deny.status.success(),
            "icacls failed: {}",
            String::from_utf8_lossy(&deny.stderr)
        );
        let os_denied = fs::read_dir(&saves)
            .err()
            .is_some_and(|error| error.kind() == io::ErrorKind::PermissionDenied);
        let unreadable = list_worlds(&fixture.0, false);
        let restore = Command::new("icacls.exe")
            .arg(&saves)
            .args(["/remove:d", &principal])
            .output()
            .unwrap();
        assert!(
            restore.status.success(),
            "could not undo temporary test ACL"
        );
        if os_denied {
            assert!(unreadable.unwrap_err().contains("No se pudo"));
        } else {
            // The hosted token may retain directory enumeration despite its ACL.
            // Exercise a real read failure under that token as well.
            eprintln!("The Windows runner token can enumerate the directory despite the deny ACL.");
            fs::rename(&saves, fixture.0.join("saved-worlds")).unwrap();
            fs::write(&saves, b"not a directory").unwrap();
            let result = list_worlds(&fixture.0, false);
            fs::remove_file(&saves).unwrap();
            fs::rename(fixture.0.join("saved-worlds"), &saves).unwrap();
            assert!(result.unwrap_err().contains("No se pudo"));
        }
        assert_eq!(list_worlds(&fixture.0, false).unwrap().len(), 1);
    }

    #[test]
    fn game_directory_is_bounded_consistent_and_ambiguity_is_not_empty_state() {
        let fixture = Fixture::new();
        fixture.world(".minecraft/saves/new world");
        assert_eq!(
            resolve_game_dir(&fixture.0, false).unwrap(),
            fixture.0.join(".minecraft")
        );
        assert_eq!(list_worlds(&fixture.0, false).unwrap().len(), 1);
        fixture.world("minecraft/saves/other");
        assert!(
            resolve_game_dir(&fixture.0, false)
                .unwrap_err()
                .contains("varios")
        );
        fs::create_dir(fixture.0.join("saves")).unwrap();
        assert_eq!(resolve_game_dir(&fixture.0, false).unwrap(), fixture.0);
    }

    #[test]
    fn nbt_rename_preserves_identity_root_name_mod_fields_and_long_precision() {
        let fixture = Fixture::new();
        let world = fixture.world("saves/new world");
        let (mut expected, prefix, _) = read_level(&world).unwrap();
        if let Value::Compound(root) = &mut expected {
            if let Some(Value::Compound(data)) = root.get_mut("Data") {
                data.insert(
                    "LevelName".into(),
                    Value::String("Mi nombre nuevo — 日本語".into()),
                );
            }
        }
        rename_world(&fixture.0, false, "new world", "Mi nombre nuevo — 日本語").unwrap();
        let (actual, actual_prefix, compressed) = read_level(&world).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual_prefix, prefix);
        assert!(compressed);
        assert!(world.exists());
        assert!(!fixture.0.join("saves/Mi nombre nuevo — 日本語").exists());
    }

    #[test]
    fn corrupt_level_is_visible_preserved_and_not_renamed_as_plain_text() {
        let fixture = Fixture::new();
        let world = fixture.world("saves/new world");
        fs::write(world.join("level.dat"), b"invalid NBT").unwrap();
        let worlds = list_worlds(&fixture.0, false).unwrap();
        assert_eq!(worlds.len(), 1);
        assert!(worlds[0].metadata_error.is_some());
        assert!(rename_world(&fixture.0, false, "new world", "new name").is_err());
        assert_eq!(fs::read(world.join("level.dat")).unwrap(), b"invalid NBT");
        assert!(backup_world(&fixture.0, false, "new world").is_ok());
    }

    #[test]
    fn server_custom_level_name_and_dimensions_are_not_client_saves() {
        let fixture = Fixture::new();
        let world = fixture.world("universos/mi mundo");
        fs::create_dir(world.join("DIM-1")).unwrap();
        fs::write(
            fixture.0.join("server.properties"),
            "level-name=universos/mi mundo\nserver-port=25565\n",
        )
        .unwrap();
        let worlds = list_worlds(&fixture.0, true).unwrap();
        assert_eq!(worlds.len(), 1);
        assert_eq!(worlds[0].name, "universos/mi mundo");
        assert!(worlds[0].server_primary);
        assert_eq!(worlds[0].dimensions, vec!["DIM-1"]);
        assert!(list_worlds(&fixture.0, false).unwrap().is_empty());
    }

    #[test]
    fn backup_restore_is_verified_and_preserves_selected_and_safety_archives() {
        let fixture = Fixture::new();
        let world = fixture.world("saves/new world");
        let original = fs::read(world.join("level.dat")).unwrap();
        let selected = backup_world(&fixture.0, false, "new world").unwrap();
        rename_world(&fixture.0, false, "new world", "changed").unwrap();
        restore_world(&fixture.0, false, "new world", &selected.id).unwrap();
        assert_eq!(fs::read(world.join("level.dat")).unwrap(), original);
        let backups = list_backups(&fixture.0, "new world").unwrap();
        assert_eq!(backups.len(), 2);
        assert!(
            backups
                .iter()
                .any(|backup| backup.id == selected.id && backup.sha256 == selected.sha256)
        );
    }

    #[test]
    fn tampered_backup_never_replaces_current_world() {
        let fixture = Fixture::new();
        let world = fixture.world("saves/new world");
        let selected = backup_world(&fixture.0, false, "new world").unwrap();
        let original = fs::read(world.join("level.dat")).unwrap();
        fs::write(
            backup_dir(&fixture.0, "new world")
                .unwrap()
                .join(selected.id.clone()),
            b"tampered",
        )
        .unwrap();
        assert!(restore_world(&fixture.0, false, "new world", &selected.id).is_err());
        assert_eq!(fs::read(world.join("level.dat")).unwrap(), original);
    }

    #[test]
    fn hostile_zip_paths_and_case_collisions_are_rejected_transactionally() {
        let fixture = Fixture::new();
        let destination = fixture.world("saves/new world");
        let original = fs::read(destination.join("level.dat")).unwrap();
        for name in [
            "../escaped",
            "/absolute",
            "C:/absolute",
            "World\\escape",
            "a/../escape",
            "a/./x",
            "aux.txt",
        ] {
            let archive = fixture.0.join("bad.zip");
            zip_entries(&archive, &[(name, b"bad")]);
            assert!(restore_directory(&archive, &destination).is_err(), "{name}");
            assert_eq!(fs::read(destination.join("level.dat")).unwrap(), original);
            assert!(!fixture.0.join("escaped").exists());
        }
        let archive = fixture.0.join("case.zip");
        zip_entries(
            &archive,
            &[("same/file", b"first"), ("SAME/FILE", b"second")],
        );
        assert!(restore_directory(&archive, &destination).is_err());
        assert_eq!(fs::read(destination.join("level.dat")).unwrap(), original);
    }

    #[test]
    fn symlink_entries_are_rejected_before_extraction() {
        let fixture = Fixture::new();
        let zip = fixture.0.join("link.zip");
        let mut writer = zip::ZipWriter::new(File::create(&zip).unwrap());
        writer
            .add_symlink(
                "link",
                "../../outside",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        writer.finish().unwrap();
        assert!(verify_archive(&zip, MAX_PACK).is_err());
        assert!(restore_directory(&zip, &fixture.0.join("restored")).is_err());
        assert!(!fixture.0.join("restored").exists());
    }

    #[test]
    fn datapack_local_cycle_checks_format_version_and_preserves_source() {
        let fixture = Fixture::new();
        fixture.world("saves/new world");
        let source = fixture.0.join("pack.zip");
        pack(&source, 61);
        let source_hash = hash_file(&source).unwrap();
        install_datapack(&fixture.0, false, "new world", &source).unwrap();
        let packs = list_datapacks(&fixture.0, false, "new world").unwrap();
        assert_eq!(packs.len(), 1);
        assert!(packs[0].enabled);
        assert_eq!(packs[0].compatibility, "compatible");
        assert!(install_datapack(&fixture.0, false, "new world", &source).is_err());
        change_datapack(&fixture.0, false, "new world", "pack.zip", true, false).unwrap();
        assert!(!list_datapacks(&fixture.0, false, "new world").unwrap()[0].enabled);
        let world = fixture.0.join("saves/new world");
        assert!(disabled_datapack(
            &read_level(&world).unwrap().0,
            "pack.zip"
        ));
        assert!(world.join("datapacks/pack.zip").exists());
        change_datapack(&fixture.0, false, "new world", "pack.zip", false, false).unwrap();
        assert!(list_datapacks(&fixture.0, false, "new world").unwrap()[0].enabled);
        change_datapack(&fixture.0, false, "new world", "pack.zip", true, true).unwrap();
        let (level, _, _) = read_level(&world).unwrap();
        let removed = Value::String("file/pack.zip".into());
        for field in ["Enabled", "Disabled"] {
            if let Some(Value::List(entries)) =
                datapack_lists(&level).and_then(|packs| packs.get(field))
            {
                assert!(!entries.contains(&removed));
            }
        }
        assert!(
            list_datapacks(&fixture.0, false, "new world")
                .unwrap()
                .is_empty()
        );
        assert_eq!(hash_file(&source).unwrap(), source_hash);
        pack(&source, 71);
        install_datapack(&fixture.0, false, "new world", &source).unwrap();
        assert_eq!(
            list_datapacks(&fixture.0, false, "new world").unwrap()[0].compatibility,
            "incompatible"
        );
    }

    #[test]
    fn game_disabled_pack_and_legacy_parked_pack_can_both_be_enabled() {
        let fixture = Fixture::new();
        let world = fixture.world("saves/new world");
        let active = datapack_root(&world, true).unwrap();
        let parked = datapack_root(&world, false).unwrap();
        fs::create_dir(&active).unwrap();
        fs::create_dir(&parked).unwrap();
        pack(&active.join("from-game.zip"), 61);
        pack(&parked.join("from-launcher.zip"), 61);
        let (mut level, prefix, gzip) = read_level(&world).unwrap();
        update_datapack_status(&mut level, "from-game.zip", false).unwrap();
        write_level(&world, &level, prefix, gzip).unwrap();
        let packs = list_datapacks(&fixture.0, false, "new world").unwrap();
        assert_eq!(packs.len(), 2);
        assert!(packs.iter().all(|pack| !pack.enabled));
        for id in ["from-game.zip", "from-launcher.zip"] {
            change_datapack(&fixture.0, false, "new world", id, false, false).unwrap();
        }
        assert!(active.join("from-launcher.zip").exists());
        assert!(!parked.join("from-launcher.zip").exists());
        assert!(
            list_datapacks(&fixture.0, false, "new world")
                .unwrap()
                .iter()
                .all(|pack| pack.enabled)
        );
    }

    #[test]
    fn malicious_or_incomplete_datapacks_are_never_partially_installed() {
        let fixture = Fixture::new();
        fixture.world("saves/new world");
        let source = fixture.0.join("evil.zip");
        let meta = br#"{"pack":{"pack_format":61,"description":"test"}}"#;
        zip_entries(
            &source,
            &[("pack.mcmeta", meta), ("../outside.txt", b"evil")],
        );
        assert!(install_datapack(&fixture.0, false, "new world", &source).is_err());
        zip_entries(&source, &[("nested/pack.mcmeta", meta)]);
        assert!(install_datapack(&fixture.0, false, "new world", &source).is_err());
        assert!(
            list_datapacks(&fixture.0, false, "new world")
                .unwrap()
                .is_empty()
        );
        assert!(!fixture.0.join("outside.txt").exists());
    }

    #[test]
    fn active_session_lock_blocks_rename_delete_backup_and_datapacks() {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};

        let fixture = Fixture::new();
        let world = fixture.world("saves/new world");
        fs::write(world.join("session.lock"), b"lock").unwrap();
        let mut holder = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "services::worlds::tests::hold_fixture_world_lock",
                "--ignored",
                "--nocapture",
            ])
            .env("TFL_TEST_WORLD_LOCK", &world)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let reader = BufReader::new(holder.stdout.as_mut().unwrap());
        assert!(
            reader
                .lines()
                .map_while(Result::ok)
                .any(|line| line.contains("TFL_FIXTURE_WORLD_LOCKED"))
        );
        let source = fixture.0.join("pack.zip");
        pack(&source, 61);
        assert!(rename_world(&fixture.0, false, "new world", "changed").is_err());
        assert!(delete_world(&fixture.0, false, "new world").is_err());
        assert!(backup_world(&fixture.0, false, "new world").is_err());
        assert!(install_datapack(&fixture.0, false, "new world", &source).is_err());
        drop(holder.stdin.take());
        assert!(holder.wait().unwrap().success());
        backup_world(&fixture.0, false, "new world").unwrap();
    }

    #[test]
    #[ignore = "child lock holder launched by active_session_lock_blocks_rename_delete_backup_and_datapacks"]
    fn hold_fixture_world_lock() {
        let world = PathBuf::from(std::env::var_os("TFL_TEST_WORLD_LOCK").expect("fixture path"));
        let _session = session_guard(&world).unwrap().expect("fixture lock");
        println!("TFL_FIXTURE_WORLD_LOCKED");
        let mut keep_alive = String::new();
        std::io::stdin().read_line(&mut keep_alive).unwrap();
    }

    #[test]
    fn delete_keeps_a_verified_recoverable_backup_and_refuses_escape() {
        let fixture = Fixture::new();
        fixture.world("saves/new world");
        assert!(delete_world(&fixture.0, false, "../outside").is_err());
        delete_world(&fixture.0, false, "new world").unwrap();
        assert!(!fixture.0.join("saves/new world").exists());
        let backups = list_backups(&fixture.0, "new world").unwrap();
        assert_eq!(backups.len(), 1);
        let archive = backup_dir(&fixture.0, "new world")
            .unwrap()
            .join(&backups[0].id);
        verify_archive(&archive, MAX_ARCHIVE).unwrap();
        let worlds = list_worlds(&fixture.0, false).unwrap();
        assert_eq!(worlds.len(), 1);
        assert!(worlds[0].missing);
        restore_world(&fixture.0, false, "new world", &backups[0].id).unwrap();
        assert!(fixture.0.join("saves/new world/level.dat").is_file());
        assert!(!list_worlds(&fixture.0, false).unwrap()[0].missing);
    }

    #[test]
    fn nonexistent_saves_is_empty_but_unreadable_structure_is_an_error() {
        let fixture = Fixture::new();
        assert!(list_worlds(&fixture.0, false).unwrap().is_empty());
        fs::write(fixture.0.join("saves"), b"not a directory").unwrap();
        assert!(list_worlds(&fixture.0, false).is_err());
    }

    #[test]
    fn bounded_known_pack_formats_do_not_invent_support_for_other_versions() {
        assert_eq!(
            pack_compatibility(
                &serde_json::json!({"pack":{"pack_format":61}}),
                Some("1.21.4")
            ),
            "compatible"
        );
        assert_eq!(
            pack_compatibility(
                &serde_json::json!({"pack":{"pack_format":61}}),
                Some("1.21.5")
            ),
            "incompatible"
        );
        assert_eq!(
            pack_compatibility(
                &serde_json::json!({"pack":{"pack_format":61,"supported_formats":[61,71]}}),
                Some("1.21.5")
            ),
            "compatible"
        );
        assert_eq!(
            pack_compatibility(
                &serde_json::json!({"pack":{"pack_format":61}}),
                Some("unverified-version")
            ),
            "unknown"
        );
    }

    #[test]
    fn malformed_nbt_depth_and_sequence_lengths_are_rejected_before_allocating() {
        let mut nested = vec![10, 0, 0];
        for _ in 0..110 {
            nested.extend_from_slice(&[10, 0, 1, b'x']);
        }
        nested.extend(std::iter::repeat_n(0, 111));
        assert!(validate_nbt_shape(&nested, 3).is_err());
        let huge_list = [10, 0, 0, 9, 0, 1, b'x', 1, 0x7f, 0xff, 0xff, 0xff, 0];
        assert!(validate_nbt_shape(&huge_list, 3).is_err());
    }
    #[test]
    fn backup_copy_refuses_file_growth_after_the_size_budget_was_reserved() {
        let fixture = Fixture::new();
        let file = fixture.0.join("growing-region.mca");
        fs::write(&file, b"old").unwrap();
        let planned = fs::metadata(&file).unwrap().len();
        let mut output = Vec::new();
        copy_planned_file(&file, &mut output, planned).unwrap();
        assert_eq!(output, b"old");
        let mut file_handle = OpenOptions::new().append(true).open(&file).unwrap();
        file_handle.write_all(b"new chunk").unwrap();
        drop(file_handle);
        let mut output = Vec::new();
        assert!(
            copy_planned_file(&file, &mut output, planned)
                .unwrap_err()
                .contains("cambió de tamaño")
        );
        assert!(output.is_empty());
        assert_eq!(fs::read(&file).unwrap(), b"oldnew chunk");
    }
}
