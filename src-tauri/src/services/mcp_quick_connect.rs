//! Bounded, opt-in edits to local MCP configuration. No credentials are returned,
//! subprocesses started, permissions granted, or historical backups restored on disconnect.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

const MAX_CONFIG: u64 = 1024 * 1024;
static EDIT: Mutex<()> = Mutex::new(());
const ENTRY: &str = "tflclient";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub id: String,
    pub client: String,
    pub label: String,
    pub path: String,
    pub installed: bool,
    pub supported: bool,
    pub configured: bool,
    pub managed: bool,
    pub revision: String,
    pub reason: Option<String>,
    #[serde(skip)]
    adapter: Adapter,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
enum Adapter {
    Codex,
    OpenCode,
    Claude,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Owned {
    adapter: Adapter,
    digest: String,
}
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    schema: u32,
    entries: BTreeMap<String, Owned>,
}

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn fingerprint(raw: &Option<String>) -> String {
    raw.as_ref()
        .map_or_else(|| "missing".into(), |s| digest(s.as_bytes()))
}
fn semantic_digest(value: &Value) -> String {
    digest(serde_json::to_string(value).expect("JSON value").as_bytes())
}
fn no_links(path: &Path) -> Result<(), String> {
    // A config file or parent symlink/junction can redirect an otherwise fixed destination.
    for part in path.ancestors() {
        match fs::symlink_metadata(part) {
            Ok(meta) => {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err("El destino contiene un enlace o junction; usa la configuración manual.".into());
                    }
                }
                if meta.file_type().is_symlink() {
                    return Err(
                        "El destino contiene un enlace; usa la configuración manual.".into(),
                    );
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("No se pudo comprobar el destino de configuración.".into()),
        }
    }
    Ok(())
}
fn read_config(path: &Path) -> Result<Option<String>, String> {
    no_links(path)?;
    match fs::metadata(path) {
        Ok(meta) if !meta.is_file() || meta.len() > MAX_CONFIG => {
            Err("La configuración no es un archivo válido o supera 1 MiB.".into())
        }
        Ok(_) => fs::read_to_string(path)
            .map(Some)
            .map_err(|_| "No se pudo leer la configuración como UTF-8.".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("No se pudo leer la configuración. Revisa sus permisos.".into()),
    }
}
fn read_registry(path: &Path) -> Result<Registry, String> {
    let Some(raw) = read_config(path)? else {
        return Ok(Registry {
            schema: 1,
            ..Default::default()
        });
    };
    let registry: Registry = serde_json::from_str(&raw).map_err(|_| {
        "El registro de conexiones está dañado; no se modificó la configuración.".to_string()
    })?;
    if registry.schema != 1 {
        return Err("Versión de registro MCP no compatible.".into());
    }
    Ok(registry)
}
fn executable_in_path(name: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|path| {
        std::env::split_paths(&path).take(100).any(|dir| {
            [
                name.to_string(),
                format!("{name}.exe"),
                format!("{name}.cmd"),
            ]
            .iter()
            .any(|name| dir.join(name).is_file())
        })
    })
}
fn installed(home: &Path, adapter: Adapter) -> bool {
    let name = match adapter {
        Adapter::Codex => "codex",
        Adapter::OpenCode => "opencode",
        Adapter::Claude => "claude",
    };
    if executable_in_path(name) {
        return true;
    }
    let candidates: &[&str] = match adapter {
        Adapter::Codex => &[
            ".local/bin/codex",
            "AppData/Local/OpenAI/Codex",
            "AppData/Local/Programs/OpenAI/Codex",
        ],
        Adapter::OpenCode => &[
            ".opencode/bin/opencode",
            ".local/bin/opencode",
            "AppData/Local/Programs/ai.opencode.desktop/resources/opencode-cli.exe",
        ],
        Adapter::Claude => &[
            ".local/bin/claude",
            ".local/bin/claude.exe",
            ".claude/local/claude",
            "AppData/Roaming/npm/claude.cmd",
        ],
    };
    candidates.iter().any(|p| home.join(p).exists())
        || match adapter {
            Adapter::Codex => {
                Path::new("/Applications/Codex.app").is_dir()
                    || Path::new("/Applications/ChatGPT.app").is_dir()
            }
            Adapter::OpenCode => Path::new("/Applications/OpenCode.app").is_dir(),
            Adapter::Claude => false,
        }
}
fn entry_for(adapter: Adapter, command: &str) -> Value {
    match adapter {
        Adapter::Codex => json!({"command":command,"startup_timeout_sec":15,"tool_timeout_sec":45}),
        Adapter::OpenCode => json!({"type":"local","command":[command],"enabled":true}),
        Adapter::Claude => json!({"type":"stdio","command":command,"args":[]}),
    }
}

