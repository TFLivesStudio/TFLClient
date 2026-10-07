//! Exportar una instancia a `.mrpack`.
//!
//! Los mods que resuelven contra el proveedor se referencian por URL (igual
//! que hace cualquier .mrpack real) — el export sale liviano, no va un jar de
//! 20MB adentro por cada mod. Todo lo que NO resuelve (jars de otro lado,
//! config, resourcepacks, shaderpacks) se empaqueta de verdad adentro del
//! zip, en overrides/ — así es como el formato .mrpack espera ese contenido.

use super::ContentService;
use super::filesystem::hash_installed_files;
use super::providers::ContentProvider;
use super::types::ContentVersion;
use crate::core::PathManager;
use crate::services::instance_manager;
use crate::services::zip_util::walkdir_files;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Serialize)]
struct MrpackIndexFile {
    path: String,
    hashes: HashMap<String, String>,
    env: HashMap<String, String>,
    downloads: Vec<String>,
    #[serde(rename = "fileSize")]
    file_size: u64,
}

#[derive(Serialize)]
struct MrpackIndex {
    game: String,
    #[serde(rename = "formatVersion")]
    format_version: u32,
    #[serde(rename = "versionId")]
    version_id: String,
    name: String,
    summary: Option<String>,
    files: Vec<MrpackIndexFile>,
    dependencies: HashMap<String, String>,
}

fn loader_dependency_key(loader: instance_manager::LoaderKind) -> Option<&'static str> {
    match loader {
        instance_manager::LoaderKind::Fabric => Some("fabric-loader"),
        instance_manager::LoaderKind::Forge => Some("forge"),
        instance_manager::LoaderKind::NeoForge => Some("neoforge"),
        instance_manager::LoaderKind::Quilt => Some("quilt-loader"),
        instance_manager::LoaderKind::Vanilla => None,
    }
}

/// Entrada del índice para un mod que resolvió contra el proveedor. `None`
/// si no se puede referenciar por URL con su hash (va embebido en overrides).
fn index_entry(filename: &str, sha1: &str, version: &ContentVersion) -> Option<MrpackIndexFile> {
    let file = version.files.iter().find(|f| f.has_sha1(sha1))?;
    let sha1 = file.sha1.clone()?;
    let mut hashes = HashMap::new();
    hashes.insert("sha1".to_string(), sha1);
    if let Some(sha512) = file.sha512.clone() {
        hashes.insert("sha512".to_string(), sha512);
    }
    Some(MrpackIndexFile {
        path: format!("mods/{filename}"),
        hashes,
        env: HashMap::from([
            ("client".to_string(), "required".to_string()),
            ("server".to_string(), "required".to_string()),
        ]),
        downloads: vec![file.url.clone()],
        file_size: file.size.unwrap_or(0),
    })
}

/// Nombre de archivo seguro a partir del nombre de la instancia.
fn safe_export_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Agrega una carpeta entera al zip bajo `overrides/<nombre de la carpeta>`
/// — sync, corre dentro de spawn_blocking. Si la carpeta no existe (ej. la
/// instancia no tiene shaderpacks), no hace nada, no es un error.
fn add_override_dir(
    zip: &mut zip::ZipWriter<std::fs::File>,
    source: &std::path::Path,
    zip_prefix: &str,
) -> std::io::Result<()> {
    if !source.is_dir() {
        return Ok(());
    }
    for entry in walkdir_files(source) {
        let rel = entry.strip_prefix(source).unwrap_or(&entry);
        let zip_path = format!("{zip_prefix}/{}", rel.to_string_lossy().replace('\\', "/"));
        let bytes = std::fs::read(&entry)?;
        zip.start_file(zip_path, zip::write::SimpleFileOptions::default())?;
        std::io::Write::write_all(zip, &bytes)?;
    }
    Ok(())
}

fn build_mrpack_zip(
    instance_dir: &std::path::Path,
    mods_dir: &std::path::Path,
    unresolved_mod_filenames: &HashSet<String>,
    index: &MrpackIndex,
    export_path: &std::path::Path,
) -> std::io::Result<()> {
    if let Some(parent) = export_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(export_path)?;
    let mut zip = zip::ZipWriter::new(file);

    let index_json =
        serde_json::to_vec_pretty(index).map_err(|e| std::io::Error::other(e.to_string()))?;
    zip.start_file(
        "modrinth.index.json",
        zip::write::SimpleFileOptions::default(),
    )?;
    std::io::Write::write_all(&mut zip, &index_json)?;

    // Mods que no resolvieron contra el proveedor — van embebidos de verdad,
    // no hay URL pública de donde bajarlos después.
    for filename in unresolved_mod_filenames {
        let path = mods_dir.join(filename);
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        zip.start_file(
            format!("overrides/mods/{filename}"),
            zip::write::SimpleFileOptions::default(),
        )?;
        std::io::Write::write_all(&mut zip, &bytes)?;
    }

    for subdir in ["config", "resourcepacks", "shaderpacks"] {
        add_override_dir(
            &mut zip,
            &instance_dir.join(subdir),
            &format!("overrides/{subdir}"),
        )?;
    }

    zip.finish()?;
    Ok(())
}

