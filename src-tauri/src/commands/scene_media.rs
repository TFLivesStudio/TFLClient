//! Managed local videos. No network fetch, codec installation or arbitrary asset URLs.
use crate::core::PathManager;
use crate::services::SettingsManager;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tokio::io::AsyncReadExt;

const MAX_BYTES: u64 = 128 * 1024 * 1024;
const MAX_FILES: usize = 12;
static MEDIA_MUTATION: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
fn directory() -> std::path::PathBuf {
    PathManager::get()
        .get_shared_dir()
        .join("appearance")
        .join("scenes")
}
#[derive(Debug, serde::Serialize)]
pub struct SceneMedia {
    pub id: String,
    pub path: String,
    pub bytes: u64,
    pub mime: String,
}
fn valid_id(id: &str) -> bool {
    id.strip_prefix("scene-")
        .is_some_and(|s| (10..=40).contains(&s.len()) && s.bytes().all(|c| c.is_ascii_hexdigit()))
}
fn video_format(ext: &str, bytes: &[u8]) -> bool {
    match ext {
        "mp4" => bytes.get(4..8) == Some(b"ftyp"),
        "webm" => bytes.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]),
        _ => false,
    }
}
#[tauri::command]
pub async fn pick_scene_video_file(app: AppHandle) -> Option<String> {
    if SettingsManager::read().native_dialog_mode != "manual" {
        return None;
    }
    app.dialog()
        .file()
        .add_filter("Video (MP4 / WebM)", &["mp4", "webm"])
        .blocking_pick_file()
        .and_then(|f| f.into_path().ok())
        .map(|p| p.to_string_lossy().to_string())
}
#[tauri::command]
pub async fn list_scene_media() -> Result<Vec<SceneMedia>, String> {
    list_at(&directory()).await
}
async fn list_at(dir: &std::path::Path) -> Result<Vec<SceneMedia>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = tokio::fs::read_dir(dir).await.map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let id = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if !valid_id(id) || !["mp4", "webm"].contains(&ext) {
            continue;
        }
        let metadata = entry.metadata().await.map_err(|e| e.to_string())?;
        if !metadata.is_file() {
            continue;
        }
        out.push(SceneMedia {
            id: id.into(),
            path: path.to_string_lossy().to_string(),
            bytes: metadata.len(),
            mime: format!("video/{ext}"),
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}
#[tauri::command]
pub async fn import_scene_video(source_path: String) -> Result<SceneMedia, String> {
    import_into(source_path, directory()).await
}
async fn import_into(source_path: String, dir: std::path::PathBuf) -> Result<SceneMedia, String> {
    let _guard = MEDIA_MUTATION.lock().await;
    let source = std::path::PathBuf::from(source_path);
    let ext = source
        .extension()
        .and_then(|s| s.to_str())
        .map(str::to_ascii_lowercase)
        .filter(|s| ["mp4", "webm"].contains(&s.as_str()))
        .ok_or("Usá un vídeo MP4 o WebM")?;
    let metadata = tokio::fs::metadata(&source)
        .await
        .map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() < 16 || metadata.len() > MAX_BYTES {
        return Err("El vídeo debe pesar entre 16 bytes y 128 MB".into());
    }
    let mut header = [0u8; 16];
    tokio::fs::File::open(&source)
        .await
        .map_err(|e| e.to_string())?
        .read_exact(&mut header)
        .await
        .map_err(|e| e.to_string())?;
    if !video_format(&ext, &header) {
        return Err("El contenido del archivo no coincide con MP4/WebM".into());
    }
    if list_at(&dir).await?.len() >= MAX_FILES {
        return Err(
            "La biblioteca admite hasta 12 vídeos. Quitá uno antes de importar otro.".into(),
        );
    }
    let id = format!(
        "scene-{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    );
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;
    let dest = dir.join(format!("{id}.{ext}"));
    let temp = dir.join(format!("{id}.tmp"));
    if let Err(error) = tokio::fs::copy(&source, &temp).await {
        let _ = tokio::fs::remove_file(&temp).await;
        return Err(error.to_string());
    }
    let copied_bytes = tokio::fs::metadata(&temp)
        .await
        .map_err(|e| e.to_string())?
        .len();
    if !(16..=MAX_BYTES).contains(&copied_bytes) {
        let _ = tokio::fs::remove_file(&temp).await;
        return Err("El archivo cambió de tamaño durante la importación".into());
    }
    if let Err(error) = tokio::fs::rename(&temp, &dest).await {
        let _ = tokio::fs::remove_file(&temp).await;
        return Err(error.to_string());
    }
    Ok(SceneMedia {
        id,
        path: dest.to_string_lossy().to_string(),
        bytes: copied_bytes,
        mime: format!("video/{ext}"),
    })
}
#[tauri::command]
pub async fn remove_scene_media(id: String) -> Result<(), String> {
    remove_at(&id, &directory()).await
}
async fn remove_at(id: &str, dir: &std::path::Path) -> Result<(), String> {
    let _guard = MEDIA_MUTATION.lock().await;
    if !valid_id(id) {
        return Err("Id de vídeo inválido".into());
    }
    for ext in ["mp4", "webm"] {
        let path = dir.join(format!("{id}.{ext}"));
        if path.exists() {
            tokio::fs::remove_file(path)
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn video_headers_and_ids_are_bounded() {
        assert!(video_format("mp4", b"\0\0\0\x18ftypisom"));
        assert!(video_format("webm", &[0x1a, 0x45, 0xdf, 0xa3]));
        assert!(!video_format("mp4", b"not a video file"));
        assert!(!video_format("svg", b"<svg>"));
        assert!(!valid_id("scene-../../settings"));
        assert!(!valid_id("scene-a"));
        assert!(valid_id("scene-123456abcdef"));
    }
    #[tokio::test]
    async fn managed_import_listing_quota_and_removal_use_real_files() {
        let root = std::env::temp_dir().join(format!(
            "tfl-scene-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        tokio::fs::create_dir_all(&root).await.unwrap();
        let source = root.join("original.webm");
        let mut bytes = vec![0u8; 32];
        bytes[..4].copy_from_slice(&[0x1a, 0x45, 0xdf, 0xa3]);
        tokio::fs::write(&source, &bytes).await.unwrap();
        let managed = root.join("managed");
        let item = import_into(source.to_string_lossy().into(), managed.clone())
            .await
            .unwrap();
        assert_eq!(item.bytes, 32);
        assert!(std::path::Path::new(&item.path).starts_with(&managed));
        assert_eq!(tokio::fs::read(&item.path).await.unwrap(), bytes);
        assert_eq!(list_at(&managed).await.unwrap().len(), 1);
        assert!(remove_at("../../original", &managed).await.is_err());
        remove_at(&item.id, &managed).await.unwrap();
        assert!(list_at(&managed).await.unwrap().is_empty());
        assert!(source.exists());
        let fake = root.join("fake.mp4");
        tokio::fs::write(&fake, b"this is not a media container")
            .await
            .unwrap();
        assert!(
            import_into(fake.to_string_lossy().into(), managed.clone())
                .await
                .is_err()
        );
        let huge = root.join("huge.webm");
        tokio::fs::File::create(&huge)
            .await
            .unwrap()
            .set_len(MAX_BYTES + 1)
            .await
            .unwrap();
        assert!(
            import_into(huge.to_string_lossy().into(), managed.clone())
                .await
                .is_err()
        );
        for i in 0..MAX_FILES {
            tokio::fs::write(managed.join(format!("scene-{i:010x}.webm")), &bytes)
                .await
                .unwrap();
        }
        assert!(
            import_into(source.to_string_lossy().into(), managed.clone())
                .await
                .unwrap_err()
                .contains("12")
        );
        tokio::fs::remove_dir_all(root).await.unwrap();
    }
}
