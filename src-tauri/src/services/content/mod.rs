//! Servicio de contenido de instancias (mods, shaders, resource packs,
//! plugins): buscar, instalar con dependencias, listar, actualizar, deshacer,
//! exportar.
//!
//! Capas:
//!
//! ```text
//! commands/content.rs          (fino: parsea args, llama al servicio)
//!   -> ContentService<P>       (este módulo: lógica común, sin saber de APIs)
//!        -> providers::*       (Modrinth, mañana CurseForge: metadatos y descargas)
//!        -> dependencies / download / install / installed / updates /
//!           rollback / duplicates / export / filesystem
//! ```
//!
//! Solo se abstrae lo que Modrinth y CurseForge realmente comparten: el
//! trait `providers::ContentProvider` es la ÚNICA frontera con la API de un
//! sitio; todo lo demás trabaja con los tipos normalizados de `types`.

pub mod dependencies;
pub mod download;
pub mod duplicates;
pub mod export;
pub mod filesystem;
pub mod install;
pub mod installed;
pub mod local_files;
pub mod project_cache;
pub mod providers;
pub mod rollback;
pub mod search;
pub mod types;
pub mod updates;

use providers::ContentProvider;
use providers::modrinth::ModrinthProvider;

/// Lógica de contenido sobre un proveedor concreto. Los métodos están
/// repartidos por responsabilidad en los submódulos (cada uno con su propio
/// `impl`).
pub struct ContentService<P: ContentProvider> {
    provider: P,
}

impl<P: ContentProvider> ContentService<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }
}

/// El servicio con el proveedor activo hoy (Modrinth). `ModrinthProvider` no
/// tiene estado, así que crearlo por comando no cuesta nada.
pub fn service() -> ContentService<ModrinthProvider> {
    ContentService::new(ModrinthProvider)
}

/// Shared by UI and MCP mutations: official curation is installation provenance, not a loader.
pub(crate) async fn ensure_mutable(instance: &str, kind: types::ContentKind) -> Result<(), String> {
    if matches!(kind, types::ContentKind::Mod | types::ContentKind::Shader) {
        let instance = crate::services::instance_manager::get_instance(instance).await?;
        if crate::services::selection::installer::is_selection_instance(&instance.dir()) {
            if kind == types::ContentKind::Mod {
                return Err("TFL Selection administra este cliente. Usa verificar, reparar o actualizar el cliente completo.".into());
            }
            if !instance
                .selection
                .is_some_and(|identity| identity.shader_support)
            {
                return Err("Este cliente no declara compatibilidad con shaders.".into());
            }
        }
    }
    Ok(())
}

// Shared by Tauri and MCP; no UI-level guard can replace this lock.
pub(crate) async fn mutation_lock(instance: &str) -> tokio::sync::OwnedMutexGuard<()> {
    static LOCKS: std::sync::LazyLock<
        parking_lot::Mutex<
            std::collections::HashMap<String, std::sync::Weak<tokio::sync::Mutex<()>>>,
        >,
    > = std::sync::LazyLock::new(Default::default);
    let lock = {
        let mut locks = LOCKS.lock();
        locks.retain(|_, v| v.strong_count() > 0);
        if let Some(existing) = locks.get(instance).and_then(std::sync::Weak::upgrade) {
            existing
        } else {
            let lock = std::sync::Arc::new(tokio::sync::Mutex::new(()));
            locks.insert(instance.into(), std::sync::Arc::downgrade(&lock));
            lock
        }
    };
    lock.lock_owned().await
}
