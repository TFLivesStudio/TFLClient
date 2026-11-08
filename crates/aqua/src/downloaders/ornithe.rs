use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use log::{info, warn};
use serde::Deserialize;

use super::batch::{DownloadBatch, DownloadItemSpec};
use crate::AquaError;
use crate::manifest::fetch_release_times;
use crate::progress::DownloadStage;
use crate::utilities::{HTTP_CLIENT, fetch_text_retrying};

const ORNITHE_VERSIONS_URL: &str = "https://meta.ornithemc.net/v3/versions";
/// Generación de intermediary que usa todo TFL Selection hoy (ver la URL
/// `/v3/versions/gen2/fabric-loader/...` más abajo) — hardcodeado a propósito,
/// no a vanguardia genérica: si algún día se soporta gen1, hay que revisar
/// esto también.
const INTERMEDIARY_GENERATION: u32 = 2;

#[derive(Deserialize)]
#[allow(dead_code)]
struct OrnitheProfile {
    id: String,
    libraries: Vec<OrnitheLibrary>,
}

#[derive(Deserialize, Clone)]
struct OrnitheLibrary {
    name: String,
    url: String,
}

/// Una entrada de `libraryUpgrades` en `/v3/versions` — Ornithe publica acá
/// las librerías (log4j, slf4j, fastutil, gson, parches propios) que
/// reemplazan a las que trae el manifiesto vanilla heredado para versiones
/// viejas de Minecraft. El perfil de `/fabric-loader/.../profile/json` NO
/// las incluye — hay que traerlas de acá aparte e inyectarlas nosotros.
#[derive(Deserialize, Clone)]
struct LibraryUpgrade {
    name: String,
    url: String,
    #[serde(default, rename = "minGameVersion")]
    min_game_version: Option<String>,
    #[serde(default, rename = "maxGameVersion")]
    max_game_version: Option<String>,
    #[serde(default, rename = "minIntermediaryGeneration")]
    min_intermediary_generation: Option<u32>,
    #[serde(default, rename = "maxIntermediaryGeneration")]
    max_intermediary_generation: Option<u32>,
}

#[derive(Deserialize)]
struct OrnitheVersionsMeta {
    #[serde(rename = "libraryUpgrades")]
    library_upgrades: Vec<LibraryUpgrade>,
}

/// `group:artifact` de un coordenada Maven completa (`group:artifact:version`
/// o `group:artifact:version:classifier`) — para comparar/deduplicar
/// ignorando la versión, igual que `maven_key` en
/// `crates/zellkern/src/resolvers/classpath.rs`.
fn maven_group_artifact(name: &str) -> Option<&str> {
    let mut parts = name.splitn(3, ':');
    let group = parts.next()?;
    let artifact = parts.next()?;
    let group_end = group.len();
    let artifact_end = group_end + 1 + artifact.len();
    name.get(..artifact_end)
}

fn upgrade_applies(
    upgrade: &LibraryUpgrade,
    game_version: &str,
    release_times: &HashMap<String, String>,
) -> bool {
    if let Some(min_gen) = upgrade.min_intermediary_generation
        && INTERMEDIARY_GENERATION < min_gen
    {
        return false;
    }
    if let Some(max_gen) = upgrade.max_intermediary_generation
        && INTERMEDIARY_GENERATION > max_gen
    {
        return false;
    }

    let Some(target_time) = release_times.get(game_version) else {
        // No se pudo ubicar la versión en el manifiesto oficial de Mojang —
        // no hay forma segura de evaluar un límite por versión, así que una
        // mejora acotada (`min/maxGameVersion`) se descarta por las dudas.
        // Una mejora sin límite de versión (como el upgrade de log4j, que
        // no tiene min/maxGameVersion) sigue aplicando igual.
        return upgrade.min_game_version.is_none() && upgrade.max_game_version.is_none();
    };

    if let Some(max_v) = &upgrade.max_game_version {
        match release_times.get(max_v) {
            Some(max_time) if target_time.as_str() > max_time.as_str() => return false,
            None => return false,
            _ => {}
        }
    }
    if let Some(min_v) = &upgrade.min_game_version {
        match release_times.get(min_v) {
            Some(min_time) if target_time.as_str() < min_time.as_str() => return false,
            None => return false,
            _ => {}
        }
    }
    true
}

