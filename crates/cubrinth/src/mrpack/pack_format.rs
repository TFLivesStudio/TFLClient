use serde::Deserialize;
use std::collections::HashMap;
use zellkern::{GameVersion, Loader};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackFormat {
    pub game: String,
    #[allow(dead_code)]
    pub format_version: i32,
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<PackFile>,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackFile {
    pub path: String,
    #[allow(dead_code)]
    pub hashes: HashMap<String, String>,
    pub env: Option<HashMap<String, String>>,
    pub downloads: Vec<String>,
    #[allow(dead_code)]
    pub file_size: u32,
}

#[derive(Debug, Clone)]
pub struct MrpackMetadata {
    pub name: String,
    pub version_id: String,
    pub summary: Option<String>,
    pub game_version: Option<GameVersion>,
    pub file_count: usize,
    /// Rutas relativas a `instance_dir` de cada archivo que
    /// `install_mrpack` escribió (mods declarados + overrides) — vacío
    /// para `parse_mrpack`, que solo lee metadata sin instalar nada.
    /// Permite desinstalar el modpack más tarde borrando exactamente lo
    /// que puso, sin adivinar.
    pub installed_paths: Vec<String>,
    /// Claves de `dependencies` que no son Minecraft ni un loader conocido
    /// (`fabric-loader`/`quilt-loader`/`forge`/`neoforge`) — ej. loaders
    /// legacy que este motor no instala. Vacío en un pack normal. Permite
    /// avisar "loader no soportado" en vez de crear una instancia vanilla
    /// a la que le faltan los mods.
    pub unsupported_dependencies: Vec<String>,
}

const KNOWN_DEPENDENCIES: &[&str] = &[
    "minecraft",
    "fabric-loader",
    "quilt-loader",
    "forge",
    "neoforge",
];

impl PackFormat {
    pub fn extract_metadata(&self) -> MrpackMetadata {
        let mc_version = self.dependencies.get("minecraft").cloned();

        let game_version = mc_version.map(|mc_ver| {
            let loader = self
                .dependencies
                .get("quilt-loader")
                .map(|v| Loader::Quilt(v.clone()))
                .or_else(|| {
                    self.dependencies
                        .get("fabric-loader")
                        .map(|v| Loader::Fabric(v.clone()))
                })
                .or_else(|| {
                    self.dependencies
                        .get("forge")
                        .map(|v| Loader::Forge(v.clone()))
                })
                .or_else(|| {
                    self.dependencies
                        .get("neoforge")
                        .map(|v| Loader::NeoForge(v.clone()))
                })
                .unwrap_or(Loader::Vanilla);

            GameVersion {
                mc_version: mc_ver,
                loader,
            }
        });

        MrpackMetadata {
            name: self.name.clone(),
            version_id: self.version_id.clone(),
            summary: self.summary.clone(),
            game_version,
            file_count: self.files.len(),
            installed_paths: Vec::new(),
            unsupported_dependencies: {
                let mut v: Vec<String> = self
                    .dependencies
                    .keys()
                    .filter(|k| !KNOWN_DEPENDENCIES.contains(&k.as_str()))
                    .cloned()
                    .collect();
                v.sort();
                v
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(deps: &[(&str, &str)]) -> PackFormat {
        PackFormat {
            game: "minecraft".into(),
            format_version: 1,
            version_id: "1.0.0".into(),
            name: "Test".into(),
            summary: None,
            files: Vec::new(),
            dependencies: deps
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn known_loader_has_no_unsupported_dependencies() {
        let m = pack(&[("minecraft", "1.20.1"), ("fabric-loader", "0.15.11")]).extract_metadata();
        assert!(m.unsupported_dependencies.is_empty());
        assert!(matches!(
            m.game_version.unwrap().loader,
            Loader::Fabric(v) if v == "0.15.11"
        ));
    }

    #[test]
    fn unknown_dependency_is_reported() {
        let m = pack(&[("minecraft", "1.12.2"), ("legacy-fabric", "0.1")]).extract_metadata();
        assert_eq!(m.unsupported_dependencies, vec!["legacy-fabric"]);
    }
}
