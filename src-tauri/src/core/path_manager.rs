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
            .join(".tflclient"),
        None => base.join(".tflclient"),
    }
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
        assert!(test.ends_with(".tflclient"));
    }
}
