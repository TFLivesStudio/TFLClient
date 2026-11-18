//! Bounded discovery adapters for the MultiMC family, CurseForge installed
//! instances and Modrinth's documented SQLite schema. Import/copy and manual
//! source preview are implemented by import_sources, never by brand names.

use crate::services::instance_manager;
use directories::BaseDirs;
use serde::Serialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, command};

/// Qué se copia de la carpeta de juego de la instancia original. Lo demás
/// (logs, crash-reports, versions, libraries, assets…) lo regenera el
/// launcher o no sirve fuera de su launcher de origen.
pub(super) const COPY_ENTRIES: &[&str] = &[
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
    "screenshots",
    "texturepacks",
    "optionsof.txt",
];

#[derive(Debug, Serialize, Clone)]
pub struct ImportCandidate {
    /// Origin label for discovery. The preview selects the format adapter.
    pub source: String,
    pub name: String,
    /// Carpeta raíz de la instancia en su launcher de origen.
    pub path: String,
    pub mc_version: Option<String>,
    pub loader: String,
    pub mod_count: u32,
    pub loader_version: Option<String>,
}

pub(super) struct Detected {
    pub candidate: ImportCandidate,
    pub game_dir: PathBuf,
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
        ("freesm", data.join("FreesmLauncher").join("instances")),
        (
            "freesm",
            home.join(".var/app/org.freesmlauncher.FreesmLauncher/data/FreesmLauncher/instances"),
        ),
        (
            "polymc",
            home.join(".var/app/org.polymc.PolyMC/data/PolyMC/instances"),
        ),
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
            rd.take(50_000)
                .flatten()
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

fn read_small(path: &Path) -> Option<String> {
    const LIMIT: u64 = 2 * 1024 * 1024;
    crate::services::worlds::no_link(path).ok()?;
    let file = std::fs::File::open(path).ok()?;
    let meta = file.metadata().ok()?;
    if !meta.is_file() || meta.len() > LIMIT {
        return None;
    }
    let mut data = String::new();
    file.take(LIMIT + 1).read_to_string(&mut data).ok()?;
    (data.len() as u64 <= LIMIT).then_some(data)
}

/// Prism/MultiMC/PolyMC: `mmc-pack.json` lista los componentes (Minecraft +
/// loader) y `instance.cfg` el nombre. La carpeta de juego es `minecraft/`
/// o `.minecraft/` según la versión del launcher.
pub(super) fn detect_mmc_style(source: &str, dir: &Path) -> Option<Detected> {
    let pack_raw = read_small(&dir.join("mmc-pack.json"))?;
    let pack: serde_json::Value = serde_json::from_str(&pack_raw).ok()?;
    let components = pack.get("components")?.as_array()?;

    let mut mc_version = None;
    let mut loader = "vanilla";
    let mut loader_version = None;
    for c in components {
        let uid = c.get("uid").and_then(|u| u.as_str()).unwrap_or("");
        let version = c.get("version").and_then(|v| v.as_str());
        match uid {
            "net.minecraft" => mc_version = version.map(String::from),
            "net.fabricmc.fabric-loader" => {
                loader = "fabric";
                loader_version = version.map(String::from);
            }
            "org.quiltmc.quilt-loader" => {
                loader = "quilt";
                loader_version = version.map(String::from);
            }
            "net.neoforged" => {
                loader = "neoforge";
                loader_version = version.map(String::from);
            }
            "net.minecraftforge" => {
                loader = "forge";
                loader_version = version.map(String::from);
            }
            _ => {}
        }
    }

    let game_dir = [".minecraft", "minecraft"]
        .iter()
        .map(|d| dir.join(d))
        .find(|d| d.is_dir())?;

    let name = read_small(&dir.join("instance.cfg"))
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
            loader_version,
        },
        game_dir,
    })
}

