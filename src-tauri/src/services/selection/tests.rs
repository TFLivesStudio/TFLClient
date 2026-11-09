//! Pruebas del catálogo firmado: firma, manipulación, replay, offline,
//! descubrimiento dinámico y compatibilidad. Usan el catálogo REAL
//! publicado como fixture y catálogos sintéticos firmados con una clave de
//! test inyectada solo en tests.

use super::catalog::*;
use super::net::*;
use super::signatures::TrustedKey;
use base64::Engine;
use futures::future::BoxFuture;
use ring::signature::{Ed25519KeyPair, KeyPair};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const REAL_CATALOG: &[u8] = include_bytes!("testdata/catalog.json");
pub const REAL_CATALOG_SIG: &[u8] = include_bytes!("testdata/catalog.json.sig");

pub fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("tfl-sel-{tag}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Transporte falso con respuestas programadas por URL.
#[derive(Default)]
pub struct MockTransport {
    pub responses: Mutex<std::collections::HashMap<String, Result<Fetched, FetchError>>>,
    pub calls: Mutex<Vec<(String, Option<String>)>>,
}

impl MockTransport {
    pub fn set(&self, url: &str, r: Result<Fetched, FetchError>) {
        self.responses.lock().unwrap().insert(url.to_string(), r);
    }
    pub fn serve_catalog(&self, raw: &[u8], sig: &[u8], etag: Option<&str>) {
        self.set(
            CATALOG_URL,
            Ok(Fetched {
                status: 200,
                body: raw.to_vec(),
                etag: etag.map(String::from),
            }),
        );
        self.set(
            CATALOG_SIG_URL,
            Ok(Fetched {
                status: 200,
                body: sig.to_vec(),
                etag: None,
            }),
        );
    }
    pub fn calls_to(&self, url: &str) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.0 == url)
            .count()
    }
}

impl Transport for MockTransport {
    fn get<'a>(
        &'a self,
        url: &'a str,
        _max: usize,
        etag: Option<&'a str>,
    ) -> BoxFuture<'a, Result<Fetched, FetchError>> {
        Box::pin(async move {
            self.calls
                .lock()
                .unwrap()
                .push((url.to_string(), etag.map(String::from)));
            self.responses
                .lock()
                .unwrap()
                .get(url)
                .cloned()
                .unwrap_or(Err(FetchError::Network("sin respuesta programada".into())))
        })
    }
    fn download<'a>(
        &'a self,
        _url: &'a str,
        _dest: &'a Path,
        _resume_from: u64,
        _max: u64,
        _on: &'a (dyn Fn(u64) + Send + Sync),
    ) -> BoxFuture<'a, Result<DownloadResult, FetchError>> {
        Box::pin(async { Err(FetchError::Network("no aplica".into())) })
    }
}

/// Clave de test + helper para firmar catálogos sintéticos.
pub struct TestSigner {
    pair: Ed25519KeyPair,
    pub keys: &'static [TrustedKey],
}

impl TestSigner {
    pub fn new() -> Self {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let public: &'static str = Box::leak(
            base64::engine::general_purpose::STANDARD
                .encode(pair.public_key().as_ref())
                .into_boxed_str(),
        );
        let keys: &'static [TrustedKey] = Box::leak(Box::new([TrustedKey {
            id: "test-key",
            public_key_b64: public,
        }]));
        Self { pair, keys }
    }
    pub fn sign(&self, raw: &[u8]) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .encode(self.pair.sign(raw).as_ref())
            .into_bytes()
    }
}

pub fn synthetic_catalog(generation: u64, extra_targets: &str) -> Vec<u8> {
    format!(
        r#"{{"schemaVersion":2,"packId":"tfl-selection","distributionRepository":"TFLivesStudio/TFLClientSelection","generation":{generation},"targets":[
{{"targetId":"mc-1.8.9-ornithe","minecraftVersion":"1.8.9","loader":"ornithe-gen2","channels":{{"beta":{{"release":"0.1.0-preview.1","tag":"selection-mc-1.8.9-ornithe-v0.1.0-preview.1","manifestSha256":"f0806d39879c5f625634d27530f2577bcc2e15cfc10d639d5ba202d031a6e155"}}}},"displayName":"TFL Selection 1.8.9","sortOrder":1}}{extra_targets}]}}"#
    )
    .into_bytes()
}

