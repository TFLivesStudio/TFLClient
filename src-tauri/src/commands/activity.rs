//! Comando fino sobre `services::activity_log` — la lógica vive en el
//! service, acá solo se expone a la ventana principal.

use crate::services::activity_log::{self, ActivityEntry};
use tauri::command;

const DEFAULT_LIMIT: usize = 20;

#[command]
pub async fn get_recent_activity(limit: Option<usize>) -> Vec<ActivityEntry> {
    activity_log::recent(limit.unwrap_or(DEFAULT_LIMIT)).await
}

#[command]
pub async fn get_activity_tasks() -> crate::services::activity_tasks::Snapshot {
    crate::services::activity_tasks::hub().snapshot()
}

#[command]
pub async fn cancel_activity_task(task_id: String) -> Result<(), String> {
    crate::services::activity_tasks::hub().cancel(&task_id)
}
