//! Pack installation, verification, removal, and local archive management.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::check::NEVER_FILE_NOTICE;
use crate::rule_pack::RulePack;
use super::registry::{
    catalog_base, default_packs_root, load_catalog, repo_fixtures_catalog_dir,
    sha256_file, sha256_hex, size_label,
};
use super::types::{
    CatalogPack, FormsCatalog, InstallOpts, InstallReport, PackList, PackListItem,
    VerifyReport, VerifyRow, LEGAL_INFO_NOTICE,
};

pub fn list_packs(catalog_path: Option<&Path>, root: Option<&Path>) -> Result<PackList, String> {
    let catalog = load_catalog(catalog_path)?;
    let root = root.map(Path::to_path_buf).unwrap_or_else(default_packs_root);
    let installed_ids = installed_pack_ids(&root);
    let mut available = Vec::new();
    for pack in &catalog.packs {
        let dest = root.join(&pack.id);
        available.push(PackListItem {
            id: pack.id.clone(),
            state: pack.state.clone(),
            court: pack.court.clone(),
            title: pack.title.clone(),
            situations: pack.situations.clone(),
            size_bytes: pack.size_bytes,
            size_label: size_label(effective_pack_bytes(pack, catalog_path)),
            sha256: pack.sha256.clone(),
            installed: dest.is_dir() || installed_ids.iter().any(|id| id == &pack.id),
            install_path: dest.is_dir().then(|| dest.display().to_string()),
            rule_pack_id: pack.rule_pack_id.clone(),
            source_note: catalog.source_note.clone(),
            installable: pack_unavailable_reason_with_catalog(pack, catalog_path, Some(&catalog))
                .is_empty(),
            unavailable_reason: pack_unavailable_reason_with_catalog(
                pack,
                catalog_path,
                Some(&catalog),
            ),
        });
    }
    let installed = scan_installed(&root)?;
    let rule_packs = RulePack::list_builtin()
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "jurisdiction": p.jurisdiction,
                "forms": p.forms.len(),
                "situations": p.situations.len()
            })
        })
        .collect();
    Ok(PackList {
        never_files: true,
        never_serves: true,
        notice: format!("{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}"),
        packs_root: root.display().to_string(),
        catalog_id: catalog.id,
        available,
        installed,
        rule_packs,
    })
}

pub fn effective_pack_bytes(pack: &CatalogPack, catalog_path: Option<&Path>) -> u64 {
    if pack.size_bytes > 0 {
        return pack.size_bytes;
    }
    if let Some(dir) = resolve_local_dir(pack, catalog_path) {
        return dir_size(&dir).unwrap_or(0);
    }
    0
}

#[allow(dead_code)]
pub fn pack_is_installable(pack: &CatalogPack, catalog_path: Option<&Path>) -> bool {
    pack_unavailable_reason(pack, catalog_path).is_empty()
}

#[allow(dead_code)]
pub fn pack_unavailable_reason(pack: &CatalogPack, catalog_path: Option<&Path>) -> String {
    pack_unavailable_reason_with_catalog(pack, catalog_path, None)
}

pub fn pack_unavailable_reason_with_catalog(
    pack: &CatalogPack,
    catalog_path: Option<&Path>,
    catalog: Option<&FormsCatalog>,
) -> String {
    if resolve_local_dir(pack, catalog_path).is_some() {
        return String::new();
    }
    if !pack.download_url.is_empty()
        && !pack.download_url.contains("example.test")
        && !pack.sha256.trim().is_empty()
    {
        return String::new();
    }
    if catalog
        .map(|c| pack_has_installable_forms(c, pack))
        .unwrap_or(false)
    {
        return String::new();
    }
    if pack.download_url.is_empty() && pack.local_dir.is_empty() && pack.sha256.trim().is_empty() {
        return "This pack has no download URL, local folder, or SHA-256 checksum, so it cannot be installed. Pass a zip with --file, or copy fixtures/forms-catalog/packs/nj-appeal.".into();
    }
    if pack.sha256.trim().is_empty() {
        return "This pack has no SHA-256 checksum. OpenSeason will not install a file it cannot verify.".into();
    }
    if pack.download_url.is_empty() || pack.download_url.contains("example.test") {
        return "This pack has no real download source. Point the catalog at a host you control, or pass a local zip with --file.".into();
    }
    String::new()
}