/// Only these destinations may be written. Callers select an opaque ID, never a path.
pub fn targets(home: &Path, registry_path: &Path) -> Result<Vec<Target>, String> {
    let registry = read_registry(registry_path)?;
    let mut destinations = vec![
        (
            Adapter::Codex,
            home.join(".codex/config.toml"),
            "Codex · app/CLI · usuario",
        ),
        (
            Adapter::OpenCode,
            home.join(".config/opencode/opencode.json"),
            "OpenCode · app/CLI · JSON",
        ),
        (
            Adapter::OpenCode,
            home.join(".config/opencode/opencode.jsonc"),
            "OpenCode · app/CLI · JSONC",
        ),
        (
            Adapter::Claude,
            home.join(".claude.json"),
            "Claude Code · usuario",
        ),
    ];
    if let Some(root) = std::env::var_os("CODEX_HOME") {
        let root = PathBuf::from(root);
        if root.is_absolute() && root.is_dir() && root != home.join(".codex") {
            destinations.push((
                Adapter::Codex,
                root.join("config.toml"),
                "Codex · perfil CODEX_HOME",
            ));
        }
    }
    let opencode_has_json = home.join(".config/opencode/opencode.json").is_file();
    let opencode_has_jsonc = home.join(".config/opencode/opencode.jsonc").is_file();
    let mut result = Vec::new();
    for (adapter, path, label) in destinations {
        if adapter == Adapter::OpenCode
            && !path.exists()
            && (opencode_has_json
                || opencode_has_jsonc
                || path.extension().is_some_and(|s| s == "jsonc"))
        {
            continue;
        }
        let client = match adapter {
            Adapter::Codex => "codex",
            Adapter::OpenCode => "opencode",
            Adapter::Claude => "claude",
        };
        let id = format!(
            "{client}-{}",
            &digest(path.to_string_lossy().as_bytes())[..24]
        );
        let mut target = Target {
            id,
            client: client.into(),
            label: label.into(),
            path: path.to_string_lossy().into(),
            installed: installed(home, adapter),
            supported: true,
            configured: false,
            managed: false,
            revision: "missing".into(),
            reason: None,
            adapter,
        };
        if adapter == Adapter::Claude && std::env::var_os("CLAUDE_CONFIG_DIR").is_some() {
            target.supported = false;
            target.reason = Some("Claude Code usa un perfil personalizado. Selecciona su proyecto y configura MCP manualmente en .mcp.json.".into());
        }
        match read_config(&path) {
            Ok(raw) => {
                target.revision = fingerprint(&raw);
                match current_entry(raw.as_deref().unwrap_or(""), adapter) {
                    Ok(entry) => {
                        target.configured = entry.is_some();
                        target.managed = entry.as_ref().is_some_and(|entry| {
                            registry.entries.get(&target.id).is_some_and(|owned| {
                                owned.adapter == adapter && owned.digest == semantic_digest(entry)
                            })
                        });
                        if target.configured && !target.managed {
                            target.reason = Some("Ya existe una entrada tflclient ajena o modificada. Revísala manualmente; TFL no la reemplazará.".into());
                        }
                    }
                    Err(e) => {
                        target.supported = false;
                        target.reason = Some(e);
                    }
                }
            }
            Err(e) => {
                target.supported = false;
                target.reason = Some(e);
            }
        }
        if !target.installed && target.reason.is_none() {
            target.reason = Some("No se detectó el cliente instalado. Instálalo o usa la configuración manual en Avanzado.".into());
        }
        result.push(target);
    }
    Ok(result)
}

