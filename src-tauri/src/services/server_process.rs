//! Ciclo de vida de procesos de servidor Minecraft — deliberadamente
//! separado del registro de cliente en `launcher.rs` (`RUNNING`, un único
//! slot global): un servidor es un proceso Java autocontenido sin auth ni
//! ventana de juego, no hay razón real para que bloquee (ni que lo
//! bloquee) tener un cliente abierto al mismo tiempo. Este registro admite
//! múltiples servidores corriendo a la vez, uno por nombre de instancia.
use crate::core::event_bus::{AppEvent, emit};
use crate::services::port_forward;
use crate::services::{instance_manager, java_manager};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::ChildStdin;
use tracing::{error, info, warn};

struct RunningServer {
    stdin: Arc<tokio::sync::Mutex<Option<ChildStdin>>>,
    pid: Option<u32>,
    port_forwarded: Arc<AtomicBool>,
    /// Puerto mapeado por UPnP, para poder cerrarlo al parar el servidor —
    /// `None` hasta que el intento de mapeo (asincrónico, corre en paralelo
    /// al arranque) termina con éxito.
    mapped_port: Arc<Mutex<Option<u16>>>,
}

static RUNNING_SERVERS: LazyLock<Mutex<HashMap<String, RunningServer>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn is_server_running(instance_name: &str) -> bool {
    RUNNING_SERVERS.lock().unwrap().contains_key(instance_name)
}

pub fn any_server_running() -> bool {
    !RUNNING_SERVERS.lock().unwrap().is_empty()
}

pub fn running_server_pid(instance_name: &str) -> Option<u32> {
    RUNNING_SERVERS
        .lock()
        .unwrap()
        .get(instance_name)
        .and_then(|r| r.pid)
}

/// Si el UPnP del router aceptó el mapeo automático del puerto — la UI usa
/// esto para decirle al usuario si ya puede compartir la IP pública tal
/// cual o si el router no lo dejó (y hace falta port forwarding manual).
pub fn is_port_forwarded(instance_name: &str) -> bool {
    RUNNING_SERVERS
        .lock()
        .unwrap()
        .get(instance_name)
        .is_some_and(|r| r.port_forwarded.load(Ordering::Relaxed))
}

/// Reading this file never changes the user's legal choice.
async fn ensure_eula(instance_dir: &std::path::Path) -> Result<(), String> {
    let content = tokio::fs::read_to_string(instance_dir.join("eula.txt"))
        .await
        .unwrap_or_default();
    if crate::services::server_ports::properties(&content)?
        .get("eula")
        .is_some_and(|v| v.eq_ignore_ascii_case("true"))
    {
        Ok(())
    } else {
        Err(
            "EULA_REQUIRED: Lee y acepta la EULA de Minecraft antes de iniciar este servidor"
                .into(),
        )
    }
}

