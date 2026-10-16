//! Tests for case profile creation, layout, validation, and sealing.

use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use super::profile::*;
use super::storage::*;
use super::types::*;
use super::validation::*;

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("os-case-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn creates_expected_layout() {
    let dir = temp_dir();
    ensure_case_layout(&dir).unwrap();
    for sub in CASE_SUBDIRS {
        assert!(dir.join(sub).is_dir(), "missing {}", sub);
    }
    assert!(dir.join("court-rules").join("generic.json").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn sample_case_validates() {
    let profile = sample_appeal_profile();
    let report = validate_case(&profile, None);
    assert!(report.ok, "{:?}", report.issues);
}

#[test]
fn missing_fields_and_placeholders_fail() {
    let mut profile = sample_appeal_profile();
    profile.docket_number.clear();
    profile.notice_of_appeal.user_text = "See [PLAINTIFF] and {{docket}} TODO".into();
    let report = validate_case(&profile, None);
    assert!(!report.ok);
    let fields: Vec<_> = report.issues.iter().map(|i| i.field.as_str()).collect();
    assert!(fields.contains(&"docket_number"));
    assert!(fields.contains(&"placeholders"));
}

#[test]
fn fixture_sample_case_validates() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/sample-appeal-case");
    let profile = load_case(&fixture).expect("sample fixture should load");
    let report = validate_case(&profile, Some(&fixture));
    assert!(report.ok, "{:?}", report.issues);
    assert_eq!(profile.filer.name, "Jordan Example");
}

#[test]
fn complaint_and_motion_validation_uses_their_own_text() {
    let mut profile = sample_appeal_profile();
    profile.complaint.user_text.clear();
    let report = validate_pleading(&profile, None, "complaint");
    assert!(!report.ok);
    assert!(report.issues.iter().any(|i| i.field == "complaint.user_text"));
    profile.complaint.user_text = "User drafted claim text without placeholders.".into();
    assert!(validate_pleading(&profile, None, "complaint").ok);
    profile.motion.user_text.clear();
    let motion = validate_pleading(&profile, None, "motion");
    assert!(!motion.ok);
}

#[test]
fn convert_is_one_way() {
    let standard = sample_appeal_profile();
    assert_eq!(
        standard.reopen_as_standard().unwrap_err(),
        "This case is not sealed."
    );

    let mut profile = sample_appeal_profile();
    profile.convert_to_confidential().unwrap();
    assert_eq!(profile.mode, CaseMode::Confidential);
    assert!(profile.sealed_at.is_some());
    assert!(profile.is_sync_forbidden());
    let err = profile.reopen_as_standard().unwrap_err();
    assert!(
        err.contains("cannot be made syncable"),
        "sealed reopen must use the confidential error, got: {err}"
    );
    profile.mode = CaseMode::Standard;
    assert!(profile.is_sync_forbidden());
}

#[test]
fn test_roster_counsel_substitution_preserves_history() {
    let mut profile = sample_appeal_profile();
    
    // Add initial opposing counsel
    let former = RosterEntry {
        id: "counsel-001".into(),
        name: "Jane Smith, Esq.".into(),
        role: "opposing_counsel".into(),
        bar_number: Some("NJ-12345".into()),
        firm_name: Some("Smith & Partners".into()),
        is_active: true,
        ..Default::default()
    };
    profile.upsert_roster_entry(former);
    assert_eq!(profile.roster.len(), 1);
    assert!(profile.roster[0].is_active);

    // Substitute with new counsel
    let new_counsel = RosterEntry {
        id: "counsel-002".into(),
        name: "Robert Davis, Esq.".into(),
        role: "opposing_counsel".into(),
        bar_number: Some("NJ-67890".into()),
        firm_name: Some("Davis Legal LLC".into()),
        is_active: true,
        ..Default::default()
    };

    let res = profile.substitute_counsel(
        "counsel-001",
        new_counsel,
        "2026-02-15",
        Some("filings/substitution-notice-2026-02-15.pdf".into()),
    );
    assert!(res.is_ok());

    // Former counsel is inactive with history preserved
    let old = profile.roster.iter().find(|r| r.id == "counsel-001").unwrap();
    assert!(!old.is_active);
    assert_eq!(old.substitution_history.len(), 1);
    assert_eq!(old.substitution_history[0].substituted_by, "Robert Davis, Esq.");

    // New counsel is active
    let active = profile.roster.iter().find(|r| r.id == "counsel-002").unwrap();
    assert!(active.is_active);

    // Timeline event logged
    assert!(profile.timeline.iter().any(|t| t.title.contains("Substitution of Counsel")));
}

#[test]
fn test_issue_ledger_and_receipt_logging() {
    let mut profile = sample_appeal_profile();

    profile.upsert_issue(IssueItem {
        id: "issue-001".into(),
        title: "Untimely Rule 59 motion tolling dispute".into(),
        category: "jurisdictional".into(),
        asserted_by: "court".into(),
        record_citations: vec!["orders/2026-10-06-show-cause.pdf:p2".into()],
        contrary_authorities: vec!["Dobler v. CDOC, 2026".into()],
        status: "unresolved".into(),
        notes: "Colorado physical clerk receipt rule vs postmark date".into(),
    });

    assert_eq!(profile.issues.len(), 1);
    assert_eq!(profile.issues[0].status, "unresolved");

    profile.log_receipt(OperationReceipt {
        id: "rcpt-001".into(),
        timestamp: "2026-10-16T13:45:00Z".into(),
        operation: "order.decode".into(),
        invoked_by: "agent".into(),
        status: "needs_verification".into(),
        files_modified: vec!["orders/2026-10-06-show-cause.pdf".into()],
        notes: "Extracted candidate response deadline 2026-10-27".into(),
    });

    assert_eq!(profile.receipts.len(), 1);
    assert_eq!(profile.receipts[0].operation, "order.decode");
}

