//! Importar un `.mrpack` suelto, elegido a mano por el usuario (exportado de
//! Prism, de la app de Modrinth, de CurseForge o bajado de cualquier lado).
//!
//! El motor pesado ya existía (`cubrinth::mrpack`): `parse_mrpack` lee el zip
//! LOCAL y `install_mrpack` baja los mods del índice y extrae los overrides.
//! Acá solo se encadena: leer metadata → crear la instancia con la versión de
//! Minecraft y el loader del pack → instalar → dejar el manifest en
//! `.tfl_modpacks/` (igual que un modpack instalado desde Modrinth, para que
//! se pueda verificar/desinstalar después).
//!
//! El loader declarado por el pack se conserva. Si la comprobación exacta
//! falla, la importación informa el error y permite reintentar.

use super::import_sources::{ActiveImport, canceled};
use crate::commands::{instance::assign_random_icon, modpacks};
use crate::core::{HTTP, PathManager, get_json_retrying};
use crate::services::instance_manager::{self, InstanceData, LoaderKind};
use crate::services::progress;
use cubrinth::mrpack::{self, MrpackError, MrpackMetadata};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, command};
use zellkern::Loader;

/// Lo que el usuario ve antes de confirmar el import.
#[derive(Debug, Serialize)]
pub struct MrpackPreview {
    pub name: String,
    /// Nombre libre para la instancia (el del pack, o con " (2)" si ya existe).
    pub suggested_name: String,
    pub summary: Option<String>,
    pub version_id: String,
    pub mc_version: String,
    /// "vanilla" | "fabric" | "quilt" | "forge" | "neoforge"
    pub loader: String,
    pub loader_version: Option<String>,
    pub file_count: usize,
}

#[derive(Debug)]
struct Parsed {
    meta: MrpackMetadata,
    mc_version: String,
    loader: LoaderKind,
    pack_loader_version: Option<String>,
}

fn validate_path(path: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(path);
    let is_mrpack = p
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("mrpack"));
    if !is_mrpack {
        return Err("El archivo tiene que ser un .mrpack".into());
    }
    if !p.is_file() {
        return Err("No se encontró el archivo .mrpack elegido".into());
    }
    Ok(p)
}

/// Mensajes claros para el usuario — el texto crudo de `MrpackError`
/// ("ZIP error: invalid Zip archive…") no le dice nada.
fn friendly(e: MrpackError) -> String {
    match e {
        MrpackError::Zip(_) => {
            "El archivo está corrupto o no es un .mrpack válido (no se pudo leer como ZIP)".into()
        }
        MrpackError::Json(_) => {
            "El modpack está dañado: su índice (modrinth.index.json) no se pudo leer".into()
        }
        MrpackError::Invalid(msg) => format!("El archivo no es un .mrpack válido: {msg}"),
        MrpackError::Io(e) => format!("No se pudo leer el archivo: {e}"),
        MrpackError::Download(msg) => {
            format!("No se pudieron descargar los mods del modpack: {msg}")
        }
        MrpackError::Cancelled => "Importación cancelada. El archivo de origen se conserva.".into(),
    }
}

fn kind_and_version(loader: &Loader) -> (LoaderKind, Option<String>) {
    match loader {
        Loader::Vanilla => (LoaderKind::Vanilla, None),
        Loader::Fabric(v) => (LoaderKind::Fabric, Some(v.clone())),
        Loader::Quilt(v) => (LoaderKind::Quilt, Some(v.clone())),
        Loader::Forge(v) => (LoaderKind::Forge, Some(v.clone())),
        Loader::NeoForge(v) => (LoaderKind::NeoForge, Some(v.clone())),
    }
}

fn loader_str(kind: LoaderKind) -> &'static str {
    match kind {
        LoaderKind::Vanilla => "vanilla",
        LoaderKind::Fabric => "fabric",
        LoaderKind::Quilt => "quilt",
        LoaderKind::Forge => "forge",
        LoaderKind::NeoForge => "neoforge",
        LoaderKind::OrnitheGen2 => "ornithe-gen2",
    }
}

