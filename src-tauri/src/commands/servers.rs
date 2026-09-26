//! Comandos Tauri para instancias de servidor Minecraft — crear, lanzar,
//! consola interactiva, worlds y un file manager genérico scoped a la
//! carpeta de la instancia. Todo lo que es "descargar el jar" vive en
//! `services::server_downloads`, el ciclo de vida del proceso en
//! `services::server_process` — este archivo es la capa fina que el
//! frontend invoca.
use crate::core::{HTTP, get_bytes_retrying};
use crate::services::instance_manager::{self, InstanceData, LoaderKind, ServerType};
use crate::services::{server_downloads, server_process};
use aqua::path_security::safe_join;
use serde::Serialize;
use std::time::Duration;
use tauri::command;

#[command]
pub async fn get_server_versions(server_type: String) -> Result<Vec<String>, String> {
    let kind = ServerType::parse(&server_type)
        .ok_or_else(|| format!("Tipo de servidor desconocido: {server_type}"))?;
    server_downloads::list_server_versions(kind).await
}

#[derive(Debug, Serialize)]
pub struct ServerConnectionInfo {
    pub local_ip: Option<String>,
    pub public_ip: Option<String>,
    pub port: u16,
    /// true si TFL Client logró abrir el puerto solo en el router (UPnP) —
    /// si es false, `public_ip` funciona igual pero quien se conecte desde
    /// afuera de la red va a necesitar que se abra el puerto a mano.
    pub port_forwarded: bool,
}

/// IP pública de esta conexión a internet — la única forma real de que
/// alguien fuera de la red local sepa a qué dirección conectarse. Se le
/// pregunta a un servicio externo porque no hay forma de "leerla" del
/// sistema operativo (a diferencia de la LAN, que sí sale de una interfaz
/// local). Timeout corto y sin reintento: si no hay internet o el servicio
/// no responde, se muestra solo la LAN en vez de trabar la UI.
async fn fetch_public_ip() -> Option<String> {
    let resp = HTTP
        .get("https://api.ipify.org")
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    let ip = resp.text().await.ok()?;
    let ip = ip.trim();
    (!ip.is_empty()).then(|| ip.to_string())
}

/// IP de la máquina en la red local — no hay forma portable de "preguntarle
/// al SO" directo, así que se usa el truco estándar: abrir un socket UDP
/// "conectado" a una IP externa cualquiera (8.8.8.8, no hace falta que
/// responda ni que la red tenga salida a internet) y leer qué interfaz
/// local eligió el propio SO para esa ruta — es la misma IP que usaría
/// para cualquier tráfico saliente real, sin mandar un solo byte.
fn detect_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

/// Minecraft escribe server.properties recién en el primer arranque (no al
/// descargar el jar), así que hasta ese momento no existe — se devuelve el
/// default real de Mojang (25565) en vez de fallar, es lo que igual va a
/// terminar usando la primera vez que se inicie.
pub(crate) async fn read_server_port(instance_dir: &std::path::Path) -> u16 {
    let path = instance_dir.join("server.properties");
    let Ok(content) = tokio::fs::read_to_string(&path).await else {
        return 25565;
    };
    content
        .lines()
        .find_map(|line| line.strip_prefix("server-port="))
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(25565)
}

#[command]
pub async fn get_server_connection_info(instance_name: String) -> Result<ServerConnectionInfo, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let port = read_server_port(&instance.dir()).await;
    let public_ip = fetch_public_ip().await;
    Ok(ServerConnectionInfo {
        local_ip: detect_local_ip(),
        public_ip,
        port,
        port_forwarded: server_process::is_port_forwarded(&instance_name),
    })
}

#[command]
pub async fn create_server_instance(
    name: String,
    mc_version: String,
    server_type: String,
) -> Result<InstanceData, String> {
    let kind = ServerType::parse(&server_type)
        .ok_or_else(|| format!("Tipo de servidor desconocido: {server_type}"))?;
    let download = server_downloads::resolve_server_download(kind, &mc_version).await?;

    let data = instance_manager::create_instance_full(
        name,
        mc_version.clone(),
        LoaderKind::Vanilla,
        None,
        mc_version,
        Some(kind),
        Some(download.filename.clone()),
        Some(download.build_label),
    )
    .await?;

    let bytes = get_bytes_retrying(&download.url).await.map_err(|e| {
        // La instancia ya quedó creada en disco — no la dejamos a medio
        // instalar sin avisar por qué falló la parte que sí importa (el
        // jar en sí).
        format!("Instancia creada pero no se pudo descargar el servidor: {e}")
    })?;
    tokio::fs::write(data.dir().join(&download.filename), &bytes)
        .await
        .map_err(|e| format!("Instancia creada pero no se pudo guardar el jar: {e}"))?;

    Ok(data)
}

