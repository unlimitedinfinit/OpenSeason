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
            case_commands::stamp_pdf_signature_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