/// App de CurseForge: `minecraftinstance.json` con `gameVersion` y
/// `baseModLoader.name` ("forge-47.2.0", "fabric-0.15.11-1.20.1"…). La
/// carpeta de juego es la propia carpeta de la instancia.
pub(super) fn detect_curseforge(dir: &Path) -> Option<Detected> {
    let raw = read_small(&dir.join("minecraftinstance.json"))?;
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
    let loader = if loader_name.starts_with("neoforge-") {
        "neoforge"
    } else if loader_name.starts_with("forge-") {
        "forge"
    } else if loader_name.starts_with("fabric-") {
        "fabric"
    } else if loader_name.starts_with("quilt-") {
        "quilt"
    } else if loader_name.is_empty() || loader_name == "vanilla" {
        "vanilla"
    } else {
        &loader_name
    };
    let name = json
        .get("name")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| dir.file_name().and_then(|n| n.to_str()).map(String::from))?;

    let loader_version = loader_name
        .strip_prefix(&format!("{loader}-"))
        .map(|value| {
            let value = value
                .strip_suffix(&format!("-{}", mc_version.as_deref().unwrap_or("")))
                .unwrap_or(value);
            value
                .strip_prefix(&format!("{}-", mc_version.as_deref().unwrap_or("")))
                .unwrap_or(value)
                .to_string()
        });
    Some(Detected {
        candidate: ImportCandidate {
            source: "curseforge".into(),
            name,
            path: dir.to_string_lossy().to_string(),
            mc_version,
            loader: loader.into(),
            mod_count: count_mods(dir),
            loader_version,
        },
        game_dir: dir.to_path_buf(),
    })
}

/// App de Modrinth (Theseus): desde que pasó a guardar sus datos en SQLite
/// (antes era un `profile.json` por instancia) hay que leer `app.db` con una
/// consulta de solo lectura — nunca se escribe ahí. Esquema verificado
/// leyendo el código fuente real de `packages/app-lib` en
/// github.com/modrinth/code (no hay forma de adivinar esto sin leerlo):
/// tabla `instances` (name, path relativo a `profiles/`,
/// applied_content_set_id) y tabla `instance_content_sets` (game_version,
/// loader — ya en minúsculas: "vanilla"/"fabric"/"quilt"/"forge"/"neoforge",
/// mismo vocabulario que usa este archivo para Prism/CurseForge).
///
/// Limitación conocida: si el usuario cambió la carpeta de instancias desde
/// los Ajustes de la app de Modrinth (`custom_dir`), esto no la encuentra —
/// solo mira la ubicación estándar, igual que el resto de este archivo con
/// Prism/CurseForge.
fn detect_modrinth_app() -> Vec<Detected> {
    let Some(base) = BaseDirs::new() else {
        return Vec::new();
    };
    let settings_dir = base.data_dir().join("ModrinthApp");
    let db_path = settings_dir.join("app.db");
    if !db_path.is_file() {
        return Vec::new();
    }
    match query_modrinth_app_db(&settings_dir, &db_path) {
        Ok(rows) => rows,
        Err(e) => {
            // No bloquea el resto de la detección (Prism/CurseForge) por un
            // problema puntual acá — ej. la app de Modrinth tiene el
            // archivo bloqueado en ese instante, o cambió de esquema en una
            // versión nueva.
            tracing::warn!("No se pudo leer instancias de la app de Modrinth: {e}");
            Vec::new()
        }
    }
}