fn parse_blocking(path: &Path) -> Result<Parsed, String> {
    let meta = mrpack::parse_mrpack(path).map_err(friendly)?;
    if !meta.unsupported_dependencies.is_empty() {
        return Err(format!(
            "Este modpack usa un loader o componente que TFL Client no soporta: {}",
            meta.unsupported_dependencies.join(", ")
        ));
    }
    let gv = meta.game_version.clone().ok_or(
        "El modpack no indica la versión de Minecraft (falta \"minecraft\" en dependencies)",
    )?;
    let (loader, pack_loader_version) = kind_and_version(&gv.loader);
    Ok(Parsed {
        meta,
        mc_version: gv.mc_version,
        loader,
        pack_loader_version,
    })
}

/// Reemplaza caracteres que `valid_instance_name` rechaza y acota el largo
/// (el campo de la UI tiene `maxlength=60`).
fn sanitize_name(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| {
            if "/\\:*?\"<>|".contains(c) || c.is_control() {
                '-'
            } else {
                c
            }
        })
        .collect();
    let trimmed: String = cleaned.trim().chars().take(60).collect();
    let trimmed = trimmed.trim().to_string();
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        "Modpack importado".into()
    } else {
        trimmed
    }
}

fn free_name(base: &str, exists: impl Fn(&str) -> bool) -> String {
    let base = sanitize_name(base);
    if !exists(&base) {
        return base;
    }
    (2..1000)
        .map(|n| format!("{base} ({n})"))
        .find(|candidate| !exists(candidate))
        .unwrap_or(base)
}

fn instance_exists(name: &str) -> bool {
    PathManager::get().get_instance_dir().join(name).exists()
}

/// Algunos packs escriben la versión de Forge con el prefijo de Minecraft
/// (`1.20.1-47.2.0`); el resto del launcher usa solo `47.2.0`.
fn normalize_loader_version(kind: LoaderKind, mc_version: &str, version: &str) -> String {
    if kind == LoaderKind::Forge {
        if let Some(rest) = version.strip_prefix(&format!("{mc_version}-")) {
            return rest.to_string();
        }
    }
    version.to_string()
}

async fn verify_loader_url(client_get: bool, url: &str) -> Result<(), String> {
    let req = if client_get {
        HTTP.get(url)
    } else {
        HTTP.head(url)
    };
    let response = req.timeout(std::time::Duration::from_secs(12)).send().await
        .map_err(|_| "LOADER_CHECK_FAILED: No se pudo comprobar el proveedor. Revisa la conexión e inténtalo de nuevo.".to_string())?;
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }
    if status == reqwest::StatusCode::NOT_FOUND || status == reqwest::StatusCode::GONE {
        return Err(
            "LOADER_VERSION_UNAVAILABLE: El proveedor no ofrece esa versión exacta.".into(),
        );
    }
    Err(format!(
        "LOADER_CHECK_FAILED: El proveedor respondió HTTP {}. Inténtalo de nuevo.",
        status.as_u16()
    ))
}

/// ¿Se puede instalar esta versión exacta del loader? Comprueba la misma
/// URL que usa el launcher al jugar (ver aqua/downloaders).
async fn verify_exact_loader(kind: LoaderKind, mc_version: &str, lv: &str) -> Result<(), String> {
    match kind {
        LoaderKind::Vanilla => Ok(()),
        LoaderKind::Fabric => {
            verify_loader_url(
                true,
                &format!(
                    "https://meta.fabricmc.net/v2/versions/loader/{mc_version}/{lv}/profile/json"
                ),
            )
            .await
        }
        LoaderKind::Quilt => {
            verify_loader_url(
                true,
                &format!(
                    "https://meta.quiltmc.org/v3/versions/loader/{mc_version}/{lv}/profile/json"
                ),
            )
            .await
        }
        LoaderKind::Forge => {
            verify_loader_url(
                false,
                &aqua::ForgeBatch::resolve_installer_url(mc_version, lv),
            )
            .await
        }
        LoaderKind::NeoForge => {
            verify_loader_url(false, &aqua::NeoForgeBatch::resolve_installer_url(lv)).await
        }
        LoaderKind::OrnitheGen2 => {
            verify_loader_url(
                true,
                &format!(
                    "https://meta.ornithemc.net/v3/versions/gen2/fabric-loader/{mc_version}/{lv}/profile/json"
                ),
            )
            .await
        }
    }
}

