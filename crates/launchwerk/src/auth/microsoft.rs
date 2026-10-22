// Copyright (C) 2026 Santiagolxx, Notstaff and CubicLauncher contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::auth::{
    MinecraftUser,
    oauth::{LoginControl, auth_error},
};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;
use serde::{Deserialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    time::{Duration, Instant},
};
use url::Url;

/// Existing CubicLauncher registration, retained until the owner activates an approved ID.
pub const DEFAULT_CLIENT_ID: &str = "cb4acffe-727f-4dd1-ad5e-fd3553d84a7f";
pub const SCOPE: &str = "XboxLive.signin offline_access";
const AUTHORIZE: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const TOKEN: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const DEVICE: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const XBL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MC_LOGIN: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const PROFILE: &str = "https://api.minecraftservices.com/minecraft/profile";

#[derive(Deserialize)]
pub struct DeviceCodeResponse {
    pub user_code: String,
    pub device_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    #[serde(default = "default_interval")]
    pub interval: u64,
}
fn default_interval() -> u64 {
    5
}
#[derive(Deserialize)]
struct MicrosoftTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}
#[derive(Deserialize)]
struct XblResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: DisplayClaims,
}
#[derive(Deserialize)]
struct DisplayClaims {
    xui: Vec<Xui>,
}
#[derive(Deserialize)]
struct Xui {
    uhs: String,
}
#[derive(Deserialize)]
struct MinecraftLoginResponse {
    access_token: String,
    #[serde(default)]
    expires_in: Option<u64>,
}
#[derive(Deserialize)]
struct MinecraftProfileResponse {
    id: String,
    name: String,
}

#[derive(Debug, Clone)]
pub struct MicrosoftAuth {
    pub client_id: String,
}
impl Default for MicrosoftAuth {
    fn default() -> Self {
        Self::new(DEFAULT_CLIENT_ID.into())
    }
}

fn http() -> crate::Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("TFLClient/MicrosoftAuth2")
        .build()
        .map_err(|_| auth_error("NETWORK_ERROR"))
}
fn body(response: reqwest::blocking::Response) -> crate::Result<(u16, serde_json::Value)> {
    let status = response.status().as_u16();
    let mut bytes = zeroize::Zeroizing::new(Vec::new());
    response
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(|_| auth_error("NETWORK_ERROR"))?;
    if bytes.len() > 65536 {
        return Err(auth_error("AUTH_RESPONSE_INVALID"));
    }
    let json = serde_json::from_slice(&bytes).map_err(|_| {
        auth_error(if status >= 500 {
            "AUTH_SERVICE_UNAVAILABLE"
        } else {
            "AUTH_RESPONSE_INVALID"
        })
    })?;
    Ok((status, json))
}
fn send(request: reqwest::blocking::RequestBuilder) -> crate::Result<(u16, serde_json::Value)> {
    body(request.send().map_err(|_| auth_error("NETWORK_ERROR"))?)
}
fn decode<T: DeserializeOwned>(json: serde_json::Value) -> crate::Result<T> {
    serde_json::from_value(json).map_err(|_| auth_error("AUTH_RESPONSE_INVALID"))
}
fn validate_initial_token(token: &MicrosoftTokenResponse) -> crate::Result<()> {
    if token.access_token.is_empty() {
        return Err(auth_error("AUTH_RESPONSE_INVALID"));
    }
    if token
        .refresh_token
        .as_ref()
        .is_none_or(|value| value.is_empty())
    {
        return Err(auth_error("MICROSOFT_CONSENT_REQUIRED"));
    }
    Ok(())
}
fn successful(status: u16) -> bool {
    (200..300).contains(&status)
}

