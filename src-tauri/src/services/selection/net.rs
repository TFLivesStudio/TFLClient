//! Transporte HTTP de TFL Selection.
//!
//! Detrás de un trait para poder probar catálogos, descargas interrumpidas
//! y caídas de GitHub sin red. La implementación real solo habla HTTPS con
//! hosts de GitHub y rechaza cualquier redirección fuera de esa lista.

use futures::future::BoxFuture;
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    Network(String),
    Status(u16),
    TooLarge,
    UntrustedUrl(String),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "error de red: {e}"),
            Self::Status(s) => write!(f, "GitHub respondió HTTP {s}"),
            Self::TooLarge => write!(f, "la respuesta supera el tamaño permitido"),
            Self::UntrustedUrl(u) => write!(f, "URL no permitida: {u}"),
        }
    }
}

impl std::error::Error for FetchError {}

#[derive(Debug, Clone)]
pub struct Fetched {
    pub status: u16,
    pub body: Vec<u8>,
    pub etag: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadResult {
    /// HTTP final (200 completo, 206 reanudado).
    pub status: u16,
    /// Tamaño total del archivo parcial tras la llamada.
    pub total_len: u64,
}

pub trait Transport: Send + Sync {
    /// GET en memoria con tope de tamaño y ETag condicional opcional.
    fn get<'a>(
        &'a self,
        url: &'a str,
        max_bytes: usize,
        etag: Option<&'a str>,
    ) -> BoxFuture<'a, Result<Fetched, FetchError>>;

    /// Descarga en streaming a `dest`. Con `resume_from > 0` pide `Range`;
    /// si el servidor responde 200 en vez de 206 se reinicia el archivo.
    /// `on_bytes` recibe los bytes nuevos escritos.
    fn download<'a>(
        &'a self,
        url: &'a str,
        dest: &'a Path,
        resume_from: u64,
        max_bytes: u64,
        on_bytes: &'a (dyn Fn(u64) + Send + Sync),
    ) -> BoxFuture<'a, Result<DownloadResult, FetchError>>;
}

/// Hosts a los que el flujo oficial puede conectarse (incluidas las
/// redirecciones legítimas del CDN de assets de GitHub).
const ALLOWED_HOSTS: &[&str] = &[
    "raw.githubusercontent.com",
    "github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
    "github-releases.githubusercontent.com",
];

pub fn url_allowed(url: &reqwest::Url) -> bool {
    url.scheme() == "https"
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
        && url
            .host_str()
            .is_some_and(|h| ALLOWED_HOSTS.contains(&h.to_ascii_lowercase().as_str()))
}

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(crate::core::http_client::USER_AGENT)
        .connect_timeout(Duration::from_secs(15))
        // Sin timeout total: un JAR de ~78 MB en una conexión lenta no debe
        // cortarse a los 30 s. Lo que sí se corta es una conexión muerta.
        .read_timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 5 {
                attempt.error("demasiadas redirecciones")
            } else if url_allowed(attempt.url()) {
                attempt.follow()
            } else {
                let target = attempt.url().to_string();
                attempt.error(format!("redirección a host no permitido: {target}"))
            }
        }))
        .build()
        .expect("no se pudo crear el cliente HTTP de Selection")
});

pub struct GithubTransport;

fn parse_allowed(url: &str) -> Result<reqwest::Url, FetchError> {
    let parsed = reqwest::Url::parse(url).map_err(|_| FetchError::UntrustedUrl(url.into()))?;
    if url_allowed(&parsed) {
        Ok(parsed)
    } else {
        Err(FetchError::UntrustedUrl(url.into()))
    }
}

fn net_err(e: reqwest::Error) -> FetchError {
    FetchError::Network(e.to_string())
}

impl Transport for GithubTransport {
    fn get<'a>(
        &'a self,
        url: &'a str,
        max_bytes: usize,
        etag: Option<&'a str>,
    ) -> BoxFuture<'a, Result<Fetched, FetchError>> {
        Box::pin(async move {
            let url = parse_allowed(url)?;
            let mut req = CLIENT.get(url);
            if let Some(tag) = etag {
                req = req.header(reqwest::header::IF_NONE_MATCH, tag);
            }
            let mut resp = req.send().await.map_err(net_err)?;
            let status = resp.status().as_u16();
            let etag = resp
                .headers()
                .get(reqwest::header::ETAG)
                .and_then(|v| v.to_str().ok())
                .map(String::from);
            if resp.content_length().is_some_and(|l| l > max_bytes as u64) {
                return Err(FetchError::TooLarge);
            }
            let mut body = Vec::new();
            while let Some(chunk) = resp.chunk().await.map_err(net_err)? {
                if body.len() + chunk.len() > max_bytes {
                    return Err(FetchError::TooLarge);
                }
                body.extend_from_slice(&chunk);
            }
            Ok(Fetched { status, body, etag })
        })
    }

    fn download<'a>(
        &'a self,
        url: &'a str,
        dest: &'a Path,
        resume_from: u64,
        max_bytes: u64,
        on_bytes: &'a (dyn Fn(u64) + Send + Sync),
    ) -> BoxFuture<'a, Result<DownloadResult, FetchError>> {
        Box::pin(async move {
            use tokio::io::AsyncWriteExt;
            let url = parse_allowed(url)?;
            let mut req = CLIENT.get(url);
            if resume_from > 0 {
                req = req.header(reqwest::header::RANGE, format!("bytes={resume_from}-"));
            }
            let mut resp = req.send().await.map_err(net_err)?;
            let status = resp.status().as_u16();
            let (mut file, mut written) = match status {
                206 if resume_from > 0 => {
                    let f = tokio::fs::OpenOptions::new()
                        .append(true)
                        .open(dest)
                        .await
                        .map_err(|e| FetchError::Network(e.to_string()))?;
                    (f, resume_from)
                }
                200 => {
                    let f = tokio::fs::File::create(dest)
                        .await
                        .map_err(|e| FetchError::Network(e.to_string()))?;
                    (f, 0)
                }
                other => return Err(FetchError::Status(other)),
            };
            while let Some(chunk) = resp.chunk().await.map_err(net_err)? {
                written += chunk.len() as u64;
                if written > max_bytes {
                    return Err(FetchError::TooLarge);
                }
                file.write_all(&chunk)
                    .await
                    .map_err(|e| FetchError::Network(e.to_string()))?;
                on_bytes(chunk.len() as u64);
            }
            file.flush()
                .await
                .map_err(|e| FetchError::Network(e.to_string()))?;
            Ok(DownloadResult {
                status,
                total_len: written,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_https_github_hosts_are_allowed() {
        for ok in [
            "https://raw.githubusercontent.com/TFLivesStudio/x/main/a.json",
            "https://github.com/o/r/releases/download/t/a.bin",
            "https://objects.githubusercontent.com/github-production-release-asset/1",
            "https://release-assets.githubusercontent.com/x",
        ] {
            assert!(url_allowed(&reqwest::Url::parse(ok).unwrap()), "{ok}");
        }
        for bad in [
            "http://github.com/o/r",
            "https://evil.example/a",
            "https://github.com.evil.example/a",
            "https://user:pw@github.com/a",
            "https://github.com:8443/a",
            "ftp://github.com/a",
            "https://api.github.com/x",
        ] {
            assert!(!url_allowed(&reqwest::Url::parse(bad).unwrap()), "{bad}");
        }
    }
}
