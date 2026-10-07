//! Reglas de dependencias y de incompatibilidades conocidas. Las decisiones
//! (qué dependencia seguir, cuándo complementar con la lista del proyecto)
//! viven acá; el recorrido recursivo que descarga e instala está en
//! `install.rs`.

use super::ContentService;
use super::providers::ContentProvider;
use super::types::{ContentDependency, ContentVersion, DependencyKind};

/// Profundidad máxima de dependencias de dependencias que se siguen.
pub(crate) const MAX_DEPENDENCY_DEPTH: u8 = 10;

// Grupos de mods mutuamente excluyentes conocidos — Modrinth no expone esta
// relación como metadata estructurada (ni Sodium ni Canvas declaran
// "incompatible" entre sí en la versión ni a nivel de proyecto, se
// verificó contra la API real), así que se mantiene a mano. Cada entrada
// es (id corto canónico, slug) de Modrinth, para no depender de cuál de
// los dos venga en `project_id` según el contexto.
const KNOWN_CONFLICT_GROUPS: &[&[(&str, &str)]] = &[
    // Motores de renderizado alternativos — cada uno reemplaza el
    // renderer de Minecraft entero, no pueden coexistir. Iris queda
    // afuera del grupo a propósito: es compatible con Sodium (de hecho lo
    // requiere), solo Canvas es el que choca.
    &[("AANobbMI", "sodium"), ("VOYxIjFI", "canvas")],
];

pub(crate) fn conflicts_with(project_id: &str, installed_project_id: &str) -> bool {
    KNOWN_CONFLICT_GROUPS.iter().any(|group| {
        let has = |pid: &str| group.iter().any(|(id, slug)| *id == pid || *slug == pid);
        has(project_id) && has(installed_project_id) && project_id != installed_project_id
    })
}

/// Dependencias que obligan a instalar otro proyecto.
pub(crate) fn required_dependencies(
    version: &ContentVersion,
) -> impl Iterator<Item = &ContentDependency> {
    version
        .dependencies
        .iter()
        .filter(|d| d.kind == DependencyKind::Required)
}

impl<P: ContentProvider> ContentService<P> {
    /// Proyecto al que apunta una dependencia: directo, o a través de la
    /// versión puntual que exige. `None` si no declara ninguno de los dos.
    pub(crate) async fn dependency_project_id(
        &self,
        dep: &ContentDependency,
    ) -> Result<Option<String>, String> {
        match (&dep.project_id, &dep.version_id) {
            (Some(p), _) => Ok(Some(p.clone())),
            (None, Some(vid)) => Ok(Some(self.provider.get_version(vid).await?.project_id)),
            (None, None) => Ok(None),
        }
    }

    /// Complemento SOLO cuando la versión puntual no declaró ninguna
    /// dependencia required utilizable (el caso real que motivó esto:
    /// Create Crafts & Additions con `dependencies` vacío en su build más
    /// reciente, aunque el mod sí depende de Create en la práctica). Si la
    /// versión SÍ trae dependencias, se confía en eso y no se toca este
    /// complemento — `required_dependency_project_ids` junta TODO lo que
    /// el proyecto alguna vez declaró en CUALQUIER versión, sin contexto de
    /// a cuál build aplica cada una. Corriéndolo siempre (como era antes)
    /// causó un caso real: un mod que en el pasado soportaba Canvas y migró
    /// a Sodium instalaba los DOS —Sodium y Canvas son motores de
    /// renderizado mutuamente excluyentes— dejando la instancia rota con
    /// "Incompatible mods found!" aunque el usuario nunca pidió Canvas.
    pub(crate) async fn supplementary_dependency_projects(
        &self,
        project_id: &str,
        had_required_dep: bool,
    ) -> Vec<String> {
        if had_required_dep {
            return Vec::new();
        }
        self.provider
            .required_dependency_project_ids(project_id)
            .await
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dep(project: Option<&str>, kind: DependencyKind) -> ContentDependency {
        ContentDependency {
            project_id: project.map(String::from),
            version_id: None,
            kind,
        }
    }

    #[test]
    fn sodium_and_canvas_conflict_by_id_or_slug() {
        assert!(conflicts_with("AANobbMI", "VOYxIjFI"));
        assert!(conflicts_with("sodium", "canvas"));
        assert!(conflicts_with("canvas", "AANobbMI"));
    }

    #[test]
    fn unrelated_or_identical_projects_do_not_conflict() {
        assert!(!conflicts_with("AANobbMI", "AANobbMI"));
        assert!(!conflicts_with("sodium", "sodium"));
        // Iris queda afuera del grupo a propósito.
        assert!(!conflicts_with("iris", "sodium"));
        assert!(!conflicts_with("lithium", "canvas"));
    }

    #[test]
    fn only_required_dependencies_are_followed() {
        let version = ContentVersion {
            id: "v".into(),
            project_id: "p".into(),
            name: "n".into(),
            version_number: String::new(),
            date_published: String::new(),
            changelog: None,
            files: vec![],
            dependencies: vec![
                dep(Some("a"), DependencyKind::Required),
                dep(Some("b"), DependencyKind::Optional),
                dep(Some("c"), DependencyKind::Incompatible),
                dep(Some("d"), DependencyKind::Required),
            ],
        };
        let ids: Vec<_> = required_dependencies(&version)
            .filter_map(|d| d.project_id.as_deref())
            .collect();
        assert_eq!(ids, vec!["a", "d"]);
    }
}