pub(super) fn query_modrinth_app_db(
    settings_dir: &Path,
    db_path: &Path,
) -> rusqlite::Result<Vec<Detected>> {
    if crate::services::worlds::no_link(db_path).is_err() {
        return Err(rusqlite::Error::InvalidQuery);
    }
    let flags =
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = rusqlite::Connection::open_with_flags(db_path, flags)?;
    conn.pragma_update(None, "query_only", true)?;
    conn.pragma_update(None, "trusted_schema", false)?;
    conn.pragma_update(None, "cache_size", -1024)?;
    // Si la app de Modrinth tiene el archivo con un lock de escritura en
    // este instante, no nos quedamos colgados esperando.
    conn.busy_timeout(std::time::Duration::from_millis(500))?;
    let has_loader_version = {
        let mut columns = conn.prepare("PRAGMA table_info(instance_content_sets)")?;
        columns
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(Result::ok)
            .any(|name| name == "loader_version")
    };
    let loader_version_column = if has_loader_version {
        "cs.loader_version"
    } else {
        "NULL"
    };
    let mut stmt = conn.prepare(&format!(
        "SELECT i.name, i.path, cs.game_version, cs.loader, {loader_version_column} \
         FROM instances i \
         LEFT JOIN instance_content_sets cs ON cs.id = i.applied_content_set_id LIMIT 1000"
    ))?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
        ))
    })?;

    let custom_dir = conn
        .query_row("SELECT custom_dir FROM settings LIMIT 1", [], |row| {
            row.get::<_, Option<String>>(0)
        })
        .ok()
        .flatten()
        .map(PathBuf::from)
        .filter(|path| path.is_absolute());
    let profiles_dir = custom_dir
        .unwrap_or_else(|| settings_dir.to_path_buf())
        .join("profiles");
    let mut out = Vec::new();
    for row in rows.flatten() {
        let (name, rel_path, mc_version, loader, loader_version) = row;
        if rel_path.is_empty()
            || rel_path.contains('\\')
            || rel_path
                .split('/')
                .any(|part| crate::services::worlds::valid_name(part).is_err())
        {
            continue;
        }
        let game_dir = profiles_dir.join(&rel_path);
        if !game_dir.is_dir() {
            // Fila vieja de una instancia que ya se borró — la carpeta no
            // existe más.
            continue;
        }
        out.push(Detected {
            candidate: ImportCandidate {
                source: "modrinth_app".into(),
                name,
                path: game_dir.to_string_lossy().to_string(),
                mc_version,
                loader: loader.unwrap_or_else(|| "vanilla".into()),
                mod_count: count_mods(&game_dir),
                loader_version,
            },
            game_dir,
        });
    }
    Ok(out)
}