fn current_entry(raw: &str, adapter: Adapter) -> Result<Option<Value>, String> {
    if raw.trim().is_empty() {
        return Ok(None);
    }
    match adapter {
        Adapter::Codex => {
            let doc: toml_edit::DocumentMut = raw
                .parse()
                .map_err(|_| "La configuración TOML no es válida; no se modificó.".to_string())?;
            let value: toml::Value =
                toml::from_str(&doc.to_string()).map_err(|_| "TOML no compatible".to_string())?;
            let Some(root) = value.get("mcp_servers") else {
                return Ok(None);
            };
            if !root.is_table() {
                return Err("mcp_servers debe ser una tabla TOML.".into());
            }
            root.get(ENTRY)
                .map(|v| serde_json::to_value(v).map_err(|_| "Entrada TOML no compatible".into()))
                .transpose()
        }
        _ => {
            if adapter == Adapter::Claude {
                serde_json::from_str::<Value>(raw).map_err(|_| {
                    "Claude Code necesita JSON válido, sin comentarios ni comas finales."
                        .to_string()
                })?;
            }
            let node = parse_jsonc(raw)?;
            let value = node.value;
            let (path, alternate) = json_entry_path(&value, adapter)?;
            if let Some(other) = alternate
                && lookup(&value, &other).is_some()
            {
                if lookup(&value, &path).is_some() {
                    return Err("Hay dos entradas tflclient de OpenCode; resuelve el conflicto manualmente.".into());
                }
                return Ok(lookup(&value, &other).cloned());
            }
            Ok(lookup(&value, &path).cloned())
        }
    }
}
fn lookup<'a>(mut value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    for key in path {
        value = value.get(*key)?;
    }
    Some(value)
}
type JsonPath<'a> = (Vec<&'a str>, Option<Vec<&'a str>>);
fn json_entry_path(value: &Value, adapter: Adapter) -> Result<JsonPath<'static>, String> {
    if !value.is_object() {
        return Err("La configuración debe ser un objeto JSON.".into());
    }
    if adapter == Adapter::Claude {
        return Ok((vec!["mcpServers", ENTRY], None));
    }
    if let Some(mcp) = value.get("mcp") {
        if !mcp.is_object() {
            return Err("mcp debe ser un objeto JSON.".into());
        }
        if mcp.get("servers").is_some() {
            if !mcp["servers"].is_object() {
                return Err("mcp.servers debe ser un objeto JSON.".into());
            }
            return Ok((vec!["mcp", "servers", ENTRY], Some(vec!["mcp", ENTRY])));
        }
    }
    // The documented V1 form remains accepted by V2, avoiding a whole-file migration.
    Ok((vec!["mcp", ENTRY], Some(vec!["mcp", "servers", ENTRY])))
}

fn edit_config(raw: &str, adapter: Adapter, entry: Option<&Value>) -> Result<String, String> {
    if adapter == Adapter::Codex {
        let mut doc: toml_edit::DocumentMut = raw
            .parse()
            .map_err(|_| "TOML inválido; no se modificó.".to_string())?;
        if doc.get("mcp_servers").is_none() {
            if entry.is_none() {
                return Ok(raw.into());
            }
            doc["mcp_servers"] = toml_edit::Item::Table(toml_edit::Table::new());
        }
        let inline = doc["mcp_servers"].is_inline_table();
        let root = doc["mcp_servers"]
            .as_table_like_mut()
            .ok_or("mcp_servers debe ser una tabla.")?;
        if let Some(entry) = entry {
            let command = entry["command"].as_str().ok_or("Falta el auxiliar MCP.")?;
            if inline {
                let mut table = toml_edit::InlineTable::new();
                table.insert("command", command.into());
                table.insert("startup_timeout_sec", 15.into());
                table.insert("tool_timeout_sec", 45.into());
                root.insert(
                    ENTRY,
                    toml_edit::Item::Value(toml_edit::Value::InlineTable(table)),
                );
            } else {
                let mut table = toml_edit::Table::new();
                table["command"] = toml_edit::value(command);
                table["startup_timeout_sec"] = toml_edit::value(15);
                table["tool_timeout_sec"] = toml_edit::value(45);
                root.insert(ENTRY, toml_edit::Item::Table(table));
            }
        } else {
            root.remove(ENTRY);
        }
        let out = doc.to_string();
        current_entry(&out, adapter)?;
        return Ok(out);
    }
    let source = if raw.trim().is_empty() { "{}" } else { raw };
    let node = parse_jsonc(source)?;
    let (mut path, alternate) = json_entry_path(&node.value, adapter)?;
    if let Some(other) = alternate
        && lookup(&node.value, &other).is_some()
    {
        if lookup(&node.value, &path).is_some() {
            return Err("Entradas MCP duplicadas; no se modificó.".into());
        }
        path = other;
    }
    let mut entry = entry.cloned();
    if adapter == Adapter::OpenCode
        && path.len() == 3
        && let Some(value) = entry.as_mut()
    {
        let map = value.as_object_mut().ok_or("Entrada MCP inválida")?;
        map.remove("enabled");
        map.insert("disabled".into(), Value::Bool(false));
    }
    let out = edit_json_path(source, &node, &path, entry.as_ref())?;
    parse_jsonc(&out)?;
    Ok(out)
}

