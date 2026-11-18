//! Skin y capa de la cuenta — API oficial de Mojang
//! (`api.minecraftservices.com`), la misma que usa el launcher oficial.
//! Solo funciona con cuentas Microsoft (necesita el access_token real de
//! Minecraft Services); cuentas offline/Yggdrasil no tienen perfil ahí.
use crate::core::HTTP;
use crate::services::mojang_auth::active_user_fresh;
use base64::Engine;
use launchwerk::auth::AccountType;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    sync::LazyLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::command;
use tracing::warn;

const API_BASE: &str = "https://api.minecraftservices.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MojangSkin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MojangCape {
    pub id: String,
    pub state: String,
    pub url: String,
    #[serde(default)]
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MojangProfile {
    pub skins: Vec<MojangSkin>,
    pub capes: Vec<MojangCape>,
}

async fn require_microsoft_token() -> Result<String, String> {
    let user = active_user_fresh().await?;
    if user.user_type != AccountType::Microsoft {
        return Err("Esto necesita una cuenta Microsoft — las cuentas offline no tienen skin ni capas propias en Mojang.".into());
    }
    // Si cargar o refrescar el token falló silenciosamente (sin internet,
    // storage corrupto, etc.) `access_token` puede quedar vacío — mandarlo
    // igual como Bearer da un 401 genérico de Mojang que no dice nada de
    // por qué. Mejor cortar acá con un mensaje que sí explica la causa real.
    if user.access_token.trim().is_empty() {
        return Err(
            "No se pudo obtener una sesión de Microsoft válida — revisa tu conexión a internet o cierra sesión y vuelve a entrar.".into(),
        );
    }
    Ok(user.access_token)
}

fn map_status(status: reqwest::StatusCode) -> String {
    match status.as_u16() {
        401 | 403 => {
            "La sesión de Microsoft no es válida — prueba cerrar sesión y volver a entrar.".into()
        }
        429 => "Mojang está limitando pedidos por ahora — prueba de nuevo en un minuto.".into(),
        _ => format!("Mojang devolvió un error ({status})"),
    }
}

#[command]
pub async fn get_mojang_profile() -> Result<MojangProfile, String> {
    let token = require_microsoft_token().await?;
    let resp = HTTP
        .get(format!("{API_BASE}/minecraft/profile"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| {
            warn!("get_mojang_profile: fallo de red pidiendo el perfil: {e}");
            format!("No se pudo conectar con Mojang: {e}")
        })?;
    if !resp.status().is_success() {
        let status = resp.status();
        warn!("get_mojang_profile: Mojang devolvió {status}");
        return Err(map_status(status));
    }
    let raw = resp.text().await.map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
        warn!("get_mojang_profile: respuesta no es JSON válido");
        format!("Mojang devolvió una respuesta inesperada: {e}")
    })?;
    let mut profile = MojangProfile {
        skins: serde_json::from_value(json["skins"].clone()).unwrap_or_default(),
        capes: serde_json::from_value(json["capes"].clone()).unwrap_or_default(),
    };
    for skin in &mut profile.skins {
        skin.url = https_url(&skin.url);
    }
    for cape in &mut profile.capes {
        cape.url = https_url(&cape.url);
    }
    // Only the UUID returned by Mojang can seed its own cosmetic cache. An
    // account switch during this request must never attribute this skin to
    // the newly selected account. No credential is read by the cache writer.
    if let Some(id) = json["id"].as_str().and_then(normalize_uuid) {
        let user = crate::services::SettingsManager::read()
            .user
            .iter()
            .find(|user| {
                user.user_type == AccountType::Microsoft
                    && normalize_uuid(&user.uuid).as_deref() == Some(id.as_str())
            })
            .cloned();
        if let Some(user) = user {
            let _guard = CHARACTER_CACHE_LOCK.lock().await;
            if let Ok(cached) = cache_textures(&profile).await {
                let _ = write_character_cache(&cache_key(&user), &cached).await;
            }
        }
    }
    Ok(profile)
}

