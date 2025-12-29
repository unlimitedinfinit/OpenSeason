mod crypto;
mod commands;
mod db;
mod bundle;

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
            commands::create_hunt,
            commands::export_hunt_cmd,
            commands::import_hunt_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
