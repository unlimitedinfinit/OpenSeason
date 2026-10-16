use std::path::{Path, PathBuf};

use crate::case_profile;
use super::types::{case_dir_from_id, cases_root};

#[tauri::command]
pub fn list_rule_packs() -> Result<Vec<crate::rule_pack::RulePack>, String> {
    Ok(crate::rule_pack::RulePack::list_builtin())
}

#[tauri::command]
pub fn list_form_packs_cmd() -> Result<crate::forms::PackList, String> {
    crate::forms::list_packs(None, None)
}

#[tauri::command]
pub fn install_form_pack_cmd(
    state: String,
    situation: Option<String>,
    file: Option<String>,
) -> Result<crate::forms::InstallReport, String> {
    crate::forms::install(crate::forms::InstallOpts {
        state,
        situation,
        catalog: None,
        file: file.map(std::path::PathBuf::from),
        root: None,
        // Clicking Install is consent to fetch a verified file. Fixture example.test URLs are still refused.
        allow_network: true,
    })
}

#[tauri::command]
pub fn remove_form_pack_cmd(state: String) -> Result<String, String> {
    crate::forms::remove_pack(&state, None)
}

#[tauri::command]
pub fn verify_form_pack_cmd(state: Option<String>) -> Result<Vec<crate::forms::VerifyReport>, String> {
    crate::forms::verify_pack(state.as_deref(), None)
}

#[tauri::command]
pub fn guide_questions_cmd() -> Result<crate::guide::GuideQuestions, String> {
    crate::guide::questions()
}

#[tauri::command]
pub fn guide_find_cmd(
    state: String,
    court: Option<String>,
    situation: String,
    trigger_date: Option<String>,
) -> Result<crate::guide::GuideResult, String> {
    crate::guide::find(
        &state,
        court.as_deref(),
        &situation,
        trigger_date.as_deref(),
    )
}

#[tauri::command]
pub fn help_links_cmd() -> Result<crate::help_links::HelpLinks, String> {
    crate::help_links::HelpLinks::load()
}

#[tauri::command]
pub fn check_case_cmd(
    case_id: String,
    pack: Option<String>,
    rule: Option<String>,
    trigger_date: Option<String>,
) -> Result<crate::check::CheckAllReport, String> {
    crate::check::run_all(
        &case_dir_from_id(&case_id)?,
        pack.as_deref(),
        rule.as_deref(),
        trigger_date.as_deref(),
    )
}

#[tauri::command]
pub fn deadline_cmd(
    case_id: String,
    pack: String,
    rule: String,
    trigger_date: String,
) -> Result<crate::deadlines::DeadlineResult, String> {
    crate::check::run_deadline(&case_dir_from_id(&case_id)?, &pack, &rule, &trigger_date)
}

#[tauri::command]
pub fn guided_sections_cmd(case_id: String) -> Result<crate::guided::GuidedDraft, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = case_profile::load_case(&dir)?;
    crate::guided::load_or_init(&dir, &profile.document_kind)
}

#[tauri::command]
pub fn guided_fill_cmd(case_id: String, id: String, text: String) -> Result<crate::guided::GuidedDraft, String> {
    crate::guided::set_section(&case_dir_from_id(&case_id)?, &id, &text)
}

#[tauri::command]
pub fn exhibit_index_cmd(case_id: String) -> Result<crate::exhibits::ExhibitIndex, String> {
    crate::exhibits::rebuild(&case_dir_from_id(&case_id)?)
}

#[tauri::command]
pub fn link_fact_cmd(case_id: String, fact_id: u32, letter: String) -> Result<crate::check::FactCheckReport, String> {
    crate::exhibits::link_fact(&case_dir_from_id(&case_id)?, fact_id, &letter)
}

#[tauri::command]
pub fn set_fact_cite_cmd(
    case_id: String,
    fact_id: u32,
    cite: String,
    exhibit: String,
    verified: bool,
) -> Result<crate::check::FactCheckReport, String> {
    crate::check::set_fact_cite(&case_dir_from_id(&case_id)?, fact_id, &cite, &exhibit, verified)
}

#[tauri::command]
pub fn set_authority_cmd(
    case_id: String,
    authority_id: u32,
    status: String,
    source: String,
    note: String,
) -> Result<crate::check::AuthorityReport, String> {
    let st = match status.as_str() {
        "verified" => crate::check::AuthorityStatus::Verified,
        "not_found" => crate::check::AuthorityStatus::NotFound,
        _ => crate::check::AuthorityStatus::Unverified,
    };
    crate::check::set_authority_status(&case_dir_from_id(&case_id)?, authority_id, st, &source, &note)
}

#[tauri::command]
pub fn set_signoff_cmd(case_id: String, id: String, checked: bool) -> Result<crate::check::SignoffReport, String> {
    crate::check::set_signoff(&case_dir_from_id(&case_id)?, &id, checked)
}

