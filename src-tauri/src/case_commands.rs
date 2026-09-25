use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::case_profile::{
    self, CaseMode, CaseProfile, ValidationReport, REVIEW_NOTICE,
};
use crate::crypto::AppState;
use crate::documents::{self, ExportPaths};
use crate::sync::{self, SyncAction, SyncRefusal};

fn cases_root() -> PathBuf {
    case_profile::default_cases_root()
}

fn case_dir_from_id(case_id: &str) -> Result<PathBuf, String> {
    crate::sandbox::resolve_id_under_root(&cases_root(), case_id)
}

#[tauri::command]
pub fn get_cases_root() -> Result<String, String> {
    Ok(cases_root().to_string_lossy().into_owned())
}

#[tauri::command]
pub fn list_cases() -> Result<Vec<CaseProfile>, String> {
    case_profile::list_cases_in(&cases_root())
}

#[derive(Serialize)]
pub struct WorkspaceCase {
    pub profile: CaseProfile,
    pub source: String,
    pub has_vault: bool,
    pub sealed: bool,
    pub folder: String,
}

fn hunt_as_profile(id: &str, name: &str) -> CaseProfile {
    let mut profile = CaseProfile::new_standard(name, "confidential");
    profile.id = id.to_string();
    profile.mode = CaseMode::Confidential;
    profile
}

