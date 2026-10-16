export type WorkflowStepStatus =
  | "Pending"
  | "InProgress"
  | "AwaitingApproval"
  | "Completed"
  | "Failed"
  | "Skipped";

export interface WorkflowStep {
  id: string;
  title: string;
  description: string;
  tool_command: string;
  params: Record<string, unknown>;
  status: WorkflowStepStatus;
  requires_human_approval: boolean;
  approval_prompt?: string | null;
  approval_granted: boolean;
  verification_criteria: string;
  receipt_ref?: string | null;
  output_summary?: string | null;
}

export interface WorkflowPlan {
  id: string;
  objective: string;
  created_at: string;
  updated_at: string;
  current_step_index: number;
  steps: WorkflowStep[];
  is_complete: boolean;
}

export interface WorkflowStepResult {
  plan_id: string;
  step_id: string;
  status: WorkflowStepStatus;
  output_summary: string;
  receipt_id?: string | null;
  requires_approval: boolean;
  approval_prompt?: string | null;
  next_step_prompt?: string | null;
}
