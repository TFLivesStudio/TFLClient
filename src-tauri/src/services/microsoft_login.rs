//! Rust owns OAuth secrets, single-use callbacks and the commit/cancel boundary.
use crate::{core::microsoft_identity, services::SettingsManager};
use launchwerk::auth::{
    MinecraftUser,
    microsoft::MicrosoftAuth,
    oauth::{LOGIN_TIMEOUT, LoginControl, NATIVE_REDIRECT, OAuthTransaction, microsoft_navigation},
};
use parking_lot::Mutex;
use serde::Serialize;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, LazyLock, mpsc},
    time::Duration,
};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_opener::OpenerExt;
use url::Url;

#[derive(Clone, Serialize)]
pub struct LoginSnapshot {
    pub id: String,
    pub phase: String,
    pub method: String,
    pub domain: Option<String>,
    pub user_code: Option<String>,
    pub verification_uri: Option<String>,
    pub error: Option<String>,
    pub user: Option<MinecraftUser>,
    pub expires_in: u64,
    pub test_build: bool,
}
struct State {
    snapshot: LoginSnapshot,
    pending: Option<MinecraftUser>,
    committing: bool,
}
struct Attempt {
    id: String,
    control: LoginControl,
    state: Mutex<State>,
    expected_uuid: Option<String>,
}
static CURRENT: LazyLock<Mutex<Option<Arc<Attempt>>>> = LazyLock::new(|| Mutex::new(None));
fn terminal(phase: &str) -> bool {
    matches!(phase, "success" | "error" | "cancelled" | "expired")
}
impl Attempt {
    fn begin_commit(&self) -> Result<(), String> {
        let mut state = self.state.lock();
        self.control
            .check()
            .map_err(|e| safe_error(&e.to_string()))?;
        if terminal(&state.snapshot.phase) {
            return Err("AUTH_CANCELLED".into());
        }
        if state.committing {
            return Err("AUTH_FINISHING".into());
        }
        state.committing = true;
        state.snapshot.phase = "saving".into();
        Ok(())
    }
    fn request_cancel(&self) -> Result<(), String> {
        let state = self.state.lock();
        if state.committing {
            return Err("AUTH_FINISHING".into());
        }
        self.control.cancel();
        Ok(())
    }
    fn snapshot(&self) -> LoginSnapshot {
        let mut public = self.state.lock().snapshot.clone();
        public.expires_in = self
            .control
            .deadline
            .saturating_duration_since(std::time::Instant::now())
            .as_secs();
        public
    }
    fn phase(&self, phase: &str) {
        let mut state = self.state.lock();
        if !terminal(&state.snapshot.phase) {
            state.snapshot.phase = phase.into();
            if matches!(
                phase,
                "microsoft" | "xbox" | "xsts" | "minecraft" | "profile" | "saving"
            ) {
                state.snapshot.user_code = None;
                state.snapshot.verification_uri = None;
            }
        }
    }
    fn fail(&self, error: String) {
        let mut state = self.state.lock();
        if state.committing || terminal(&state.snapshot.phase) {
            return;
        }
        if let Some(mut user) = state.pending.take() {
            user.forget_tokens();
        }
        state.snapshot.user_code = None;
        state.snapshot.verification_uri = None;
        state.snapshot.phase = if error.contains("AUTH_CANCELLED") {
            "cancelled"
        } else if error.contains("AUTH_TIMEOUT") {
            "expired"
        } else {
            "error"
        }
        .into();
        state.snapshot.error = Some(safe_error(&error));
    }
}
/// Only known codes cross IPC. Never echo a network error, URL or panic payload.
pub fn safe_error(error: &str) -> String {
    const CODES: &[&str] = &[
        "AUTH_CANCELLED",
        "AUTH_TIMEOUT",
        "AUTH_DECLINED",
        "AUTH_BUSY",
        "AUTH_FINISHING",
        "ACCOUNT_MISMATCH",
        "REAUTHENTICATION_REQUIRED",
        "SECURE_STORE_UNAVAILABLE",
        "SECURE_STORE_VERIFY_FAILED",
        "SESSION_KEY_MISSING",
        "SESSION_KEY_INVALID",
        "SESSION_DECRYPTION_FAILED",
        "LEGACY_SESSION_INVALID",
        "LOCAL_STORAGE_ERROR",
        "BROWSER_UNAVAILABLE",
        "EMBEDDED_UNAVAILABLE",
        "EMBEDDED_NAVIGATION_UNSUPPORTED",
        "CALLBACK_INVALID",
        "STATE_MISMATCH",
        "AUTH_CODE_REUSED",
        "AUTH_RATE_LIMITED",
        "AUTH_SERVICE_UNAVAILABLE",
        "AUTH_RESPONSE_INVALID",
        "MICROSOFT_REGISTRATION_ERROR",
        "MICROSOFT_CONSENT_REQUIRED",
        "MICROSOFT_AUTH_ERROR",
        "MINECRAFT_APP_NOT_APPROVED",
        "MINECRAFT_SERVICES_ERROR",
        "MINECRAFT_PROFILE_REQUIRED",
        "MINECRAFT_PROFILE_INVALID",
        "MINECRAFT_PROFILE_ERROR",
        "XBOX_PROFILE_REQUIRED",
        "XBOX_FAMILY_RESTRICTION",
        "XBOX_LIVE_ERROR",
        "XBOX_XSTS_ERROR",
        "NETWORK_ERROR",
        "AUTH_TEST_CONFIGURATION_REQUIRED",
        "AUTH_CLIENT_ID_INVALID",
    ];
    CODES
        .iter()
        .find(|code| error.contains(**code))
        .unwrap_or(&"AUTH_RESPONSE_INVALID")
        .to_string()
}
fn reserve(current: &mut Option<Arc<Attempt>>, attempt: Arc<Attempt>) -> Result<(), String> {
    if current
        .as_ref()
        .is_some_and(|a| !terminal(&a.state.lock().snapshot.phase))
    {
        return Err("AUTH_BUSY".into());
    }
    *current = Some(attempt);
    Ok(())
}
fn find(id: &str) -> Result<Arc<Attempt>, String> {
    CURRENT
        .lock()
        .as_ref()
        .filter(|a| a.id == id)
        .cloned()
        .ok_or_else(|| "AUTH_OPERATION_NOT_FOUND".into())
}
pub fn status(id: &str) -> Result<LoginSnapshot, String> {
    let a = find(id)?;
    if a.control.check().is_err() && !terminal(&a.snapshot().phase) {
        a.fail("AUTH_TIMEOUT".into());
    }
    Ok(a.snapshot())
}

