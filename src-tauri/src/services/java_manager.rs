//! Detección y auto-instalación de runtimes Java. El usuario nunca instala
//! Java a mano: se resuelve la versión requerida por la instancia y, si no
//! está presente, se descarga (aqua::jre) antes de lanzar.
use crate::core::PathManager;
use crate::services::progress;
use aqua::{DownloadManager, JreBatch, JreProviderChain, required_arch_for_mc};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tokio::sync::Mutex;

const MANAGED_MAJORS: [u8; 4] = [8, 17, 21, 25];

// Dos lanzamientos casi simultáneos que necesitan el mismo Java (o
// distinto — se serializa toda instalación de Java, no solo por major,
// no vale la pena la complejidad de un lock por versión para algo que no
// es frecuente) podían pasar juntos el chequeo `bin.exists() == false` y
// terminar descargando/extrayendo al mismo directorio a la vez,
// pisándose entre sí. El resultado observado podía ser un runtime
// corrupto que fallaba siempre con el mismo error críptico.
static INSTALL_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn runtimes_dir() -> PathBuf {
    PathManager::get().get_shared_dir().join("runtimes")
}

/// `arch` solo difiere de la nativa del host para el caso de Minecraft
/// viejo en Mac Apple Silicon (ver `required_arch_for_mc`) — se le agrega
/// sufijo a la carpeta para no pisar/colisionar con un JRE arm64 nativo
/// ya instalado para ese mismo major (otra instancia más nueva puede
/// necesitar los dos a la vez).
fn jre_home(major: u8, arch: &str) -> PathBuf {
    if arch == aqua::current_arch() {
        runtimes_dir().join(format!("jre{major}"))
    } else {
        runtimes_dir().join(format!("jre{major}-{arch}"))
    }
}

fn java_bin(home: &Path) -> PathBuf {
    if cfg!(target_os = "windows") {
        home.join("bin").join("javaw.exe")
    } else {
        home.join("bin").join("java")
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct JavaStatus {
    pub major: u8,
    pub installed: bool,
    pub path: Option<String>,
}

pub fn status() -> Vec<JavaStatus> {
    MANAGED_MAJORS
        .iter()
        .map(|&major| {
            let bin = java_bin(&jre_home(major, aqua::current_arch()));
            let installed = bin.exists();
            JavaStatus {
                major,
                installed,
                path: installed.then(|| bin.display().to_string()),
            }
        })
        .collect()
}

/// Devuelve la ruta al binario de Java para la versión mayor pedida,
/// descargándola primero (con progreso real vía `AppEvent`) si hace falta.
/// `mc_version` decide la arquitectura del JRE (ver `required_arch_for_mc`
/// — en Mac Apple Silicon, Minecraft viejo necesita Java x64 vía Rosetta
/// porque sus natives de LWJGL no tienen build arm64).
pub async fn ensure_java(major: u8, mc_version: &str) -> Result<PathBuf, String> {
    let arch = required_arch_for_mc(mc_version);
    let home = jre_home(major, arch);
    let bin = java_bin(&home);
    if bin.exists() {
        return Ok(bin);
    }

    let _guard = INSTALL_LOCK.lock().await;
    // Re-chequear tras el lock: si otra tarea ya lo instaló mientras
    // esperábamos acá, no hay nada más que hacer.
    if bin.exists() {
        return Ok(bin);
    }

    let pkg = JreProviderChain::get_latest_package(major, arch)
        .await
        .map_err(|e| e.to_string())?;

    let manager = DownloadManager::new(runtimes_dir());
    let batch = JreBatch::new(major, pkg, home.clone());
    let handle = manager
        .prepare_batch(Box::new(batch))
        .await
        .map_err(|e| e.to_string())?;

    let (tx, watcher) = progress::watch(&format!("java:{major}"));
    let result = handle
        .download_all(Some(tx))
        .await
        .map_err(|e| e.to_string());
    watcher.finish(result).await?;

    if !bin.exists() {
        return Err(format!(
            "Java {major} se descargó pero no se encontró el binario en {}",
            bin.display()
        ));
    }
    Ok(bin)
}
