//! Nonblocking, recoverable server preparation. Only TFL-owned partial downloads are removed.
use super::{
    instance_manager::{self, InstanceData, LoaderKind, ServerType},
    server_downloads, server_ports,
};
use crate::core::{HTTP, PathManager};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::LazyLock};
use tokio::io::AsyncWriteExt;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerTask {
    pub id: String,
    pub instance_name: String,
    #[serde(default)]
    pub instance_uuid: Option<String>,
    #[serde(default)]
    pub staged_jar: Option<String>,
    #[serde(default)]
    pub staged_sha256: Option<String>,
    #[serde(default)]
    pub staged_build: Option<String>,
    pub mc_version: String,
    pub server_type: ServerType,
    pub status: String,
    pub bytes_current: u64,
    pub bytes_total: Option<u64>,
    pub error: Option<String>,
}
static TASKS: LazyLock<tokio::sync::Mutex<Vec<ServerTask>>> = LazyLock::new(|| {
    let path = PathManager::get()
        .get_settings_dir()
        .join("server-tasks.json");
    let mut tasks: Vec<ServerTask> = std::fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    for task in &mut tasks {
        if active(&task.status) {
            task.status = "interrupted".into();
            task.error=Some("La preparación se interrumpió. Puedes reintentar sin perder los datos de la instancia.".into());
        }
    }
    tokio::sync::Mutex::new(tasks)
});
static CANCEL: LazyLock<std::sync::Mutex<HashMap<String, tokio::sync::watch::Sender<bool>>>> =
    LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
