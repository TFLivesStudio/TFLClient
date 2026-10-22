// Copyright (C) 2025 Santiagolxx, CubicLauncher contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::error::Error;
use crate::models::{Loader, VersionManifest};
use log::{debug, info, warn};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::sync::oneshot;
use uuid::Uuid;
use zellkern::resolvers::CommandBuilder;
use zellkern::resolvers::natives::natives_subdir;
use zellkern::{LaunchConfig, extract_natives};

// ─── Internal state ───────────────────────────────────────────────────────────

pub(crate) struct InstanceInner {
    pub uuid: Uuid,
    pub manifest: VersionManifest,
    pub loader: Loader,
    pub config: LaunchConfig,
    /// Root shared directory (contains libraries/, assets/, versions/).
    pub shared_dir: PathBuf,
    /// Per-instance game directory (saves, configs, mods).
    pub instance_dir: PathBuf,
    pub stdout_tx: Sender<String>,
    pub stderr_tx: Sender<String>,
    pub runtime: Mutex<InstanceRuntime>,
}

/// `kill()` y `wait()` antes compartían el mismo `Mutex<Option<Child>>`,
/// con `wait()` reteniendo el lock mientras esperaba `child.wait().await`
/// — es decir, DURANTE TODA LA SESIÓN de juego (la tarea de fondo que
/// limpia `RUNNING`/`LAUNCHWERK` llama `wait()` apenas arranca el proceso
/// y no vuelve hasta que termina). Un `kill()` posterior se quedaba
/// esperando ESE MISMO lock para siempre: deadlock — "Detener" no hacía
/// nada, siempre, mientras el juego estuviera realmente corriendo. Ahora
/// un único task dueño del `Child` corre `select!` entre esperar su
/// salida natural y una señal de `kill()` por canal — ninguno de los dos
/// retiene el lock por más que el tiempo de tomar/dejar el canal.
pub(crate) struct InstanceRuntime {
    pub pid: Option<u32>,
    pub kill_tx: Option<oneshot::Sender<()>>,
    pub exit_rx: Option<oneshot::Receiver<Option<i32>>>,
}

// ─── Public handle ────────────────────────────────────────────────────────────

/// A handle to a prepared (or running) Minecraft instance.
///
/// Clone the receivers to watch stdout/stderr from multiple tasks.
pub struct InstanceHandle {
    pub(crate) inner: Arc<InstanceInner>,
    /// Subscribe to game stdout lines.
    pub stdout: Receiver<String>,
    /// Subscribe to game stderr lines.
    pub stderr: Receiver<String>,
}

impl InstanceHandle {
    /// UUID that uniquely identifies this instance within `Launchwerk`.
    pub fn id(&self) -> Uuid {
        self.inner.uuid
    }

    /// The parsed version manifest for this instance.
    pub fn manifest(&self) -> &VersionManifest {
        &self.inner.manifest
    }

    /// The detected / configured mod loader.
    pub fn loader(&self) -> &Loader {
        &self.inner.loader
    }

    /// Subscribe to a new stdout receiver (useful when sharing the handle
    /// between tasks – each caller gets an independent broadcast receiver).
    pub fn subscribe_stdout(&self) -> Receiver<String> {
        self.inner.stdout_tx.subscribe()
    }

    /// Subscribe to a new stderr receiver.
    pub fn subscribe_stderr(&self) -> Receiver<String> {
        self.inner.stderr_tx.subscribe()
    }

