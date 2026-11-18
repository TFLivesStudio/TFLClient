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
    update(task.clone()).await;
    let _guard = super::content::mutation_lock(&data.name).await;
    if *cancel.borrow() {
        return Err("PREPARATION_CANCELLED".into());
    }
    let mut data = instance_manager::get_instance(&data.name).await?;
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
    if data
        .server_jar
        .as_ref()
        .is_some_and(|j| data.dir().join(j).exists())
    {
        return Err(
            "La instancia ya tiene un servidor; revisa sus archivos antes de reintentar".into(),
        );
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