fn ensure_identity(task: &ServerTask, data: &InstanceData) -> Result<(), String> {
    if task.instance_uuid.as_deref() != Some(data.uuid.as_str())
        || data.server_type != Some(task.server_type)
    {
        return Err("ACTIVITY_INSTANCE_CHANGED: Esta actividad pertenece a otra instancia o no conserva su identidad. Se conservaron los archivos y la actividad histórica.".into());
    }
    Ok(())
}
fn artifact_matches(path: &std::path::Path, expected: &str) -> Result<bool, String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let metadata = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > 512 * 1024 * 1024
    {
        return Ok(false);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Ok(false);
        }
    }
    let mut file = std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(512 * 1024 * 1024 + 1);
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hex::encode(hash.finalize()) == expected)
}
fn active(status: &str) -> bool {
    matches!(status, "preparing" | "downloading")
}
async fn persist(tasks: &[ServerTask]) -> Result<(), String> {
    let path = PathManager::get()
        .get_settings_dir()
        .join("server-tasks.json");
    let tmp = path.with_extension("tmp");
    let bytes = serde_json::to_vec_pretty(tasks).map_err(|e| e.to_string())?;
    let mut file = tokio::fs::File::create(&tmp)
        .await
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes).await.map_err(|e| e.to_string())?;
    file.sync_all().await.map_err(|e| e.to_string())?;
    drop(file);
    tokio::fs::rename(tmp, path)
        .await
        .map_err(|e| e.to_string())
}
async fn update(task: ServerTask) {
    let mut tasks = TASKS.lock().await;
    if let Some(old) = tasks.iter_mut().find(|t| t.id == task.id) {
        *old = task;
    } else {
        tasks.push(task);
    }
    if let Err(e) = persist(&tasks).await {
        tracing::error!("No se pudo guardar el estado de actividades: {e}");
    }
    crate::core::emit(crate::core::AppEvent::ServerTasksChanged {});
}
pub async fn list() -> Vec<ServerTask> {
    TASKS.lock().await.clone()
}
pub async fn active_tasks() -> bool {
    TASKS.lock().await.iter().any(|task| active(&task.status))
}
pub async fn recover_after_restart() {
    for mut task in list()
        .await
        .into_iter()
        .filter(|task| task.status == "interrupted")
    {
        let _guard = super::content::mutation_lock(&task.instance_name).await;
        if let Ok(mut data) = instance_manager::get_instance(&task.instance_name).await {
            if let Err(error) = ensure_identity(&task, &data) {
                task.status = "failed".into();
                task.error = Some(error);
                update(task).await;
                continue;
            }
            if data.preparation.is_none() && data.server_jar.is_some() {
                task.status = "ready".into();
                task.error = None;
            } else {
                data.preparation = Some("interrupted".into());
                if let Err(error) = data.save().await {
                    tracing::warn!(%error, "No se pudo recuperar la preparación de servidor");
                }
            }
        }
        update(task).await;
    }
}
pub async fn create(
    name: String,
    version: String,
    kind: ServerType,
    port: Option<u16>,
    accept_eula: bool,
) -> Result<InstanceData, String> {
    let _maintenance = super::maintenance::operation()?;
    if version.is_empty()
        || version.len() > 48
        || !version
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-+".contains(&b))
    {
        return Err("La versión del servidor no es válida".into());
    }
    let _guard = server_ports::CREATION.lock().await;
    if list()
        .await
        .iter()
        .filter(|task| active(&task.status))
        .count()
        >= 50
    {
        return Err("Hay demasiadas preparaciones activas".into());
    }
    let port = server_ports::reserve_port(&name, port).await?;
    let mut data = instance_manager::create_instance_full(
        name,
        version.clone(),
        LoaderKind::Vanilla,
        None,
        version.clone(),
        Some(kind),
        None,
        None,
    )
    .await?;
    let task = ServerTask {
        id: uuid::Uuid::new_v4().to_string(),
        instance_name: data.name.clone(),
        instance_uuid: Some(data.uuid.clone()),
        staged_jar: None,
        staged_sha256: None,
        staged_build: None,
        mc_version: version,
        server_type: kind,
        status: "preparing".into(),
        bytes_current: 0,
        bytes_total: None,
        error: None,
    };
    let result = async {
        server_ports::write_reservation(&data.dir(), port).await?;
        if accept_eula {
            tokio::fs::write(
                data.dir().join("eula.txt"),
                "# Aceptación explícita del usuario en TFL Client\neula=true\n",
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        let mut tasks = TASKS.lock().await;
        let mut next = tasks.clone();
        if next.len() >= 50
            && let Some(index) = next.iter().position(|t| t.status == "ready")
        {
            next.remove(index);
        }
        next.push(task.clone());
        persist(&next).await?;
        *tasks = next;
        Ok::<(), String>(())
    }
    .await;
    if let Err(error) = result {
        data.preparation = Some("failed".into());
        if let Err(save_error) = data.save().await {
            tracing::warn!(%save_error, "No se pudo registrar el fallo de preparación");
        }
        return Err(error);
    }
    crate::core::emit(crate::core::AppEvent::ServerTasksChanged {});
    spawn(task);
    Ok(data)
}
fn spawn(task: ServerTask) {
    let (tx, rx) = tokio::sync::watch::channel(false);
    CANCEL.lock().unwrap().insert(task.id.clone(), tx);
    tokio::spawn(async move {
        let id = task.id.clone();
        let name = task.instance_name.clone();
        let result = prepare(task.clone(), rx).await;
        let mut finished = list()
            .await
            .into_iter()
            .find(|t| t.id == id)
            .unwrap_or(task);
        finished.status = match &result {
            Ok(()) => "ready",
            Err(e) if e == "PREPARATION_CANCELLED" => "cancelled",
            Err(_) => "failed",
        }
        .into();
        finished.error = result.err().filter(|e| e != "PREPARATION_CANCELLED");
        let _guard = super::content::mutation_lock(&name).await;
        if let Ok(mut data) = instance_manager::get_instance(&name).await {
            if let Err(error) = ensure_identity(&finished, &data) {
                finished.status = "failed".into();
                finished.error = Some(error);
                update(finished).await;
                CANCEL.lock().unwrap().remove(&id);
                return;
            }
            data.preparation = if finished.status == "ready" {
                None
            } else {
                Some(finished.status.clone())
            };
            if let Err(e) = data.save().await {
                finished.status = "failed".into();
                finished.error = Some(format!("No se pudo confirmar la instancia: {e}"));
            }
            let partial = data.dir().join(format!(".tfl-server-{id}.part"));
            let _ = tokio::fs::remove_file(partial).await;
        }
        update(finished).await;
        CANCEL.lock().unwrap().remove(&id);
    });
}
async fn prepare(
    mut task: ServerTask,
    mut cancel: tokio::sync::watch::Receiver<bool>,
) -> Result<(), String> {
    let download = tokio::select! {r=server_downloads::resolve_server_download(task.server_type,&task.mc_version)=>r?,_=cancel.changed()=>return Err("PREPARATION_CANCELLED".into())};
    if !download.filename.ends_with(".jar")
        || download.filename.contains(['/', '\\', ':'])
        || download.filename.starts_with('.')
    {
        return Err("El proveedor devolvió un nombre de servidor inválido".into());
    }
    let url = url::Url::parse(&download.url).map_err(|_| "URL del proveedor inválida")?;
    if url.scheme() != "https" || url.host_str().is_none() {
        return Err("La descarga del servidor requiere HTTPS".into());
    }
    let response=tokio::select!{r=HTTP.get(url).send()=>r.map_err(|_|"No se pudo conectar con el proveedor del servidor")?,_=cancel.changed()=>return Err("PREPARATION_CANCELLED".into())}.error_for_status().map_err(|e|format!("El proveedor rechazó la descarga: {}",e.status().map(|s|s.as_u16()).unwrap_or(0)))?;
    const LIMIT: u64 = 512 * 1024 * 1024;
    task.bytes_total = response.content_length();
    if task.bytes_total.is_some_and(|n| n > LIMIT) {
        return Err("El servidor supera el límite de descarga de 512 MiB".into());
    }
    let data = instance_manager::get_instance(&task.instance_name).await?;
    ensure_identity(&task, &data)?;
    let partial = data.dir().join(format!(".tfl-server-{}.part", task.id));
    let mut file = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&partial)
        .await
        .map_err(|e| e.to_string())?;
    let mut response = response;
    task.status = "downloading".into();
    update(task.clone()).await;
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    let mut tick = std::time::Instant::now();
    while let Some(bytes) = tokio::select! {r=response.chunk()=>r.map_err(|_|"La descarga del servidor se interrumpió; puedes reintentar")?,_=cancel.changed()=>return Err("PREPARATION_CANCELLED".into())}
    {
        task.bytes_current = task
            .bytes_current
            .checked_add(bytes.len() as u64)
            .ok_or("Descarga fuera de rango")?;
        if task.bytes_current > LIMIT {
            return Err("El servidor supera el límite de descarga de 512 MiB".into());
        }
        digest.update(&bytes);
        file.write_all(&bytes).await.map_err(|e| e.to_string())?;
        if tick.elapsed().as_millis() >= 500 {
            update(task.clone()).await;
            tick = std::time::Instant::now();
        }
    }
    if task.bytes_total.is_some_and(|n| n != task.bytes_current) {
        return Err("La descarga del servidor está incompleta".into());
    }
    file.sync_all().await.map_err(|e| e.to_string())?;
    drop(file);
    let verify = partial.clone();
    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(verify).map_err(|e| e.to_string())?;
        let mut jar =
            zip::ZipArchive::new(file).map_err(|_| "El proveedor no entregó un JAR válido")?;
        if jar.by_name("META-INF/MANIFEST.MF").is_err() {
            return Err("Al JAR del servidor le falta el manifiesto".into());
        }
        Ok::<(), String>(())
    })
    .await
    .map_err(|_| "No se pudo verificar el JAR")??;
    task.staged_jar = Some(download.filename.clone());
    task.staged_sha256 = Some(hex::encode(digest.finalize()));
    task.staged_build = Some(download.build_label.clone());
    {
        let mut tasks = TASKS.lock().await;
        let mut next = tasks.clone();
        let saved = next
            .iter_mut()
            .find(|saved| saved.id == task.id)
            .ok_or("Actividad no encontrada")?;
        *saved = task.clone();
        persist(&next).await?;
        *tasks = next;
    }
    let _guard = super::content::mutation_lock(&data.name).await;
    if *cancel.borrow() {
        return Err("PREPARATION_CANCELLED".into());
    }
    let mut data = instance_manager::get_instance(&data.name).await?;
    ensure_identity(&task, &data)?;
    let destination = data.dir().join(&download.filename);
    if destination.exists() {
        return Err("Ya existe ese JAR; no se reemplaza contenido del usuario".into());
    }
    tokio::fs::rename(&partial, destination)
        .await
        .map_err(|e| e.to_string())?;
    data.server_jar = Some(download.filename);
    data.server_build = Some(download.build_label);
    data.preparation = None;
    data.save().await.map_err(|e| e.to_string())?;
    Ok(())
}
pub async fn cancel(id: &str) -> Result<(), String> {
    let send = CANCEL
        .lock()
        .unwrap()
        .get(id)
        .cloned()
        .ok_or("Esta preparación ya no está activa")?;
    send.send(true)
        .map_err(|_| "La preparación ya terminó".into())
}
pub async fn retry(id: &str) -> Result<(), String> {
    let _maintenance = super::maintenance::operation()?;
    let task = list()
        .await
        .into_iter()
        .find(|t| t.id == id)
        .ok_or("Actividad no encontrada")?;
    let _instance_guard = super::content::mutation_lock(&task.instance_name).await;
    let mut task = list()
        .await
        .into_iter()
        .find(|t| t.id == id)
        .ok_or("Actividad no encontrada")?;
    if active(&task.status) {
        return Err("La preparación ya está activa".into());
    }
    let mut data = instance_manager::get_instance(&task.instance_name).await?;
    ensure_identity(&task, &data)?;
    if data
        .server_jar
        .as_ref()
        .is_some_and(|j| data.dir().join(j).exists())
    {
        return Err(
            "La instancia ya tiene un servidor; revisa sus archivos antes de reintentar".into(),
        );
    }
    if let (Some(jar), Some(expected)) = (&task.staged_jar, &task.staged_sha256) {
        let path = aqua::path_security::safe_join(&data.dir(), jar)?;
        if path.try_exists().map_err(|e| e.to_string())? {
            let verify_path = path.clone();
            let expected = expected.clone();
            if !tokio::task::spawn_blocking(move || artifact_matches(&verify_path, &expected))
                .await
                .map_err(|_| "No se pudo verificar el JAR recuperado")??
            {
                return Err("El JAR cambió desde la descarga. Se conservó para revisión y no se sobrescribió.".into());
            }
            data.server_jar = Some(jar.clone());
            data.server_build = task.staged_build.clone();
            data.preparation = None;
            data.save().await.map_err(|e| e.to_string())?;
            task.status = "ready".into();
            task.error = None;
            update(task).await;
            return Ok(());
        }
    }
    if uuid::Uuid::parse_str(&task.id).is_err() {
        return Err("La actividad tiene un identificador inválido".into());
    }
    let partial = data.dir().join(format!(".tfl-server-{}.part", task.id));
    match tokio::fs::remove_file(partial).await {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("No se pudo recuperar la descarga parcial: {e}")),
    }
    data.preparation = Some("preparing".into());
    data.save().await.map_err(|e| e.to_string())?;
    task.status = "preparing".into();
    task.error = None;
    task.bytes_current = 0;
    update(task.clone()).await;
    spawn(task);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task() -> ServerTask {
        ServerTask {
            id: uuid::Uuid::new_v4().to_string(),
            instance_name: "Same name".into(),
            instance_uuid: Some("original-identity".into()),
            staged_jar: None,
            staged_sha256: None,
            staged_build: None,
            mc_version: "1.21.4".into(),
            server_type: ServerType::Paper,
            status: "cancelled".into(),
            bytes_current: 0,
            bytes_total: None,
            error: None,
        }
    }
    fn instance() -> InstanceData {
        serde_json::from_value(serde_json::json!({
            "uuid":"original-identity", "name":"Same name", "mc_version":"1.21.4",
            "loader":"vanilla", "launch_version_id":"1.21.4", "server_type":"paper"
        }))
        .unwrap()
    }

    #[test]
    fn retry_identity_rejects_reused_names_changed_kind_and_unbound_history() {
        let mut data = instance();
        let mut activity = task();
        ensure_identity(&activity, &data).unwrap();
        data.uuid = "replacement-identity".into();
        assert!(
            ensure_identity(&activity, &data)
                .unwrap_err()
                .starts_with("ACTIVITY_INSTANCE_CHANGED")
        );
        data.uuid = "original-identity".into();
        data.server_type = None;
        assert!(ensure_identity(&activity, &data).is_err());
        data.server_type = Some(ServerType::Paper);
        activity.instance_uuid = None;
        assert!(ensure_identity(&activity, &data).is_err());
        let legacy = serde_json::to_value(&activity).unwrap();
        let mut legacy = legacy.as_object().unwrap().clone();
        legacy.remove("instance_uuid");
        legacy.remove("staged_jar");
        legacy.remove("staged_sha256");
        legacy.remove("staged_build");
        let restored: ServerTask = serde_json::from_value(legacy.into()).unwrap();
        assert_eq!(restored.instance_name, activity.instance_name);
        assert!(ensure_identity(&restored, &data).is_err());
    }

    #[test]
    fn recovery_only_accepts_download_bytes_with_the_recorded_hash() {
        use sha2::{Digest, Sha256};
        let directory =
            std::env::temp_dir().join(format!("tfl-task-recovery-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let jar = directory.join("server.jar");
        std::fs::write(&jar, b"downloaded").unwrap();
        let expected = hex::encode(Sha256::digest(b"downloaded"));
        assert!(artifact_matches(&jar, &expected).unwrap());
        std::fs::write(&jar, b"external edit").unwrap();
        assert!(!artifact_matches(&jar, &expected).unwrap());
        assert!(!artifact_matches(&directory, &expected).unwrap());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