#[tauri::command]
pub fn export_packet_cmd(case_id: String) -> Result<crate::packet::PacketReport, String> {
    crate::packet::assemble_for_case(&case_dir_from_id(&case_id)?)
}

#[tauri::command]
pub fn export_ics_cmd(
    case_id: String,
    pack: String,
    rule: String,
    trigger_date: String,
) -> Result<String, String> {
    let dir = case_dir_from_id(&case_id)?;
    let result = crate::check::run_deadline(&dir, &pack, &rule, &trigger_date)?;
    Ok(crate::calendar::write_ics(&dir, &result)?.display().to_string())
}

#[tauri::command]
pub fn seed_sample_cmd() -> Result<crate::case_profile::CaseProfile, String> {
    let dest = cases_root().join("lucille-example");
    if dest.join("case.json").is_file() {
        return case_profile::load_case(&dest);
    }
    let (_dir, profile) = crate::seed::seed_lucille(&dest)?;
    Ok(profile)
}

#[tauri::command]
pub fn gap_add_cmd(
    case_id: String,
    title: String,
    court: String,
    rule: String,
    what_happened: String,
    what_was_hard: String,
) -> Result<crate::gaps::GapLog, String> {
    crate::gaps::add(
        &case_dir_from_id(&case_id)?,
        &title,
        &court,
        &rule,
        &what_happened,
        &what_was_hard,
    )
}

#[tauri::command]
pub fn gap_export_cmd(case_id: String) -> Result<String, String> {
    Ok(crate::gaps::export_markdown(&case_dir_from_id(&case_id)?)?.display().to_string())
}

#[tauri::command]
pub fn field_help_cmd() -> Result<Vec<crate::help::FieldHelp>, String> {
    Ok(crate::help::field_help())
}

#[tauri::command]
pub fn playbook_cmd() -> Result<String, String> {
    Ok(crate::help::playbook())
}

#[tauri::command]
pub fn cloud_login_cmd(email: String, password: String) -> Result<crate::sync::CloudSession, String> {
    crate::sync::cloud_sign_in_password(&email, &password)
}

#[tauri::command]
pub fn cloud_status_cmd() -> Result<Option<crate::sync::CloudSession>, String> {
    Ok(crate::sync::load_cloud_session())
}

#[tauri::command]
pub fn cloud_logout_cmd() -> Result<(), String> {
    crate::sync::clear_cloud_session()
}

#[tauri::command]
pub fn cloud_pull_cmd(case_id: Option<String>) -> Result<crate::sync::PullReport, String> {
    let session = crate::sync::load_cloud_session().ok_or_else(|| "Not logged in to JustLegal cloud. Please log in first.".to_string())?;
    crate::sync::cloud_pull_all(&session, &cases_root(), case_id.as_deref())
}

#[tauri::command]
pub fn cloud_push_cmd(case_id: String) -> Result<crate::sync::PushReport, String> {
    let session = crate::sync::load_cloud_session().ok_or_else(|| "Not logged in to JustLegal cloud. Please log in first.".to_string())?;
    let dir = case_dir_from_id(&case_id)?;
    crate::sync::cloud_push_case(&dir, &session)
}

#[tauri::command]
pub fn cloud_get_case_sync_meta_cmd(case_id: String) -> Result<crate::sync::CaseSyncMetadata, String> {
    let dir = case_dir_from_id(&case_id)?;
    Ok(crate::sync::load_case_sync_metadata(&dir))
}

#[tauri::command]
pub fn preflight_audit_cmd(case_id: String) -> Result<crate::check::PreFlightAudit, String> {
    let dir = case_dir_from_id(&case_id)?;
    crate::check::preflight_audit(&dir)
}

#[tauri::command]
pub fn import_folder_cmd(
    case_id: Option<String>,
    from: String,
    title: Option<String>,
) -> Result<crate::folder_import::FolderImportReport, String> {
    let dest = match case_id {
        Some(id) if !id.trim().is_empty() => case_dir_from_id(&id)?,
        _ => cases_root(),
    };
    crate::folder_import::import_folder(&dest, std::path::Path::new(&from), title.as_deref())
}

#[tauri::command]
pub fn agent_manifest_cmd() -> Result<serde_json::Value, String> {
    Ok(crate::help::agent_manifest())
}

#[tauri::command]
pub fn export_exhibit_packet_cmd(
    case_id: String,
    bates_prefix: Option<String>,
    starting_number: Option<u32>,
) -> Result<crate::packet::ExhibitPacketReport, String> {
    let dir = case_dir_from_id(&case_id)?;
    crate::packet::assemble_exhibit_packet(&dir, bates_prefix.as_deref(), starting_number)
}

