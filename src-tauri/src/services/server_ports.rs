//! Persistent reservations are the actual server.properties of every managed server.
//! The caller holds CREATION while selecting/writing, including concurrent UI windows.
use socket2::{Domain, Socket, Type};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::Path,
};
pub static CREATION: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn unescape(raw: &str) -> Result<String, String> {
    let mut chars = raw.chars();
    let mut units = Vec::new();
    while let Some(c) = chars.next() {
        let c = if c == '\\' {
            match chars.next().ok_or("Propiedad con escape incompleto")? {
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    if hex.len() != 4 {
                        return Err("Escape Unicode incompleto".into());
                    }
                    units.push(
                        u16::from_str_radix(&hex, 16).map_err(|_| "Escape Unicode inválido")?,
                    );
                    continue;
                }
                't' => '\t',
                'n' => '\n',
                'r' => '\r',
                'f' => '\u{c}',
                other => other,
            }
        } else {
            c
        };
        units.extend(c.encode_utf16(&mut [0; 2]).iter().copied());
    }
    String::from_utf16(&units).map_err(|_| "Unicode inválido en server.properties".into())
}
pub fn properties(raw: &str) -> Result<BTreeMap<String, String>, String> {
    if raw.len() > 262_144 {
        return Err("server.properties supera el límite permitido".into());
    }
    let mut logical = Vec::new();
    let mut pending = String::new();
    for line in raw.lines() {
        let line = if pending.is_empty() {
            line
        } else {
            line.trim_start()
        };
        pending.push_str(line);
        let slash = pending.chars().rev().take_while(|c| *c == '\\').count();
        if slash % 2 == 1 {
            pending.pop();
            continue;
        }
        logical.push(std::mem::take(&mut pending));
    }
    if !pending.is_empty() {
        logical.push(pending);
    }
    let mut out = BTreeMap::new();
    for line in logical {
        let line = line.trim_start();
        if line.is_empty() || line.starts_with(['#', '!']) {
            continue;
        }
        let mut escaped = false;
        let mut split = line.len();
        for (idx, c) in line.char_indices() {
            if !escaped && (c == '=' || c == ':' || c.is_ascii_whitespace()) {
                split = idx;
                break;
            }
            if c == '\\' {
                escaped = !escaped;
            } else {
                escaped = false;
            }
        }
        let key = unescape(&line[..split])?;
        let value = line[split..].trim_start();
        let value = value.strip_prefix(['=', ':']).unwrap_or(value).trim_start();
        out.insert(key, unescape(value)?);
    }
    Ok(out)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Port {
    pub number: u16,
    pub udp: bool,
}
pub fn configured(raw: &str) -> Result<Vec<Port>, String> {
    let p = properties(raw)?;
    let number = |key: &str, default: u16| -> Result<u16, String> {
        match p.get(key) {
            Some(v) => v
                .parse::<u16>()
                .ok()
                .filter(|v| *v > 0)
                .ok_or_else(|| format!("{key} debe ser un puerto entre 1 y 65535")),
            None => Ok(default),
        }
    };
    let mut ports = vec![Port {
        number: number("server-port", 25565)?,
        udp: false,
    }];
    if p.get("enable-query")
        .is_some_and(|v| v.eq_ignore_ascii_case("true"))
    {
        ports.push(Port {
            number: number("query.port", 25565)?,
            udp: true,
        });
    }
    if p.get("enable-rcon")
        .is_some_and(|v| v.eq_ignore_ascii_case("true"))
    {
        ports.push(Port {
            number: number("rcon.port", 25575)?,
            udp: false,
        });
    }
    for (i, a) in ports.iter().enumerate() {
        if ports[..i].iter().any(|b| b == a) {
            return Err("Dos servicios del servidor usan el mismo puerto y protocolo".into());
        }
    }
    Ok(ports)
}
pub fn available(port: Port) -> bool {
    // std::net::TcpListener enables address reuse on Unix. For a reservation
    // probe, an exclusive bind must conflict with an existing specific address.
    let bind = |ip: IpAddr, domain| -> std::io::Result<()> {
        let kind = if port.udp { Type::DGRAM } else { Type::STREAM };
        let socket = Socket::new(domain, kind, None)?;
        socket.bind(&SocketAddr::new(ip, port.number).into())?;
        if !port.udp {
            socket.listen(1)?;
        }
        Ok(())
    };
    if bind(Ipv4Addr::UNSPECIFIED.into(), Domain::IPV4).is_err() {
        return false;
    }
    let v6 = bind(Ipv6Addr::UNSPECIFIED.into(), Domain::IPV6);
    v6.is_ok()
        || v6.is_err_and(|e| {
            matches!(
                e.kind(),
                std::io::ErrorKind::AddrNotAvailable | std::io::ErrorKind::Unsupported
            ) || matches!(e.raw_os_error(), Some(47 | 97 | 10047))
        })
}
async fn reserved(except: &str) -> Result<BTreeSet<u16>, String> {
    let mut taken = BTreeSet::new();
    for instance in super::instance_manager::list_instances().await {
        if instance.server_type.is_none() || instance.name == except {
            continue;
        }
        let raw = read(&instance.dir()).await?;
        for port in configured(&raw)? {
            taken.insert(port.number);
        }
    }
    Ok(taken)
}
pub async fn read(dir: &Path) -> Result<String, String> {
    match tokio::fs::read_to_string(dir.join("server.properties")).await {
        Ok(v) => Ok(v),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(format!("No se pudo leer server.properties: {e}")),
    }
}
pub async fn reserve_port(name: &str, explicit: Option<u16>) -> Result<u16, String> {
    let taken = reserved(name).await?;
    let port = if let Some(p) = explicit {
        if p == 0
            || taken.contains(&p)
            || !available(Port {
                number: p,
                udp: false,
            })
        {
            return Err(format!(
                "El puerto explícito {p} ya está reservado u ocupado"
            ));
        }
        p
    } else {
        (25565..=25665)
            .find(|p| {
                !taken.contains(p)
                    && available(Port {
                        number: *p,
                        udp: false,
                    })
            })
            .ok_or("No hay puertos libres en el rango 25565–25665; elige uno explícito")?
    };
    Ok(port)
}

pub async fn write_reservation(dir: &Path, port: u16) -> Result<(), String> {
    let path = dir.join("server.properties");
    if path.exists() {
        return Err("La instancia ya tiene propiedades; no se reemplazan sus puertos".into());
    }
    use tokio::io::AsyncWriteExt;
    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(|e| e.to_string())?;
    f.write_all(format!("# Puerto reservado por TFL Client\nserver-port={port}\nenable-query=false\nenable-rcon=false\n").as_bytes()).await.map_err(|e|e.to_string())?;
    f.sync_all().await.map_err(|e| e.to_string())
}

pub async fn assign(dir: &Path, name: &str, explicit: Option<u16>) -> Result<u16, String> {
    let port = reserve_port(name, explicit).await?;
    write_reservation(dir, port).await?;
    Ok(port)
}
pub async fn check_start(dir: &Path, name: &str) -> Result<(), String> {
    let taken = reserved(name).await?;
    for port in configured(&read(dir).await?)? {
        if taken.contains(&port.number) {
            return Err(format!(
                "El puerto {} está reservado por otro servidor; revisa sus propiedades",
                port.number
            ));
        }
        if !available(port) {
            return Err(format!(
                "El puerto {} {} está ocupado por otro proceso. Cierra ese proceso o cambia el puerto explícitamente",
                port.number,
                if port.udp { "UDP" } else { "TCP" }
            ));
        }
    }
    Ok(())
}

pub async fn check_reserved(name: &str, content: &str) -> Result<(), String> {
    let taken = reserved(name).await?;
    for port in configured(content)? {
        if taken.contains(&port.number) {
            return Err(format!(
                "El puerto {} está reservado por otro servidor",
                port.number
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    #[test]
    fn properties_follow_java_escapes_continuations_and_last_value() {
        let p = properties(
            "!comment\nserver\\u002dport : 25566\nname=Nuevo\\\n mundo\nserver-port=25567\n",
        )
        .unwrap();
        assert_eq!(p["server-port"], "25567");
        assert_eq!(p["name"], "Nuevomundo");
    }
    #[test]
    fn services_preserve_explicit_ports_and_reject_conflicts() {
        let p=configured("server-port=25599\nenable-query=true\nquery.port=25600\nenable-rcon=true\nrcon.port=25601").unwrap();
        assert_eq!(
            p.iter().map(|p| p.number).collect::<Vec<_>>(),
            vec![25599, 25600, 25601]
        );
        assert!(configured("server-port=0").is_err());
        assert!(configured("enable-rcon=true\nrcon.port=25565").is_err());
    }
    #[test]
    fn external_listener_is_detected_without_changing_properties() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        assert!(!available(Port {
            number: listener.local_addr().unwrap().port(),
            udp: false
        }));
    }
}
