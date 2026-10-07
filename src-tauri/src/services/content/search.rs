//! Búsqueda de proyectos y consulta de versiones disponibles.

use super::ContentService;
use super::providers::{ContentProvider, SearchQuery};
use super::types::{ContentKind, ContentSearchHit, ContentVersion, ContentVersionSummary};

impl<P: ContentProvider> ContentService<P> {
    pub async fn search(
        &self,
        query: &str,
        mc_version: &str,
        loader: &str,
        categories: &[String],
        kind: ContentKind,
    ) -> Result<Vec<ContentSearchHit>, String> {
        let _perf = crate::core::perf::span("content.search");
        self.provider
            .search(&SearchQuery {
                query,
                mc_version,
                loader: kind.loader_filter(loader),
                categories,
                kind,
            })
            .await
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
