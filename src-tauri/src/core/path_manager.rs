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
        PathManager {
            instances_dir: base_dir
                .join(".tflclient")
                .join("instances")
                .into_boxed_path(),
            shared_dir: base_dir.join(".tflclient").join("shared").into_boxed_path(),
            settings_dir: base_dir
                .join(".tflclient")
                .join("settings")
                .into_boxed_path(),
            themes_dir: base_dir.join(".tflclient").join("themes").into_boxed_path(),
            skin_closet_dir: base_dir.join(".tflclient").join("skins").into_boxed_path(),
        }
    }
}

fn resolve_base_dir() -> PathBuf {
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
