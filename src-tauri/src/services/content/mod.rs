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
