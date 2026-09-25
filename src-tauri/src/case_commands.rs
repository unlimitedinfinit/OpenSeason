use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::case_profile::{
    self, CaseMode, CaseProfile, ValidationReport, REVIEW_NOTICE,
};
use crate::crypto::AppState;
use crate::db::HuntDatabase;
use crate::documents::{self, ExportPaths};
use crate::sync::{self, SyncAction, SyncRefusal};

fn cases_root() -> PathBuf {
    case_profile::default_cases_root()
}

fn case_dir_from_id(case_id: &str) -> PathBuf {
    case_profile::resolve_case_dir(case_id)
}

#[tauri::command]
pub fn get_cases_root() -> Result<String, String> {
    Ok(cases_root().to_string_lossy().into_owned())
}

#[tauri::command]
pub fn list_cases() -> Result<Vec<CaseProfile>, String> {
    let all = case_profile::list_cases_in(&cases_root())?;
    Ok(all
        .into_iter()
        .filter(|c| !c.is_sync_forbidden() || c.mode == CaseMode::Standard)
        .filter(|c| c.mode == CaseMode::Standard)
        .collect())
}

#[tauri::command]
pub fn create_case(title: String, document_kind: String) -> Result<CaseProfile, String> {
    let root = cases_root();
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    // Always create a child folder. The cases root may be empty on first use
    // and must not itself become case.json.
    let profile = case_profile::CaseProfile::new_standard(&title, &document_kind);
    if title.trim().is_empty() {
        return Err("A case title is required.".to_string());
    }
    let case_dir = root.join(&profile.id);
    case_profile::ensure_case_layout(&case_dir)?;
    case_profile::save_case(&case_dir, &profile)?;
    case_profile::load_case(&case_dir)
}

#[tauri::command]
pub fn get_case(case_id: String) -> Result<CaseProfile, String> {
    let dir = case_dir_from_id(&case_id);
    case_profile::load_case(&dir)
}

#[tauri::command]
pub fn save_case(case_id: String, profile: CaseProfile) -> Result<CaseProfile, String> {
    if profile.is_sync_forbidden() && profile.mode != CaseMode::Confidential {
        return Err("A sealed case cannot be saved as a standard case.".to_string());
    }
    let dir = case_dir_from_id(&case_id);
    case_profile::save_case(&dir, &profile)?;
    case_profile::load_case(&dir)
}

#[tauri::command]
pub fn validate_case_cmd(case_id: String) -> Result<ValidationReport, String> {
    let dir = case_dir_from_id(&case_id);
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
    let dir = case_dir_from_id(&case_id);
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
) -> Result<CaseProfile, String> {
    let key = state
        .get_key()
        .ok_or_else(|| "Unlock Confidential mode first. The Legal Airlock and vault password are required before a case can be sealed.".to_string())?;

    let dir = case_dir_from_id(&case_id);
    let mut profile = case_profile::load_case(&dir)?;
    if profile.is_sync_forbidden() && profile.mode == CaseMode::Confidential {
        return Err("This case is already confidential.".to_string());
    }
    profile.convert_to_confidential()?;
    case_profile::save_case(&dir, &profile)?;

    let vaults = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?
        .join("vaults");
    fs::create_dir_all(&vaults).map_err(|e| e.to_string())?;
    let hunt_dir = vaults.join(&profile.id);
    copy_dir_recursive(&dir, &hunt_dir)?;

    let evidence_dir = hunt_dir.join("evidence");
    if evidence_dir.exists() {
        encrypt_plain_evidence(&evidence_dir, &key)?;
    }

    let db_path = hunt_dir.join("metadata.db");
    let db = HuntDatabase::open(&db_path).map_err(|e| e.to_string())?;
    db.conn
        .execute(
            "INSERT INTO info (name, created_at, status, mode) VALUES (?1, ?2, 'Sealed', 'confidential')",
            rusqlite::params![profile.title, profile.created_at],
        )
        .map_err(|e| e.to_string())?;

    // Leave a stub in the Documents tree so the case no longer looks standard.
    write_sealed_stub(&dir, &profile)?;

    Ok(profile)
}

fn write_sealed_stub(dir: &std::path::Path, profile: &CaseProfile) -> Result<(), String> {
    for sub in ["orders", "filings", "evidence", "drafts", "exports"] {
        let path = dir.join(sub);
        if path.exists() {
            let _ = fs::remove_dir_all(&path);
        }
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    let stub = serde_json::json!({
        "schema_version": profile.schema_version,
        "id": profile.id,
        "title": profile.title,
        "mode": "confidential",
        "document_kind": profile.document_kind,
        "created_at": profile.created_at,
        "updated_at": profile.updated_at,
        "sealed_at": profile.sealed_at,
        "court": profile.court,
        "docket_number": profile.docket_number,
        "caption": { "plaintiffs": [], "defendants": [], "appellants": [], "appellees": [] },
        "filer": { "name": "", "role": "", "address_lines": [], "phone": "", "email": "", "signature_name": "" },
        "notice_of_appeal": {
            "judgment_date": "",
            "judgment_description": "",
            "trial_court_name": "",
            "trial_court_docket": "",
            "user_text": ""
        }
    });
    fs::write(
        dir.join("case.json"),
        serde_json::to_string_pretty(&stub).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        dir.join("SEALED.txt"),
        "This case was converted to Confidential mode.\nOpen it from Confidential (Open Season) after unlocking the vault.\nThe working copy now lives in the local encrypted vault, not in this folder.\n",
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn copy_dir_recursive(from: &std::path::Path, to: &std::path::Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(from).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src = entry.path();
        let dest = to.join(entry.file_name());
        if src.is_dir() {
            copy_dir_recursive(&src, &dest)?;
        } else {
            fs::copy(&src, &dest).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn encrypt_plain_evidence(
    evidence_dir: &std::path::Path,
    key: &crate::crypto::SessionKey,
) -> Result<(), String> {
    use sha2::{Digest, Sha256};

    for entry in fs::read_dir(evidence_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) == Some("enc") {
            continue;
        }
        let bytes = fs::read(&path).map_err(|e| e.to_string())?;
        let scrubbed = crate::crypto::strip_metadata(&bytes);
        let mut hasher = Sha256::new();
        hasher.update(&scrubbed);
        let hash_hex = format!("{:x}", hasher.finalize());
        let (encrypted, _nonce) = crate::crypto::encrypt_data(&scrubbed, key)?;
        let enc_path = evidence_dir.join(format!("{}.enc", hash_hex));
        fs::write(&enc_path, encrypted).map_err(|e| e.to_string())?;
        let _ = fs::remove_file(&path);
    }
    Ok(())
}

#[tauri::command]
pub fn sync_case_cmd(case_id: String, action: String) -> Result<SyncRefusal, String> {
    let dir = case_dir_from_id(&case_id);
    let profile = case_profile::load_case(&dir)?;
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
    Ok(sync::describe_sync_error(&profile, action))
}