// Cosmetic lookup is deliberately independent of authenticated profile editing.
// A cached skin is never evidence of a valid session or Minecraft ownership.
static CHARACTER_CACHE_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));
static COSMETIC_HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(6))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("TFLClient cosmetic profile cache")
        .build()
        .expect("cosmetic HTTP client")
});
const PROFILE_LIMIT: usize = 64 * 1024;
const TEXTURE_LIMIT: usize = 512 * 1024;
const CACHE_LIMIT: u64 = 2 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct CachedCharacter {
    schema: u32,
    fetched_at: u64,
    profile: MojangProfile,
}
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
fn normalize_uuid(value: &str) -> Option<String> {
    uuid::Uuid::parse_str(value)
        .ok()
        .map(|id| id.simple().to_string())
}
fn cache_key(user: &launchwerk::auth::MinecraftUser) -> String {
    let identity = if user.user_type == AccountType::Cracked {
        format!(
            "offline:{}:{}",
            user.uuid,
            user.username.to_ascii_lowercase()
        )
    } else {
        format!(
            "{:?}:{}:{}",
            user.user_type,
            user.uuid,
            user.yggdrasil_server_url.as_deref().unwrap_or("")
        )
    };
    hex::encode(Sha256::digest(identity.as_bytes()))
}
fn cache_dir() -> PathBuf {
    crate::core::PathManager::get()
        .get_shared_dir()
        .join("character-cache-v1")
}
async fn read_character_cache(key: &str) -> Option<CachedCharacter> {
    let path = cache_dir().join(format!("{key}.json"));
    let metadata = tokio::fs::symlink_metadata(&path).await.ok()?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > CACHE_LIMIT {
        return None;
    }
    let cached: CachedCharacter =
        serde_json::from_slice(&tokio::fs::read(path).await.ok()?).ok()?;
    if cached.schema != 1 || !valid_cached_profile(&cached.profile) {
        return None;
    }
    Some(cached)
}
fn valid_cached_profile(profile: &MojangProfile) -> bool {
    if profile.skins.len() > 1 || profile.capes.len() > 1 {
        return false;
    }
    profile
        .skins
        .iter()
        .all(|s| valid_data_png(&s.url, false) && matches!(s.variant.as_str(), "CLASSIC" | "SLIM"))
        && profile.capes.iter().all(|c| valid_data_png(&c.url, true))
}
fn valid_png(bytes: &[u8], cape: bool) -> bool {
    if bytes.len() < 33
        || bytes.len() > TEXTURE_LIMIT
        || &bytes[..8] != b"\x89PNG\r\n\x1a\n"
        || &bytes[12..16] != b"IHDR"
    {
        return false;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    width == 64 && (height == 32 || (!cape && height == 64))
}
fn valid_data_png(value: &str, cape: bool) -> bool {
    let Some(raw) = value.strip_prefix("data:image/png;base64,") else {
        return false;
    };
    if raw.len() > TEXTURE_LIMIT * 4 / 3 + 4 {
        return false;
    }
    base64::engine::general_purpose::STANDARD
        .decode(raw)
        .ok()
        .is_some_and(|b| valid_png(&b, cape))
}
async fn write_character_cache(key: &str, profile: &MojangProfile) -> Result<(), String> {
    if !valid_cached_profile(profile) {
        return Err("Invalid cosmetic cache".into());
    }
    let directory = cache_dir();
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|_| "Cosmetic cache unavailable")?;
    if tokio::fs::symlink_metadata(&directory)
        .await
        .map_err(|_| "Cosmetic cache unavailable")?
        .file_type()
        .is_symlink()
    {
        return Err("Cosmetic cache cannot be a link".into());
    }
    let payload = serde_json::to_vec(&CachedCharacter {
        schema: 1,
        fetched_at: now_secs(),
        profile: profile.clone(),
    })
    .map_err(|_| "Invalid cosmetic cache")?;
    let destination = directory.join(format!("{key}.json"));
    let temporary = directory.join(format!("{key}.{}.tmp", uuid::Uuid::new_v4()));
    tokio::fs::write(&temporary, payload)
        .await
        .map_err(|_| "Cosmetic cache unavailable")?;
    if tokio::fs::rename(&temporary, &destination).await.is_err() {
        let _ = tokio::fs::remove_file(&temporary).await;
        return Err("Cosmetic cache unavailable".into());
    }
    // Bound only files that this cache owns; no user skins/closet are touched.
    let mut entries = tokio::fs::read_dir(&directory)
        .await
        .map_err(|_| "Cosmetic cache unavailable")?;
    let mut managed = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().into_owned();
        let stem = name.strip_suffix(".json").unwrap_or("");
        if stem.len() != 64 || !stem.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        if let Ok(meta) = entry.metadata().await {
            if meta.is_file() {
                managed.push((meta.modified().unwrap_or(UNIX_EPOCH), entry.path()));
            }
        }
    }
    managed.sort_by_key(|(time, _)| *time);
    let surplus = managed.len().saturating_sub(64);
    for (_, path) in managed.into_iter().take(surplus) {
        if path != destination {
            let _ = tokio::fs::remove_file(path).await;
        }
    }
    Ok(())
}
async fn bounded_get(url: &str, limit: usize) -> Result<Vec<u8>, String> {
    let mut response = COSMETIC_HTTP
        .get(url)
        .send()
        .await
        .map_err(|_| "Cosmetic service unavailable")?;
    if !response.status().is_success()
        || response
            .content_length()
            .is_some_and(|length| length > limit as u64)
    {
        return Err("Cosmetic service unavailable".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "Cosmetic service unavailable")?
    {
        if bytes.len().saturating_add(chunk.len()) > limit {
            return Err("Cosmetic response too large".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
fn texture_url(value: &str) -> Option<String> {
    let url = url::Url::parse(&https_url(value)).ok()?;
    let hash = url.path().strip_prefix("/texture/")?;
    (url.scheme() == "https"
        && url.host_str() == Some("textures.minecraft.net")
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
        && url.query().is_none()
        && url.fragment().is_none()
        && hash.len() == 64
        && hash.bytes().all(|b| b.is_ascii_hexdigit()))
    .then(|| url.to_string())
}
async fn texture_data(value: &str, cape: bool) -> Result<String, String> {
    let url = texture_url(value).ok_or("Invalid cosmetic texture")?;
    let bytes = bounded_get(&url, TEXTURE_LIMIT).await?;
    if !valid_png(&bytes, cape) {
        return Err("Invalid cosmetic texture".into());
    }
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}
async fn cache_textures(profile: &MojangProfile) -> Result<MojangProfile, String> {
    let skin = profile
        .skins
        .iter()
        .find(|s| s.state == "ACTIVE")
        .ok_or("No skin available")?;
    let skin = MojangSkin {
        id: skin.id.clone(),
        state: "ACTIVE".into(),
        url: texture_data(&skin.url, false).await?,
        variant: if skin.variant.eq_ignore_ascii_case("slim") {
            "SLIM"
        } else {
            "CLASSIC"
        }
        .into(),
    };
    let cape = match profile.capes.iter().find(|c| c.state == "ACTIVE") {
        Some(cape) => texture_data(&cape.url, true)
            .await
            .ok()
            .map(|url| MojangCape {
                id: cape.id.clone(),
                state: "ACTIVE".into(),
                url,
                alias: cape.alias.clone(),
            }),
        None => None,
    };
    Ok(MojangProfile {
        skins: vec![skin],
        capes: cape.into_iter().collect(),
    })
}
fn public_profile(raw: &[u8], expected_uuid: &str) -> Result<MojangProfile, String> {
    let json: serde_json::Value =
        serde_json::from_slice(raw).map_err(|_| "Invalid cosmetic profile")?;
    if json["id"].as_str().and_then(normalize_uuid).as_deref() != Some(expected_uuid) {
        return Err("Cosmetic profile identity mismatch".into());
    }
    let property = json["properties"]
        .as_array()
        .and_then(|p| p.iter().find(|p| p["name"] == "textures"))
        .and_then(|p| p["value"].as_str())
        .ok_or("No cosmetic textures")?;
    if property.len() > PROFILE_LIMIT {
        return Err("Cosmetic response too large".into());
    }
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(property)
        .map_err(|_| "Invalid cosmetic textures")?;
    let textures: serde_json::Value =
        serde_json::from_slice(&decoded).map_err(|_| "Invalid cosmetic textures")?;
    let skin = &textures["textures"]["SKIN"];
    let skin_url = skin["url"]
        .as_str()
        .and_then(texture_url)
        .ok_or("No cosmetic skin")?;
    let variant = if skin["metadata"]["model"] == "slim" {
        "SLIM"
    } else {
        "CLASSIC"
    };
    let cape = textures["textures"]["CAPE"]["url"]
        .as_str()
        .and_then(texture_url)
        .map(|url| MojangCape {
            id: "public-cape".into(),
            state: "ACTIVE".into(),
            url,
            alias: None,
        });
    Ok(MojangProfile {
        skins: vec![MojangSkin {
            id: expected_uuid.into(),
            state: "ACTIVE".into(),
            url: skin_url,
            variant: variant.into(),
        }],
        capes: cape.into_iter().collect(),
    })
}
async fn lookup_character(user: &launchwerk::auth::MinecraftUser) -> Result<MojangProfile, String> {
    let uuid = if user.user_type == AccountType::Cracked {
        let name = &user.username;
        if !(3..=16).contains(&name.len())
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err("No public profile for this name".into());
        }
        let bytes = bounded_get(
            &format!("https://api.mojang.com/users/profiles/minecraft/{name}"),
            PROFILE_LIMIT,
        )
        .await?;
        let json: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| "Invalid public profile")?;
        json["id"]
            .as_str()
            .and_then(normalize_uuid)
            .ok_or("Invalid public profile")?
    } else if user.user_type == AccountType::Microsoft {
        normalize_uuid(&user.uuid).ok_or("Invalid profile identity")?
    } else {
        // A custom Yggdrasil identity must never be mistaken for an official UUID.
        // Its last successfully obtained appearance can still be displayed.
        return Err("No supported public cosmetic provider".into());
    };
    let bytes = bounded_get(
        &format!("https://sessionserver.mojang.com/session/minecraft/profile/{uuid}"),
        PROFILE_LIMIT,
    )
    .await?;
    cache_textures(&public_profile(&bytes, &uuid)?).await
}

#[command]
pub async fn get_character_profile(account_uuid: String) -> Result<MojangProfile, String> {
    let user = crate::services::SettingsManager::read()
        .user
        .iter()
        .find(|u| u.uuid == account_uuid)
        .cloned()
        .ok_or("La cuenta ya no está disponible")?;
    let key = cache_key(&user);
    let _guard = CHARACTER_CACHE_LOCK.lock().await;
    let cached = read_character_cache(&key).await;
    let lifetime = if cached.as_ref().is_some_and(|c| c.profile.skins.is_empty()) {
        60
    } else {
        600
    };
    if let Some(cache) = &cached {
        if now_secs().saturating_sub(cache.fetched_at) < lifetime {
            return Ok(cache.profile.clone());
        }
    }
    match lookup_character(&user).await {
        Ok(profile) => {
            let _ = write_character_cache(&key, &profile).await;
            Ok(profile)
        }
        Err(_) => {
            if let Some(cache) = cached {
                return Ok(cache.profile);
            }
            let fallback = MojangProfile::default();
            let _ = write_character_cache(&key, &fallback).await;
            Ok(fallback)
        }
    }
}

/// Mojang devuelve las texturas como `http://textures.minecraft.net/...`. La
/// CSP de producción solo permite imágenes por https, así que sin esto el
/// visor 3D y las miniaturas de capas no cargan. El host sirve https igual.
fn https_url(url: &str) -> String {
    match url.strip_prefix("http://") {
        Some(rest) => format!("https://{rest}"),
        None => url.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texturas_de_mojang_pasan_a_https() {
        assert_eq!(
            https_url("http://textures.minecraft.net/texture/abc"),
            "https://textures.minecraft.net/texture/abc"
        );
        assert_eq!(
            https_url("https://textures.minecraft.net/texture/abc"),
            "https://textures.minecraft.net/texture/abc"
        );
    }

    #[test]
    fn cosmetic_lookup_accepts_only_the_official_texture_host_and_hash() {
        let hash = "a".repeat(64);
        assert_eq!(
            texture_url(&format!("http://textures.minecraft.net/texture/{hash}")),
            Some(format!("https://textures.minecraft.net/texture/{hash}"))
        );
        for value in [
            format!("https://textures.minecraft.net.evil.example/texture/{hash}"),
            format!("https://textures.minecraft.net@127.0.0.1/texture/{hash}"),
            format!("https://textures.minecraft.net/texture/{hash}?token=secret"),
            format!("https://textures.minecraft.net:8080/texture/{hash}"),
            "file:///etc/passwd".into(),
            "data:image/svg+xml,<svg onload='bad()'/>".into(),
        ] {
            assert!(texture_url(&value).is_none(), "must reject {value}");
        }
    }

    #[test]
    fn cosmetic_profile_verifies_identity_and_preserves_slim_and_external_cape() {
        let id = "069a79f444e94726a5befca90e38aaf5";
        let textures = serde_json::json!({ "textures": {
            "SKIN": {"url": format!("https://textures.minecraft.net/texture/{}", "a".repeat(64)), "metadata": {"model": "slim"}},
            "CAPE": {"url": format!("https://textures.minecraft.net/texture/{}", "b".repeat(64))}
        }});
        let document = serde_json::json!({"id": id, "properties": [{"name": "textures", "value": base64::engine::general_purpose::STANDARD.encode(serde_json::to_vec(&textures).unwrap())}]});
        let bytes = serde_json::to_vec(&document).unwrap();
        let profile = public_profile(&bytes, id).unwrap();
        assert_eq!(profile.skins[0].variant, "SLIM");
        assert_eq!(profile.capes.len(), 1);
        assert!(public_profile(&bytes, "c06f89064c8a49119c29ea1dbd1aab82").is_err());
    }

    #[test]
    fn cosmetic_cache_contains_original_png_pixels_and_never_tokens() {
        let original = include_bytes!("../../../static/skins/steve.png");
        assert!(valid_png(original, false));
        let data = format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(original)
        );
        assert!(valid_data_png(&data, false));
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(data.strip_prefix("data:image/png;base64,").unwrap())
                .unwrap(),
            original
        );
        assert!(!valid_data_png("data:image/svg+xml,<svg/>", false));
        assert!(!valid_png(b"not a texture", false));
        let profile = MojangProfile {
            skins: vec![MojangSkin {
                id: "skin".into(),
                state: "ACTIVE".into(),
                url: data,
                variant: "CLASSIC".into(),
            }],
            capes: vec![],
        };
        assert!(valid_cached_profile(&profile));
        let payload = serde_json::to_string(&CachedCharacter {
            schema: 1,
            fetched_at: 1,
            profile,
        })
        .unwrap();
        assert!(!payload.contains("access_token"));
        assert!(!payload.contains("refresh_token"));
    }

    #[test]
    fn cosmetic_identity_is_separate_from_authentication_and_other_accounts() {
        let mut user = launchwerk::auth::MinecraftUser::cracked("Example");
        let offline = cache_key(&user);
        user.username = "OtherName".into();
        assert_ne!(offline, cache_key(&user));
        user.user_type = AccountType::Microsoft;
        let official = cache_key(&user);
        user.username = "RenamedOfficial".into();
        user.access_token = "expired-secret".into();
        user.access_token_expires_at = Some(0);
        assert_eq!(
            official,
            cache_key(&user),
            "official cosmetic identity is UUID, not session validity or name"
        );
        assert_ne!(offline, official);
    }
}

/// `variant` es "classic" o "slim" (brazos finos tipo Alex). El archivo se
/// sube tal cual desde el picker genérico de imágenes ya existente
/// (`pick_image_file`) — si no es un PNG válido de 64x64/64x32, Mojang lo
/// rechaza con un mensaje propio, no hace falta validarlo acá de nuevo.
#[command]
pub async fn set_skin_from_file(path: String, variant: String) -> Result<(), String> {
    let token = require_microsoft_token().await?;
    let bytes = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
    let filename = std::path::Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "skin.png".into());

    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name(filename)
        .mime_str("image/png")
        .map_err(|e| e.to_string())?;
    let form = reqwest::multipart::Form::new()
        .text("variant", variant)
        .part("file", part);

    let resp = HTTP
        .post(format!("{API_BASE}/minecraft/profile/skins"))
        .bearer_auth(token)
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(map_status(resp.status()));
    }
    Ok(())
}

#[command]
pub async fn reset_skin() -> Result<(), String> {
    let token = require_microsoft_token().await?;
    let resp = HTTP
        .delete(format!("{API_BASE}/minecraft/profile/skins/active"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(map_status(resp.status()));
    }
    Ok(())
}

#[command]
pub async fn set_active_cape(cape_id: String) -> Result<(), String> {
    let token = require_microsoft_token().await?;
    let resp = HTTP
        .put(format!("{API_BASE}/minecraft/profile/capes/active"))
        .bearer_auth(token)
        .json(&serde_json::json!({ "capeId": cape_id }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(map_status(resp.status()));
    }
    Ok(())
}

#[command]
pub async fn hide_cape() -> Result<(), String> {
    let token = require_microsoft_token().await?;
    let resp = HTTP
        .delete(format!("{API_BASE}/minecraft/profile/capes/active"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(map_status(resp.status()));
    }
    Ok(())
}
