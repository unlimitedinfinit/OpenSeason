pub mod crypto;
pub mod commands;
pub mod db;
pub mod bundle;
pub mod usaspending;
pub mod pdf;

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
            commands::purge_vault_cache
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
