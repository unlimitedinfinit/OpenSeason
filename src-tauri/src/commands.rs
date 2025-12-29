use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use tauri::State;
use crate::crypto::{self, AppState};
use crate::bundle;
use crate::usaspending::{self, AwardSummary};
use crate::db::HuntDatabase;
use crate::pdf;

#[tauri::command]
pub fn save_disclosure_cmd(hunt_id: String, target: String, count: usize, value: f64) -> Result<String, String> {
    let pdf_bytes = pdf::compile_report(&target, count, value)?;
    
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "No Home".to_string())?;
    
    let hunt_dir = PathBuf::from(&home).join(".open-season").join("hunts").join(&hunt_id);
    if !hunt_dir.exists() {
        return Err("Hunt not found".to_string());
    }

    let output_path = hunt_dir.join("disclosure_statement.pdf");
    fs::write(&output_path, pdf_bytes).map_err(|e| e.to_string())?;
    
    Ok(output_path.to_string_lossy().into_owned())
}


#[tauri::command]
pub async fn verify_target_cmd(name: String) -> Result<Vec<AwardSummary>, String> {
    // Perform blocking IO on a blocking thread to avoid freezing async runtime if using async, 
    // but here we used blocking reqwest. Ideally use async reqwest or spawn_blocking.
    // Since we are in an async command, we should use spawn_blocking.
    
    tauri::async_runtime::spawn_blocking(move || {
        usaspending::check_target(&name)
    }).await.map_err(|e| e.to_string())?
}


#[tauri::command]
pub fn export_hunt_cmd(hunt_id: String, target_path: String) -> Result<(), String> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "No Home".to_string())?;
    
    let hunt_path = PathBuf::from(&home).join(".open-season").join("hunts").join(&hunt_id);
    let output_path = PathBuf::from(&target_path);

    if !hunt_path.exists() {
        return Err("Hunt not found".to_string());
    }

    bundle::export_hunt(&hunt_path, &output_path)
}

#[tauri::command]
pub fn import_hunt_cmd(osb_path: String) -> Result<String, String> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "No Home".to_string())?;
    
    let vaults_root = PathBuf::from(&home).join(".open-season").join("hunts");
    fs::create_dir_all(&vaults_root).map_err(|e| e.to_string())?; // Ensure root exists

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

#[tauri::command]
pub fn get_salt() -> Result<String, String> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Could not find Home directory".to_string())?;
    
    let path = PathBuf::from(home).join(".open-season").join("chk.dat"); 

    if path.exists() {
        fs::read_to_string(&path).map_err(|e| e.to_string())
    } else {
        let salt = crypto::generate_salt();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
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
pub fn list_hunts() -> Result<Vec<HuntMetadata>, String> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "No Home".to_string())?;
    let hunts_dir = PathBuf::from(home).join(".open-season").join("hunts");
    
    if !hunts_dir.exists() {
        return Ok(Vec::new());
    }

    let mut hunts = Vec::new();
    let entries = fs::read_dir(hunts_dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().is_dir() {
            let name = entry.file_name().into_string().unwrap_or_default();
            // In a real app, read metadata.json for pretty name.
            // For MVP, directory name is the ID/Name.
            hunts.push(HuntMetadata {
                id: name.clone(),
                name,
                created: "Unknown".to_string(), // Placeholder
            });
        }
    }
    Ok(hunts)
}

#[tauri::command]
pub fn create_hunt(name: String, state: State<'_, AppState>) -> Result<String, String> {
    // Ensure unlocked
    if state.get_key().is_none() {
        return Err("Vault Locked".to_string());
    }

    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "No Home".to_string())?;
    
    // Sanitize name for path safety
    let safe_name = name.replace(|c: char| !c.is_alphanumeric() && c != '-', "_");
    let hunt_dir = PathBuf::from(home).join(".open-season").join("hunts").join(&safe_name);

    if hunt_dir.exists() {
        return Err("Hunt already exists".to_string());
    }

    fs::create_dir_all(&hunt_dir).map_err(|e| e.to_string())?;

    // Initialize DB
    let db_path = hunt_dir.join("hunt.db");
    let _db = HuntDatabase::open(&db_path).map_err(|e| e.to_string())?;

    Ok(safe_name)
}

