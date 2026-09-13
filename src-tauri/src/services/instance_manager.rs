use crate::core::PathManager;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoaderKind {
    Vanilla,
    Fabric,
    Forge,
    NeoForge,
    Quilt,
}

impl LoaderKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "vanilla" => Some(Self::Vanilla),
            "fabric" => Some(Self::Fabric),
            "forge" => Some(Self::Forge),
            "neoforge" => Some(Self::NeoForge),
            "quilt" => Some(Self::Quilt),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceData {
    pub uuid: String,
    pub name: String,
    /// Versión base de Minecraft (ej. "1.21.1"), siempre la vanilla.
    pub mc_version: String,
    pub loader: LoaderKind,
    /// `None` para Vanilla. Guardado aparte de `launch_version_id` para no
    /// tener que reparsear el id combinado al relanzar.
    #[serde(default)]
    pub loader_version: Option<String>,
    /// Version id a lanzar — igual a `mc_version` en Vanilla, o el id
    /// combinado del loader (ej. "fabric-loader-0.15.11-1.21.1") en el
    /// resto. Se resuelve una vez al crear la instancia, no en cada launch,
    /// para no pegarle a las APIs de Fabric/Forge/NeoForge cada vez.
    pub launch_version_id: String,
    #[serde(default)]
    pub last_played: i64,
    #[serde(default)]
    pub min_memory: Option<u32>,
    #[serde(default)]
    pub max_memory: Option<u32>,
}

impl InstanceData {
    pub fn dir(&self) -> PathBuf {
        PathManager::get().get_instance_dir().join(&self.name)
    }

    fn manifest_path(&self) -> PathBuf {
        self.dir().join("instance.tfl.json")
    }

    pub async fn save(&self) -> std::io::Result<()> {
        tokio::fs::create_dir_all(self.dir()).await?;
        let json = serde_json::to_string_pretty(self)?;
        tokio::fs::write(self.manifest_path(), json).await
    }
}

fn valid_instance_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("El nombre de la instancia no puede estar vacío".into());
    }
    if trimmed.chars().any(|c| "/\\:*?\"<>|".contains(c)) {
        return Err("El nombre de la instancia tiene caracteres inválidos".into());
    }
    Ok(())
}

pub async fn create_instance(
    name: String,
    mc_version: String,
    loader: LoaderKind,
    loader_version: Option<String>,
    launch_version_id: String,
) -> Result<InstanceData, String> {
    valid_instance_name(&name)?;
    let data = InstanceData {
        uuid: Uuid::new_v4().to_string(),
        name,
        mc_version,
        loader,
        loader_version,
        launch_version_id,
        last_played: 0,
        min_memory: None,
        max_memory: None,
    };
    if data.dir().exists() {
        return Err("Ya existe una instancia con ese nombre".into());
    }
    data.save().await.map_err(|e| e.to_string())?;
    Ok(data)
}

pub async fn list_instances() -> Vec<InstanceData> {
    let dir = PathManager::get().get_instance_dir();
    let mut out = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return out;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let manifest = entry.path().join("instance.tfl.json");
        if let Ok(raw) = tokio::fs::read_to_string(&manifest).await
            && let Ok(data) = serde_json::from_str::<InstanceData>(&raw)
        {
            out.push(data);
        }
    }
    out
}

pub async fn get_instance(name: &str) -> Result<InstanceData, String> {
    let manifest = PathManager::get()
        .get_instance_dir()
        .join(name)
        .join("instance.tfl.json");
    let raw = tokio::fs::read_to_string(&manifest)
        .await
        .map_err(|_| "Instancia no encontrada".to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub async fn delete_instance(name: &str) -> Result<(), String> {
    let dir = PathManager::get().get_instance_dir().join(name);
    tokio::fs::remove_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())
}

pub async fn rename_instance(old_name: &str, new_name: String) -> Result<InstanceData, String> {
    valid_instance_name(&new_name)?;
    let mut data = get_instance(old_name).await?;
    if old_name == new_name {
        return Ok(data);
    }
    let new_dir = PathManager::get().get_instance_dir().join(&new_name);
    if new_dir.exists() {
        return Err("Ya existe una instancia con ese nombre".into());
    }
    tokio::fs::rename(data.dir(), &new_dir)
        .await
        .map_err(|e| e.to_string())?;
    data.name = new_name;
    data.save().await.map_err(|e| e.to_string())?;
    Ok(data)
}

pub async fn update_instance_memory(
    name: &str,
    min_memory: Option<u32>,
    max_memory: Option<u32>,
) -> Result<InstanceData, String> {
    let mut data = get_instance(name).await?;
    data.min_memory = min_memory;
    data.max_memory = max_memory;
    data.save().await.map_err(|e| e.to_string())?;
    Ok(data)
}

pub async fn mark_last_played(name: &str) -> Result<(), String> {
    let mut data = get_instance(name).await?;
    data.last_played = now_secs();
    data.save().await.map_err(|e| e.to_string())
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
