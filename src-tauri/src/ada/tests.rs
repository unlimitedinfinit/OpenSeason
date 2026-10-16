use super::generator::*;
use super::types::*;
use crate::case_profile::sample_appeal_profile;

#[test]
fn test_generate_ada_petition_nj_authorities() {
    let mut profile = sample_appeal_profile();
    profile.governing_state = "NJ".into();

    let payload = AdaRequestPayload {
        case_id: profile.id.clone(),
        requester_name: "Jordan Example".into(),
        requester_role: "Appellant (Self-Represented)".into(),
        contact_phone: "555-123-4567".into(),
        contact_email: "jordan@example.com".into(),
        accommodations: vec![
            AccommodationCategory::RemoteTelephonicAppearance,
            AccommodationCategory::ExtendedReadingTime,
        ],
        specific_needs_narrative: "Requester has severe mobility and respiratory limitations requiring virtual attendance.".into(),
        upcoming_date: Some("2026-11-15".into()),
    };

    let report = generate_ada_petition(&profile, &payload).unwrap();
    assert_eq!(report.state, "NJ");
    assert!(report.ada_coordinator_title.contains("Trial Court Administrator"));
    assert!(report.rendered_petition_text.contains("CONFIDENTIAL REQUEST FOR ADA ADMINISTRATIVE ACCOMMODATION"));
    assert!(report.rendered_petition_text.contains("Remote Video / Telephonic Appearance"));
    assert!(report.rendered_petition_text.contains("42 U.S.C. § 12132"));
    assert!(report.statutory_authorities.iter().any(|a| a.contains("Directive #01-19")));
}