async fn resolve_loader_for_pack(
    mc_version: &str,
    kind: LoaderKind,
    pack_version: Option<&str>,
) -> Result<(Option<String>, String), String> {
    if kind == LoaderKind::Vanilla {
        return Ok((None, mc_version.to_string()));
    }
    if let Some(v) = pack_version {
        let v = normalize_loader_version(kind, mc_version, v);
        verify_exact_loader(kind, mc_version, &v)
            .await
            .map_err(|error| {
                format!(
                    "{error} El modpack requiere {} {v} para Minecraft {mc_version}.",
                    loader_str(kind)
                )
            })?;
        let id = instance_manager::launch_version_id(mc_version, kind, &v);
        return Ok((Some(v), id));
    }
    instance_manager::resolve_loader(mc_version, kind).await
}

#[derive(Deserialize)]
struct MojangManifest {
    versions: Vec<MojangEntry>,
}
#[derive(Deserialize)]
struct MojangEntry {
    id: String,
}

/// Rechaza una versión de Minecraft que Mojang no lista. Si no se puede
/// consultar el manifest (sin red) no se bloquea: el resto del flujo ya
/// necesita red y fallará con su propio mensaje.
async fn check_mc_version(mc_version: &str) -> Result<(), String> {
    let Ok(manifest) = get_json_retrying::<MojangManifest>(aqua::MOJANG_MANIFEST_URL).await else {
        return Ok(());
    };
    if manifest.versions.iter().any(|v| v.id == mc_version) {
        Ok(())
    } else {
        Err(format!(
            "La versión de Minecraft del modpack ({mc_version}) no se reconoce"
        ))
    }
}

/// Lee el `.mrpack` (sin red para los mods, sin instalar nada) y devuelve lo
/// que se va a crear, para que el usuario confirme y elija el nombre.
#[command]
pub async fn inspect_mrpack(path: String) -> Result<MrpackPreview, String> {
    let path = validate_path(&path)?;
    let parsed = tokio::task::spawn_blocking(move || parse_blocking(&path))
        .await
        .map_err(|e| e.to_string())??;
    check_mc_version(&parsed.mc_version).await?;
    Ok(MrpackPreview {
        suggested_name: free_name(&parsed.meta.name, instance_exists),
        name: parsed.meta.name,
        summary: parsed.meta.summary,
        version_id: parsed.meta.version_id,
        mc_version: parsed.mc_version,
        loader: loader_str(parsed.loader).into(),
        loader_version: parsed.pack_loader_version,
        file_count: parsed.meta.file_count,
    })
}

/// Crea una instancia nueva a partir del `.mrpack` y le instala mods y
/// overrides. Si algo falla a mitad no deja una instancia a medio armar.
#[command]
pub async fn import_mrpack_file(
    app: AppHandle,
    path: String,
    new_name: Option<String>,
    request_id: Option<String>,
) -> Result<InstanceData, String> {
    let active = ActiveImport::register(request_id)?;
    let path = validate_path(&path)?;
    crate::services::worlds::no_link(&path)?;
    let parse_path = path.clone();
    let parsed = tokio::task::spawn_blocking(move || parse_blocking(&parse_path))
        .await
        .map_err(|e| e.to_string())??;
    check_mc_version(&parsed.mc_version).await?;
    canceled(&active.cancel)?;

    let name = match new_name.map(|n| n.trim().to_string()) {
        Some(n) if !n.is_empty() => n,
        _ => free_name(&parsed.meta.name, instance_exists),
    };

    let (loader_version, launch_id) = resolve_loader_for_pack(
        &parsed.mc_version,
        parsed.loader,
        parsed.pack_loader_version.as_deref(),
    )
    .await?;
    canceled(&active.cancel)?;

    instance_manager::valid_instance_name(&name)?;
    crate::services::worlds::valid_name(&name)?;
    let _mutation = crate::services::content::mutation_lock(&name).await;
    canceled(&active.cancel)?;
    let data: InstanceData = serde_json::from_value(serde_json::json!({
        "uuid": uuid::Uuid::new_v4().to_string(),
        "name": name,
        "mc_version": parsed.mc_version,
        "loader": parsed.loader,
        "loader_version": loader_version,
        "launch_version_id": launch_id,
    }))
    .map_err(|_| "No se pudo crear el manifiesto del modpack.")?;
    let destination = data.dir();
    tokio::fs::create_dir(&destination).await.map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            "Ya existe una instancia con ese nombre. Sus archivos siguen intactos.".into()
        } else {
            format!("No se pudo preparar la importación: {error}")
        }
    })?;

    let shared_dir = PathManager::get().get_shared_dir().to_path_buf();
    let (tx, watcher) = progress::watch(&format!("mrpack:{}", data.name))?;
    let installed = mrpack::install_mrpack_cancellable(
        &path,
        &destination,
        &shared_dir,
        Some(tx),
        Some(&active.cancel),
    )
    .await
    .map_err(friendly);
    let result: Result<(), String> = async {
        let metadata = installed?;
        canceled(&active.cancel)?;
        let version_id = if modpacks::valid_version_id(&metadata.version_id).is_ok() {
            metadata.version_id.clone()
        } else {
            format!("local-{}", uuid::Uuid::new_v4())
        };
        write_manifest(&destination, &version_id, &metadata).await?;
        canceled(&active.cancel)?;
        data.save().await.map_err(|e| e.to_string())
    }
    .await;
    let _ = watcher.finish(result.clone()).await;
    if let Err(reason) = result {
        if let Err(error) = tokio::fs::remove_dir_all(&destination).await {
            return Err(format!(
                "{reason} No se pudo retirar la carpeta incompleta: {error}"
            ));
        }
        return Err(reason);
    }
    if !destination.join("icon.png").is_file() {
        assign_random_icon(&app, &data).await;
    }
    Ok(data)
}