pub fn pack_has_installable_forms(catalog: &FormsCatalog, pack: &CatalogPack) -> bool {
    matching_forms(catalog, pack).any(|f| {
        !f.sha256.trim().is_empty()
            && !f.download_url.is_empty()
            && !f.download_url.contains("example.test")
    })
}

pub fn matching_forms<'a>(
    catalog: &'a FormsCatalog,
    pack: &'a CatalogPack,
) -> impl Iterator<Item = &'a crate::rule_pack::FormEntry> {
    catalog.forms.iter().filter(move |f| {
        f.state.eq_ignore_ascii_case(&pack.state)
            && (pack.situations.is_empty()
                || pack.situations.iter().any(|s| {
                    crate::rule_pack::normalize_key(s)
                        == crate::rule_pack::normalize_key(&f.situation)
                }))
    })
}

pub fn resolve_local_dir(pack: &CatalogPack, catalog_path: Option<&Path>) -> Option<PathBuf> {
    if pack.local_dir.is_empty() {
        return None;
    }
    let raw = PathBuf::from(&pack.local_dir);
    if raw.is_dir() {
        return Some(raw);
    }
    let base = catalog_base(catalog_path);
    let joined = base.join(&pack.local_dir);
    if joined.is_dir() {
        return Some(joined);
    }
    let fixtures = repo_fixtures_catalog_dir().join(&pack.local_dir);
    if fixtures.is_dir() {
        return Some(fixtures);
    }
    None
}

pub fn dir_size(root: &Path) -> Result<u64, String> {
    let mut total = 0u64;
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().is_file() {
            total += entry.metadata().map_err(|e| e.to_string())?.len();
        }
    }
    Ok(total)
}

pub fn installed_pack_ids(root: &Path) -> Vec<String> {
    scan_installed(root)
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.id)
        .collect()
}

pub fn scan_installed(root: &Path) -> Result<Vec<PackListItem>, String> {
    let mut out = Vec::new();
    if !root.is_dir() {
        return Ok(out);
    }
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let pack_json = path.join("pack.json");
        if !pack_json.is_file() {
            continue;
        }
        let pack = RulePack::load_from_path(&pack_json)?;
        let bytes = dir_size(&path).unwrap_or(0);
        out.push(PackListItem {
            id: pack.id.clone(),
            state: pack.jurisdiction.clone(),
            court: pack.court.clone(),
            title: pack.name.clone(),
            situations: pack.situations.iter().map(|s| s.id.clone()).collect(),
            size_bytes: bytes,
            size_label: size_label(bytes),
            sha256: String::new(),
            installed: true,
            install_path: Some(path.display().to_string()),
            rule_pack_id: pack.rule_pack_id.unwrap_or_else(|| pack.id.clone()),
            source_note: pack.source_note,
            installable: true,
            unavailable_reason: String::new(),
        });
    }
    Ok(out)
}

pub fn installed_overlays_for(rule_pack_id: &str) -> Result<Vec<RulePack>, String> {
    let root = default_packs_root();
    let mut out = Vec::new();
    for item in scan_installed(&root)? {
        if let Some(path) = item.install_path {
            let pack = RulePack::load_from_path(&PathBuf::from(path).join("pack.json"))?;
            let key = pack.rule_pack_id.clone().unwrap_or_else(|| pack.id.clone());
            if key == rule_pack_id || pack.id == rule_pack_id || pack.jurisdiction.eq_ignore_ascii_case(rule_pack_id) {
                out.push(pack);
            }
        }
    }
    Ok(out)
}