#[tauri::command]
pub fn inspect_court_form_cmd(case_id: Option<String>, form_path: String) -> Result<crate::forms::FormInspection, String> {
    let case_dir = match &case_id {
        Some(id) if !id.trim().is_empty() => Some(case_dir_from_id(id)?),
        _ => None,
    };
    crate::forms::inspect_court_form(std::path::Path::new(&form_path), case_dir.as_deref())
}

#[tauri::command]
pub fn fill_court_form_cmd(
    case_id: String,
    form_path: String,
    custom_values: Option<std::collections::HashMap<String, String>>,
) -> Result<crate::forms::FormFillResult, String> {
    let dir = case_dir_from_id(&case_id)?;
    crate::forms::fill_court_form(std::path::Path::new(&form_path), &dir, None, custom_values)
}

#[tauri::command]
pub fn check_transcripts_cmd(case_id: String) -> Result<crate::check::TranscriptReport, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = crate::case_profile::load_case(&dir)?;
    let user_draft = profile.notice_of_appeal.user_text.as_str();
    crate::check::check_transcripts(&dir, user_draft)
}

#[tauri::command]
pub fn pdf_merge_cmd(input_paths: Vec<String>, output_path: String) -> Result<crate::pdf_tools::PdfOperationResult, String> {
    let paths: Vec<PathBuf> = input_paths.into_iter().map(PathBuf::from).collect();
    crate::pdf_tools::merge_pdfs(&paths, Path::new(&output_path))
}

#[tauri::command]
pub fn pdf_split_cmd(input_path: String, pages_to_keep: Vec<u32>, output_path: String) -> Result<crate::pdf_tools::PdfOperationResult, String> {
    crate::pdf_tools::split_pdf(Path::new(&input_path), &pages_to_keep, Path::new(&output_path))
}

#[tauri::command]
pub fn pdf_rotate_cmd(input_path: String, degrees: i64, page_numbers: Option<Vec<u32>>, output_path: String) -> Result<crate::pdf_tools::PdfOperationResult, String> {
    crate::pdf_tools::rotate_pdf(Path::new(&input_path), degrees, page_numbers, Path::new(&output_path))
}

#[tauri::command]
pub fn pdf_redact_cmd(input_path: String, custom_patterns: Vec<String>, output_path: String) -> Result<crate::pdf_tools::RedactOperationResult, String> {
    crate::pdf_tools::redact_pdf_text(Path::new(&input_path), &custom_patterns, Path::new(&output_path))
}

#[tauri::command]
pub fn get_signature_profile_cmd() -> Result<crate::signature::SignatureProfile, String> {
    crate::signature::load_signature_profile()
}

#[tauri::command]
pub fn save_signature_profile_cmd(profile: crate::signature::SignatureProfile) -> Result<bool, String> {
    crate::signature::save_signature_profile(&profile)?;
    Ok(true)
}

#[tauri::command]
pub fn stamp_pdf_signature_cmd(
    pdf_path: String,
    signature_text: String,
    page_number: u32,
    x: f32,
    y: f32,
    output_path: Option<String>,
) -> Result<crate::signature::SignatureStampResult, String> {
    let in_p = Path::new(&pdf_path);
    let out_p = output_path
        .map(PathBuf::from)
        .unwrap_or_else(|| in_p.with_file_name(format!("signed-{}", in_p.file_name().unwrap().to_string_lossy())));
    crate::signature::stamp_pdf(in_p, &signature_text, page_number, x, y, &out_p)
}

#[tauri::command]
pub fn search_forms_cmd(
    query: Option<String>,
    state: Option<String>,
    task: Option<String>,
) -> Result<Vec<crate::forms::library::FormSearchResult>, String> {
    let q = query.unwrap_or_default();
    crate::forms::library::search_forms(&q, state.as_deref(), task.as_deref(), None)
}

#[tauri::command]
pub fn recommend_forms_cmd(
    case_id: String,
) -> Result<crate::forms::library::CaseRecommendations, String> {
    let dir = case_dir_from_id(&case_id)?;
    crate::forms::library::recommend_forms_for_case(&dir, None)
}

#[tauri::command]
pub fn fetch_form_cmd(
    form_id: String,
    allow_network: Option<bool>,
) -> Result<crate::forms::library::FetchFormResult, String> {
    crate::forms::library::fetch_form(&form_id, allow_network.unwrap_or(true), None, None)
}

#[tauri::command]
pub fn attach_form_cmd(
    case_id: String,
    form_id: String,
    as_template: Option<bool>,
) -> Result<crate::forms::library::AttachFormResult, String> {
    let dir = case_dir_from_id(&case_id)?;
    crate::forms::library::attach_form_to_case(&form_id, &dir, None, as_template.unwrap_or(false))
}
