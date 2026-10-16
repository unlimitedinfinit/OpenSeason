#[cfg(test)]
mod tests {
    use super::super::directory::{find_court_access_profile, get_all_court_profiles};
    use super::super::wizard::{execute_record_retrieval_request, plan_record_retrieval};
    use crate::case_profile::storage::create_case_folder;
    use crate::case_profile::CaseMode;

    #[test]
    fn test_court_access_directory_contains_nj_co_and_fed() {
        let all = get_all_court_profiles();
        assert!(all.len() >= 4);

        let nj = find_court_access_profile("nj").expect("NJ profile should exist");
        assert!(nj.court_name.contains("New Jersey"));
        assert!(nj.transcript_request_form.contains("CN 10200"));
        assert!(nj.pro_se_efiling_allowed);

        let co = find_court_access_profile("colorado").expect("Colorado profile should exist");
        assert!(co.court_name.contains("Colorado"));
        assert!(co.transcript_request_form.contains("JDF 4"));
        assert!(co.special_instructions.iter().any(|s| s.contains("CLERK RECEIPT RULE")));

        let fed = find_court_access_profile("federal").expect("Federal profile should exist");
        assert!(fed.electronic_portal_name.contains("PACER"));
    }

    #[test]
    fn test_record_retrieval_wizard_plan_and_execution() {
        let temp_dir = std::env::temp_dir().join(format!("test-court-access-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let (case_path, mut profile) = create_case_folder(&temp_dir, "Example v. Clerk", CaseMode::Standard, "motion").unwrap();
        profile.court.name = "Superior Court of New Jersey, Appellate Division".into();
        profile.docket_number = "A-001234-24".into();

        let plan = plan_record_retrieval(&case_path, &profile);
        assert_eq!(plan.recommended_form_number, "CN 10200");
        assert!(!plan.missing_records.is_empty());
        assert!(plan.fee_waiver_rule.is_some());
        assert_eq!(plan.followup_reminder_days, 14);

        let packet = execute_record_retrieval_request(&case_path, &mut profile, Some("NJ")).unwrap();
        assert_eq!(packet.state, "NJ");
        assert!(packet.rendered_request_text.contains("CN 10200"));

        // Verify task and receipt logged in profile
        assert!(profile.tasks.iter().any(|t| t.title.contains("Court Records Request")));
        assert!(profile.receipts.iter().any(|r| r.operation == "court_records_request_generated"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
