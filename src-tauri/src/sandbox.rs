use std::fs;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::seal::SEAL_MARKER_NAME;

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

fn canonicalize_existing_prefix(path: &Path) -> PathBuf {
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

/// Refuse export targets inside the cases root, or any file named `.sealed`.
pub fn assert_export_target_allowed(
    target: &Path,
    cases_root: &Path,
) -> Result<PathBuf, String> {
    if target.as_os_str().is_empty() {
        return Err("Export target path is required.".to_string());
    }
    let name = target_file_name(target);
    if name == SEAL_MARKER_NAME || name == ".sealed" {
        return Err("Export cannot write a file named .sealed.".to_string());
    }

    let resolved = canonicalize_existing_prefix(target);
    let resolved_name = target_file_name(&resolved);
    if resolved_name == SEAL_MARKER_NAME || resolved_name == ".sealed" {
        return Err("Export cannot write a file named .sealed.".to_string());
    }

    if cases_root.exists() || cases_root.parent().map(|p| p.exists()).unwrap_or(false) {
        let cases_canon = if cases_root.exists() {
            cases_root.canonicalize().ok()
        } else {
            None
        };
        if let Some(cases_canon) = cases_canon {
            if resolved == cases_canon || resolved.starts_with(&cases_canon) {
                return Err(
                    "Export cannot write inside the cases folder (that path can overwrite the seal marker)."
                        .to_string(),
                );
            }
        }
    }

    Ok(resolved)
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
        fs::create_dir_all(&cases).unwrap();
        let case_id = Uuid::new_v4().to_string();
        let case_dir = cases.join(&case_id);
        fs::create_dir_all(&case_dir).unwrap();
        let sealed = case_dir.join(".sealed");

        let inside = assert_export_target_allowed(&case_dir.join("bundle.osb"), &cases).unwrap_err();
        assert!(
            inside.to_lowercase().contains("cases"),
            "export inside cases root must be refused: {inside}"
        );

        let named = assert_export_target_allowed(&tmp.join(".sealed"), &cases).unwrap_err();
        assert!(
            named.to_lowercase().contains(".sealed"),
            "export named .sealed must be refused: {named}"
        );

        let overwrite = assert_export_target_allowed(&sealed, &cases).unwrap_err();
        assert!(
            overwrite.to_lowercase().contains(".sealed")
                || overwrite.to_lowercase().contains("cases"),
            "export onto a case .sealed must be refused: {overwrite}"
        );

        let ok = tmp.join("safe-export.osb");
        assert!(assert_export_target_allowed(&ok, &cases).is_ok());
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
}
