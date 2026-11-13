// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure OAuth transaction validation, shared by the native window and loopback.
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use url::Url;
use zeroize::Zeroizing;

pub const NATIVE_REDIRECT: &str = "https://login.microsoftonline.com/common/oauth2/nativeclient";
pub const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

pub fn auth_error(code: &str) -> crate::Error {
    crate::Error::AuthError(code.into())
}
pub fn random_secret() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

#[derive(Clone)]
pub struct LoginControl {
    pub cancelled: Arc<AtomicBool>,
    pub deadline: Instant,
    progress: Arc<dyn Fn(&'static str) + Send + Sync>,
}
impl Default for LoginControl {
    fn default() -> Self {
        Self::new(LOGIN_TIMEOUT, |_| {})
    }
}
impl LoginControl {
    pub fn new(timeout: Duration, progress: impl Fn(&'static str) + Send + Sync + 'static) -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            deadline: Instant::now() + timeout,
            progress: Arc::new(progress),
        }
    }
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
    pub fn check(&self) -> crate::Result<()> {
        if self.cancelled.load(Ordering::SeqCst) {
            return Err(auth_error("AUTH_CANCELLED"));
        }
        if Instant::now() >= self.deadline {
            return Err(auth_error("AUTH_TIMEOUT"));
        }
        Ok(())
    }
    pub fn phase(&self, name: &'static str) -> crate::Result<()> {
        self.check()?;
        (self.progress)(name);
        Ok(())
    }
    pub fn wait(&self, duration: Duration) -> crate::Result<()> {
        let end = Instant::now() + duration;
        while Instant::now() < end {
            self.check()?;
            std::thread::sleep(
                Duration::from_millis(100).min(end.saturating_duration_since(Instant::now())),
            );
        }
        self.check()
    }
}

pub struct OAuthTransaction {
    pub verifier: Zeroizing<String>,
    pub challenge: String,
    state: Zeroizing<String>,
    redirect: Url,
    consumed: bool,
    pub control: LoginControl,
}
impl OAuthTransaction {
    pub fn new(redirect: Url, control: LoginControl) -> Self {
        let (verifier, challenge) = super::microsoft::MicrosoftAuth::generate_pkce();
        Self {
            verifier: Zeroizing::new(verifier),
            challenge,
            state: Zeroizing::new(random_secret()),
            redirect,
            consumed: false,
            control,
        }
    }
    pub fn state(&self) -> &str {
        &self.state
    }
    pub fn redirect(&self) -> &str {
        self.redirect.as_str()
    }
    pub fn matches_redirect(&self, url: &Url) -> bool {
        url.scheme() == self.redirect.scheme()
            && url.host_str() == self.redirect.host_str()
            && url.port_or_known_default() == self.redirect.port_or_known_default()
            && url.path() == self.redirect.path()
            && url.username().is_empty()
            && url.password().is_none()
    }
    pub fn consume(&mut self, callback: &Url) -> crate::Result<Zeroizing<String>> {
        self.control.check()?;
        if self.consumed {
            return Err(auth_error("AUTH_CODE_REUSED"));
        }
        if !self.matches_redirect(callback)
            || callback.fragment().is_some()
            || callback.as_str().len() > 8192
        {
            return Err(auth_error("CALLBACK_INVALID"));
        }
        let pairs: Vec<_> = callback.query_pairs().collect();
        let unique = |key: &str| -> crate::Result<Option<String>> {
            let values: Vec<_> = pairs.iter().filter(|(k, _)| k == key).collect();
            if values.len() > 1 {
                return Err(auth_error("CALLBACK_INVALID"));
            }
            Ok(values.first().map(|(_, v)| v.to_string()))
        };
        let returned = unique("state")?.ok_or_else(|| auth_error("STATE_MISMATCH"))?;
        // Random equal-length state, constant-time comparison without early exit.
        let matches = returned.len() == self.state.len()
            && returned
                .bytes()
                .zip(self.state.bytes())
                .fold(0u8, |acc, (a, b)| acc | (a ^ b))
                == 0;
        if !matches {
            return Err(auth_error("STATE_MISMATCH"));
        }
        let code = unique("code")?;
        let error = unique("error")?;
        if code.is_some() && error.is_some() {
            return Err(auth_error("CALLBACK_INVALID"));
        }
        if let Some(error) = error {
            self.consumed = true;
            return Err(auth_error(if error == "access_denied" {
                "AUTH_DECLINED"
            } else {
                "MICROSOFT_AUTH_ERROR"
            }));
        }
        let code = code
            .filter(|c| !c.is_empty() && c.len() <= 4096)
            .ok_or_else(|| auth_error("CALLBACK_INVALID"))?;
        self.consumed = true;
        Ok(Zeroizing::new(code))
    }
}

