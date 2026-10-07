//! Proveedores de contenido: de dónde salen los metadatos y las descargas.
//!
//! Un proveedor SOLO sabe hablar con su API y traducir sus respuestas a los
//! tipos normalizados de `content::types`. Todo lo demás —resolver
//! dependencias, descargar con verificación de hash, instalar, reemplazar
//! builds viejas, rollback, duplicados, export— es común y vive en los
//! servicios de `content`, que solo ven el trait `ContentProvider`.
//!
//! # Cómo agregar un proveedor (ej. CurseForge)
//!
//! 1. Crear `providers/curseforge.rs` con un struct (la API key se lee de la
//!    configuración al construirlo) que implemente `ContentProvider`,
//!    mapeando los JSON de CurseForge a `ContentVersion`, `ContentFile`,
//!    `ContentDependency`, etc. Todo lo específico de CurseForge (ids de
//!    clase, ids numéricos de loader, enums de tipo de dependencia) queda
//!    adentro de ese archivo.
//! 2. Declararlo acá abajo con `pub mod curseforge;`.
//! 3. Instanciar `ContentService::new(CurseForgeProvider::new(..))` donde
//!    corresponda (hoy `content::service()` devuelve el de Modrinth).
//!
//! No hace falta registro ni `dyn`: hay una sola implementación activa por
//! servicio y el trait se resuelve en compilación. Dos cosas a tener en
//! cuenta al implementarlo:
//! - Los hashes que se usan para reconocer archivos ya instalados son
//!   SHA1 en hex (`versions_from_hashes`). Un proveedor que identifique
//!   por otro esquema (ej. el fingerprint de CurseForge) tiene que
//!   traducirlo adentro; si no puede, devuelve un mapa vacío — el archivo
//!   queda como "no resuelto", igual que uno instalado a mano.
//! - Los métodos de lote (`versions_from_hashes`, `projects`, …) son
//!   best-effort para el servicio: un error ahí degrada a "sin resolver",
//!   no aborta la operación.

use super::types::{ContentKind, ContentProject, ContentSearchHit, ContentVersion};
use std::collections::HashMap;
use std::future::Future;

pub mod modrinth;

/// Parámetros de una búsqueda ya normalizados.
pub struct SearchQuery<'a> {
    pub query: &'a str,
    pub mc_version: &'a str,
    /// `Some` solo si el tipo de contenido se filtra por loader (ver
    /// `ContentKind::filters_by_loader`).
    pub loader: Option<&'a str>,
    pub categories: &'a [String],
    pub kind: ContentKind,
}

/// Operaciones que los servicios necesitan de un proveedor. Los futuros son
/// `Send` porque se ejecutan dentro de comandos de Tauri.
pub trait ContentProvider: Send + Sync {
    fn search(
        &self,
        query: &SearchQuery<'_>,
    ) -> impl Future<Output = Result<Vec<ContentSearchHit>, String>> + Send;

    /// Versiones de un proyecto compatibles con `mc_version` (y con `loader`
    /// si viene), la más nueva primero.
    fn list_versions(
        &self,
        project_id: &str,
        mc_version: &str,
        loader: Option<&str>,
    ) -> impl Future<Output = Result<Vec<ContentVersion>, String>> + Send;

    fn get_version(
        &self,
        version_id: &str,
    ) -> impl Future<Output = Result<ContentVersion, String>> + Send;

    /// Resuelve en lote archivos por su SHA1 → la versión a la que
    /// pertenecen. Los hashes que no resuelven no aparecen en el mapa.
    fn versions_from_hashes(
        &self,
        hashes: &[String],
    ) -> impl Future<Output = Result<HashMap<String, ContentVersion>, String>> + Send;

    /// Para cada SHA1 instalado, la build más nueva compatible con
    /// `loader` + `mc_version` (puede ser la misma que ya está instalada).
    fn latest_versions_from_hashes(
        &self,
        hashes: &[String],
        loader: &str,
        mc_version: &str,
    ) -> impl Future<Output = Result<HashMap<String, ContentVersion>, String>> + Send;

    /// Datos de varios proyectos en una sola llamada. Los ids que no existen
    /// simplemente no vienen en la lista.
    fn projects(
        &self,
        project_ids: &[String],
    ) -> impl Future<Output = Result<Vec<ContentProject>, String>> + Send;

    /// Ids de proyecto que el proveedor declara como dependencias
    /// requeridas (del lado cliente) de CUALQUIER versión del proyecto.
    /// Los proveedores que no tengan algo equivalente devuelven vacío.
    fn required_dependency_project_ids(
        &self,
        project_id: &str,
    ) -> impl Future<Output = Result<Vec<String>, String>> + Send;
}
