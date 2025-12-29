use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use tauri::State;
use crate::crypto::{self, AppState};
use crate::bundle;
use crate::usaspending::{self, AwardSummary};
use crate::db::HuntDatabase;
use crate::pdf;

use tauri::{AppHandle, Manager}; // Added Manager for path access if needed, or just AppHandle methods in v2

// Helper for standard storage path
fn get_vault_root(app: &AppHandle) -> Result<PathBuf, String> {
    // Tauri v2: app.path().app_local_data_dir() typically resolves to:
    // Windows: C:\Users\User\AppData\Local\com.openseason.app
    // Linux: /home/user/.local/share/com.openseason.app
    // macOS: /Users/User/Library/Application Support/com.openseason.app
    let root = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    
    let vaults = root.join("vaults");
    fs::create_dir_all(&vaults).map_err(|e| e.to_string())?;
    Ok(vaults)
}

#[tauri::command]
pub fn save_disclosure_cmd(
    app: AppHandle,
    hunt_id: String, 
    target: String, 
    count: usize, 
    value: f64
) -> Result<String, String> {
    let pdf_bytes = pdf::compile_report(&target, count, value)?;
    
    let vaults_root = get_vault_root(&app)?;
    let hunt_dir = vaults_root.join(&hunt_id);

    if !hunt_dir.exists() {
        return Err("Hunt not found".to_string());
    }

    let output_path = hunt_dir.join("disclosure_statement.pdf");
    fs::write(&output_path, pdf_bytes).map_err(|e| e.to_string())?;
    
    Ok(output_path.to_string_lossy().into_owned())
}


#[tauri::command]
pub async fn verify_target_cmd(name: String) -> Result<Vec<AwardSummary>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        usaspending::check_target(&name)
    }).await.map_err(|e| e.to_string())?
}


#[tauri::command]
pub fn export_hunt_cmd(app: AppHandle, hunt_id: String, target_path: String) -> Result<(), String> {
    let vaults_root = get_vault_root(&app)?;
    let hunt_path = vaults_root.join(&hunt_id);
    let output_path = PathBuf::from(&target_path);

    if !hunt_path.exists() {
        return Err("Hunt not found".to_string());
    }

    bundle::export_hunt(&hunt_path, &output_path)
}

#[tauri::command]
pub fn import_hunt_cmd(app: AppHandle, osb_path: String) -> Result<String, String> {
    let vaults_root = get_vault_root(&app)?;
    // Ensure root exists (handled by get_vault_root)

    let input_path = PathBuf::from(&osb_path);
    
    if !input_path.exists() {
        return Err("OSB file not found".to_string());
    }

    bundle::import_hunt(&input_path, &vaults_root)
}

#[derive(Serialize, Deserialize)]
pub struct HuntMetadata {
    id: String,
    name: String,
    created: String,
}

// --- Auth / Key Management ---

// --- Auth / Key Management ---

#[tauri::command]
pub fn get_salt(app: AppHandle) -> Result<String, String> {
    let root = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let path = root.join("master_salt.bin"); 

    if path.exists() {
        fs::read_to_string(&path).map_err(|e| e.to_string())
    } else {
        let salt = crypto::generate_salt();
        fs::write(&path, &salt).map_err(|e| e.to_string())?;
        Ok(salt)
    }
}

#[tauri::command]
pub fn unlock_vault(
    password: String, 
    salt: String,
    state: State<'_, AppState>
) -> Result<bool, String> {
    let session_key = crypto::derive_key(&password, &salt)?;
    state.set_key(session_key);
    Ok(true)
}

#[tauri::command]
pub fn lock_vault(state: State<'_, AppState>) -> Result<(), String> {
    state.clear_key();
    Ok(())
}

#[tauri::command]
pub fn is_locked(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.get_key().is_none())
}

#[tauri::command]
pub fn list_hunts(app: AppHandle) -> Result<Vec<HuntMetadata>, String> {
    let vaults_root = get_vault_root(&app)?;
    
    if !vaults_root.exists() {
        return Ok(Vec::new());
    }

    let mut hunts = Vec::new();
    let entries = fs::read_dir(vaults_root).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().is_dir() {
            let id = entry.file_name().into_string().unwrap_or_default();
            let mut name = id.clone();
            
            // Try to read name from metadata.db if possible, or just use ID for now.
            // For MVP + speed, we'll try to look for a specific simple metadata file if DB is too heavy, 
            // but prompt asked for metadata.db. 
            // Let's rely on just ID for now or basic DB check if we had a lightweight way.
            // Actually, let's try to query the DB for the name.
            let db_path = entry.path().join("metadata.db");
            if db_path.exists() {
                 if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                     let mut stmt = conn.prepare("SELECT name FROM info LIMIT 1").ok();
                     if let Some(mut s) = stmt {
                         if let Ok(n) = s.query_row([], |row| row.get(0)) {
                             name = n;
                         }
                     }
                 }
            }

            hunts.push(HuntMetadata {
                id,
                name,
                created: "Unknown".to_string(), // Placeholder or read from DB
            });
        }
    }
    Ok(hunts)
}

#[tauri::command]
pub fn create_new_hunt(app: AppHandle, name: String, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    use uuid::Uuid;
     // Ensure unlocked
    if state.get_key().is_none() {
        return Err("Vault Locked".to_string());
    }

    let uuid = Uuid::new_v4();
    let vaults_root = get_vault_root(&app)?;
    let hunt_dir = vaults_root.join(uuid.to_string());

    fs::create_dir_all(&hunt_dir.join("evidence"))
        .map_err(|e| format!("Failed to create dir: {}", e))?;

    // Create and init metadata.db
    let db_path = hunt_dir.join("metadata.db");
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS info (name TEXT, created_at TEXT, status TEXT)", 
        []
    ).map_err(|e| e.to_string())?;
    
    let created_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO info (name, created_at, status) VALUES (?1, ?2, 'Draft')",
        [&name, &created_at]
    ).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "id": uuid.to_string(),
        "name": name,
        "created_at": created_at
    }))
}

#[tauri::command]
pub fn update_hunt(app: AppHandle, hunt_id: String, name: String) -> Result<(), String> {
    let vaults_root = get_vault_root(&app)?;
    let hunt_dir = vaults_root.join(&hunt_id);
    let db_path = hunt_dir.join("metadata.db");
    
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
    conn.execute("UPDATE info SET name = ?1", [&name]).map_err(|e| e.to_string())?;
    Ok(())
}

// Deprecated or kept for compatibility if needed, but create_new_hunt supersedes it.
// We'll remove the old create_hunt to avoid confusion.