/// Trae y filtra los `libraryUpgrades` de Ornithe que aplican a
/// `game_version`. Fallo soft: si no se puede resolver (red caída, formato
/// inesperado), se loguea y se sigue sin upgrades — es mejor que Ornithe
/// Gen2 arranque con el log4j viejo de vanilla (como ya hacía antes de este
/// fix) a que una falla de red acá bloquee la instalación entera.
async fn fetch_applicable_library_upgrades(game_version: &str) -> Vec<LibraryUpgrade> {
    let fetch = async {
        let meta: OrnitheVersionsMeta = HTTP_CLIENT
            .get(ORNITHE_VERSIONS_URL)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let release_times = fetch_release_times().await?;
        Ok::<_, AquaError>((meta, release_times))
    };

    match fetch.await {
        Ok((meta, release_times)) => meta
            .library_upgrades
            .into_iter()
            .filter(|u| upgrade_applies(u, game_version, &release_times))
            .collect(),
        Err(e) => {
            warn!(
                "No se pudieron resolver los libraryUpgrades de Ornithe para {game_version}: {e} — se instala sin ellos"
            );
            Vec::new()
        }
    }
}

/// `true` si el `.json` de versión Ornithe Gen2 ya cacheado en disco es de
/// ANTES de este fix (issue #16): sin `log4j-api` en su classpath, así que
/// el log4j viejo de vanilla 1.8.9 (2.0-beta9) le gana y cualquier mod
/// compilado contra una API de log4j más nueva revienta con
/// `NoSuchMethodError` al arrancar (Sarcio, en concreto). Quien ya tiene una
/// instancia instalada necesita que esto se detecte solo, sin tener que
/// borrar nada a mano — lo llama `ensure_downloaded` en
/// `src-tauri/src/services/launcher.rs` para decidir si hace falta
/// re-resolver el perfil en vez de confiar en el `.json` tal cual está.
pub fn needs_log4j_repair(shared_dir: &Path, version_id: &str) -> bool {
    let json_path = shared_dir
        .join("versions")
        .join(version_id)
        .join(format!("{version_id}.json"));
    let Ok(text) = std::fs::read_to_string(&json_path) else {
        // No existe todavía — lo resuelve el flujo normal de instalación,
        // no hace falta "reparar" nada.
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return true;
    };
    let Some(libs) = value.get("libraries").and_then(|v| v.as_array()) else {
        return true;
    };
    !libs.iter().any(|l| {
        l.get("name")
            .and_then(|n| n.as_str())
            .map(|n| n.starts_with("org.apache.logging.log4j:log4j-api"))
            .unwrap_or(false)
    })
}

pub struct OrnitheBatch {
    version_id: String,
    game_version: String,
    loader_version: String,
    shared_dir: PathBuf,
    items: Vec<DownloadItemSpec>,
    profile_json: String,
}

