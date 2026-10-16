use super::inventory::*;
use super::reconciliation::*;
use super::requests::*;
use super::types::*;
use crate::case_profile::sample_appeal_profile;

#[test]
fn test_evidentiary_tier_properties() {
    let t1 = EvidentiaryTier::OfficialCourtRecord;
    let t2 = EvidentiaryTier::HearingAndEvidentiaryRecord;
    let t3 = EvidentiaryTier::PersonalCaseArchive;
    let t4 = EvidentiaryTier::PotentialAdditionalEvidence;

    assert!(t1.is_official());
    assert!(t2.is_official());
    assert!(!t3.is_official());
    assert!(!t4.is_official());
    assert_eq!(t1.label(), "Tier 1: Official Court Record");
    assert_eq!(t4.label(), "Tier 4: Potential Additional Evidence (Unsubmitted)");
}

#[test]
fn test_parse_text_docket() {
    let docket_text = "2026-05-04\tMotion for Leave to File\tFiled by Appellant\n\
                       2026-05-18\tOpposition to Motion\tFiled by Respondent\n\
                       2026-06-10\tOrder of Dismissal\tSigned by Judge Sample\n\
                       2026-06-24\tOral Argument Proceeding\tHearing held before Appellate Panel\n";

    let entries = parse_text_docket(docket_text);
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].date, "2026-05-04");
    assert_eq!(entries[0].title, "Motion for Leave to File");
    assert_eq!(entries[2].title, "Order of Dismissal");
    assert_eq!(entries[3].title, "Oral Argument Proceeding");
}

fn temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rec-test-{}", uuid::Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[test]
fn test_reconciliation_detects_missing_hearing_transcript() {
    let dir = temp_dir();
    let profile = sample_appeal_profile();

    let docket_text = "2026-06-10\tOrder of Dismissal\tOrder\n\
                       2026-06-24\tOral Argument Proceeding\tHearing\n";
    let entries = parse_text_docket(docket_text);

    let report = reconcile_docket(&dir, &profile, &entries).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    // Should flag missing order and missing hearing transcript
    assert_eq!(report.missing_records.len(), 2);
    let transcript_item = report.missing_records.iter().find(|m| m.record_type == "transcript");
    assert_eq!(transcript_item.unwrap().title, "Oral Argument Proceeding");
    assert!(transcript_item.unwrap().reason.contains("2026-06-24"));
    assert!(transcript_item.unwrap().request_form.is_some());
}

#[test]
fn test_generate_records_request_nj_fees() {
    let dir = temp_dir();
    let profile = sample_appeal_profile();

    let packet = generate_records_request(&dir, &profile, Some("NJ")).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(packet.state, "NJ");
    assert!(packet.statutory_fee_notice.contains("5¢"));
    assert!(packet.statutory_fee_notice.contains("7¢"));
    assert!(packet.rendered_request_text.contains("OFFICIAL COURT RECORDS REQUEST"));
    assert!(packet.rendered_request_text.contains(&profile.docket_number));
}

#[test]
fn test_provenance_dimensions_independent() {
    let mut prov = DocumentProvenance::default();
    assert_eq!(prov.origin, "unknown");
    assert_eq!(prov.version, 1);
    assert_eq!(prov.evidentiary_tier, EvidentiaryTier::PersonalCaseArchive);
    assert_eq!(prov.completeness, "unverified");

    // Modify dimensions independently
    prov.origin = "court".into();
    prov.submission_channel = Some("eCourts".into());
    prov.accepted_date = Some("2026-03-01".into());
    prov.docket_sequence = Some(14);
    prov.served_recipients.push("opposing_counsel_001".into());
    prov.evidentiary_tier = EvidentiaryTier::OfficialCourtRecord;
    prov.completeness = "complete".into();

    assert_eq!(prov.origin, "court");
    assert_eq!(prov.docket_sequence, Some(14));
    assert_eq!(prov.served_recipients.len(), 1);
    assert!(prov.evidentiary_tier.is_official());
}

