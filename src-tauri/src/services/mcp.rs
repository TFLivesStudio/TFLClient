//! Private authenticated loopback IPC (NOT HTTP). No listener when disabled.
use super::{
    mcp_policy::{self, Job, Policy},
    mcp_tools,
};
use crate::core::PathManager;
use parking_lot::Mutex;
use serde_json::{Value, json};
use std::sync::{Arc, LazyLock};
use tauri::{AppHandle, Emitter, Manager};
use tfl_mcp::{
    Endpoint, Request, Response,
    catalog::{self, Permission},
};
use tokio::{
    io::BufReader,
    net::{TcpListener, TcpStream},
    sync::Semaphore,
};
pub static STATE: LazyLock<Arc<Mutex<Policy>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Policy::default())));
static LISTENER: LazyLock<Mutex<Option<tauri::async_runtime::JoinHandle<()>>>> =
    LazyLock::new(|| Mutex::new(None));
static ENABLE_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));
static MUTATIONS: LazyLock<tokio::sync::Mutex<()>> = LazyLock::new(|| tokio::sync::Mutex::new(()));
fn config_path() -> std::path::PathBuf {
    PathManager::get()
        .get_settings_dir()
        .join("mcp-enabled.json")
}
fn endpoint_path() -> std::path::PathBuf {
    config_path().parent().unwrap().join("mcp/endpoint.json")
}
pub fn notify(app: &AppHandle) {
    let _ = app.emit("mcp-changed", ());
    let audit = STATE.lock().audit.clone();
    let p = config_path().with_file_name("mcp-audit.json");
    if let Ok(raw) = serde_json::to_vec(&audit) {
        let _ = std::fs::write(p, raw);
    }
}
pub fn restore(app: AppHandle) {
    if let Ok(raw) = std::fs::read(config_path().with_file_name("mcp-audit.json"))
        && raw.len() < 200000
        && let Ok(a) = serde_json::from_slice::<std::collections::VecDeque<mcp_policy::Audit>>(&raw)
    {
        STATE.lock().audit = a
            .into_iter()
            .rev()
            .take(300)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
    }
    if std::fs::read(config_path())
        .ok()
        .and_then(|b| serde_json::from_slice::<bool>(&b).ok())
        == Some(true)
    {
        tauri::async_runtime::spawn(async move {
            if let Err(e) = enable(app, true).await {
                tracing::warn!("MCP startup disabled: {e}");
            }
        });
    }
}
pub fn sidecar_path(app: &AppHandle) -> Option<String> {
    let name = if cfg!(windows) {
        "tflclient-mcp.exe"
    } else {
        "tflclient-mcp"
    };
    let mut dirs = Vec::new();
    if let Ok(p) = std::env::current_exe()
        && let Some(d) = p.parent()
    {
        dirs.push(d.to_path_buf());
    }
    if let Ok(d) = app.path().resource_dir() {
        dirs.push(d);
    }
    dirs.into_iter()
        .map(|d| d.join(name))
        .find(|p| p.is_file())
        .map(|p| p.to_string_lossy().to_string())
}
pub async fn enable(app: AppHandle, enabled: bool) -> Result<Value, String> {
    let _enable_lock = ENABLE_LOCK.lock().await;
    if let Some(h) = LISTENER.lock().take() {
        h.abort();
    }
    {
        let mut s = STATE.lock();
        s.enabled = false;
        let ids = s.clients.iter().map(|c| c.id.clone()).collect::<Vec<_>>();
        for id in ids {
            s.revoke(&id);
        }
        s.record(
            "local-ui",
            "enable",
            if enabled { "requested" } else { "disabled" },
        );
    }
    let p = endpoint_path();
    let _ = std::fs::remove_file(&p);
    std::fs::write(config_path(), b"false").map_err(|_| "Cannot save MCP state")?;
    if enabled {
        let dir = p.parent().unwrap();
        std::fs::create_dir_all(dir).map_err(|_| "Cannot create private MCP directory")?;
        tfl_mcp::protect(dir, true)
            .map_err(|_| "Cannot protect MCP directory; integration remains disabled")?;
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .map_err(|_| "Cannot bind private loopback relay")?;
        let endpoint = Endpoint {
            port: listener
                .local_addr()
                .map_err(|_| "IPC address unavailable")?
                .port(),
            token: format!(
                "{}{}",
                uuid::Uuid::new_v4().simple(),
                uuid::Uuid::new_v4().simple()
            ),
            contract: 1,
        };
        // Create a protected file before writing the secret. Its parent is owner-only.
        std::fs::File::create(&p).map_err(|_| "Cannot create MCP endpoint")?;
        tfl_mcp::protect(&p, false)
            .map_err(|_| "Cannot protect MCP endpoint; integration remains disabled")?;
        std::fs::write(
            &p,
            serde_json::to_vec(&endpoint).map_err(|_| "Endpoint serialization")?,
        )
        .map_err(|_| "Cannot save MCP endpoint")?;
        STATE.lock().enabled = true;
        std::fs::write(config_path(), b"true").map_err(|_| "Cannot save MCP state")?;
        let slots = Arc::new(Semaphore::new(8));
        let a = app.clone();
        let handle = tauri::async_runtime::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                if !addr.ip().is_loopback() {
                    continue;
                }
                let Ok(slot) = slots.clone().try_acquire_owned() else {
                    continue;
                };
                let token = endpoint.token.clone();
                let app = a.clone();
                tokio::spawn(async move {
                    let _slot = slot;
                    let _ = connection(app, stream, &token).await;
                });
            }
        });
        *LISTENER.lock() = Some(handle);
    }
    notify(&app);
    Ok(status(&app))
}
pub fn status(app: &AppHandle) -> Value {
    let mut v = STATE.lock().snapshot();
    v["sidecar_path"] = json!(sidecar_path(app));
    v["http_enabled"] = json!(false);
    v["remote_supported"] = json!(false);
    v["contract"] = json!(1);
    v
}
async fn connection(app: AppHandle, stream: TcpStream, token: &str) -> Result<(), String> {
    let mut stream = BufReader::new(stream);
    let raw = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        tfl_mcp::read_frame(&mut stream, 4096),
    )
    .await
    .map_err(|_| "Handshake timeout")??;
    let hello: Value = serde_json::from_slice(&raw).map_err(|_| "Invalid handshake")?;
    let supplied = hello["token"].as_str().unwrap_or("");
    let valid = supplied.len() == token.len()
        && supplied
            .bytes()
            .zip(token.bytes())
            .fold(0u8, |a, (b, c)| a | (b ^ c))
            == 0;
    if !valid {
        return Err("Unauthorized IPC".into());
    }
    let name = hello["client"]
        .as_str()
        .unwrap_or("MCP client")
        .chars()
        .filter(|c| !c.is_control())
        .take(80)
        .collect();
    let id = STATE.lock().connect(name)?;
    notify(&app);
    let mut start = std::time::Instant::now();
    let mut requests = 0;
    loop {
        let raw = match tokio::time::timeout(
            std::time::Duration::from_secs(300),
            tfl_mcp::read_frame(&mut stream, tfl_mcp::MAX_REQUEST),
        )
        .await
        {
            Ok(Ok(r)) => r,
            _ => break,
        };
        if start.elapsed().as_secs() >= 60 {
            start = std::time::Instant::now();
            requests = 0;
        }
        requests += 1;
        if requests > 120 {
            break;
        }
        let request: Request = match serde_json::from_slice(&raw) {
            Ok(r) => r,
            Err(_) => break,
        };
        let r = handle(app.clone(), &id, request).await;
        let response = Response::from_result(
            r.map(mcp_tools::redact_value)
                .map_err(|e| mcp_tools::redact(&e)),
        );
        if tfl_mcp::write_frame(stream.get_mut(), &response)
            .await
            .is_err()
        {
            break;
        }
    }
    STATE.lock().disconnect(&id);
    notify(&app);
    Ok(())
}
async fn handle(app: AppHandle, client: &str, r: Request) -> Result<Value, String> {
    if r.method == "__client_identity" {
        let mut s = STATE.lock();
        let c = s
            .clients
            .iter_mut()
            .find(|c| c.id == client && c.connected)
            .ok_or("SESSION_CLOSED")?;
        c.name = r.params["name"]
            .as_str()
            .unwrap_or("MCP client")
            .chars()
            .filter(|c| !c.is_control())
            .take(80)
            .collect();
        return Ok(json!({"identified":true}));
    }
    let d = catalog::definitions()
        .into_iter()
        .find(|d| d.name == r.method)
        .ok_or("UNKNOWN_TOOL")?;
    catalog::validate(&d.schema, &r.params)?;
    {
        let mut s = STATE.lock();
        let result = s.authorize(client, d.permission, r.params["instance_id"].as_str());
        s.record(
            client,
            d.name,
            if result.is_ok() {
                "authorized"
            } else {
                "denied"
            },
        );
        result?;
    }
    notify(&app);
    match d.name {
        "tfl_get_job" => {
            let s = STATE.lock();
            Ok(json!(
                s.own_job(client, r.params["job_id"].as_str().unwrap())?
            ))
        }
        "tfl_list_jobs" => {
            let s = STATE.lock();
            Ok(json!(
                s.jobs
                    .iter()
                    .rev()
                    .filter(|j| j.client_id == client)
                    .take(100)
                    .collect::<Vec<_>>()
            ))
        }
        "tfl_cancel_job" => {
            let mut s = STATE.lock();
            let j = s
                .jobs
                .iter_mut()
                .find(|j| j.client_id == client && j.id == r.params["job_id"])
                .ok_or("JOB_NOT_FOUND")?;
            if j.status != "queued" {
                return Err(
                    "CANNOT_CANCEL_RUNNING_JOB: shared installer has no transactional cancellation"
                        .into(),
                );
            }
            j.status = "cancelled".into();
            Ok(json!({"cancelled":true}))
        }
        "tfl_plan_action" => {
            let _lock = MUTATIONS.lock().await;
            let instance = mcp_tools::instance(&r.params).await?;
            let fingerprint = mcp_tools::fingerprint(&instance, &r.params).await?;
            let plan = STATE.lock().plan(client, r.params, fingerprint)?;
            notify(&app);
            Ok(json!({"plan_id":plan.id,"status":"awaiting_local_approval","expires":plan.expires}))
        }
        _ if d.job => {
            let id = {
                let mut s = STATE.lock();
                let key = r.params["idempotency_key"]
                    .as_str()
                    .map(str::to_owned)
                    .or_else(|| r.params["plan_id"].as_str().map(str::to_owned));
                if let Some(old) = s
                    .jobs
                    .iter()
                    .find(|j| j.client_id == client && j.idempotency == key && key.is_some())
                {
                    if old.tool != d.name || old.params != r.params {
                        return Err("IDEMPOTENCY_KEY_CONFLICT".into());
                    }
                    return Ok(json!({"job_id":old.id,"status":old.status}));
                }
                if d.name == "tfl_execute_plan" {
                    let plan = s
                        .plans
                        .iter()
                        .find(|p| p.id == r.params["plan_id"] && p.client_id == client)
                        .ok_or("PLAN_NOT_FOUND")?;
                    if !plan.approved || !plan.decided || plan.expires <= mcp_policy::now() {
                        return Err("LOCAL_APPROVAL_REQUIRED_OR_EXPIRED".into());
                    }
                }
                if d.name == "tfl_create_instance"
                    && s.clients
                        .iter()
                        .find(|c| c.id == client)
                        .is_some_and(|c| !c.instances.is_empty())
                {
                    return Err(
                        "INSTANCE_SCOPE_DENIED: creation requires unrestricted instance scope"
                            .into(),
                    );
                }
                s.jobs.retain(|j| {
                    j.created + 3600 > mcp_policy::now()
                        || j.status == "running"
                        || j.status == "queued"
                });
                if s.jobs.len() >= 200
                    || s.jobs
                        .iter()
                        .filter(|j| j.status == "running" || j.status == "queued")
                        .count()
                        >= 8
                {
                    return Err("JOB_LIMIT_REACHED".into());
                }
                let id = uuid::Uuid::new_v4().to_string();
                s.jobs.push(Job {
                    id: id.clone(),
                    client_id: client.into(),
                    tool: d.name.into(),
                    instance_id: r.params["instance_id"].as_str().map(str::to_owned),
                    status: "queued".into(),
                    result: None,
                    error: None,
                    created: mcp_policy::now(),
                    idempotency: key,
                    params: r.params.clone(),
                });
                id
            };
            let job_id = id.clone();
            let client = client.to_string();
            tokio::spawn(async move {
                let _lock = MUTATIONS.lock().await;
                let allowed = {
                    let mut s = STATE.lock();
                    let allowed =
                        s.authorize(&client, d.permission, r.params["instance_id"].as_str());
                    if let Some(j) = s.jobs.iter_mut().find(|j| j.id == job_id) {
                        if j.status == "cancelled" {
                            return;
                        }
                        j.status = "running".into();
                    }
                    allowed
                };
                let result = match allowed {
                    Ok(()) => mcp_tools::execute(&app, &client, d.name, r.params).await,
                    Err(e) => Err(e),
                };
                {
                    let mut s = STATE.lock();
                    if let Some(j) = s.jobs.iter_mut().find(|j| j.id == job_id) {
                        match result {
                            Ok(v) => {
                                j.status = "succeeded".into();
                                j.result = Some(mcp_tools::redact_value(v));
                            }
                            Err(e) => {
                                j.status = "failed".into();
                                j.error = Some(mcp_tools::redact(&e));
                            }
                        }
                    }
                    s.record(&client, d.name, "job_finished");
                }
                let _ = app.emit("mcp-library-changed", ());
                notify(&app);
            });
            Ok(json!({"job_id":id,"status":"queued"}))
        }
        _ => {
            if d.permission > Permission::Read {
                let _lock = MUTATIONS.lock().await;
                let v = mcp_tools::execute(&app, client, d.name, r.params).await?;
                let _ = app.emit("mcp-library-changed", ());
                Ok(v)
            } else {
                tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    mcp_tools::execute(&app, client, d.name, r.params),
                )
                .await
                .map_err(|_| "TOOL_TIMEOUT")?
            }
        }
    }
}
