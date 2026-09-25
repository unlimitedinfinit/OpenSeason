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

/// Shared by the Tauri command and tests. Checks the on-disk seal marker.
pub fn save_case_inner(dir: &std::path::Path, profile: CaseProfile) -> Result<CaseProfile, String> {
    case_profile::save_case(dir, &profile)?;
    case_profile::load_case(dir)
}

#[tauri::command]
pub fn save_case(case_id: String, profile: CaseProfile) -> Result<CaseProfile, String> {
    save_case_inner(&case_dir_from_id(&case_id), profile)
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
    if crate::seal::is_dir_sealed(&dir) {
        return Err("This case is already confidential.".to_string());
    }
    profile.convert_to_confidential()?;
    let sealed_at = profile
        .sealed_at
        .clone()
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    crate::seal::write_seal_marker(&dir, &profile.id, &sealed_at)?;
    case_profile::save_case(&dir, &profile)?;

    let vaults = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?
        .join("vaults");
    fs::create_dir_all(&vaults).map_err(|e| e.to_string())?;
    let hunt_dir = vaults.join(&profile.id);
    crate::seal::copy_dir_recursive(&dir, &hunt_dir)?;
    crate::seal::write_seal_marker(&hunt_dir, &profile.id, &sealed_at)?;
    crate::seal::encrypt_vault_payloads(&hunt_dir, &key)?;

    let db_path = hunt_dir.join("metadata.db");
    let db = HuntDatabase::open(&db_path).map_err(|e| e.to_string())?;
    db.conn
        .execute(
            "INSERT INTO info (name, created_at, status, mode) VALUES (?1, ?2, 'Sealed', 'confidential')",
            rusqlite::params![profile.title, profile.created_at],
        )
        .map_err(|e| e.to_string())?;

    crate::seal::write_stripped_stub(&dir, &profile)?;

    Ok(profile)
}

#[tauri::command]
pub fn sync_case_cmd(case_id: String, action: String) -> Result<SyncRefusal, String> {
    let dir = case_dir_from_id(&case_id);
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