/// Parsed spans preserve bytes outside the single edited property, including JSONC comments.
struct Node {
    start: usize,
    end: usize,
    value: Value,
    members: Vec<Member>,
}
struct Member {
    key: String,
    start: usize,
    comma: Option<usize>,
    node: Node,
}
struct JsonParser<'a> {
    raw: &'a str,
    at: usize,
}
fn parse_jsonc(raw: &str) -> Result<Node, String> {
    if raw.len() as u64 > MAX_CONFIG {
        return Err("Configuración demasiado grande.".into());
    }
    let mut parser = JsonParser { raw, at: 0 };
    parser.space()?;
    let node = parser.node(0)?;
    parser.space()?;
    if parser.at != raw.len() {
        return Err("Hay contenido adicional en la configuración JSON.".into());
    }
    Ok(node)
}
impl JsonParser<'_> {
    fn byte(&self) -> Option<u8> {
        self.raw.as_bytes().get(self.at).copied()
    }
    fn space(&mut self) -> Result<(), String> {
        loop {
            while self.byte().is_some_and(|b| b.is_ascii_whitespace()) {
                self.at += 1;
            }
            if self.raw[self.at..].starts_with("//") {
                while self.byte().is_some_and(|b| b != b'\n') {
                    self.at += 1;
                }
            } else if self.raw[self.at..].starts_with("/*") {
                let rest = &self.raw[self.at + 2..];
                self.at += 2 + rest.find("*/").ok_or("Comentario JSON sin cerrar.")? + 2;
            } else {
                break;
            }
        }
        Ok(())
    }
    fn string(&mut self) -> Result<String, String> {
        if self.byte() != Some(b'"') {
            return Err("Se esperaba una clave JSON.".into());
        }
        let start = self.at;
        self.at += 1;
        loop {
            match self.byte().ok_or("Texto JSON sin cerrar.")? {
                b'\\' => {
                    self.at += 1;
                    if self.byte().is_none() {
                        return Err("Escape JSON incompleto.".into());
                    }
                    self.at += 1;
                }
                b'"' => {
                    self.at += 1;
                    break;
                }
                _ => self.at += 1,
            }
        }
        serde_json::from_str(&self.raw[start..self.at]).map_err(|_| "Texto JSON inválido.".into())
    }
    fn node(&mut self, depth: usize) -> Result<Node, String> {
        if depth > 64 {
            return Err("La configuración JSON supera el límite de profundidad.".into());
        }
        self.space()?;
        let start = self.at;
        let mut members = Vec::new();
        let value = match self.byte().ok_or("JSON incompleto.")? {
            b'{' => {
                self.at += 1;
                self.space()?;
                let mut object = serde_json::Map::new();
                while self.byte() != Some(b'}') {
                    let member_start = self.at;
                    let key = self.string()?;
                    if object.contains_key(&key) {
                        return Err(
                            "Claves JSON duplicadas; resuelve el conflicto manualmente.".into()
                        );
                    }
                    self.space()?;
                    if self.byte() != Some(b':') {
                        return Err("Falta ':' en la configuración JSON.".into());
                    }
                    self.at += 1;
                    let child = self.node(depth + 1)?;
                    object.insert(key.clone(), child.value.clone());
                    self.space()?;
                    let comma = if self.byte() == Some(b',') {
                        let at = self.at;
                        self.at += 1;
                        Some(at)
                    } else {
                        None
                    };
                    members.push(Member {
                        key,
                        start: member_start,
                        comma,
                        node: child,
                    });
                    self.space()?;
                    if comma.is_none() && self.byte() != Some(b'}') {
                        return Err("Falta ',' en la configuración JSON.".into());
                    }
                    if self.byte().is_none() {
                        return Err("Objeto JSON sin cerrar.".into());
                    }
                }
                self.at += 1;
                Value::Object(object)
            }
            b'[' => {
                self.at += 1;
                self.space()?;
                let mut array = Vec::new();
                while self.byte() != Some(b']') {
                    array.push(self.node(depth + 1)?.value);
                    self.space()?;
                    if self.byte() == Some(b',') {
                        self.at += 1;
                        self.space()?;
                    } else if self.byte() != Some(b']') {
                        return Err("Lista JSON inválida.".into());
                    }
                    if self.byte().is_none() {
                        return Err("Lista JSON sin cerrar.".into());
                    }
                }
                self.at += 1;
                Value::Array(array)
            }
            b'"' => Value::String(self.string()?),
            _ => {
                while self.byte().is_some_and(|b| {
                    !b.is_ascii_whitespace() && ![b',', b'}', b']', b'/'].contains(&b)
                }) {
                    self.at += 1;
                }
                if self.at == start {
                    return Err("Valor JSON inválido.".into());
                }
                serde_json::from_str(&self.raw[start..self.at])
                    .map_err(|_| "Valor JSON inválido.".to_string())?
            }
        };
        Ok(Node {
            start,
            end: self.at,
            value,
            members,
        })
    }
}
fn edit_json_path(
    raw: &str,
    node: &Node,
    path: &[&str],
    value: Option<&Value>,
) -> Result<String, String> {
    if !node.value.is_object() || path.is_empty() {
        return Err("La ruta MCP no es un objeto válido.".into());
    }
    if let Some((index, member)) = node
        .members
        .iter()
        .enumerate()
        .find(|(_, m)| m.key == path[0])
    {
        if path.len() > 1 {
            return edit_json_path(raw, &member.node, &path[1..], value);
        }
        let (start, end, text) = if let Some(value) = value {
            (
                member.node.start,
                member.node.end,
                serde_json::to_string(value).map_err(|_| "Entrada inválida")?,
            )
        } else if let Some(comma) = member.comma {
            (member.start, comma + 1, String::new())
        } else if index > 0 {
            (
                node.members[index - 1].comma.ok_or("JSON inválido")?,
                member.node.end,
                String::new(),
            )
        } else {
            (member.start, member.node.end, String::new())
        };
        let mut out = raw.to_string();
        out.replace_range(start..end, &text);
        return Ok(out);
    }
    let Some(value) = value else {
        return Ok(raw.into());
    };
    let mut nested = value.clone();
    for key in path[1..].iter().rev() {
        let mut object = serde_json::Map::new();
        object.insert((*key).to_string(), nested);
        nested = Value::Object(object);
    }
    let prefix = if node.members.last().is_some_and(|m| m.comma.is_none()) {
        ","
    } else {
        ""
    };
    let addition = format!(
        "{prefix}\n  {}: {}\n",
        serde_json::to_string(path[0]).unwrap(),
        serde_json::to_string(&nested).map_err(|_| "Entrada inválida")?
    );
    let mut out = raw.to_string();
    out.insert_str(node.end - 1, &addition);
    Ok(out)
}

