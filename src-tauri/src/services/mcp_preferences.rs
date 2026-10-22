//! Revisioned visual preference authority. UI writes compare-and-swap; localStorage is a migration cache.
use crate::core::PathManager;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::LazyLock;
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Preferences {
    pub revision: u32,
    pub values: std::collections::BTreeMap<String, Value>,
}
static DATA: LazyLock<Mutex<Preferences>> = LazyLock::new(|| {
    Mutex::new(
        std::fs::read(path())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default(),
    )
});
fn path() -> std::path::PathBuf {
    PathManager::get()
        .get_settings_dir()
        .join("visual-authority.json")
}
pub fn snapshot() -> Preferences {
    DATA.lock().clone()
}
pub fn write(key: &str, value: Value, revision: u32) -> Result<Preferences, String> {
    validate(key, &value)?;
    let mut d = DATA.lock();
    if d.revision != revision {
        return Err("PREFERENCE_CONFLICT; reload latest revision".into());
    }
    let mut next = d.clone();
    next.revision = next.revision.checked_add(1).ok_or("Revision limit")?;
    next.values.insert(key.into(), value.clone());
    if key == "visual" {
        next.values.remove("style");
        next.values.remove("accent");
    }
    if key == "scenes" {
        next.values.remove("scene");
        next.values.remove("scene_motion");
    }
    let target = match key {
        "style" => Some(("visual", "style")),
        "accent" => Some(("visual", "customAccent")),
        "scene" => Some(("scenes", "preset")),
        "scene_motion" => Some(("scenes", "motion")),
        _ => None,
    };
    if let Some((doc, field)) = target {
        let entry = next.values.entry(doc.into()).or_insert_with(|| {
            if doc == "scenes" {
                json!({"version":1,"global":{},"instances":{}})
            } else {
                json!({"version":1})
            }
        });
        if doc == "scenes" {
            if !entry["global"].is_object() {
                entry["global"] = json!({});
            }
            entry["global"][field] = value;
        } else {
            entry[field] = value;
        }
    }
    let p = path();
    let tmp = p.with_extension("tmp");
    std::fs::write(
        &tmp,
        serde_json::to_vec(&next).map_err(|_| "Serialization error")?,
    )
    .map_err(|_| "Preference save failed")?;
    std::fs::rename(&tmp, p).map_err(|_| "Preference replace failed")?;
    *d = next.clone();
    Ok(next)
}
pub fn validate(key: &str, v: &Value) -> Result<(), String> {
    let ok = match key {
        "theme" => ["dark", "light"].contains(&v.as_str().unwrap_or("")),
        "quality" => ["Lite", "Balanced", "Experience"].contains(&v.as_str().unwrap_or("")),
        "style" => ["modern", "pixel"].contains(&v.as_str().unwrap_or("")),
        "accent" => v.as_str().is_some_and(|s| {
            s.len() == 7 && s.starts_with('#') && s[1..].bytes().all(|c| c.is_ascii_hexdigit())
        }),
        "scene" => [
            "legacy", "forest", "embers", "void", "night", "cave", "aurora", "pixel",
        ]
        .contains(&v.as_str().unwrap_or("")),
        "scene_motion" => ["off", "subtle", "full"].contains(&v.as_str().unwrap_or("")),
        // Full UI documents are private Tauri IPC only, bounded and never accepted as MCP tool args.
        "visual" | "scenes" | "console" => v.is_object() && v.to_string().len() <= 1024 * 1024,
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        Err("INVALID_PREFERENCE".into())
    }
}
pub fn public() -> Value {
    let d = snapshot();
    let s = crate::services::SettingsManager::snapshot();
    let mut values = serde_json::Map::new();
    for key in ["style", "accent", "scene", "scene_motion"] {
        if let Some(v) = d.values.get(key) {
            values.insert(key.into(), v.clone());
        }
    }
    if let Some(v) = d.values.get("visual") {
        for (key, field) in [("style", "style"), ("accent", "customAccent")] {
            if !v[field].is_null() {
                values.insert(key.into(), v[field].clone());
            }
        }
    }
    if let Some(v) = d.values.get("scenes") {
        for (key, field) in [("scene", "preset"), ("scene_motion", "motion")] {
            if !v["global"][field].is_null() {
                values.insert(key.into(), v["global"][field].clone());
            }
        }
    }
    json!({"revision":d.revision,"theme":s.theme,"quality":s.quality_profile,"values":values,"console":d.values.get("console")})
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preferences_are_allowlisted() {
        assert!(validate("access_token", &json!("secret")).is_err());
        assert!(validate("scene", &json!("https://evil")).is_err());
        assert!(validate("style", &json!("pixel")).is_ok());
        assert!(validate("accent", &json!("#ffffff")).is_ok());
    }
}

pub fn instance_metadata(id: &str) -> Value {
    let d = DATA.lock();
    let m = d
        .values
        .get("visual")
        .and_then(|v| v.get("instances"))
        .and_then(|v| v.get(id));
    json!({"favorite":m.and_then(|v|v.get("favorite")).and_then(Value::as_bool).unwrap_or(false),"group":m.and_then(|v|v.get("group")).and_then(Value::as_str).unwrap_or("")})
}
