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
    count: usize, // Ignored, kept for compatibility with Svelte invokes
    value: f64
) -> Result<String, String> {
    let vaults_root = get_vault_root(&app)?;
    let hunt_dir = vaults_root.join(&hunt_id);
    if !hunt_dir.exists() {
        return Err("Hunt not found".to_string());
    }

    let db_path = hunt_dir.join("metadata.db");
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;

    // 1. Fetch and format Timeline Events
    let mut timeline_markup = String::new();
    if let Ok(mut stmt) = conn.prepare("SELECT title, description, event_date FROM events ORDER BY event_date ASC") {
        let rows = stmt.query_map([], |row| {
            let title: String = row.get(0)?;
            let desc: Option<String> = row.get(1)?;
            let date: String = row.get(2)?;
            Ok((date, title, desc.unwrap_or_default()))
        });
        if let Ok(r_iter) = rows {
            for r in r_iter.flatten() {
                let clean_title = r.1.replace("[", "\\[").replace("]", "\\]");
                let clean_desc = r.2.replace("[", "\\[").replace("]", "\\]");
                timeline_markup.push_str(&format!("- *{} - {}*: {}\n", r.0, clean_title, clean_desc));
            }
        }
    }

    // 2. Fetch and format Evidence Files
    let mut evidence_markup = String::new();
    if let Ok(mut stmt) = conn.prepare("SELECT description, file_path, sha256_hash FROM evidence ORDER BY created_at ASC") {
        let rows = stmt.query_map([], |row| {
            let desc: String = row.get(0)?;
            let path: String = row.get(1)?;
            let hash: Option<String> = row.get(2)?;
            Ok((desc, path, hash.unwrap_or_else(|| "N/A".to_string())))
        });
        if let Ok(r_iter) = rows {
            for r in r_iter.flatten() {
                let clean_path = r.1.replace("[", "\\[").replace("]", "\\]");
                let clean_desc = r.0.replace("[", "\\[").replace("]", "\\]");
                let clean_hash = r.2.replace("[", "\\[").replace("]", "\\]");
                evidence_markup.push_str(&format!("  [{}], [{}], [{}],\n", clean_path, clean_desc, clean_hash));
            }
        }
    }

    // 3. Fetch and format Complaint Sections
    let mut complaint_markup = String::new();
    if let Ok(mut stmt) = conn.prepare("SELECT section_id, content FROM complaint_sections") {
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let content: String = row.get(1)?;
            Ok((id, content))
        });
        if let Ok(r_iter) = rows {
            for r in r_iter.flatten() {
                let title = match r.0.as_str() {
                    "introduction" => "A. Introduction / Background",
                    "jurisdiction" => "B. Jurisdiction & Venue",
                    "parties" => "C. Parties Involved",
                    "facts" => "D. Statement of Facts",
                    "violations" => "E. Violations of the False Claims Act",
                    _ => "Other Details",
                };
                let clean_content = r.1.replace("[", "\\[").replace("]", "\\]");
                complaint_markup.push_str(&format!("== {}\n{}\n\n", title, clean_content));
            }
        }
    }

    let pdf_bytes = pdf::compile_report(&target, value, &timeline_markup, &evidence_markup, &complaint_markup)?;
    
    // 1. Save to Vault (Archive)
    let vault_path = hunt_dir.join("disclosure_statement.pdf");
    fs::write(&vault_path, &pdf_bytes).map_err(|e| e.to_string())?;

    // 2. Save to User Downloads (User Request)
    let download_dir = app.path().download_dir()
        .map_err(|e| e.to_string())?;
    
    let sanitized_target = target.replace(" ", "_").replace("/", "-");
    let filename = format!("Disclosure_{}.pdf", sanitized_target);
    let output_path = download_dir.join(&filename);
    
    fs::write(&output_path, &pdf_bytes).map_err(|e| e.to_string())?;
    
    Ok(output_path.to_string_lossy().into_owned())
}


#[tauri::command]
pub async fn verify_target_cmd(name: String) -> Result<Vec<AwardSummary>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        usaspending::check_target(&name)
    }).await.map_err(|e| e.to_string())?
}


