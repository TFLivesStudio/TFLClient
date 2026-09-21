use std::sync::LazyLock;
use std::time::Duration;

// TODO(TFL): swap the contact URL once the real TFLives domain exists.
pub const USER_AGENT: &str = "TFLClient/0.1.0 (+https://tflives.example)";

pub static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(8)
        .build()
        .expect("Failed to create HTTP client")
});

const MAX_RETRY_ATTEMPTS: u32 = 3;

/// GET con reintento — Modrinth, Forge y NeoForge se pedían con una sola
/// llamada sin reintento en toda esta capa (a diferencia del motor de
/// descarga de `aqua`, que sí reintenta), así que cualquier hipo de red
/// tumbaba la búsqueda/instalación de un mod o loader sin necesidad real.
pub async fn get_text_retrying(url: &str) -> Result<String, String> {
    let mut last_err = String::new();
    for attempt in 1..=MAX_RETRY_ATTEMPTS {
        match HTTP.get(url).send().await {
            Ok(resp) => match resp.error_for_status() {
                Ok(resp) => match resp.text().await {
                    Ok(text) => return Ok(text),
                    Err(e) => last_err = format!("error leyendo respuesta: {e}"),
                },
                Err(e) => return Err(format!("{url}: {e}")),
            },
            Err(e) => last_err = format!("error de red: {e}"),
        }
        tracing::warn!("Fallo intento {attempt}/{MAX_RETRY_ATTEMPTS} pidiendo {url}: {last_err}");
        if attempt < MAX_RETRY_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(200 * (1 << (attempt - 1)))).await;
        }
    }
    Err(format!(
        "No se pudo obtener {url} tras {MAX_RETRY_ATTEMPTS} intentos: {last_err}"
    ))
}

pub async fn get_json_retrying<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, String> {
    let text = get_text_retrying(url).await?;
    serde_json::from_str(&text).map_err(|e| format!("{url}: parseo JSON: {e}"))
}

/// POST con reintento y body JSON — usado para las consultas por lote de
/// Modrinth (resolver mods instalados por hash, chequear actualizaciones),
/// que solo tienen forma de endpoint POST (mandan una lista de hashes en
/// el body, no caben en query string).
pub async fn post_json_retrying<B: serde::Serialize, T: serde::de::DeserializeOwned>(
    url: &str,
    body: &B,
) -> Result<T, String> {
    let mut last_err = String::new();
    for attempt in 1..=MAX_RETRY_ATTEMPTS {
        match HTTP.post(url).json(body).send().await {
            Ok(resp) => match resp.error_for_status() {
                Ok(resp) => match resp.json::<T>().await {
                    Ok(parsed) => return Ok(parsed),
                    Err(e) => last_err = format!("error leyendo respuesta: {e}"),
                },
                Err(e) => return Err(format!("{url}: {e}")),
            },
            Err(e) => last_err = format!("error de red: {e}"),
        }
        tracing::warn!("Fallo intento {attempt}/{MAX_RETRY_ATTEMPTS} pidiendo {url}: {last_err}");
        if attempt < MAX_RETRY_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(200 * (1 << (attempt - 1)))).await;
        }
    }
    Err(format!(
        "No se pudo obtener {url} tras {MAX_RETRY_ATTEMPTS} intentos: {last_err}"
    ))
}

pub async fn get_bytes_retrying(url: &str) -> Result<Vec<u8>, String> {
    let mut last_err = String::new();
    for attempt in 1..=MAX_RETRY_ATTEMPTS {
        match HTTP.get(url).send().await {
            Ok(resp) => match resp.error_for_status() {
                Ok(resp) => match resp.bytes().await {
                    Ok(bytes) => return Ok(bytes.to_vec()),
                    Err(e) => last_err = format!("error leyendo respuesta: {e}"),
                },
                Err(e) => return Err(format!("{url}: {e}")),
            },
            Err(e) => last_err = format!("error de red: {e}"),
        }
        tracing::warn!("Fallo intento {attempt}/{MAX_RETRY_ATTEMPTS} bajando {url}: {last_err}");
        if attempt < MAX_RETRY_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(200 * (1 << (attempt - 1)))).await;
        }
    }
    Err(format!(
        "No se pudo descargar {url} tras {MAX_RETRY_ATTEMPTS} intentos: {last_err}"
    ))
}
