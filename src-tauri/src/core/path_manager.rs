use directories::UserDirs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tracing::info;

static PATH_MANAGER: LazyLock<PathManager> = LazyLock::new(PathManager::initialize);

pub struct PathManager {
    instances_dir: Box<Path>,
    shared_dir: Box<Path>,
    settings_dir: Box<Path>,
    themes_dir: Box<Path>,
    skin_closet_dir: Box<Path>,
}

impl PathManager {
    pub fn get() -> &'static PathManager {
        &PATH_MANAGER
    }

    pub fn get_instance_dir(&self) -> &Path {
        &self.instances_dir
    }
    pub fn get_shared_dir(&self) -> &Path {
        &self.shared_dir
    }
    pub fn get_settings_dir(&self) -> &Path {
        &self.settings_dir
    }
    pub fn get_themes_dir(&self) -> &Path {
        &self.themes_dir
    }
    pub fn get_skin_closet_dir(&self) -> &Path {
        &self.skin_closet_dir
    }

    pub fn ensure_dirs() -> std::io::Result<()> {
        for dir in [
            Self::get().get_instance_dir(),
            Self::get().get_shared_dir(),
            Self::get().get_settings_dir(),
            Self::get().get_themes_dir(),
            Self::get().get_skin_closet_dir(),
        ] {
            std::fs::create_dir_all(dir)?;
            info!("Directorio asegurado: {:?}", dir);
        }
        Ok(())
    }

    fn initialize() -> PathManager {
        let base_dir = resolve_base_dir();
        let identity = if cfg!(feature = "auth-test-client") {
            Some(
                crate::core::microsoft_identity::configured()
                    .expect("invalid auth test build configuration"),
            )
        } else {
            None
        };
        let root = profile_root(&base_dir, identity.as_ref().map(|id| id.client_id.as_str()));
        PathManager {
            instances_dir: root.join("instances").into_boxed_path(),
            shared_dir: root.join("shared").into_boxed_path(),
            settings_dir: root.join("settings").into_boxed_path(),
            themes_dir: root.join("themes").into_boxed_path(),
            skin_closet_dir: root.join("skins").into_boxed_path(),
        }
    }
}

fn profile_root(base: &Path, test_identity: Option<&str>) -> PathBuf {
    match test_identity {
        Some(client) => base
            .join(".tflclient-auth-tests")
            .join(client)
            .join(".TFLClient"),
        None => production_profile_root(base),
    }
}

/// Carpeta de perfil real del jugador (instancias, ajustes, temas, skins).
/// Se renombró de `.tflclient` (todo minúscula) a `.TFLClient` para que
/// coincida con el nombre de marca que ya se muestra en toda la app.
///
/// En Windows y macOS (sistema de archivos insensible a mayúsculas) ambas
/// rutas apuntan a la misma carpeta real: no hace falta mover nada, alcanza
/// con usar el nombre nuevo como string — ahí es donde se ve el nombre viejo
/// (panel de Estado del sistema, por ejemplo). En Linux (sensible a
/// mayúsculas) son carpetas distintas de verdad: si ya existe la vieja y
/// todavía no la nueva, se renombra en el lugar (operación de metadatos en
/// el mismo volumen, no una copia) para no perder instancias ni ajustes.
fn production_profile_root(base: &Path) -> PathBuf {
    let current = base.join(".TFLClient");
    let legacy = base.join(".tflclient");
    if legacy.is_dir() && !current.is_dir() {
        match std::fs::rename(&legacy, &current) {
            Ok(()) => info!("Perfil migrado de {:?} a {:?}", legacy, current),
            Err(e) => {
                tracing::warn!(
                    "No se pudo migrar el perfil de {:?} a {:?} ({e}); se sigue usando el nombre anterior",
                    legacy,
                    current
                );
                return legacy;
            }
        }
    }
    current
}
fn resolve_base_dir() -> PathBuf {
    // Explicit portable/test profile; never exposed as an MCP tool parameter.
    if let Some(root) = std::env::var_os("TFL_DATA_DIR").map(PathBuf::from)
        && root.is_absolute()
        && root.is_dir()
    {
        return root;
    }
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from)
        && home.is_dir()
    {
        return home;
    }
    if let Some(d) = UserDirs::new() {
        return d.home_dir().to_path_buf();
    }
    PathBuf::from(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_test_profile_cannot_point_at_the_production_profile() {
        let base = std::env::temp_dir().join("auth-profile-fixture");
        let production = profile_root(&base, None);
        let test = profile_root(&base, Some(launchwerk::auth::microsoft::DEFAULT_CLIENT_ID));
        assert_ne!(production, test);
        assert!(!test.starts_with(&production));
        assert!(test.ends_with(".TFLClient"));
    }

    #[test]
    fn fresh_install_uses_the_new_capitalized_name() {
        let base = std::env::temp_dir().join(format!("tflclient-fresh-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let root = production_profile_root(&base);
        assert_eq!(root, base.join(".TFLClient"));
        assert!(!base.join(".tflclient").exists());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn existing_lowercase_profile_is_migrated_without_losing_data() {
        let base = std::env::temp_dir().join(format!("tflclient-migrate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let legacy = base.join(".tflclient");
        std::fs::create_dir_all(legacy.join("instances").join("MyWorld")).unwrap();
        std::fs::write(
            legacy.join("instances").join("MyWorld").join("marker.txt"),
            b"keep me",
        )
        .unwrap();

        let root = production_profile_root(&base);

        assert_eq!(root, base.join(".TFLClient"));
        let moved_marker = root.join("instances").join("MyWorld").join("marker.txt");
        assert_eq!(std::fs::read(&moved_marker).unwrap(), b"keep me");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn migrating_twice_is_a_no_op_and_keeps_data_in_place() {
        let base =
            std::env::temp_dir().join(format!("tflclient-migrate-twice-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join(".tflclient")).unwrap();

        let first = production_profile_root(&base);
        std::fs::write(first.join("marker.txt"), b"first run").unwrap();
        let second = production_profile_root(&base);

        assert_eq!(first, second);
        assert_eq!(
            std::fs::read(second.join("marker.txt")).unwrap(),
            b"first run"
        );
        let _ = std::fs::remove_dir_all(&base);
    }
}
