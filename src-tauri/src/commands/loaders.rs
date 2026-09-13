//! Resolución automática de la versión de loader "recomendada" para una
//! versión de Minecraft — el usuario elige Fabric/Forge/NeoForge/Quilt en
//! Crear Instancia, nunca una versión de loader a mano.
use crate::core::{get_json_retrying, get_text_retrying};
use aqua::{FabricBatch, QuiltBatch};
use serde::Deserialize;
use tauri::command;

#[command]
pub async fn get_fabric_loader(mc_version: String) -> Result<String, String> {
    FabricBatch::resolve_latest_loader(&mc_version)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_quilt_loader(mc_version: String) -> Result<String, String> {
    QuiltBatch::resolve_latest_loader(&mc_version)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
struct ForgePromotions {
    promos: std::collections::HashMap<String, String>,
}

#[command]
pub async fn get_forge_version(mc_version: String) -> Result<String, String> {
    let promos: ForgePromotions = get_json_retrying(
        "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json",
    )
    .await?;

    promos
        .promos
        .get(&format!("{mc_version}-recommended"))
        .or_else(|| promos.promos.get(&format!("{mc_version}-latest")))
        .cloned()
        .ok_or_else(|| format!("No hay build de Forge para Minecraft {mc_version}"))
}

/// NeoForge versiona como `<mc_minor>.<mc_patch>.<build>` (ej. MC 1.21.1 →
/// NeoForge 21.1.x). No hay API de "recomendado"; se toma el build más
/// alto listado en el maven que matchea ese prefijo.
#[command]
pub async fn get_neoforge_version(mc_version: String) -> Result<String, String> {
    let prefix = neoforge_prefix(&mc_version)
        .ok_or_else(|| format!("Minecraft {mc_version} no es compatible con NeoForge"))?;

    let xml = get_text_retrying(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml",
    )
    .await?;

    let mut candidates: Vec<String> = xml
        .split("<version>")
        .skip(1)
        .filter_map(|chunk| chunk.split("</version>").next())
        .map(str::to_string)
        .filter(|v| v.starts_with(&prefix) && !v.contains('-')) // sin beta/rc
        .collect();

    candidates.sort_by(|a, b| version_key(a).cmp(&version_key(b)));

    candidates
        .pop()
        .ok_or_else(|| format!("No hay build de NeoForge para Minecraft {mc_version}"))
}

fn neoforge_prefix(mc_version: &str) -> Option<String> {
    let rest = mc_version.strip_prefix("1.")?;
    let mut parts = rest.splitn(2, '.');
    let minor = parts.next()?;
    let patch = parts.next().unwrap_or("0");
    Some(format!("{minor}.{patch}."))
}

fn version_key(v: &str) -> Vec<u32> {
    v.split('.').filter_map(|p| p.parse().ok()).collect()
}
