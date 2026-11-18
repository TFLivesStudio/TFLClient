//! Private instance snapshots and portable server exports use separate formats.
use super::{
    instance_manager::{self, InstanceData},
    server_ports, worlds,
};
use crate::core::PathManager;
use serde_json::json;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};
use zip::{ZipWriter, write::SimpleFileOptions};

const MAX_BYTES: u64 = 64 * 1024 * 1024 * 1024;
const MAX_FILES: usize = 100_000;

fn plain(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.ends_with([' ', '.'])
        && !name
            .chars()
            .any(|c| c.is_control() || "/\\:*?\"<>|".contains(c))
}

fn is_link(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    meta.file_type().is_symlink()
}

struct Archive {
    zip: ZipWriter<File>,
    files: usize,
    bytes: u64,
}

impl Archive {
    fn add(&mut self, source: &Path, relative: &str) -> Result<(), String> {
        let meta = fs::symlink_metadata(source).map_err(error)?;
        if is_link(&meta) || !meta.is_file() {
            return Err(format!("El archivo {relative} no es un archivo normal"));
        }
        self.files += 1;
        self.bytes = self
            .bytes
            .checked_add(meta.len())
            .ok_or("Tamaño fuera de rango")?;
        if self.files > MAX_FILES || self.bytes > MAX_BYTES {
            return Err("La instancia supera el límite de respaldo".into());
        }
        self.zip
            .start_file(
                relative,
                SimpleFileOptions::default().large_file(meta.len() >= u64::from(u32::MAX)),
            )
            .map_err(error)?;
        let mut input = File::open(source).map_err(error)?;
        let written = io::copy(
            &mut std::io::Read::take(&mut input, meta.len().saturating_add(1)),
            &mut self.zip,
        )
        .map_err(error)?;
        if written != meta.len() {
            return Err(format!(
                "El archivo {relative} cambió mientras se respaldaba"
            ));
        }
        Ok(())
    }
    fn tree(&mut self, root: &Path, rel: &str, depth: usize) -> Result<(), String> {
        if depth > 64 {
            return Err("La instancia tiene demasiados niveles de carpetas".into());
        }
        for child in fs::read_dir(root).map_err(error)? {
            let child = child.map_err(error)?;
            let name = child
                .file_name()
                .into_string()
                .map_err(|_| "Nombre de archivo no compatible")?;
            if !plain(&name) {
                return Err("Nombre de archivo no compatible".into());
            }
            if name.starts_with(".tfl-server-") && name.ends_with(".part") {
                continue;
            }
            let relative = if rel.is_empty() {
                name
            } else {
                format!("{rel}/{name}")
            };
            let meta = fs::symlink_metadata(child.path()).map_err(error)?;
            if is_link(&meta) {
                return Err(format!("El respaldo contiene un enlace: {relative}"));
            }
            if meta.is_dir() {
                self.tree(&child.path(), &relative, depth + 1)?;
            } else {
                self.add(&child.path(), &relative)?;
            }
        }
        Ok(())
    }
    fn text(&mut self, name: &str, bytes: &[u8]) -> Result<(), String> {
        self.zip
            .start_file(name, SimpleFileOptions::default())
            .map_err(error)?;
        self.zip.write_all(bytes).map_err(error)
    }
}

fn error(value: impl std::fmt::Display) -> String {
    value.to_string()
}

// Use a positive allowlist. Local addresses, EULA, RCON and arbitrary MOTD/config
// values may disclose private content even when they have an innocuous name.
pub(crate) fn portable_properties(input: &str) -> Result<String, String> {
    let properties = server_ports::properties(input)?;
    let mut result = String::from("enable-rcon=false\nenable-query=false\n");
    for key in [
        "online-mode",
        "hardcore",
        "pvp",
        "allow-nether",
        "spawn-protection",
        "max-players",
        "view-distance",
        "simulation-distance",
        "gamemode",
        "difficulty",
    ] {
        let Some(value) = properties.get(key) else {
            continue;
        };
        let valid = match key {
            "online-mode" | "hardcore" | "pvp" | "allow-nether" => {
                matches!(value.as_str(), "true" | "false")
            }
            "gamemode" => matches!(
                value.as_str(),
                "survival" | "creative" | "adventure" | "spectator" | "0" | "1" | "2" | "3"
            ),
            "difficulty" => matches!(
                value.as_str(),
                "peaceful" | "easy" | "normal" | "hard" | "0" | "1" | "2" | "3"
            ),
            _ => value.len() <= 5 && value.parse::<u16>().is_ok(),
        };
        if valid {
            result.push_str(&format!("{key}={value}\n"));
        }
    }
    Ok(result)
}