/// Returns stable, safe codes. Never returns response bodies, OAuth codes or tokens.
pub fn service_error(stage: &str, status: u16, json: &serde_json::Value) -> crate::Error {
    let code = if status == 429 {
        "AUTH_RATE_LIMITED"
    } else if status >= 500 {
        "AUTH_SERVICE_UNAVAILABLE"
    } else {
        match stage {
            "microsoft" => match json.get("error").and_then(|v| v.as_str()) {
                Some("access_denied" | "authorization_declined") => "AUTH_DECLINED",
                Some("expired_token") => "AUTH_TIMEOUT",
                Some("invalid_grant" | "interaction_required" | "consent_required") => {
                    "REAUTHENTICATION_REQUIRED"
                }
                Some("invalid_client" | "unauthorized_client") => "MICROSOFT_REGISTRATION_ERROR",
                _ => "MICROSOFT_AUTH_ERROR",
            },
            "xbl" => "XBOX_LIVE_ERROR",
            "xsts" => match json.get("XErr").and_then(|v| v.as_u64()) {
                Some(2148916233) => "XBOX_PROFILE_REQUIRED",
                Some(2148916238) => "XBOX_FAMILY_RESTRICTION",
                _ => "XBOX_XSTS_ERROR",
            },
            "minecraft"
                if status == 403
                    && ["error", "errorMessage"].iter().any(|key| {
                        json.get(*key).and_then(|v| v.as_str()).is_some_and(|s| {
                            s.to_ascii_lowercase().contains("invalid app registration")
                        })
                    }) =>
            {
                "MINECRAFT_APP_NOT_APPROVED"
            }
            "minecraft" => "MINECRAFT_SERVICES_ERROR",
            "profile" if status == 404 => "MINECRAFT_PROFILE_REQUIRED",
            "profile" => "MINECRAFT_PROFILE_ERROR",
            _ => "AUTH_RESPONSE_INVALID",
        }
    };
    auth_error(code)
}