pub fn start(
    app: AppHandle,
    method: &str,
    expected_uuid: Option<String>,
) -> Result<LoginSnapshot, String> {
    let identity = microsoft_identity::configured()?;
    if !matches!(method, "auto" | "embedded" | "external" | "device") {
        return Err("AUTH_METHOD_INVALID".into());
    }
    if let Some(uuid) = &expected_uuid
        && !SettingsManager::read()
            .user
            .iter()
            .any(|u| u.uuid == *uuid && u.user_type == launchwerk::auth::AccountType::Microsoft)
    {
        return Err("ACCOUNT_NOT_FOUND".into());
    }
    let id = uuid::Uuid::new_v4().to_string();
    let progress_id = id.clone();
    let control = LoginControl::new(LOGIN_TIMEOUT, move |phase| {
        if let Ok(a) = find(&progress_id) {
            a.phase(phase);
        }
    });
    let attempt = Arc::new(Attempt {
        id: id.clone(),
        control,
        expected_uuid,
        state: Mutex::new(State {
            snapshot: LoginSnapshot {
                id,
                phase: "preparing".into(),
                method: method.into(),
                domain: None,
                user_code: None,
                verification_uri: None,
                error: None,
                user: None,
                expires_in: LOGIN_TIMEOUT.as_secs(),
                test_build: identity.test_build,
            },
            pending: None,
            committing: false,
        }),
    });
    reserve(&mut CURRENT.lock(), attempt.clone())?;
    let initial = attempt.snapshot();
    let method = method.to_string();
    let watchdog = attempt.clone();
    let watchdog_app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(LOGIN_TIMEOUT).await;
        if !terminal(&watchdog.snapshot().phase) {
            watchdog.control.cancel();
            watchdog.fail("AUTH_TIMEOUT".into());
            close_window(&watchdog_app, &watchdog.id);
        }
    });
    tauri::async_runtime::spawn(async move {
        let worker = attempt.clone();
        let worker_app = app.clone();
        let result = tauri::async_runtime::spawn_blocking(move || {
            run(
                &worker_app,
                &worker,
                &identity.client_id,
                identity.custom_identity,
                &method,
            )
        })
        .await;
        match result {
            Ok(Ok(user)) => {
                if let Err(error) = persist(&attempt, user, false).await {
                    attempt.fail(error);
                }
            }
            Ok(Err(error)) => attempt.fail(error),
            Err(_) => attempt.fail("AUTH_RESPONSE_INVALID".into()),
        }
        close_window(&app, &attempt.id);
    });
    Ok(initial)
}
fn close_window(app: &AppHandle, id: &str) {
    if let Some(w) = app.get_webview_window(&format!("microsoft-auth-{id}")) {
        let _ = w.close();
    }
}
pub fn cancel(app: &AppHandle, id: &str) -> Result<LoginSnapshot, String> {
    let a = find(id)?;
    a.request_cancel()?;
    a.fail("AUTH_CANCELLED".into());
    close_window(app, id);
    Ok(a.snapshot())
}
pub async fn accept_memory(id: &str) -> Result<LoginSnapshot, String> {
    let a = find(id)?;
    a.control.check().map_err(|e| safe_error(&e.to_string()))?;
    let user = {
        let mut s = a.state.lock();
        if s.snapshot.phase != "storage_required" {
            return Err("AUTH_STATE_INVALID".into());
        }
        s.pending.take().ok_or("AUTH_STATE_INVALID")?
    };
    persist(&a, user, true).await?;
    Ok(a.snapshot())
}
pub async fn retry_storage(id: &str) -> Result<LoginSnapshot, String> {
    let a = find(id)?;
    a.control.check().map_err(|e| safe_error(&e.to_string()))?;
    let user = {
        let mut s = a.state.lock();
        if s.snapshot.phase != "storage_required" {
            return Err("AUTH_STATE_INVALID".into());
        }
        s.pending.take().ok_or("AUTH_STATE_INVALID")?
    };
    persist(&a, user, false).await?;
    Ok(a.snapshot())
}
async fn persist(a: &Arc<Attempt>, mut user: MinecraftUser, memory: bool) -> Result<(), String> {
    let _account_guard = crate::services::mojang_auth::ACCOUNT_SESSION_LOCK
        .lock()
        .await;
    a.control.check().map_err(|e| safe_error(&e.to_string()))?;
    if a.expected_uuid
        .as_ref()
        .is_some_and(|uuid| !SettingsManager::read().user.iter().any(|u| u.uuid == *uuid))
    {
        return Err("ACCOUNT_MISMATCH".into());
    }
    if a.expected_uuid
        .as_ref()
        .is_some_and(|uuid| *uuid != user.uuid)
    {
        return Err("ACCOUNT_MISMATCH".into());
    }
    user.session_only = memory;
    a.begin_commit()?;
    let copy = user.clone();
    let saved = tauri::async_runtime::spawn_blocking(move || {
        if memory {
            copy.discard_persistent_tokens()
        } else {
            copy.save_tokens()
        }
    })
    .await;
    if !matches!(saved, Ok(Ok(()))) {
        let mut state = a.state.lock();
        state.committing = false;
        if a.control.check().is_err() {
            state.snapshot.phase = "expired".into();
            state.snapshot.error = Some("AUTH_TIMEOUT".into());
            return Ok(());
        }
        state.pending = Some(user);
        state.snapshot.phase = "storage_required".into();
        state.snapshot.user_code = None;
        state.snapshot.error = Some(
            if memory {
                "LOCAL_STORAGE_ERROR"
            } else {
                "SECURE_STORE_UNAVAILABLE"
            }
            .into(),
        );
        return Ok(());
    }
    let previous = SettingsManager::snapshot();
    let write = SettingsManager::write(|s| {
        s.add_user(user.clone());
        s.active_user_idx = s
            .user
            .iter()
            .position(|u| u.uuid == user.uuid)
            .unwrap_or(s.active_user_idx);
        s.onboarded = true;
    });
    let result = if write.is_ok() {
        SettingsManager::save().await
    } else {
        Err("LOCAL_STORAGE_ERROR".into())
    };
    if result.is_err() {
        let _ = SettingsManager::write(|s| {
            s.user = previous.user;
            s.active_user_idx = previous.active_user_idx;
            s.onboarded = previous.onboarded;
        });
        let mut state = a.state.lock();
        state.committing = false;
        state.pending = Some(user);
        state.snapshot.phase = "storage_required".into();
        state.snapshot.error = Some("LOCAL_STORAGE_ERROR".into());
        return Ok(());
    }
    let mut state = a.state.lock();
    state.committing = false;
    state.pending = None;
    user.forget_tokens();
    state.snapshot.user = Some(user);
    state.snapshot.phase = "success".into();
    state.snapshot.error = None;
    state.snapshot.user_code = None;
    state.snapshot.verification_uri = None;
    drop(state);
    crate::core::event_bus::accounts_changed();
    Ok(())
}
fn run(
    app: &AppHandle,
    a: &Arc<Attempt>,
    id: &str,
    custom: bool,
    method: &str,
) -> Result<MinecraftUser, String> {
    let auth = MicrosoftAuth::new(id.into());
    // Preserve the known-working production method until the new registration is
    // explicitly tested. Never assume someone else's registered redirects changed.
    let selected = if method == "auto" {
        if custom { "embedded" } else { "device" }
    } else {
        method
    };
    if selected == "device" {
        return device(app, a, &auth);
    }
    let browser = browser_or_fallback(
        selected == "embedded",
        || embedded(app, a, &auth),
        || {
            if selected == "embedded" {
                a.phase("external_fallback");
            }
            external(app, a, &auth, custom)
        },
    );
    match browser {
        Ok(None) => {
            a.phase("device_fallback");
            device(app, a, &auth)
        }
        Ok(Some((code, tx))) => auth
            .exchange_code_control(&code, &tx.verifier, tx.redirect(), &a.control)
            .map_err(|e| safe_error(&e.to_string())),
        Err(error) => Err(error),
    }
}
fn browser_or_fallback<T>(
    use_embedded: bool,
    embedded: impl FnOnce() -> Result<T, String>,
    external: impl FnOnce() -> Result<T, String>,
) -> Result<Option<T>, String> {
    if use_embedded {
        match embedded() {
            Ok(result) => return Ok(Some(result)),
            Err(error)
                if matches!(
                    error.as_str(),
                    "EMBEDDED_UNAVAILABLE" | "EMBEDDED_NAVIGATION_UNSUPPORTED"
                ) => {}
            Err(error) => return Err(error),
        }
    }
    match external() {
        Ok(result) => Ok(Some(result)),
        Err(error) if error == "BROWSER_UNAVAILABLE" => Ok(None),
        Err(error) => Err(error),
    }
}

