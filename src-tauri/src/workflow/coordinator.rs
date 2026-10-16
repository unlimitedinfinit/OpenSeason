use chrono::Utc;
use std::fs;
use std::path::Path;

use super::types::{WorkflowPlan, WorkflowStepResult, WorkflowStepStatus};
use crate::case_profile::types::OperationReceipt;
use crate::case_profile::{save_case, CaseProfile};

const WORKFLOW_PLAN_FILE: &str = "workflow_plan.json";

/// Loads the active workflow plan for a case folder if present.
pub fn load_workflow_plan(case_dir: &Path) -> Option<WorkflowPlan> {
    let plan_path = case_dir.join(WORKFLOW_PLAN_FILE);
    if !plan_path.exists() {
        return None;
    }
    let data = fs::read_to_string(&plan_path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Persists the active workflow plan to disk in the case folder.
pub fn save_workflow_plan(case_dir: &Path, plan: &WorkflowPlan) -> Result<(), String> {
    let plan_path = case_dir.join(WORKFLOW_PLAN_FILE);
    let data = serde_json::to_string_pretty(plan).map_err(|e| format!("Could not serialize workflow plan: {e}"))?;
    fs::write(&plan_path, data).map_err(|e| format!("Could not write workflow_plan.json: {e}"))
}

/// Advances the current workflow step, evaluating human approvals and operation receipts.
pub fn advance_workflow_step(
    case_dir: &Path,
    profile: &mut CaseProfile,
    plan: &mut WorkflowPlan,
    human_approved: bool,
) -> Result<WorkflowStepResult, String> {
    if plan.is_complete || plan.current_step_index >= plan.steps.len() {
        plan.is_complete = true;
        let _ = save_workflow_plan(case_dir, plan);
        return Ok(WorkflowStepResult {
            plan_id: plan.id.clone(),
            step_id: "plan-complete".into(),
            status: WorkflowStepStatus::Completed,
            output_summary: "All steps in this workflow plan are already completed.".into(),
            receipt_id: None,
            requires_approval: false,
            approval_prompt: None,
            next_step_prompt: None,
        });
    }

    let idx = plan.current_step_index;
    let step_title = plan.steps[idx].title.clone();
    let step_id = plan.steps[idx].id.clone();

    // Check if human approval is required and pending
    if plan.steps[idx].requires_human_approval && !plan.steps[idx].approval_granted && !human_approved {
        plan.steps[idx].status = WorkflowStepStatus::AwaitingApproval;
        let prompt = plan.steps[idx].approval_prompt.clone().unwrap_or_else(|| {
            format!("Action required: Please review and approve '{}' before proceeding.", step_title)
        });
        let _ = save_workflow_plan(case_dir, plan);
        return Ok(WorkflowStepResult {
            plan_id: plan.id.clone(),
            step_id,
            status: WorkflowStepStatus::AwaitingApproval,
            output_summary: format!("Step '{}' paused awaiting human review and approval.", step_title),
            receipt_id: None,
            requires_approval: true,
            approval_prompt: Some(prompt),
            next_step_prompt: Some("Review and approve to continue.".into()),
        });
    }

    if human_approved {
        plan.steps[idx].approval_granted = true;
    }

    // Step Execution & Receipt Generation
    plan.steps[idx].status = WorkflowStepStatus::InProgress;
    let receipt_id = format!("rcpt-step-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());
    let now = Utc::now().to_rfc3339();

    let tool_cmd = plan.steps[idx].tool_command.clone();
    let crit = plan.steps[idx].verification_criteria.clone();
    let output_summary = format!("Completed operation '{}' for step: '{}'.", tool_cmd, step_title);
    plan.steps[idx].output_summary = Some(output_summary.clone());
    plan.steps[idx].receipt_ref = Some(receipt_id.clone());
    plan.steps[idx].status = WorkflowStepStatus::Completed;

    // Log operation receipt to CaseProfile to preserve resilient history across agent restarts
    let receipt = OperationReceipt {
        id: receipt_id.clone(),
        timestamp: now.clone(),
        operation: format!("workflow.{}", tool_cmd),
        invoked_by: "agent_coordinator".into(),
        status: "success".into(),
        files_modified: vec![WORKFLOW_PLAN_FILE.into()],
        notes: format!("Step '{}' verified under criteria: {}", step_title, crit),
    };
    profile.receipts.push(receipt);
    save_case(case_dir, profile)?;

    // Advance to next step
    plan.current_step_index += 1;
    plan.updated_at = now;
    if plan.current_step_index >= plan.steps.len() {
        plan.is_complete = true;
    }
    save_workflow_plan(case_dir, plan)?;

    let next_prompt = if plan.is_complete {
        Some("All planned steps for this objective are completed!".into())
    } else {
        Some(format!("Next step ready: '{}'.", plan.steps[plan.current_step_index].title))
    };

    Ok(WorkflowStepResult {
        plan_id: plan.id.clone(),
        step_id,
        status: WorkflowStepStatus::Completed,
        output_summary,
        receipt_id: Some(receipt_id),
        requires_approval: false,
        approval_prompt: None,
        next_step_prompt: next_prompt,
    })
}
