#[cfg(test)]
mod tests {
    use super::super::coordinator::{advance_workflow_step, load_workflow_plan, save_workflow_plan};
    use super::super::planner::plan_workflow;
    use super::super::types::WorkflowStepStatus;
    use crate::case_profile::storage::create_case_folder;
    use crate::case_profile::CaseMode;

    #[test]
    fn test_workflow_planner_creates_ordered_steps_with_approval_gates() {
        let temp_dir = std::env::temp_dir().join(format!("test-wf-plan-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let (case_path, profile) = create_case_folder(&temp_dir, "Jordan Example Appeal", CaseMode::Standard, "motion").unwrap();

        let plan = plan_workflow(&case_path, &profile, "File a timely notice of appeal");
        assert!(plan.steps.len() >= 4);
        assert_eq!(plan.current_step_index, 0);
        assert!(!plan.is_complete);

        // Verification: initial steps are read-only, final steps require human approval
        let draft_step = plan.steps.iter().find(|s| s.title.contains("Notice of Appeal")).unwrap();
        assert!(draft_step.requires_human_approval);
        assert!(draft_step.approval_prompt.is_some());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_workflow_step_advancement_and_approval_checkpoint() {
        let temp_dir = std::env::temp_dir().join(format!("test-wf-exec-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let (case_path, mut profile) = create_case_folder(&temp_dir, "Example Matter", CaseMode::Standard, "motion").unwrap();
        let mut plan = plan_workflow(&case_path, &profile, "obtain missing records and transcript");
        save_workflow_plan(&case_path, &plan).unwrap();

        // Step 1: Does not require approval
        let res1 = advance_workflow_step(&case_path, &mut profile, &mut plan, false).unwrap();
        assert_eq!(res1.status, WorkflowStepStatus::Completed);
        assert_eq!(plan.current_step_index, 1);
        assert!(profile.receipts.len() == 1);

        // Step 2: Does not require approval
        let res2 = advance_workflow_step(&case_path, &mut profile, &mut plan, false).unwrap();
        assert_eq!(res2.status, WorkflowStepStatus::Completed);
        assert_eq!(plan.current_step_index, 2);

        // Step 3: Requires human approval! If unapproved, must pause with AwaitingApproval
        let res3 = advance_workflow_step(&case_path, &mut profile, &mut plan, false).unwrap();
        assert_eq!(res3.status, WorkflowStepStatus::AwaitingApproval);
        assert!(res3.requires_approval);
        assert!(res3.approval_prompt.is_some());
        assert_eq!(plan.current_step_index, 2); // Did NOT advance without approval!

        // Now human provides approval
        let res3_approved = advance_workflow_step(&case_path, &mut profile, &mut plan, true).unwrap();
        assert_eq!(res3_approved.status, WorkflowStepStatus::Completed);
        assert!(plan.is_complete);

        // Verify loaded plan from disk matches
        let loaded = load_workflow_plan(&case_path).unwrap();
        assert!(loaded.is_complete);
        assert_eq!(loaded.current_step_index, 3);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
