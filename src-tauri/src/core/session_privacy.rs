//! Redact auth material before reports/diagnostics cross the Tauri boundary.
use regex::Regex;
use std::sync::LazyLock;
static JWT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]*").expect("static JWT pattern")
});
static LABELLED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(?P<key>(?:--)?(?:access[_-]?token|refresh[_-]?token|client[_-]?token|client[_-]?secret|device[_-]?code|code[_-]?verifier|authorization|password)["']?\s*(?:[:=]|\s)\s*)(?:Bearer\s+)?(?:"[^"\r\n]*"|'[^'\r\n]*'|[^\s,;}]+)"#).expect("static credential pattern")
});
pub fn redact(text: &str) -> String {
    let jwt = JWT.replace_all(text, "[REDACTED]");
    LABELLED.replace_all(&jwt, "${key}[REDACTED]").into_owned()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crash_command_lines_json_credentials_and_jwts_are_redacted() {
        let input = "--accessToken fixture-access --uuid public-uuid\n{\"refresh_token\":\"fixture-refresh\"}\nAuthorization: Bearer fixture-bearer\nJWT eyJmaXh0dXJl.cGF5bG9hZA.c2lnbmF0dXJl\nOutOfMemoryError: Java heap space";
        let output = redact(input);
        assert!(!output.contains("fixture-access"));
        assert!(!output.contains("fixture-refresh"));
        assert!(!output.contains("fixture-bearer"));
        assert!(!output.contains("eyJ"));
        assert!(output.contains("public-uuid"));
        assert!(output.contains("OutOfMemoryError"));
    }
}
