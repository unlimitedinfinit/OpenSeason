pub mod crypto;
pub mod commands;
pub mod case_commands;
pub mod case_profile;
pub mod sandbox;
pub mod court_rules;
pub mod documents;
pub mod seal;
pub mod sync;
pub mod db;
pub mod bundle;
pub mod usaspending;
pub mod pdf;
pub mod holidays;
pub mod rule_pack;
pub mod deadlines;
pub mod check;
pub mod court_md;
pub mod agent;
pub mod mcp;
pub mod guided;
pub mod exhibits;
pub mod calendar;
pub mod gaps;
pub mod help;
pub mod seed;
pub mod packet;
pub mod forms;
pub mod guide;
pub mod help_links;
pub mod service;
pub mod pdf_tools;
pub mod signature;
pub mod legal_os;
pub mod intake;
pub mod portfolio;
pub mod pleading;
pub mod folder_import;
pub mod capabilities;
pub mod record;
pub mod accountability;
pub mod ada;
pub mod order;
pub mod court_access;
pub mod workflow;
pub mod consultation;
pub mod qui_tam;

use crypto::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_salt,
            commands::unlock_vault,
            commands::lock_vault,
            commands::is_locked,
            commands::list_hunts,
            commands::create_new_hunt,
            commands::update_hunt,
            commands::delete_hunt,
            commands::export_hunt_cmd,
            commands::import_hunt_cmd,
            commands::verify_target_cmd,
            commands::save_disclosure_cmd,
            commands::get_hunt_timeline,
            commands::add_hunt_event,
            commands::delete_hunt_event,
            commands::get_hunt_parties,
            commands::add_hunt_party,
            commands::delete_hunt_party,
            commands::get_complaint_sections,
            commands::save_complaint_section,
            commands::get_hunt_evidence,
            commands::add_hunt_evidence,
            commands::add_hunt_evidence_bytes,
            commands::delete_hunt_evidence,
            commands::purge_vault_cache,
            case_commands::get_cases_root,
            case_commands::list_cases,
            case_commands::list_workspace_cases,
            case_commands::create_case,
            case_commands::get_case,
            case_commands::save_case,
            case_commands::validate_case_cmd,
            case_commands::export_notice_of_appeal_cmd,
            case_commands::export_pleading_cmd,
            case_commands::open_case_folder,
            case_commands::list_case_artifacts,
            case_commands::add_case_evidence,
            case_commands::convert_case_to_confidential,
            case_commands::sync_case_cmd,
            case_commands::list_rule_packs,
            case_commands::import_markdown_cmd,
            case_commands::check_case_cmd,
            case_commands::deadline_cmd,
            case_commands::guided_sections_cmd,
            case_commands::guided_fill_cmd,
            case_commands::exhibit_index_cmd,
            case_commands::link_fact_cmd,
            case_commands::set_fact_cite_cmd,
            case_commands::set_authority_cmd,
            case_commands::set_signoff_cmd,
            case_commands::export_packet_cmd,
            case_commands::export_ics_cmd,
            case_commands::seed_sample_cmd,
            case_commands::gap_add_cmd,
            case_commands::gap_export_cmd,
            case_commands::field_help_cmd,
            case_commands::playbook_cmd,
            case_commands::list_form_packs_cmd,
            case_commands::install_form_pack_cmd,
            case_commands::remove_form_pack_cmd,
            case_commands::verify_form_pack_cmd,
            case_commands::guide_questions_cmd,
            case_commands::guide_find_cmd,
            case_commands::help_links_cmd,
            case_commands::cloud_login_cmd,
            case_commands::cloud_set_session_cmd,
            case_commands::cloud_status_cmd,
            case_commands::cloud_logout_cmd,
            case_commands::cloud_pull_cmd,
            case_commands::cloud_push_cmd,
            case_commands::cloud_get_case_sync_meta_cmd,
            case_commands::preflight_audit_cmd,
            case_commands::export_exhibit_packet_cmd,
            case_commands::inspect_court_form_cmd,
            case_commands::fill_court_form_cmd,
            case_commands::check_transcripts_cmd,
            case_commands::list_case_filings_cmd,
            case_commands::save_case_filing_cmd,
            case_commands::delete_case_filing_cmd,
            case_commands::export_case_filing_cmd,
            case_commands::generate_certificate_of_service_cmd,
            case_commands::generate_proposed_order_cmd,
            case_commands::pdf_merge_cmd,
            case_commands::pdf_split_cmd,
            case_commands::pdf_rotate_cmd,
            case_commands::pdf_redact_cmd,
            case_commands::get_signature_profile_cmd,
            case_commands::save_signature_profile_cmd,
            case_commands::stamp_pdf_signature_cmd,
            portfolio::case_context_cmd,
            portfolio::intake_status_cmd,
            portfolio::intake_answer_cmd,
            portfolio::intake_scan_cmd,
            portfolio::case_next_steps_cmd,
            portfolio::check_venue_competence_cmd,
            portfolio::escalate_case_cmd,
            portfolio::portfolio_status_cmd,
            portfolio::get_global_filer_cmd,
            portfolio::save_global_filer_cmd,
            portfolio::pdf_extract_text_cmd,
            portfolio::scoped_sections_cmd,
            portfolio::scoped_fill_cmd,
            case_commands::import_folder_cmd,
            case_commands::agent_manifest_cmd,
            case_commands::preview_markdown_cmd,
            case_commands::export_markdown_draft_cmd,
            case_commands::search_forms_cmd,
            case_commands::recommend_forms_cmd,
            case_commands::fetch_form_cmd,
            case_commands::attach_form_cmd,
            case_commands::record_inventory_cmd,
            case_commands::record_docket_match_cmd,
            case_commands::record_missing_cmd,
            case_commands::record_request_form_cmd,
            case_commands::accountability_audit_cmd,
            case_commands::accountability_routes_cmd,
            case_commands::accountability_resources_cmd,
            case_commands::accountability_packet_cmd,
            case_commands::ada_petition_cmd,
            case_commands::order_decode_cmd,
            case_commands::order_ingest_cmd,
            case_commands::timeline_audit_mail_cmd,
            case_commands::case_bootstrap_cmd,
            case_commands::court_access_directory_cmd,
            case_commands::court_access_profile_cmd,
            case_commands::record_retrieval_plan_cmd,
            case_commands::record_retrieval_execute_cmd,
            case_commands::workflow_plan_cmd,
            case_commands::workflow_status_cmd,
            case_commands::workflow_advance_cmd,
            case_commands::stress_test_issue_cmd,
            case_commands::consultation_dossier_cmd,
            case_commands::extension_evaluate_cmd,
            case_commands::extension_suggest_cmd,
            case_commands::quitam_verify_entity_cmd,
            case_commands::quitam_audit_9b_cmd,
            case_commands::quitam_ingest_observation_cmd,
            case_commands::quitam_records_pack_cmd,
            case_commands::quitam_export_disclosure_cmd,
            case_commands::quitam_format_portal_cmd,
            case_commands::quitam_format_inquiry_cmd,
            case_commands::doctor_diagnostic_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
