use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::crypto;
use crate::seal;

/// Hunt and case ids accepted by Tauri commands must be UUID text, not paths.
pub fn parse_record_id(id: &str) -> Result<Uuid, String> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err("A case or hunt id is required.".to_string());
    }
    if trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains("..")
        || Path::new(trimmed).is_absolute()
    {
        return Err("Invalid case or hunt id. Use a UUID, not a file path.".to_string());
    }
    Uuid::parse_str(trimmed)
        .map_err(|_| "Invalid case or hunt id. Expected a UUID.".to_string())
}

pub fn is_reserved_marker_name(name: &str) -> bool {
    name.trim().eq_ignore_ascii_case(seal::SEAL_MARKER_NAME)
}

pub fn default_app_data_root() -> PathBuf {
    dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.openseason.app")
}

fn is_under(root: &Path, path: &Path) -> bool {
    path.starts_with(root) && path != root
}

/// Resolve `{root}/{uuid}` and prove the canonical path stays under `root`.
pub fn resolve_id_under_root(root: &Path, id: &str) -> Result<PathBuf, String> {
    let uuid = parse_record_id(id)?;
    fs::create_dir_all(root).map_err(|e| {
        format!("Could not create {}: {}", root.display(), e)
    })?;
    let root_canon = root
        .canonicalize()
        .map_err(|e| format!("Could not resolve {}: {}", root.display(), e))?;
    let candidate = root_canon.join(uuid.to_string());
    if candidate.exists() {
        let canon = candidate
            .canonicalize()
            .map_err(|e| format!("Could not resolve {}: {}", candidate.display(), e))?;
        if !is_under(&root_canon, &canon) {
            return Err("Resolved path is outside the allowed folder.".to_string());
        }
        return Ok(canon);
    }
    if !candidate.starts_with(&root_canon) {
        return Err("Resolved path is outside the allowed folder.".to_string());
    }
    Ok(candidate)
}

/// Delete `{root}/{uuid}` only after the path is proven to stay under `root`.
pub fn delete_id_under_root(root: &Path, id: &str) -> Result<bool, String> {
    let path = resolve_id_under_root(root, id)?;
    if path.exists() {
        fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn target_file_name(target: &Path) -> String {
    target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

pub fn canonicalize_existing_prefix(path: &Path) -> PathBuf {
    if path.exists() {
        return path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    }
    if let Some(parent) = path.parent() {
        if parent.exists() {
            if let Ok(canon_parent) = parent.canonicalize() {
                if let Some(name) = path.file_name() {
                    return canon_parent.join(name);
                }
            }
        }
    }
    path.to_path_buf()
}

fn resolved_is_under_or_equal(root: &Path, target: &Path) -> bool {
    let resolved = canonicalize_existing_prefix(target);
    if root.exists() {
        if let Ok(root_canon) = root.canonicalize() {
            return resolved == root_canon || resolved.starts_with(&root_canon);
        }
    }
    resolved == root || resolved.starts_with(root)
}

pub fn is_drive_relative_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() >= 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && bytes.get(2) != Some(&b'/')
        && bytes.get(2) != Some(&b'\\')
}

pub fn reject_unsafe_export_form(target: &Path) -> Result<(), String> {
    if target.as_os_str().is_empty() {
        return Err("A target path is required.".to_string());
    }
    let raw = target.to_string_lossy();
    if raw.starts_with("\\\\") || raw.starts_with("//") {
        return Err("UNC export targets are not allowed.".to_string());
    }
    if is_drive_relative_name(&raw) {
        return Err("Drive-relative export targets are not allowed.".to_string());
    }
    if !target.is_absolute() {
        return Err("Export target must be an absolute path.".to_string());
    }
    if let Ok(meta) = fs::symlink_metadata(target) {
        if meta.file_type().is_symlink() {
            return Err("Export cannot write through a symlink.".to_string());
        }
    }
    Ok(())
}

pub fn sanitize_download_stem(name: &str) -> Result<String, String> {
    let mut s = name.replace('\\', "-").replace('/', "-");
    s = s.replace("..", "-");
    s = s.replace('\0', "");
    s = s.trim().trim_matches('.').to_string();
    if s.is_empty() {
        s = "export".to_string();
    }
    if is_reserved_marker_name(&s) || is_reserved_marker_name(&format!(".{}", s)) {
        return Err("Refusing to write a file named .sealed.".to_string());
    }
    if is_drive_relative_name(&s) {
        return Err("Drive-relative download names are not allowed.".to_string());
    }
    Ok(s)
}

/// Refuse writes that can clobber keys, the seal marker, or a sealed vault copy.
pub fn assert_external_write(
    target: &Path,
    forbidden_roots: &[&Path],
    allow_overwrite: bool,
) -> Result<PathBuf, String> {
    reject_unsafe_export_form(target)?;
    let name = target_file_name(target);
    if is_reserved_marker_name(&name) {
        return Err("Refusing to write a file named .sealed.".to_string());
    }

    let resolved = canonicalize_existing_prefix(target);
    let resolved_name = target_file_name(&resolved);
    if is_reserved_marker_name(&resolved_name) {
        return Err("Refusing to write a file named .sealed.".to_string());
    }

    if !allow_overwrite && resolved.exists() {
        return Err(format!(
            "Refusing to overwrite existing file {}",
            resolved.display()
        ));
    }

    for root in forbidden_roots {
        if resolved_is_under_or_equal(root, &resolved) {
            return Err(
                "Refusing to write inside the app data folder or the cases folder (that can overwrite the vault, salt, verifier, or seal marker)."
                    .to_string(),
            );
        }
    }

    Ok(resolved)
}

pub fn create_new_file(path: &Path) -> Result<fs::File, String> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("Could not create {}: {}", path.display(), e))
}

