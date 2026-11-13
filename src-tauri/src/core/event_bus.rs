use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AppEvent {
    AuthSessionNotice {
        reason: String,
    },
    InstanceCreated {
        name: String,
    },
    InstanceStatusChanged {
        name: String,
        status: String,
    },
    InstanceCrashed {
        name: String,
        reason: Option<String>,
    },
    DownloadProgress {
        task: String,
        stage: String,
        item_current: usize,
        item_total: usize,
        bytes_current: u64,
        bytes_total: u64,
        current_item: Option<String>,
    },
    DownloadFinished {
        task: String,
    },
    DownloadFailed {
        task: String,
        error: String,
    },
    InstanceLogLine {
        instance: String,
        stream: String,
        line: String,
    },
    InstanceExited {
        instance: String,
        code: Option<i32>,
    },
}

static APP_HANDLE: std::sync::OnceLock<AppHandle> = std::sync::OnceLock::new();

pub fn init(handle: AppHandle) {
    let _ = APP_HANDLE.set(handle);
}

pub fn emit(event: AppEvent) {
    if let Some(handle) = APP_HANDLE.get() {
        let _ = handle.emit("app-event", &event);
    }
}

/// Public invalidation only; credentials never enter an event payload.
pub fn accounts_changed() {
    if let Some(handle) = APP_HANDLE.get() {
        let _ = handle.emit_to("main", "accounts-changed", ());
    }
}
