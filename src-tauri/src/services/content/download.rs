//! Descarga de los archivos de una versión a la carpeta de la instancia.
//! Provider-agnóstico: trabaja sobre `ContentVersion`/`ContentFile`.

use super::filesystem::{is_auxiliary_jar, is_unsafe_filename, sha1_hex};
use super::types::{ContentFile, ContentKind, ContentVersion};
use crate::core::{AppEvent, emit, get_bytes_retrying};
use crate::services::instance_manager;

pub(crate) async fn download_single_file(
    dest_dir: &std::path::Path,
    file: &ContentFile,
) -> Result<(), String> {
    if is_unsafe_filename(&file.filename) {
        return Err(format!(
            "Nombre de archivo inválido recibido del proveedor: {}",
            file.filename
        ));
    }

    let dest = dest_dir.join(&file.filename);
    if dest.exists() {
        return Ok(());
    }

    let expected_sha1 = file.sha1.as_ref();

    // El motor `aqua` reintenta y valida hash para todo lo demás (Vanilla,
    // Fabric, Forge…) — acá era una escritura ciega sin checksum, así que
    // una descarga truncada por un corte de red quedaba instalada como si
    // nada.
    let mut last_err = String::new();
    for attempt in 1..=3u8 {
        let bytes = get_bytes_retrying(&file.url).await?;
        if let Some(expected) = expected_sha1 {
            let actual = sha1_hex(&bytes);
            if &actual != expected {
                last_err =
                    format!("hash SHA1 no coincide (esperado {expected}, obtenido {actual})");
                tracing::warn!(
                    "Descarga de {} corrupta en intento {attempt}/3: {last_err}",
                    file.filename
                );
                continue;
            }
        }
        tokio::fs::write(&dest, &bytes)
            .await
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    Err(format!(
        "No se pudo descargar {} de forma íntegra: {last_err}",
        file.filename
    ))
}

/// Qué archivos de la versión se instalan de verdad: el proveedor suele
/// adjuntar en la misma versión el `-sources.jar` (código fuente) o
/// `-javadoc.jar` junto al jar real — instalados en /mods, el loader los
/// lee como un segundo mod duplicado y la lista muestra cada mod dos veces.
/// Se bajan solo los archivos que son el mod de verdad.
pub(crate) fn files_to_install(files: &[ContentFile]) -> Vec<&ContentFile> {
    let wanted: Vec<&ContentFile> = files
        .iter()
        .filter(|f| f.primary || !is_auxiliary_jar(&f.filename))
        .collect();
    // Si el filtro dejara la lista vacía (versión con solo archivos auxiliares),
    // mejor bajar lo que haya que no instalar nada.
    if wanted.is_empty() {
        files.iter().collect()
    } else {
        wanted
    }
}

/// Baja TODOS los archivos que trae la versión, no solo el "primary" — la
/// mayoría de las versiones traen uno solo, pero algunas empaquetan varios
/// en la misma versión (el mod + una lib que necesita sí o sí) y antes esto
/// se perdía en silencio, solo se bajaba el primero.
pub(crate) async fn download_version_files(
    instance_name: &str,
    version: &ContentVersion,
    kind: ContentKind,
) -> Result<Vec<String>, String> {
    if version.files.is_empty() {
        return Err("No tiene archivos para descargar".into());
    }

    let instance = instance_manager::get_instance(instance_name).await?;
    let dest_dir = instance.dir().join(kind.subdir());
    tokio::fs::create_dir_all(&dest_dir)
        .await
        .map_err(|e| e.to_string())?;

    let mut filenames = Vec::new();
    for file in files_to_install(&version.files) {
        download_single_file(&dest_dir, file).await?;
        filenames.push(file.filename.clone());
    }

    emit(AppEvent::DownloadFinished {
        task: format!("{}:{}", kind.subdir(), version.name),
    });
    Ok(filenames)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(name: &str, primary: bool) -> ContentFile {
        ContentFile {
            url: format!("https://example.invalid/{name}"),
            filename: name.into(),
            primary,
            sha1: None,
            sha512: None,
            size: None,
        }
    }

    fn names(files: &[&ContentFile]) -> Vec<String> {
        files.iter().map(|f| f.filename.clone()).collect()
    }

    #[test]
    fn auxiliary_files_are_never_installed_next_to_the_real_jar() {
        let files = vec![
            file("mod-1.0.jar", true),
            file("mod-1.0-sources.jar", false),
            file("mod-1.0-javadoc.jar", false),
            file("mod-1.0-dev.jar", false),
            file("mod-lib-1.0.jar", false),
        ];
        assert_eq!(
            names(&files_to_install(&files)),
            vec!["mod-1.0.jar", "mod-lib-1.0.jar"]
        );
    }

    #[test]
    fn a_primary_file_is_kept_even_if_it_looks_auxiliary() {
        let files = vec![file("odd-dev.jar", true)];
        assert_eq!(names(&files_to_install(&files)), vec!["odd-dev.jar"]);
    }

    #[test]
    fn only_auxiliary_files_falls_back_to_downloading_them() {
        let files = vec![
            file("mod-sources.jar", false),
            file("mod-javadoc.jar", false),
        ];
        assert_eq!(
            names(&files_to_install(&files)),
            vec!["mod-sources.jar", "mod-javadoc.jar"]
        );
    }

    #[tokio::test]
    async fn unsafe_filenames_from_the_provider_are_refused() {
        let dir = std::env::temp_dir();
        let err = download_single_file(&dir, &file("../evil.jar", true))
            .await
            .unwrap_err();
        assert!(err.contains("Nombre de archivo inválido"));
    }
}
