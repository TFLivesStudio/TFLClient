//! Operaciones sobre los archivos de contenido dentro de una instancia:
//! listar, borrar, apagar/prender (`.disabled`), hashear. No saben nada del
//! proveedor.

use super::types::ContentKind;
use crate::services::instance_manager;

/// Sufijo con el que se "apaga" un mod/plugin sin borrarlo: ni los loaders
/// de cliente ni Paper/Purpur cargan un archivo que no termine en `.jar`.
pub(crate) const DISABLED_SUFFIX: &str = ".disabled";

pub(crate) fn is_disabled_file(filename: &str) -> bool {
    filename.ends_with(DISABLED_SUFFIX)
}

/// `-sources.jar`, `-javadoc.jar` y `-dev.jar` — archivos que el proveedor
/// adjunta junto al jar real y que nunca deben instalarse como mod.
pub(crate) fn is_auxiliary_jar(filename: &str) -> bool {
    let n = filename.to_ascii_lowercase();
    let n = n.strip_suffix(DISABLED_SUFFIX).unwrap_or(&n);
    n.ends_with("-sources.jar") || n.ends_with("-javadoc.jar") || n.ends_with("-dev.jar")
}

/// Un nombre de archivo que viene de afuera (frontend, API del proveedor) y
/// termina en una ruta: no puede traer separadores ni `..`.
pub(crate) fn is_unsafe_filename(filename: &str) -> bool {
    filename.contains('/') || filename.contains('\\') || filename.contains("..")
}

pub(crate) fn sha1_hex(bytes: &[u8]) -> String {
    use sha1::{Digest, Sha1};
    Sha1::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub async fn list_files(instance_name: &str, kind: ContentKind) -> Result<Vec<String>, String> {
    let instance = instance_manager::get_instance(instance_name).await?;
    let dir = instance.dir().join(kind.subdir());
    let mut out = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
        return Ok(out);
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Some(name) = entry.file_name().to_str() {
            out.push(name.to_string());
        }
    }
    Ok(out)
}

pub async fn remove_file(
    instance_name: &str,
    kind: ContentKind,
    filename: &str,
) -> Result<(), String> {
    if is_unsafe_filename(filename) {
        return Err("Nombre de archivo inválido".into());
    }
    let instance = instance_manager::get_instance(instance_name).await?;
    let path = instance.dir().join(kind.subdir()).join(filename);
    match tokio::fs::remove_file(&path).await {
        Ok(()) => Ok(()),
        // Si el archivo ya no está (lo borraron a mano de la carpeta con el
        // launcher abierto), el resultado que pidió el usuario ya se cumple:
        // devolver error dejaba la entrada fantasma en la lista para siempre.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Nombre de archivo resultante de apagar (`enabled == false`, agrega
/// `.disabled`) o prender (lo saca) un archivo.
fn toggled_name(filename: &str, enabled: bool) -> String {
    if enabled {
        filename
            .strip_suffix(DISABLED_SUFFIX)
            .unwrap_or(filename)
            .to_string()
    } else if is_disabled_file(filename) {
        filename.to_string()
    } else {
        format!("{filename}{DISABLED_SUFFIX}")
    }
}

/// Apaga o prende un archivo renombrándolo `x.jar` ↔ `x.jar.disabled`.
/// Devuelve el nombre de archivo resultante.
pub async fn set_enabled(
    instance_name: &str,
    kind: ContentKind,
    filename: &str,
    enabled: bool,
) -> Result<String, String> {
    if is_unsafe_filename(filename) {
        return Err("Nombre de archivo inválido".into());
    }
    let instance = instance_manager::get_instance(instance_name).await?;
    let dir = instance.dir().join(kind.subdir());
    let new_name = toggled_name(filename, enabled);
    if new_name == filename {
        return Ok(new_name);
    }
    if dir.join(&new_name).exists() {
        return Err(format!("Ya existe un archivo llamado \"{new_name}\""));
    }
    tokio::fs::rename(dir.join(filename), dir.join(&new_name))
        .await
        .map_err(|e| e.to_string())?;
    Ok(new_name)
}

/// rename y, si cruza de disco o falla, copia + borra.
pub(crate) async fn move_file(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
    if tokio::fs::rename(from, to).await.is_ok() {
        return Ok(());
    }
    tokio::fs::copy(from, to).await?;
    tokio::fs::remove_file(from).await
}

/// (filename, sha1) de cada archivo con la extensión dada en `dir` — no
/// recursivo, mods/shaderpacks/resourcepacks viven todos sueltos ahí, no en
/// subcarpetas.
pub(crate) async fn hash_installed_files(
    dir: &std::path::Path,
    ext: &str,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return out;
    };
    let enabled_suffix = format!(".{ext}");
    let disabled_suffix = format!(".{ext}{DISABLED_SUFFIX}");
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        let Some(filename) = path.file_name().and_then(|f| f.to_str()) else {
            continue;
        };
        // También los desactivados ("x.jar.disabled") — siguen siendo parte
        // de lo instalado, el usuario tiene que poder verlos y reactivarlos.
        if !filename.ends_with(&enabled_suffix) && !filename.ends_with(&disabled_suffix) {
            continue;
        }
        let Ok(bytes) = tokio::fs::read(&path).await else {
            continue;
        };
        out.push((filename.to_string(), sha1_hex(&bytes)));
    }
    out
}

