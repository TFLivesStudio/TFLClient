use crate::core::get_json_retrying;
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Deserialize)]
struct RawManifest {
    versions: Vec<RawEntry>,
}

#[derive(Debug, Deserialize)]
struct RawEntry {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(rename = "releaseTime")]
    release_time: String,
}

#[derive(Debug, Serialize)]
pub struct MinecraftVersion {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub release_time: String,
}

#[command]
pub async fn get_available_versions() -> Result<Vec<MinecraftVersion>, String> {
    let raw: RawManifest = get_json_retrying(aqua::MOJANG_MANIFEST_URL).await?;

    Ok(raw
        .versions
        .into_iter()
        .map(|v| MinecraftVersion {
            id: v.id,
            kind: v.kind,
            release_time: v.release_time,
        })
        .collect())
}
