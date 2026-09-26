//! Abre el puerto del servidor en el router automáticamente vía UPnP/IGD,
//! para que el cliente pueda pasar su IP pública a cualquiera sin tener que
//! entrar al router a mano (requisito del cliente: "no solo en la misma red
//! Wi-Fi"). Best-effort: muchos routers tienen UPnP apagado por defecto o no
//! lo soportan — si falla, el servidor sigue funcionando igual, solo que la
//! conexión desde internet va a requerir port forwarding manual.
use igd_next::aio::tokio::search_gateway;
use igd_next::{PortMappingProtocol, SearchOptions};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;
use tracing::{info, warn};

const LEASE_SECONDS: u32 = 0; // 0 = sin expiración, el router lo libera al cerrar/perder la ruta.

/// Intenta mapear `port` (TCP, mismo puerto interno y externo) hacia
/// `local_ip` en el gateway UPnP de la red. `local_ip` tiene que ser la IP
/// LAN de esta máquina (la misma que ya se calcula para mostrarla en la UI).
pub async fn try_open_port(local_ip: Ipv4Addr, port: u16) -> Result<(), String> {
    let gateway = search_gateway(SearchOptions {
        timeout: Some(Duration::from_secs(3)),
        ..Default::default()
    })
    .await
    .map_err(|e| e.to_string())?;

    let local_addr = SocketAddr::V4(SocketAddrV4::new(local_ip, port));
    gateway
        .add_port(
            PortMappingProtocol::TCP,
            port,
            local_addr,
            LEASE_SECONDS,
            "TFL Client - servidor Minecraft",
        )
        .await
        .map_err(|e| e.to_string())?;

    info!("UPnP: puerto {port} abierto hacia {local_ip} en el router");
    Ok(())
}

/// Deshace el mapeo al detener el servidor — no es crítico si falla (el
/// lease de 0 igual lo libera el router solo eventualmente), pero prolijo
/// dejarlo cerrado apenas el servidor para.
pub async fn close_port(port: u16) {
    let gateway = match search_gateway(SearchOptions {
        timeout: Some(Duration::from_secs(3)),
        ..Default::default()
    })
    .await
    {
        Ok(g) => g,
        Err(e) => {
            warn!("UPnP: no se pudo re-ubicar el gateway para cerrar el puerto {port}: {e}");
            return;
        }
    };
    if let Err(e) = gateway.remove_port(PortMappingProtocol::TCP, port).await {
        warn!("UPnP: no se pudo cerrar el puerto {port}: {e}");
    }
}

/// Devuelve la IP LAN como `Ipv4Addr` para pasarle a `try_open_port` —
/// reusa el mismo truco de socket UDP que ya existe para mostrarla en la UI,
/// pero tipado (UPnP solo trabaja con IPv4).
pub fn detect_local_ipv4() -> Option<Ipv4Addr> {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    match socket.local_addr().ok()?.ip() {
        IpAddr::V4(v4) => Some(v4),
        IpAddr::V6(_) => None,
    }
}
