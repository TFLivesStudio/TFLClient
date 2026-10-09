//! Historial acotado de sesiones jugadas — antes solo se guardaba
//! `last_played` por instancia (el momento más reciente), sin rastro de
//! sesiones anteriores. Esto es lo que alimenta "Actividad reciente" en la
//! home. Mismo patrón de archivo que `commands::favorite_servers`: un JSON
//! chico en `settings_dir`, sin base de datos.

use crate::core::PathManager;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Cuántas entradas se guardan — es solo para mostrar "lo último que
/// jugaste", no un historial completo; sin tope crecería para siempre.
const MAX_ENTRIES: usize = 20;

/// Sesiones más cortas que esto no se registran: ruido de una instancia que
/// se abre y se cierra enseguida (prueba, crash instantáneo) no aporta nada
/// a "actividad reciente" y ensuciaría la lista.
const MIN_SESSION_SECS: u64 = 30;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ActivityEntry {
    pub id: String,
    pub instance_name: String,
    pub mc_version: String,
    pub loader: String,
    pub duration_secs: u64,
    pub ended_at: i64,
}

fn log_path() -> PathBuf {
    PathManager::get()
        .get_settings_dir()
        .join("activity_log.tfl")
}

async fn load() -> Vec<ActivityEntry> {
    match tokio::fs::read_to_string(log_path()).await {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

async fn save(list: &[ActivityEntry]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(list).map_err(|e| e.to_string())?;
    tokio::fs::write(log_path(), json)
        .await
        .map_err(|e| e.to_string())
}

/// Inserta `entry` al principio y recorta a `max` — separado de la carga/
/// guardado en disco para poder probarlo sin tocar el filesystem.
fn prepend_and_trim(
    mut list: Vec<ActivityEntry>,
    entry: ActivityEntry,
    max: usize,
) -> Vec<ActivityEntry> {
    list.insert(0, entry);
    list.truncate(max);
    list
}

/// Se llama al terminar una sesión de juego (ver `services::launcher`). Sin
/// efecto si la sesión fue más corta que `MIN_SESSION_SECS`.
pub async fn record(instance_name: String, mc_version: String, loader: String, duration_secs: u64) {
    if duration_secs < MIN_SESSION_SECS {
        return;
    }
    let entry = ActivityEntry {
        id: uuid::Uuid::new_v4().to_string(),
        instance_name,
        mc_version,
        loader,
        duration_secs,
        ended_at: now_secs(),
    };
    let list = prepend_and_trim(load().await, entry, MAX_ENTRIES);
    let _ = save(&list).await;
}

pub async fn recent(limit: usize) -> Vec<ActivityEntry> {
    let mut list = load().await;
    list.truncate(limit);
    list
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str) -> ActivityEntry {
        ActivityEntry {
            id: name.to_string(),
            instance_name: name.to_string(),
            mc_version: "1.21.1".into(),
            loader: "fabric".into(),
            duration_secs: 120,
            ended_at: 0,
        }
    }

    #[test]
    fn prepend_pone_lo_nuevo_primero() {
        let list = prepend_and_trim(vec![entry("vieja")], entry("nueva"), 20);
        assert_eq!(list[0].instance_name, "nueva");
        assert_eq!(list[1].instance_name, "vieja");
    }

    #[test]
    fn prepend_recorta_al_maximo() {
        let existing: Vec<_> = (0..5).map(|i| entry(&i.to_string())).collect();
        let list = prepend_and_trim(existing, entry("nueva"), 3);
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].instance_name, "nueva");
    }
}
