use super::types::case_dir_from_id;
use crate::case_profile::load_case;

#[tauri::command]
pub fn record_inventory_cmd(case_id: String) -> Result<crate::record::RecordInventoryReport, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = load_case(&dir)?;
    crate::record::inventory::scan_case_record(&dir, &profile)
}

#[tauri::command]
pub fn record_docket_match_cmd(
    case_id: String,
    docket_text: String,
) -> Result<crate::record::RecordInventoryReport, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = load_case(&dir)?;
    let entries = crate::record::reconciliation::parse_text_docket(&docket_text);
    crate::record::reconciliation::reconcile_docket(&dir, &profile, &entries)
}

#[tauri::command]
pub fn record_missing_cmd(case_id: String) -> Result<Vec<crate::record::MissingRecordItem>, String> {
    let dir = case_dir_from_id(&case_id)?;
    let inv = crate::record::inventory::load_record_inventory(&dir);
    Ok(inv.map(|r| r.missing_records).unwrap_or_default())
}

#[tauri::command]
pub fn record_request_form_cmd(
    case_id: String,
    state: Option<String>,
) -> Result<crate::record::CourtRecordsRequestPacket, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = load_case(&dir)?;
    crate::record::requests::generate_records_request(&dir, &profile, state.as_deref())
}

#[tauri::command]
pub fn accountability_audit_cmd(case_id: String) -> Result<crate::accountability::DiscrepancyReport, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = load_case(&dir)?;
    crate::accountability::audit::audit_record_discrepancies(&dir, &profile)
}

#[tauri::command]
pub fn accountability_routes_cmd(
    case_id: String,
    issue_id: Option<String>,
) -> Result<Vec<crate::accountability::AccountabilityRoute>, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = load_case(&dir)?;
    let report = crate::accountability::audit::audit_record_discrepancies(&dir, &profile)?;
    if let Some(target_id) = issue_id {
        if let Some(issue) = report.issues.into_iter().find(|i| i.id == target_id) {
            return Ok(issue.available_routes);
        }
    }
    // Default routes for case posture
    Ok(crate::accountability::routes::determine_routes_for_issue(
        &profile,
        crate::accountability::types::RecordSupportStatus::UnsupportedInRecord,
        false,
    ))
}

#[tauri::command]
pub fn accountability_resources_cmd(
    state: Option<String>,
    category: Option<String>,
) -> Result<Vec<crate::accountability::AccountabilityResource>, String> {
    Ok(crate::accountability::resources::list_accountability_resources(
        state.as_deref(),
        category.as_deref(),
    ))
}

#[tauri::command]
pub fn accountability_packet_cmd(
    case_id: String,
    issue_id: String,
) -> Result<crate::accountability::FactualPacketReport, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = load_case(&dir)?;
    let report = crate::accountability::audit::audit_record_discrepancies(&dir, &profile)?;
    let issue = report
        .issues
        .into_iter()
        .find(|i| i.id == issue_id)
        .ok_or_else(|| format!("Issue '{issue_id}' not found in case."))?;
    crate::accountability::packets::assemble_factual_packet(&dir, &profile, &issue)
}

#[tauri::command]
pub fn ada_petition_cmd(payload: crate::ada::AdaRequestPayload) -> Result<crate::ada::AdaPetitionReport, String> {
    let dir = case_dir_from_id(&payload.case_id)?;
    let profile = load_case(&dir)?;
    crate::ada::generator::generate_ada_petition(&profile, &payload)
}

#[tauri::command]
pub fn order_decode_cmd(pdf_path: String) -> Result<crate::order::DecodedOrder, String> {
    let path = std::path::PathBuf::from(&pdf_path);
    crate::order::decoder::decode_order_pdf(&path)
}

#[tauri::command]
pub fn timeline_audit_mail_cmd(
    case_id: String,
    order_entry_date: String,
    mailing_postmark: Option<String>,
    delivery_receipt_date: Option<String>,
    clerk_receipt_date: Option<String>,
    governing_state: Option<String>,
) -> Result<crate::order::MailAuditReport, String> {
    Ok(crate::order::mail_audit::audit_mail_gap(
        &case_id,
        &order_entry_date,
        mailing_postmark.as_deref(),
        delivery_receipt_date.as_deref(),
        clerk_receipt_date.as_deref(),
        governing_state.as_deref(),
    ))
}

