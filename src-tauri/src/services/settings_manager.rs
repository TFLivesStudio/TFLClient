use crate::core::{AppError, PathManager};
use compact_str::CompactString;
use launchwerk::auth::MinecraftUser;
use parking_lot::{RwLock, RwLockReadGuard};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing::{info, warn};

static SETTINGS: LazyLock<RwLock<SettingsManager>> =
    LazyLock::new(|| RwLock::new(SettingsManager::load()));

fn default_user() -> Vec<MinecraftUser> {
    vec![MinecraftUser::cracked("Steve")]
}
fn default_min_mem() -> u32 {
    1024
}
fn default_max_mem() -> u32 {
    2048
}
fn default_lang() -> CompactString {
    CompactString::from("es")
}
fn default_theme() -> CompactString {
    CompactString::from("dark")
}
fn default_true() -> bool {
    true
}

/// Perfil de calidad visual (Lite/Balanced/Experience). Maneja defaults para
/// los flags de performance granulares; esos flags siguen editables
/// individualmente como override "avanzado".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QualityProfile {
    Lite,
    #[default]
    Balanced,
    Experience,
}

pub struct QualityProfileFlags {
    pub reduce_animations: bool,
    pub disable_blur_effects: bool,
    pub disable_infinite_animations: bool,
}

impl QualityProfile {
    pub fn granular_defaults(&self) -> QualityProfileFlags {
        match self {
            QualityProfile::Lite => QualityProfileFlags {
                reduce_animations: true,
                disable_blur_effects: true,
                disable_infinite_animations: true,
            },
            QualityProfile::Balanced => QualityProfileFlags {
                reduce_animations: false,
                disable_blur_effects: false,
                disable_infinite_animations: true,
            },
            QualityProfile::Experience => QualityProfileFlags {
                reduce_animations: false,
                disable_blur_effects: false,
                disable_infinite_animations: false,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SettingsManager {
    #[serde(default = "default_user")]
    pub user: Vec<MinecraftUser>,
    #[serde(default)]
    pub active_user_idx: usize,
    #[serde(default = "default_min_mem")]
    pub min_memory: u32,
    #[serde(default = "default_max_mem")]
    pub max_memory: u32,
    #[serde(default)]
    pub jre8_path: PathBuf,
    #[serde(default)]
    pub jre17_path: PathBuf,
    #[serde(default)]
    pub jre21_path: PathBuf,
    #[serde(default)]
    pub jre25_path: PathBuf,
    #[serde(default = "default_lang")]
    pub language: CompactString,
    #[serde(default = "default_theme")]
    pub theme: CompactString,
    #[serde(default)]
    pub quality_profile: QualityProfile,
    #[serde(default = "default_true")]
    pub discord_presence: bool,
    /// El usuario ya pasó por la pantalla de bienvenida (eligió cuenta
    /// Microsoft/offline). Antes de eso siempre se muestra el onboarding,
    /// incluso si `user` ya trae el "Steve" cracked por defecto.
    #[serde(default)]
    pub onboarded: bool,
    #[serde(default)]
    pub jvm_args: CompactString,
    #[serde(default)]
    pub env_vars: HashMap<CompactString, String>,
    #[serde(default)]
    pub reduce_animations: bool,
    #[serde(default)]
    pub disable_blur_effects: bool,
    #[serde(default)]
    pub disable_infinite_animations: bool,
    #[serde(skip)]
    pub dirty: bool,
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self {
            user: default_user(),
            active_user_idx: 0,
            min_memory: default_min_mem(),
            max_memory: default_max_mem(),
            jre8_path: PathBuf::new(),
            jre17_path: PathBuf::new(),
            jre21_path: PathBuf::new(),
            jre25_path: PathBuf::new(),
            language: default_lang(),
            theme: default_theme(),
            quality_profile: QualityProfile::default(),
            discord_presence: true,
            onboarded: false,
            jvm_args: CompactString::default(),
            env_vars: HashMap::new(),
            reduce_animations: false,
            disable_blur_effects: false,
            disable_infinite_animations: false,
            dirty: false,
        }
    }
}

impl SettingsManager {
    pub fn read() -> RwLockReadGuard<'static, SettingsManager> {
        SETTINGS.read()
    }

    pub fn write(f: impl FnOnce(&mut SettingsManager)) -> Result<(), AppError> {
        let mut settings = SETTINGS.write();
        f(&mut settings);
        settings.dirty = true;
        Ok(())
    }

    pub fn snapshot() -> SettingsManager {
        SETTINGS.read().clone()
    }

    pub fn apply_quality_profile(profile: QualityProfile) -> Result<(), AppError> {
        let flags = profile.granular_defaults();
        SettingsManager::write(|s| {
            s.quality_profile = profile;
            s.reduce_animations = flags.reduce_animations;
            s.disable_blur_effects = flags.disable_blur_effects;
            s.disable_infinite_animations = flags.disable_infinite_animations;
        })
    }

    pub fn get_user(&self) -> MinecraftUser {
        self.user
            .get(self.active_user_idx)
            .cloned()
            .unwrap_or_else(|| MinecraftUser::cracked("Steve"))
    }

    pub fn add_user(&mut self, user: MinecraftUser) {
        if let Some(idx) = self.user.iter().position(|u| u.uuid == user.uuid) {
            self.user[idx] = user;
        } else {
            self.user.push(user);
            self.active_user_idx = self.user.len() - 1;
        }
    }

    fn settings_path() -> PathBuf {
        PathManager::get().get_settings_dir().join("settings.tfl")
    }

    pub fn load() -> Self {
        let path = Self::settings_path();
        match std::fs::read_to_string(&path) {
            Ok(raw) => match serde_json::from_str::<SettingsManager>(&raw) {
                Ok(s) => {
                    info!("Configuración cargada desde {:?}", path);
                    s
                }
                Err(e) => {
                    warn!("Config corrupta, usando defaults: {}", e);
                    SettingsManager::default()
                }
            },
            Err(_) => {
                info!("No hay archivo de configuración, usando valores por defecto");
                SettingsManager::default()
            }
        }
    }

    pub async fn save() -> Result<(), String> {
        let snapshot = Self::snapshot();
        let path = Self::settings_path();
        let json = serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())?;
        tokio::fs::write(&path, json)
            .await
            .map_err(|e| e.to_string())?;
        SettingsManager::write(|s| s.dirty = false).ok();
        Ok(())
    }
}
