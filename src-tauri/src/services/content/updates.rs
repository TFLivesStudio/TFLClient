//! Chequeo de actualizaciones de mods y "actualizar todo" con rollback.

use super::ContentService;
use super::download::download_version_files;
use super::filesystem::{hash_installed_files, is_disabled_file};
use super::providers::ContentProvider;
use super::rollback::{RollbackRecorder, rollback_dir};
use super::types::{ContentKind, ContentUpdate, ContentVersion};
use crate::services::instance_manager;
use crate::services::world_backup::backup_world_dir;

fn instance_loader_str(instance: &instance_manager::InstanceData) -> String {
    serde_json::to_value(instance.loader)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

/// `Some` si `newest` (la build más nueva que informó el proveedor) es
/// distinta del archivo instalado (`installed_sha1`).
fn pending_update(
    filename: String,
    installed_sha1: &str,
    newest: &ContentVersion,
) -> Option<ContentUpdate> {
    // El proveedor devuelve la versión más nueva pase lo que pase — si
    // coincide con el hash ya instalado, en realidad no hay update.
    let already_current = newest.files.iter().any(|f| f.has_sha1(installed_sha1));
    if already_current {
        return None;
    }
    Some(ContentUpdate {
        filename,
        project_id: newest.project_id.clone(),
        title: newest.name.clone(),
        new_version_id: newest.id.clone(),
    })
}

impl<P: ContentProvider> ContentService<P> {
    /// Mods instalados con una versión más nueva disponible para la versión
    /// de Minecraft + loader de ESTA instancia puntual — usa el endpoint del
    /// proveedor pensado justo para esto (resuelve por hash y devuelve la
    /// última build compatible, sin tener que buscar mod por mod).
    pub async fn check_updates(&self, instance_name: &str) -> Result<Vec<ContentUpdate>, String> {
        let instance = instance_manager::get_instance(instance_name).await?;
        let hashed = hash_installed_files(&instance.dir().join("mods"), "jar").await;
        if hashed.is_empty() {
            return Ok(Vec::new());
        }
        let hashes: Vec<String> = hashed.iter().map(|(_, h)| h.clone()).collect();
        let loader_str = instance_loader_str(&instance);
        let latest = self
            .provider
            .latest_versions_from_hashes(&hashes, &loader_str, &instance.mc_version)
            .await
            .unwrap_or_default();

        let mut out = Vec::new();
        for (filename, hash) in hashed {
            // Un mod apagado a propósito no se actualiza: al bajar la build
            // nueva quedaría activo otra vez sin que el usuario lo pida.
            if is_disabled_file(&filename) {
                continue;
            }
            let Some(newest) = latest.get(&hash) else {
                continue;
            };
            if let Some(update) = pending_update(filename, &hash, newest) {
                out.push(update);
            }
        }
        Ok(out)
    }

    /// Actualiza todos los mods con versión nueva disponible de una — sigue
    /// de largo si un mod puntual falla (red, archivo corrupto, etc), no
    /// aborta el resto del lote. Devuelve cuántos se actualizaron de verdad.
    pub async fn update_all(&self, instance_name: &str) -> Result<u32, String> {
        let updates = self.check_updates(instance_name).await?;
        if updates.is_empty() {
            return Ok(0);
        }
        let instance = instance_manager::get_instance(instance_name).await?;
        // Backup del mundo antes de tocar mods — un mod nuevo puede romper un
        // mundo existente (cambios de formato de chunk, IDs de bloque
        // distintos, etc). Best-effort: si falla (sin carpeta saves/, error de
        // disco) no bloquea la actualización, solo no queda backup.
        let _ = backup_world_dir(&instance.dir()).await;

        let mut rollback = RollbackRecorder::begin(rollback_dir(&instance.dir())).await;

        let mods_dir = instance.dir().join("mods");
        let mut count = 0u32;
        for update in updates {
            let Ok(version) = self.provider.get_version(&update.new_version_id).await else {
                continue;
            };
            let Ok(new_files) =
                download_version_files(instance_name, &version, ContentKind::Mod).await
            else {
                continue;
            };
            if !new_files.contains(&update.filename) {
                rollback
                    .retire(&mods_dir, &update.filename, new_files)
                    .await;
            }
            count += 1;
        }

        rollback.finish().await;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::content::types::ContentFile;

    fn newest(sha1s: &[&str]) -> ContentVersion {
        ContentVersion {
            id: "new".into(),
            project_id: "P".into(),
            name: "Mod 2.0".into(),
            version_number: "2.0".into(),
            date_published: String::new(),
            changelog: None,
            files: sha1s
                .iter()
                .map(|h| ContentFile {
                    url: String::new(),
                    filename: format!("{h}.jar"),
                    primary: true,
                    sha1: Some((*h).into()),
                    sha512: None,
                    size: None,
                })
                .collect(),
            dependencies: vec![],
        }
    }

    #[test]
    fn same_hash_means_no_update() {
        assert!(pending_update("a.jar".into(), "abc", &newest(&["abc"])).is_none());
    }

    #[test]
    fn different_hash_reports_the_new_version() {
        let u = pending_update("a.jar".into(), "old", &newest(&["new1"])).unwrap();
        assert_eq!(u.filename, "a.jar");
        assert_eq!(u.project_id, "P");
        assert_eq!(u.title, "Mod 2.0");
        assert_eq!(u.new_version_id, "new");
    }
}