#[tauri::command]
pub fn list_workspace_cases(app: AppHandle) -> Result<Vec<WorkspaceCase>, String> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let vaults = app
        .path()
        .app_local_data_dir()
        .ok()
        .map(|root| root.join("vaults"));

    for profile in case_profile::list_cases_in(&cases_root())? {
        let dir = case_dir_from_id(&profile.id).unwrap_or_else(|_| cases_root().join(&profile.id));
        let sealed = crate::seal::is_dir_sealed(&dir) || profile.sealed_at.is_some();
        let has_vault = vaults
            .as_ref()
            .map(|v| v.join(&profile.id).is_dir())
            .unwrap_or(false);
        seen.insert(profile.id.clone());
        out.push(WorkspaceCase {
            source: "documents".to_string(),
            has_vault,
            sealed,
            folder: dir.to_string_lossy().into_owned(),
            profile,
        });
    }

    if let Some(vaults) = vaults {
        if vaults.is_dir() {
            for entry in fs::read_dir(&vaults).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let id = entry.file_name().to_string_lossy().into_owned();
                if crate::sandbox::parse_record_id(&id).is_err() || seen.contains(&id) {
                    continue;
                }
                let name = {
                    let db = path.join("metadata.db");
                    rusqlite::Connection::open(&db)
                        .ok()
                        .and_then(|conn| {
                            conn.query_row("SELECT name FROM info LIMIT 1", [], |row| {
                                row.get::<_, String>(0)
                            })
                            .ok()
                        })
                        .unwrap_or_else(|| "Confidential case".to_string())
                };
                let profile = hunt_as_profile(&id, &name);
                let sealed = crate::seal::is_dir_sealed(&path);
                seen.insert(id);
                out.push(WorkspaceCase {
                    source: "vault".to_string(),
                    has_vault: true,
                    sealed,
                    folder: path.to_string_lossy().into_owned(),
                    profile,
                });
            }
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn open_case_folder(path: String) -> Result<CaseProfile, String> {
    let raw = PathBuf::from(path.trim());
    if raw.as_os_str().is_empty() {
        return Err("Choose a case folder that contains case.json.".to_string());
    }
    let case_dir = if raw.join("case.json").is_file() {
        raw
    } else if raw.is_file() && raw.file_name().and_then(|s| s.to_str()) == Some("case.json") {
        raw.parent()
            .ok_or_else(|| "case.json has no parent folder.".to_string())?
            .to_path_buf()
    } else {
        return Err("That folder is not an OpenSeason case (missing case.json).".to_string());
    };
    let profile = case_profile::load_case(&case_dir)?;
    let dest = cases_root().join(&profile.id);
    if dest != case_dir && !dest.join("case.json").is_file() {
        copy_case_tree(&case_dir, &dest)?;
    }
    case_profile::load_case(&case_dir_from_id(&profile.id)?)
}

fn copy_case_tree(from: &std::path::Path, to: &std::path::Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in walkdir::WalkDir::new(from) {
        let entry = entry.map_err(|e| e.to_string())?;
        let rel = entry.path().strip_prefix(from).map_err(|e| e.to_string())?;
        let dest = to.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(entry.path(), &dest).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct CaseArtifact {
    pub kind: String,
    pub name: String,
    pub path: String,
}

#[tauri::command]
pub fn list_case_artifacts(app: AppHandle, case_id: String) -> Result<Vec<CaseArtifact>, String> {
    let dir = case_dir_from_id(&case_id)?;
    let hunt = app
        .path()
        .app_local_data_dir()
        .ok()
        .map(|root| root.join("vaults").join(&case_id));
    list_case_artifacts_at(&dir, hunt.as_deref())
}

pub fn list_case_artifacts_at(
    documents_dir: &std::path::Path,
    hunt_dir: Option<&std::path::Path>,
) -> Result<Vec<CaseArtifact>, String> {
    let mut out = Vec::new();
    if !crate::seal::is_dir_sealed(documents_dir) {
        for (kind, folder) in [
            ("evidence", "evidence"),
            ("draft", "drafts"),
            ("export", "exports"),
            ("filing", "filings"),
            ("order", "orders"),
        ] {
            let root = documents_dir.join(folder);
            if !root.is_dir() {
                continue;
            }
            for entry in walkdir::WalkDir::new(&root).max_depth(4) {
                let entry = entry.map_err(|e| e.to_string())?;
                if !entry.file_type().is_file() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with('.') {
                    continue;
                }
                out.push(CaseArtifact {
                    kind: kind.to_string(),
                    name,
                    path: entry.path().to_string_lossy().into_owned(),
                });
            }
        }
    }
    if let Some(hunt) = hunt_dir {
        out.extend(vault_evidence_artifacts(hunt)?);
    }
    Ok(out)
}

fn vault_evidence_artifacts(hunt_dir: &std::path::Path) -> Result<Vec<CaseArtifact>, String> {
    let db_path = hunt_dir.join("metadata.db");
    if !db_path.is_file() {
        return Ok(Vec::new());
    }
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
    let mut stmt = match conn.prepare(
        "SELECT description, file_path, sha256_hash FROM evidence ORDER BY created_at ASC",
    ) {
        Ok(stmt) => stmt,
        Err(_) => return Ok(Vec::new()),
    };
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0).unwrap_or_default(),
                row.get::<_, String>(1).unwrap_or_default(),
                row.get::<_, Option<String>>(2).unwrap_or(None),
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        let (description, file_path, hash) = row.map_err(|e| e.to_string())?;
        let name = if !description.trim().is_empty() {
            description
        } else if !file_path.trim().is_empty() {
            file_path.clone()
        } else {
            "exhibit".to_string()
        };
        let path = hash
            .filter(|h| crate::sandbox::is_hex_sha256(h))
            .map(|h| hunt_dir.join("evidence").join(format!("{h}.enc")))
            .unwrap_or_else(|| hunt_dir.join("evidence").join(&file_path));
        out.push(CaseArtifact {
            kind: "evidence".to_string(),
            name,
            path: path.to_string_lossy().into_owned(),
        });
    }
    Ok(out)
}

#[tauri::command]
pub fn add_case_evidence(
    case_id: String,
    filename: String,
    file_bytes: Vec<u8>,
    description: String,
) -> Result<String, String> {
    let dir = case_dir_from_id(&case_id)?;
    if crate::seal::is_dir_sealed(&dir) {
        return Err("This case is sealed. Add evidence in the unlocked vault, with the vault password.".to_string());
    }
    let evidence = dir.join("evidence");
    fs::create_dir_all(&evidence).map_err(|e| e.to_string())?;
    let safe = crate::sandbox::sanitize_download_stem(&filename).unwrap_or_else(|_| "exhibit".into());
    let dest = evidence.join(&safe);
    if dest.exists() {
        return Err("An exhibit with that name already exists. Rename the file and try again.".to_string());
    }
    fs::write(&dest, &file_bytes).map_err(|e| e.to_string())?;
    let note = evidence.join(format!("{}.txt", safe));
    if !description.trim().is_empty() {
        let _ = fs::write(note, description);
    }
    Ok(dest.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn export_pleading_cmd(case_id: String, kind: String) -> Result<ExportResult, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = case_profile::load_case(&dir)?;
    let paths: ExportPaths = documents::export_pleading(&dir, &profile, &kind, None)?;
    Ok(ExportResult {
        docx: paths.docx.to_string_lossy().into_owned(),
        pdf: paths.pdf.to_string_lossy().into_owned(),
        notice: REVIEW_NOTICE.to_string(),
    })
}

#[tauri::command]
pub fn create_case(
    app: AppHandle,
    state: State<'_, AppState>,
    title: String,
    document_kind: String,
    mode: Option<String>,
) -> Result<CaseProfile, String> {
    let root = cases_root();
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    if title.trim().is_empty() {
        return Err("A case title is required.".to_string());
    }
    let mut profile = case_profile::CaseProfile::new_standard(&title, &document_kind);
    let requested = mode
        .as_deref()
        .unwrap_or("standard")
        .trim()
        .to_ascii_lowercase();
    if requested == "confidential" {
        if state.get_key().is_none() {
            return Err("Unlock Confidential mode first (Legal Airlock and vault password).".to_string());
        }
        profile.mode = CaseMode::Confidential;
        let app_root = app
            .path()
            .app_local_data_dir()
            .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
        let vaults = app_root.join("vaults");
        fs::create_dir_all(&vaults).map_err(|e| e.to_string())?;
        let hunt_dir = crate::sandbox::resolve_id_under_root(&vaults, &profile.id)?;
        fs::create_dir_all(hunt_dir.join("evidence")).map_err(|e| e.to_string())?;
        let db = crate::db::HuntDatabase::open(hunt_dir.join("metadata.db")).map_err(|e| e.to_string())?;
        db.conn
            .execute(
                "INSERT INTO info (name, created_at, status, mode) VALUES (?1, ?2, 'Draft', 'confidential')",
                rusqlite::params![&profile.title, &profile.created_at],
            )
            .map_err(|e| e.to_string())?;
    }
    let case_dir = crate::sandbox::resolve_id_under_root(&root, &profile.id)?;
    case_profile::ensure_case_layout(&case_dir)?;
    case_profile::save_case(&case_dir, &profile)?;
    case_profile::load_case(&case_dir)
}

#[tauri::command]
pub fn get_case(app: AppHandle, case_id: String) -> Result<CaseProfile, String> {
    if let Ok(dir) = case_dir_from_id(&case_id) {
        if dir.join("case.json").is_file() {
            return case_profile::load_case(&dir);
        }
    }
    let app_root = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let hunt = app_root.join("vaults").join(&case_id);
    if hunt.is_dir() {
        let name = rusqlite::Connection::open(hunt.join("metadata.db"))
            .ok()
            .and_then(|conn| {
                conn.query_row("SELECT name FROM info LIMIT 1", [], |row| {
                    row.get::<_, String>(0)
                })
                .ok()
            })
            .unwrap_or_else(|| "Confidential case".to_string());
        if let Ok(dir) = case_dir_from_id(&case_id) {
            return materialize_vault_case(&dir, &hunt, &case_id, &name);
        }
        return Ok(hunt_as_profile(&case_id, &name));
    }
    Err("Case not found.".to_string())
}

pub fn materialize_vault_case(
    documents_dir: &std::path::Path,
    hunt_dir: &std::path::Path,
    case_id: &str,
    name: &str,
) -> Result<CaseProfile, String> {
    if documents_dir.join("case.json").is_file() {
        return case_profile::load_case(documents_dir);
    }
    let mut profile = hunt_as_profile(case_id, name);
    if crate::seal::is_dir_sealed(hunt_dir) {
        let when = crate::seal::read_marker_sealed_at(hunt_dir)
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
        profile.sealed_at = Some(when);
        fs::create_dir_all(documents_dir).map_err(|e| e.to_string())?;
        case_profile::save_case(documents_dir, &profile)?;
        return case_profile::load_case(documents_dir);
    }
    case_profile::ensure_case_layout(documents_dir)?;
    case_profile::save_case(documents_dir, &profile)?;
    case_profile::load_case(documents_dir)
}

/// Shared by the Tauri command and tests. Checks the on-disk seal marker.
pub fn save_case_inner(dir: &std::path::Path, profile: CaseProfile) -> Result<CaseProfile, String> {
    case_profile::save_case(dir, &profile)?;
    case_profile::load_case(dir)
}

#[tauri::command]
pub fn save_case(case_id: String, profile: CaseProfile) -> Result<CaseProfile, String> {
    save_case_inner(&case_dir_from_id(&case_id)?, profile)
}

#[tauri::command]
pub fn validate_case_cmd(case_id: String) -> Result<ValidationReport, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = case_profile::load_case(&dir)?;
    Ok(case_profile::validate_case(&profile, Some(&dir)))
}

#[derive(Serialize)]
pub struct ExportResult {
    pub docx: String,
    pub pdf: String,
    pub notice: String,
}

#[tauri::command]
pub fn export_notice_of_appeal_cmd(case_id: String) -> Result<ExportResult, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = case_profile::load_case(&dir)?;
    let paths: ExportPaths = documents::export_notice_of_appeal(&dir, &profile, None)?;
    Ok(ExportResult {
        docx: paths.docx.to_string_lossy().into_owned(),
        pdf: paths.pdf.to_string_lossy().into_owned(),
        notice: REVIEW_NOTICE.to_string(),
    })
}

