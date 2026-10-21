//! Comandos de Tauri para mods, shaders, resource packs y plugins. Son
//! deliberadamente finos: reciben los argumentos del frontend, llaman al
//! servicio (`services::content`) y devuelven su resultado. La lógica —
//! búsqueda, instalación con dependencias, rollback, export — vive allá.
//!
//! Los NOMBRES de comandos y de argumentos son contrato con el frontend
//! (`src/lib/api/tflApi.ts`): Tauri convierte camelCase de JS a los nombres
//! snake_case de acá, así que renombrar un parámetro rompe la UI sin avisar.
//!
//! CurseForge queda para una próxima iteración — necesita una API key
//! propia que todavía no existe (spec §26). Ver `services::content::providers`
//! para cómo se agrega un proveedor.
use crate::services::content::install::InstallRequest;
use crate::services::content::providers::ProjectType;
use crate::services::content::rollback::{self, ModRollbackInfo};
use crate::services::content::search::SearchRequest;
use crate::services::content::types::{
    ContentKind, ContentProjectDetail, ContentSearchPage, ContentUpdate, ContentVersionSummary,
    InstalledContentInfo, SearchIndex,
};
use crate::services::content::{self, filesystem, local_files};
use tauri::{AppHandle, command};
use tauri_plugin_opener::OpenerExt;

// ── Búsqueda ────────────────────────────────────────────────────────────────

/// Mismo parseo de `index` para los 4 comandos de búsqueda — si el
/// frontend manda algo que Modrinth no soporta, se rechaza con mensaje
/// claro en vez de dejar que la URL de Modrinth falle con un 400 opaco.
fn parse_index(index: &str) -> Result<SearchIndex, String> {
    SearchIndex::try_from(index)
}

#[command]
pub async fn search_mods(
    query: String,
    mc_version: String,
    loader: String,
    categories: Vec<String>,
    offset: u32,
    limit: u32,
    index: String,
) -> Result<ContentSearchPage, String> {
    content::service()
        .search(SearchRequest {
            query: &query,
            mc_version: &mc_version,
            loader: &loader,
            categories: &categories,
            project_type: ProjectType::Mod,
            offset,
            limit,
            index: parse_index(&index)?,
        })
        .await
}

#[command]
pub async fn search_shaders(
    query: String,
    mc_version: String,
    categories: Vec<String>,
    offset: u32,
    limit: u32,
    index: String,
) -> Result<ContentSearchPage, String> {
    content::service()
        .search(SearchRequest {
            query: &query,
            mc_version: &mc_version,
            loader: "",
            categories: &categories,
            project_type: ProjectType::Shader,
            offset,
            limit,
            index: parse_index(&index)?,
        })
        .await
}

#[command]
pub async fn search_resourcepacks(
    query: String,
    mc_version: String,
    categories: Vec<String>,
    offset: u32,
    limit: u32,
    index: String,
) -> Result<ContentSearchPage, String> {
    content::service()
        .search(SearchRequest {
            query: &query,
            mc_version: &mc_version,
            loader: "",
            categories: &categories,
            project_type: ProjectType::ResourcePack,
            offset,
            limit,
            index: parse_index(&index)?,
        })
        .await
}

#[command]
pub async fn search_plugins(
    query: String,
    mc_version: String,
    server_type: String,
    categories: Vec<String>,
    offset: u32,
    limit: u32,
    index: String,
) -> Result<ContentSearchPage, String> {
    content::service()
        .search(SearchRequest {
            query: &query,
            mc_version: &mc_version,
            loader: &server_type,
            categories: &categories,
            project_type: ProjectType::Plugin,
            offset,
            limit,
            index: parse_index(&index)?,
        })
        .await
}

// ── Ficha de proyecto ───────────────────────────────────────────────────────
// Un comando por kind, por la misma convención de nombres que el resto de
// este archivo — el fetch en sí (`ContentService::project_detail`) es
// agnóstico de kind, Modrinth no lo necesita para `/project/{id}`.

#[command]
pub async fn get_mod_project_detail(project_id: String) -> Result<ContentProjectDetail, String> {
    content::service().project_detail(&project_id).await
}

#[command]
pub async fn get_shader_project_detail(project_id: String) -> Result<ContentProjectDetail, String> {
    content::service().project_detail(&project_id).await
}

#[command]
pub async fn get_resourcepack_project_detail(
    project_id: String,
) -> Result<ContentProjectDetail, String> {
    content::service().project_detail(&project_id).await
}

#[command]
pub async fn get_plugin_project_detail(project_id: String) -> Result<ContentProjectDetail, String> {
    content::service().project_detail(&project_id).await
}

