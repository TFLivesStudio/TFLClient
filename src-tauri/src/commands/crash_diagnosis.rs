//! Cuando el juego se cierra mal, intenta señalar QUÉ mod lo causó leyendo el
//! crash report y el `latest.log` — el usuario promedio no sabe leer un stack
//! trace, y "sacá mods de a uno hasta que ande" es lo que hacía a mano. Es un
//! diagnóstico de mejor esfuerzo: nombra sospechosos, no promete certeza.

use crate::services::instance_manager;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::command;

/// Ids que aparecen en los logs pero no son un jar de /mods/ que se pueda
/// culpar ni desactivar.
const NON_MOD_IDS: &[&str] = &[
    "minecraft",
    "forge",
    "neoforge",
    "fml",
    "fabricloader",
    "fabric-loader",
    "fabric_loader",
    "quilt_loader",
    "quilt-loader",
    "java",
    "mixin",
    "mixinextras",
];

const MAX_SUSPECTS: usize = 5;
const CRASH_REPORT_MAX_AGE_SECS: u64 = 30 * 60;
const LOG_TAIL_BYTES: usize = 400 * 1024;

#[derive(Debug, Serialize, Clone)]
pub struct CrashSuspect {
    pub filename: String,
    pub mod_id: String,
    pub mod_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CrashDiagnosis {
    pub suspects: Vec<CrashSuspect>,
    /// Hay una actualización de mods reciente que se puede deshacer — si el
    /// juego se rompió justo después de actualizar, es la salida más segura.
    pub rollback_available: bool,
}

struct JarMod {
    filename: String,
    ids: Vec<String>,
    name: Option<String>,
}

fn read_jar_mods(path: &Path) -> Option<JarMod> {
    let filename = path.file_name()?.to_str()?.to_string();
    let file = std::fs::File::open(path).ok()?;
    let mut zip = zip::ZipArchive::new(file).ok()?;
    let mut ids = Vec::new();
    let mut name = None;

    if let Ok(mut entry) = zip.by_name("fabric.mod.json") {
        let mut raw = String::new();
        if std::io::Read::read_to_string(&mut entry, &mut raw).is_ok() {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
                    ids.push(id.to_string());
                }
                if let Some(provides) = v.get("provides").and_then(|x| x.as_array()) {
                    ids.extend(provides.iter().filter_map(|p| p.as_str().map(String::from)));
                }
                name = v.get("name").and_then(|x| x.as_str()).map(String::from);
            }
        }
    }
    if let Ok(mut entry) = zip.by_name("quilt.mod.json") {
        let mut raw = String::new();
        if std::io::Read::read_to_string(&mut entry, &mut raw).is_ok() {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(id) = v.pointer("/quilt_loader/id").and_then(|x| x.as_str()) {
                    ids.push(id.to_string());
                }
                if name.is_none() {
                    name = v
                        .pointer("/quilt_loader/metadata/name")
                        .and_then(|x| x.as_str())
                        .map(String::from);
                }
            }
        }
    }
    for toml_path in ["META-INF/mods.toml", "META-INF/neoforge.mods.toml"] {
        let Ok(mut entry) = zip.by_name(toml_path) else {
            continue;
        };
        let mut raw = String::new();
        if std::io::Read::read_to_string(&mut entry, &mut raw).is_err() {
            continue;
        }
        let Ok(parsed) = raw.parse::<toml::Table>() else {
            continue;
        };
        if let Some(mods) = parsed.get("mods").and_then(|m| m.as_array()) {
            for m in mods {
                if let Some(id) = m.get("modId").and_then(|x| x.as_str()) {
                    ids.push(id.to_string());
                }
                if name.is_none() {
                    name = m
                        .get("displayName")
                        .and_then(|x| x.as_str())
                        .map(String::from);
                }
            }
        }
    }

    if ids.is_empty() {
        return None;
    }
    Some(JarMod {
        filename,
        ids,
        name,
    })
}

