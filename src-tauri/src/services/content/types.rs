//! Tipos normalizados, independientes del proveedor (Modrinth, y más
//! adelante CurseForge). Los servicios de `content` solo hablan en estos
//! términos: cada proveedor es responsable de traducir su API a esto.
//!
//! Los tipos que terminan en el frontend (`ContentSearchHit`,
//! `ContentVersionSummary`, `InstalledContentInfo`, `ContentUpdate`)
//! conservan EXACTAMENTE los nombres de campo que ya consumía la UI.

use super::providers::ProjectType;
use serde::{Deserialize, Serialize};

/// Tipo de contenido instalable en una instancia. A diferencia de
/// `ProjectType` (que solo describe cómo armar una búsqueda en el
/// proveedor), esto es un concepto de INSTALACIÓN — de acá salen la
/// carpeta y la extensión de archivo, y lo usan los matches exhaustivos de
/// `install.rs`/`filesystem.rs`/`local_files.rs`/`rollback.rs`. Los
/// modpacks nunca pasan por ese motor (tienen su propio instalador,
/// `cubrinth::mrpack`), por eso no tienen variante acá — ver
/// `ProjectType::Modpack` para el lado de búsqueda.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentKind {
    Mod,
    Shader,
    ResourcePack,
    /// Plugin de servidor (Paper/Purpur/Spigot/Bukkit) — Modrinth los
    /// modela con `project_type=plugin`, filtrando por software de
    /// servidor con el mismo mecanismo de "categories:x" que ya usan los
    /// loaders de cliente (ver `filters_by_loader`).
    Plugin,
}

impl ContentKind {
    /// Carpeta de la instancia donde va el archivo instalado.
    pub fn subdir(self) -> &'static str {
        match self {
            ContentKind::Mod => "mods",
            ContentKind::Shader => "shaderpacks",
            ContentKind::ResourcePack => "resourcepacks",
            ContentKind::Plugin => "plugins",
        }
    }

    /// Extensión de los archivos de este tipo (sin el punto).
    pub fn extension(self) -> &'static str {
        match self {
            ContentKind::Mod | ContentKind::Plugin => "jar",
            ContentKind::Shader | ContentKind::ResourcePack => "zip",
        }
    }

    /// El `ProjectType` equivalente para armar una búsqueda — las 4
    /// variantes de instalación mapean 1:1, solo `ProjectType::Modpack`
    /// no tiene contraparte acá.
    pub fn as_project_type(self) -> ProjectType {
        match self {
            ContentKind::Mod => ProjectType::Mod,
            ContentKind::Shader => ProjectType::Shader,
            ContentKind::ResourcePack => ProjectType::ResourcePack,
            ContentKind::Plugin => ProjectType::Plugin,
        }
    }

    /// El `loader` de la instancia si este tipo se filtra por loader (ver
    /// `ProjectType::filters_by_loader`).
    pub fn loader_filter(self, loader: &str) -> Option<&str> {
        self.as_project_type().filters_by_loader().then_some(loader)
    }
}

/// Resultado de búsqueda (una tarjeta de la pestaña Descargar).
#[derive(Debug, Deserialize, Serialize)]
pub struct ContentSearchHit {
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub author: String,
}

/// Archivo descargable de una versión.
#[derive(Debug, Clone)]
pub struct ContentFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub sha1: Option<String>,
    pub sha512: Option<String>,
    pub size: Option<u64>,
}

/// Cómo se relaciona una dependencia con la versión que la declara.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyKind {
    Required,
    Optional,
    Incompatible,
    Embedded,
    /// Cualquier otro valor que declare el proveedor.
    Other,
}

#[derive(Debug, Clone)]
pub struct ContentDependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub kind: DependencyKind,
}

/// Una versión publicada de un proyecto, ya normalizada.
#[derive(Debug, Clone)]
pub struct ContentVersion {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub version_number: String,
    /// ISO-8601: se compara como texto para ordenar por fecha.
    pub date_published: String,
    pub changelog: Option<String>,
    pub files: Vec<ContentFile>,
    pub dependencies: Vec<ContentDependency>,
}

impl ContentFile {
    /// ¿Es este el archivo con ese SHA1?
    pub fn has_sha1(&self, sha1: &str) -> bool {
        self.sha1.as_deref() == Some(sha1)
    }
}