#[tauri::command]
pub fn convert_case_to_confidential(
    app: AppHandle,
    state: State<'_, AppState>,
    case_id: String,
    password: String,
) -> Result<CaseProfile, String> {
    if state.get_key().is_none() {
        return Err("Unlock Confidential mode first. The Legal Airlock and vault password are required before a case can be sealed.".to_string());
    }

    let app_root = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let salt_path = app_root.join("master_salt.bin");
    let verifier_path = app_root.join("master_verifier.bin");
    if !verifier_path.exists() {
        return Err("Unlock Confidential mode first so the vault password can be checked. Plaintext will not be deleted until that check passes.".to_string());
    }
    let salt = fs::read_to_string(&salt_path).map_err(|e| format!("Could not read vault salt: {}", e))?;
    let verifier = fs::read(&verifier_path).map_err(|e| format!("Could not read password verifier: {}", e))?;

    let dir = case_dir_from_id(&case_id)?;
    let vaults = app_root.join("vaults");
    fs::create_dir_all(&vaults).map_err(|e| e.to_string())?;
    let hunt_dir = crate::sandbox::resolve_id_under_root(&vaults, &case_id)?;

    crate::seal::seal_standard_case_with_password(&dir, &hunt_dir, &password, &salt, &verifier)
}

#[tauri::command]
pub fn sync_case_cmd(case_id: String, action: String) -> Result<SyncRefusal, String> {
    let dir = case_dir_from_id(&case_id)?;
    let action = match action.as_str() {
        "backup" => SyncAction::Backup,
        "publish" => SyncAction::Publish,
        "pull_account" | "pull" => SyncAction::PullAccount,
        other => {
            return Err(format!(
                "Unknown sync action '{}'. Use backup, publish, or pull_account.",
                other
            ))
        }
    };
    sync::describe_sync_for_dir(&dir, action)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::HuntDatabase;

    fn temp_dir(prefix: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("{prefix}-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn vault_only_hunt_evidence_appears_on_unified_case() {
        let hunt = temp_dir("os-vault-art");
        fs::create_dir_all(hunt.join("evidence")).unwrap();
        let db = HuntDatabase::open(hunt.join("metadata.db")).unwrap();
        db.conn
            .execute(
                "INSERT INTO evidence (description, file_path, encrypted_key_nonce, sha256_hash) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    "Invoice copy",
                    "invoice.pdf",
                    vec![0u8; 24],
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                ],
            )
            .unwrap();
        let docs = temp_dir("os-docs-empty");
        let items = list_case_artifacts_at(&docs, Some(&hunt)).unwrap();
        assert!(
            items.iter().any(|a| a.kind == "evidence" && a.name == "Invoice copy"),
            "existing hunt exhibits must show on the unified case: {items:?}"
        );
        let _ = fs::remove_dir_all(&hunt);
        let _ = fs::remove_dir_all(&docs);
    }

    #[test]
    fn materialize_unsealed_vault_hunt_writes_case_json() {
        let hunt = temp_dir("os-vault-open");
        HuntDatabase::open(hunt.join("metadata.db")).unwrap();
        let docs = temp_dir("os-docs-open").join("00000000-0000-4000-8000-000000000099");
        let profile = materialize_vault_case(
            &docs,
            &hunt,
            "00000000-0000-4000-8000-000000000099",
            "Relator v. Sample Contractor",
        )
        .unwrap();
        assert_eq!(profile.mode, crate::case_profile::CaseMode::Confidential);
        assert!(profile.sealed_at.is_none());
        assert!(docs.join("case.json").is_file());
        let again = materialize_vault_case(
            &docs,
            &hunt,
            "00000000-0000-4000-8000-000000000099",
            "ignored",
        )
        .unwrap();
        assert_eq!(again.title, "Relator v. Sample Contractor");
        let _ = fs::remove_dir_all(&hunt);
        let _ = fs::remove_dir_all(docs.parent().unwrap());
    }
}
