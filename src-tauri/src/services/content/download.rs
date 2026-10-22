//! Descarga de los archivos de una versión a la carpeta de la instancia.
//! Provider-agnóstico: trabaja sobre `ContentVersion`/`ContentFile`.

use super::filesystem::{is_auxiliary_jar, is_unsafe_filename};
use super::types::{ContentFile, ContentKind, ContentVersion};
use crate::core::{AppEvent, emit};
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
    let expected = file
        .sha1
        .as_ref()
        .filter(|s| s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or("Missing content integrity hash")?;
    if dest.exists() {
        if let Ok(hash) = hash_file(&dest).await {
            if hash == *expected {
                return Ok(());
            }
        }
    }
    let mut last_err = String::new();
    for _attempt in 1..=3u8 {
        let tmp = dest_dir.join(format!(".tfl-download-{}", uuid::Uuid::new_v4()));
        let result = stream_content_download(file, &tmp).await;
        match result {
            Ok(actual) if actual == *expected => {
                if let Err(e) = tokio::fs::rename(&tmp, &dest).await {
                    let _ = tokio::fs::remove_file(&tmp).await;
                    return Err(e.to_string());
                }
                return Ok(());
            }
            Ok(_) => last_err = "Content SHA1 mismatch".into(),
            Err(e) => last_err = e,
        }
        let _ = tokio::fs::remove_file(&tmp).await;
    }
    Err(format!("Content download failed: {last_err}"))
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

fn allowed_content_url(raw: &str) -> bool {
    url::Url::parse(raw).is_ok_and(|u| {
        u.scheme() == "https"
            && u.host_str() == Some("cdn.modrinth.com")
            && u.port_or_known_default() == Some(443)
            && u.username().is_empty()
            && u.password().is_none()
    })
}
async fn hash_file(path: &std::path::Path) -> Result<String, String> {
    use sha1::{Digest, Sha1};
    use tokio::io::AsyncReadExt;
    let mut f = tokio::fs::File::open(path)
        .await
        .map_err(|_| "File unavailable")?;
    let mut buffer = [0u8; 65536];
    let mut hash = Sha1::new();
    loop {
        let count = f.read(&mut buffer).await.map_err(|_| "File read failed")?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hex::encode(hash.finalize()))
}
async fn stream_content_download(
    file: &ContentFile,
    tmp: &std::path::Path,
) -> Result<String, String> {
    use sha1::{Digest, Sha1};
    use tokio::io::AsyncWriteExt;
    const MAX: u64 = 512 * 1024 * 1024;
    if !allowed_content_url(&file.url) || file.size.is_some_and(|s| s > MAX) {
        return Err("Untrusted content origin or content exceeds 512 MiB".into());
    }
    static CLIENT: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(180))
            .redirect(reqwest::redirect::Policy::custom(|a| {
                if a.previous().len() < 3 && allowed_content_url(a.url().as_str()) {
                    a.follow()
                } else {
                    a.stop()
                }
            }))
            .build()
            .expect("content HTTP client")
    });
    let mut response = CLIENT
        .get(&file.url)
        .send()
        .await
        .map_err(|_| "Content download failed")?
        .error_for_status()
        .map_err(|_| "Content provider HTTP error")?;
    if !allowed_content_url(response.url().as_str())
        || response.content_length().is_some_and(|s| s > MAX)
    {
        return Err("Content response rejected".into());
    }
    let mut output = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(tmp)
        .await
        .map_err(|_| "Cannot create content staging file")?;
    let mut received = 0u64;
    let mut hash = Sha1::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "Content stream failed")?
    {
        received += chunk.len() as u64;
        if received > MAX {
            return Err("Content exceeds download limit".into());
        }
        hash.update(&chunk);
        output
            .write_all(&chunk)
            .await
            .map_err(|_| "Content write failed")?;
    }
    output.flush().await.map_err(|_| "Content flush failed")?;
    if file.size.is_some_and(|s| s != received) {
        return Err("Content size mismatch".into());
    }
    Ok(hex::encode(hash.finalize()))
}

#[cfg(test)]
mod origin_tests {
    use super::*;
    #[test]
    fn refuse_ssrf_and_credentials() {
        for u in [
            "http://cdn.modrinth.com/x",
            "https://127.0.0.1/a",
            "https://cdn.modrinth.com.evil/a",
            "https://u:p@cdn.modrinth.com/a",
            "https://cdn.modrinth.com:444/a",
        ] {
            assert!(!allowed_content_url(u));
        }
        assert!(allowed_content_url(
            "https://cdn.modrinth.com/data/abc/versions/v/mod.jar"
        ));
    }
}