fn device(
    _app: &AppHandle,
    a: &Arc<Attempt>,
    auth: &MicrosoftAuth,
) -> Result<MinecraftUser, String> {
    a.control
        .phase("opening")
        .map_err(|e| safe_error(&e.to_string()))?;
    let device = auth
        .get_device_code()
        .map_err(|e| safe_error(&e.to_string()))?;
    a.control.check().map_err(|e| safe_error(&e.to_string()))?;
    {
        let mut state = a.state.lock();
        state.snapshot.method = "device".into();
        state.snapshot.phase = "waiting".into();
        state.snapshot.domain = Url::parse(&device.verification_uri)
            .ok()
            .and_then(|url| url.host_str().map(str::to_string));
        state.snapshot.user_code = Some(device.user_code);
        state.snapshot.verification_uri = Some(device.verification_uri);
    }
    auth.device_login(
        &device.device_code,
        device.interval,
        device.expires_in,
        &a.control,
    )
    .map_err(|e| safe_error(&e.to_string()))
}
pub fn open_device(app: &AppHandle, id: &str) -> Result<(), String> {
    let a = find(id)?;
    a.control.check().map_err(|e| safe_error(&e.to_string()))?;
    let url = a
        .state
        .lock()
        .snapshot
        .verification_uri
        .clone()
        .ok_or("AUTH_STATE_INVALID")?;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|_| "BROWSER_UNAVAILABLE".into())
}