fn sibling_with_suffix(desired: &Path, n: u32) -> Result<PathBuf, String> {
    let parent = desired
        .parent()
        .ok_or_else(|| "Export target has no parent folder.".to_string())?;
    let stem = desired
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("export");
    let ext = desired
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("bin");
    if n == 0 {
        Ok(desired.to_path_buf())
    } else {
        Ok(parent.join(format!("{}-{}.{}", stem, n, ext)))
    }
}

/// Allocate a unique path and create it with O_EXCL so exists-then-create cannot race.
pub fn prepare_export_file(
    desired: &Path,
    cases_root: &Path,
    app_data_root: &Path,
) -> Result<(fs::File, PathBuf), String> {
    reject_unsafe_export_form(desired)?;
    for n in 0..1000 {
        let candidate = sibling_with_suffix(desired, n)?;
        match assert_export_target_allowed(&candidate, cases_root, app_data_root) {
            Err(err) if err.to_lowercase().contains("overwrite") => continue,
            Err(err) => return Err(err),
            Ok(_) => match create_new_file(&candidate) {
                Ok(file) => return Ok((file, candidate)),
                Err(err) if err.to_lowercase().contains("already") || err.contains("exists") => {
                    continue;
                }
                Err(err) => {
                    if let Err(io) = OpenOptions::new().write(true).create_new(true).open(&candidate)
                    {
                        if io.kind() == ErrorKind::AlreadyExists {
                            continue;
                        }
                    }
                    return Err(err);
                }
            },
        }
    }
    Err("Could not allocate a unique export file name.".to_string())
}

pub fn require_password_if_sealed(
    hunt_dir: &Path,
    password: Option<&str>,
    salt: Option<&str>,
    verifier: Option<&[u8]>,
) -> Result<(), String> {
    if !seal::is_dir_sealed(hunt_dir) {
        return Ok(());
    }
    let password = password.ok_or_else(|| {
        "This hunt is sealed. Re-enter the vault password before changing or deleting it.".to_string()
    })?;
    let salt = salt.ok_or_else(|| {
        "Cannot verify the vault password (salt missing). Sealed hunt was not changed.".to_string()
    })?;
    let verifier = verifier.ok_or_else(|| {
        "Cannot verify the vault password (verifier missing). Sealed hunt was not changed.".to_string()
    })?;
    crypto::confirm_password_for_seal(password, salt, verifier)?;
    Ok(())
}

pub fn assert_export_target_allowed(
    target: &Path,
    cases_root: &Path,
    app_data_root: &Path,
) -> Result<PathBuf, String> {
    assert_external_write(target, &[cases_root, app_data_root], false)
}

#[derive(Debug)]
pub struct PurgeReport {
    pub deleted: usize,
    pub skipped_sealed: usize,
}

/// Delete a hunt folder. Sealed vaults require the vault password.
pub fn delete_hunt_checked(
    vaults_root: &Path,
    hunt_id: &str,
    password: Option<&str>,
    salt: Option<&str>,
    verifier: Option<&[u8]>,
) -> Result<bool, String> {
    let path = resolve_id_under_root(vaults_root, hunt_id)?;
    if !path.exists() {
        return Ok(false);
    }
    if seal::is_dir_sealed(&path) {
        let password = password.ok_or_else(|| {
            "This is a sealed Confidential vault, often the only encrypted copy. Re-enter the vault password to delete it.".to_string()
        })?;
        let salt = salt.ok_or_else(|| {
            "Cannot verify the vault password (salt missing). Sealed vault was not deleted.".to_string()
        })?;
        let verifier = verifier.ok_or_else(|| {
            "Cannot verify the vault password (verifier missing). Sealed vault was not deleted.".to_string()
        })?;
        crypto::confirm_password_for_seal(password, salt, verifier)?;
    }
    fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(true)
}

