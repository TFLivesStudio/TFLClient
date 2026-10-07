//! Detección y limpieza de mods duplicados.

use super::ContentService;
use super::filesystem::{is_auxiliary_jar, is_disabled_file, remove_file};
use super::providers::ContentProvider;
use super::types::{ContentKind, ContentVersion, InstalledContentInfo};
use crate::services::instance_manager;
use std::collections::HashMap;

/// Agrupa por proyecto los archivos activos; los grupos de más de un archivo
/// son duplicados.
fn duplicate_groups(infos: Vec<InstalledContentInfo>) -> Vec<Vec<String>> {
    let mut by_project: HashMap<String, Vec<String>> = HashMap::new();
    for info in infos {
        if info.disabled {
            continue;
        }
        if let Some(pid) = info.project_id {
            by_project.entry(pid).or_default().push(info.filename);
        }
    }
    by_project.into_values().filter(|v| v.len() > 1).collect()
}

/// De un grupo de builds del mismo proyecto, cuáles borrar: se queda con un
/// jar real (un auxiliar solo sobrevive si no hay ninguno real) y, entre
/// esos, con la build más nueva.
fn files_to_remove(mut group: Vec<(String, ContentVersion)>) -> Vec<String> {
    if group.len() < 2 {
        return Vec::new();
    }
    // El mejor queda último: build más nueva, jar real antes que auxiliar.
    group.sort_by(|(fa, va), (fb, vb)| {
        (!is_auxiliary_jar(fa), &va.date_published, fa).cmp(&(
            !is_auxiliary_jar(fb),
            &vb.date_published,
            fb,
        ))
    });
    group.pop();
    group.into_iter().map(|(filename, _)| filename).collect()
}

impl<P: ContentProvider> ContentService<P> {
    /// Grupos de archivos que resuelven al MISMO mod (dos builds distintas
    /// del mismo mod instaladas a la vez) — el caso de "mods
    /// duplicados/incompatibles" más común y bien definido; detectar choques
    /// entre mods NO relacionados es un problema mucho más especulativo,
    /// fuera de alcance acá.
    pub async fn find_duplicates(&self, instance_name: &str) -> Result<Vec<Vec<String>>, String> {
        let infos = self.installed_info(instance_name, ContentKind::Mod).await?;
        Ok(duplicate_groups(infos))
    }

    /// Deja una sola copia por mod: de cada grupo de duplicados se queda con
    /// la build más nueva (y, a igual versión, con el jar real en vez del
    /// `-sources.jar`) y borra el resto. Devuelve cuántos archivos sacó.
    pub async fn remove_duplicates(&self, instance_name: &str) -> Result<u32, String> {
        let instance = instance_manager::get_instance(instance_name).await?;
        let kind = ContentKind::Mod;
        let files = self
            .resolve_installed_files(&instance, kind.subdir(), kind.extension())
            .await;

        let mut by_project: HashMap<String, Vec<(String, ContentVersion)>> = HashMap::new();
        for (filename, version) in files {
            if is_disabled_file(&filename) {
                continue;
            }
            if let Some(v) = version {
                by_project
                    .entry(v.project_id.clone())
                    .or_default()
                    .push((filename, v));
            }
        }

        let mut removed = 0u32;
        for (_, group) in by_project {
            for filename in files_to_remove(group) {
                if remove_file(instance_name, kind, &filename).await.is_ok() {
                    removed += 1;
                }
            }
        }
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(filename: &str, pid: Option<&str>, disabled: bool) -> InstalledContentInfo {
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

    fn build(date: &str) -> ContentVersion {
        ContentVersion {
            id: String::new(),
            project_id: "P".into(),
            name: String::new(),
            version_number: String::new(),
            date_published: date.into(),
            changelog: None,
            files: vec![],
            dependencies: vec![],
        }
    }

    #[test]
    fn duplicates_ignore_disabled_unresolved_and_singletons() {
        let groups = duplicate_groups(vec![
            info("a-1.jar", Some("A"), false),
            info("a-2.jar", Some("A"), false),
            info("b.jar", Some("B"), false),
            info("b-off.jar.disabled", Some("B"), true),
            info("mine.jar", None, false),
        ]);
        assert_eq!(groups.len(), 1);
        let mut only = groups.into_iter().next().unwrap();
        only.sort();
        assert_eq!(only, vec!["a-1.jar", "a-2.jar"]);
    }

    #[test]
    fn the_newest_build_survives() {
        let removed = files_to_remove(vec![
            ("new.jar".into(), build("2024-02-01")),
            ("old.jar".into(), build("2023-01-01")),
        ]);
        assert_eq!(removed, vec!["old.jar"]);
    }

    #[test]
    fn at_equal_date_the_real_jar_beats_the_sources_jar() {
        let removed = files_to_remove(vec![
            ("a.jar".into(), build("2024-02-01")),
            ("a-sources.jar".into(), build("2024-02-01")),
        ]);
        assert_eq!(removed, vec!["a-sources.jar"]);
    }

    #[test]
    fn a_real_jar_beats_a_newer_auxiliary_one() {
        let removed = files_to_remove(vec![
            ("a.jar".into(), build("2023-01-01")),
            ("a-sources.jar".into(), build("2024-02-01")),
        ]);
        assert_eq!(removed, vec!["a-sources.jar"]);
    }

    #[test]
    fn a_single_file_is_never_removed() {
        assert!(files_to_remove(vec![("a.jar".into(), build("2024-02-01"))]).is_empty());
    }
}
