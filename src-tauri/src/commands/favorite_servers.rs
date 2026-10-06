//! Servidores favoritos: una lista guardada de direcciones con su estado en
//! vivo (si responde, MOTD, jugadores, versión, latencia) vía el "Server List
//! Ping" estándar de Minecraft — el mismo que usa la pantalla de Multijugador
//! del juego, sin depender de ningún servicio de terceros.

use crate::core::PathManager;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tauri::command;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const DEFAULT_PORT: u16 = 25565;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(4);
const IO_TIMEOUT: Duration = Duration::from_secs(4);
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const MAX_FAVORITES: usize = 50;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FavoriteServer {
    pub id: String,
    pub name: String,
    pub address: String,
}

fn favorites_path() -> PathBuf {
    PathManager::get().get_settings_dir().join("favorite_servers.tfl")
}

async fn load_favorites() -> Vec<FavoriteServer> {
    match tokio::fs::read_to_string(favorites_path()).await {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

async fn save_favorites(list: &[FavoriteServer]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(list).map_err(|e| e.to_string())?;
    tokio::fs::write(favorites_path(), json)
        .await
        .map_err(|e| e.to_string())
}

/// `host` o `host:puerto` → (host, puerto). Solo caracteres de nombre de
/// host: la dirección termina en una conexión TCP saliente, no debe poder
/// llevar nada raro.
fn parse_address(address: &str) -> Result<(String, u16), String> {
    let address = address.trim();
    let (host, port) = match address.rsplit_once(':') {
        Some((h, p)) => (
            h,
            p.parse::<u16>()
                .map_err(|_| "El puerto de la dirección no es válido".to_string())?,
        ),
        None => (address, DEFAULT_PORT),
    };
    let valid_host = !host.is_empty()
        && host.len() <= 253
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_');
    if !valid_host || port == 0 {
        return Err("La dirección del servidor no es válida".into());
    }
    Ok((host.to_string(), port))
}

#[command]
pub async fn get_favorite_servers() -> Vec<FavoriteServer> {
    load_favorites().await
}

#[command]
pub async fn add_favorite_server(name: String, address: String) -> Result<FavoriteServer, String> {
    let (host, port) = parse_address(&address)?;
    let normalized = if port == DEFAULT_PORT {
        host.clone()
    } else {
        format!("{host}:{port}")
    };
    let name = name.trim();
    let name = if name.is_empty() { host.as_str() } else { name };
    let name: String = name.chars().take(40).collect();

    let mut list = load_favorites().await;
    if list.iter().any(|f| f.address.eq_ignore_ascii_case(&normalized)) {
        return Err("Ese servidor ya está en favoritos".into());
    }
    if list.len() >= MAX_FAVORITES {
        return Err(format!("Máximo {MAX_FAVORITES} servidores favoritos"));
    }
    let entry = FavoriteServer {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        address: normalized,
    };
    list.push(entry.clone());
    save_favorites(&list).await?;
    Ok(entry)
}

#[command]
pub async fn remove_favorite_server(id: String) -> Result<(), String> {
    let mut list = load_favorites().await;
    list.retain(|f| f.id != id);
    save_favorites(&list).await
}

// ── Server List Ping ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Default)]
pub struct ServerStatus {
    pub online: bool,
    pub motd: Option<String>,
    pub players_online: Option<u32>,
    pub players_max: Option<u32>,
    pub version: Option<String>,
    pub latency_ms: Option<u64>,
}

fn write_varint(buf: &mut Vec<u8>, value: i32) {
    let mut v = value as u32;
    loop {
        if v & !0x7F == 0 {
            buf.push(v as u8);
            return;
        }
        buf.push((v & 0x7F) as u8 | 0x80);
        v >>= 7;
    }
}

async fn read_varint(stream: &mut TcpStream) -> Result<i32, String> {
    let mut result: u32 = 0;
    for shift in 0..5 {
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte).await.map_err(|e| e.to_string())?;
        result |= ((byte[0] & 0x7F) as u32) << (7 * shift);
        if byte[0] & 0x80 == 0 {
            return Ok(result as i32);
        }
    }
    Err("VarInt demasiado largo".into())
}

fn framed(payload: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 5);
    write_varint(&mut out, payload.len() as i32);
    out.extend(payload);
    out
}