fn external(
    app: &AppHandle,
    a: &Arc<Attempt>,
    auth: &MicrosoftAuth,
    custom: bool,
) -> Result<(String, OAuthTransaction), String> {
    a.control.check().map_err(|e| safe_error(&e.to_string()))?;
    let listener =
        TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).map_err(|_| "BROWSER_UNAVAILABLE")?;
    listener
        .set_nonblocking(true)
        .map_err(|_| "BROWSER_UNAVAILABLE")?;
    let port = listener
        .local_addr()
        .map_err(|_| "BROWSER_UNAVAILABLE")?
        .port();
    let redirect = Url::parse(&format!(
        "http://localhost:{port}/{}",
        if custom { "" } else { "callback" }
    ))
    .map_err(|_| "CALLBACK_INVALID")?;
    let mut tx = OAuthTransaction::new(redirect, a.control.clone());
    let url = auth.authorization_url(&tx.challenge, tx.state(), tx.redirect());
    {
        let mut state = a.state.lock();
        state.snapshot.method = "external".into();
        state.snapshot.domain = Some("login.microsoftonline.com".into());
        state.snapshot.phase = "opening".into();
        state.snapshot.user_code = None;
        state.snapshot.verification_uri = None;
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|_| "BROWSER_UNAVAILABLE")?;
    a.phase("waiting");
    loop {
        a.control.check().map_err(|e| safe_error(&e.to_string()))?;
        match listener.accept() {
            Ok((mut stream, peer)) => {
                if !peer.ip().is_loopback() {
                    continue;
                }
                match callback_request(&mut stream, port, &mut tx) {
                    Ok(Some(code)) => {
                        respond(&mut stream, true);
                        return Ok((code, tx));
                    }
                    Ok(None) => respond(&mut stream, false),
                    Err(error) => {
                        respond(&mut stream, false);
                        if matches!(error.as_str(), "AUTH_DECLINED" | "MICROSOFT_AUTH_ERROR") {
                            return Err(error);
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => a
                .control
                .wait(Duration::from_millis(100))
                .map_err(|e| safe_error(&e.to_string()))?,
            Err(_) => return Err("CALLBACK_INVALID".into()),
        }
    }
}
fn callback_request(
    stream: &mut TcpStream,
    port: u16,
    tx: &mut OAuthTransaction,
) -> Result<Option<String>, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .map_err(|_| "CALLBACK_INVALID")?;
    let mut bytes = Vec::new();
    let mut buf = [0u8; 1024];
    while !bytes.windows(4).any(|v| v == b"\r\n\r\n") {
        if bytes.len() >= 8192 {
            return Err("CALLBACK_INVALID".into());
        }
        let n = stream.read(&mut buf).map_err(|_| "CALLBACK_INVALID")?;
        if n == 0 {
            return Err("CALLBACK_INVALID".into());
        }
        bytes.extend_from_slice(&buf[..n]);
    }
    parse_callback(&bytes, port, tx)
}
fn parse_callback(
    bytes: &[u8],
    port: u16,
    tx: &mut OAuthTransaction,
) -> Result<Option<String>, String> {
    if bytes.len() > 8192 {
        return Err("CALLBACK_INVALID".into());
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "CALLBACK_INVALID")?;
    let mut lines = text.split("\r\n");
    let first: Vec<_> = lines.next().unwrap_or("").split(' ').collect();
    if first.len() != 3
        || first[0] != "GET"
        || first[2] != "HTTP/1.1"
        || !first[1].starts_with('/')
        || first[1].starts_with("//")
    {
        return Err("CALLBACK_INVALID".into());
    }
    let mut hosts = Vec::new();
    for line in lines.take_while(|line| !line.is_empty()) {
        let (name, value) = line.split_once(':').ok_or("CALLBACK_INVALID")?;
        if name.eq_ignore_ascii_case("host") {
            hosts.push(value.trim());
        }
        if name.eq_ignore_ascii_case("origin")
            && !Url::parse(value.trim())
                .ok()
                .is_some_and(|u| microsoft_navigation(&u))
        {
            return Err("CALLBACK_INVALID".into());
        }
        if name.eq_ignore_ascii_case("content-length") && value.trim() != "0"
            || name.eq_ignore_ascii_case("transfer-encoding")
        {
            return Err("CALLBACK_INVALID".into());
        }
    }
    if hosts.len() != 1 || !hosts[0].eq_ignore_ascii_case(&format!("localhost:{port}")) {
        return Err("CALLBACK_INVALID".into());
    }
    let url = Url::parse(&format!("http://localhost:{port}{}", first[1]))
        .map_err(|_| "CALLBACK_INVALID")?;
    if !tx.matches_redirect(&url) {
        return Ok(None);
    }
    tx.consume(&url)
        .map(|code| Some(code.to_string()))
        .map_err(|e| safe_error(&e.to_string()))
}
fn respond(stream: &mut TcpStream, ok: bool) {
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
    let nonce = launchwerk::auth::oauth::random_secret();
    // This is our own loopback confirmation document, never the Microsoft page.
    // It only removes callback parameters from visible browser history; it reads
    // neither the authorization code nor any form, cookie or account credential.
    let message = if ok {
        "Authorization received. Return to TFL Client; you can close this tab."
    } else {
        "This callback was not accepted. Return to your original Microsoft sign-in tab."
    };
    let body = format!(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><title>TFL Client</title><script nonce=\"{nonce}\">history.replaceState(null, '', '/complete');</script><h1>TFL Client</h1><p>{message}</p></html>"
    );
    let _ = write!(
        stream,
        "HTTP/1.1 {}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nPragma: no-cache\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'none'; script-src 'nonce-{}'; base-uri 'none'; frame-ancestors 'none'\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n{}",
        if ok { "200 OK" } else { "400 Bad Request" },
        body.len(),
        nonce,
        body
    );
}

fn embedded(
    app: &AppHandle,
    a: &Arc<Attempt>,
    auth: &MicrosoftAuth,
) -> Result<(String, OAuthTransaction), String> {
    let tx = OAuthTransaction::new(
        NATIVE_REDIRECT
            .parse()
            .map_err(|_| "EMBEDDED_UNAVAILABLE")?,
        a.control.clone(),
    );
    let url = auth
        .authorization_url(&tx.challenge, tx.state(), tx.redirect())
        .parse()
        .map_err(|_| "EMBEDDED_UNAVAILABLE")?;
    let tx = Arc::new(Mutex::new(Some(tx)));
    let navigation_tx = tx.clone();
    let (sender, receiver) = mpsc::sync_channel::<Result<String, String>>(1);
    let navigate = sender.clone();
    let popup = sender.clone();
    let label = format!("microsoft-auth-{}", a.id);
    {
        let mut state = a.state.lock();
        state.snapshot.method = "embedded".into();
        state.snapshot.domain = Some("login.microsoftonline.com".into());
        state.snapshot.phase = "opening".into();
    }
    // Deliberately no capability matches this label, no remote capability is granted,
    // no initialization script, no IPC command or arbitrary opener is exposed.
    let page_attempt = a.clone();
    let loaded = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let page_loaded = loaded.clone();
    let first_load_deadline = std::time::Instant::now() + Duration::from_secs(20);
    let builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(url))
        .title("Iniciar sesión con Microsoft — login.microsoftonline.com")
        .inner_size(540.0, 740.0)
        .min_inner_size(420.0, 580.0)
        .resizable(true)
        .decorations(true)
        .incognito(true)
        .devtools(false)
        .disable_drag_drop_handler()
        .on_navigation(move |url| {
            let mut holder = navigation_tx.lock();
            let Some(tx) = holder.as_mut() else {
                return false;
            };
            if tx.matches_redirect(url) {
                let result = tx
                    .consume(url)
                    .map(|c| c.to_string())
                    .map_err(|e| safe_error(&e.to_string()));
                let _ = navigate.try_send(result);
                return false;
            }
            if microsoft_navigation(url) {
                true
            } else {
                let _ = navigate.try_send(Err("EMBEDDED_NAVIGATION_UNSUPPORTED".into()));
                false
            }
        })
        .on_new_window(move |_, _| {
            let _ = popup.try_send(Err("EMBEDDED_NAVIGATION_UNSUPPORTED".into()));
            tauri::webview::NewWindowResponse::Deny
        })
        .on_page_load(move |window, payload| {
            if microsoft_navigation(payload.url()) {
                let host = payload.url().host_str().unwrap_or("Microsoft");
                let _ = window.set_title(&format!("Iniciar sesión con Microsoft — {host}"));
                page_attempt.state.lock().snapshot.domain = Some(host.into());
                if payload.event() == tauri::webview::PageLoadEvent::Finished {
                    page_loaded.store(true, std::sync::atomic::Ordering::SeqCst);
                    page_attempt.phase("waiting");
                } else {
                    page_attempt.phase("opening");
                }
            }
        });
    let window = builder.build().map_err(|_| "EMBEDDED_UNAVAILABLE")?;
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::CloseRequested { .. }) {
            let _ = sender.try_send(Err("AUTH_CANCELLED".into()));
        }
    });
    let result = loop {
        if !loaded.load(std::sync::atomic::Ordering::SeqCst)
            && std::time::Instant::now() >= first_load_deadline
        {
            break Err("EMBEDDED_UNAVAILABLE".into());
        }
        if let Err(error) = a.control.check() {
            break Err(safe_error(&error.to_string()));
        }
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(code)) => {
                let transaction = tx.lock().take().ok_or("AUTH_STATE_INVALID")?;
                break Ok((code, transaction));
            }
            Ok(Err(error)) => break Err(error),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break Err("EMBEDDED_UNAVAILABLE".into()),
        }
    };
    let _ = window.close();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tx() -> OAuthTransaction {
        OAuthTransaction::new(
            "http://localhost:4321/".parse().unwrap(),
            LoginControl::default(),
        )
    }
    #[test]
    fn loopback_requires_host_path_method_state_and_single_use() {
        let mut t = tx();
        let state = t.state().to_string();
        let valid =
            format!("GET /?code=fixture&state={state} HTTP/1.1\r\nHost: localhost:4321\r\n\r\n");
        for wrong in [
            valid.replace("localhost:4321", "evil.test"),
            valid.replace("GET /?", "POST /?"),
            valid.replace(&state, "wrong"),
            valid.replace("\r\n\r\n", "\r\nHost: localhost:4321\r\n\r\n"),
            valid.replace("\r\n\r\n", "\r\nOrigin: https://evil.test\r\n\r\n"),
        ] {
            assert!(parse_callback(wrong.as_bytes(), 4321, &mut t).is_err());
        }
        assert_eq!(
            parse_callback(valid.as_bytes(), 4321, &mut t)
                .unwrap()
                .as_deref(),
            Some("fixture")
        );
        assert!(parse_callback(valid.as_bytes(), 4321, &mut t).is_err());
    }
    #[test]
    fn unknown_errors_are_redacted() {
        assert_eq!(
            safe_error("https://callback?code=secret&access_token=private"),
            "AUTH_RESPONSE_INVALID"
        );
    }
}