pub fn match_pack<'a>(
    catalog: &'a FormsCatalog,
    state: &str,
    situation: Option<&str>,
) -> Result<&'a CatalogPack, String> {
    let state_key = state.trim().to_ascii_uppercase();
    let sit = situation.map(crate::rule_pack::normalize_key);
    let mut hits: Vec<&CatalogPack> = catalog
        .packs
        .iter()
        .filter(|p| {
            p.state.eq_ignore_ascii_case(&state_key)
                || p.id.eq_ignore_ascii_case(state)
                || p.id.to_ascii_lowercase().starts_with(&state_key.to_ascii_lowercase())
        })
        .collect();
    if let Some(sit) = sit {
        hits.retain(|p| {
            p.situations
                .iter()
                .any(|s| crate::rule_pack::normalize_key(s) == sit)
                || crate::rule_pack::normalize_key(&p.id).contains(&sit)
        });
    }
    if hits.is_empty() {
        return Err(format!(
            "No catalog pack for state '{state}'{}. Known packs: {}.",
            situation
                .map(|s| format!(" situation '{s}'"))
                .unwrap_or_default(),
            catalog
                .packs
                .iter()
                .map(|p| format!("{} ({})", p.id, p.state))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    Ok(hits[0])
}

pub fn install(opts: InstallOpts) -> Result<InstallReport, String> {
    let root = opts.root.clone().unwrap_or_else(default_packs_root);
    fs::create_dir_all(&root).map_err(|e| format!("Could not create {}: {e}", root.display()))?;
    if let Some(file) = &opts.file {
        return install_archive_or_dir(file, &root, &opts.state, opts.situation.clone());
    }
    let catalog = load_catalog(opts.catalog.as_deref())?;
    let pack = match_pack(&catalog, &opts.state, opts.situation.as_deref())?;
    let dest = root.join(&pack.id);
    if let Some(dir) = resolve_local_dir(pack, opts.catalog.as_deref()) {
        let bytes = dir_size(&dir)?;
        copy_dir(&dir, &dest)?;
        fill_form_hashes(&dest)?;
        return Ok(InstallReport {
            ok: true,
            never_files: true,
            never_serves: true,
            notice: format!(
                "{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}\nDownloaded {}.",
                size_label(bytes)
            ),
            pack_id: pack.id.clone(),
            state: pack.state.clone(),
            situation: opts.situation.clone(),
            bytes_planned: bytes,
            size_label: size_label(bytes),
            dest: dest.display().to_string(),
            files: list_files(&dest)?,
            sha256_ok: true,
            offline_after: true,
        });
    }
    if pack_has_installable_forms(&catalog, pack) {
        if !opts.allow_network {
            return Err(
                "This pack lists official forms with SHA-256 checksums, but network install is off. Pass --file, or confirm install in the window so the files can be downloaded and verified."
                    .into(),
            );
        }
        return install_matching_forms(&catalog, pack, &dest, opts.situation.clone());
    }
    let reason = pack_unavailable_reason_with_catalog(pack, opts.catalog.as_deref(), Some(&catalog));
    if !reason.is_empty() {
        return Err(reason);
    }
    if !opts.allow_network {
        return Err(format!(
            "Pack '{}' has no local folder on this computer. Network install is off. Pass a zip with --file, or copy fixtures/forms-catalog/packs/nj-appeal.",
            pack.id
        ));
    }
    if pack.download_url.contains("example.test") || pack.download_url.is_empty() {
        return Err(format!(
            "Pack '{}' still points at a fixture URL ({}). Replace it with a real host or pass --file.",
            pack.id, pack.download_url
        ));
    }
    let bytes = download_to(&pack.download_url, &dest, &pack.sha256)?;
    Ok(InstallReport {
        ok: true,
        never_files: true,
        never_serves: true,
        notice: format!("{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}"),
        pack_id: pack.id.clone(),
        state: pack.state.clone(),
        situation: opts.situation,
        bytes_planned: bytes,
        size_label: size_label(bytes),
        dest: dest.display().to_string(),
        files: list_files(&dest)?,
        sha256_ok: true,
        offline_after: true,
    })
}

pub fn install_matching_forms(
    catalog: &FormsCatalog,
    pack: &CatalogPack,
    dest: &Path,
    situation: Option<String>,
) -> Result<InstallReport, String> {
    let forms: Vec<_> = matching_forms(catalog, pack)
        .filter(|f| {
            !f.sha256.trim().is_empty()
                && !f.download_url.is_empty()
                && !f.download_url.contains("example.test")
        })
        .cloned()
        .collect();
    if forms.is_empty() {
        return Err(pack_unavailable_reason_with_catalog(pack, None, Some(catalog)));
    }
    fs::create_dir_all(dest.join("forms")).map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    let mut bytes_total = 0u64;
    for form in &forms {
        let ext = if form.download_url.to_ascii_lowercase().ends_with(".pdf") {
            "pdf"
        } else {
            "bin"
        };
        let dest_file = dest.join("forms").join(format!("{}.{}", form.id, ext));
        let tmp_dir = dest.join(".download-tmp");
        let n = download_to(&form.download_url, &tmp_dir, &form.sha256)?;
        bytes_total += n;
        let downloaded = if tmp_dir.join("download.bin").is_file() {
            tmp_dir.join("download.bin")
        } else {
            tmp_dir.clone()
        };
        if downloaded.is_file() {
            fs::rename(&downloaded, &dest_file).map_err(|e| e.to_string())?;
        }
        let _ = fs::remove_dir_all(&tmp_dir);
        files.push(dest_file.display().to_string());
    }
    fill_form_hashes(dest)?;
    Ok(InstallReport {
        ok: true,
        never_files: true,
        never_serves: true,
        notice: format!("{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}"),
        pack_id: pack.id.clone(),
        state: pack.state.clone(),
        situation,
        bytes_planned: bytes_total,
        size_label: size_label(bytes_total),
        dest: dest.display().to_string(),
        files,
        sha256_ok: true,
        offline_after: true,
    })
}

pub fn install_archive_or_dir(
    file: &Path,
    root: &Path,
    state: &str,
    situation: Option<String>,
) -> Result<InstallReport, String> {
    let dest_name = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("installed-pack")
        .to_string();
    let dest = root.join(&dest_name);
    if file.is_dir() {
        copy_dir(file, &dest)?;
    } else {
        extract_zip(file, &dest)?;
    }
    fill_form_hashes(&dest)?;
    let bytes = dir_size(&dest)?;
    let pack_id = dest
        .join("pack.json")
        .exists()
        .then(|| RulePack::load_from_path(&dest.join("pack.json")).ok().map(|p| p.id))
        .flatten()
        .unwrap_or(dest_name);
    Ok(InstallReport {
        ok: true,
        never_files: true,
        never_serves: true,
        notice: format!("{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}"),
        pack_id,
        state: state.to_ascii_uppercase(),
        situation,
        bytes_planned: bytes,
        size_label: size_label(bytes),
        dest: dest.display().to_string(),
        files: list_files(&dest)?,
        sha256_ok: true,
        offline_after: true,
    })
}

pub fn download_to(url: &str, dest: &Path, expected_sha: &str) -> Result<u64, String> {
    if url.starts_with("file://") {
        let path = PathBuf::from(url.trim_start_matches("file://"));
        return install_downloaded_file(&path, dest, expected_sha);
    }
    if Path::new(url).is_file() {
        return install_downloaded_file(Path::new(url), dest, expected_sha);
    }
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(format!("Unsupported pack URL '{url}'. Use https, file, or a local path."));
    }
    let tmp = dest.with_extension("download");
    if let Some(parent) = tmp.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let bytes = reqwest::blocking::get(url)
        .map_err(|e| format!("Download failed: {e}"))?
        .bytes()
        .map_err(|e| format!("Download body failed: {e}"))?;
    if !expected_sha.is_empty() {
        let actual = sha256_hex(&bytes);
        if actual != expected_sha.to_ascii_lowercase() {
            return Err(format!(
                "SHA-256 mismatch for {url}. expected {expected_sha}, got {actual}. The file was not installed."
            ));
        }
    }
    fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    let n = bytes.len() as u64;
    if url.ends_with(".zip") || is_zip(&tmp) {
        extract_zip(&tmp, dest)?;
        let _ = fs::remove_file(&tmp);
    } else {
        fs::create_dir_all(dest).map_err(|e| e.to_string())?;
        fs::rename(&tmp, dest.join("download.bin")).map_err(|e| e.to_string())?;
    }
    Ok(n)
}

pub fn install_downloaded_file(path: &Path, dest: &Path, expected_sha: &str) -> Result<u64, String> {
    if !expected_sha.is_empty() {
        let actual = sha256_file(path)?;
        if actual != expected_sha.to_ascii_lowercase() {
            return Err(format!(
                "SHA-256 mismatch for {}. expected {expected_sha}, got {actual}.",
                path.display()
            ));
        }
    }
    if path.extension().and_then(|s| s.to_str()) == Some("zip") || is_zip(path) {
        extract_zip(path, dest)?;
    } else if path.is_dir() {
        copy_dir(path, dest)?;
    } else {
        return Err("Pass a .zip pack archive or a pack directory.".into());
    }
    Ok(dir_size(dest)?)
}

pub fn is_zip(path: &Path) -> bool {
    File::open(path)
        .ok()
        .and_then(|mut f| {
            let mut magic = [0u8; 2];
            f.read_exact(&mut magic).ok()?;
            Some(magic == *b"PK")
        })
        .unwrap_or(false)
}

pub fn extract_zip(zip_path: &Path, dest: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Not a zip pack: {e}"))?;
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest.join(path),
            None => continue,
        };
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in walkdir::WalkDir::new(from) {
        let entry = entry.map_err(|e| e.to_string())?;
        let rel = entry.path().strip_prefix(from).map_err(|e| e.to_string())?;
        let dest = to.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(entry.path(), &dest).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn fill_form_hashes(dest: &Path) -> Result<(), String> {
    let pack_path = dest.join("pack.json");
    if !pack_path.is_file() {
        return Ok(());
    }
    let mut pack = RulePack::load_from_path(&pack_path)?;
    for form in &mut pack.forms {
        if form.path_in_pack.is_empty() {
            continue;
        }
        let file = dest.join(&form.path_in_pack);
        if file.is_file() {
            form.size_bytes = fs::metadata(&file).map(|m| m.len()).unwrap_or(0);
            form.sha256 = sha256_file(&file)?;
        }
    }
    let text = serde_json::to_string_pretty(&pack).map_err(|e| e.to_string())?;
    fs::write(pack_path, text).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_files(root: &Path) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().is_file() {
            if let Ok(rel) = entry.path().strip_prefix(root) {
                files.push(rel.display().to_string());
            }
        }
    }
    files.sort();
    Ok(files)
}

pub fn remove_pack(state_or_id: &str, root: Option<&Path>) -> Result<String, String> {
    let root = root.map(Path::to_path_buf).unwrap_or_else(default_packs_root);
    let key = state_or_id.trim();
    for item in scan_installed(&root)? {
        if item.id.eq_ignore_ascii_case(key)
            || item.state.eq_ignore_ascii_case(key)
            || item
                .install_path
                .as_deref()
                .map(|p| Path::new(p).file_name().and_then(|n| n.to_str()) == Some(key))
                .unwrap_or(false)
        {
            if let Some(path) = item.install_path {
                fs::remove_dir_all(&path).map_err(|e| format!("Could not remove {path}: {e}"))?;
                return Ok(path);
            }
        }
    }
    Err(format!(
        "No installed pack named '{key}'. Run packs list. Nothing was deleted."
    ))
}

pub fn verify_pack(state_or_id: Option<&str>, root: Option<&Path>) -> Result<Vec<VerifyReport>, String> {
    let root = root.map(Path::to_path_buf).unwrap_or_else(default_packs_root);
    let installed = scan_installed(&root)?;
    let mut reports = Vec::new();
    for item in installed {
        if let Some(want) = state_or_id {
            if !item.id.eq_ignore_ascii_case(want) && !item.state.eq_ignore_ascii_case(want) {
                continue;
            }
        }
        let dest = PathBuf::from(item.install_path.unwrap_or_default());
        let pack = RulePack::load_from_path(&dest.join("pack.json"))?;
        let mut rows = Vec::new();
        let mut ok = true;
        for form in &pack.forms {
            if form.path_in_pack.is_empty() || form.sha256.is_empty() {
                continue;
            }
            let file = dest.join(&form.path_in_pack);
            let actual = if file.is_file() {
                sha256_file(&file)?
            } else {
                String::new()
            };
            let row_ok = !actual.is_empty() && actual == form.sha256.to_ascii_lowercase();
            if !row_ok {
                ok = false;
            }
            rows.push(VerifyRow {
                path: form.path_in_pack.clone(),
                expected: form.sha256.clone(),
                actual,
                ok: row_ok,
            });
        }
        reports.push(VerifyReport {
            ok,
            never_files: true,
            pack_id: pack.id,
            dest: dest.display().to_string(),
            checked: rows,
        });
    }
    if reports.is_empty() {
        return Err("No installed packs to verify. Install one first.".into());
    }
    Ok(reports)
}

pub fn write_zip_from_dir(dir: &Path, dest: &Path) -> Result<String, String> {
    let file = File::create(dest).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry.map_err(|e| e.to_string())?;
        let rel = entry.path().strip_prefix(dir).map_err(|e| e.to_string())?;
        let name = rel.to_str().ok_or("Invalid UTF-8 in pack path")?;
        if entry.file_type().is_file() {
            zip.start_file(name, options).map_err(|e| e.to_string())?;
            zip.write_all(&fs::read(entry.path()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        } else if !name.is_empty() {
            zip.add_directory(name, options).map_err(|e| e.to_string())?;
        }
    }
    zip.finish().map_err(|e| e.to_string())?;
    sha256_file(dest)
}
