use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::case_profile::{CaseMode, CaseProfile};
use crate::seal;

/// Network-facing actions that must never receive a confidential case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncAction {
    Backup,
    Publish,
    PullAccount,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "kind", content = "message")]
pub enum SyncError {
    #[error("Confidential cases cannot be sent through the sync, backup, or publish layer. Use a deliberate local export if you need a copy.")]
    ConfidentialForbidden,
    #[error("Account sync is not implemented in this build.")]
    NotImplemented,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncRefusal {
    pub action: String,
    pub mode: String,
    pub reason: String,
    pub kind: String,
}

pub fn assert_sync_allowed(profile: &CaseProfile) -> Result<(), SyncError> {
    if profile.is_sync_forbidden() {
        Err(SyncError::ConfidentialForbidden)
    } else {
        Ok(())
    }
}

/// Stub account sync. Standard cases get a not-implemented error.
/// Confidential cases, and any case that was ever sealed, are refused first.
pub fn request_sync(profile: &CaseProfile, action: SyncAction) -> Result<(), SyncError> {
    assert_sync_allowed(profile)?;
    let _ = action;
    let _ = CaseMode::Standard;
    Err(SyncError::NotImplemented)
}

/// Checks the on-disk `.sealed` marker first, then case.json. Editing case.json cannot unseal.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_profile::sample_appeal_profile;

    #[test]
    fn confidential_case_cannot_backup_publish_or_pull() {
        let mut profile = sample_appeal_profile();
        profile.convert_to_confidential().unwrap();

        for action in [SyncAction::Backup, SyncAction::Publish, SyncAction::PullAccount] {
            let err = request_sync(&profile, action).unwrap_err();
            assert_eq!(err, SyncError::ConfidentialForbidden);
            let described = describe_sync_error(&profile, action);
            assert_eq!(described.kind, "confidential_forbidden");
        }
    }

    #[test]
    fn sealed_case_stays_blocked_even_if_mode_flag_is_flipped() {
        let mut profile = sample_appeal_profile();
        profile.convert_to_confidential().unwrap();
        profile.mode = CaseMode::Standard;
        let err = request_sync(&profile, SyncAction::Publish).unwrap_err();
        assert_eq!(err, SyncError::ConfidentialForbidden);
    }

    #[test]
    fn standard_case_hits_stub_not_confidential_error() {
        let profile = sample_appeal_profile();
        assert!(!profile.is_sync_forbidden());
        let err = request_sync(&profile, SyncAction::Backup).unwrap_err();
        assert_eq!(err, SyncError::NotImplemented);
        let described = describe_sync_error(&profile, SyncAction::Backup);
        assert_eq!(described.kind, "not_implemented");
    }

    #[test]
    fn local_export_is_not_a_sync_action() {
        // This test documents the contract: Word/PDF export and .osb export
        // are local disk operations and do not call request_sync.
        let mut profile = sample_appeal_profile();
        profile.convert_to_confidential().unwrap();
        assert!(profile.is_sync_forbidden());
        assert!(matches!(
            request_sync(&profile, SyncAction::Publish),
            Err(SyncError::ConfidentialForbidden)
        ));
    }
}
