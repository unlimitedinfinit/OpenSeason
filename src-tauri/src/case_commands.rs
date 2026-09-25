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
    let case_dir = crate::sandbox::resolve_id_under_root(&root, &profile.id)?;
    case_profile::ensure_case_layout(&case_dir)?;
    case_profile::save_case(&case_dir, &profile)?;
    case_profile::load_case(&case_dir)
}

#[tauri::command]
pub fn get_case(case_id: String) -> Result<CaseProfile, String> {
    let dir = case_dir_from_id(&case_id)?;
    case_profile::load_case(&dir)
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
