//! Túnel automático vía playit.gg — hace lo que UPnP no siempre puede: la
//! dirección de conexión funciona siempre, sin importar si el router del
//! cliente soporta o tiene habilitado UPnP. Requiere vincular una cuenta de
//! playit.gg (gratis, sin tarjeta) una sola vez — el secret que devuelve
//! queda guardado en disco y se reusa para siempre, sin volver a pedir nada.
//!
//! El flujo real (verificado contra el código fuente de
//! github.com/playit-cloud/playit-agent, que es lo que corre atrás del CLI
//! oficial): un código random se registra vía `claim_setup`, el usuario abre
//! `https://playit.gg/claim/<code>` y lo acepta, `claim_exchange` canjea ese
//! código por el secret. Con el secret ya se puede crear túneles (API HTTP)
//! y correr el agente de datos (conexión UDP de control + proxy TCP/UDP)
//! embebido como librería (`playit-agent-core`), sin depender de un binario
//! externo ni de que el usuario instale nada aparte.
use playit_agent_core::network::origin_lookup::OriginLookup;
use playit_agent_core::network::tcp::tcp_settings::TcpSettings;
use playit_agent_core::network::udp::udp_settings::UdpSettings;
use playit_agent_core::playit_agent::{PlayitAgent, PlayitAgentSettings};
use playit_api_client::PlayitApi;
use playit_api_client::api::{
    AgentType, AssignedAgentCreate, ClaimSetupResponse, PortType, ReqClaimExchange, ReqClaimSetup,
    ReqTunnelsCreate, TunnelOriginCreate, TunnelType,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, OnceCell};
use tracing::{info, warn};

use crate::core::PathManager;

const API_BASE: &str = "https://api.playit.gg";

#[derive(Debug, Default, Serialize, Deserialize)]
struct PlayitConfig {
    secret_key: Option<String>,
}

/// Archivo propio, separado de `settings.tfl` — así el secret nunca pasa
/// por el flujo genérico de guardado de Ajustes del frontend (que manda el
/// objeto entero de settings de vuelta al backend), evitando que se pise
/// por accidente al cambiar cualquier otra opción.
fn config_path() -> PathBuf {
    PathManager::get().get_settings_dir().join("playit.tfl")
}

async fn load_config() -> PlayitConfig {
    match tokio::fs::read_to_string(config_path()).await {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => PlayitConfig::default(),
    }
}

async fn save_config(cfg: &PlayitConfig) -> Result<(), String> {
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    tokio::fs::write(config_path(), json).await.map_err(|e| e.to_string())
}

pub async fn is_linked() -> bool {
    load_config().await.secret_key.is_some()
}

pub async fn unlink() -> Result<(), String> {
    save_config(&PlayitConfig::default()).await
}

/// 10 caracteres hex (5 bytes random) — mismo formato que genera el CLI
/// oficial de playit.gg, el servidor valida ese formato exacto.
pub fn generate_claim_code() -> String {
    let mut buffer = [0u8; 5];
    rand::rng().fill(&mut buffer);
    hex::encode(buffer)
}

pub fn claim_url(code: &str) -> String {
    format!("https://playit.gg/claim/{code}")
}

pub enum ClaimPoll {
    Waiting,
    Accepted,
    Rejected,
}

/// Un solo chequeo de estado — el polling en sí lo hace el frontend
/// llamando esto cada 2s, no un loop bloqueante acá, así la UI puede
/// mostrar progreso y el usuario puede cancelar en cualquier momento.
pub async fn poll_claim(code: &str) -> Result<ClaimPoll, String> {
    let api = PlayitApi::create(API_BASE.to_string(), None);
    let res = api
        .claim_setup(ReqClaimSetup {
            code: code.to_string(),
            agent_type: AgentType::SelfManaged,
            version: format!("TFLClient/{}", env!("CARGO_PKG_VERSION")),
        })
        .await
        .map_err(|e| e.to_string())?;
    Ok(match res {
        ClaimSetupResponse::WaitingForUserVisit | ClaimSetupResponse::WaitingForUser => {
            ClaimPoll::Waiting
        }
        ClaimSetupResponse::UserAccepted => ClaimPoll::Accepted,
        ClaimSetupResponse::UserRejected => ClaimPoll::Rejected,
    })
}