#[tauri::command]
pub fn export_hunt_cmd(app: AppHandle, hunt_id: String, target_path: String) -> Result<String, String> {
    let vaults_root = get_vault_root(&app)?;
    let hunt_path = vaults_root.join(&hunt_id);

    if !hunt_path.exists() {
        return Err("Hunt not found".to_string());
    }

    let output_path = if target_path == "DOWNLOADS" {
        let download_dir = app.path().download_dir()
            .map_err(|e| e.to_string())?;
            
        // Get hunt name from database for filename
        let db_path = hunt_path.join("metadata.db");
        let mut name = hunt_id.clone();
        
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
             let mut stmt = conn.prepare("SELECT name FROM info LIMIT 1").ok();
             if let Some(mut s) = stmt {
                 if let Ok(n) = s.query_row([], |row| row.get(0)) {
                     name = n;
                 }
             }
        }
        
        let sanitized_name = name.replace(" ", "_").replace("/", "-");
        let filename = format!("{}.osb", sanitized_name);
        download_dir.join(filename)
    } else {
        PathBuf::from(&target_path)
    };

    bundle::export_hunt(&hunt_path, &output_path).map_err(|e| e.to_string())?;
    
    Ok(output_path.to_string_lossy().into_owned())
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
pub async fn update_hunt(app: AppHandle, hunt_id: String, name: String) -> Result<(), String> {
    let vault_path = get_vault_root(&app)?;
    let db_path = vault_path.join(&hunt_id).join("metadata.db");

    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;
    
    conn.execute(
        "UPDATE info SET name = ?1",
        rusqlite::params![name],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_hunt(app: AppHandle, hunt_id: String) -> Result<(), String> {
    let vault_path = get_vault_root(&app)?;
    let hunt_path = vault_path.join(&hunt_id);

    if hunt_path.exists() {
        std::fs::remove_dir_all(hunt_path).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}


#[derive(Serialize, Deserialize)]
pub struct EventEntry {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub event_date: String,
    pub event_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct PartyEntry {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SectionEntry {
    pub section_id: String,
    pub content: Option<String>,
}

#[tauri::command]
pub fn get_hunt_timeline(app: AppHandle, hunt_id: String) -> Result<Vec<EventEntry>, String> {
    let vault_path = get_vault_root(&app)?;
    let db_path = vault_path.join(&hunt_id).join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT id, title, description, event_date, event_type FROM events ORDER BY event_date ASC")
        .map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([], |row| {
        Ok(EventEntry {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            event_date: row.get(3)?,
            event_type: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut events = Vec::new();
    for r in rows {
        events.push(r.map_err(|e| e.to_string())?);
    }
    Ok(events)
}

#[tauri::command]
pub fn add_hunt_event(
    app: AppHandle,
    hunt_id: String,
    title: String,
    description: String,
    event_date: String,
    event_type: String,
) -> Result<(), String> {
    let vault_path = get_vault_root(&app)?;
    let db_path = vault_path.join(&hunt_id).join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO events (title, description, event_date, event_type) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![title, description, event_date, event_type],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_hunt_event(app: AppHandle, hunt_id: String, event_id: i64) -> Result<(), String> {
    let vault_path = get_vault_root(&app)?;
    let db_path = vault_path.join(&hunt_id).join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM events WHERE id = ?1", rusqlite::params![event_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_hunt_parties(app: AppHandle, hunt_id: String) -> Result<Vec<PartyEntry>, String> {
    let vault_path = get_vault_root(&app)?;
    let db_path = vault_path.join(&hunt_id).join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT id, name, role, email, phone, notes FROM parties ORDER BY name ASC")
        .map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([], |row| {
        Ok(PartyEntry {
            id: row.get(0)?,
            name: row.get(1)?,
            role: row.get(2)?,
            email: row.get(3)?,
            phone: row.get(4)?,
            notes: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut parties = Vec::new();
    for r in rows {
        parties.push(r.map_err(|e| e.to_string())?);
    }
    Ok(parties)
}

#[tauri::command]
pub fn add_hunt_party(
    app: AppHandle,
    hunt_id: String,
    name: String,
    role: String,
    email: String,
    phone: String,
    notes: String,
) -> Result<(), String> {
    let vault_path = get_vault_root(&app)?;
    let db_path = vault_path.join(&hunt_id).join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO parties (name, role, email, phone, notes) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![name, role, email, phone, notes],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_hunt_party(app: AppHandle, hunt_id: String, party_id: i64) -> Result<(), String> {
    let vault_path = get_vault_root(&app)?;
    let db_path = vault_path.join(&hunt_id).join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM parties WHERE id = ?1", rusqlite::params![party_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_complaint_sections(app: AppHandle, hunt_id: String) -> Result<Vec<SectionEntry>, String> {
    let vault_path = get_vault_root(&app)?;
    let db_path = vault_path.join(&hunt_id).join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT section_id, content FROM complaint_sections")
        .map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([], |row| {
        Ok(SectionEntry {
            section_id: row.get(0)?,
            content: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut sections = Vec::new();
    for r in rows {
        sections.push(r.map_err(|e| e.to_string())?);
    }
    Ok(sections)
}

#[tauri::command]
pub fn save_complaint_section(
    app: AppHandle,
    hunt_id: String,
    section_id: String,
    content: String,
) -> Result<(), String> {
    let vault_path = get_vault_root(&app)?;
    let db_path = vault_path.join(&hunt_id).join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO complaint_sections (section_id, content) VALUES (?1, ?2) \
         ON CONFLICT(section_id) DO UPDATE SET content = excluded.content, updated_at = CURRENT_TIMESTAMP",
        rusqlite::params![section_id, content],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct EvidenceEntry {
    pub id: i64,
    pub description: String,
    pub file_path: String,
    pub sha256_hash: Option<String>,
    pub created_at: String,
}

#[tauri::command]
pub fn get_hunt_evidence(app: AppHandle, hunt_id: String) -> Result<Vec<EvidenceEntry>, String> {
    let vault_path = get_vault_root(&app)?;
    let db_path = vault_path.join(&hunt_id).join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT id, description, file_path, sha256_hash, created_at FROM evidence ORDER BY created_at ASC")
        .map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([], |row| {
        Ok(EvidenceEntry {
            id: row.get(0)?,
            description: row.get(1)?,
            file_path: row.get(2)?,
            sha256_hash: row.get(3)?,
            created_at: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut evidence = Vec::new();
    for r in rows {
        evidence.push(r.map_err(|e| e.to_string())?);
    }
    Ok(evidence)
}

#[tauri::command]
pub fn add_hunt_evidence(
    app: AppHandle,
    state: State<'_, AppState>,
    hunt_id: String,
    file_path: String,
    description: String,
) -> Result<(), String> {
    // 1. Check unlocked
    let key = state.get_key().ok_or("Vault Locked")?;

    // 2. Read source file
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err("Source file does not exist".to_string());
    }
    let file_bytes = fs::read(&path).map_err(|e| format!("Failed to read source file: {}", e))?;

    // 3. Strip metadata (JPEG/PNG)
    let scrubbed_bytes = crypto::strip_metadata(&file_bytes);

    // 4. Compute SHA-256 hash
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&scrubbed_bytes);
    let hash_hex = format!("{:x}", hasher.finalize());

    // 5. Encrypt data
    let (encrypted_bytes, nonce) = crypto::encrypt_data(&scrubbed_bytes, &key)?;

    // 6. Write encrypted file to vault directory
    let vault_path = get_vault_root(&app)?;
    let hunt_dir = vault_path.join(&hunt_id);
    let evidence_dir = hunt_dir.join("evidence");
    fs::create_dir_all(&evidence_dir).map_err(|e| e.to_string())?;

    let enc_filename = format!("{}.enc", hash_hex);
    let enc_dest_path = evidence_dir.join(&enc_filename);
    fs::write(&enc_dest_path, &encrypted_bytes).map_err(|e| e.to_string())?;

    // 7. Add entry to SQLite database
    let db_path = hunt_dir.join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    let original_filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    conn.execute(
        "INSERT INTO evidence (description, file_path, encrypted_key_nonce, sha256_hash) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![description, original_filename, nonce, hash_hex],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn add_hunt_evidence_bytes(
    app: AppHandle,
    state: State<'_, AppState>,
    hunt_id: String,
    filename: String,
    file_bytes: Vec<u8>,
    description: String,
) -> Result<(), String> {
    // 1. Check unlocked
    let key = state.get_key().ok_or("Vault Locked")?;

    // 2. Strip metadata (JPEG/PNG)
    let scrubbed_bytes = crypto::strip_metadata(&file_bytes);

    // 3. Compute SHA-256 hash
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&scrubbed_bytes);
    let hash_hex = format!("{:x}", hasher.finalize());

    // 4. Encrypt data
    let (encrypted_bytes, nonce) = crypto::encrypt_data(&scrubbed_bytes, &key)?;

    // 5. Write encrypted file to vault directory
    let vault_path = get_vault_root(&app)?;
    let hunt_dir = vault_path.join(&hunt_id);
    let evidence_dir = hunt_dir.join("evidence");
    fs::create_dir_all(&evidence_dir).map_err(|e| e.to_string())?;

    let enc_filename = format!("{}.enc", hash_hex);
    let enc_dest_path = evidence_dir.join(&enc_filename);
    fs::write(&enc_dest_path, &encrypted_bytes).map_err(|e| e.to_string())?;

    // 6. Add entry to SQLite database
    let db_path = hunt_dir.join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO evidence (description, file_path, encrypted_key_nonce, sha256_hash) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![description, filename, nonce, hash_hex],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_hunt_evidence(app: AppHandle, hunt_id: String, evidence_id: i64) -> Result<(), String> {
    let vault_path = get_vault_root(&app)?;
    let hunt_dir = vault_path.join(&hunt_id);
    let db_path = hunt_dir.join("metadata.db");
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;

    // 1. Get SHA-256 hash from DB to delete the file
    let mut stmt = conn.prepare("SELECT sha256_hash FROM evidence WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    let hash_opt: Option<String> = stmt.query_row(rusqlite::params![evidence_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    // 2. Delete file if it exists
    if let Some(hash_hex) = hash_opt {
        let enc_filename = format!("{}.enc", hash_hex);
        let enc_path = hunt_dir.join("evidence").join(&enc_filename);
        if enc_path.exists() {
            let _ = fs::remove_file(enc_path);
        }
    }

    // 3. Delete from DB
    conn.execute("DELETE FROM evidence WHERE id = ?1", rusqlite::params![evidence_id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn purge_vault_cache(app: AppHandle) -> Result<(), String> {
    let root = app.path().app_local_data_dir()
        .map_err(|e| e.to_string())?;
    let vaults = root.join("vaults");
    if vaults.exists() {
        std::fs::remove_dir_all(&vaults).map_err(|e| e.to_string())?;
    }
    Ok(())
}

