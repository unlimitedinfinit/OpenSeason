// ... existing imports ...
use crate::bundle; // Need to add to lib.rs mod

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

// ... existing get_salt, unlock_vault ... (keeping these, just adding below)

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