// ── Catálogo real publicado ─────────────────────────────────────────────

#[test]
fn real_catalog_parses_and_matches_the_documented_state() {
    let c = verify_and_parse(REAL_CATALOG, REAL_CATALOG_SIG).expect("catálogo real");
    assert_eq!(c.generation, 1);
    assert_eq!(c.targets.len(), 2);
    let t = &c.targets[0];
    assert_eq!(t.target_id, "mc-1.8.9-ornithe");
    assert_eq!(t.minecraft_version, "1.8.9");
    assert_eq!(t.loader, "ornithe-gen2");
    let beta = t.channels.beta.as_ref().unwrap();
    assert_eq!(beta.release, "0.1.0-preview.1");
    assert_eq!(
        beta.manifest_sha256,
        "f0806d39879c5f625634d27530f2577bcc2e15cfc10d639d5ba202d031a6e155"
    );
    assert!(t.channels.stable.is_none() && t.channels.dev.is_none());
}

#[test]
fn only_targets_with_a_published_release_are_visible() {
    let c = verify_and_parse(REAL_CATALOG, REAL_CATALOG_SIG).unwrap();
    let visible = visible_targets(&c, false);
    // 26.3 existe en el catálogo pero NO tiene canal publicado.
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].minecraft_version, "1.8.9");
    assert_eq!(visible[0].channels.len(), 1);
    assert_eq!(visible[0].channels[0].channel, Channel::Beta);
    assert_eq!(default_channel(&visible[0]), Some(Channel::Beta));
    assert_eq!(visible[0].support, Support::Supported);
}

#[test]
fn modified_catalog_is_rejected() {
    let mut tampered = REAL_CATALOG.to_vec();
    let pos = tampered.windows(5).position(|w| w == b"beta\"").unwrap();
    tampered[pos..pos + 4].copy_from_slice(b"stab");
    assert!(matches!(
        verify_and_parse(&tampered, REAL_CATALOG_SIG),
        Err(CatalogError::Signature(_))
    ));
}

// ── Esquema estricto ────────────────────────────────────────────────────

