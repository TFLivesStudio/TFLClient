//! "TFL Selection" — modpacks curados por TFLives, listados acá para que
//! el usuario los instale en cualquier instancia compatible con un solo
//! click. TFLives todavía no entregó su lista final de modpacks propios
//! (spec §26) — la entrada de abajo es un placeholder real y funcional
//! (un modpack público y estable de Modrinth) para que la feature se
//! pueda usar y probar ya mismo; reemplazar por los project_id reales de
//! TFLives cuando estén.
use serde::Serialize;
use tauri::command;

#[derive(Debug, Serialize, Clone)]
pub struct TflSelectionEntry {
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
}

#[command]
pub fn get_tfl_selection() -> Vec<TflSelectionEntry> {
    vec![
        // TODO(TFL): reemplazar por la lista real de modpacks de TFLives.
        TflSelectionEntry {
            project_id: "1KVo5zza".into(), // Fabulously Optimized (Modrinth)
            title: "Fabulously Optimized".into(),
            description: "Rendimiento y calidad de vida para Fabric, sin cambiar el juego base."
                .into(),
            icon_url: None,
        },
    ]
}
