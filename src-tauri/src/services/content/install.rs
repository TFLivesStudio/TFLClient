//! Instalación de contenido y, recursivamente, sus dependencias requeridas
//! — el usuario nunca tiene que buscar e instalar una dependencia a mano.
//! Provider-agnóstico: la versión llega normalizada desde el proveedor.

use super::ContentService;
use super::dependencies::{MAX_DEPENDENCY_DEPTH, conflicts_with, required_dependencies};
use super::filesystem::remove_file_unlocked as remove_file;
use super::installed::InstalledProjects;
use super::providers::ContentProvider;
use super::types::ContentKind;
use crate::core::{AppEvent, emit};
use crate::services::activity_tasks::{Operation, Source, Stage, Target};
use std::collections::HashSet;

/// Qué se pidió instalar.
pub struct InstallRequest<'a> {
    pub instance_name: &'a str,
    pub mc_version: &'a str,
    /// Mod loader (mods) o software de servidor (plugins); vacío para el
    /// resto de los tipos.
    pub loader: &'a str,
    pub project_id: &'a str,
    /// `Some` si el usuario eligió una build puntual del dropdown.
    pub version_id: Option<&'a str>,
    pub kind: ContentKind,
}

/// Contexto fijo de una instalación (no cambia al bajar por las
/// dependencias).
struct InstallContext<'a> {
    activity: &'a Operation,
    request: &'a InstallRequest<'a>,
    /// Lo que ya había instalado al empezar.
    installed: &'a InstalledProjects,
}

impl<P: ContentProvider> ContentService<P> {
    pub async fn install(&self, request: InstallRequest<'_>) -> Result<(), String> {
        self.install_from(request, Source::Ui).await.map(|_| ())
    }

    pub async fn install_from(
        &self,
        request: InstallRequest<'_>,
        source: Source,
    ) -> Result<String, String> {
        let target = match request.kind {
            ContentKind::Mod => Target::Mod,
            ContentKind::Shader => Target::Shader,
            ContentKind::ResourcePack => Target::ResourcePack,
            ContentKind::Plugin => Target::Plugin,
        };
        let activity = Operation::content(source, target)?;
        let activity_id = activity.id().to_owned();
        let label = format!("{}: instalación", request.kind.subdir());
        let result = self.install_tracked(request, &activity).await;
        activity.finish(result.is_ok());
        // Existing notifications remain; the floating bar now projects the domain task.
        if result.is_ok() {
            emit(AppEvent::DownloadFinished { task: label });
        } else {
            emit(AppEvent::DownloadFailed {
                task: label,
                error: "No se pudo completar la instalación de contenido".into(),
            });
        }
        result.map(|_| activity_id)
    }

    async fn install_tracked(
        &self,
        request: InstallRequest<'_>,
        activity: &Operation,
    ) -> Result<(), String> {
        let _mutation = super::mutation_lock(request.instance_name).await;
        activity.stage(Stage::Resolving);
        let instance =
            crate::services::instance_manager::get_instance(request.instance_name).await?;
        activity.bind_target(&instance.uuid, None);
        let _perf = crate::core::perf::span("content.install");
        let installed = if request.kind == ContentKind::Mod {
            // Chequeo de conflictos conocidos ANTES de instalar — antes esto
            // lo descubría recién Fabric Loader al lanzar el juego
            // ("Incompatible mods found!"), con el usuario ya instalados los
            // dos a mano sin ningún aviso previo del launcher.
            let already_installed = self
                .installed_info(request.instance_name, request.kind)
                .await?;
            if let Some(conflict) = already_installed.iter().find(|i| {
                i.project_id
                    .as_deref()
                    .is_some_and(|pid| conflicts_with(request.project_id, pid))
            }) {
                let conflict_name = conflict
                    .title
                    .clone()
                    .unwrap_or_else(|| conflict.filename.clone());
                return Err(format!(
                    "Este mod no es compatible con \"{conflict_name}\", que ya tenés instalado — son motores de renderizado alternativos, no pueden convivir. Sacá uno de los dos antes de instalar el otro."
                ));
            }
            super::installed::projects_from_info(&already_installed)
        } else {
            self.installed_projects(request.instance_name, request.kind)
                .await
        };

        let ctx = InstallContext {
            activity,
            request: &request,
            installed: &installed,
        };
        let mut seen = HashSet::new();
        self.install_recursive(&ctx, request.project_id, request.version_id, 0, &mut seen)
            .await
    }

    async fn install_recursive(
        &self,
        ctx: &InstallContext<'_>,
        project_id: &str,
        explicit_version_id: Option<&str>,
        depth: u8,
        seen: &mut HashSet<String>,
    ) -> Result<(), String> {
        let req = ctx.request;
        if depth > MAX_DEPENDENCY_DEPTH || !seen.insert(project_id.to_string()) {
            return Ok(());
        }

        let version = if let Some(vid) = explicit_version_id {
            self.provider.get_version(vid).await?
        } else {
            match self
                .latest_version(
                    project_id,
                    req.mc_version,
                    req.kind.loader_filter(req.loader),
                )
                .await?
            {
                Some(v) => v,
                None => {
                    if depth == 0 {
                        return Err("No compatible content version".into());
                    }
                    tracing::warn!(
                        "No hay versión de {project_id} compatible con {}/{}, se omite",
                        req.mc_version,
                        req.loader
                    );
                    return Ok(());
                }
            }
        };

        if depth == 0 {
            let instance =
                crate::services::instance_manager::get_instance(req.instance_name).await?;
            ctx.activity
                .bind_target(&instance.uuid, Some(&version.project_id));
        }
        // Una dependencia que el usuario ya tiene (de cualquier versión) no se
        // vuelve a bajar: antes se instalaba otra build al lado y el mod quedaba
        // duplicado — el loader se queja de "duplicate mod" y la lista lo muestra
        // dos veces. Solo se pisa si la dependencia exige una versión puntual.
        if depth > 0
            && explicit_version_id.is_none()
            && ctx.installed.contains_key(&version.project_id)
        {
            return Ok(());
        }

        ctx.activity.stage(Stage::Downloading);
        let new_files = super::download::download_version_files_tracked(
            req.instance_name,
            &version,
            req.kind,
            Some(ctx.activity),
        )
        .await?;
        ctx.activity.stage(Stage::Installing);

        // Si ya había OTRA build de este mismo proyecto instalada (incluye sus
        // `-sources.jar` de instalaciones viejas), se saca: instalar otra
        // versión de un mod es reemplazar, no sumar un segundo jar.
        if let Some(old_files) = ctx.installed.get(&version.project_id) {
            for old in old_files {
                if !new_files.contains(old) {
                    let _ = remove_file(req.instance_name, req.kind, old).await;
                }
            }
        }

        let mut had_required_dep = false;
        for dep in required_dependencies(&version) {
            ctx.activity.stage(Stage::Resolving);
            let Some(dep_project) = self.dependency_project_id(dep).await? else {
                continue;
            };
            had_required_dep = true;
            Box::pin(self.install_recursive(
                ctx,
                &dep_project,
                dep.version_id.as_deref(),
                depth + 1,
                seen,
            ))
            .await?;
        }

        for dep_project in self
            .supplementary_dependency_projects(project_id, had_required_dep)
            .await
        {
            Box::pin(self.install_recursive(ctx, &dep_project, None, depth + 1, seen)).await?;
        }

        Ok(())
    }
}
