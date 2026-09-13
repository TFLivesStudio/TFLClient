//! Detección y auto-instalación de runtimes Java. El usuario nunca instala
//! Java a mano: se resuelve la versión requerida por la instancia y, si no
//! está presente, se descarga (aqua::jre) antes de lanzar.
use crate::core::PathManager;
use crate::services::progress;
use aqua::{DownloadManager, JreBatch, JreProviderChain};
use serde::Serialize;
use std::path::{Path, PathBuf};

const MANAGED_MAJORS: [u8; 4] = [8, 17, 21, 25];

fn runtimes_dir() -> PathBuf {
    PathManager::get().get_shared_dir().join("runtimes")
}

fn jre_home(major: u8) -> PathBuf {
    runtimes_dir().join(format!("jre{major}"))
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
            let bin = java_bin(&jre_home(major));
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
pub async fn ensure_java(major: u8) -> Result<PathBuf, String> {
    let home = jre_home(major);
    let bin = java_bin(&home);
    if bin.exists() {
        return Ok(bin);
    }

    let pkg = JreProviderChain::get_latest_package(major)
        .await
        .map_err(|e| e.to_string())?;

    let manager = DownloadManager::new(runtimes_dir());
    let batch = JreBatch::new(major, pkg, home.clone());
    let handle = manager
        .prepare_batch(Box::new(batch))
        .await
        .map_err(|e| e.to_string())?;

    let (tx, watcher) = progress::watch(&format!("java:{major}"));
    let result = handle.download_all(Some(tx)).await.map_err(|e| e.to_string());
    watcher.finish(result).await?;

    if !bin.exists() {
        return Err(format!(
            "Java {major} se descargó pero no se encontró el binario en {}",
            bin.display()
        ));
    }
    Ok(bin)
}
