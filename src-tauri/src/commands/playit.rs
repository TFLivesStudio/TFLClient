//! Comandos para vincular/desvincular playit.gg — capa fina sobre
//! `services::playit_tunnel`, ver ahí la explicación del flujo completo.
use crate::services::playit_tunnel;
use serde::Serialize;
use tauri::{AppHandle, command};
use tauri_plugin_opener::OpenerExt;

#[command]
pub async fn playit_is_linked() -> bool {
    playit_tunnel::is_linked().await
}

#[derive(Debug, Serialize)]
pub struct PlayitClaimInfo {
    pub code: String,
    pub url: String,
}

#[command]
pub fn playit_start_claim() -> PlayitClaimInfo {
    let code = playit_tunnel::generate_claim_code();
    let url = playit_tunnel::claim_url(&code);
    PlayitClaimInfo { code, url }
}

/// El frontend llama esto cada 2s con el mismo `code` de `playit_start_claim`
/// hasta que devuelve algo distinto de "waiting" — "accepted" ya deja todo
/// vinculado y guardado, no hace falta un paso más.
#[command]
pub async fn playit_poll_claim(code: String) -> Result<String, String> {
    match playit_tunnel::poll_claim(&code).await? {
        playit_tunnel::ClaimPoll::Waiting => Ok("waiting".into()),
        playit_tunnel::ClaimPoll::Rejected => Ok("rejected".into()),
        playit_tunnel::ClaimPoll::Accepted => {
            playit_tunnel::finish_claim(&code).await?;
            Ok("accepted".into())
        }
    }
}

#[command]
pub async fn playit_unlink() -> Result<(), String> {
    playit_tunnel::unlink().await
}

/// Abre el link de vinculación en el navegador del sistema — comando
/// separado (no el `open_external_url` genérico, que está limitado a
/// GitHub a propósito) porque acá el `code` lo generamos nosotros mismos
/// en `playit_start_claim` justo antes, nunca llega una URL cruda desde
/// el frontend.
#[command]
pub fn playit_open_claim_url(app: AppHandle, code: String) -> Result<(), String> {
    if code.len() != 10 || !code.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Código de vinculación inválido".into());
    }
    app.opener()
        .open_url(playit_tunnel::claim_url(&code), None::<&str>)
        .map_err(|e| e.to_string())
}
