//! Tipos normalizados, independientes del proveedor (Modrinth, y más
//! adelante CurseForge). Los servicios de `content` solo hablan en estos
//! términos: cada proveedor es responsable de traducir su API a esto.
//!
//! Los tipos que terminan en el frontend (`ContentSearchHit`,
//! `ContentVersionSummary`, `InstalledContentInfo`, `ContentUpdate`)
//! conservan EXACTAMENTE los nombres de campo que ya consumía la UI.

use serde::{Deserialize, Serialize};

/// Tipo de contenido instalable en una instancia.
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

    /// Los shaders y resource packs no se filtran por mod loader
    /// (Fabric/Forge/…) en Modrinth — son compatibles con vanilla o vía un
    /// mod aparte (Iris/OptiFine para shaders), no por loader propio.
    /// Filtrar por loader ahí no tendría sentido y dejaría la búsqueda
    /// vacía. Los plugins sí filtran, pero por software de SERVIDOR
    /// (paper/purpur/spigot/bukkit) en el mismo parámetro `loader` — es el
    /// mismo facet `categories:x` de Modrinth, solo cambia qué valor
    /// llega ahí desde el frontend.
    pub fn filters_by_loader(self) -> bool {
        matches!(self, ContentKind::Mod | ContentKind::Plugin)
    }

    /// El `loader` de la instancia si este tipo se filtra por loader.
    pub fn loader_filter(self, loader: &str) -> Option<&str> {
        self.filters_by_loader().then_some(loader)
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