#[tauri::command]
pub fn case_bootstrap_cmd(
    source_folder: String,
    destination_parent: Option<String>,
    case_title: Option<String>,
) -> Result<crate::order::BootstrapResult, String> {
    let src = std::path::PathBuf::from(&source_folder);
    let dest = if let Some(d) = destination_parent.filter(|s| !s.trim().is_empty()) {
        std::path::PathBuf::from(d)
    } else {
        crate::case_profile::storage::default_cases_root()
    };
    crate::order::bootstrap::bootstrap_case_from_folder(&src, &dest, case_title.as_deref())
}

#[tauri::command]
pub fn court_access_directory_cmd() -> Result<Vec<crate::court_access::CourtAccessProfile>, String> {
    Ok(crate::court_access::get_all_court_profiles())
}

#[tauri::command]
pub fn court_access_profile_cmd(query: String) -> Result<Option<crate::court_access::CourtAccessProfile>, String> {
    Ok(crate::court_access::find_court_access_profile(&query))
}

#[tauri::command]
pub fn record_retrieval_plan_cmd(case_id: String) -> Result<crate::court_access::RecordRetrievalWizardPlan, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = load_case(&dir)?;
    Ok(crate::court_access::plan_record_retrieval(&dir, &profile))
}

#[tauri::command]
pub fn record_retrieval_execute_cmd(
    case_id: String,
    state_override: Option<String>,
) -> Result<crate::record::CourtRecordsRequestPacket, String> {
    let dir = case_dir_from_id(&case_id)?;
    let mut profile = load_case(&dir)?;
    crate::court_access::execute_record_retrieval_request(&dir, &mut profile, state_override.as_deref())
}

#[tauri::command]
pub fn workflow_plan_cmd(case_id: String, objective: String) -> Result<crate::workflow::WorkflowPlan, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = load_case(&dir)?;
    let plan = crate::workflow::plan_workflow(&dir, &profile, &objective);
    crate::workflow::save_workflow_plan(&dir, &plan)?;
    Ok(plan)
}

#[tauri::command]
pub fn workflow_status_cmd(case_id: String) -> Result<Option<crate::workflow::WorkflowPlan>, String> {
    let dir = case_dir_from_id(&case_id)?;
    Ok(crate::workflow::load_workflow_plan(&dir))
}

#[tauri::command]
pub fn workflow_advance_cmd(
    case_id: String,
    approve: bool,
) -> Result<crate::workflow::WorkflowStepResult, String> {
    let dir = case_dir_from_id(&case_id)?;
    let mut profile = load_case(&dir)?;
    let mut plan = crate::workflow::load_workflow_plan(&dir)
        .ok_or_else(|| "No active workflow plan found for this case. Formulate a plan first.".to_string())?;
    crate::workflow::advance_workflow_step(&dir, &mut profile, &mut plan, approve)
}

#[tauri::command]
pub fn stress_test_issue_cmd(
    case_id: String,
    issue_id: Option<String>,
) -> Result<Vec<crate::consultation::IssueStressTestResult>, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = load_case(&dir)?;
    if let Some(target_id) = issue_id {
        if let Some(issue) = profile.issues.iter().find(|i| i.id == target_id) {
            return Ok(vec![crate::consultation::stress_test_issue(&profile, issue)]);
        }
        return Err(format!("Issue with id '{}' not found in case.", target_id));
    }
    Ok(crate::consultation::stress_test_all_issues(&profile))
}

#[tauri::command]
pub fn consultation_dossier_cmd(
    case_id: String,
    custom_questions: Option<Vec<String>>,
) -> Result<crate::consultation::ConsultationDossier, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = load_case(&dir)?;
    let questions = custom_questions.unwrap_or_default();
    crate::consultation::generate_consultation_dossier(&dir, &profile, &questions)
}

#[tauri::command]
pub fn extension_evaluate_cmd(
    case_id: String,
    target_deadline: String,
    requested_days: u32,
    consent_obtained: bool,
) -> Result<crate::consultation::ExtensionProposal, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = load_case(&dir)?;
    Ok(crate::consultation::evaluate_extension_request(
        &profile,
        &target_deadline,
        requested_days,
        consent_obtained,
    ))
}

#[tauri::command]
pub fn extension_suggest_cmd(
    case_id: String,
) -> Result<crate::consultation::BaselineExtensionAssessment, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = load_case(&dir)?;
    Ok(crate::consultation::assess_baseline_extension(
        &profile,
        Some(&dir),
    ))
}



