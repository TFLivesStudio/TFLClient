//! Conecta el `watch::Sender<DownloadProgress>` de aqua con el event bus de
//! Tauri, para que el frontend pinte una barra de progreso real (no un
//! spinner genérico) durante descargas de versiones/loaders/mods/Java.
use crate::core::{AppEvent, emit};
use aqua::{DownloadProgress, DownloadStage, ProgressSender};
use tokio::task::JoinHandle;

fn stage_str(stage: &DownloadStage) -> String {
    stage.as_str().to_string()
}

pub struct ProgressWatcher {
    task: String,
    handle: JoinHandle<()>,
}

impl ProgressWatcher {
    /// Termina el watcher y emite `DownloadFinished`/`DownloadFailed` según
    /// el resultado de la descarga.
    pub async fn finish(self, result: Result<(), String>) -> Result<(), String> {
        self.handle.abort();
        match &result {
            Ok(()) => emit(AppEvent::DownloadFinished { task: self.task }),
            Err(e) => emit(AppEvent::DownloadFailed {
                task: self.task,
                error: e.clone(),
            }),
        }
        result
    }
}

/// Manda un único tick "en proceso" sin porcentaje (para pasos que no
/// exponen progreso granular, como el instalador de Forge/NeoForge).
pub fn mark_processing(tx: &ProgressSender, label: &str) {
    let _ = tx.send(DownloadProgress {
        stage: DownloadStage::Processing,
        item_current: 0,
        item_total: 0,
        bytes_current: 0,
        bytes_total: 0,
        current_item: Some(label.to_string()),
        current_item_bytes: 0,
        current_item_total: None,
    });
}

/// Arranca a escuchar un canal de progreso nuevo para `task`. Usar el
/// `ProgressSender` devuelto con `handle.download_all(Some(tx))`, y llamar
/// `watcher.finish(result)` con el resultado de esa descarga.
pub fn watch(task: &str) -> (ProgressSender, ProgressWatcher) {
    let (tx, mut rx) = tokio::sync::watch::channel(DownloadProgress {
        stage: DownloadStage::Resolving,
        item_current: 0,
        item_total: 0,
        bytes_current: 0,
        bytes_total: 0,
        current_item: None,
        current_item_bytes: 0,
        current_item_total: None,
    });

    let task_owned = task.to_string();
    let handle = tokio::spawn(async move {
        while rx.changed().await.is_ok() {
            let p = rx.borrow().clone();
            emit(AppEvent::DownloadProgress {
                task: task_owned.clone(),
                stage: stage_str(&p.stage),
                item_current: p.item_current,
                item_total: p.item_total,
                bytes_current: p.bytes_current,
                bytes_total: p.bytes_total,
                current_item: p.current_item,
            });
        }
    });

    (
        tx,
        ProgressWatcher {
            task: task.to_string(),
            handle,
        },
    )
}