/// Versión liviana para el dropdown de "ver versiones" de la pestaña
/// Descargar — no trae `files`/`dependencies`, que no hacen falta ahí y
/// solo abultan la respuesta.
#[derive(Debug, Serialize)]
pub struct ContentVersionSummary {
    pub id: String,
    pub name: String,
    pub version_number: String,
    pub date_published: String,
}

impl From<ContentVersion> for ContentVersionSummary {
    fn from(v: ContentVersion) -> Self {
        Self {
            id: v.id,
            name: v.name,
            version_number: v.version_number,
            date_published: v.date_published,
        }
    }
}

/// Datos del PROYECTO (no de una build puntual): nombre real, ícono y
/// categorías.
#[derive(Debug, Clone)]
pub struct ContentProject {
    pub id: String,
    pub title: String,
    pub icon_url: Option<String>,
    pub categories: Vec<String>,
}

/// Un archivo instalado, resuelto (o no) contra el proveedor.
#[derive(Debug, Serialize, Clone)]
pub struct InstalledContentInfo {
    pub filename: String,
    pub project_id: Option<String>,
    pub title: Option<String>,
    pub version_id: Option<String>,
    pub icon_url: Option<String>,
    pub categories: Vec<String>,
    /// El archivo termina en `.disabled`: instalado pero apagado, el juego no
    /// lo carga.
    pub disabled: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct ContentUpdate {
    pub filename: String,
    pub project_id: String,
    pub title: String,
    pub new_version_id: String,
}

/// Una página de resultados de búsqueda — a diferencia del `Vec` plano de
/// antes, trae lo necesario para que el frontend sepa cuántas páginas hay
/// en total sin inventar un número.
#[derive(Debug, Serialize)]
pub struct ContentSearchPage {
    pub hits: Vec<ContentSearchHit>,
    pub total_hits: u64,
    pub offset: u32,
    pub limit: u32,
}

/// Índices de orden que Modrinth API v2 realmente admite — no se agrega
/// ningún valor que el proveedor no soporte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchIndex {
    Relevance,
    Downloads,
    Follows,
    Newest,
    Updated,
}

impl SearchIndex {
    /// El string literal que espera el parámetro `index` de Modrinth.
    pub fn as_modrinth_param(self) -> &'static str {
        match self {
            SearchIndex::Relevance => "relevance",
            SearchIndex::Downloads => "downloads",
            SearchIndex::Follows => "follows",
            SearchIndex::Newest => "newest",
            SearchIndex::Updated => "updated",
        }
    }
}

impl TryFrom<&str> for SearchIndex {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "relevance" => Ok(SearchIndex::Relevance),
            "downloads" => Ok(SearchIndex::Downloads),
            "follows" => Ok(SearchIndex::Follows),
            "newest" => Ok(SearchIndex::Newest),
            "updated" => Ok(SearchIndex::Updated),
            other => Err(format!(
                "Orden de búsqueda no soportado: \"{other}\" (válidos: relevance, downloads, follows, newest, updated)"
            )),
        }
    }
}

/// Una imagen de la galería de un proyecto.
#[derive(Debug, Clone, Serialize)]
pub struct ContentGalleryImage {
    pub url: String,
    pub title: Option<String>,
    pub featured: bool,
}

/// Ficha completa de un proyecto — se pide a demanda al abrir el detalle,
/// nunca durante una búsqueda (eso solo trae `ContentSearchHit`, mucho más
/// liviano).
///
/// Sin `author`: `GET /project/{id}` de Modrinth no lo expone (solo un
/// `team` id, que requeriría una llamada aparte a `/team/{id}/members` que
/// no vale la pena solo para un nombre — ver verificación contra la API
/// real). El frontend ya tiene el autor del `ContentSearchHit` desde el
/// que se abrió esta ficha.
#[derive(Debug, Clone, Serialize)]
pub struct ContentProjectDetail {
    pub project_id: String,
    pub title: String,
    pub icon_url: Option<String>,
    pub description: String,
    /// Markdown crudo (el `body` de Modrinth) — el frontend lo sanea antes
    /// de renderizar, esto no es HTML.
    pub body: String,
    pub categories: Vec<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub downloads: u64,
    pub followers: u64,
    pub published: String,
    pub updated: String,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub issues_url: Option<String>,
    pub gallery: Vec<ContentGalleryImage>,
}
