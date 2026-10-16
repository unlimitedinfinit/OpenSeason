use super::local::*;
use super::types::*;
use crate::case_profile::{sample_appeal_profile, CaseMode};

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
    let mut profile = sample_appeal_profile();
    profile.convert_to_confidential().unwrap();
    assert!(profile.is_sync_forbidden());
    assert!(matches!(
        request_sync(&profile, SyncAction::Publish),
        Err(SyncError::ConfidentialForbidden)
    ));
}

#[test]
fn doctype_routing_maps_real_values_correctly() {
    assert_eq!(route_doctype_to_subfolder("Order"), "orders");
    assert_eq!(route_doctype_to_subfolder("Motion"), "filings");
    assert_eq!(route_doctype_to_subfolder("Brief"), "filings");
    assert_eq!(route_doctype_to_subfolder("Court Filing"), "filings");
    assert_eq!(route_doctype_to_subfolder("Evidence"), "evidence");
    assert_eq!(route_doctype_to_subfolder("Receipt"), "evidence");
}