/// El crash report más nuevo, si es de hace poco (no se culpa a un mod por
/// un crash de hace una semana).
async fn newest_recent_crash_report(instance_dir: &Path) -> Option<String> {
    let mut entries = tokio::fs::read_dir(instance_dir.join("crash-reports"))
        .await
        .ok()?;
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let Ok(meta) = entry.metadata().await else {
            continue;
        };
        let Ok(modified) = meta.modified() else {
            continue;
        };
        if best.as_ref().is_none_or(|(t, _)| modified > *t) {
            best = Some((modified, entry.path()));
        }
    }
    let (modified, path) = best?;
    let age = std::time::SystemTime::now().duration_since(modified).ok()?;
    if age.as_secs() > CRASH_REPORT_MAX_AGE_SECS {
        return None;
    }
    let bytes = tokio::fs::read(path).await.ok()?;
    Some(String::from_utf8_lossy(&bytes).to_string())
}

async fn latest_log_tail(instance_dir: &Path) -> Option<String> {
    let bytes = tokio::fs::read(instance_dir.join("logs").join("latest.log"))
        .await
        .ok()?;
    let start = bytes.len().saturating_sub(LOG_TAIL_BYTES);
    Some(String::from_utf8_lossy(&bytes[start..]).to_string())
}

/// Ids de mod / nombres de jar que el texto menciona como causa, en orden
/// de aparición y sin repetir.
fn extract_culprits(text: &str) -> (Vec<String>, Vec<String>) {
    let mut ids: Vec<String> = Vec::new();
    let mut jars: Vec<String> = Vec::new();
    let push_id = |ids: &mut Vec<String>, id: &str| {
        let id = id.trim();
        if !id.is_empty() && !ids.iter().any(|x| x == id) {
            ids.push(id.to_string());
        }
    };

    // Forge/NeoForge: bloque "Suspected Mods:" con una línea por mod.
    if let Some(pos) = text.find("Suspected Mod") {
        let suspected_line =
            Regex::new(r"^\s+.+? \(([A-Za-z0-9_.\-]+)\), Version:").expect("regex válido");
        for line in text[pos..].lines().skip(1) {
            if line.trim().is_empty() {
                break;
            }
            if let Some(c) = suspected_line.captures(line) {
                push_id(&mut ids, &c[1]);
            }
        }
    }

    let id_patterns = [
        r"provided by '([A-Za-z0-9_.\-]+)'",
        r"Mod ID: '([A-Za-z0-9_.\-]+)'",
        r"- Mod '[^']+' \(([A-Za-z0-9_.\-]+)\) \S+ (?:requires|breaks|conflicts)",
        r"[Mm]ixin apply for mod ([A-Za-z0-9_.\-]+) failed",
    ];
    for pattern in id_patterns {
        let re = Regex::new(pattern).expect("regex válido");
        for c in re.captures_iter(text) {
            push_id(&mut ids, &c[1]);
        }
    }

    let jar_re = Regex::new(r"Mod File: (.+?\.jar)").expect("regex válido");
    for c in jar_re.captures_iter(text) {
        let name = c[1].rsplit(['/', '\\']).next().unwrap_or(&c[1]).to_string();
        if !jars.contains(&name) {
            jars.push(name);
        }
    }

    ids.retain(|id| !NON_MOD_IDS.contains(&id.to_ascii_lowercase().as_str()));
    (ids, jars)
}