#[cfg(test)]
mod operation_tests {
    use super::*;
    fn attempt() -> Arc<Attempt> {
        Arc::new(Attempt {
            id: "isolated-test".into(),
            control: LoginControl::default(),
            expected_uuid: None,
            state: Mutex::new(State {
                pending: None,
                committing: false,
                snapshot: LoginSnapshot {
                    id: "isolated-test".into(),
                    phase: "waiting".into(),
                    method: "device".into(),
                    domain: None,
                    user_code: Some("PUBLIC-CODE".into()),
                    verification_uri: None,
                    error: None,
                    user: None,
                    expires_in: 300,
                    test_build: false,
                },
            }),
        })
    }
    #[test]
    fn cancellation_and_commit_are_mutually_exclusive() {
        for _ in 0..64 {
            let a = attempt();
            let b = a.clone();
            let barrier = Arc::new(std::sync::Barrier::new(2));
            let other = barrier.clone();
            let commit = std::thread::spawn(move || {
                other.wait();
                b.begin_commit()
            });
            barrier.wait();
            let cancelled = a.request_cancel();
            let committed = commit.join().unwrap();
            assert_ne!(cancelled.is_ok(), committed.is_ok());
            if cancelled.is_ok() {
                a.fail("AUTH_CANCELLED".into());
                assert_eq!(a.snapshot().phase, "cancelled");
                assert!(a.snapshot().user_code.is_none());
            } else {
                assert_eq!(a.snapshot().phase, "saving");
            }
        }
    }
    #[test]
    fn expired_or_terminal_attempt_cannot_save_or_be_revived() {
        let a = attempt();
        a.phase("xbox");
        assert!(a.snapshot().user_code.is_none());
        assert!(a.snapshot().verification_uri.is_none());
        let a = attempt();
        a.fail("AUTH_TIMEOUT".into());
        assert!(a.begin_commit().is_err());
        a.phase("waiting");
        assert_eq!(a.snapshot().phase, "expired");
        let a = attempt();
        a.control.cancel();
        assert!(a.begin_commit().is_err());
    }
    #[test]
    fn secret_fields_are_not_part_of_public_progress() {
        let a = attempt();
        let mut s = a.state.lock();
        s.pending = Some(MinecraftUser::premium(
            "Fixture".into(),
            "id".into(),
            "never-access".into(),
            Some("never-refresh".into()),
        ));
        s.snapshot.phase = "storage_required".into();
        drop(s);
        let public = serde_json::to_string(&a.snapshot()).unwrap();
        assert!(!public.contains("never-"));
        assert!(!public.contains("device_code"));
        assert!(!public.contains("verifier"));
        a.request_cancel().unwrap();
        a.fail("AUTH_CANCELLED".into());
        assert!(a.state.lock().pending.is_none());
    }
    #[test]
    fn loopback_socket_rejects_host_injection_and_accepts_one_valid_response() {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let mut tx = OAuthTransaction::new(
            format!("http://localhost:{port}/").parse().unwrap(),
            LoginControl::default(),
        );
        let state = tx.state().to_string();
        let server = std::thread::spawn(move || {
            for accepted in [false, true] {
                let (mut stream, peer) = listener.accept().unwrap();
                assert!(peer.ip().is_loopback());
                let result = callback_request(&mut stream, port, &mut tx);
                if accepted {
                    assert_eq!(result.unwrap().as_deref(), Some("fixture-code"));
                } else {
                    assert!(result.is_err());
                }
                respond(&mut stream, accepted);
            }
        });
        for host in ["untrusted.invalid".to_string(), format!("localhost:{port}")] {
            let mut client = TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, port)).unwrap();
            client
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            write!(
                client,
                "GET /?code=fixture-code&state={state} HTTP/1.1\r\nHost: {host}\r\n\r\n"
            )
            .unwrap();
            let mut response = String::new();
            client.read_to_string(&mut response).unwrap();
            assert!(response.contains("Cache-Control: no-store"));
            assert!(!response.contains("fixture-code"));
        }
        server.join().unwrap();
    }
}

