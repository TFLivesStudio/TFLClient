//! Rollback de la última actualización de mods.
//!
//! `update_all` ya no borra los jars viejos: los mueve a `.tfl_rollback/`
//! junto con un manifest de qué reemplazó a qué. Si después de actualizar el
//! juego no arranca (o el usuario simplemente no quiere la versión nueva), un
//! click lo deja como estaba. Se guarda solo la última tanda: una
//! actualización nueva reemplaza el rollback anterior.

use super::filesystem::{move_file, unix_now};
use crate::services::instance_manager;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
struct RollbackEntry {
    /// Jar viejo que se movió a la carpeta de rollback.
    old: String,
    /// Archivos nuevos que lo reemplazaron en `mods/`.
    new: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RollbackManifest {
    created_at: i64,
    entries: Vec<RollbackEntry>,
}

#[derive(Debug, Serialize)]
pub struct ModRollbackInfo {
    pub created_at: i64,
    pub count: u32,
}

pub(crate) fn rollback_dir(instance_dir: &Path) -> PathBuf {
    instance_dir.join(".tfl_rollback")
}

async fn read_rollback_manifest(dir: &Path) -> Option<RollbackManifest> {
    let raw = tokio::fs::read(dir.join("manifest.json")).await.ok()?;
    serde_json::from_slice(&raw).ok()
}

/// Arma el rollback mientras `update_all` va reemplazando mods.
pub(crate) struct RollbackRecorder {
    dir: PathBuf,
    ready: bool,
    entries: Vec<RollbackEntry>,
}

impl RollbackRecorder {
    /// Descarta el rollback anterior y prepara la carpeta. Si no se puede
    /// crear, las actualizaciones siguen igual pero sin posibilidad de deshacer.
    pub(crate) async fn begin(dir: PathBuf) -> Self {
        let _ = tokio::fs::remove_dir_all(&dir).await;
        let ready = tokio::fs::create_dir_all(&dir).await.is_ok();
        Self {
            dir,
            ready,
            entries: Vec::new(),
        }
    }

    /// Saca el jar viejo de `mods_dir`: lo guarda en el rollback si se puede,
    /// y si no lo borra (como antes de que existiera el rollback).
    pub(crate) async fn retire(&mut self, mods_dir: &Path, old: &str, new: Vec<String>) {
        let old_path = mods_dir.join(old);
        if self.ready && move_file(&old_path, &self.dir.join(old)).await.is_ok() {
            self.entries.push(RollbackEntry {
                old: old.to_string(),
                new,
            });
        } else {
            let _ = tokio::fs::remove_file(&old_path).await;
        }
    }

