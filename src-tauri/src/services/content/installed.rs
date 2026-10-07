//! Contenido ya instalado en una instancia: qué proyecto del proveedor es
//! cada archivo.
//!
//! `get_instance_mods` solo devolvía nombres de archivo crudos — no había
//! forma de saber a qué mod correspondía cada .jar instalado (ni para
//! mostrar el nombre real, ni para ofrecer actualizarlo). El proveedor
//! resuelve por hash SHA1 del archivo, sin importar si se instaló desde este
//! launcher o no.

use super::ContentService;
use super::filesystem::{hash_installed_files, is_disabled_file};
use super::providers::ContentProvider;
use super::types::{ContentKind, ContentProject, ContentVersion, InstalledContentInfo};
use crate::services::instance_manager;
use std::collections::{HashMap, HashSet};

/// project_id → archivos instalados de ese proyecto (sin contar los
/// desactivados). Es lo que permite reemplazar en vez de duplicar al instalar.
pub(crate) type InstalledProjects = HashMap<String, Vec<String>>;

pub(crate) fn projects_from_info(infos: &[InstalledContentInfo]) -> InstalledProjects {
    let mut out = InstalledProjects::new();
    for info in infos {
        if info.disabled {
            continue;
        }
        if let Some(pid) = &info.project_id {
            out.entry(pid.clone())
                .or_default()
                .push(info.filename.clone());
        }
    }
    out
}

/// Arma la fila de un archivo instalado a partir de lo que resolvió el
/// proveedor (`None` si el archivo no resolvió).
fn describe_installed(
    filename: String,
    version: Option<ContentVersion>,
    project_info: &HashMap<String, ContentProject>,
) -> InstalledContentInfo {
    let disabled = is_disabled_file(&filename);
    match version {
        Some(v) => {
            let info = project_info.get(&v.project_id);
            InstalledContentInfo {
                filename,
                project_id: Some(v.project_id.clone()),
                // Preferí el título del proyecto (el nombre real del
                // mod) sobre el `name` de la versión puntual — si el
                // bulk lookup falla (red, proveedor caído) se cae al
                // de la versión como antes, mejor eso que nada.
                title: Some(
                    info.map(|i| i.title.clone())
                        .unwrap_or_else(|| v.name.clone()),
                ),
                version_id: Some(v.id.clone()),
                icon_url: info.and_then(|i| i.icon_url.clone()),
                categories: info.map(|i| i.categories.clone()).unwrap_or_default(),
                disabled,
            }
        }
        None => InstalledContentInfo {
            filename,
            project_id: None,
            title: None,
            version_id: None,
            categories: Vec::new(),
            icon_url: None,
            disabled,
        },
    }
}

impl<P: ContentProvider> ContentService<P> {
    /// Cada archivo de `subdir/` junto con la versión a la que resuelve por
    /// hash (`None` si no resuelve: build propia, de otro sitio, o sin red).
    /// Una sola llamada en bulk, sin importar cuántos archivos haya.
    pub(crate) async fn resolve_installed_files(
        &self,
        instance: &instance_manager::InstanceData,
        subdir: &str,
        ext: &str,
    ) -> Vec<(String, Option<ContentVersion>)> {
        let hashed = hash_installed_files(&instance.dir().join(subdir), ext).await;
        if hashed.is_empty() {
            return Vec::new();
        }
        let hashes: Vec<String> = hashed.iter().map(|(_, h)| h.clone()).collect();
        let resolved = self
            .provider
            .versions_from_hashes(&hashes)
            .await
            .unwrap_or_default();
        hashed
            .into_iter()
            .map(|(filename, hash)| {
                let version = resolved.get(&hash).cloned();
                (filename, version)
            })
            .collect()
    }

    pub(crate) async fn installed_projects(
        &self,
        instance_name: &str,
        kind: ContentKind,
    ) -> InstalledProjects {
        let Ok(instance) = instance_manager::get_instance(instance_name).await else {
            return InstalledProjects::new();
        };
        let mut out = InstalledProjects::new();
        for (filename, version) in self
            .resolve_installed_files(&instance, kind.subdir(), kind.extension())
            .await
        {
            if is_disabled_file(&filename) {
                continue;
            }
            if let Some(v) = version {
                out.entry(v.project_id).or_default().push(filename);
            }
        }
        out
    }