/// Delete unsealed hunts. Sealed vaults are skipped unless include_sealed and the password matches.
pub fn purge_vaults_checked(
    vaults_root: &Path,
    password: Option<&str>,
    salt: Option<&str>,
    verifier: Option<&[u8]>,
    include_sealed: bool,
) -> Result<PurgeReport, String> {
    let mut report = PurgeReport {
        deleted: 0,
        skipped_sealed: 0,
    };
    if !vaults_root.exists() {
        return Ok(report);
    }

    if include_sealed {
        let password = password.ok_or_else(|| {
            "Sealed Confidential vaults are the only encrypted copy. Re-enter the vault password to delete them.".to_string()
        })?;
        let salt = salt.ok_or_else(|| {
            "Cannot verify the vault password (salt missing). Sealed vaults were not deleted.".to_string()
        })?;
        let verifier = verifier.ok_or_else(|| {
            "Cannot verify the vault password (verifier missing). Sealed vaults were not deleted.".to_string()
        })?;
        crypto::confirm_password_for_seal(password, salt, verifier)?;
    }

    let entries: Vec<_> = fs::read_dir(vaults_root)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .collect();

    for entry in entries {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if parse_record_id(&name).is_err() {
            continue;
        }
        if seal::is_dir_sealed(&path) {
            if !include_sealed {
                report.skipped_sealed += 1;
                continue;
            }
            fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
            report.deleted += 1;
        } else {
            fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
            report.deleted += 1;
        }
    }
    Ok(report)
}