    /// Escribe el manifest, o limpia la carpeta si no quedó nada para deshacer.
    pub(crate) async fn finish(self) {
        if !self.ready {
            return;
        }
        if self.entries.is_empty() {
            let _ = tokio::fs::remove_dir_all(&self.dir).await;
        } else {
            let manifest = RollbackManifest {
                created_at: unix_now(),
                entries: self.entries,
            };
            if let Ok(json) = serde_json::to_vec_pretty(&manifest) {
                let _ = tokio::fs::write(self.dir.join("manifest.json"), json).await;
            }
        }
    }
}

/// `None` si no hay una actualización que se pueda deshacer.
pub async fn rollback_info(instance_name: &str) -> Result<Option<ModRollbackInfo>, String> {
    let instance = instance_manager::get_instance(instance_name).await?;
    Ok(read_rollback_manifest(&rollback_dir(&instance.dir()))
        .await
        .map(|m| ModRollbackInfo {
            created_at: m.created_at,
            count: m.entries.len() as u32,
        }))
}

/// Deshace la última actualización: saca los archivos nuevos y devuelve los
/// jars viejos a `mods/`. Devuelve cuántos mods volvieron a su versión anterior.
pub async fn rollback_update(instance_name: &str) -> Result<u32, String> {
    let instance = instance_manager::get_instance(instance_name).await?;
    let instance_dir = instance.dir();
    restore_rollback(&rollback_dir(&instance_dir), &instance_dir.join("mods")).await
}

async fn restore_rollback(rb_dir: &Path, mods_dir: &Path) -> Result<u32, String> {
    let Some(manifest) = read_rollback_manifest(rb_dir).await else {
        return Err("No hay ninguna actualización para deshacer".into());
    };
    let mut restored = 0u32;
    for entry in manifest.entries {
        for new in &entry.new {
            if new != &entry.old {
                let _ = tokio::fs::remove_file(mods_dir.join(new)).await;
            }
        }
        if move_file(&rb_dir.join(&entry.old), &mods_dir.join(&entry.old))
            .await
            .is_ok()
        {
            restored += 1;
        }
    }
    let _ = tokio::fs::remove_dir_all(rb_dir).await;
    Ok(restored)
}

pub async fn discard_rollback(instance_name: &str) -> Result<(), String> {
    let instance = instance_manager::get_instance(instance_name).await?;
    let _ = tokio::fs::remove_dir_all(rollback_dir(&instance.dir())).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tfl-rb-test-{tag}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn manifest_format_is_stable() {
        let manifest = RollbackManifest {
            created_at: 1_700_000_000,
            entries: vec![RollbackEntry {
                old: "a-1.0.jar".into(),
                new: vec!["a-2.0.jar".into()],
            }],
        };
        let value = serde_json::to_value(&manifest).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "created_at": 1_700_000_000,
                "entries": [{"old": "a-1.0.jar", "new": ["a-2.0.jar"]}]
            })
        );
        let back: RollbackManifest = serde_json::from_value(value).unwrap();
        assert_eq!(back.entries, manifest.entries);
    }

    #[tokio::test]
    async fn update_then_rollback_restores_the_old_jars() {
        let base = temp_dir("roundtrip");
        let mods = base.join("mods");
        std::fs::create_dir_all(&mods).unwrap();
        let rb_dir = rollback_dir(&base);
        // Estado post-actualización: el jar nuevo ya está en mods/, el viejo
        // todavía también (lo retira el recorder).
        std::fs::write(mods.join("a-1.0.jar"), b"viejo").unwrap();
        std::fs::write(mods.join("a-2.0.jar"), b"nuevo").unwrap();

        let mut recorder = RollbackRecorder::begin(rb_dir.clone()).await;
        recorder
            .retire(&mods, "a-1.0.jar", vec!["a-2.0.jar".into()])
            .await;
        recorder.finish().await;

        assert!(!mods.join("a-1.0.jar").exists());
        assert!(rb_dir.join("a-1.0.jar").exists());
        let manifest = read_rollback_manifest(&rb_dir).await.unwrap();
        assert_eq!(manifest.entries.len(), 1);

        let restored = restore_rollback(&rb_dir, &mods).await.unwrap();
        assert_eq!(restored, 1);
        assert_eq!(std::fs::read(mods.join("a-1.0.jar")).unwrap(), b"viejo");
        assert!(!mods.join("a-2.0.jar").exists());
        assert!(!rb_dir.exists());
        let _ = std::fs::remove_dir_all(base);
    }

    #[tokio::test]
    async fn restoring_without_a_manifest_is_an_error() {
        let base = temp_dir("empty");
        let err = restore_rollback(&rollback_dir(&base), &base.join("mods"))
            .await
            .unwrap_err();
        assert!(err.contains("No hay ninguna actualización"));
        let _ = std::fs::remove_dir_all(base);
    }

    #[tokio::test]
    async fn nothing_retired_leaves_no_rollback_folder() {
        let base = temp_dir("none");
        let rb_dir = rollback_dir(&base);
        let recorder = RollbackRecorder::begin(rb_dir.clone()).await;
        recorder.finish().await;
        assert!(!rb_dir.exists());
        let _ = std::fs::remove_dir_all(base);
    }
}