    /// Resuelve cada archivo instalado contra el proveedor por hash —
    /// funciona con cualquier build publicada ahí, se haya instalado desde
    /// este launcher o no. Los que no resuelven (de otro lado, builds
    /// custom) quedan con los campos en None, no rompe nada. Es lo que
    /// permite marcar con check, en la pestaña Descargar, los resultados
    /// que ya están instalados.
    pub async fn installed_info(
        &self,
        instance_name: &str,
        kind: ContentKind,
    ) -> Result<Vec<InstalledContentInfo>, String> {
        let instance = instance_manager::get_instance(instance_name).await?;
        let files = self
            .resolve_installed_files(&instance, kind.subdir(), kind.extension())
            .await;
        if files.is_empty() {
            return Ok(Vec::new());
        }

        let project_ids: Vec<String> = files
            .iter()
            .filter_map(|(_, v)| v.as_ref().map(|v| v.project_id.clone()))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        // Nombre e ícono reales del MOD (ej. "Canvas Renderer"), no de la
        // build puntual instalada — algunos autores le ponen a sus versiones
        // un `name` técnico que no identifica el mod en absoluto (ej. Canvas
        // Renderer nombra sus builds "fabric-20.0.2625", que a simple vista
        // parece ser el propio Fabric Loader instalándose solo), y la
        // resolución por hash no trae ícono en absoluto. Un solo request en
        // bulk para todos los project_id resueltos a la vez, en vez de uno
        // por mod.
        let project_info: HashMap<String, ContentProject> = self
            .provider
            .projects(&project_ids)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|p| (p.id.clone(), p))
            .collect();

        Ok(files
            .into_iter()
            .map(|(filename, version)| describe_installed(filename, version, &project_info))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(filename: &str, pid: Option<&str>, disabled: bool) -> InstalledContentInfo {
        InstalledContentInfo {
            filename: filename.into(),
            project_id: pid.map(String::from),
            title: None,
            version_id: None,
            icon_url: None,
            categories: vec![],
            disabled,
        }
    }

    fn version(project_id: &str, name: &str) -> ContentVersion {
        ContentVersion {
            id: "v1".into(),
            project_id: project_id.into(),
            name: name.into(),
            version_number: String::new(),
            date_published: String::new(),
            changelog: None,
            files: vec![],
            dependencies: vec![],
        }
    }

    #[test]
    fn projects_map_skips_disabled_and_unresolved() {
        let infos = vec![
            mk("a.jar", Some("P1"), false),
            mk("a-sources.jar", Some("P1"), false),
            mk("b.jar.disabled", Some("P2"), true),
            mk("c.jar", None, false),
        ];
        let map = projects_from_info(&infos);
        assert_eq!(map.len(), 1);
        assert_eq!(
            map["P1"],
            vec!["a.jar".to_string(), "a-sources.jar".to_string()]
        );
    }

    #[test]
    fn installed_title_prefers_the_project_name_over_the_build_name() {
        let mut projects = HashMap::new();
        projects.insert(
            "P1".to_string(),
            ContentProject {
                id: "P1".into(),
                title: "Canvas Renderer".into(),
                icon_url: Some("https://icon".into()),
                categories: vec!["optimization".into()],
            },
        );
        let info = describe_installed(
            "canvas.jar".into(),
            Some(version("P1", "fabric-20.0.2625")),
            &projects,
        );
        assert_eq!(info.title.as_deref(), Some("Canvas Renderer"));
        assert_eq!(info.icon_url.as_deref(), Some("https://icon"));
        assert_eq!(info.categories, vec!["optimization".to_string()]);
        assert!(!info.disabled);
    }

    #[test]
    fn installed_title_falls_back_to_the_build_name_and_flags_disabled_files() {
        let info = describe_installed(
            "x.jar.disabled".into(),
            Some(version("P9", "Build 3")),
            &HashMap::new(),
        );
        assert_eq!(info.title.as_deref(), Some("Build 3"));
        assert_eq!(info.version_id.as_deref(), Some("v1"));
        assert!(info.disabled);

        let unresolved = describe_installed("mine.jar".into(), None, &HashMap::new());
        assert_eq!(unresolved.project_id, None);
        assert_eq!(unresolved.title, None);
    }
}
