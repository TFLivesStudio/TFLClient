//! Skin y capa de la cuenta — API oficial de Mojang
//! (`api.minecraftservices.com`), la misma que usa el launcher oficial.
//! Solo funciona con cuentas Microsoft (necesita el access_token real de
//! Minecraft Services); cuentas offline/Yggdrasil no tienen perfil ahí.
use crate::core::HTTP;
use crate::services::mojang_auth::active_user_fresh;
use launchwerk::auth::AccountType;
use serde::{Deserialize, Serialize};
use tauri::command;
use tracing::warn;

const API_BASE: &str = "https://api.minecraftservices.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MojangSkin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MojangCape {
    pub id: String,
    pub state: String,
    pub url: String,
    #[serde(default)]
    pub alias: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MojangProfile {
    pub skins: Vec<MojangSkin>,
    pub capes: Vec<MojangCape>,
}

async fn require_microsoft_token() -> Result<String, String> {
    let user = active_user_fresh().await;
    if user.user_type != AccountType::Microsoft {
        return Err("Esto necesita una cuenta Microsoft — las cuentas offline no tienen skin ni capas propias en Mojang.".into());
    }
    // Si cargar o refrescar el token falló silenciosamente (sin internet,
    // storage corrupto, etc.) `access_token` puede quedar vacío — mandarlo
    // igual como Bearer da un 401 genérico de Mojang que no dice nada de
    // por qué. Mejor cortar acá con un mensaje que sí explica la causa real.
    if user.access_token.trim().is_empty() {
        return Err(
            "No se pudo obtener una sesión de Microsoft válida — revisá tu conexión a internet o cerrá sesión y volvé a entrar.".into(),
        );
    }
    Ok(user.access_token)
}

fn map_status(status: reqwest::StatusCode) -> String {
    match status.as_u16() {
        401 | 403 => "La sesión de Microsoft no es válida — probá cerrar sesión y volver a entrar.".into(),
        429 => "Mojang está limitando pedidos por ahora — probá de nuevo en un minuto.".into(),
        _ => format!("Mojang devolvió un error ({status})"),
    }
}

#[command]
pub async fn get_mojang_profile() -> Result<MojangProfile, String> {
    let token = require_microsoft_token().await?;
    let resp = HTTP
        .get(format!("{API_BASE}/minecraft/profile"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| {
            warn!("get_mojang_profile: fallo de red pidiendo el perfil: {e}");
            format!("No se pudo conectar con Mojang: {e}")
        })?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        warn!("get_mojang_profile: Mojang devolvió {status}: {body}");
        return Err(map_status(status));
    }
    let raw = resp.text().await.map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
        warn!("get_mojang_profile: respuesta no es JSON válido ({e}): {raw}");
        format!("Mojang devolvió una respuesta inesperada: {e}")
    })?;
    Ok(MojangProfile {
        skins: serde_json::from_value(json["skins"].clone()).unwrap_or_default(),
        capes: serde_json::from_value(json["capes"].clone()).unwrap_or_default(),
    })
}

/// `variant` es "classic" o "slim" (brazos finos tipo Alex). El archivo se
/// sube tal cual desde el picker genérico de imágenes ya existente
/// (`pick_image_file`) — si no es un PNG válido de 64x64/64x32, Mojang lo
/// rechaza con un mensaje propio, no hace falta validarlo acá de nuevo.
#[command]
pub async fn set_skin_from_file(path: String, variant: String) -> Result<(), String> {
    let token = require_microsoft_token().await?;
    let bytes = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
    let filename = std::path::Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "skin.png".into());

    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name(filename)
        .mime_str("image/png")
        .map_err(|e| e.to_string())?;
    let form = reqwest::multipart::Form::new().text("variant", variant).part("file", part);

    let resp = HTTP
        .post(format!("{API_BASE}/minecraft/profile/skins"))
        .bearer_auth(token)
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(map_status(resp.status()));
    }
    Ok(())
}

#[command]
pub async fn reset_skin() -> Result<(), String> {
    let token = require_microsoft_token().await?;
    let resp = HTTP
        .delete(format!("{API_BASE}/minecraft/profile/skins/active"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(map_status(resp.status()));
    }
    Ok(())
}

#[command]
pub async fn set_active_cape(cape_id: String) -> Result<(), String> {
    let token = require_microsoft_token().await?;
    let resp = HTTP
        .put(format!("{API_BASE}/minecraft/profile/capes/active"))
        .bearer_auth(token)
        .json(&serde_json::json!({ "capeId": cape_id }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(map_status(resp.status()));
    }
    Ok(())
}

#[command]
pub async fn hide_cape() -> Result<(), String> {
    let token = require_microsoft_token().await?;
    let resp = HTTP
        .delete(format!("{API_BASE}/minecraft/profile/capes/active"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(map_status(resp.status()));
    }
    Ok(())
}