    /// Build the command, spawn the process and start pumping stdout/stderr
    /// into the broadcast channels.
    pub async fn launch(&self) -> Result<(), Error> {
        let inner = &self.inner;
        let config = &inner.config;

        debug!(
            "Launching instance {} (loader={}, version={})",
            inner.uuid, inner.loader, inner.manifest.id_raw
        );

        // ── Extract natives ───────────────────────────────────────────────
        let lib_dir = inner.shared_dir.join("libraries");
        // Resolve base_id: for Forge, inherits_from points to the vanilla version
        let base_id = inner
            .manifest
            .inherits_from
            .clone()
            .unwrap_or_else(|| inner.manifest.id_raw.clone());
        let sub = natives_subdir(&inner.manifest.id);
        let natives_dir = inner.shared_dir.join("natives").join(&base_id).join(sub);
        debug!("Natives dir (base_id={base_id}): {}", natives_dir.display());

        extract_natives(&inner.manifest, &lib_dir, &natives_dir)?;

        // ── Build the command ─────────────────────────────────────────────
        let cmd_args = CommandBuilder::new(
            &inner.manifest,
            &inner.shared_dir,
            &inner.instance_dir,
            config,
        )
        .build()?;

        debug!(
            "Launching instance {} with {} args",
            inner.uuid,
            cmd_args.len()
        );
        // Never log argument values: Minecraft's command line contains its access token.

        // ── Spawn ─────────────────────────────────────────────────────────
        let java = config.java_path.to_string_lossy().to_string();
        let mut cmd = tokio::process::Command::new(&java);
        cmd.args(&cmd_args[1..]) // cmd_args[0] is java itself
            .current_dir(&inner.instance_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        // JAVA_HOME
        if let Some(java_home) = config.java_path.parent() {
            cmd.env("JAVA_HOME", java_home);
        }

        // ── Wayland → X11 fallback ────────────────────────────────────────
        // GLFW 3.4 auto-detects Wayland when WAYLAND_DISPLAY is set, but
        // Minecraft/Forge call glfwSetWindowPos() which Wayland doesn't
        // support. Force X11 by breaking the Wayland detection.
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            info!("Wayland detected, forcing X11 for GLFW compatibility");
            cmd.env("WAYLAND_DISPLAY", "")
                .env("XDG_SESSION_TYPE", "x11");
            if std::env::var("DISPLAY").is_err() {
                cmd.env("DISPLAY", ":0");
            }
        }

        for (k, v) in &config.env {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn()?;

        // ── Pump stdout ───────────────────────────────────────────────────
        let stdout_pipe = child.stdout.take().unwrap();
        let stdout_tx = inner.stdout_tx.clone();
        let output_secret = config
            .access_token
            .as_ref()
            .filter(|token| token.len() >= 8)
            .cloned()
            .map(Arc::new);
        let stdout_secret = output_secret.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout_pipe).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if stdout_tx
                    .send(redact_game_output(
                        line,
                        stdout_secret.as_deref().map(String::as_str),
                    ))
                    .is_err()
                {
                    break; // no receivers left
                }
            }
        });

        // ── Pump stderr ───────────────────────────────────────────────────
        let stderr_pipe = child.stderr.take().unwrap();
        let stderr_tx = inner.stderr_tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr_pipe).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if stderr_tx
                    .send(redact_game_output(
                        line,
                        output_secret.as_deref().map(String::as_str),
                    ))
                    .is_err()
                {
                    break;
                }
            }
        });

        // ── Un único dueño del Child, elegible entre salida natural o kill ──
        // Guardar `child` acá adentro (en vez de en `InstanceRuntime` detrás
        // del Mutex) es lo que evita el deadlock de arriba: nadie más vuelve
        // a tocar el Child directamente — `wait()`/`kill()` solo hablan con
        // esta tarea por canal.
        let pid = child.id();
        let (kill_tx, kill_rx) = oneshot::channel::<()>();
        let (exit_tx, exit_rx) = oneshot::channel::<Option<i32>>();
        {
            let mut rt = self.inner.runtime.lock().await;
            rt.pid = pid;
            rt.kill_tx = Some(kill_tx);
            rt.exit_rx = Some(exit_rx);
        }
        let uuid = inner.uuid;
        tokio::spawn(async move {
            let code = tokio::select! {
                result = child.wait() => exit_code_from_status(uuid, result),
                _ = kill_rx => {
                    if let Err(e) = child.kill().await {
                        warn!("Error matando instancia {uuid}: {e}");
                    }
                    exit_code_from_status(uuid, child.wait().await)
                }
            };
            let _ = exit_tx.send(code);
        });

        info!("Instance {} launched ({})", inner.uuid, inner.manifest.id);
        Ok(())
    }

    /// Wait for the running process to exit and return its exit code.
    /// Returns `None` if the instance was never launched, or if another
    /// caller already consumed the exit notification — solo el primer
    /// `wait()` recibe el resultado, como ya asumía quien llama esto (una
    /// única tarea "dueña" por instancia).
    pub async fn wait(&self) -> Option<i32> {
        let rx = {
            let mut rt = self.inner.runtime.lock().await;
            rt.exit_rx.take()
        };
        match rx {
            Some(rx) => rx.await.unwrap_or(None),
            None => None,
        }
    }

    /// Send SIGKILL / TerminateProcess to the running instance. Solo avisa
    /// por canal a la tarea dueña del proceso — no espera a que termine de
    /// matarlo (eso lo observa quien llame a `wait()`).
    pub async fn kill(&self) -> Result<(), Error> {
        let tx = {
            let mut rt = self.inner.runtime.lock().await;
            rt.kill_tx.take()
        };
        if let Some(tx) = tx {
            let _ = tx.send(());
        }
        Ok(())
    }

    /// OS process id of the running Java process, if it's still running.
    /// Lets a caller monitor CPU/RAM usage externally (e.g. via `sysinfo`)
    /// without launchwerk itself needing to depend on a system-monitoring
    /// crate.
    pub async fn pid(&self) -> Option<u32> {
        self.inner.runtime.lock().await.pid
    }
}