impl MicrosoftAuth {
    pub fn new(client_id: String) -> Self {
        Self { client_id }
    }
    pub fn generate_pkce() -> (String, String) {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        let verifier = general_purpose::URL_SAFE_NO_PAD.encode(bytes);
        let challenge =
            general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        (verifier, challenge)
    }
    pub fn authorization_url(&self, challenge: &str, state: &str, redirect: &str) -> String {
        let mut url = Url::parse(AUTHORIZE).expect("static Microsoft endpoint");
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("response_type", "code")
            .append_pair("response_mode", "query")
            .append_pair("redirect_uri", redirect)
            .append_pair("scope", SCOPE)
            .append_pair("code_challenge", challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("state", state)
            .append_pair("prompt", "select_account");
        url.into()
    }
    pub fn exchange_code(
        &self,
        code: &str,
        verifier: &str,
        redirect: &str,
    ) -> crate::Result<MinecraftUser> {
        self.exchange_code_control(code, verifier, redirect, &LoginControl::default())
    }
    pub fn exchange_code_control(
        &self,
        code: &str,
        verifier: &str,
        redirect: &str,
        control: &LoginControl,
    ) -> crate::Result<MinecraftUser> {
        self.exchange_code_at(
            code,
            verifier,
            redirect,
            control,
            [TOKEN, XBL, XSTS, MC_LOGIN, PROFILE],
        )
    }
    fn exchange_code_at(
        &self,
        code: &str,
        verifier: &str,
        redirect: &str,
        control: &LoginControl,
        endpoints: [&str; 5],
    ) -> crate::Result<MinecraftUser> {
        control.phase("microsoft")?;
        let (status, json) = send(http()?.post(endpoints[0]).form(&[
            ("client_id", self.client_id.as_str()),
            ("code", code),
            ("code_verifier", verifier),
            ("redirect_uri", redirect),
            ("grant_type", "authorization_code"),
        ]))?;
        if !successful(status) {
            return Err(service_error("microsoft", status, &json));
        }
        let token: MicrosoftTokenResponse = decode(json)?;
        validate_initial_token(&token)?;
        control.check()?;
        self.complete_login_at(
            &token.access_token,
            token.refresh_token,
            control,
            [endpoints[1], endpoints[2], endpoints[3], endpoints[4]],
        )
    }
    pub fn get_device_code(&self) -> crate::Result<DeviceCodeResponse> {
        let (status, json) = send(
            http()?
                .post(DEVICE)
                .form(&[("client_id", self.client_id.as_str()), ("scope", SCOPE)]),
        )?;
        if !successful(status) {
            return Err(service_error("microsoft", status, &json));
        }
        let device: DeviceCodeResponse = decode(json)?;
        if device.user_code.is_empty()
            || device.user_code.len() > 32
            || device.device_code.is_empty()
            || device.device_code.len() > 4096
            || device.expires_in == 0
        {
            return Err(auth_error("AUTH_RESPONSE_INVALID"));
        }
        let uri = Url::parse(&device.verification_uri)
            .map_err(|_| auth_error("AUTH_RESPONSE_INVALID"))?;
        if uri.scheme() != "https"
            || !matches!(
                uri.host_str(),
                Some("microsoft.com" | "www.microsoft.com" | "login.microsoftonline.com")
            )
            || !uri.username().is_empty()
            || uri.password().is_some()
            || uri.port_or_known_default() != Some(443)
        {
            return Err(auth_error("AUTH_RESPONSE_INVALID"));
        }
        Ok(device)
    }
    pub fn authenticate_with_device_code(
        &self,
        device_code: &str,
        interval: u64,
        expires_in: u64,
    ) -> crate::Result<MinecraftUser> {
        self.device_login(device_code, interval, expires_in, &LoginControl::default())
    }
    pub fn device_login(
        &self,
        device_code: &str,
        interval: u64,
        expires_in: u64,
        control: &LoginControl,
    ) -> crate::Result<MinecraftUser> {
        let client = http()?;
        let start = Instant::now();
        let expires = Duration::from_secs(expires_in.min(900));
        let mut interval = interval.clamp(5, 900);
        loop {
            control.wait(Duration::from_secs(interval))?;
            if start.elapsed() >= expires {
                return Err(auth_error("AUTH_TIMEOUT"));
            }
            let (status, json) = send(client.post(TOKEN).form(&[
                ("client_id", self.client_id.as_str()),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ]))?;
            control.check()?;
            if successful(status) {
                let token: MicrosoftTokenResponse = decode(json)?;
                validate_initial_token(&token)?;
                return self.complete_login(&token.access_token, token.refresh_token, control);
            }
            match json.get("error").and_then(|v| v.as_str()) {
                Some("authorization_pending") => {}
                Some("slow_down") => interval = interval.saturating_add(5).min(900),
                _ => return Err(service_error("microsoft", status, &json)),
            }
        }
    }
    pub fn refresh_token(&self, refresh: &str) -> crate::Result<MinecraftUser> {
        self.refresh_at(refresh, TOKEN, [XBL, XSTS, MC_LOGIN, PROFILE])
    }
    fn refresh_at(
        &self,
        refresh: &str,
        token_endpoint: &str,
        downstream: [&str; 4],
    ) -> crate::Result<MinecraftUser> {
        let control = LoginControl::default();
        let (status, json) = send(http()?.post(token_endpoint).form(&[
            ("client_id", self.client_id.as_str()),
            ("refresh_token", refresh),
            ("grant_type", "refresh_token"),
            ("scope", SCOPE),
        ]))?;
        if !successful(status) {
            return Err(service_error("microsoft", status, &json));
        }
        let token: MicrosoftTokenResponse = decode(json)?;
        self.complete_login_at(
            &token.access_token,
            token.refresh_token.or_else(|| Some(refresh.into())),
            &control,
            downstream,
        )
    }
    fn complete_login(
        &self,
        ms_token: &str,
        refresh: Option<String>,
        control: &LoginControl,
    ) -> crate::Result<MinecraftUser> {
        self.complete_login_at(ms_token, refresh, control, [XBL, XSTS, MC_LOGIN, PROFILE])
    }
    fn complete_login_at(
        &self,
        ms_token: &str,
        refresh: Option<String>,
        control: &LoginControl,
        endpoints: [&str; 4],
    ) -> crate::Result<MinecraftUser> {
        let client = http()?;
        control.phase("xbox")?;
        let (status,json)=send(client.post(endpoints[0]).json(&serde_json::json!({"Properties":{"AuthMethod":"RPS","SiteName":"user.auth.xboxlive.com","RpsTicket":format!("d={ms_token}")},"RelyingParty":"http://auth.xboxlive.com","TokenType":"JWT"})))?;
        if !successful(status) {
            return Err(service_error("xbl", status, &json));
        }
        let xbl: XblResponse = decode(json)?;
        let hash = xbl
            .display_claims
            .xui
            .first()
            .filter(|x| !x.uhs.is_empty())
            .ok_or_else(|| auth_error("XBOX_PROFILE_REQUIRED"))?
            .uhs
            .clone();
        if xbl.token.is_empty() {
            return Err(auth_error("XBOX_LIVE_ERROR"));
        }
        control.phase("xsts")?;
        let (status,json)=send(client.post(endpoints[1]).json(&serde_json::json!({"Properties":{"SandboxId":"RETAIL","UserTokens":[xbl.token]},"RelyingParty":"rp://api.minecraftservices.com/","TokenType":"JWT"})))?;
        if !successful(status) {
            return Err(service_error("xsts", status, &json));
        }
        let xsts: XblResponse = decode(json)?;
        if xsts.token.is_empty() || !xsts.display_claims.xui.iter().any(|x| x.uhs == hash) {
            return Err(auth_error("XBOX_XSTS_ERROR"));
        }
        control.phase("minecraft")?;
        let (status, json) =
            send(client.post(endpoints[2]).json(
                &serde_json::json!({"identityToken":format!("XBL3.0 x={hash};{}",xsts.token)}),
            ))?;
        if !successful(status) {
            return Err(service_error("minecraft", status, &json));
        }
        let mc: MinecraftLoginResponse = decode(json)?;
        if mc.access_token.is_empty() {
            return Err(auth_error("MINECRAFT_SERVICES_ERROR"));
        }
        control.phase("profile")?;
        let (status, json) = send(client.get(endpoints[3]).bearer_auth(&mc.access_token))?;
        if !successful(status) {
            return Err(service_error("profile", status, &json));
        }
        let profile: MinecraftProfileResponse = decode(json)?;
        if profile.id.len() != 32
            || !profile.id.bytes().all(|c| c.is_ascii_hexdigit())
            || profile.name.is_empty()
            || profile.name.len() > 16
            || !profile
                .name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            return Err(auth_error("MINECRAFT_PROFILE_INVALID"));
        }
        control.check()?;
        let mut user = MinecraftUser::premium(profile.name, profile.id, mc.access_token, refresh);
        user.microsoft_client_id = Some(self.client_id.clone());
        user.access_token_expires_at =
            mc.expires_in
                .filter(|duration| *duration > 0)
                .and_then(|duration| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|now| now.as_secs().saturating_add(duration.min(86400)))
                });
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn safe_errors_distinguish_stages_and_never_echo_responses() {
        let malicious =
            serde_json::json!({"error":"Invalid app registration","access_token":"do-not-print"});
        assert!(
            service_error("minecraft", 403, &malicious)
                .to_string()
                .contains("MINECRAFT_APP_NOT_APPROVED")
        );
        for (stage, status, expected) in [
            ("microsoft", 400, "MICROSOFT_AUTH_ERROR"),
            ("xbl", 401, "XBOX_LIVE_ERROR"),
            ("xsts", 401, "XBOX_XSTS_ERROR"),
            ("profile", 404, "MINECRAFT_PROFILE_REQUIRED"),
        ] {
            let e = service_error(stage, status, &malicious).to_string();
            assert!(e.contains(expected));
            assert!(!e.contains("do-not-print"));
        }
    }
    #[test]
    fn authorization_url_has_matching_redirect_and_no_secret() {
        let auth = MicrosoftAuth::default();
        let url = Url::parse(&auth.authorization_url(
            "challenge",
            "state",
            super::super::oauth::NATIVE_REDIRECT,
        ))
        .unwrap();
        let p: std::collections::HashMap<_, _> = url.query_pairs().collect();
        assert_eq!(p["redirect_uri"], super::super::oauth::NATIVE_REDIRECT);
        assert_eq!(p["code_challenge_method"], "S256");
        assert_eq!(p["scope"], "XboxLive.signin offline_access");
        assert!(!p.contains_key("client_secret"));
    }
    #[test]
    fn real_http_fixture_pipeline_and_missing_claims() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
        };
        for missing in [false, true] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let endpoint = format!("http://{}/", listener.local_addr().unwrap());
            let server = std::thread::spawn(move || {
                let responses = if missing {
                    vec![serde_json::json!({"Token":"xbl","DisplayClaims":{"xui":[]}})]
                } else {
                    vec![
                        serde_json::json!({"Token":"xbl","DisplayClaims":{"xui":[{"uhs":"hash"}]}}),
                        serde_json::json!({"Token":"xsts","DisplayClaims":{"xui":[{"uhs":"hash"}]}}),
                        serde_json::json!({"access_token":"fixture-minecraft"}),
                        serde_json::json!({"id":"0123456789abcdef0123456789abcdef","name":"Fixture"}),
                    ]
                };
                for response in responses {
                    let (mut stream, peer) = listener.accept().unwrap();
                    assert!(peer.ip().is_loopback());
                    stream
                        .set_read_timeout(Some(Duration::from_secs(3)))
                        .unwrap();
                    let mut buf = [0u8; 8192];
                    let _ = stream.read(&mut buf).unwrap();
                    let body = response.to_string();
                    write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
                }
            });
            let result = MicrosoftAuth::default().complete_login_at(
                "fixture-ms",
                Some("fixture-refresh".into()),
                &LoginControl::default(),
                [&endpoint, &endpoint, &endpoint, &endpoint],
            );
            if missing {
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("XBOX_PROFILE_REQUIRED")
                );
            } else {
                let user = result.unwrap();
                assert_eq!(user.username, "Fixture");
                assert_eq!(user.microsoft_client_id.as_deref(), Some(DEFAULT_CLIENT_ID));
                assert!(!serde_json::to_string(&user).unwrap().contains("fixture-"));
            }
            server.join().unwrap();
        }
    }
}