pub async fn launch_server(instance_name: String) -> Result<(), String> {
    let mutation_guard = crate::services::content::mutation_lock(&instance_name).await;
    if is_server_running(&instance_name) {
        return Err(format!("El servidor \"{instance_name}\" ya está corriendo"));
    }

    let data = instance_manager::get_instance(&instance_name).await?;
    if data.preparation.is_some() {
        return Err("La instancia aún no está lista. Revisa su preparación en Actividades".into());
    }
    let server_type = data
        .server_type
        .ok_or_else(|| "Esta instancia no es un servidor".to_string())?;
    let jar_name = data
        .server_jar
        .clone()
        .ok_or_else(|| "Esta instancia no tiene jar de servidor instalado".to_string())?;

    let instance_dir = data.dir();
    let jar_path = aqua::path_security::safe_join(&instance_dir, &jar_name)?;
    if !jar_path.exists() {
        return Err(format!(
            "No se encuentra {jar_name} en la carpeta de la instancia. Reinstala el servidor."
        ));
    }

    ensure_eula(&instance_dir).await?;
    let java_major = aqua::infer_java_version(&data.mc_version);
    let java_path = java_manager::ensure_java(java_major, &data.mc_version)
        .await
        .inspect_err(|e| {
            error!(
                "No se pudo resolver Java {java_major} para el servidor \"{instance_name}\": {e}"
            );
        })?;

    let port_guard = crate::services::server_ports::CREATION.lock().await;
    crate::services::server_ports::check_start(&instance_dir, &instance_name).await?;

    let min_mem = data.min_memory.unwrap_or(1024);
    let max_mem = data.max_memory.unwrap_or(2048);

    info!("Lanzando servidor \"{instance_name}\" ({server_type:?}, Java {java_major})");

    let mut child = tokio::process::Command::new(&java_path)
        .arg(format!("-Xms{min_mem}M"))
        .arg(format!("-Xmx{max_mem}M"))
        .arg("-jar")
        .arg(&jar_path)
        .arg("nogui")
        .current_dir(&instance_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("No se pudo lanzar el servidor: {e}"))?;

    let pid = child.id();
    let stdin = child.stdin.take();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let stdin_arc = Arc::new(tokio::sync::Mutex::new(stdin));
    let port_forwarded = Arc::new(AtomicBool::new(false));
    let mapped_port = Arc::new(Mutex::new(None));
    RUNNING_SERVERS.lock().unwrap().insert(
        instance_name.clone(),
        RunningServer {
            stdin: stdin_arc,
            pid,
            port_forwarded: port_forwarded.clone(),
            mapped_port: mapped_port.clone(),
        },
    );

    drop(port_guard);
    drop(mutation_guard);
    let _ = instance_manager::mark_activity(&instance_name).await;
    emit(AppEvent::InstanceStatusChanged {
        name: instance_name.clone(),
        status: "running".into(),
    });

    // Abrir el puerto en el router es best-effort y no debería demorar el
    // arranque del servidor — corre en paralelo, no se espera acá.
    {
        let instance_dir = instance_dir.clone();
        let server_name = instance_name.clone();
        tokio::spawn(async move {
            let port = crate::commands::servers::read_server_port(&instance_dir).await;
            let Some(local_ip) = port_forward::detect_local_ipv4() else {
                warn!(
                    "UPnP: no se pudo detectar la IP LAN para \"{server_name}\", se omite el port forward automático"
                );
                return;
            };
            match port_forward::try_open_port(local_ip, port).await {
                Ok(()) => {
                    port_forwarded.store(true, Ordering::Relaxed);
                    *mapped_port.lock().unwrap() = Some(port);
                }
                Err(e) => {
                    warn!(
                        "UPnP: no se pudo abrir el puerto {port} automáticamente para \"{server_name}\" ({e}) — va a hacer falta port forwarding manual en el router"
                    );
                }
            }
        });
    }

    if let Some(stdout) = stdout {
        let name = instance_name.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                emit(AppEvent::InstanceLogLine {
                    instance: name.clone(),
                    stream: "stdout".into(),
                    line,
                });
            }
        });
    }
    if let Some(stderr) = stderr {
        let name = instance_name.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                emit(AppEvent::InstanceLogLine {
                    instance: name.clone(),
                    stream: "stderr".into(),
                    line,
                });
            }
        });
    }

    tokio::spawn(async move {
        let code = child.wait().await.ok().and_then(|s| s.code());
        RUNNING_SERVERS.lock().unwrap().remove(&instance_name);
        let _ = instance_manager::mark_activity(&instance_name).await;
        emit(AppEvent::InstanceExited {
            instance: instance_name.clone(),
            code,
        });
        emit(AppEvent::InstanceStatusChanged {
            name: instance_name,
            status: "stopped".into(),
        });
    });

    Ok(())
}

async fn write_stdin_line(instance_name: &str, line: &str) -> Result<(), String> {
    let stdin_arc = {
        let guard = RUNNING_SERVERS.lock().unwrap();
        let running = guard
            .get(instance_name)
            .ok_or_else(|| format!("El servidor \"{instance_name}\" no está corriendo"))?;
        running.stdin.clone()
    };
    let mut guard = stdin_arc.lock().await;
    let stdin = guard
        .as_mut()
        .ok_or_else(|| "El servidor no tiene stdin disponible".to_string())?;
    stdin
        .write_all(format!("{line}\n").as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    stdin.flush().await.map_err(|e| e.to_string())
}

/// Manda un comando de consola — solo tiene sentido en instancias de
/// servidor, el cliente valida esto antes de exponer el input al usuario.
pub async fn send_command(instance_name: String, command: String) -> Result<(), String> {
    write_stdin_line(&instance_name, &command).await
}

/// Apagado limpio vía el comando estándar de Minecraft (`stop`) — guarda el
/// mundo antes de cerrar, a diferencia de matar el proceso a la fuerza.
pub async fn stop_server(instance_name: String) -> Result<(), String> {
    write_stdin_line(&instance_name, "stop").await?;

    let mapped_port = RUNNING_SERVERS
        .lock()
        .unwrap()
        .get(&instance_name)
        .and_then(|r| *r.mapped_port.lock().unwrap());
    if let Some(port) = mapped_port {
        port_forward::close_port(port).await;
    }
    Ok(())
}