/// `status.code()` da `None` cuando el proceso terminó por señal en vez de
/// salir normalmente — el caso típico es el OOM killer del sistema
/// operativo cortándolo (SIGKILL), que antes colapsaba a un genérico -1
/// indistinguible de cualquier otra salida anómala. En Unix se recupera la
/// señal real y se codifica como su negativo (mismo criterio que usan los
/// shells POSIX con $?), para que quien reciba el código pueda distinguir
/// "el sistema mató el proceso por falta de RAM" de otros fallos.
fn exit_code_from_status(
    uuid: Uuid,
    result: std::io::Result<std::process::ExitStatus>,
) -> Option<i32> {
    match result {
        Ok(status) => {
            let code = status.code().unwrap_or_else(|| {
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    status.signal().map(|s| -s).unwrap_or(-1)
                }
                #[cfg(not(unix))]
                {
                    -1
                }
            });
            if status.success() {
                info!("Instance {uuid} exited cleanly");
            } else {
                warn!("Instance {uuid} exited with code {code}");
            }
            Some(code)
        }
        Err(e) => {
            warn!("Error waiting on instance {uuid}: {e}");
            None
        }
    }
}

// ─── Constructor (crate-internal) ────────────────────────────────────────────

impl InstanceHandle {
    pub(crate) fn new(
        manifest: VersionManifest,
        config: LaunchConfig,
        shared_dir: PathBuf,
        instance_dir: PathBuf,
    ) -> Self {
        let (stdout_tx, stdout_rx) = broadcast::channel(512);
        let (stderr_tx, stderr_rx) = broadcast::channel(512);
        let loader = Loader::from_version_id(&manifest.id_raw);
        let uuid = Uuid::new_v4();

        let inner = Arc::new(InstanceInner {
            uuid,
            manifest,
            loader,
            config,
            shared_dir,
            instance_dir,
            stdout_tx,
            stderr_tx,
            runtime: Mutex::new(InstanceRuntime {
                pid: None,
                kill_tx: None,
                exit_rx: None,
            }),
        });

        Self {
            inner,
            stdout: stdout_rx,
            stderr: stderr_rx,
        }
    }
}

fn redact_game_output(line: String, token: Option<&str>) -> String {
    match token {
        Some(token) if !token.is_empty() => line.replace(token, "[REDACTED]"),
        _ => line,
    }
}
#[cfg(test)]
mod session_privacy_tests {
    use super::redact_game_output;
    #[test]
    fn game_output_does_not_broadcast_the_access_token() {
        assert_eq!(
            redact_game_output(
                "access=fixture-secret; fixture-secret".into(),
                Some("fixture-secret")
            ),
            "access=[REDACTED]; [REDACTED]"
        );
        assert_eq!(
            redact_game_output("ordinary log".into(), None),
            "ordinary log"
        );
    }
}