pub(super) fn detect_all() -> Vec<Detected> {
    let mut out: Vec<Detected> = detect_modrinth_app();
    for (source, root) in launcher_roots() {
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.take(1000).flatten() {
            let dir = entry.path();
            if !dir.is_dir() || crate::services::worlds::no_link(&dir).is_err() {
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
    out.sort_by(|a, b| {
        a.candidate
            .name
            .to_lowercase()
            .cmp(&b.candidate.name.to_lowercase())
    });
    out
}

#[command]
pub async fn detect_importable_instances() -> Result<Vec<ImportCandidate>, String> {
    tokio::task::spawn_blocking(|| detect_all().into_iter().map(|d| d.candidate).collect())
        .await
        .map_err(|e| e.to_string())
}

/// Compatibility route; manual and discovered sources use one verified copier.
#[command]
pub async fn import_external_instance(
    app: AppHandle,
    path: String,
    new_name: String,
) -> Result<instance_manager::InstanceData, String> {
    super::import_sources::import_instance_source(app, path, new_name, None, None).await
}
#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("tfl-import-test-{tag}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn prism_instance_is_detected() {
        let dir = temp_dir("prism");
        std::fs::write(
            dir.join("mmc-pack.json"),
            r#"{"components":[{"uid":"net.minecraft","version":"1.20.1"},{"uid":"net.fabricmc.fabric-loader","version":"0.15.11"}],"formatVersion":1}"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("instance.cfg"),
            "InstanceType=OneSix\nname=Mi Pack\n",
        )
        .unwrap();
        std::fs::create_dir_all(dir.join(".minecraft").join("mods")).unwrap();
        std::fs::write(dir.join(".minecraft").join("mods").join("a.jar"), b"x").unwrap();
        std::fs::write(dir.join(".minecraft").join("mods").join("notes.txt"), b"x").unwrap();

        let d = detect_mmc_style("prism", &dir).expect("debe detectarse");
        assert_eq!(d.candidate.name, "Mi Pack");
        assert_eq!(d.candidate.mc_version.as_deref(), Some("1.20.1"));
        assert_eq!(d.candidate.loader, "fabric");
        assert_eq!(d.candidate.mod_count, 1);
        assert!(d.game_dir.ends_with(".minecraft"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn prism_vanilla_without_game_dir_is_skipped() {
        let dir = temp_dir("prism-nogame");
        std::fs::write(
            dir.join("mmc-pack.json"),
            r#"{"components":[{"uid":"net.minecraft","version":"1.21"}]}"#,
        )
        .unwrap();
        assert!(detect_mmc_style("prism", &dir).is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn curseforge_instance_is_detected() {
        let dir = temp_dir("cf");
        std::fs::write(
            dir.join("minecraftinstance.json"),
            r#"{"name":"All The Mods","gameVersion":"1.20.1","baseModLoader":{"name":"forge-47.2.0"}}"#,
        )
        .unwrap();
        let d = detect_curseforge(&dir).expect("debe detectarse");
        assert_eq!(d.candidate.name, "All The Mods");
        assert_eq!(d.candidate.loader, "forge");
        assert_eq!(d.candidate.mc_version.as_deref(), Some("1.20.1"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn curseforge_neoforge_is_not_confused_with_forge() {
        let dir = temp_dir("cf-neo");
        std::fs::write(
            dir.join("minecraftinstance.json"),
            r#"{"name":"N","gameVersion":"1.21.1","baseModLoader":{"name":"neoforge-21.1.1"}}"#,
        )
        .unwrap();
        assert_eq!(
            detect_curseforge(&dir).unwrap().candidate.loader,
            "neoforge"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    fn make_modrinth_app_db(dir: &Path) -> PathBuf {
        let settings_dir = dir.join("ModrinthApp");
        std::fs::create_dir_all(settings_dir.join("profiles").join("fabric-pack")).unwrap();
        let db_path = settings_dir.join("app.db");
        // Subconjunto mínimo del esquema real de `packages/app-lib`
        // (sqlite migrations) — solo las columnas que `query_modrinth_app_db`
        // lee.
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE instances (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                applied_content_set_id TEXT
            );
            CREATE TABLE instance_content_sets (
                id TEXT PRIMARY KEY,
                instance_id TEXT NOT NULL,
                game_version TEXT NOT NULL,
                loader TEXT NOT NULL
            );
            INSERT INTO instance_content_sets (id, instance_id, game_version, loader)
                VALUES ('cs1', 'i1', '1.21.1', 'fabric');
            INSERT INTO instances (id, name, path, applied_content_set_id)
                VALUES ('i1', 'Mi Pack Fabric', 'fabric-pack', 'cs1');
            -- Instancia sin content set aplicado todavía (recién creada,
            -- vacía) — mc_version/loader deben salir None/vanilla, no
            -- tirar error.
            INSERT INTO instances (id, name, path, applied_content_set_id)
                VALUES ('i2', 'Vacía', 'no-existe-en-disco', NULL);",
        )
        .unwrap();
        settings_dir
    }

    #[test]
    fn modrinth_app_instance_is_detected_with_loader_and_version() {
        let root = temp_dir("mrapp");
        let settings_dir = make_modrinth_app_db(&root);
        let found = query_modrinth_app_db(&settings_dir, &settings_dir.join("app.db")).unwrap();
        assert_eq!(found.len(), 1, "la fila sin carpeta en disco se descarta");
        let d = &found[0];
        assert_eq!(d.candidate.name, "Mi Pack Fabric");
        assert_eq!(d.candidate.source, "modrinth_app");
        assert_eq!(d.candidate.mc_version.as_deref(), Some("1.21.1"));
        assert_eq!(d.candidate.loader, "fabric");
        assert!(d.game_dir.ends_with("fabric-pack"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn modrinth_app_missing_db_file_returns_empty() {
        let root = temp_dir("mrapp-missing");
        let err = query_modrinth_app_db(&root, &root.join("no-existe.db"));
        assert!(err.is_err(), "abrir un archivo que no existe debe fallar");
        let _ = std::fs::remove_dir_all(root);
    }
}
