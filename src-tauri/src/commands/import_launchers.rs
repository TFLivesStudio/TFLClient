//! Importar instancias que el usuario ya tiene en otros launchers (Prism /
//! MultiMC / PolyMC y la app de CurseForge) — así quien se pasa a TFL Client
//! no rearma a mano cada instancia. Se detectan solas en las carpetas
//! estándar de cada launcher: no hace falta ningún diálogo de archivos
//! (que en modo Automático está deshabilitado a propósito).

use crate::services::instance_manager;
use directories::BaseDirs;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, command};

/// Qué se copia de la carpeta de juego de la instancia original. Lo demás
/// (logs, crash-reports, versions, libraries, assets…) lo regenera el
/// launcher o no sirve fuera de su launcher de origen.
const COPY_ENTRIES: &[&str] = &[
    "mods",
    "config",
    "defaultconfigs",
    "kubejs",
    "scripts",
    "resourcepacks",
    "shaderpacks",
    "saves",
    "options.txt",
    "optionsshaders.txt",
    "servers.dat",
];

#[derive(Debug, Serialize, Clone)]
pub struct ImportCandidate {
    /// "prism" | "multimc" | "polymc" | "curseforge"
    pub source: String,
    pub name: String,
    /// Carpeta raíz de la instancia en su launcher de origen — identifica al
    /// candidato (el import la vuelve a buscar entre los detectados, nunca
    /// copia de una ruta arbitraria).
    pub path: String,
    pub mc_version: Option<String>,
    pub loader: String,
    pub mod_count: u32,
}

struct Detected {
    candidate: ImportCandidate,
    game_dir: PathBuf,
}

fn launcher_roots() -> Vec<(&'static str, PathBuf)> {
    let Some(base) = BaseDirs::new() else {
        return Vec::new();
    };
    let home = base.home_dir().to_path_buf();
    let data = base.data_dir().to_path_buf();
    vec![
        ("prism", data.join("PrismLauncher").join("instances")),
        (
            "prism",
            home.join(".var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher/instances"),
        ),
        ("multimc", data.join("MultiMC").join("instances")),
        ("multimc", data.join("multimc").join("instances")),
        ("polymc", data.join("PolyMC").join("instances")),
        ("polymc", data.join("polymc").join("instances")),
        (
            "curseforge",
            home.join("curseforge").join("minecraft").join("Instances"),
        ),
        (
            "curseforge",
            home.join("Documents")
                .join("curseforge")
                .join("minecraft")
                .join("Instances"),
        ),
    ]
}

fn count_mods(game_dir: &Path) -> u32 {
    std::fs::read_dir(game_dir.join("mods"))
        .map(|rd| {
            rd.flatten()
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|x| x.to_str())
                        .is_some_and(|x| x.eq_ignore_ascii_case("jar"))
                })
                .count() as u32
        })
        .unwrap_or(0)
}

/// Prism/MultiMC/PolyMC: `mmc-pack.json` lista los componentes (Minecraft +
/// loader) y `instance.cfg` el nombre. La carpeta de juego es `minecraft/`
/// o `.minecraft/` según la versión del launcher.
fn detect_mmc_style(source: &str, dir: &Path) -> Option<Detected> {
    let pack_raw = std::fs::read_to_string(dir.join("mmc-pack.json")).ok()?;
    let pack: serde_json::Value = serde_json::from_str(&pack_raw).ok()?;
    let components = pack.get("components")?.as_array()?;

    let mut mc_version = None;
    let mut loader = "vanilla";
    for c in components {
        let uid = c.get("uid").and_then(|u| u.as_str()).unwrap_or("");
        let version = c.get("version").and_then(|v| v.as_str());
        match uid {
            "net.minecraft" => mc_version = version.map(String::from),
            "net.fabricmc.fabric-loader" => loader = "fabric",
            "org.quiltmc.quilt-loader" => loader = "quilt",
            "net.neoforged" => loader = "neoforge",
            "net.minecraftforge" => loader = "forge",
            _ => {}
        }
    }

    let game_dir = ["minecraft", ".minecraft"]
        .iter()
        .map(|d| dir.join(d))
        .find(|d| d.is_dir())?;

    let name = std::fs::read_to_string(dir.join("instance.cfg"))
        .ok()
        .and_then(|cfg| {
            cfg.lines()
                .find_map(|l| l.strip_prefix("name=").map(|n| n.trim().to_string()))
        })
        .filter(|n| !n.is_empty())
        .or_else(|| dir.file_name().and_then(|n| n.to_str()).map(String::from))?;

    Some(Detected {
        candidate: ImportCandidate {
            source: source.into(),
            name,
            path: dir.to_string_lossy().to_string(),
            mc_version,
            loader: loader.into(),
            mod_count: count_mods(&game_dir),
        },
        game_dir,
    })
}

