use std::error::Error;
use serde_json::json;
use open_season_lib::case_profile::load_case;
use super::args::WorkflowCmd;

pub fn handle_workflow(cmd: WorkflowCmd, json: bool) -> Result<(), Box<dyn Error>> {
    match cmd {
        WorkflowCmd::Plan { path, objective } => {
            let profile = load_case(&path)?;
            let plan = open_season_lib::workflow::plan_workflow(&path, &profile, &objective);
            open_season_lib::workflow::save_workflow_plan(&path, &plan)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "never_serves": true,
                    "data": plan
                }))?);
            } else {
                println!("=== Formulated Workflow Plan: {} ===", plan.objective);
                println!("Plan ID: {}", plan.id);
                println!("Total Steps: {}", plan.steps.len());
                for (idx, step) in plan.steps.iter().enumerate() {
                    let req_approval = if step.requires_human_approval { " [APPROVAL REQUIRED]" } else { "" };
                    println!("  {}. [{:?}] {} (tool: {}){}", idx + 1, step.status, step.title, step.tool_command, req_approval);
                    println!("     {}", step.description);
                }
            }
        }
        WorkflowCmd::Status { path } => {
            let plan = open_season_lib::workflow::load_workflow_plan(&path);
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "never_serves": true,
                    "data": plan
                }))?);
            } else {
                match plan {
                    Some(p) => {
                        println!("=== Active Workflow Plan: {} ===", p.objective);
                        println!("Plan ID: {}", p.id);
                        println!("Status: Step {} of {} (Complete: {})", p.current_step_index + 1, p.steps.len(), p.is_complete);
                        if let Some(step) = p.steps.get(p.current_step_index) {
                            println!("\nCurrent Step: [{:?}] {}", step.status, step.title);
                            println!("Description: {}", step.description);
                            if step.requires_human_approval && !step.approval_granted {
                                println!("\n[!] HUMAN APPROVAL REQUIRED TO PROCEED:");
                                if let Some(prompt) = &step.approval_prompt {
                                    println!("    {}", prompt);
                                }
                                println!("    Run: openseason workflow advance <PATH> --approve");
                            }
                        }
                    }
                    None => {
                        println!("No active workflow plan found for case at {}", path.display());
                    }
                }
            }
        }
        WorkflowCmd::Advance { path, approve } => {
            let mut profile = load_case(&path)?;
            let mut plan = open_season_lib::workflow::load_workflow_plan(&path)
                .ok_or_else(|| "No active workflow plan found for this case. Formulate a plan first.".to_string())?;
            let res = open_season_lib::workflow::advance_workflow_step(&path, &mut profile, &mut plan, approve)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "never_serves": true,
                    "data": res
                }))?);
            } else {
                println!("=== Workflow Step Result: [{:?}] ===", res.status);
                println!("Step ID: {}", res.step_id);
                println!("Summary: {}", res.output_summary);
                if let Some(receipt) = &res.receipt_id {
                    println!("Receipt ID: {}", receipt);
                }
                if res.requires_approval {
                    println!("\n[!] ACTION REQUIRED: Human approval required before next step can execute.");
                    if let Some(p) = &res.approval_prompt {
                        println!("    {}", p);
                    }
                } else if let Some(prompt) = &res.next_step_prompt {
                    println!("\nNext Recommended Step: {}", prompt);
                }
            }
        }
    }
    Ok(())
}