async fn write_manifest(
    instance_dir: &Path,
    version_id: &str,
    metadata: &MrpackMetadata,
) -> Result<(), String> {
    let dir = modpacks::manifest_dir(instance_dir);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;
    // Sin "/" en el project_id: ese formato es el de los modpacks
    // comunitarios de GitHub (los únicos que `check_modpack_updates` mira).
    let manifest = serde_json::json!({
        "project_id": format!("local:{}", sanitize_name(&metadata.name).replace('/', "-")),
        "version_id": version_id,
        "title": metadata.name,
        "source": "mrpack-file",
        "installed_paths": metadata.installed_paths,
    });
    tokio::fs::write(
        dir.join(format!("{version_id}.json")),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .await
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("tfl-mrpack-test-{tag}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_mrpack(dir: &Path, index: &str) -> PathBuf {
        let path = dir.join("pack.mrpack");
        let file = std::fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file(
            "modrinth.index.json",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        zip.write_all(index.as_bytes()).unwrap();
        zip.finish().unwrap();
        path
    }

    fn index(deps: &str) -> String {
        format!(
            r#"{{"game":"minecraft","formatVersion":1,"versionId":"1.2.3","name":"Mi Pack","summary":"hola","files":[],"dependencies":{deps}}}"#
        )
    }

    #[test]
    fn rejects_wrong_extension_and_missing_file() {
        assert!(validate_path("/tmp/algo.zip").is_err());
        assert!(
            validate_path("/tmp/no-existe-tfl.mrpack")
                .unwrap_err()
                .contains("No se encontró")
        );
    }

    #[test]
    fn fabric_pack_is_parsed() {
        let dir = temp_dir("fabric");
        let p = write_mrpack(
            &dir,
            &index(r#"{"minecraft":"1.20.1","fabric-loader":"0.15.11"}"#),
        );
        let parsed = parse_blocking(&p).unwrap();
        assert_eq!(parsed.mc_version, "1.20.1");
        assert_eq!(parsed.loader, LoaderKind::Fabric);
        assert_eq!(parsed.pack_loader_version.as_deref(), Some("0.15.11"));
        assert_eq!(parsed.meta.name, "Mi Pack");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn forge_neoforge_quilt_and_vanilla_are_mapped() {
        let dir = temp_dir("loaders");
        for (deps, kind) in [
            (
                r#"{"minecraft":"1.20.1","forge":"47.2.0"}"#,
                LoaderKind::Forge,
            ),
            (
                r#"{"minecraft":"1.21.1","neoforge":"21.1.1"}"#,
                LoaderKind::NeoForge,
            ),
            (
                r#"{"minecraft":"1.20.1","quilt-loader":"0.25.0"}"#,
                LoaderKind::Quilt,
            ),
            (r#"{"minecraft":"1.20.1"}"#, LoaderKind::Vanilla),
        ] {
            let p = write_mrpack(&dir, &index(deps));
            assert_eq!(parse_blocking(&p).unwrap().loader, kind, "{deps}");
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn corrupt_file_gives_clear_error_not_a_panic() {
        let dir = temp_dir("corrupt");
        let p = dir.join("roto.mrpack");
        std::fs::write(&p, b"esto no es un zip").unwrap();
        let err = parse_blocking(&p).unwrap_err();
        assert!(err.contains("corrupto"), "{err}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn zip_without_index_is_rejected() {
        let dir = temp_dir("noindex");
        let path = dir.join("vacio.mrpack");
        let file = std::fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("otra-cosa.txt", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"x").unwrap();
        zip.finish().unwrap();
        let err = parse_blocking(&path).unwrap_err();
        assert!(err.contains("modrinth.index.json"), "{err}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn unsupported_loader_and_missing_minecraft_are_rejected() {
        let dir = temp_dir("unsupported");
        let p = write_mrpack(
            &dir,
            &index(r#"{"minecraft":"1.12.2","legacy-fabric":"0.1"}"#),
        );
        assert!(
            parse_blocking(&p)
                .unwrap_err()
                .contains("no soporta: legacy-fabric")
        );
        let p = write_mrpack(&dir, &index(r#"{"fabric-loader":"0.15.11"}"#));
        assert!(
            parse_blocking(&p)
                .unwrap_err()
                .contains("versión de Minecraft")
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn non_minecraft_game_is_rejected() {
        let dir = temp_dir("game");
        let p = write_mrpack(
            &dir,
            r#"{"game":"hytale","formatVersion":1,"versionId":"1","name":"X","files":[],"dependencies":{}}"#,
        );
        assert!(parse_blocking(&p).unwrap_err().contains("not 'minecraft'"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn names_are_sanitized_and_deduplicated() {
        assert_eq!(sanitize_name("  a/b:c*  "), "a-b-c-");
        assert_eq!(sanitize_name(".."), "Modpack importado");
        assert_eq!(sanitize_name(""), "Modpack importado");
        assert_eq!(sanitize_name(&"x".repeat(100)).chars().count(), 60);
        let taken = ["Pack", "Pack (2)"];
        assert_eq!(free_name("Pack", |n| taken.contains(&n)), "Pack (3)");
        assert_eq!(free_name("Otro", |n| taken.contains(&n)), "Otro");
    }

    #[test]
    fn forge_version_prefix_is_stripped_only_for_forge() {
        assert_eq!(
            normalize_loader_version(LoaderKind::Forge, "1.20.1", "1.20.1-47.2.0"),
            "47.2.0"
        );
        assert_eq!(
            normalize_loader_version(LoaderKind::Forge, "1.20.1", "47.2.0"),
            "47.2.0"
        );
        assert_eq!(
            normalize_loader_version(LoaderKind::Fabric, "1.20.1", "0.15.11"),
            "0.15.11"
        );
    }
    #[tokio::test]
    async fn exact_loader_probe_distinguishes_missing_version_from_provider_failure() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        async fn endpoint(status: &str) -> String {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let response =
                format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            tokio::spawn(async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = [0u8; 1024];
                let received = socket.read(&mut request).await.unwrap();
                assert!(received > 0, "HTTP fixture received an empty request");
                socket.write_all(response.as_bytes()).await.unwrap();
            });
            format!("http://{address}/exact-loader")
        }
        verify_loader_url(false, &endpoint("200 OK").await)
            .await
            .unwrap();
        let missing = verify_loader_url(false, &endpoint("404 Not Found").await)
            .await
            .unwrap_err();
        assert!(missing.starts_with("LOADER_VERSION_UNAVAILABLE"));
        let unavailable = verify_loader_url(false, &endpoint("503 Service Unavailable").await)
            .await
            .unwrap_err();
        assert!(unavailable.starts_with("LOADER_CHECK_FAILED"));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let unreachable = verify_loader_url(false, &format!("http://{address}/exact-loader"))
            .await
            .unwrap_err();
        assert!(unreachable.starts_with("LOADER_CHECK_FAILED"));
    }
}