fn atomic_write(path: &Path, bytes: &[u8], private: bool) -> Result<(), String> {
    no_links(path)?;
    let parent = path.parent().ok_or("Destino de configuración inválido.")?;
    fs::create_dir_all(parent).map_err(|_| "No se pudo crear el directorio de configuración.")?;
    let temp = parent.join(format!(".tfl-mcp-{}.tmp", uuid::Uuid::new_v4().simple()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|_| "No se pudo preparar la escritura de configuración.")?;
        tfl_mcp::protect(&temp, false)
            .map_err(|_| "No se pudo proteger la configuración temporal.")?;
        file.write_all(bytes)
            .map_err(|_| "No se pudo escribir la configuración.")?;
        file.sync_all()
            .map_err(|_| "No se pudo sincronizar la configuración.")?;
        #[cfg(unix)]
        if !private && let Ok(meta) = fs::metadata(path) {
            fs::set_permissions(&temp, meta.permissions())
                .map_err(|_| "No se pudieron conservar los permisos.")?;
        }
        let _ = private;
        drop(file);
        fs::rename(&temp, path).map_err(|_| {
            "No se pudo reemplazar la configuración; se conservó el original.".to_string()
        })
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

/// Snapshot token refuses stale UI edits. Ownership digest refuses deleting user edits.
pub fn write(
    home: &Path,
    registry_path: &Path,
    id: &str,
    enabled: bool,
    expected_revision: &str,
    sidecar: &Path,
) -> Result<Vec<Target>, String> {
    let _lock = EDIT.lock().map_err(|_| "La conexión MCP está ocupada.")?;
    if enabled && (!sidecar.is_absolute() || !sidecar.is_file()) {
        return Err("El auxiliar MCP no está instalado. Usa un instalador que lo incluya.".into());
    }
    let list = targets(home, registry_path)?;
    let target = list
        .into_iter()
        .find(|t| t.id == id)
        .ok_or("El destino ya no está disponible. Actualiza la lista.")?;
    if (enabled && !target.installed) || !target.supported {
        return Err(target
            .reason
            .unwrap_or_else(|| "Cliente no compatible.".into()));
    }
    if target.revision != expected_revision {
        return Err(
            "La configuración cambió en otra aplicación. Actualiza y vuelve a intentar.".into(),
        );
    }
    let path = PathBuf::from(&target.path);
    let original = read_config(&path)?;
    if fingerprint(&original) != expected_revision {
        return Err(
            "La configuración cambió en otra aplicación. Actualiza y vuelve a intentar.".into(),
        );
    }
    let mut registry = read_registry(registry_path)?;
    let current = current_entry(original.as_deref().unwrap_or(""), target.adapter)?;
    if let Some(current) = &current
        && !registry.entries.get(id).is_some_and(|owned| {
            owned.adapter == target.adapter && owned.digest == semantic_digest(current)
        })
    {
        return Err(
            "La entrada tflclient no pertenece a TFL o fue editada. No se modificó.".into(),
        );
    }
    if !enabled && current.is_none() {
        return targets(home, registry_path);
    }
    let command = sidecar
        .to_str()
        .ok_or("La ruta del auxiliar no es UTF-8.")?;
    let entry = entry_for(target.adapter, command);
    let updated = edit_config(
        original.as_deref().unwrap_or(""),
        target.adapter,
        enabled.then_some(&entry),
    )?;
    if updated == original.as_deref().unwrap_or("") {
        return targets(home, registry_path);
    }
    let updated_entry = current_entry(&updated, target.adapter)?;
    if enabled {
        registry.entries.insert(
            id.into(),
            Owned {
                adapter: target.adapter,
                digest: semantic_digest(
                    updated_entry
                        .as_ref()
                        .ok_or("No se pudo validar la entrada MCP.")?,
                ),
            },
        );
    } else {
        registry.entries.remove(id);
    }
    let root = registry_path.parent().ok_or("Registro MCP inválido.")?;
    no_links(root)?;
    fs::create_dir_all(root).map_err(|_| "No se pudo crear el registro privado de conexiones.")?;
    tfl_mcp::protect(root, true).map_err(|_| "No se pudo proteger el registro MCP.")?;
    // Full backups stay private and never cross IPC. They may contain client secrets.
    if let Some(raw) = &original {
        let backup_dir = root.join("quick-connect-backups");
        no_links(&backup_dir)?;
        fs::create_dir_all(&backup_dir)
            .map_err(|_| "No se pudo crear el respaldo de configuración.")?;
        tfl_mcp::protect(&backup_dir, true).map_err(|_| "No se pudo proteger el respaldo.")?;
        atomic_write(
            &backup_dir.join(format!(
                "{}-{}.bak",
                target.client,
                uuid::Uuid::new_v4().simple()
            )),
            raw.as_bytes(),
            true,
        )?;
    }
    // Re-read after preparing the backup: common concurrent changes abort instead of overwriting.
    if fingerprint(&read_config(&path)?) != fingerprint(&original) {
        return Err("La configuración cambió durante la operación. No se reemplazó.".into());
    }
    atomic_write(&path, updated.as_bytes(), false)?;
    let registry_bytes =
        serde_json::to_vec_pretty(&registry).map_err(|_| "Registro MCP inválido.")?;
    finish_commit(&path, original, &updated, || {
        atomic_write(registry_path, &registry_bytes, true)
    })?;
    targets(home, registry_path)
}

fn finish_commit(
    path: &Path,
    original: Option<String>,
    updated: &str,
    persist: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    if let Err(error) = persist() {
        // Never roll back over a third-party edit made after our commit.
        if read_config(path)?.as_deref() == Some(updated) {
            match original {
                Some(raw) => atomic_write(path, raw.as_bytes(), false)?,
                None => fs::remove_file(path).map_err(
                    |_| "No se pudo revertir la configuración; revisa el respaldo privado.",
                )?,
            }
        }
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonc_preserves_comments_unrelated_entries_and_strings() {
        let source = "{\n // retain\n \"mcp\": {\"other\": {\"command\":[\"https://host/a/*b*/\"],},},\n \"model\":\"value\",\n}";
        let entry = entry_for(Adapter::OpenCode, "C:\\Program Files\\TFL\\mcp.exe");
        let out = edit_config(source, Adapter::OpenCode, Some(&entry)).unwrap();
        assert!(out.contains("// retain"));
        assert!(out.contains("\"other\": {\"command\":[\"https://host/a/*b*/\"],}"));
        assert!(out.contains("\"model\":\"value\""));
        assert_eq!(current_entry(&out, Adapter::OpenCode).unwrap(), Some(entry));
        let disabled = edit_config(&out, Adapter::OpenCode, None).unwrap();
        assert!(
            current_entry(&disabled, Adapter::OpenCode)
                .unwrap()
                .is_none()
        );
        assert!(disabled.contains("// retain"));
        assert!(disabled.contains("\"other\": {\"command\":[\"https://host/a/*b*/\"],}"));
    }
    #[test]
    fn opencode_v2_edits_only_native_entry_and_detects_conflict() {
        let raw = r#"{"mcp":{"servers":{"other":{"type":"local","command":["a"]}}}}"#;
        let out = edit_config(
            raw,
            Adapter::OpenCode,
            Some(&entry_for(Adapter::OpenCode, "/bin/tfl")),
        )
        .unwrap();
        let value = parse_jsonc(&out).unwrap().value;
        assert_eq!(value["mcp"]["servers"][ENTRY]["disabled"], false);
        assert!(value["mcp"].get(ENTRY).is_none());
        assert!(
            current_entry(
                r#"{"mcp":{"tflclient":{},"servers":{"tflclient":{}}}}"#,
                Adapter::OpenCode
            )
            .is_err()
        );
    }
    #[test]
    fn jsonc_rejects_duplicate_malformed_deep_and_non_objects() {
        for raw in [
            r#"{"mcp":{},"mcp":{}}"#,
            r#"{"mcp":42}"#,
            r#"{"mcp":{"servers":false}}"#,
            "{/**",
            "{}script",
            "[]",
            "{\"a\":NaN}",
        ] {
            assert!(
                edit_config(
                    raw,
                    Adapter::OpenCode,
                    Some(&entry_for(Adapter::OpenCode, "/bin/tfl"))
                )
                .is_err(),
                "{raw}"
            );
        }
        assert!(parse_jsonc(&format!("{}0{}", "[".repeat(66), "]".repeat(66))).is_err());
    }
    #[test]
    fn remove_any_json_property_keeps_siblings_valid() {
        for raw in [
            r#"{"mcpServers":{"tflclient":{},"a":1}}"#,
            r#"{"mcpServers":{"a":1,"tflclient":{}}}"#,
            r#"{"mcpServers":{"a":1,"tflclient":{},"b":2}}"#,
            r#"{"mcpServers":{"tflclient":{}}}"#,
            r#"{"mcpServers":{"a":1,"tflclient":{},}}"#,
        ] {
            let before = parse_jsonc(raw).unwrap().value;
            let out = edit_config(raw, Adapter::Claude, None).unwrap();
            let after = parse_jsonc(&out).unwrap().value;
            assert!(after["mcpServers"].get(ENTRY).is_none());
            assert_eq!(before["mcpServers"].get("a"), after["mcpServers"].get("a"));
            assert_eq!(before["mcpServers"].get("b"), after["mcpServers"].get("b"));
        }
    }
    #[test]
    fn toml_preserves_comments_credentials_and_other_servers() {
        let raw = "# owner comment\nmodel = 'keep'\n[mcp_servers.other]\ncommand = 'other' # keep\n[mcp_servers.other.env]\nSECRET_REFERENCE = 'unchanged'\n";
        let entry = entry_for(Adapter::Codex, "C:\\TFL\\mcp.exe");
        let out = edit_config(raw, Adapter::Codex, Some(&entry)).unwrap();
        assert!(out.contains(raw));
        assert_eq!(current_entry(&out, Adapter::Codex).unwrap(), Some(entry));
        let disabled = edit_config(&out, Adapter::Codex, None).unwrap();
        assert!(disabled.contains(raw));
        assert!(current_entry(&disabled, Adapter::Codex).unwrap().is_none());
    }
    #[test]
    fn inline_toml_tables_are_valid_and_claude_requires_strict_json() {
        let raw = "# keep\nmcp_servers = { other = { command = 'unrelated' } }\n";
        let entry = entry_for(Adapter::Codex, "/bin/TFL helper");
        let out = edit_config(raw, Adapter::Codex, Some(&entry)).unwrap();
        assert!(out.contains("# keep"));
        assert!(out.contains("unrelated"));
        assert_eq!(current_entry(&out, Adapter::Codex).unwrap(), Some(entry));
        assert!(current_entry("{ // comment\n\"mcpServers\":{} }", Adapter::Claude).is_err());
        assert!(current_entry("{\"mcpServers\":{},}", Adapter::Claude).is_err());
    }
    #[test]
    fn parser_handles_unicode_escapes_slashes_and_empty_document() {
        let source = "{\"note\":\"ñ \\\" \\u1234 // /*\", \"mcpServers\":{}}";
        let entry = entry_for(Adapter::Claude, "/bin/TFL space/é");
        let updated = edit_config(source, Adapter::Claude, Some(&entry)).unwrap();
        assert_eq!(
            current_entry(&updated, Adapter::Claude).unwrap(),
            Some(entry)
        );
        assert!(
            current_entry(
                &edit_config(
                    "",
                    Adapter::Claude,
                    Some(&entry_for(Adapter::Claude, "/bin/tfl"))
                )
                .unwrap(),
                Adapter::Claude
            )
            .unwrap()
            .is_some()
        );
    }
    #[test]
    fn writes_are_idempotent_refuse_stale_edits_and_preserve_foreign_changes() {
        let home = std::env::temp_dir().join(format!("tfl-mcp-fixture-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(home.join(".codex")).unwrap();
        fs::create_dir_all(home.join(".local/bin")).unwrap();
        fs::write(home.join(".local/bin/codex"), b"fixture").unwrap();
        let path = home.join(".codex/config.toml");
        fs::write(&path, "# keep\nmodel = 'original'\n").unwrap();
        let sidecar = home.join("helper.exe");
        fs::write(&sidecar, b"fixture; never execute").unwrap();
        let registry = home.join("settings/mcp/quick-connect.json");
        let initial = targets(&home, &registry)
            .unwrap()
            .into_iter()
            .find(|t| t.path == path.to_string_lossy())
            .unwrap();
        assert!(!initial.configured);
        let next = write(
            &home,
            &registry,
            &initial.id,
            true,
            &initial.revision,
            &sidecar,
        )
        .unwrap();
        let enabled = next.into_iter().find(|t| t.id == initial.id).unwrap();
        assert!(enabled.configured && enabled.managed);
        assert!(
            write(
                &home,
                &registry,
                &initial.id,
                true,
                &initial.revision,
                &sidecar
            )
            .is_err()
        );
        write(
            &home,
            &registry,
            &enabled.id,
            true,
            &enabled.revision,
            &sidecar,
        )
        .unwrap();
        let mut raw = fs::read_to_string(&path).unwrap();
        raw = raw.replace("'original'", "'edited elsewhere'");
        fs::write(&path, &raw).unwrap();
        assert!(
            write(
                &home,
                &registry,
                &enabled.id,
                false,
                &enabled.revision,
                Path::new("")
            )
            .is_err()
        );
        let refreshed = targets(&home, &registry)
            .unwrap()
            .into_iter()
            .find(|t| t.id == initial.id)
            .unwrap();
        write(
            &home,
            &registry,
            &refreshed.id,
            false,
            &refreshed.revision,
            Path::new(""),
        )
        .unwrap();
        assert!(
            fs::read_to_string(&path)
                .unwrap()
                .contains("'edited elsewhere'")
        );
        assert!(
            current_entry(&fs::read_to_string(&path).unwrap(), Adapter::Codex)
                .unwrap()
                .is_none()
        );
        assert!(home.join("settings/mcp/quick-connect-backups").is_dir());
        fs::remove_dir_all(&home).unwrap();
    }
    #[test]
    fn edited_owned_entry_cannot_be_deleted() {
        let home = std::env::temp_dir().join(format!("tfl-mcp-fixture-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(home.join(".codex")).unwrap();
        fs::create_dir_all(home.join(".local/bin")).unwrap();
        fs::write(home.join(".local/bin/codex"), b"fixture").unwrap();
        let sidecar = home.join("helper.exe");
        fs::write(&sidecar, b"fixture").unwrap();
        let registry = home.join("settings/mcp/quick-connect.json");
        let target = targets(&home, &registry)
            .unwrap()
            .into_iter()
            .find(|t| t.path == home.join(".codex/config.toml").to_string_lossy())
            .unwrap();
        let next = write(
            &home,
            &registry,
            &target.id,
            true,
            &target.revision,
            &sidecar,
        )
        .unwrap();
        let target = next.into_iter().find(|t| t.id == target.id).unwrap();
        let path = PathBuf::from(&target.path);
        fs::write(
            &path,
            fs::read_to_string(&path)
                .unwrap()
                .replace("tool_timeout_sec = 45", "tool_timeout_sec = 99"),
        )
        .unwrap();
        let edited = targets(&home, &registry)
            .unwrap()
            .into_iter()
            .find(|t| t.id == target.id)
            .unwrap();
        assert!(edited.configured && !edited.managed);
        assert!(
            write(
                &home,
                &registry,
                &edited.id,
                false,
                &edited.revision,
                &sidecar
            )
            .is_err()
        );
        assert!(fs::read_to_string(&path).unwrap().contains("99"));
        fs::remove_dir_all(&home).unwrap();
    }
    #[test]
    fn failed_registry_commit_rolls_back_without_overwriting_a_newer_edit() {
        let root = std::env::temp_dir().join(format!("tfl-mcp-fixture-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("config");
        fs::write(&path, "updated").unwrap();
        assert!(
            finish_commit(&path, Some("original".into()), "updated", || Err(
                "disk full".into()
            ))
            .is_err()
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "original");
        fs::write(&path, "foreign edit").unwrap();
        assert!(
            finish_commit(&path, Some("original".into()), "updated", || Err(
                "disk full".into()
            ))
            .is_err()
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "foreign edit");
        fs::remove_dir_all(root).unwrap();
    }
}
