use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use log::info;
use serde::Deserialize;

use super::batch::{DownloadBatch, DownloadItemSpec};
use crate::AquaError;
use crate::progress::DownloadStage;
use crate::utilities::fetch_text_retrying;

#[derive(Deserialize)]
#[allow(dead_code)]
struct OrnitheProfile {
    id: String,
    libraries: Vec<OrnitheLibrary>,
}

#[derive(Deserialize)]
struct OrnitheLibrary {
    name: String,
    url: String,
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

        let mut profile_text = fetch_text_retrying(&profile_url)
            .await
            .map_err(|e| AquaError::Other(format!("No se pudo conectar a meta.ornithemc.net: {e}")))?;

        // En meta.ornithemc.net el inheritsFrom es "{game_version}-vanilla", pero
        // el launcher almacena la versión base como "{game_version}". Lo adaptamos
        // para que zellkern resuelva el manifiesto padre de Mojang correctamente.
        let vanilla_pattern = format!("\"{}-vanilla\"", game_version);
        let replacement = format!("\"{}\"", game_version);
        profile_text = profile_text.replace(&vanilla_pattern, &replacement);

        let profile: OrnitheProfile = serde_json::from_str(&profile_text)
            .map_err(|e| AquaError::Other(format!("Error analizando perfil Ornithe: {e}")))?;

        let version_id = profile.id.clone();
        info!(
            "Ornithe Gen2 perfil cargado: {} librerías para {}",
            profile.libraries.len(),
            version_id
        );

        let lib_base_dir = shared_dir.join("libraries");
        let mut items = Vec::with_capacity(profile.libraries.len());

        for lib in &profile.libraries {
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

        Box::pin(async move {
            if json_path.exists() {
                return Ok(());
            }
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