/// Abre un link de la ficha de proyecto (página de Modrinth, repo, wiki,
/// Discord, issues — dominios variados y legítimamente arbitrarios, a
/// diferencia de `open_external_url` que solo abre el propio repo de
/// GitHub del launcher). Solo se valida el esquema, no el dominio.
#[command]
pub async fn open_content_project_link(app: AppHandle, url: String) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("URL no permitida".into());
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

// ── Versiones disponibles ───────────────────────────────────────────────────

#[command]
pub async fn get_mod_versions(
    project_id: String,
    mc_version: String,
    loader: String,
) -> Result<Vec<ContentVersionSummary>, String> {
    content::service()
        .version_summaries(ContentKind::Mod, &project_id, &mc_version, &loader)
        .await
}

#[command]
pub async fn get_shader_versions(
    project_id: String,
    mc_version: String,
) -> Result<Vec<ContentVersionSummary>, String> {
    content::service()
        .version_summaries(ContentKind::Shader, &project_id, &mc_version, "")
        .await
}

#[command]
pub async fn get_plugin_versions(
    project_id: String,
    mc_version: String,
    server_type: String,
) -> Result<Vec<ContentVersionSummary>, String> {
    content::service()
        .version_summaries(ContentKind::Plugin, &project_id, &mc_version, &server_type)
        .await
}

#[command]
pub async fn get_resourcepack_versions(
    project_id: String,
    mc_version: String,
) -> Result<Vec<ContentVersionSummary>, String> {
    content::service()
        .version_summaries(ContentKind::ResourcePack, &project_id, &mc_version, "")
        .await
}

#[command]
pub async fn get_mod_version_changelog(version_id: String) -> Result<Option<String>, String> {
    content::service().version_changelog(&version_id).await
}

// ── Instalación ─────────────────────────────────────────────────────────────

#[command]
pub async fn install_mod(
    instance_name: String,
    project_id: String,
    mc_version: String,
    loader: String,
    version_id: Option<String>,
) -> Result<(), String> {
    content::service()
        .install(InstallRequest {
            instance_name: &instance_name,
            mc_version: &mc_version,
            loader: &loader,
            project_id: &project_id,
            version_id: version_id.as_deref(),
            kind: ContentKind::Mod,
        })
        .await
}

#[command]
pub async fn install_shader(
    instance_name: String,
    project_id: String,
    mc_version: String,
    version_id: Option<String>,
) -> Result<(), String> {
    content::service()
        .install(InstallRequest {
            instance_name: &instance_name,
            mc_version: &mc_version,
            loader: "",
            project_id: &project_id,
            version_id: version_id.as_deref(),
            kind: ContentKind::Shader,
        })
        .await
}

#[command]
pub async fn install_resourcepack(
    instance_name: String,
    project_id: String,
    mc_version: String,
    version_id: Option<String>,
) -> Result<(), String> {
    content::service()
        .install(InstallRequest {
            instance_name: &instance_name,
            mc_version: &mc_version,
            loader: "",
            project_id: &project_id,
            version_id: version_id.as_deref(),
            kind: ContentKind::ResourcePack,
        })
        .await
}

#[command]
pub async fn install_plugin(
    instance_name: String,
    project_id: String,
    mc_version: String,
    server_type: String,
    version_id: Option<String>,
) -> Result<(), String> {
    content::service()
        .install(InstallRequest {
            instance_name: &instance_name,
            mc_version: &mc_version,
            loader: &server_type,
            project_id: &project_id,
            version_id: version_id.as_deref(),
            kind: ContentKind::Plugin,
        })
        .await
}

// ── Archivos de la instancia ────────────────────────────────────────────────

#[command]
pub async fn get_instance_mods(instance_name: String) -> Result<Vec<String>, String> {
    filesystem::list_files(&instance_name, ContentKind::Mod).await
}

#[command]
pub async fn remove_mod(instance_name: String, filename: String) -> Result<(), String> {
    filesystem::remove_file(&instance_name, ContentKind::Mod, &filename).await
}

#[command]
pub async fn get_instance_shaders(instance_name: String) -> Result<Vec<String>, String> {
    filesystem::list_files(&instance_name, ContentKind::Shader).await
}

#[command]
pub async fn remove_shader(instance_name: String, filename: String) -> Result<(), String> {
    filesystem::remove_file(&instance_name, ContentKind::Shader, &filename).await
}

#[command]
pub async fn get_instance_resourcepacks(instance_name: String) -> Result<Vec<String>, String> {
    filesystem::list_files(&instance_name, ContentKind::ResourcePack).await
}

#[command]
pub async fn get_instance_plugins(instance_name: String) -> Result<Vec<String>, String> {
    filesystem::list_files(&instance_name, ContentKind::Plugin).await
}