/// Saca los códigos de color `§x` del MOTD.
fn strip_color_codes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '§' {
            chars.next();
        } else {
            out.push(c);
        }
    }
    out
}

/// El MOTD llega como string o como componente de texto con `extra` anidado.
fn flatten_description(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(o) => {
            let mut out = o
                .get("text")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            if let Some(extra) = o.get("extra").and_then(|e| e.as_array()) {
                for part in extra {
                    out.push_str(&flatten_description(part));
                }
            }
            out
        }
        serde_json::Value::Array(a) => a.iter().map(flatten_description).collect(),
        _ => String::new(),
    }
}

async fn ping_inner(host: &str, port: u16) -> Result<ServerStatus, String> {
    let mut stream = tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect((host, port)))
        .await
        .map_err(|_| "Tiempo de conexión agotado".to_string())?
        .map_err(|e| e.to_string())?;

    // Handshake (siguiente estado = status) + pedido de status.
    let mut handshake = Vec::new();
    write_varint(&mut handshake, 0x00);
    write_varint(&mut handshake, 765);
    write_varint(&mut handshake, host.len() as i32);
    handshake.extend_from_slice(host.as_bytes());
    handshake.extend_from_slice(&port.to_be_bytes());
    write_varint(&mut handshake, 1);
    let mut request = framed(handshake);
    request.extend(framed(vec![0x00]));
    stream.write_all(&request).await.map_err(|e| e.to_string())?;

    let response = tokio::time::timeout(IO_TIMEOUT, async {
        let _packet_len = read_varint(&mut stream).await?;
        let packet_id = read_varint(&mut stream).await?;
        if packet_id != 0x00 {
            return Err("Respuesta inesperada del servidor".to_string());
        }
        let json_len = read_varint(&mut stream).await?;
        if json_len < 0 || json_len as usize > MAX_RESPONSE_BYTES {
            return Err("Respuesta demasiado grande".to_string());
        }
        let mut json = vec![0u8; json_len as usize];
        stream.read_exact(&mut json).await.map_err(|e| e.to_string())?;
        Ok(json)
    })
    .await
    .map_err(|_| "El servidor no respondió".to_string())??;

    let parsed: serde_json::Value =
        serde_json::from_slice(&response).map_err(|e| e.to_string())?;

    // Latencia: ida y vuelta de un ping aparte. Si falla no importa, el
    // resto del estado ya está.
    let mut latency_ms = None;
    let mut ping = Vec::new();
    write_varint(&mut ping, 0x01);
    ping.extend_from_slice(&0i64.to_be_bytes());
    let started = Instant::now();
    if stream.write_all(&framed(ping)).await.is_ok() {
        let pong = tokio::time::timeout(IO_TIMEOUT, async {
            read_varint(&mut stream).await?;
            read_varint(&mut stream).await
        })
        .await;
        if matches!(pong, Ok(Ok(0x01))) {
            latency_ms = Some(started.elapsed().as_millis() as u64);
        }
    }

    Ok(ServerStatus {
        online: true,
        motd: parsed
            .get("description")
            .map(|d| strip_color_codes(&flatten_description(d)).trim().to_string())
            .filter(|m| !m.is_empty()),
        players_online: parsed
            .pointer("/players/online")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32),
        players_max: parsed
            .pointer("/players/max")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32),
        version: parsed
            .pointer("/version/name")
            .and_then(|v| v.as_str())
            .map(|v| strip_color_codes(v)),
        latency_ms,
    })
}

/// Estado actual de un servidor. Un servidor caído o que no responde NO es un
/// error: devuelve `online: false`, así la lista puede mostrarlo apagado.
#[command]
pub async fn ping_server(address: String) -> Result<ServerStatus, String> {
    let (host, port) = parse_address(&address)?;
    Ok(ping_inner(&host, port).await.unwrap_or_default())
}