#[cfg(test)]
mod exchange_tests {
    use super::*;
    fn fixture(
        responses: Vec<(u16, serde_json::Value)>,
    ) -> (String, std::thread::JoinHandle<Vec<String>>) {
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}/", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            use std::io::Write;
            let mut requests = Vec::new();
            for (status, json) in responses {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut request = Vec::new();
                let mut buffer = [0u8; 2048];
                loop {
                    let n = stream.read(&mut buffer).unwrap();
                    if n == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..n]);
                    if let Some(end) = request.windows(4).position(|v| v == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&request[..end]);
                        let size = headers
                            .lines()
                            .find_map(|line| {
                                let (key, value) = line.split_once(':')?;
                                key.eq_ignore_ascii_case("content-length")
                                    .then(|| value.trim().parse::<usize>().unwrap())
                            })
                            .unwrap_or(0);
                        if request.len() >= end + 4 + size {
                            break;
                        }
                    }
                    assert!(request.len() < 16384);
                }
                requests.push(String::from_utf8(request).unwrap());
                let body = json.to_string();
                write!(stream,"HTTP/1.1 {status} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
            }
            requests
        });
        (address, server)
    }
    fn pipeline() -> Vec<(u16, serde_json::Value)> {
        vec![
            (
                200,
                serde_json::json!({"Token":"fixture-xbl","DisplayClaims":{"xui":[{"uhs":"hash"}]}}),
            ),
            (
                200,
                serde_json::json!({"Token":"fixture-xsts","DisplayClaims":{"xui":[{"uhs":"hash"}]}}),
            ),
            (
                200,
                serde_json::json!({"access_token":"fixture-minecraft","expires_in":86400}),
            ),
            (
                200,
                serde_json::json!({"id":"0123456789abcdef0123456789abcdef","name":"Fixture"}),
            ),
        ]
    }
    #[test]
    fn authorization_code_exchange_and_refresh_use_rust_forms_and_preserve_rotation() {
        for (refreshing, rotated) in [(false, false), (true, false), (true, true)] {
            let mut responses = vec![(
                200,
                serde_json::json!({"access_token":"fixture-microsoft","refresh_token":if refreshing && !rotated {None}else{Some("new-refresh")}}),
            )];
            responses.extend(pipeline());
            let (url, server) = fixture(responses);
            let auth = MicrosoftAuth::default();
            let user = if refreshing {
                auth.refresh_at("original-refresh", &url, [&url, &url, &url, &url])
                    .unwrap()
            } else {
                auth.exchange_code_at(
                    "one-use-code",
                    "rust-verifier",
                    "http://localhost/",
                    &LoginControl::default(),
                    [&url, &url, &url, &url, &url],
                )
                .unwrap()
            };
            assert!(
                user.cached_session_valid_at(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                )
            );
            assert_eq!(
                user.refresh_token.as_deref(),
                Some(if refreshing && !rotated {
                    "original-refresh"
                } else {
                    "new-refresh"
                })
            );
            let requests = server.join().unwrap();
            let form = requests[0].split("\r\n\r\n").nth(1).unwrap();
            let fields: std::collections::HashMap<_, _> =
                url::form_urlencoded::parse(form.as_bytes()).collect();
            assert!(!fields.contains_key("client_secret"));
            assert_eq!(fields["client_id"], DEFAULT_CLIENT_ID);
            if refreshing {
                assert_eq!(fields["grant_type"], "refresh_token");
            } else {
                assert_eq!(fields["code_verifier"], "rust-verifier");
                assert_eq!(fields["redirect_uri"], "http://localhost/");
                assert_eq!(fields["grant_type"], "authorization_code");
            }
            assert!(!serde_json::to_string(&user).unwrap().contains("fixture-"));
        }
    }
    #[test]
    fn microsoft_errors_and_invalid_app_registration_are_not_generic_or_reflected() {
        let (url, server) = fixture(vec![(
            400,
            serde_json::json!({"error":"consent_required","error_description":"fixture-secret-that-must-not-be-reflected"}),
        )]);
        let result = MicrosoftAuth::default()
            .exchange_code_at(
                "fixture-code",
                "fixture-verifier",
                "http://localhost/",
                &LoginControl::default(),
                [&url, &url, &url, &url, &url],
            )
            .unwrap_err()
            .to_string();
        assert!(result.contains("REAUTHENTICATION_REQUIRED"));
        assert!(!result.contains("fixture-"));
        server.join().unwrap();
        for key in ["error", "errorMessage"] {
            let json = serde_json::json!({key:"Invalid app registration"});
            assert!(
                service_error("minecraft", 403, &json)
                    .to_string()
                    .contains("MINECRAFT_APP_NOT_APPROVED")
            );
        }
        assert!(
            !service_error("minecraft", 403, &serde_json::json!({"error":"other"}))
                .to_string()
                .contains("NOT_APPROVED")
        );
    }
}

#[cfg(test)]
mod consent_tests {
    use super::*;
    #[test]
    fn missing_offline_consent_does_not_create_an_unusable_saved_account() {
        let token = MicrosoftTokenResponse {
            access_token: "fixture-access".into(),
            refresh_token: None,
        };
        assert!(
            validate_initial_token(&token)
                .unwrap_err()
                .to_string()
                .contains("MICROSOFT_CONSENT_REQUIRED")
        );
        let token = MicrosoftTokenResponse {
            access_token: "fixture-access".into(),
            refresh_token: Some("fixture-refresh".into()),
        };
        assert!(validate_initial_token(&token).is_ok());
    }
}