fn archive(
    data: InstanceData,
    portable: bool,
    path: PathBuf,
    properties: Option<String>,
) -> Result<String, String> {
    let root = data.dir();
    if is_link(&fs::symlink_metadata(&root).map_err(error)?) {
        return Err("La instancia es un enlace".into());
    }
    fs::create_dir_all(path.parent().ok_or("Destino inválido")?).map_err(error)?;
    let tmp = path.with_extension("partial");
    let result = (|| {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(error)?;
        let mut archive = Archive {
            zip: ZipWriter::new(file),
            files: 0,
            bytes: 0,
        };
        if portable {
            let jar = data
                .server_jar
                .as_deref()
                .filter(|j| plain(j) && j.ends_with(".jar") && !j.starts_with('.'))
                .ok_or("No hay un JAR de servidor válido")?;
            let kind = data.server_type.ok_or("Esta instancia no es un servidor")?;
            archive.text(
                "tfl-server.json",
                &serde_json::to_vec_pretty(&json!({
                    "format_version": 1, "name": data.name, "mc_version": data.mc_version,
                    "server_type": kind, "server_jar": jar, "server_build": data.server_build,
                }))
                .map_err(error)?,
            )?;
            archive.add(&root.join(jar), jar)?;
            archive.text(
                "server.properties",
                properties
                    .as_deref()
                    .ok_or("Faltan las propiedades de exportación")?
                    .as_bytes(),
            )?;
            let plugins = root.join("plugins");
            if kind != instance_manager::ServerType::Vanilla {
                match fs::symlink_metadata(&plugins) {
                    Ok(meta) if meta.is_dir() && !is_link(&meta) => {
                        for child in fs::read_dir(&plugins).map_err(error)? {
                            let child = child.map_err(error)?;
                            let name = child
                                .file_name()
                                .into_string()
                                .map_err(|_| "Nombre de plugin incompatible")?;
                            if plain(&name) && !name.starts_with('.') && name.ends_with(".jar") {
                                archive.add(&child.path(), &format!("plugins/{name}"))?;
                            }
                        }
                    }
                    Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                    Ok(_) => return Err("La carpeta plugins no es un directorio normal".into()),
                    Err(e) => return Err(error(e)),
                }
            }
        } else {
            archive.tree(&root, "", 0)?;
        }
        archive
            .zip
            .finish()
            .map_err(error)?
            .sync_all()
            .map_err(error)?;
        fs::rename(&tmp, &path).map_err(error)?;
        Ok(path.display().to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

pub async fn create(name: String, portable: bool) -> Result<String, String> {
    let _guard = super::content::mutation_lock(&name).await;
    worlds::ensure_stopped(&name)?;
    let data = instance_manager::get_instance(&name).await?;
    if data.preparation.is_some() {
        return Err("Termina la preparación antes de respaldar la instancia".into());
    }
    let (folder, suffix, properties) = if portable {
        if data.server_type.is_none() {
            return Err("Esta instancia no es un servidor".into());
        }
        (
            "exports",
            ".tflserver.zip",
            Some(portable_properties(
                &server_ports::read(&data.dir()).await?,
            )?),
        )
    } else {
        ("instance-backups", ".tflbackup.zip", None)
    };
    let path = PathManager::get()
        .get_shared_dir()
        .join(folder)
        .join(format!("{}{suffix}", uuid::Uuid::new_v4()));
    tokio::task::spawn_blocking(move || archive(data, portable, path, properties))
        .await
        .map_err(|_| "No se pudo crear el archivo".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn server_export_retains_auth_policy_and_omits_credentials() {
        let output = portable_properties("online-mode=true\nrcon.password=secret\nenable-rcon=true\nserver-ip=192.0.2.1\nlevel-name=C:/Private\nmotd=token\ndifficulty=hard").unwrap();
        assert_eq!(
            output,
            "enable-rcon=false\nenable-query=false\nonline-mode=true\ndifficulty=hard\n"
        );
    }
}
