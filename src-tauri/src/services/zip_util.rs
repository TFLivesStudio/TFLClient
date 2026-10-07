//! Helpers de zip/árbol de archivos compartidos entre los backups de mundo
//! (`world_backup`) y el export `.mrpack` (`content::export`). Son síncronos:
//! se llaman desde `spawn_blocking`.

/// Lista recursiva simple de archivos (no directorios) bajo `dir`.
pub(crate) fn walkdir_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walkdir_files(&path));
        } else {
            out.push(path);
        }
    }
    out
}

/// Comprime `source` entero (rutas relativas a `source`) en `dest`.
pub(crate) fn zip_dir(source: &std::path::Path, dest: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(dest)?;
    let mut zip = zip::ZipWriter::new(file);
    for entry in walkdir_files(source) {
        let rel = entry.strip_prefix(source).unwrap_or(&entry);
        let zip_path = rel.to_string_lossy().replace('\\', "/");
        let bytes = std::fs::read(&entry)?;
        zip.start_file(zip_path, zip::write::SimpleFileOptions::default())?;
        std::io::Write::write_all(&mut zip, &bytes)?;
    }
    zip.finish()?;
    Ok(())
}