#[command]
pub async fn remove_plugin(instance_name: String, filename: String) -> Result<(), String> {
    filesystem::remove_file(&instance_name, ContentKind::Plugin, &filename).await
}

#[command]
pub async fn remove_resourcepack(instance_name: String, filename: String) -> Result<(), String> {
    filesystem::remove_file(&instance_name, ContentKind::ResourcePack, &filename).await
}

/// Apaga o prende un mod renombrando su `.jar` ↔ `.jar.disabled`. Devuelve
/// el nombre de archivo resultante.
#[command]
pub async fn set_mod_enabled(
    instance_name: String,
    filename: String,
    enabled: bool,
) -> Result<String, String> {
    filesystem::set_enabled(&instance_name, ContentKind::Mod, &filename, enabled).await
}

#[command]
pub async fn set_plugin_enabled(
    instance_name: String,
    filename: String,
    enabled: bool,
) -> Result<String, String> {
    filesystem::set_enabled(&instance_name, ContentKind::Plugin, &filename, enabled).await
}

// ── Contenido instalado (resuelto por hash) ─────────────────────────────────

#[command]
pub async fn get_installed_mods_info(
    instance_name: String,
) -> Result<Vec<InstalledContentInfo>, String> {
    content::service()
        .installed_info(&instance_name, ContentKind::Mod)
        .await
}

#[command]
pub async fn get_installed_shaders_info(
    instance_name: String,
) -> Result<Vec<InstalledContentInfo>, String> {
    content::service()
        .installed_info(&instance_name, ContentKind::Shader)
        .await
}

#[command]
pub async fn get_installed_resourcepacks_info(
    instance_name: String,
) -> Result<Vec<InstalledContentInfo>, String> {
    content::service()
        .installed_info(&instance_name, ContentKind::ResourcePack)
        .await
}

#[command]
pub async fn get_installed_plugins_info(
    instance_name: String,
) -> Result<Vec<InstalledContentInfo>, String> {
    content::service()
        .installed_info(&instance_name, ContentKind::Plugin)
        .await
}

// ── Actualizaciones, rollback y duplicados ──────────────────────────────────

#[command]
pub async fn check_mod_updates(instance_name: String) -> Result<Vec<ContentUpdate>, String> {
    content::service().check_updates(&instance_name).await
}

#[command]
pub async fn update_all_mods(instance_name: String) -> Result<u32, String> {
    content::service().update_all(&instance_name).await
}

/// `None` si no hay una actualización que se pueda deshacer.
#[command]
pub async fn get_mod_rollback_info(
    instance_name: String,
) -> Result<Option<ModRollbackInfo>, String> {
    rollback::rollback_info(&instance_name).await
}

#[command]
pub async fn rollback_mod_update(instance_name: String) -> Result<u32, String> {
    rollback::rollback_update(&instance_name).await
}

#[command]
pub async fn discard_mod_rollback(instance_name: String) -> Result<(), String> {
    rollback::discard_rollback(&instance_name).await
}

#[command]
pub async fn find_duplicate_mods(instance_name: String) -> Result<Vec<Vec<String>>, String> {
    content::service().find_duplicates(&instance_name).await
}

#[command]
pub async fn remove_duplicate_mods(instance_name: String) -> Result<u32, String> {
    content::service().remove_duplicates(&instance_name).await
}

// ── Export ──────────────────────────────────────────────────────────────────

#[command]
pub async fn export_instance_as_mrpack(instance_name: String) -> Result<String, String> {
    content::service().export_mrpack(&instance_name).await
}

// ── Archivos locales ────────────────────────────────────────────────────────

/// `async` por la misma razón que `pick_image_file` (instance.rs): el diálogo
/// bloqueante no puede correr en el hilo principal.
#[command]
pub async fn pick_content_files(
    app: AppHandle,
    extension: String,
    filter_label: String,
) -> Vec<String> {
    local_files::pick_files(&app, &extension, &filter_label)
}

#[command]
pub async fn add_local_mod_files(instance_name: String, paths: Vec<String>) -> Result<u32, String> {
    local_files::add_local_files(&instance_name, ContentKind::Mod, paths).await
}

#[command]
pub async fn add_local_shader_files(
    instance_name: String,
    paths: Vec<String>,
) -> Result<u32, String> {
    local_files::add_local_files(&instance_name, ContentKind::Shader, paths).await
}

#[command]
pub async fn add_local_resourcepack_files(
    instance_name: String,
    paths: Vec<String>,
) -> Result<u32, String> {
    local_files::add_local_files(&instance_name, ContentKind::ResourcePack, paths).await
}

#[command]
pub async fn add_local_plugin_files(
    instance_name: String,
    paths: Vec<String>,
) -> Result<u32, String> {
    local_files::add_local_files(&instance_name, ContentKind::Plugin, paths).await
}