/// Se llama una sola vez, apenas `poll_claim` devuelve `Accepted` — canjea
/// el código por el secret definitivo y lo persiste. De acá en más el
/// usuario no vuelve a ver ningún link ni tiene que repetir esto.
pub async fn finish_claim(code: &str) -> Result<(), String> {
    let api = PlayitApi::create(API_BASE.to_string(), None);
    let res = api
        .claim_exchange(ReqClaimExchange { code: code.to_string() })
        .await
        .map_err(|e| e.to_string())?;
    save_config(&PlayitConfig { secret_key: Some(res.secret_key) }).await
}

static AGENT: OnceCell<Mutex<Option<Arc<OriginLookup>>>> = OnceCell::const_new();

async fn agent_slot() -> &'static Mutex<Option<Arc<OriginLookup>>> {
    AGENT.get_or_init(|| async { Mutex::new(None) }).await
}

/// Arranca el agente de playit (conexión de control + proxy de datos) la
/// primera vez que hace falta — no hace nada si ya está corriendo. Es un
/// singleton por proceso: un solo agente atiende los túneles de todos los
/// servidores que el usuario tenga corriendo a la vez, no uno por servidor.
async fn ensure_agent_running(secret_key: &str) -> Result<(), String> {
    let mut slot = agent_slot().await.lock().await;
    if slot.is_some() {
        return Ok(());
    }

    let api = PlayitApi::create(API_BASE.to_string(), Some(secret_key.to_string()));
    let lookup = Arc::new(OriginLookup::default());

    if let Ok(data) = api.agents_rundata().await {
        lookup.update_from_run_data(&data).await;
    }

    let settings = PlayitAgentSettings {
        api_url: API_BASE.to_string(),
        secret_key: secret_key.to_string(),
        tcp_settings: TcpSettings::default(),
        udp_settings: UdpSettings::default(),
    };
    let runner = PlayitAgent::new(settings, lookup.clone())
        .await
        .map_err(|e| format!("No se pudo iniciar el túnel de playit.gg: {e}"))?;
    tokio::spawn(runner.run());

    {
        let lookup = lookup.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3));
            loop {
                interval.tick().await;
                match api.agents_rundata().await {
                    Ok(data) => lookup.update_from_run_data(&data).await,
                    Err(e) => warn!("playit.gg: no se pudo refrescar rundata: {e}"),
                }
            }
        });
    }

    info!("Agente de playit.gg iniciado");
    *slot = Some(lookup);
    Ok(())
}

/// Crea (o reusa, si ya existe una para este puerto) un túnel TCP hacia
/// `127.0.0.1:local_port` y devuelve la dirección pública lista para
/// compartir. Reintenta unos segundos porque la asignación del lado de
/// playit.gg no es instantánea las primeras veces que se crea un túnel.
pub async fn ensure_tunnel_address(local_port: u16) -> Result<String, String> {
    let secret_key = load_config()
        .await
        .secret_key
        .ok_or_else(|| "playit.gg no está vinculado".to_string())?;

    ensure_agent_running(&secret_key).await?;

    let api = PlayitApi::create(API_BASE.to_string(), Some(secret_key));
    let rundata = api.agents_rundata().await.map_err(|e| e.to_string())?;
    let agent_id = rundata.agent_id;
    let local_ip: std::net::IpAddr = "127.0.0.1".parse().unwrap();

    let existing = rundata
        .tunnels
        .iter()
        .find(|t| t.local_ip == local_ip && t.local_port == local_port)
        .map(|t| t.id);

    let tunnel_id = match existing {
        Some(id) => id,
        None => {
            let created = api
                .tunnels_create(ReqTunnelsCreate {
                    name: Some(format!("TFL Client - puerto {local_port}")),
                    tunnel_type: Some(TunnelType::MinecraftJava),
                    port_type: PortType::Tcp,
                    port_count: 1,
                    origin: TunnelOriginCreate::Agent(AssignedAgentCreate {
                        agent_id,
                        local_ip,
                        local_port: Some(local_port),
                    }),
                    enabled: true,
                    alloc: None,
                    firewall_id: None,
                    proxy_protocol: None,
                })
                .await
                .map_err(|e| e.to_string())?;
            created.id
        }
    };

    for _ in 0..10 {
        let data = api.agents_rundata().await.map_err(|e| e.to_string())?;
        if let Some(tunnel) = data.tunnels.iter().find(|t| t.id == tunnel_id) {
            if tunnel.disabled.is_none() && !tunnel.assigned_domain.is_empty() {
                return Ok(format!("{}:{}", tunnel.assigned_domain, tunnel.port.from));
            }
        }
        tokio::time::sleep(Duration::from_millis(700)).await;
    }

    Err("playit.gg no terminó de asignar la dirección del túnel — probá de nuevo en un momento".into())
}
