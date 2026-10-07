//! Proveedor Modrinth (API v2): traduce sus respuestas JSON a los tipos
//! normalizados de `content::types`.

use super::{ContentProvider, SearchQuery};
use crate::core::{get_json_retrying, post_json_retrying};
use crate::services::content::types::{
    ContentDependency, ContentFile, ContentKind, ContentProject, ContentSearchHit, ContentVersion,
    DependencyKind,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MODRINTH_API: &str = "https://api.modrinth.com/v2";

#[derive(Debug, Clone, Copy, Default)]
pub struct ModrinthProvider;

/// `project_type` de Modrinth para cada tipo de contenido.
fn project_type(kind: ContentKind) -> &'static str {
    match kind {
        ContentKind::Mod => "mod",
        ContentKind::Shader => "shader",
        ContentKind::ResourcePack => "resourcepack",
        ContentKind::Plugin => "plugin",
    }
}

/// Facets de la búsqueda: grupos que se combinan con AND entre sí, y con OR
/// adentro de cada grupo.
fn search_facets(q: &SearchQuery<'_>) -> String {
    let mut facet_groups = vec![format!(r#"["project_type:{}"]"#, project_type(q.kind))];
    if let Some(loader) = q.loader {
        facet_groups.push(format!(r#"["categories:{}"]"#, loader.to_lowercase()));
    }
    // Categorías elegidas por el usuario en el filtro de la pestaña
    // Descargar — se combinan entre sí con OR (cualquiera de las elegidas
    // sirve) y con AND contra el resto de los facets ya armados.
    if !q.categories.is_empty() {
        let cats = q
            .categories
            .iter()
            .map(|c| format!(r#""categories:{c}""#))
            .collect::<Vec<_>>()
            .join(",");
        facet_groups.push(format!("[{cats}]"));
    }
    facet_groups.push(format!(r#"["versions:{}"]"#, q.mc_version));
    format!("[{}]", facet_groups.join(","))
}

// ── Estructuras crudas de la API ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SearchResponse {
    hits: Vec<RawHit>,
}

#[derive(Debug, Deserialize)]
struct RawHit {
    project_id: String,
    title: String,
    description: String,
    icon_url: Option<String>,
    downloads: u64,
    author: String,
}

#[derive(Debug, Deserialize)]
struct RawVersion {
    id: String,
    project_id: String,
    name: String,
    #[serde(default)]
    version_number: String,
    #[serde(default)]
    date_published: String,
    #[serde(default)]
    changelog: Option<String>,
    files: Vec<RawFile>,
    dependencies: Vec<RawDependency>,
}

#[derive(Debug, Deserialize)]
struct RawFile {
    url: String,
    filename: String,
    primary: bool,
    hashes: Option<RawFileHashes>,
    #[serde(default)]
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RawFileHashes {
    sha1: Option<String>,
    #[serde(default)]
    sha512: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawDependency {
    project_id: Option<String>,
    version_id: Option<String>,
    dependency_type: String,
}

#[derive(Debug, Deserialize)]
struct RawProject {
    id: String,
    title: String,
    icon_url: Option<String>,
    #[serde(default)]
    categories: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ProjectDependenciesResponse {
    projects: Vec<RawDependencyProject>,
}

#[derive(Debug, Deserialize)]
struct RawDependencyProject {
    id: String,
    client_side: String,
}

#[derive(Serialize)]
struct HashLookupBody<'a> {
    hashes: &'a [String],
    algorithm: &'a str,
}

#[derive(Serialize)]
struct HashUpdateBody<'a> {
    hashes: &'a [String],
    algorithm: &'a str,
    loaders: Vec<&'a str>,
    game_versions: Vec<&'a str>,
}

// ── Mapeo a tipos normalizados ──────────────────────────────────────────────

impl From<RawVersion> for ContentVersion {
    fn from(v: RawVersion) -> Self {
        ContentVersion {
            id: v.id,
            project_id: v.project_id,
            name: v.name,
            version_number: v.version_number,
            date_published: v.date_published,
            changelog: v.changelog,
            files: v.files.into_iter().map(Into::into).collect(),
            dependencies: v.dependencies.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<RawFile> for ContentFile {
    fn from(f: RawFile) -> Self {
        let (sha1, sha512) = match f.hashes {
            Some(h) => (h.sha1, h.sha512),
            None => (None, None),
        };
        ContentFile {
            url: f.url,
            filename: f.filename,
            primary: f.primary,
            sha1,
            sha512,
            size: f.size,
        }
    }
}

impl From<RawDependency> for ContentDependency {
    fn from(d: RawDependency) -> Self {
        ContentDependency {
            project_id: d.project_id,
            version_id: d.version_id,
            kind: match d.dependency_type.as_str() {
                "required" => DependencyKind::Required,
                "optional" => DependencyKind::Optional,
                "incompatible" => DependencyKind::Incompatible,
                "embedded" => DependencyKind::Embedded,
                _ => DependencyKind::Other,
            },
        }
    }
}

fn into_versions_by_hash(raw: HashMap<String, RawVersion>) -> HashMap<String, ContentVersion> {
    raw.into_iter().map(|(h, v)| (h, v.into())).collect()
}

impl ContentProvider for ModrinthProvider {
    async fn search(&self, q: &SearchQuery<'_>) -> Result<Vec<ContentSearchHit>, String> {
        let facets = search_facets(q);
        let url = format!(
            "{MODRINTH_API}/search?query={}&facets={}",
            urlencoding::encode(q.query),
            urlencoding::encode(&facets)
        );
        let resp: SearchResponse = get_json_retrying(&url).await?;

        Ok(resp
            .hits
            .into_iter()
            .map(|h| ContentSearchHit {
                project_id: h.project_id,
                title: h.title,
                description: h.description,
                icon_url: h.icon_url,
                downloads: h.downloads,
                author: h.author,
            })
            .collect())
    }

    async fn list_versions(
        &self,
        project_id: &str,
        mc_version: &str,
        loader: Option<&str>,
    ) -> Result<Vec<ContentVersion>, String> {
        let game_versions = format!(r#"["{mc_version}"]"#);
        let url = if let Some(loader) = loader {
            let loaders = format!(r#"["{}"]"#, loader.to_lowercase());
            format!(
                "{MODRINTH_API}/project/{project_id}/version?loaders={}&game_versions={}",
                urlencoding::encode(&loaders),
                urlencoding::encode(&game_versions)
            )
        } else {
            format!(
                "{MODRINTH_API}/project/{project_id}/version?game_versions={}",
                urlencoding::encode(&game_versions)
            )
        };
        let raw: Vec<RawVersion> = get_json_retrying(&url).await?;
        Ok(raw.into_iter().map(Into::into).collect())
    }

    async fn get_version(&self, version_id: &str) -> Result<ContentVersion, String> {
        let raw: RawVersion =
            get_json_retrying(&format!("{MODRINTH_API}/version/{version_id}")).await?;
        Ok(raw.into())
    }

    async fn versions_from_hashes(
        &self,
        hashes: &[String],
    ) -> Result<HashMap<String, ContentVersion>, String> {
        let body = HashLookupBody {
            hashes,
            algorithm: "sha1",
        };
        let raw: HashMap<String, RawVersion> =
            post_json_retrying(&format!("{MODRINTH_API}/version_files"), &body).await?;
        Ok(into_versions_by_hash(raw))
    }

    async fn latest_versions_from_hashes(
        &self,
        hashes: &[String],
        loader: &str,
        mc_version: &str,
    ) -> Result<HashMap<String, ContentVersion>, String> {
        let body = HashUpdateBody {
            hashes,
            algorithm: "sha1",
            loaders: vec![loader],
            game_versions: vec![mc_version],
        };
        let raw: HashMap<String, RawVersion> =
            post_json_retrying(&format!("{MODRINTH_API}/version_files/update"), &body).await?;
        Ok(into_versions_by_hash(raw))
    }

    async fn projects(&self, project_ids: &[String]) -> Result<Vec<ContentProject>, String> {
        if project_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids_json = serde_json::to_string(project_ids).unwrap_or_default();
        let url = format!(
            "{MODRINTH_API}/projects?ids={}",
            urlencoding::encode(&ids_json)
        );
        let projects: Vec<RawProject> = get_json_retrying(&url).await?;
        Ok(projects
            .into_iter()
            .map(|p| ContentProject {
                id: p.id,
                title: p.title,
                icon_url: p.icon_url,
                categories: p.categories,
            })
            .collect())
    }

    async fn required_dependency_project_ids(
        &self,
        project_id: &str,
    ) -> Result<Vec<String>, String> {
        let url = format!("{MODRINTH_API}/project/{project_id}/dependencies");
        let resp: ProjectDependenciesResponse = get_json_retrying(&url).await?;
        Ok(resp
            .projects
            .into_iter()
            .filter(|p| p.client_side == "required")
            .map(|p| p.id)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_VERSION: &str = r#"{
        "id": "ver1",
        "project_id": "proj1",
        "name": "Sodium 0.6.0",
        "version_number": "0.6.0+mc1.21",
        "date_published": "2024-10-01T12:00:00.000000Z",
        "changelog": "Notas",
        "downloads": 123,
        "files": [
            {"url": "https://cdn.modrinth.com/a.jar", "filename": "a.jar", "primary": true,
             "size": 2048, "hashes": {"sha1": "aaa", "sha512": "bbb"}},
            {"url": "https://cdn.modrinth.com/a-sources.jar", "filename": "a-sources.jar",
             "primary": false, "hashes": {"sha1": "ccc"}},
            {"url": "https://cdn.modrinth.com/b.jar", "filename": "b.jar", "primary": false}
        ],
        "dependencies": [
            {"project_id": "dep1", "version_id": null, "dependency_type": "required"},
            {"project_id": null, "version_id": "v9", "dependency_type": "optional"},
            {"project_id": "dep3", "version_id": null, "dependency_type": "incompatible"},
            {"project_id": "dep4", "version_id": null, "dependency_type": "embedded"},
            {"project_id": "dep5", "version_id": null, "dependency_type": "algo-nuevo"}
        ]
    }"#;

    #[test]
    fn version_json_maps_to_normalized_types() {
        let raw: RawVersion = serde_json::from_str(SAMPLE_VERSION).unwrap();
        let v: ContentVersion = raw.into();
        assert_eq!(v.id, "ver1");
        assert_eq!(v.project_id, "proj1");
        assert_eq!(v.changelog.as_deref(), Some("Notas"));
        assert_eq!(v.files.len(), 3);

        let primary = &v.files[0];
        assert!(primary.primary);
        assert_eq!(primary.sha1.as_deref(), Some("aaa"));
        assert_eq!(primary.sha512.as_deref(), Some("bbb"));
        assert_eq!(primary.size, Some(2048));
        assert!(primary.has_sha1("aaa"));

        // sha512 y size son opcionales; `hashes` entero también.
        assert_eq!(v.files[1].sha512, None);
        assert_eq!(v.files[1].size, None);
        assert_eq!(v.files[2].sha1, None);

        let kinds: Vec<_> = v.dependencies.iter().map(|d| d.kind).collect();
        assert_eq!(
            kinds,
            vec![
                DependencyKind::Required,
                DependencyKind::Optional,
                DependencyKind::Incompatible,
                DependencyKind::Embedded,
                DependencyKind::Other,
            ]
        );
        assert_eq!(v.dependencies[0].project_id.as_deref(), Some("dep1"));
        assert_eq!(v.dependencies[1].version_id.as_deref(), Some("v9"));
    }

    #[test]
    fn version_json_tolerates_missing_optional_fields() {
        let raw: RawVersion = serde_json::from_str(
            r#"{"id":"v","project_id":"p","name":"n","files":[],"dependencies":[]}"#,
        )
        .unwrap();
        let v: ContentVersion = raw.into();
        assert_eq!(v.version_number, "");
        assert_eq!(v.date_published, "");
        assert_eq!(v.changelog, None);
    }

    #[test]
    fn hash_map_response_maps_every_entry() {
        let json = format!(r#"{{"aaa": {SAMPLE_VERSION}}}"#);
        let raw: HashMap<String, RawVersion> = serde_json::from_str(&json).unwrap();
        let map = into_versions_by_hash(raw);
        assert_eq!(map["aaa"].project_id, "proj1");
    }

    #[test]
    fn search_facets_match_the_original_query_shape() {
        let cats = vec!["optimization".to_string(), "utility".to_string()];
        let q = SearchQuery {
            query: "sodium",
            mc_version: "1.21.1",
            loader: Some("Fabric"),
            categories: &cats,
            kind: ContentKind::Mod,
        };
        assert_eq!(
            search_facets(&q),
            r#"[["project_type:mod"],["categories:fabric"],["categories:optimization","categories:utility"],["versions:1.21.1"]]"#
        );

        let q = SearchQuery {
            query: "",
            mc_version: "1.21.1",
            loader: None,
            categories: &[],
            kind: ContentKind::Shader,
        };
        assert_eq!(
            search_facets(&q),
            r#"[["project_type:shader"],["versions:1.21.1"]]"#
        );
    }
}