/// Only these HTTPS origins are allowed inside the unprivileged embedded window.
/// Unsupported/federated flows must move to the system browser, never gain IPC.
pub fn microsoft_navigation(url: &Url) -> bool {
    url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port_or_known_default() == Some(443)
        && matches!(
            url.host_str(),
            Some(
                "login.microsoftonline.com"
                    | "login.live.com"
                    | "account.live.com"
                    | "account.microsoft.com"
                    | "login.microsoft.com"
            )
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tx() -> OAuthTransaction {
        OAuthTransaction::new(
            "http://localhost:3456/".parse().unwrap(),
            LoginControl::default(),
        )
    }
    fn response(t: &OAuthTransaction) -> Url {
        let mut u = Url::parse(t.redirect()).unwrap();
        u.query_pairs_mut()
            .append_pair("state", t.state())
            .append_pair("code", "only-once");
        u
    }
    #[test]
    fn pkce_random_s256_and_rfc7636_vector() {
        use sha2::{Digest, Sha256};
        let (a, b) = super::super::microsoft::MicrosoftAuth::generate_pkce();
        let (other, _) = super::super::microsoft::MicrosoftAuth::generate_pkce();
        assert_ne!(a, other);
        assert_eq!(a.len(), 43);
        assert_eq!(b, URL_SAFE_NO_PAD.encode(Sha256::digest(a.as_bytes())));
        assert_eq!(
            URL_SAFE_NO_PAD.encode(Sha256::digest(
                b"dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"
            )),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }
    #[test]
    fn valid_callback_consumed_once() {
        let mut t = tx();
        let u = response(&t);
        assert_eq!(&**t.consume(&u).unwrap(), "only-once");
        assert!(t.consume(&u).is_err());
    }
    #[test]
    fn malicious_callbacks_do_not_consume_valid_transaction() {
        let mut t = tx();
        let good = response(&t);
        for u in [
            "http://evil.test/",
            "http://localhost:3456/other?state=x&code=y",
            "http://localhost:3456/?code=x",
            "http://localhost:3456/?state=wrong&code=y",
        ] {
            assert!(t.consume(&u.parse().unwrap()).is_err());
        }
        let mut duplicate = good.clone();
        duplicate.query_pairs_mut().append_pair("code", "extra");
        assert!(t.consume(&duplicate).is_err());
        let mut fragment = good.clone();
        fragment.set_fragment(Some("secret"));
        assert!(t.consume(&fragment).is_err());
        assert!(t.consume(&good).is_ok());
    }
    #[test]
    fn cancellation_timeout_and_decline() {
        let mut t = tx();
        let u = response(&t);
        t.control.cancel();
        assert!(
            t.consume(&u)
                .unwrap_err()
                .to_string()
                .contains("AUTH_CANCELLED")
        );
        let mut t = OAuthTransaction::new(
            "http://localhost/".parse().unwrap(),
            LoginControl::new(Duration::ZERO, |_| {}),
        );
        assert!(t.consume(&response(&t)).is_err());
        let mut t = tx();
        let mut u = Url::parse(t.redirect()).unwrap();
        u.query_pairs_mut()
            .append_pair("state", t.state())
            .append_pair("error", "access_denied");
        assert!(
            t.consume(&u)
                .unwrap_err()
                .to_string()
                .contains("AUTH_DECLINED")
        );
    }
    #[test]
    fn navigation_has_no_lookalike_or_local_origin() {
        for u in [
            "https://login.live.com.evil.test/",
            "http://login.live.com/",
            "tauri://localhost/",
            "https://login.live.com:444/",
            "https://attacker@login.live.com/",
        ] {
            assert!(!microsoft_navigation(&u.parse().unwrap()));
        }
        assert!(microsoft_navigation(
            &"https://login.live.com/oauth20_authorize.srf"
                .parse()
                .unwrap()
        ));
    }
}