#[test]
fn schema_is_strict() {
    let ok = synthetic_catalog(5, "");
    assert!(parse_catalog(&ok).is_ok());

    let cases: Vec<(&str, String)> = vec![
        (
            "campo extra en raíz",
            String::from_utf8(ok.clone()).unwrap().replacen(
                "\"generation\"",
                "\"evil\":1,\"generation\"",
                1,
            ),
        ),
        (
            "otro repositorio",
            String::from_utf8(ok.clone())
                .unwrap()
                .replace("TFLivesStudio/TFLClientSelection", "attacker/Repo"),
        ),
        (
            "generation 0",
            String::from_utf8(ok.clone())
                .unwrap()
                .replace("\"generation\":5", "\"generation\":0"),
        ),
        (
            "tag de otro target",
            String::from_utf8(ok.clone()).unwrap().replace(
                "\"tag\":\"selection-mc-1.8.9-ornithe-v0.1.0-preview.1\"",
                "\"tag\":\"selection-mc-26.3-fabric-v0.1.0-preview.1\"",
            ),
        ),
        (
            "sha256 con mayúsculas",
            String::from_utf8(ok.clone())
                .unwrap()
                .replace("f0806d39", "F0806D39"),
        ),
        (
            "targetId con path traversal",
            String::from_utf8(ok.clone()).unwrap().replace(
                "mc-1.8.9-ornithe\",\"minecraftVersion",
                "mc-../../x\",\"minecraftVersion",
            ),
        ),
        (
            "release no semver",
            String::from_utf8(ok.clone()).unwrap().replacen(
                "0.1.0-preview.1\",\"tag",
                "latest\",\"tag",
                1,
            ),
        ),
    ];
    for (name, bad) in cases {
        assert!(
            parse_catalog(bad.as_bytes()).is_err(),
            "debía fallar: {name}"
        );
    }
    assert!(matches!(
        parse_catalog(br#"{"schemaVersion":3,"targets":[]}"#),
        Err(CatalogError::UnknownSchema(3))
    ));
    assert!(parse_catalog(b"no es json").is_err());
}

#[test]
fn duplicate_target_ids_are_rejected() {
    let dup = synthetic_catalog(
        2,
        r#",{"targetId":"mc-1.8.9-ornithe","minecraftVersion":"1.8.9","loader":"fabric","channels":{}}"#,
    );
    assert!(
        matches!(parse_catalog(&dup), Err(CatalogError::Invalid(m)) if m.contains("duplicado"))
    );
}

// ── Descubrimiento dinámico y compatibilidad ────────────────────────────

#[test]
fn new_versions_appear_from_the_signed_catalog_without_code_changes() {
    let extra = r#",{"targetId":"mc-1.21.4-fabric","minecraftVersion":"1.21.4","loader":"fabric","channels":{"stable":{"release":"1.0.0","tag":"selection-mc-1.21.4-fabric-v1.0.0","manifestSha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}},"displayName":"Selection 1.21.4","sortOrder":2}"#;
    let signer = TestSigner::new();
    let raw = synthetic_catalog(2, extra);
    let sig = signer.sign(&raw);
    let c = verify_and_parse_with(signer.keys, &raw, &sig).unwrap();
    let visible = visible_targets(&c, false);
    assert_eq!(
        visible
            .iter()
            .map(|t| t.minecraft_version.as_str())
            .collect::<Vec<_>>(),
        vec!["1.8.9", "1.21.4"]
    );
    assert_eq!(visible[1].display_name, "Selection 1.21.4");
    assert_eq!(default_channel(&visible[1]), Some(Channel::Stable));
}

#[test]
fn unknown_loader_is_flagged_as_launcher_update_required() {
    let extra = r#",{"targetId":"mc-1.21.4-quilt","minecraftVersion":"1.21.4","loader":"quilt","channels":{"stable":{"release":"1.0.0","tag":"selection-mc-1.21.4-quilt-v1.0.0","manifestSha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}"#;
    let raw = synthetic_catalog(2, &format!("{extra},\"sortOrder\":9}}"));
    let c = parse_catalog(&raw).unwrap();
    let v = visible_targets(&c, false);
    let quilt = v.iter().find(|t| t.loader == "quilt").unwrap();
    assert!(matches!(
        quilt.support,
        Support::LauncherUpdateRequired { .. }
    ));
    // Y el soportado de siempre sigue instalable.
    assert_eq!(
        v.iter()
            .find(|t| t.loader == "ornithe-gen2")
            .unwrap()
            .support,
        Support::Supported
    );
}

#[test]
fn dev_channel_is_hidden_unless_requested() {
    let extra = r#",{"targetId":"mc-1.21.4-fabric","minecraftVersion":"1.21.4","loader":"fabric","channels":{"dev":{"release":"1.0.0","tag":"selection-mc-1.21.4-fabric-v1.0.0","manifestSha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}"#;
    let c = parse_catalog(&synthetic_catalog(2, &format!("{extra},\"sortOrder\":9}}"))).unwrap();
    assert_eq!(visible_targets(&c, false).len(), 1);
    assert_eq!(visible_targets(&c, true).len(), 2);
}

#[test]
fn launcher_version_gate() {
    assert!(launcher_meets("0.1.0", "0.15.2"));
    assert!(launcher_meets("0.15.2", "0.15.2"));
    assert!(launcher_meets("0.15.2", "0.15.2-beta.3"));
    assert!(!launcher_meets("0.16.0", "0.15.2"));
    assert!(!launcher_meets("2.0.0", "0.15.2"));
    assert!(!launcher_meets("basura", "0.15.2"));
}

// ── Replay, offline y caché ─────────────────────────────────────────────

#[test]
fn generation_policy() {
    assert!(check_generation(None, 1, false).is_ok());
    assert!(check_generation(Some(3), 4, false).is_ok());
    assert!(check_generation(Some(3), 3, true).is_ok());
    assert_eq!(
        check_generation(Some(3), 2, false),
        Err(CatalogError::Replay {
            seen: 3,
            offered: 2
        })
    );
    assert_eq!(
        check_generation(Some(3), 3, false),
        Err(CatalogError::Conflict(3))
    );
}

#[tokio::test]
async fn refresh_online_then_replay_of_older_generation_is_refused() {
    let signer = TestSigner::new();
    let dir = temp_dir("replay");
    let store = CatalogStore::with_keys(&dir, signer.keys);
    let t = MockTransport::default();

    let g5 = synthetic_catalog(5, "");
    t.serve_catalog(&g5, &signer.sign(&g5), Some("\"e5\""));
    let s = refresh(&t, &store, true).await.unwrap();
    assert_eq!((s.catalog.generation, s.origin), (5, Origin::Online));

    // Un atacante (o un CDN viejo) sirve el catálogo de la generación 4,
    // correctamente firmado: NO se acepta y se sigue en la 5.
    let g4 = synthetic_catalog(4, "");
    t.serve_catalog(&g4, &signer.sign(&g4), Some("\"e4\""));
    let s = refresh(&t, &store, true).await.unwrap();
    assert_eq!(s.catalog.generation, 5);
    assert_eq!(s.origin, Origin::Cached);
    assert!(s.degraded_reason.unwrap().contains("anterior"));

    // Y la generación mayor sí se acepta.
    let g6 = synthetic_catalog(6, "");
    t.serve_catalog(&g6, &signer.sign(&g6), Some("\"e6\""));
    let s = refresh(&t, &store, true).await.unwrap();
    assert_eq!((s.catalog.generation, s.origin), (6, Origin::Online));
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn replay_protection_survives_restart() {
    let signer = TestSigner::new();
    let dir = temp_dir("restart");
    {
        let store = CatalogStore::with_keys(&dir, signer.keys);
        let t = MockTransport::default();
        let g9 = synthetic_catalog(9, "");
        t.serve_catalog(&g9, &signer.sign(&g9), None);
        refresh(&t, &store, true).await.unwrap();
    }
    // "Reinicio": store nuevo sobre el mismo directorio.
    let store = CatalogStore::with_keys(&dir, signer.keys);
    let t = MockTransport::default();
    let g3 = synthetic_catalog(3, "");
    t.serve_catalog(&g3, &signer.sign(&g3), None);
    let s = refresh(&t, &store, true).await.unwrap();
    assert_eq!(s.catalog.generation, 9);
    assert_eq!(s.origin, Origin::Cached);
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn same_generation_with_different_content_is_refused() {
    let signer = TestSigner::new();
    let dir = temp_dir("conflict");
    let store = CatalogStore::with_keys(&dir, signer.keys);
    let t = MockTransport::default();
    let a = synthetic_catalog(7, "");
    t.serve_catalog(&a, &signer.sign(&a), None);
    refresh(&t, &store, true).await.unwrap();
    let b = String::from_utf8(synthetic_catalog(7, ""))
        .unwrap()
        .replace("TFL Selection 1.8.9", "Otro nombre");
    t.serve_catalog(b.as_bytes(), &signer.sign(b.as_bytes()), None);
    let s = refresh(&t, &store, true).await.unwrap();
    assert_eq!(s.origin, Origin::Cached);
    assert!(s.degraded_reason.unwrap().contains("otro contenido"));
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn github_down_uses_verified_cache_and_says_so() {
    let signer = TestSigner::new();
    let dir = temp_dir("offline");
    let store = CatalogStore::with_keys(&dir, signer.keys);
    let t = MockTransport::default();
    let g = synthetic_catalog(2, "");
    t.serve_catalog(&g, &signer.sign(&g), None);
    refresh(&t, &store, true).await.unwrap();

    t.set(CATALOG_URL, Err(FetchError::Network("sin conexión".into())));
    let s = refresh(&t, &store, true).await.unwrap();
    assert_eq!(s.origin, Origin::Cached);
    assert!(s.degraded_reason.unwrap().contains("sin conexión"));
    assert_eq!(visible_targets(&s.catalog, false).len(), 1);

    t.set(
        CATALOG_URL,
        Ok(Fetched {
            status: 503,
            body: vec![],
            etag: None,
        }),
    );
    let s = refresh(&t, &store, true).await.unwrap();
    assert_eq!(s.origin, Origin::Cached);
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn github_down_without_cache_is_an_error_not_an_empty_list() {
    let signer = TestSigner::new();
    let dir = temp_dir("nocache");
    let store = CatalogStore::with_keys(&dir, signer.keys);
    let t = MockTransport::default();
    t.set(CATALOG_URL, Err(FetchError::Network("caído".into())));
    assert!(matches!(
        refresh(&t, &store, true).await,
        Err(CatalogError::Unavailable(_))
    ));
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn tampered_online_catalog_never_replaces_the_cache() {
    let signer = TestSigner::new();
    let dir = temp_dir("tamper");
    let store = CatalogStore::with_keys(&dir, signer.keys);
    let t = MockTransport::default();
    let good = synthetic_catalog(2, "");
    t.serve_catalog(&good, &signer.sign(&good), None);
    refresh(&t, &store, true).await.unwrap();

    let evil = String::from_utf8(synthetic_catalog(3, ""))
        .unwrap()
        .replace("0.1.0-preview.1", "9.9.9");
    // Firma de OTRO contenido: la verificación falla.
    t.serve_catalog(evil.as_bytes(), &signer.sign(&good), None);
    let s = refresh(&t, &store, true).await.unwrap();
    assert_eq!(s.catalog.generation, 2);
    assert_eq!(s.origin, Origin::Cached);
    assert!(s.degraded_reason.unwrap().contains("firma"));
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn corrupted_cache_on_disk_is_not_trusted() {
    let signer = TestSigner::new();
    let dir = temp_dir("badcache");
    let store = CatalogStore::with_keys(&dir, signer.keys);
    let t = MockTransport::default();
    let g = synthetic_catalog(2, "");
    t.serve_catalog(&g, &signer.sign(&g), None);
    refresh(&t, &store, true).await.unwrap();
    // Alguien edita el catálogo cacheado en disco.
    let path = dir.join("catalog.json");
    let edited = std::fs::read_to_string(&path)
        .unwrap()
        .replace("0.1.0-preview.1", "6.6.6");
    std::fs::write(&path, edited).unwrap();
    t.set(CATALOG_URL, Err(FetchError::Network("offline".into())));
    // Sin red y con caché adulterado: no hay snapshot verificable.
    assert!(matches!(
        refresh(&t, &store, true).await,
        Err(CatalogError::Unavailable(_))
    ));
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn fresh_cache_avoids_network_and_304_keeps_snapshot() {
    let signer = TestSigner::new();
    let dir = temp_dir("fresh");
    let store = CatalogStore::with_keys(&dir, signer.keys);
    let t = MockTransport::default();
    let g = synthetic_catalog(2, "");
    t.serve_catalog(&g, &signer.sign(&g), Some("\"abc\""));
    refresh(&t, &store, false).await.unwrap();
    assert_eq!(t.calls_to(CATALOG_URL), 1);

    // Dentro de la ventana fresca no se vuelve a llamar a GitHub.
    refresh(&t, &store, false).await.unwrap();
    assert_eq!(t.calls_to(CATALOG_URL), 1);

    // Refresco manual: pregunta con ETag y un 304 conserva el snapshot.
    t.set(
        CATALOG_URL,
        Ok(Fetched {
            status: 304,
            body: vec![],
            etag: None,
        }),
    );
    let s = refresh(&t, &store, true).await.unwrap();
    assert_eq!(s.origin, Origin::Online);
    let last = t.calls.lock().unwrap().last().cloned().unwrap();
    assert_eq!(last, (CATALOG_URL.to_string(), Some("\"abc\"".to_string())));
    // El .sig no se vuelve a pedir si el catálogo no cambió.
    assert_eq!(t.calls_to(CATALOG_SIG_URL), 1);
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn real_published_catalog_flows_through_refresh_with_production_keys() {
    let dir = temp_dir("real");
    let store = CatalogStore::new(&dir);
    let t = MockTransport::default();
    t.serve_catalog(REAL_CATALOG, REAL_CATALOG_SIG, None);
    let s = refresh(&t, &store, true).await.unwrap();
    assert_eq!(s.catalog.generation, 1);
    assert_eq!(s.origin, Origin::Online);
    let _ = std::fs::remove_dir_all(dir);
}