#[command]
pub async fn diagnose_crash(instance_name: String) -> Result<CrashDiagnosis, String> {
    let instance = instance_manager::get_instance(&instance_name).await?;
    let dir = instance.dir();

    let mut text = String::new();
    if let Some(report) = newest_recent_crash_report(&dir).await {
        text.push_str(&report);
        text.push('\n');
    }
    if let Some(log) = latest_log_tail(&dir).await {
        text.push_str(&log);
    }

    let rollback_available = tokio::fs::try_exists(dir.join(".tfl_rollback").join("manifest.json"))
        .await
        .unwrap_or(false);

    if text.trim().is_empty() {
        return Ok(CrashDiagnosis {
            suspects: Vec::new(),
            rollback_available,
        });
    }

    let (culprit_ids, culprit_jars) = extract_culprits(&text);

    let mods_dir = dir.join("mods");
    let installed: Vec<JarMod> = tokio::task::spawn_blocking(move || {
        let mut out = Vec::new();
        let Ok(entries) = std::fs::read_dir(&mods_dir) else {
            return out;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jar") {
                continue;
            }
            if let Some(m) = read_jar_mods(&path) {
                out.push(m);
            }
        }
        out
    })
    .await
    .map_err(|e| e.to_string())?;

    let by_id: HashMap<String, &JarMod> = installed
        .iter()
        .flat_map(|m| m.ids.iter().map(move |id| (id.to_ascii_lowercase(), m)))
        .collect();

    let mut suspects: Vec<CrashSuspect> = Vec::new();
    let mut add = |filename: &str, mod_id: &str, mod_name: Option<String>| {
        if suspects.len() < MAX_SUSPECTS && !suspects.iter().any(|s| s.filename == filename) {
            suspects.push(CrashSuspect {
                filename: filename.to_string(),
                mod_id: mod_id.to_string(),
                mod_name,
            });
        }
    };
    for id in &culprit_ids {
        if let Some(m) = by_id.get(&id.to_ascii_lowercase()) {
            add(&m.filename, id, m.name.clone());
        }
    }
    for jar in &culprit_jars {
        if let Some(m) = installed.iter().find(|m| &m.filename == jar) {
            let id = m.ids.first().cloned().unwrap_or_default();
            add(&m.filename, &id, m.name.clone());
        }
    }

    Ok(CrashDiagnosis {
        suspects,
        rollback_available,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forge_suspected_mods_block() {
        let report = "\
---- Minecraft Crash Report ----
Description: Exception in server tick loop

Suspected Mods:
\tCreate (create), Version: 0.5.1
\t\tIssue tracker URL: https://example.com
\tSodium (sodium), Version: 0.5.3

Stacktrace:
";
        let (ids, _) = extract_culprits(report);
        assert_eq!(ids, vec!["create".to_string(), "sodium".to_string()]);
    }

    #[test]
    fn suspected_none_yields_nothing() {
        let (ids, jars) = extract_culprits("Suspected Mods: NONE\n\nStacktrace:\n");
        assert!(ids.is_empty());
        assert!(jars.is_empty());
    }

    #[test]
    fn fabric_entrypoint_and_mod_file() {
        let log = "\
Could not execute entrypoint stage 'main' due to errors, provided by 'badmod'!
Mod File: /home/u/.tflclient/instances/x/mods/other-mod-1.0.jar
Mod ID: 'minecraft'
";
        let (ids, jars) = extract_culprits(log);
        assert_eq!(ids, vec!["badmod".to_string()]);
        assert_eq!(jars, vec!["other-mod-1.0.jar".to_string()]);
    }

    #[test]
    fn fabric_dependency_error() {
        let log =
            "- Mod 'Fancy' (fancymod) 1.0.0 requires version 2.x of mod 'lib', which is missing!";
        let (ids, _) = extract_culprits(log);
        assert_eq!(ids, vec!["fancymod".to_string()]);
    }

    fn write_jar(entries: &[(&str, &str)]) -> PathBuf {
        let path = std::env::temp_dir().join(format!("tfl-jar-test-{}.jar", uuid::Uuid::new_v4()));
        let file = std::fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        for (name, body) in entries {
            zip.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            std::io::Write::write_all(&mut zip, body.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
        path
    }

    #[test]
    fn reads_fabric_mod_id_and_name() {
        let jar = write_jar(&[(
            "fabric.mod.json",
            r#"{"id":"sodium","name":"Sodium","provides":["indium"]}"#,
        )]);
        let m = read_jar_mods(&jar).unwrap();
        assert_eq!(m.ids, vec!["sodium".to_string(), "indium".to_string()]);
        assert_eq!(m.name.as_deref(), Some("Sodium"));
        let _ = std::fs::remove_file(jar);
    }

    #[test]
    fn reads_forge_mods_toml() {
        let toml = "modLoader=\"javafml\"\n[[mods]]\nmodId=\"create\"\ndisplayName=\"Create\"\n";
        let jar = write_jar(&[("META-INF/mods.toml", toml)]);
        let m = read_jar_mods(&jar).unwrap();
        assert_eq!(m.ids, vec!["create".to_string()]);
        assert_eq!(m.name.as_deref(), Some("Create"));
        let _ = std::fs::remove_file(jar);
    }

    #[test]
    fn jar_without_metadata_is_ignored() {
        let jar = write_jar(&[("readme.txt", "hi")]);
        assert!(read_jar_mods(&jar).is_none());
        let _ = std::fs::remove_file(jar);
    }
}