impl<P: ContentProvider> ContentService<P> {
    /// Exporta la instancia como un .mrpack real y portable — cualquier
    /// launcher compatible con el formato (incluido este) puede instalarlo.
    /// Devuelve la ruta del archivo generado.
    pub async fn export_mrpack(&self, instance_name: &str) -> Result<String, String> {
        let instance = instance_manager::get_instance(instance_name).await?;
        let instance_dir = instance.dir();
        let mods_dir = instance_dir.join("mods");

        let hashed = hash_installed_files(&mods_dir, "jar").await;
        let resolved: HashMap<String, ContentVersion> = if hashed.is_empty() {
            Default::default()
        } else {
            let hashes: Vec<String> = hashed.iter().map(|(_, h)| h.clone()).collect();
            self.provider
                .versions_from_hashes(&hashes)
                .await
                .unwrap_or_default()
        };

        let mut index_files = Vec::new();
        let mut unresolved = HashSet::new();

        for (filename, hash) in &hashed {
            let entry = resolved
                .get(hash)
                .and_then(|version| index_entry(filename, hash, version));
            match entry {
                Some(entry) => index_files.push(entry),
                None => {
                    unresolved.insert(filename.clone());
                }
            }
        }

        let mut dependencies = HashMap::new();
        dependencies.insert("minecraft".to_string(), instance.mc_version.clone());
        if let (Some(loader_version), Some(key)) = (
            &instance.loader_version,
            loader_dependency_key(instance.loader),
        ) {
            dependencies.insert(key.to_string(), loader_version.clone());
        }

        let export_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let index = MrpackIndex {
            game: "minecraft".into(),
            format_version: 1,
            version_id: format!("tfl-export-{export_id}"),
            name: instance.name.clone(),
            summary: None,
            files: index_files,
            dependencies,
        };

        let safe_name = safe_export_name(&instance.name);
        let export_path = PathManager::get()
            .get_shared_dir()
            .join("exports")
            .join(format!("{safe_name}-{export_id}.mrpack"));

        let instance_dir_clone = instance_dir.clone();
        let mods_dir_clone = mods_dir.clone();
        let export_path_clone = export_path.clone();
        tokio::task::spawn_blocking(move || {
            build_mrpack_zip(
                &instance_dir_clone,
                &mods_dir_clone,
                &unresolved,
                &index,
                &export_path_clone,
            )
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

        Ok(export_path.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::content::types::ContentFile;

    fn version_with(file: ContentFile) -> ContentVersion {
        ContentVersion {
            id: "v".into(),
            project_id: "p".into(),
            name: "n".into(),
            version_number: String::new(),
            date_published: String::new(),
            changelog: None,
            files: vec![file],
            dependencies: vec![],
        }
    }

    fn file(sha1: Option<&str>, sha512: Option<&str>) -> ContentFile {
        ContentFile {
            url: "https://cdn.example/a.jar".into(),
            filename: "a.jar".into(),
            primary: true,
            sha1: sha1.map(String::from),
            sha512: sha512.map(String::from),
            size: Some(10),
        }
    }

    #[test]
    fn resolved_mod_is_referenced_by_url_and_hashes() {
        let v = version_with(file(Some("aaa"), Some("bbb")));
        let entry = index_entry("a.jar", "aaa", &v).unwrap();
        assert_eq!(entry.path, "mods/a.jar");
        assert_eq!(entry.hashes["sha1"], "aaa");
        assert_eq!(entry.hashes["sha512"], "bbb");
        assert_eq!(entry.downloads, vec!["https://cdn.example/a.jar"]);
        assert_eq!(entry.file_size, 10);
    }

    #[test]
    fn mod_whose_hash_is_not_in_the_version_is_embedded_instead() {
        let v = version_with(file(Some("aaa"), None));
        assert!(index_entry("a.jar", "otro", &v).is_none());
    }

    #[test]
    fn export_name_keeps_only_safe_characters() {
        assert_eq!(safe_export_name("Mi Pack: v1/2"), "Mi_Pack__v1_2");
        assert_eq!(safe_export_name("ok-name_1"), "ok-name_1");
    }
}
