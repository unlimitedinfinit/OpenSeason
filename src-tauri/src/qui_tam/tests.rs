//! Unit tests for Qui Tam / EO 14395 Fraud Elimination Module.

use std::fs;
use uuid::Uuid;

use super::*;
use crate::case_profile::{CaseMode, CaseProfile};

fn make_test_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("test_quitam_{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_records_pack_generation_minnesota() {
    let dir = make_test_dir();
    let mut profile = CaseProfile::new_standard("Minnesota Care Center LLC", "qui_tam");
    profile.mode = CaseMode::Confidential;
    profile.governing_state = "MN".into();
    profile.caption.defendants.push(crate::case_profile::Party {
        name: "Minnesota Care Center LLC".into(),
        role: "Defendant / Target".into(),
        address: "100 Cedar Ave, Minneapolis, MN 55404".into(),
        ..Default::default()
    });

    let pack = generate_records_pack(&dir, &profile, Some("MN")).unwrap();
    assert_eq!(pack.state, "MN");
    assert!(pack.statute_title.contains("Minnesota Government Data Practices Act"));
    assert!(pack.formal_letter_text.contains("100 Cedar Ave"));
    assert!(pack.formal_letter_text.contains("FACILITY LICENSING APPLICATION"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_records_pack_generation_new_jersey() {
    let dir = make_test_dir();
    let profile = CaseProfile::new_standard("Sample Clinic NJ", "qui_tam");

    let pack = generate_records_pack(&dir, &profile, Some("NJ")).unwrap();
    assert_eq!(pack.state, "NJ");
    assert!(pack.statute_title.contains("New Jersey Open Public Records Act"));
    assert!(pack.formal_letter_text.contains("7 business days"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_rule9b_audit_scoring() {
    let dir = make_test_dir();
    let mut profile = CaseProfile::new_standard("Target Shell Hospital LLC", "qui_tam");
    profile.mode = CaseMode::Confidential;
    profile.caption.defendants.push(crate::case_profile::Party {
        name: "Target Shell Hospital LLC".into(),
        role: "Defendant".into(),
        address: "500 Lake St, Minneapolis, MN".into(),
        ..Default::default()
    });

    // Incomplete case should have score around 15 (entity identified only)
    let audit1 = audit_case_rule9b(&dir, &profile).unwrap();
    assert_eq!(audit1.score, 15);
    assert_eq!(audit1.viability_tier, "Preliminary / Speculative (Missing Core Rule 9(b) Elements)");
    assert!(!audit1.missing_requirements.is_empty());

    // When draft provides claims, falsity, scienter, and loss mentions
    profile.complaint.user_text = "Target knowingly submitted false Medicaid claims for unprovided inpatient services at a ghost clinic. Signed false certifications under penalty of perjury. Estimated federal loss exceeds $4,500,000.".into();

    let audit2 = audit_case_rule9b(&dir, &profile).unwrap();
    assert!(audit2.score >= 80);
    assert_eq!(audit2.viability_tier, "High Viability (Attorney & DOJ Review Ready)");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_observation_ingestion_and_chain() {
    let dir = make_test_dir();
    let dummy_photo = dir.join("test_observation.jpg");
    fs::write(&dummy_photo, b"FAKE_PHOTO_DATA_FOR_PROVENANCE_TEST_12345").unwrap();

    let entry = ingest_observation(
        &dir,
        &dummy_photo,
        "Observed clinic doors locked and lights off at 2:15 PM on a Tuesday.",
        Some("Citizen Investigator #42"),
    ).unwrap();

    assert!(!entry.sha256.is_empty());
    assert!(!entry.chain_of_custody_hash.is_empty());
    assert_eq!(entry.notes, "Observed clinic doors locked and lights off at 2:15 PM on a Tuesday.");

    let provenance_path = dir.join("evidence").join("provenance.json");
    assert!(provenance_path.exists());
    let content = fs::read_to_string(&provenance_path).unwrap();
    assert!(content.contains(&entry.sha256));
    assert!(content.contains(&entry.chain_of_custody_hash));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_standardized_calling_card_and_dual_track_formatting() {
    let dir = make_test_dir();
    let mut profile = CaseProfile::new_standard("Metro Hospice LLC", "qui_tam");
    profile.caption.defendants.push(crate::case_profile::Party {
        name: "Metro Hospice LLC".into(),
        role: "Defendant".into(),
        address: "700 Broadway, Denver, CO".into(),
        ..Default::default()
    });
    profile.filer.name = "Jordan Example".into();
    profile.complaint.user_text = "Target knowingly billed Medicaid for hospice care to patients not terminally ill. False certifications under penalty of perjury. Estimated federal loss exceeds $2,500,000.".into();

    let packet = export_doj_disclosure(&dir, &profile, Some("Provider Enrollments & Ghost Clinics"), None).unwrap();
    assert!(packet.calling_card.is_some());
    let card = packet.calling_card.as_ref().unwrap();
    assert!(card.formatted_header.contains("OPEN-SEASON STANDARDIZED FRAUD DISCLOSURE"));
    assert!(card.formatted_header.contains("https://justlegal.me/openseason-standard"));
    assert!(card.formatted_header.contains("RULE 9(b) READINESS GRADE"));

    // Track A: Attorney Inquiry Package
    let inquiry = generate_attorney_inquiry_package(&packet, &profile);
    assert!(inquiry.subject.contains("Metro Hospice LLC"));
    assert!(inquiry.privilege_notice.contains("ABA MODEL RULE 1.18"));
    assert!(inquiry.complete_email_body.contains("OPEN-SEASON STANDARDIZED FRAUD DISCLOSURE"));
    assert!(inquiry.transmission_checklist.len() >= 4);

    // Track B: Portal Submission Block
    let portal = generate_portal_submission_block(&packet, "HHS");
    assert_eq!(portal.agency, "HHS");
    assert!(portal.portal_url.contains("oig.hhs.gov"));
    assert!(portal.field_narrative_summary.contains("OPEN-SEASON STANDARDIZED FRAUD DISCLOSURE"));
    assert!(portal.field_suspect_entity.contains("Metro Hospice LLC"));

    let _ = fs::remove_dir_all(&dir);
}