#[command]
pub async fn launch_server(instance_name: String) -> Result<(), String> {
    server_process::launch_server(instance_name).await
}

#[command]
pub async fn stop_server(instance_name: String) -> Result<(), String> {
    server_process::stop_server(instance_name).await
}

#[command]
pub async fn send_server_command(instance_name: String, command: String) -> Result<(), String> {
    server_process::send_command(instance_name, command).await
}

#[command]
pub fn is_server_running(instance_name: String) -> bool {
    server_process::is_server_running(&instance_name)
}

// ─── Worlds ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct WorldInfo {
    pub name: String,
    pub size_bytes: u64,
}

async fn dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
            continue;
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let Ok(meta) = entry.metadata().await else { continue };
            if meta.is_dir() {
                stack.push(entry.path());
            } else {
                total += meta.len();
            }
        }
    }
    total
}

/// Un "world" de servidor es cualquier carpeta en la raíz de la instancia
/// que tenga un `level.dat` adentro — así se detectan world/world_nether/
/// world_the_end (los nombres estándar) y también mundos con nombre
/// custom (server.properties `level-name`) sin tener que parsear ese
/// archivo de propiedades.
#[command]
pub async fn list_server_worlds(instance_name: String) -> Result<Vec<WorldInfo>, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let dir = instance.dir();
    let mut worlds = Vec::new();
    let mut entries = tokio::fs::read_dir(&dir).await.map_err(|e| e.to_string())?;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() && path.join("level.dat").exists() {
            let name = entry.file_name().to_string_lossy().to_string();
            let size_bytes = dir_size(&path).await;
            worlds.push(WorldInfo { name, size_bytes });
        }
    }
    Ok(worlds)
}

#[command]
pub async fn delete_server_world(instance_name: String, world_name: String) -> Result<(), String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let path = safe_join(&instance.dir(), &world_name)?;
    if !path.join("level.dat").exists() {
        return Err("Esa carpeta no es un mundo válido".into());
    }
    tokio::fs::remove_dir_all(&path).await.map_err(|e| e.to_string())
}

// ─── File manager (genérico, scoped a la carpeta de la instancia) ───────

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
}

#[command]
pub async fn list_instance_dir(instance_name: String, subpath: String) -> Result<Vec<FileEntry>, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let dir = safe_join(&instance.dir(), &subpath)?;
    let mut out = Vec::new();
    let mut entries = tokio::fs::read_dir(&dir).await.map_err(|e| e.to_string())?;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let Ok(meta) = entry.metadata().await else { continue };
        out.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir: meta.is_dir(),
            size_bytes: if meta.is_dir() { 0 } else { meta.len() },
        });
    }
    out.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    Ok(out)
}

// 2 MiB — un archivo de texto de configuración nunca llega ni cerca; evita
// que alguien abra por error un mundo/jar de gigabytes y tire el launcher
// abajo intentando cargarlo entero como UTF-8 en memoria.
const MAX_TEXT_FILE_SIZE: u64 = 2 * 1024 * 1024;

#[command]
pub async fn read_instance_text_file(instance_name: String, subpath: String) -> Result<String, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let path = safe_join(&instance.dir(), &subpath)?;
    let meta = tokio::fs::metadata(&path).await.map_err(|e| e.to_string())?;
    if meta.len() > MAX_TEXT_FILE_SIZE {
        return Err("El archivo es demasiado grande para editar acá (más de 2 MB)".into());
    }
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|_| "No se pudo leer como texto — ¿es un archivo binario?".to_string())
}

#[command]
pub async fn write_instance_text_file(
    instance_name: String,
    subpath: String,
    content: String,
) -> Result<(), String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let path = safe_join(&instance.dir(), &subpath)?;
    tokio::fs::write(&path, content).await.map_err(|e| e.to_string())
}

#[command]
pub async fn delete_instance_path(instance_name: String, subpath: String) -> Result<(), String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let path = safe_join(&instance.dir(), &subpath)?;
    let meta = tokio::fs::metadata(&path).await.map_err(|e| e.to_string())?;
    if meta.is_dir() {
        tokio::fs::remove_dir_all(&path).await.map_err(|e| e.to_string())
    } else {
        tokio::fs::remove_file(&path).await.map_err(|e| e.to_string())
    }
}

#[command]
pub async fn create_instance_dir(instance_name: String, subpath: String) -> Result<(), String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let path = safe_join(&instance.dir(), &subpath)?;
    tokio::fs::create_dir_all(&path).await.map_err(|e| e.to_string())
}