impl OrnitheBatch {
    pub async fn new(
        shared_dir: &Path,
        game_version: &str,
        loader_version: &str,
    ) -> Result<Self, AquaError> {
        let profile_url = format!(
            "https://meta.ornithemc.net/v3/versions/gen2/fabric-loader/{}/{}/profile/json",
            game_version, loader_version
        );

        let mut profile_text = fetch_text_retrying(&profile_url).await.map_err(|e| {
            AquaError::Other(format!("No se pudo conectar a meta.ornithemc.net: {e}"))
        })?;

        // En meta.ornithemc.net el inheritsFrom es "{game_version}-vanilla", pero
        // el launcher almacena la versión base como "{game_version}". Lo adaptamos
        // para que zellkern resuelva el manifiesto padre de Mojang correctamente.
        let vanilla_pattern = format!("\"{}-vanilla\"", game_version);
        let replacement = format!("\"{}\"", game_version);
        profile_text = profile_text.replace(&vanilla_pattern, &replacement);

        let profile: OrnitheProfile = serde_json::from_str(&profile_text)
            .map_err(|e| AquaError::Other(format!("Error analizando perfil Ornithe: {e}")))?;

        let version_id = profile.id.clone();

        // El perfil de /fabric-loader/.../profile/json NO trae log4j/slf4j —
        // Minecraft 1.8.9 hereda el log4j-api/core 2.0-beta9 de vanilla, y
        // mods compilados contra una API de log4j más nueva (ej. Sarcio,
        // contra 2.19.0) revientan con NoSuchMethodError al arrancar. Ornithe
        // publica el fix oficial como libraryUpgrades en /v3/versions — ver
        // issue #16. Se inyectan acá los que apliquen a esta versión.
        let upgrades = fetch_applicable_library_upgrades(game_version).await;

        let mut libraries = profile.libraries.clone();
        if !upgrades.is_empty() {
            let upgrade_keys: Vec<&str> = upgrades
                .iter()
                .filter_map(|u| maven_group_artifact(&u.name))
                .collect();
            // Si el perfil original ya trae una librería que un upgrade va a
            // reemplazar, se saca — el "last wins" de zellkern igual
            // resolvería bien el conflicto, pero así no queda el jar viejo
            // sin usar ocupando descarga/disco.
            libraries.retain(|lib| {
                maven_group_artifact(&lib.name)
                    .map(|k| !upgrade_keys.contains(&k))
                    .unwrap_or(true)
            });
            for u in &upgrades {
                libraries.push(OrnitheLibrary {
                    name: u.name.clone(),
                    url: u.url.clone(),
                });
            }
            info!(
                "Ornithe Gen2 {game_version}: {} librería(s) de upgrade aplicadas ({})",
                upgrades.len(),
                upgrades
                    .iter()
                    .map(|u| u.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        info!(
            "Ornithe Gen2 perfil cargado: {} librerías para {}",
            libraries.len(),
            version_id
        );

        let lib_base_dir = shared_dir.join("libraries");
        let mut items = Vec::with_capacity(libraries.len());

        for lib in &libraries {
            let parts: Vec<&str> = lib.name.split(':').collect();
            if parts.len() < 3 {
                continue;
            }

            let group = parts[0].replace('.', "/");
            let artifact = parts[1];
            let version = parts[2];

            let rel_path = format!(
                "{}/{}/{}/{}-{}.jar",
                group, artifact, version, artifact, version
            );
            let dest_path = lib_base_dir.join(&rel_path);
            let base_url = if lib.url.ends_with('/') {
                lib.url.clone()
            } else {
                format!("{}/", lib.url)
            };
            let download_url = format!("{}{}", base_url, rel_path);

            items.push(
                DownloadItemSpec::new(download_url, dest_path, &lib.name)
                    .with_stage(DownloadStage::Library),
            );
        }

        // El JSON de versión guardado en disco (`prepare()`, más abajo) tiene
        // que reflejar las mismas librerías que se acaban de resolver acá —
        // si no, el classpath del lanzamiento real (que lee ese archivo, no
        // esta lista en memoria) sigue viendo solo las originales del
        // perfil. Se muta el campo "libraries" sobre el JSON genérico en vez
        // de reserializar desde `OrnitheProfile` para no perder el resto del
        // perfil (arguments, mainClass, inheritsFrom, etc. — `OrnitheProfile`
        // solo tipa `id`/`libraries` a propósito, el resto no nos interesa
        // tocarlo).
        if !upgrades.is_empty() {
            let mut profile_value: serde_json::Value = serde_json::from_str(&profile_text)
                .map_err(|e| AquaError::Other(format!("Error analizando perfil Ornithe: {e}")))?;
            let libs_json: Vec<serde_json::Value> = libraries
                .iter()
                .map(|l| serde_json::json!({ "name": l.name, "url": l.url }))
                .collect();
            profile_value["libraries"] = serde_json::Value::Array(libs_json);
            profile_text = serde_json::to_string(&profile_value)
                .map_err(|e| AquaError::Other(format!("Error serializando perfil Ornithe: {e}")))?;
        }

        Ok(Self {
            version_id,
            game_version: game_version.to_string(),
            loader_version: loader_version.to_string(),
            shared_dir: shared_dir.to_path_buf(),
            items,
            profile_json: profile_text,
        })
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    pub fn game_version(&self) -> &str {
        &self.game_version
    }

    pub fn loader_version(&self) -> &str {
        &self.loader_version
    }
}

impl DownloadBatch for OrnitheBatch {
    fn name(&self) -> String {
        self.version_id.clone()
    }

    fn items(&self) -> &[DownloadItemSpec] {
        &self.items
    }

    fn prepare(&self) -> Pin<Box<dyn Future<Output = Result<(), AquaError>> + Send + '_>> {
        let version_dir = self.shared_dir.join("versions").join(&self.version_id);
        let json_path = version_dir.join(format!("{}.json", self.version_id));
        let json = self.profile_json.clone();

        // A diferencia de otros batches, acá SIEMPRE se reescribe aunque el
        // archivo ya exista: `OrnitheBatch::new()` puede cambiar qué
        // librerías trae el perfil (ver `fetch_applicable_library_upgrades`,
        // issue #16) sin que cambie `version_id`, así que un `.json` viejo
        // en caché de una ejecución anterior de este mismo launcher nunca se
        // actualizaría solo — y eso era justamente lo que dejaba el log4j
        // viejo de vanilla pisando el de la corrección. Escribir un JSON
        // chico no cuesta nada, no vale la pena el atajo de "si ya existe,
        // no tocar".
        Box::pin(async move {
            tokio::fs::create_dir_all(&version_dir)
                .await
                .map_err(|e| AquaError::Other(format!("Error creando carpeta de versión: {e}")))?;
            tokio::fs::write(&json_path, &json)
                .await
                .map_err(|e| AquaError::Other(format!("Error guardando JSON de versión: {e}")))?;
            info!("Perfil Ornithe Gen2 guardado en: {:?}", json_path);
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upgrade(name: &str, min_gv: Option<&str>, max_gv: Option<&str>) -> LibraryUpgrade {
        LibraryUpgrade {
            name: name.to_string(),
            url: "https://libraries.minecraft.net/".to_string(),
            min_game_version: min_gv.map(str::to_string),
            max_game_version: max_gv.map(str::to_string),
            min_intermediary_generation: None,
            max_intermediary_generation: None,
        }
    }

    fn release_times() -> HashMap<String, String> {
        // Orden real: 13w38c < 14w26c < 1.8.9 < 17w14a (ver issue #16).
        [
            ("13w38c", "2013-09-20T15:11:34+00:00"),
            ("14w26c", "2014-06-26T15:05:03+00:00"),
            ("1.8.9", "2015-12-03T09:24:39+00:00"),
            ("17w14a", "2017-04-05T13:58:01+00:00"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    }

    #[test]
    fn maven_group_artifact_ignora_version_y_classifier() {
        assert_eq!(
            maven_group_artifact("org.apache.logging.log4j:log4j-api:2.19.0"),
            Some("org.apache.logging.log4j:log4j-api")
        );
        assert_eq!(
            maven_group_artifact("net.fabricmc:fabric-loader:0.20.1:natives"),
            Some("net.fabricmc:fabric-loader")
        );
        assert_eq!(maven_group_artifact("sin-formato-maven"), None);
    }

    #[test]
    fn upgrade_sin_limite_de_version_siempre_aplica() {
        let log4j = upgrade("org.apache.logging.log4j:log4j-api:2.19.0", None, None);
        assert!(upgrade_applies(&log4j, "1.8.9", &release_times()));
        // Incluso para una versión que ni siquiera está en el manifiesto de
        // Mojang (no debería poder bloquear algo sin límite declarado).
        assert!(upgrade_applies(
            &log4j,
            "version-inventada",
            &HashMap::new()
        ));
    }

    #[test]
    fn upgrade_con_max_game_version_respeta_el_limite() {
        // log4j-patch real de Ornithe: maxGameVersion 17w14a. 1.8.9 es
        // anterior, así que aplica.
        let log4j_patch = upgrade("net.ornithemc:log4j-patch:1.0.0", None, Some("17w14a"));
        assert!(upgrade_applies(&log4j_patch, "1.8.9", &release_times()));

        // El límite es inclusivo: la versión límite en sí misma también pasa.
        assert!(upgrade_applies(&log4j_patch, "17w14a", &release_times()));

        // Una versión posterior al límite (14w26c < 17w14a < ninguna más en
        // el fixture, así que se agrega un límite más estricto acá) no debe
        // pasar: se reusa con un límite más bajo para probar el corte.
        let con_limite_mas_bajo =
            upgrade("net.ejemplo:solo-hasta-14w26c:1.0.0", None, Some("14w26c"));
        assert!(!upgrade_applies(
            &con_limite_mas_bajo,
            "1.8.9",
            &release_times()
        ));
    }

    #[test]
    fn upgrade_con_min_game_version_respeta_el_limite() {
        let posterior_a_1_8_9 = upgrade(
            "net.ejemplo:solo-para-versiones-nuevas:1.0.0",
            Some("17w14a"),
            None,
        );
        assert!(!upgrade_applies(
            &posterior_a_1_8_9,
            "1.8.9",
            &release_times()
        ));
    }

    #[test]
    fn upgrade_con_limite_no_verificable_se_descarta_por_las_dudas() {
        // Si el límite declarado no está en el manifiesto de Mojang que se
        // pudo resolver, no hay forma segura de evaluarlo — falla cerrado
        // en vez de aplicar una librería que quizás no corresponde.
        let con_limite_desconocido = upgrade(
            "net.ejemplo:algo:1.0.0",
            None,
            Some("version-que-no-existe"),
        );
        assert!(!upgrade_applies(
            &con_limite_desconocido,
            "1.8.9",
            &release_times()
        ));
    }

    #[test]
    fn needs_log4j_repair_detecta_json_sin_regenerar() {
        let dir = std::env::temp_dir().join(format!("tfl-ornithe-test-{}", uuid::Uuid::new_v4()));
        let version_id = "fabric-loader-0.20.1-1.8.9-ornithe-gen2";
        let version_dir = dir.join("versions").join(version_id);
        std::fs::create_dir_all(&version_dir).unwrap();
        let json_path = version_dir.join(format!("{version_id}.json"));

        // JSON viejo (pre-fix, issue #16): sin log4j-api en sus librerías.
        std::fs::write(
            &json_path,
            r#"{"id":"fabric-loader-0.20.1-1.8.9-ornithe-gen2","libraries":[{"name":"net.fabricmc:fabric-loader:0.20.1","url":"https://maven.fabricmc.net/"}]}"#,
        )
        .unwrap();
        assert!(needs_log4j_repair(&dir, version_id));

        // JSON ya reparado: con log4j-api.
        std::fs::write(
            &json_path,
            r#"{"id":"fabric-loader-0.20.1-1.8.9-ornithe-gen2","libraries":[{"name":"org.apache.logging.log4j:log4j-api:2.19.0","url":"https://libraries.minecraft.net/"}]}"#,
        )
        .unwrap();
        assert!(!needs_log4j_repair(&dir, version_id));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn needs_log4j_repair_sin_json_no_es_una_reparacion_pendiente() {
        let dir = std::env::temp_dir().join(format!("tfl-ornithe-test-{}", uuid::Uuid::new_v4()));
        // Ni siquiera existe el .json todavía — lo resuelve la instalación
        // normal, no es un caso de "reparar".
        assert!(!needs_log4j_repair(
            &dir,
            "fabric-loader-0.20.1-1.8.9-ornithe-gen2"
        ));
    }

    /// Prueba de integración real contra meta.ornithemc.net (sin mocks, igual
    /// que el resto de los tests de este crate) — reproduce exactamente el
    /// bug del issue #16: antes de este fix, el perfil de Ornithe Gen2 para
    /// 1.8.9/0.20.1 no traía log4j en absoluto.
    #[tokio::test]
    async fn ornithe_gen2_1_8_9_incluye_log4j_2_19_0_tras_el_fix() {
        let shared_dir =
            std::env::temp_dir().join(format!("tfl-ornithe-batch-test-{}", uuid::Uuid::new_v4()));

        let batch = OrnitheBatch::new(&shared_dir, "1.8.9", "0.20.1")
            .await
            .expect("debería poder resolver el perfil real de Ornithe Gen2");

        assert_eq!(
            batch.version_id(),
            "fabric-loader-0.20.1-1.8.9-ornithe-gen2"
        );

        let has_log4j_api_2_19 = batch
            .items
            .iter()
            .any(|i| i.label.contains("log4j-api") && i.label.contains("2.19.0"));
        assert!(
            has_log4j_api_2_19,
            "el batch de descarga debería incluir log4j-api 2.19.0, items: {:?}",
            batch.items.iter().map(|i| &i.label).collect::<Vec<_>>()
        );

        let profile_value: serde_json::Value = serde_json::from_str(&batch.profile_json).unwrap();
        let libs = profile_value["libraries"].as_array().unwrap();
        assert!(
            libs.iter().any(|l| l["name"]
                .as_str()
                .map(|n| n.starts_with("org.apache.logging.log4j:log4j-api:2.19.0"))
                .unwrap_or(false)),
            "el JSON de versión guardado en disco debería traer log4j-api 2.19.0 en \"libraries\""
        );
        // El resto del perfil original (mainClass, id, etc.) no debería
        // perderse al reescribir el campo "libraries".
        assert_eq!(
            profile_value["id"].as_str(),
            Some("fabric-loader-0.20.1-1.8.9-ornithe-gen2")
        );
        assert!(profile_value.get("mainClass").is_some());
    }
}
