#[cfg(test)]
mod tests {
    use crate::case_profile::sample_appeal_profile;
    use crate::case_profile::types::IssueItem;
    use crate::consultation::dossier::generate_consultation_dossier;
    use crate::consultation::extension::evaluate_extension_request;
    use crate::consultation::stress_test::stress_test_issue;
    use crate::consultation::types::{ExtensionRoute, VulnerabilitySeverity, VulnerabilityType};

    #[test]
    fn test_stress_test_flags_missing_grounding_and_standard_of_review() {
        let profile = sample_appeal_profile();
        let issue = IssueItem {
            id: "issue-1".into(),
            title: "Trial court denied motion for reconsideration without hearing".into(),
            category: "Procedural Discretion".into(),
            asserted_by: "Appellant".into(),
            record_citations: vec![],
            contrary_authorities: vec![],
            status: "active".into(),
            notes: String::new(),
        };

        let result = stress_test_issue(&profile, &issue);
        assert_eq!(result.issue_id, "issue-1");
        assert_eq!(result.overall_risk_rating, "High Procedural Risk");

        // Verify MissingRecordGrounding is flagged
        let missing_grounding = result
            .vulnerabilities
            .iter()
            .find(|v| v.vulnerability_type == VulnerabilityType::MissingRecordGrounding);
        assert!(missing_grounding.is_some());
        assert_eq!(missing_grounding.unwrap().severity, VulnerabilitySeverity::Critical);

        // Verify StandardOfReview is flagged
        let std_review = result
            .vulnerabilities
            .iter()
            .find(|v| v.vulnerability_type == VulnerabilityType::StandardOfReview);
        assert!(std_review.is_some());
        assert_eq!(std_review.unwrap().severity, VulnerabilitySeverity::Significant);
    }

    #[test]
    fn test_extension_wizard_prohibits_appeal_clock_beyond_30_days() {
        let profile = sample_appeal_profile();
        let proposal = evaluate_extension_request(&profile, "Notice of Appeal", 45, false);
        assert_eq!(proposal.route, ExtensionRoute::ProhibitedByRule);
        assert!(proposal.non_alterable_warning.is_some());
    }

    #[test]
    fn test_extension_wizard_routes_consent_stipulation_vs_formal_motion() {
        let profile = sample_appeal_profile();

        // With consent for appellant brief: automatic consent letter
        let consent_prop = evaluate_extension_request(&profile, "Appellant Brief", 30, true);
        assert_eq!(consent_prop.route, ExtensionRoute::ConsentStipulationLetter);
        assert!(consent_prop.rendered_document_text.contains("CONSENT STIPULATION"));
        assert!(!consent_prop.good_cause_required);

        // Without consent: formal motion requiring good cause
        let motion_prop = evaluate_extension_request(&profile, "Appellant Brief", 30, false);
        assert_eq!(motion_prop.route, ExtensionRoute::FormalMotionForEnlargement);
        assert!(motion_prop.rendered_document_text.contains("NOTICE OF MOTION"));
        assert!(motion_prop.good_cause_required);
        assert_eq!(motion_prop.recommended_form_number.as_deref(), Some("Form CN 10503"));
    }

    #[test]
    fn test_consultation_dossier_renders_full_packet() {
        let tmp_dir = std::env::temp_dir().join(format!("test_dossier_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&tmp_dir);
        let mut profile = sample_appeal_profile();
        profile.governing_state = "NJ".into();
        profile.issues = vec![IssueItem {
            id: "iss-101".into(),
            title: "Exclusion of witness testimony".into(),
            category: "Evidentiary".into(),
            asserted_by: "Jordan Example".into(),
            record_citations: vec!["[Tr. 45:10]".into()],
            contrary_authorities: vec![],
            status: "active".into(),
            notes: "Timely objected on the record".into(),
        }];

        let custom_q = vec!["What are the odds on emergency stay pending appeal?".to_string()];
        let dossier = generate_consultation_dossier(&tmp_dir, &profile, &custom_q).unwrap();

        assert_eq!(dossier.case_id, profile.id);
        assert_eq!(dossier.jurisdiction, "NJ");
        assert!(dossier.rendered_markdown.contains("ATTORNEY CONSULTATION DOSSIER"));
        assert!(dossier.rendered_markdown.contains("Exclusion of witness testimony"));
        assert!(dossier.rendered_markdown.contains("What are the odds on emergency stay"));
        assert!(tmp_dir.join("exports/consultation_dossier.md").exists());
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
