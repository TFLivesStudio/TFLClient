//! Proveedor Modrinth (API v2): traduce sus respuestas JSON a los tipos
//! normalizados de `content::types`.

use super::{ContentProvider, SearchQuery};
use crate::core::{get_json_retrying, post_json_retrying};
use crate::services::content::types::{
    ContentDependency, ContentFile, ContentGalleryImage, ContentProject, ContentProjectDetail,
    ContentSearchHit, ContentSearchPage, ContentVersion, DependencyKind,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MODRINTH_API: &str = "https://api.modrinth.com/v2";

#[derive(Debug, Clone, Copy, Default)]
pub struct ModrinthProvider;

/// Facets de la búsqueda: grupos que se combinan con AND entre sí, y con OR
/// adentro de cada grupo.
fn search_facets(q: &SearchQuery<'_>) -> String {
    let mut facet_groups = vec![format!(
        r#"["project_type:{}"]"#,
        q.project_type.as_modrinth_param()
    )];
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
    total_hits: u64,
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

/// `GET /project/{id}` — campos verificados contra la API real (no todos
/// los que documenta Modrinth, solo los que pide la ficha de detalle). No
/// trae autor: ese endpoint solo da un `team` id, no un nombre — ver
/// `ContentProjectDetail`.
#[derive(Debug, Deserialize)]
struct RawProjectDetail {
    id: String,
    title: String,
    icon_url: Option<String>,
    description: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    game_versions: Vec<String>,
    #[serde(default)]
    loaders: Vec<String>,
    downloads: u64,
    followers: u64,
    published: String,
    updated: String,
    #[serde(default)]
    source_url: Option<String>,
    #[serde(default)]
    wiki_url: Option<String>,
    #[serde(default)]
    discord_url: Option<String>,
    #[serde(default)]
    issues_url: Option<String>,
    #[serde(default)]
    gallery: Vec<RawGalleryImage>,
}

#[derive(Debug, Deserialize)]
struct RawGalleryImage {
    url: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    featured: bool,
}

impl From<RawProjectDetail> for ContentProjectDetail {
    fn from(p: RawProjectDetail) -> Self {
        ContentProjectDetail {
            project_id: p.id,
            title: p.title,
            icon_url: p.icon_url,
            description: p.description,
            body: p.body,
            categories: p.categories,
            game_versions: p.game_versions,
            loaders: p.loaders,
            downloads: p.downloads,
            followers: p.followers,
            published: p.published,
            updated: p.updated,
            source_url: p.source_url,
            wiki_url: p.wiki_url,
            discord_url: p.discord_url,
            issues_url: p.issues_url,
            gallery: p
                .gallery
                .into_iter()
                .map(|g| ContentGalleryImage {
                    url: g.url,
                    title: g.title,
                    featured: g.featured,
                })
                .collect(),
        }
    }
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
    async fn search(&self, q: &SearchQuery<'_>) -> Result<ContentSearchPage, String> {
        let facets = search_facets(q);
        let url = format!(
            "{MODRINTH_API}/search?query={}&facets={}&limit={}&offset={}&index={}",
            urlencoding::encode(q.query),
            urlencoding::encode(&facets),
            q.limit,
            q.offset,
            q.index.as_modrinth_param()
        );
        let resp: SearchResponse = get_json_retrying(&url).await?;

        Ok(ContentSearchPage {
            hits: resp
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
                .collect(),
            total_hits: resp.total_hits,
            offset: q.offset,
            limit: q.limit,
        })
    }

    async fn project_detail(&self, project_id: &str) -> Result<ContentProjectDetail, String> {
        let raw: RawProjectDetail =
            get_json_retrying(&format!("{MODRINTH_API}/project/{project_id}")).await?;
        Ok(raw.into())
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
    use crate::services::content::providers::ProjectType;
    use crate::services::content::types::SearchIndex;

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
            project_type: ProjectType::Mod,
            offset: 0,
            limit: 20,
            index: SearchIndex::Relevance,
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
            project_type: ProjectType::Shader,
            offset: 20,
            limit: 20,
            index: SearchIndex::Downloads,
        };
        assert_eq!(
            search_facets(&q),
            r#"[["project_type:shader"],["versions:1.21.1"]]"#
        );
    }

    #[test]
    fn project_type_as_modrinth_param_matches_each_variant() {
        assert_eq!(ProjectType::Mod.as_modrinth_param(), "mod");
        assert_eq!(ProjectType::Shader.as_modrinth_param(), "shader");
        assert_eq!(
            ProjectType::ResourcePack.as_modrinth_param(),
            "resourcepack"
        );
        assert_eq!(ProjectType::Plugin.as_modrinth_param(), "plugin");
        assert_eq!(ProjectType::Modpack.as_modrinth_param(), "modpack");
    }

    #[test]
    fn filters_by_loader_includes_mod_plugin_and_modpack_only() {
        assert!(ProjectType::Mod.filters_by_loader());
        assert!(ProjectType::Plugin.filters_by_loader());
        assert!(ProjectType::Modpack.filters_by_loader());
        assert!(!ProjectType::Shader.filters_by_loader());
        assert!(!ProjectType::ResourcePack.filters_by_loader());
    }

    #[test]
    fn search_index_try_from_accepts_every_supported_value() {
        assert_eq!(
            SearchIndex::try_from("relevance").unwrap(),
            SearchIndex::Relevance
        );
        assert_eq!(
            SearchIndex::try_from("downloads").unwrap(),
            SearchIndex::Downloads
        );
        assert_eq!(
            SearchIndex::try_from("follows").unwrap(),
            SearchIndex::Follows
        );
        assert_eq!(
            SearchIndex::try_from("newest").unwrap(),
            SearchIndex::Newest
        );
        assert_eq!(
            SearchIndex::try_from("updated").unwrap(),
            SearchIndex::Updated
        );
    }

    #[test]
    fn search_index_try_from_rejects_unsupported_values() {
        assert!(SearchIndex::try_from("trending").is_err());
        assert!(SearchIndex::try_from("").is_err());
    }

    const SAMPLE_PROJECT_DETAIL: &str = r##"{
        "id": "AANobbMI",
        "title": "Sodium",
        "icon_url": "https://cdn.modrinth.com/data/AANobbMI/icon.webp",
        "description": "Rendering engine replacement",
        "body": "# Sodium\nSome *markdown* body.",
        "categories": ["optimization"],
        "game_versions": ["1.21.1", "1.21.2"],
        "loaders": ["fabric", "neoforge", "quilt"],
        "downloads": 250361799,
        "followers": 41983,
        "published": "2021-01-03T00:53:34.185936Z",
        "updated": "2026-10-15T17:51:44.107526Z",
        "source_url": "https://github.com/CaffeineMC/sodium",
        "wiki_url": "https://github.com/CaffeineMC/sodium/wiki",
        "discord_url": "https://caffeinemc.net/discord",
        "issues_url": "https://github.com/CaffeineMC/sodium/issues",
        "gallery": [
            {"url": "https://cdn.modrinth.com/img1.webp", "title": "Lighting", "featured": true}
        ]
    }"##;

    #[test]
    fn project_detail_json_maps_to_normalized_type() {
        let raw: RawProjectDetail = serde_json::from_str(SAMPLE_PROJECT_DETAIL).unwrap();
        let detail: ContentProjectDetail = raw.into();
        assert_eq!(detail.project_id, "AANobbMI");
        assert_eq!(detail.title, "Sodium");
        assert_eq!(detail.loaders, vec!["fabric", "neoforge", "quilt"]);
        assert_eq!(detail.gallery.len(), 1);
        assert_eq!(detail.gallery[0].title.as_deref(), Some("Lighting"));
        assert!(detail.gallery[0].featured);
    }

    #[test]
    fn project_detail_json_tolerates_missing_optional_fields() {
        let raw: RawProjectDetail = serde_json::from_str(
            r#"{"id":"p","title":"t","icon_url":null,"description":"d","downloads":0,"followers":0,"published":"","updated":""}"#,
        )
        .unwrap();
        let detail: ContentProjectDetail = raw.into();
        assert_eq!(detail.body, "");
        assert_eq!(detail.categories, Vec::<String>::new());
        assert_eq!(detail.gallery.len(), 0);
        assert_eq!(detail.source_url, None);
    }
}