/// App de CurseForge: `minecraftinstance.json` con `gameVersion` y
/// `baseModLoader.name` ("forge-47.2.0", "fabric-0.15.11-1.20.1"…). La
/// carpeta de juego es la propia carpeta de la instancia.
fn detect_curseforge(dir: &Path) -> Option<Detected> {
    let raw = std::fs::read_to_string(dir.join("minecraftinstance.json")).ok()?;
    let json: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let mc_version = json
        .get("gameVersion")
        .and_then(|v| v.as_str())
        .map(String::from);
    let loader_name = json
        .pointer("/baseModLoader/name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    // "neoforge" antes que "forge": "neoforge-…" no empieza con "forge-"
    // pero conviene que el orden no dependa de eso.
    let loader = if loader_name.starts_with("neoforge") {
        "neoforge"
    } else if loader_name.starts_with("forge") {
        "forge"
    } else if loader_name.starts_with("fabric") {
        "fabric"
    } else if loader_name.starts_with("quilt") {
        "quilt"
    } else {
        "vanilla"
    };
    let name = json
        .get("name")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| dir.file_name().and_then(|n| n.to_str()).map(String::from))?;

    Some(Detected {
        candidate: ImportCandidate {
            source: "curseforge".into(),
            name,
            path: dir.to_string_lossy().to_string(),
            mc_version,
            loader: loader.into(),
            mod_count: count_mods(dir),
        },
        game_dir: dir.to_path_buf(),
    })
}

fn detect_all() -> Vec<Detected> {
    let mut out: Vec<Detected> = Vec::new();
    for (source, root) in launcher_roots() {
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let found = if source == "curseforge" {
                detect_curseforge(&dir)
            } else {
                detect_mmc_style(source, &dir)
            };
            if let Some(d) = found {
                // El mismo launcher puede aparecer por dos rutas (ej. Prism
                // nativo y flatpak apuntando a lo mismo) — sin repetir.
                if !out.iter().any(|o| o.candidate.path == d.candidate.path) {
                    out.push(d);
                }
            }
        }
    }
    out.sort_by(|a, b| a.candidate.name.to_lowercase().cmp(&b.candidate.name.to_lowercase()));
    out
}

#[command]
pub async fn detect_importable_instances() -> Result<Vec<ImportCandidate>, String> {
    tokio::task::spawn_blocking(|| detect_all().into_iter().map(|d| d.candidate).collect())
        .await
        .map_err(|e| e.to_string())
}

fn copy_recursive(from: &Path, to: &Path) -> std::io::Result<()> {
    if from.is_dir() {
        std::fs::create_dir_all(to)?;
        for entry in std::fs::read_dir(from)?.flatten() {
            copy_recursive(&entry.path(), &to.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(from, to).map(|_| ())
    }
}

/// Crea una instancia nueva con la versión y loader de la original y le
/// copia mods, configs, mundos, packs y opciones. La original no se toca.
/// El loader se instala en su última versión para esa versión de
/// Minecraft (no se replica la versión exacta del launcher de origen).
#[command]
pub async fn import_external_instance(
    app: AppHandle,
    path: String,
    new_name: String,
) -> Result<instance_manager::InstanceData, String> {
    let detected = tokio::task::spawn_blocking(detect_all)
        .await
        .map_err(|e| e.to_string())?;
    let found = detected
        .into_iter()
        .find(|d| d.candidate.path == path)
        .ok_or("Esa instancia ya no está disponible en su launcher de origen")?;
    let mc_version = found
        .candidate
        .mc_version
        .clone()
        .ok_or("No se pudo detectar la versión de Minecraft de esa instancia")?;

    let created = crate::commands::instance::create_instance(
        app,
        new_name,
        mc_version,
        found.candidate.loader.clone(),
    )
    .await?;

    let dest = created.dir();
    let game_dir = found.game_dir;
    let copy_result = tokio::task::spawn_blocking(move || -> std::io::Result<()> {
        for entry in COPY_ENTRIES {
            let from = game_dir.join(entry);
            if from.exists() {
                copy_recursive(&from, &dest.join(entry))?;
            }
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?;

    if let Err(e) = copy_result {
        // Mejor no dejar una instancia a medio copiar como si estuviera bien.
        let _ = instance_manager::delete_instance(&created.name).await;
        return Err(format!("No se pudo copiar el contenido de la instancia: {e}"));
    }
    Ok(created)
}
