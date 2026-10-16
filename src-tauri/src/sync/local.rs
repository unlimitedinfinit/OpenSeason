use std::fs;
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};

use crate::case_profile::{CaseMode, CaseProfile};
use crate::seal;
use super::types::{CaseSyncMetadata, CloudSession, SyncAction, SyncError, SyncRefusal};

pub fn assert_sync_allowed(profile: &CaseProfile) -> Result<(), SyncError> {
    if profile.is_sync_forbidden() {
        Err(SyncError::ConfidentialForbidden)
    } else {
        Ok(())
    }
}

pub fn request_sync(profile: &CaseProfile, action: SyncAction) -> Result<(), SyncError> {
    assert_sync_allowed(profile)?;
    let _ = action;
    let _ = CaseMode::Standard;
    Err(SyncError::NotImplemented)
}

pub fn request_sync_for_dir(case_dir: &Path, action: SyncAction) -> Result<(), SyncError> {
    if seal::is_dir_sealed(case_dir) {
        return Err(SyncError::ConfidentialForbidden);
    }
    let profile = crate::case_profile::load_case(case_dir)
        .map_err(|_| SyncError::NotImplemented)?;
    request_sync(&profile, action)
}

pub fn describe_sync_for_dir(case_dir: &Path, action: SyncAction) -> Result<SyncRefusal, String> {
    let mode = crate::case_profile::load_case(case_dir)
        .map(|p| p.mode.as_str().to_string())
        .unwrap_or_else(|_| {
            if seal::is_dir_sealed(case_dir) {
                "confidential".to_string()
            } else {
                "unknown".to_string()
            }
        });
    match request_sync_for_dir(case_dir, action) {
        Ok(()) => Ok(SyncRefusal {
            action: format!("{:?}", action).to_ascii_lowercase(),
            mode,
            reason: "ok".to_string(),
            kind: "ok".to_string(),
        }),
        Err(SyncError::ConfidentialForbidden) => Ok(SyncRefusal {
            action: format!("{:?}", action).to_ascii_lowercase(),
            mode,
            reason: SyncError::ConfidentialForbidden.to_string(),
            kind: "confidential_forbidden".to_string(),
        }),
        Err(SyncError::NotImplemented) => Ok(SyncRefusal {
            action: format!("{:?}", action).to_ascii_lowercase(),
            mode,
            reason: SyncError::NotImplemented.to_string(),
            kind: "not_implemented".to_string(),
        }),
    }
}

pub fn describe_sync_error(profile: &CaseProfile, action: SyncAction) -> SyncRefusal {
    match request_sync(profile, action) {
        Ok(()) => SyncRefusal {
            action: format!("{:?}", action).to_ascii_lowercase(),
            mode: profile.mode.as_str().to_string(),
            reason: "ok".to_string(),
            kind: "ok".to_string(),
        },
        Err(SyncError::ConfidentialForbidden) => SyncRefusal {
            action: format!("{:?}", action).to_ascii_lowercase(),
            mode: profile.mode.as_str().to_string(),
            reason: SyncError::ConfidentialForbidden.to_string(),
            kind: "confidential_forbidden".to_string(),
        },
        Err(SyncError::NotImplemented) => SyncRefusal {
            action: format!("{:?}", action).to_ascii_lowercase(),
            mode: profile.mode.as_str().to_string(),
            reason: SyncError::NotImplemented.to_string(),
            kind: "not_implemented".to_string(),
        },
    }
}

pub fn route_doctype_to_subfolder(doc_type: &str) -> &'static str {
    let clean = doc_type.trim().to_ascii_lowercase();
    match clean.as_str() {
        "order" => "orders",
        "motion" | "answer" | "notice" | "petition" | "complaint" | "pleading" | "brief"
        | "court filing" | "service of process" => "filings",
        "evidence" | "receipt" | "lease" => "evidence",
        _ => "evidence",
    }
}

pub fn session_file_path() -> PathBuf {
    let base = dirs::data_local_dir()
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("JustLegal").join("cloud_session.json")
}

pub fn load_cloud_session() -> Option<CloudSession> {
    let p = session_file_path();
    if !p.exists() {
        return None;
    }
    let data = fs::read_to_string(&p).ok()?;
    serde_json::from_str::<CloudSession>(&data).ok()
}

pub fn save_cloud_session(session: &CloudSession) -> Result<(), String> {
    let p = session_file_path();
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(session).map_err(|e| e.to_string())?;
    fs::write(p, data).map_err(|e| e.to_string())
}

pub fn clear_cloud_session() -> Result<(), String> {
    let p = session_file_path();
    if p.exists() {
        fs::remove_file(p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn calculate_file_sha256(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn load_case_sync_metadata(case_dir: &Path) -> CaseSyncMetadata {
    let sync_path = case_dir.join(".openseason").join("sync.json");
    if sync_path.exists() {
        if let Ok(data) = fs::read_to_string(&sync_path) {
            if let Ok(meta) = serde_json::from_str::<CaseSyncMetadata>(&data) {
                return meta;
            }
        }
    }
    CaseSyncMetadata::default()
}

pub fn save_case_sync_metadata(case_dir: &Path, meta: &CaseSyncMetadata) -> Result<(), String> {
    let dir = case_dir.join(".openseason");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let data = serde_json::to_string_pretty(meta).map_err(|e| e.to_string())?;
    fs::write(dir.join("sync.json"), data).map_err(|e| e.to_string())
}
