//! TFL Selection: biblioteca oficial de clientes con distribución firmada (catálogo Ed25519 v2 +
//! manifiestos inmutables verificados por SHA-256).
//!
//! Contrato: <https://github.com/TFLivesStudio/TFLClientSelection> →
//! `docs/LAUNCHER_CONTRACT.md`. Este módulo NO usa Modrinth, CurseForge,
//! `.mrpack` ni `latest`: el catálogo firmado es la única fuente autorizada.

pub mod catalog;
pub mod installer;
pub mod manifest;
pub mod net;
pub mod signatures;
pub mod util;

#[cfg(test)]
mod tests;
