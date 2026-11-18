use crate::core::{AppError, PathManager};
use compact_str::CompactString;
use launchwerk::auth::MinecraftUser;
use parking_lot::{RwLock, RwLockReadGuard};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing::{info, warn};

static SAVE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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
fn default_dialog_mode() -> CompactString {
    CompactString::from("auto")
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiMode {
    #[default]
    Simple,
    Advanced,
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
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub settings_revision: u64,
    #[serde(default)]
    pub ui_mode: UiMode,
    /// Keys explicitly chosen by the user; absent keys retain their versioned default.
    #[serde(default)]
    pub preference_origins: HashMap<String, String>,
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
    /// Canonical locale; None migrates the prior WebView language choice once.
    #[serde(default)]
    pub locale: Option<String>,
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
    /// Si está prendido, el launcher chequea/descarga actualizaciones
    /// solo al abrir (en segundo plano, sin bloquear nada) y avisa con
    /// un badge cuando ya está lista para instalar — instalar sigue
    /// siendo una acción explícita del usuario (click en el badge o en
    /// Ajustes), nunca se reemplaza el binario sin que lo pida. Ver
    /// `updateState.svelte.ts` en el frontend.
    #[serde(default = "default_true")]
    pub auto_updates: bool,
    /// Canal de actualizaciones: `true` = también recibe las versiones beta
    /// (y las estables). Se activa a propósito desde Ajustes, con un aviso de
    /// que pueden traer errores — ver `commands/updater.rs`.
    #[serde(default = "default_true")]
    pub beta_updates: bool,
    /// "auto" (default) oculta/deshabilita en el frontend cualquier función
    /// que dependa del diálogo nativo de archivos (subir ícono/wallpaper
    /// propio, agregar mods/shaders/resourcepacks por archivo) — el diálogo
    /// nativo es lo que rompe el launcher en macOS con hardened runtime +
    /// firma ad-hoc (ver docs/history/legacy/CHANGELOG_macos-dialog-crash-java26.txt).
    /// "manual" las habilita, a sabiendas del riesgo. No afecta Java/loader/mods por
    /// Modrinth, que siguen siendo 100% automáticos siempre.
    #[serde(default = "default_dialog_mode")]
    pub native_dialog_mode: CompactString,
    /// Si ya se le mostró al usuario la elección de modo (modal de
    /// bienvenida en instalación nueva, banner una vez en actualización) —
    /// evita volver a preguntar. No afecta el valor de `native_dialog_mode`
    /// en sí, que sigue editable en Ajustes en cualquier momento.
    #[serde(default)]
    pub native_dialog_mode_prompted: bool,
    /// Si ya se le mostró la explicación de "cómo se conectan los demás a
    /// tu servidor" (UPnP automático vs. playit.gg como fallback manual) —
    /// aparece como modal la primera vez que se crea una instancia de
    /// servidor, para que no sea sorpresa si UPnP no coopera. Una sola vez
    /// por instalación, igual que `native_dialog_mode_prompted`.
    #[serde(default)]
    pub server_connection_info_shown: bool,
    /// "No volver a preguntarme" del aviso de multi-instancia (abrir una
    /// instancia de cliente mientras ya hay otra corriendo) — editable en
    /// cualquier momento desde Ajustes, igual que `native_dialog_mode`.
    #[serde(default = "default_true")]
    pub multi_instance_warning_dismissed: bool,
    /// Sidebar en modo compacto (solo íconos) — editable desde el botón al
    /// pie del sidebar, se recuerda entre reinicios.
    #[serde(default)]
    pub sidebar_collapsed: bool,
    #[serde(skip)]
    pub dirty: bool,
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self {
            schema_version: 2,
            settings_revision: 0,
            ui_mode: UiMode::Simple,
            preference_origins: HashMap::new(),
            user: default_user(),
            active_user_idx: 0,
            min_memory: default_min_mem(),
            max_memory: default_max_mem(),
            jre8_path: PathBuf::new(),
            jre17_path: PathBuf::new(),
            jre21_path: PathBuf::new(),
            jre25_path: PathBuf::new(),
            language: default_lang(),
            locale: None,
            theme: default_theme(),
            quality_profile: QualityProfile::default(),
            discord_presence: true,
            onboarded: false,
            jvm_args: CompactString::default(),
            env_vars: HashMap::new(),
            reduce_animations: false,
            disable_blur_effects: false,
            disable_infinite_animations: false,
            auto_updates: true,
            beta_updates: true,
            native_dialog_mode: default_dialog_mode(),
            native_dialog_mode_prompted: false,
            server_connection_info_shown: false,
            multi_instance_warning_dismissed: true,
            sidebar_collapsed: false,
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
                Ok(mut s) => {
                    if s.schema_version < 2 {
                        // A prior profile used false for missing keys. Explicit values survive.
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
                            s.beta_updates = value
                                .get("beta_updates")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            s.multi_instance_warning_dismissed = value
                                .get("multi_instance_warning_dismissed")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            if let Some(fields) = value.as_object() {
                                for key in fields.keys() {
                                    s.preference_origins
                                        .insert(key.clone(), "existing-profile".into());
                                }
                            }
                        }
                        s.schema_version = 2;
                    }
                    info!("Configuración cargada desde {:?}", path);
                    s
                }
                Err(e) => {
                    // Antes esto pisaba directo con defaults en la próxima
                    // `save()` — cuentas, RAM, jvm_args, todo perdido sin
                    // dejar rastro. Se deja una copia del archivo corrupto
                    // al lado (best-effort, si falla no bloquea el arranque)
                    // por si hay algo rescatable a mano.
                    let backup_path = path.with_extension("tfl.corrupto");
                    if std::fs::write(&backup_path, &raw).is_ok() {
                        warn!(
                            "Config corrupta ({e}), usando defaults — copia guardada en {:?}",
                            backup_path
                        );
                    } else {
                        warn!("Config corrupta ({e}), usando defaults — no se pudo ni respaldar");
                    }
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
        let _save_guard = SAVE_LOCK.lock().await;
        let snapshot = Self::snapshot();
        let path = Self::settings_path();
        let json = serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())?;
        // Escritura directa: si el proceso se corta a mitad de camino
        // (crash, corte de luz, force-quit) el archivo queda truncado y
        // `load()` lo descarta entero la próxima vez. Escribir a un
        // temporal y renombrar es atómico en el mismo filesystem — o el
        // rename se completa entero, o el archivo viejo queda intacto.
        let tmp_path = path.with_extension("tfl.tmp");
        tokio::fs::write(&tmp_path, &json)
            .await
            .map_err(|e| e.to_string())?;
        tokio::fs::rename(&tmp_path, &path)
            .await
            .map_err(|e| e.to_string())?;
        SettingsManager::write(|s| s.dirty = false).ok();
        Ok(())
    }

    /// Ordered field patches prevent a stale window from overwriting unrelated preferences.
    /// Disk commit precedes the in-memory commit; a failed write leaves the last valid value.
    pub async fn patch(patch: serde_json::Value, expected_revision: u64) -> Result<Self, String> {
        let _save_guard = SAVE_LOCK.lock().await;
        tokio::task::spawn_blocking(move || {
            let mut current = SETTINGS.write();
            if current.settings_revision != expected_revision {
                return Err("SETTINGS_CONFLICT".into());
            }
            let fields = patch
                .as_object()
                .ok_or("El cambio de ajustes debe ser un objeto")?;
            let mut value = serde_json::to_value(&*current).map_err(|e| e.to_string())?;
            let target = value.as_object_mut().ok_or("Ajustes inválidos")?;
            for (key, value) in fields {
                if matches!(
                    key.as_str(),
                    "user"
                        | "active_user_idx"
                        | "onboarded"
                        | "dirty"
                        | "schema_version"
                        | "settings_revision"
                        | "preference_origins"
                ) || !target.contains_key(key)
                {
                    return Err("Ese campo no se puede modificar como preferencia".into());
                }
                target.insert(key.clone(), value.clone());
            }
            let mut next: Self =
                serde_json::from_value(value).map_err(|_| "El valor del ajuste no es válido")?;
            next.validate()?;
            next.schema_version = 2;
            next.settings_revision = current
                .settings_revision
                .checked_add(1)
                .ok_or("Revisión fuera de rango")?;
            for key in fields.keys() {
                next.preference_origins
                    .insert(key.clone(), "explicit".into());
            }
            let path = Self::settings_path();
            let tmp = path.with_extension("tfl.patch.tmp");
            let json = serde_json::to_vec_pretty(&next).map_err(|e| e.to_string())?;
            use std::io::Write;
            let mut file = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
            file.write_all(&json)
                .and_then(|_| file.sync_all())
                .map_err(|e| e.to_string())?;
            std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
            next.dirty = false;
            *current = next.clone();
            Ok(next)
        })
        .await
        .map_err(|_| "No se pudieron guardar los ajustes".to_string())?
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.locale.as_ref().is_some_and(|l| {
            l.len() > 64
                || l.is_empty()
                || l.split('-').any(|part| {
                    part.is_empty()
                        || part.len() > 8
                        || !part.bytes().all(|b| b.is_ascii_alphanumeric())
                })
        }) {
            return Err("El código de idioma no es válido".into());
        }
        if self.min_memory < 512 || self.max_memory < self.min_memory || self.max_memory > 1_048_576
        {
            return Err(
                "La memoria debe ser al menos 512 MiB y el máximo debe ser mayor o igual al mínimo"
                    .into(),
            );
        }
        if !matches!(self.theme.as_str(), "dark" | "light")
            || !matches!(self.native_dialog_mode.as_str(), "auto" | "manual")
        {
            return Err("Opción de apariencia o diálogo inválida".into());
        }
        if self.language.len() > 64
            || self.language.is_empty()
            || self.jvm_args.len() > 32_768
            || self.jvm_args.contains('\0')
            || self.env_vars.len() > 128
        {
            return Err("El ajuste supera los límites permitidos".into());
        }
        if self.env_vars.iter().any(|(key, value)| {
            key.is_empty()
                || key.contains(['=', '\0'])
                || value.contains('\0')
                || value.len() > 32_768
        }) {
            return Err("Variable de entorno inválida".into());
        }
        Ok(())
    }
}
