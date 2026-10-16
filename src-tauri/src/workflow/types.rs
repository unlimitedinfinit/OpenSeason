use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStepStatus {
    Pending,
    AwaitingApproval,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

impl Default for WorkflowStepStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub tool_command: String,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub status: WorkflowStepStatus,
    pub requires_human_approval: bool,
    #[serde(default)]
    pub approval_prompt: Option<String>,
    #[serde(default)]
    pub approval_granted: bool,
    pub verification_criteria: String,
    #[serde(default)]
    pub receipt_ref: Option<String>,
    #[serde(default)]
    pub output_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowPlan {
    pub id: String,
    pub case_id: String,
    pub objective: String,
    pub created_at: String,
    pub updated_at: String,
    pub steps: Vec<WorkflowStep>,
    pub current_step_index: usize,
    pub is_complete: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowStepResult {
    pub plan_id: String,
    pub step_id: String,
    pub status: WorkflowStepStatus,
    pub output_summary: String,
    pub receipt_id: Option<String>,
    pub requires_approval: bool,
    pub approval_prompt: Option<String>,
    pub next_step_prompt: Option<String>,
}