pub fn is_hex_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("os-sandbox-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn traversal_id_rejected() {
        let root = temp_dir();
        let outside = root.parent().unwrap().join(format!("os-outside-{}", Uuid::new_v4()));
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("keep.txt"), b"keep").unwrap();

        for id in ["../etc", "..", "../../tmp", "foo/../bar", r"..\windows"] {
            let err = resolve_id_under_root(&root, id).unwrap_err();
            assert!(
                err.to_lowercase().contains("invalid") || err.to_lowercase().contains("uuid"),
                "traversal id {id:?} must be rejected, got {err}"
            );
            let del = delete_id_under_root(&root, id).unwrap_err();
            assert!(
                del.to_lowercase().contains("invalid") || del.to_lowercase().contains("uuid"),
                "delete must refuse traversal id {id:?}, got {del}"
            );
        }
        assert!(
            outside.join("keep.txt").exists(),
            "a traversal id must not delete a sibling folder"
        );
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
    }

    #[test]
    fn absolute_path_id_rejected() {
        let root = temp_dir();
        let abs = std::env::temp_dir().join(format!("os-abs-{}", Uuid::new_v4()));
        fs::create_dir_all(&abs).unwrap();
        fs::write(abs.join("keep.txt"), b"keep").unwrap();

        for id in [
            abs.to_string_lossy().into_owned(),
            "/tmp/evil".to_string(),
            "/00000000-0000-4000-8000-000000000099".to_string(),
            format!("/{}", Uuid::new_v4()),
        ] {
            let err = resolve_id_under_root(&root, &id).unwrap_err();
            assert!(
                err.to_lowercase().contains("invalid")
                    || err.to_lowercase().contains("uuid")
                    || err.to_lowercase().contains("path"),
                "absolute id {id:?} must be rejected, got {err}"
            );
        }
        assert!(abs.join("keep.txt").exists());
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&abs);
    }

    #[test]
    fn forbidden_export_target_rejected() {
        let tmp = temp_dir();
        let cases = tmp.join("JustLegal").join("Cases");
        let app_data = tmp.join("com.openseason.app");
        fs::create_dir_all(&cases).unwrap();
        fs::create_dir_all(&app_data).unwrap();
        let case_id = Uuid::new_v4().to_string();
        let case_dir = cases.join(&case_id);
        fs::create_dir_all(&case_dir).unwrap();
        let sealed = case_dir.join(".sealed");

        let inside = assert_export_target_allowed(&case_dir.join("bundle.osb"), &cases, &app_data)
            .unwrap_err();
        assert!(
            inside.to_lowercase().contains("cases") || inside.to_lowercase().contains("app data"),
            "export inside cases root must be refused: {inside}"
        );

        let named = assert_export_target_allowed(&tmp.join(".sealed"), &cases, &app_data).unwrap_err();
        assert!(
            named.to_lowercase().contains(".sealed"),
            "export named .sealed must be refused: {named}"
        );

        let mixed = assert_export_target_allowed(&tmp.join(".SEALED"), &cases, &app_data).unwrap_err();
        assert!(
            mixed.to_lowercase().contains(".sealed"),
            "export named .SEALED must be refused: {mixed}"
        );

        let overwrite = assert_export_target_allowed(&sealed, &cases, &app_data).unwrap_err();
        assert!(
            overwrite.to_lowercase().contains(".sealed")
                || overwrite.to_lowercase().contains("cases"),
            "export onto a case .sealed must be refused: {overwrite}"
        );

        let ok = tmp.join("safe-export.osb");
        assert!(assert_export_target_allowed(&ok, &cases, &app_data).is_ok());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn export_refuses_salt_and_verifier_overwrite() {
        let tmp = temp_dir();
        let cases = tmp.join("cases");
        let app_data = tmp.join("appdata");
        fs::create_dir_all(&cases).unwrap();
        fs::create_dir_all(&app_data).unwrap();
        fs::write(app_data.join("master_salt.bin"), b"salt").unwrap();
        fs::write(app_data.join("master_verifier.bin"), b"ver").unwrap();
        let vault = app_data.join("vaults").join(Uuid::new_v4().to_string());
        fs::create_dir_all(&vault).unwrap();
        fs::write(vault.join("case.json.enc"), b"enc").unwrap();

        for target in [
            app_data.join("master_salt.bin"),
            app_data.join("master_verifier.bin"),
            vault.join("case.json.enc"),
        ] {
            let err = assert_export_target_allowed(&target, &cases, &app_data).unwrap_err();
            assert!(
                err.to_lowercase().contains("app data")
                    || err.to_lowercase().contains("overwrite")
                    || err.to_lowercase().contains("cases"),
                "must refuse writing {}: {err}",
                target.display()
            );
            assert!(target.exists(), "target must still exist: {}", target.display());
        }
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn export_refuses_existing_file_overwrite() {
        let tmp = temp_dir();
        let cases = tmp.join("cases");
        let app_data = tmp.join("appdata");
        fs::create_dir_all(&cases).unwrap();
        fs::create_dir_all(&app_data).unwrap();
        let existing = tmp.join("already.osb");
        fs::write(&existing, b"keep-me").unwrap();
        let err = assert_export_target_allowed(&existing, &cases, &app_data).unwrap_err();
        assert!(
            err.to_lowercase().contains("overwrite"),
            "existing file must not be overwritten: {err}"
        );
        assert_eq!(fs::read(&existing).unwrap(), b"keep-me");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn uuid_id_stays_under_root() {
        let root = temp_dir();
        let id = Uuid::new_v4().to_string();
        let path = resolve_id_under_root(&root, &id).unwrap();
        assert!(path.starts_with(root.canonicalize().unwrap()));
        assert_eq!(path.file_name().unwrap().to_str().unwrap(), id);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sealed_vault_delete_refused_without_password() {
        let vaults = temp_dir();
        let id = Uuid::new_v4().to_string();
        let hunt = resolve_id_under_root(&vaults, &id).unwrap();
        fs::create_dir_all(&hunt).unwrap();
        seal::write_seal_marker(&hunt, &id, "2024-01-01T00:00:00Z").unwrap();
        fs::write(hunt.join("keep.enc"), b"only-copy").unwrap();

        let err = delete_hunt_checked(&vaults, &id, None, None, None).unwrap_err();
        assert!(
            err.to_lowercase().contains("password") || err.to_lowercase().contains("sealed"),
            "sealed delete without password must be refused: {err}"
        );
        assert!(hunt.join("keep.enc").exists());

        let salt = crypto::generate_salt();
        let key = crypto::derive_key("correct-delete-password", &salt).unwrap();
        let verifier = crypto::create_password_verifier(&key).unwrap();
        let wrong = delete_hunt_checked(
            &vaults,
            &id,
            Some("wrong-password"),
            Some(&salt),
            Some(&verifier),
        )
        .unwrap_err();
        assert!(
            wrong.to_lowercase().contains("wrong password"),
            "wrong password must not delete sealed vault: {wrong}"
        );
        assert!(hunt.join("keep.enc").exists());

        delete_hunt_checked(
            &vaults,
            &id,
            Some("correct-delete-password"),
            Some(&salt),
            Some(&verifier),
        )
        .unwrap();
        assert!(!hunt.exists(), "correct password must delete the sealed vault");
        let _ = fs::remove_dir_all(&vaults);
    }

    #[test]
    fn sealed_vault_purge_refused_without_password() {
        let vaults = temp_dir();
        let sealed_id = Uuid::new_v4().to_string();
        let open_id = Uuid::new_v4().to_string();
        let sealed = resolve_id_under_root(&vaults, &sealed_id).unwrap();
        let open = resolve_id_under_root(&vaults, &open_id).unwrap();
        fs::create_dir_all(&sealed).unwrap();
        fs::create_dir_all(&open).unwrap();
        seal::write_seal_marker(&sealed, &sealed_id, "2024-01-01T00:00:00Z").unwrap();
        fs::write(sealed.join("keep.enc"), b"sealed-only-copy").unwrap();
        fs::write(open.join("draft.txt"), b"unsealed").unwrap();

        let include_err = purge_vaults_checked(&vaults, None, None, None, true).unwrap_err();
        assert!(
            include_err.to_lowercase().contains("password")
                || include_err.to_lowercase().contains("sealed"),
            "purge of sealed vaults without password must be refused: {include_err}"
        );
        assert!(sealed.join("keep.enc").exists());

        let skipped = purge_vaults_checked(&vaults, None, None, None, false).unwrap();
        assert_eq!(skipped.skipped_sealed, 1);
        assert_eq!(skipped.deleted, 1);
        assert!(sealed.join("keep.enc").exists());
        assert!(!open.exists());
        let _ = fs::remove_dir_all(&vaults);
    }

    #[test]
    fn export_refuses_app_data_new_file() {
        let tmp = temp_dir();
        let cases = tmp.join("cases");
        let app_data = tmp.join("appdata");
        fs::create_dir_all(&cases).unwrap();
        fs::create_dir_all(&app_data).unwrap();
        let brand_new = app_data.join("brand-new.osb");
        assert!(!brand_new.exists());
        let err = assert_export_target_allowed(&brand_new, &cases, &app_data).unwrap_err();
        assert!(
            err.to_lowercase().contains("app data") || err.to_lowercase().contains("cases"),
            "new file under app data must be refused: {err}"
        );
        assert!(!brand_new.exists());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn export_refuses_relative_and_unc_targets() {
        let tmp = temp_dir();
        let cases = tmp.join("cases");
        let app_data = tmp.join("appdata");
        fs::create_dir_all(&cases).unwrap();
        fs::create_dir_all(&app_data).unwrap();
        let rel = assert_export_target_allowed(Path::new("relative.osb"), &cases, &app_data)
            .unwrap_err();
        assert!(rel.to_lowercase().contains("absolute") || rel.to_lowercase().contains("relative"));
        let unc = assert_export_target_allowed(Path::new("//server/share/x.osb"), &cases, &app_data)
            .unwrap_err();
        assert!(unc.to_lowercase().contains("unc") || unc.to_lowercase().contains("absolute"));
        assert!(is_drive_relative_name("C:x"));
        assert!(sanitize_download_stem("C:x").is_err());
        assert!(sanitize_download_stem("foo/../bar").is_ok());
        assert!(!sanitize_download_stem("foo/../bar").unwrap().contains(".."));
        let dangling = tmp.join("dangling.osb");
        std::os::unix::fs::symlink(tmp.join("missing-target.osb"), &dangling).unwrap();
        let link_err = assert_export_target_allowed(&dangling, &cases, &app_data).unwrap_err();
        assert!(
            link_err.to_lowercase().contains("symlink"),
            "dangling symlink export target must be refused: {link_err}"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn export_repeat_same_name_gets_numeric_suffix() {
        let tmp = temp_dir();
        let cases = tmp.join("cases");
        let app_data = tmp.join("appdata");
        let downloads = tmp.join("downloads");
        fs::create_dir_all(&cases).unwrap();
        fs::create_dir_all(&app_data).unwrap();
        fs::create_dir_all(&downloads).unwrap();
        let desired = downloads.join("Disclosure_Target.pdf");
        let (first, path1) = prepare_export_file(&desired, &cases, &app_data).unwrap();
        drop(first);
        assert_eq!(path1, desired);
        let (second, path2) = prepare_export_file(&desired, &cases, &app_data).unwrap();
        drop(second);
        assert_eq!(path2, downloads.join("Disclosure_Target-1.pdf"));
        assert!(path2.exists());
        let _ = fs::remove_dir_all(&tmp);
    }
}
