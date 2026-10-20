//! Búsqueda de proyectos y consulta de versiones disponibles.

use super::ContentService;
use super::providers::{ContentProvider, ProjectType, SearchQuery};
use super::types::{
    ContentKind, ContentProjectDetail, ContentSearchPage, ContentVersion, ContentVersionSummary,
    SearchIndex,
};

/// Mismo patrón que `InstallRequest` (`install.rs`) — agrupa los
/// parámetros de una búsqueda en vez de una firma de 8 argumentos sueltos.
/// `loader` llega SIEMPRE como string (puede venir vacío); `project_type`
/// decide si de verdad se aplica como filtro (ver
/// `ProjectType::filters_by_loader` — shaders/resourcepacks lo ignoran).
pub struct SearchRequest<'a> {
    pub query: &'a str,
    pub mc_version: &'a str,
    pub loader: &'a str,
    pub categories: &'a [String],
    pub project_type: ProjectType,
    pub offset: u32,
    pub limit: u32,
    pub index: SearchIndex,
}

impl<P: ContentProvider> ContentService<P> {
    pub async fn search(&self, req: SearchRequest<'_>) -> Result<ContentSearchPage, String> {
        let _perf = crate::core::perf::span("content.search");
        self.provider
            .search(&SearchQuery {
                query: req.query,
                mc_version: req.mc_version,
                loader: req.project_type.filters_by_loader().then_some(req.loader),
                categories: req.categories,
                project_type: req.project_type,
                offset: req.offset,
                limit: req.limit,
                index: req.index,
            })
            .await
    }

    /// Ficha completa de un proyecto, a demanda (ver `ContentProjectDetail`).
    /// Pasa primero por `project_cache` — un proceso típico de exploración
    /// abre/cierra la misma ficha varias veces.
    pub async fn project_detail(&self, project_id: &str) -> Result<ContentProjectDetail, String> {
        if let Some(cached) = super::project_cache::PROJECT_CACHE.get(project_id) {
            return Ok(cached);
        }
        let detail = self.provider.project_detail(project_id).await?;
        super::project_cache::PROJECT_CACHE.put(project_id.to_string(), detail.clone());
        Ok(detail)
    }

    /// La versión más nueva compatible, o `None` si no hay ninguna.
    pub async fn latest_version(
        &self,
        project_id: &str,
        mc_version: &str,
        loader: Option<&str>,
    ) -> Result<Option<ContentVersion>, String> {
        // El proveedor devuelve las versiones ordenadas por fecha de
        // publicación (más nueva primero) — la primera es la que se usa como
        // "instalar directo" cuando el usuario no eligió una versión puntual
        // del dropdown.
        let versions = self
            .provider
            .list_versions(project_id, mc_version, loader)
            .await?;
        Ok(versions.into_iter().next())
    }

    /// Versiones para el dropdown de "ver versiones" de la pestaña Descargar.
    pub async fn version_summaries(
        &self,
        kind: ContentKind,
        project_id: &str,
        mc_version: &str,
        loader: &str,
    ) -> Result<Vec<ContentVersionSummary>, String> {
        let versions = self
            .provider
            .list_versions(project_id, mc_version, kind.loader_filter(loader))
            .await?;
        Ok(versions.into_iter().map(Into::into).collect())
    }

    pub async fn version_changelog(&self, version_id: &str) -> Result<Option<String>, String> {
        Ok(self.provider.get_version(version_id).await?.changelog)
    }
}