pub(crate) fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auxiliary_jars_are_detected() {
        assert!(is_auxiliary_jar("embeddium-0.1.5+mc1.20.1-sources.jar"));
        assert!(is_auxiliary_jar("Mod-1.0-JAVADOC.jar"));
        assert!(is_auxiliary_jar("mod-1.0-dev.jar"));
        assert!(is_auxiliary_jar("mod-1.0-sources.jar.disabled"));
        assert!(!is_auxiliary_jar("mod-1.0.jar"));
        assert!(!is_auxiliary_jar("developers-toolkit-1.0.jar"));
    }

    #[test]
    fn disabled_suffix_detection() {
        assert!(is_disabled_file("x.jar.disabled"));
        assert!(!is_disabled_file("x.jar"));
    }

    #[test]
    fn toggling_adds_and_removes_the_disabled_suffix() {
        assert_eq!(toggled_name("x.jar", false), "x.jar.disabled");
        assert_eq!(toggled_name("x.jar.disabled", false), "x.jar.disabled");
        assert_eq!(toggled_name("x.jar.disabled", true), "x.jar");
        assert_eq!(toggled_name("x.jar", true), "x.jar");
    }

    #[test]
    fn unsafe_filenames_are_rejected() {
        assert!(is_unsafe_filename("../x.jar"));
        assert!(is_unsafe_filename("a/b.jar"));
        assert!(is_unsafe_filename("a\\b.jar"));
        assert!(!is_unsafe_filename("mod-1.0.jar"));
    }

    #[test]
    fn sha1_hex_matches_known_vector() {
        assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
    }

    #[tokio::test]
    async fn hashing_includes_disabled_files_and_skips_other_extensions() {
        let dir = std::env::temp_dir().join(format!("tfl-fs-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.jar"), b"abc").unwrap();
        std::fs::write(dir.join("b.jar.disabled"), b"abc").unwrap();
        std::fs::write(dir.join("c.txt"), b"abc").unwrap();
        let mut hashed = hash_installed_files(&dir, "jar").await;
        hashed.sort();
        assert_eq!(hashed.len(), 2);
        assert_eq!(hashed[0].0, "a.jar");
        assert_eq!(hashed[1].0, "b.jar.disabled");
        assert_eq!(hashed[0].1, "a9993e364706816aba3e25717850c26c9cd0d89d");
        let _ = std::fs::remove_dir_all(dir);
    }
}