#[cfg(test)]
mod fallback_tests {
    use super::*;
    #[test]
    fn browser_failures_fall_back_without_hiding_denial_or_changing_identity() {
        let calls = std::cell::RefCell::new(Vec::new());
        let result = browser_or_fallback(
            true,
            || {
                calls.borrow_mut().push("embedded");
                Err("EMBEDDED_UNAVAILABLE".into())
            },
            || {
                calls.borrow_mut().push("external");
                Ok("same-client-authorization")
            },
        )
        .unwrap();
        assert_eq!(result, Some("same-client-authorization"));
        assert_eq!(*calls.borrow(), ["embedded", "external"]);
        let result = browser_or_fallback::<()>(
            true,
            || Err("EMBEDDED_NAVIGATION_UNSUPPORTED".into()),
            || Err("BROWSER_UNAVAILABLE".into()),
        )
        .unwrap();
        assert!(result.is_none());
        let result = browser_or_fallback::<()>(
            true,
            || Err("AUTH_DECLINED".into()),
            || panic!("a denial must not trigger another browser"),
        );
        assert_eq!(result.unwrap_err(), "AUTH_DECLINED");
        let result = browser_or_fallback::<()>(
            false,
            || panic!("explicit external browser does not open embedded"),
            || Err("BROWSER_UNAVAILABLE".into()),
        )
        .unwrap();
        assert!(result.is_none());
    }
    #[test]
    fn single_flight_is_released_only_after_an_attempt_terminates() {
        fn make() -> Arc<Attempt> {
            Arc::new(Attempt {
                id: uuid::Uuid::new_v4().to_string(),
                control: LoginControl::default(),
                expected_uuid: None,
                state: Mutex::new(State {
                    pending: None,
                    committing: false,
                    snapshot: LoginSnapshot {
                        id: "test".into(),
                        phase: "preparing".into(),
                        method: "auto".into(),
                        domain: None,
                        user_code: None,
                        verification_uri: None,
                        error: None,
                        user: None,
                        expires_in: 300,
                        test_build: false,
                    },
                }),
            })
        }
        let mut owner = None;
        let first = make();
        reserve(&mut owner, first.clone()).unwrap();
        assert_eq!(reserve(&mut owner, make()).unwrap_err(), "AUTH_BUSY");
        first.request_cancel().unwrap();
        first.fail("AUTH_CANCELLED".into());
        reserve(&mut owner, make()).unwrap();
        assert_ne!(owner.unwrap().id, first.id);
    }
}
