//! Only main-window ACL can grant MCP access or approve critical actions.
use crate::services::{mcp, mcp_preferences};
use serde_json::Value;
use tauri::{AppHandle, Emitter, command};
#[command]
pub fn mcp_status(app: AppHandle) -> Value {
    mcp::status(&app)
}
#[command]
pub async fn mcp_enable(app: AppHandle, enabled: bool) -> Result<Value, String> {
    mcp::enable(app, enabled).await
}
#[command]
pub fn mcp_grant(
    app: AppHandle,
    client_id: String,
    permissions: Vec<String>,
    instances: Vec<String>,
) -> Result<(), String> {
    for id in &instances {
        uuid::Uuid::parse_str(id).map_err(|_| "Invalid instance UUID")?;
    }
    mcp::STATE
        .lock()
        .grant(&client_id, permissions, instances)?;
    mcp::notify(&app);
    Ok(())
}
#[command]
pub fn mcp_revoke(app: AppHandle, client_id: String) {
    mcp::STATE.lock().revoke(&client_id);
    mcp::notify(&app);
}
#[command]
pub fn mcp_decide(app: AppHandle, plan_id: String, approve: bool) -> Result<(), String> {
    mcp::STATE.lock().decide(&plan_id, approve)?;
    mcp::notify(&app);
    Ok(())
}
#[command]
pub fn mcp_clear_audit(app: AppHandle) {
    mcp::STATE.lock().audit.clear();
    mcp::notify(&app);
}
#[command]
pub fn visual_preferences_get() -> mcp_preferences::Preferences {
    mcp_preferences::snapshot()
}
#[command]
pub fn visual_preferences_write(
    app: AppHandle,
    key: String,
    value: Value,
    expected_revision: u32,
) -> Result<mcp_preferences::Preferences, String> {
    let next = mcp_preferences::write(&key, value, expected_revision)?;
    let _ = app.emit("mcp-preferences-changed", &next);
    Ok(next)
}

#[command]
pub async fn mcp_probe(app: AppHandle) -> Result<Value, String> {
    use tfl_mcp::{Request, Response, read_frame, write_frame};
    let mut stream =
        tfl_mcp::connect(&tfl_mcp::endpoint_path(), "Local connection diagnostic").await?;
    write_frame(
        stream.get_mut(),
        &Request {
            method: "tfl_get_app_info".into(),
            params: serde_json::json!({}),
        },
    )
    .await?;
    let raw = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        read_frame(&mut stream, 65536),
    )
    .await
    .map_err(|_| "Diagnostic timed out")??;
    let r: Response = serde_json::from_slice(&raw).map_err(|_| "Invalid diagnostic response")?;
    if !r
        .error
        .as_deref()
        .is_some_and(|s| s.starts_with("APPROVAL_REQUIRED"))
    {
        return Err("Unexpected policy response".into());
    }
    mcp::notify(&app);
    Ok(serde_json::json!({"authenticated_ipc":true,"unapproved_read_denied":true}))
}
