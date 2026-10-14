use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use zip::{ZipArchive, ZipWriter};
use zip::write::SimpleFileOptions;
use walkdir::WalkDir;

pub fn export_hunt(hunt_path: &Path, output_path: &Path) -> Result<(), String> {
    let file = crate::sandbox::create_new_file(output_path)?;
    export_hunt_to_writer(hunt_path, file)
}

pub fn export_hunt_to_writer(hunt_path: &Path, file: File) -> Result<(), String> {
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let walk_dir = WalkDir::new(hunt_path);
    let it = walk_dir.into_iter();

    for entry in it {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = path.strip_prefix(hunt_path).map_err(|e| e.to_string())?;
        // Stick to string representation for zip compatibility
        let name_str = name.to_str().ok_or("Invalid UTF-8 in path")?;

        if path.is_file() {
            zip.start_file(name_str, options).map_err(|e| e.to_string())?;
            let mut f = File::open(path).map_err(|e| e.to_string())?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
            zip.write_all(&buffer).map_err(|e| e.to_string())?;
        } else if !name.as_os_str().is_empty() {
            // Only add directory if it has a name (root is empty)
            zip.add_directory(name_str, options).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn import_hunt(osb_path: &Path, vaults_root: &Path) -> Result<String, String> {
    let file = File::open(osb_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    // Determine hunt name from zip root or filename?
    // Let's use the filename of the OSB as the hunt name if possible, or check for metadata.
    // For MVP, use the OSB filename stem (e.g. "operation_x.osb" -> "operation_x").
    // Folder id must be a UUID so later hunt commands stay inside the vault root.
    let hunt_id = uuid::Uuid::new_v4().to_string();
    let target_dir = crate::sandbox::resolve_id_under_root(vaults_root, &hunt_id)?;

    if target_dir.exists() {
        return Err(format!("Hunt '{}' already exists in vault.", hunt_id));
    }

    // Extract
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        
        // Sanitize path against Zip Slip
        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }

    Ok(hunt_id)
}
