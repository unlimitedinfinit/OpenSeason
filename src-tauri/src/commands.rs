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

fn resolve_hunt_dir(app: &AppHandle, hunt_id: &str) -> Result<PathBuf, String> {
    let vaults = get_vault_root(app)?;
    crate::sandbox::resolve_id_under_root(&vaults, hunt_id)
}

fn hunt_db_path(app: &AppHandle, hunt_id: &str) -> Result<PathBuf, String> {
    Ok(resolve_hunt_dir(app, hunt_id)?.join("metadata.db"))
}

fn sealed_guard(
    app: &AppHandle,
    hunt_id: &str,
    password: Option<&str>,
) -> Result<PathBuf, String> {
    let hunt_dir = resolve_hunt_dir(app, hunt_id)?;
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let salt = fs::read_to_string(root.join("master_salt.bin")).ok();
    let verifier = fs::read(root.join("master_verifier.bin")).ok();
    crate::sandbox::require_password_if_sealed(
        &hunt_dir,
        password,
        salt.as_deref(),
        verifier.as_deref(),
    )?;
    Ok(hunt_dir)
}

#[tauri::command]
pub fn save_disclosure_cmd(
    app: AppHandle,
    hunt_id: String, 
    target: String, 
    count: usize, // Ignored, kept for compatibility with Svelte invokes
    value: f64,
    password: Option<String>,
) -> Result<String, String> {
    let hunt_dir = sealed_guard(&app, &hunt_id, password.as_deref())?;
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
    
    let stem = crate::sandbox::sanitize_download_stem(&format!(
        "Disclosure_{}",
        target.replace(' ', "_")
    ))?;
    let desired = download_dir.join(format!("{}.pdf", stem));
    let app_root = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let (mut file, output_path) = crate::sandbox::prepare_export_file(
        &desired,
        &crate::case_profile::default_cases_root(),
        &app_root,
    )?;
    use std::io::Write;
    file.write_all(&pdf_bytes).map_err(|e| e.to_string())?;
    
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
    let hunt_path = resolve_hunt_dir(&app, &hunt_id)?;

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
        
        let sanitized_name = crate::sandbox::sanitize_download_stem(&name)?;
        download_dir.join(format!("{}.osb", sanitized_name))
    } else {
        PathBuf::from(&target_path)
    };

    let app_root = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let (file, output_path) = crate::sandbox::prepare_export_file(
        &output_path,
        &crate::case_profile::default_cases_root(),
        &app_root,
    )?;
    bundle::export_hunt_to_writer(&hunt_path, file).map_err(|e| e.to_string())?;
    
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
    app: AppHandle,
    password: String,
    state: State<'_, AppState>
) -> Result<bool, String> {
    let root = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let salt_path = root.join("master_salt.bin");
    if !salt_path.exists() {
        let salt = crypto::generate_salt();
        fs::write(&salt_path, &salt).map_err(|e| e.to_string())?;
    }
    let salt = fs::read_to_string(&salt_path).map_err(|e| e.to_string())?;
    let verifier_path = root.join("master_verifier.bin");
    let vaults_dir = root.join("vaults");
    let session_key = crypto::unlock_vault_at(&password, &salt, &verifier_path, &vaults_dir)?;
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
            if crate::sandbox::parse_record_id(&id).is_err() {
                continue;
            }
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
    let hunt_dir = crate::sandbox::resolve_id_under_root(&vaults_root, &uuid.to_string())?;

    fs::create_dir_all(&hunt_dir.join("evidence"))
        .map_err(|e| format!("Failed to create dir: {}", e))?;

    // Create and init metadata.db, including events/parties/evidence tables.
    let db_path = hunt_dir.join("metadata.db");
    let db = HuntDatabase::open(&db_path).map_err(|e| e.to_string())?;
    
    let created_at = chrono::Utc::now().to_rfc3339();
    db.conn.execute(
        "INSERT INTO info (name, created_at, status, mode) VALUES (?1, ?2, 'Draft', 'confidential')",
        [&name, &created_at]
    ).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "id": uuid.to_string(),
        "name": name,
        "created_at": created_at
    }))
}

#[tauri::command]
pub async fn update_hunt(
    app: AppHandle,
    hunt_id: String,
    name: String,
    password: Option<String>,
) -> Result<(), String> {
    let hunt_dir = sealed_guard(&app, &hunt_id, password.as_deref())?;
    let db_path = hunt_dir.join("metadata.db");

    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;
    
    conn.execute(
        "UPDATE info SET name = ?1",
        rusqlite::params![name],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_hunt(
    app: AppHandle,
    hunt_id: String,
    password: Option<String>,
) -> Result<(), String> {
    let vault_path = get_vault_root(&app)?;
    let root = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let salt = fs::read_to_string(root.join("master_salt.bin")).ok();
    let verifier = fs::read(root.join("master_verifier.bin")).ok();
    crate::sandbox::delete_hunt_checked(
        &vault_path,
        &hunt_id,
        password.as_deref(),
        salt.as_deref(),
        verifier.as_deref(),
    )?;
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
    let db_path = hunt_db_path(&app, &hunt_id)?;
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
    password: Option<String>,
) -> Result<(), String> {
    let hunt_dir = sealed_guard(&app, &hunt_id, password.as_deref())?;
    let db_path = hunt_dir.join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO events (title, description, event_date, event_type) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![title, description, event_date, event_type],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_hunt_event(
    app: AppHandle,
    hunt_id: String,
    event_id: i64,
    password: Option<String>,
) -> Result<(), String> {
    let hunt_dir = sealed_guard(&app, &hunt_id, password.as_deref())?;
    let db_path = hunt_dir.join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM events WHERE id = ?1", rusqlite::params![event_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_hunt_parties(app: AppHandle, hunt_id: String) -> Result<Vec<PartyEntry>, String> {
    let db_path = hunt_db_path(&app, &hunt_id)?;
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
    password: Option<String>,
) -> Result<(), String> {
    let hunt_dir = sealed_guard(&app, &hunt_id, password.as_deref())?;
    let db_path = hunt_dir.join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO parties (name, role, email, phone, notes) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![name, role, email, phone, notes],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_hunt_party(
    app: AppHandle,
    hunt_id: String,
    party_id: i64,
    password: Option<String>,
) -> Result<(), String> {
    let hunt_dir = sealed_guard(&app, &hunt_id, password.as_deref())?;
    let db_path = hunt_dir.join("metadata.db");
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM parties WHERE id = ?1", rusqlite::params![party_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_complaint_sections(app: AppHandle, hunt_id: String) -> Result<Vec<SectionEntry>, String> {
    let db_path = hunt_db_path(&app, &hunt_id)?;
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
    password: Option<String>,
) -> Result<(), String> {
    let hunt_dir = sealed_guard(&app, &hunt_id, password.as_deref())?;
    let db_path = hunt_dir.join("metadata.db");
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
    let db_path = hunt_db_path(&app, &hunt_id)?;
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

pub fn add_hunt_evidence_bytes_at(
    hunt_dir: &std::path::Path,
    key: &crypto::SessionKey,
    filename: &str,
    file_bytes: &[u8],
    description: &str,
    password: Option<&str>,
    salt: Option<&str>,
    verifier: Option<&[u8]>,
) -> Result<String, String> {
    crate::sandbox::require_password_if_sealed(hunt_dir, password, salt, verifier)?;

    let scrubbed_bytes = crypto::strip_metadata(file_bytes);
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&scrubbed_bytes);
    let hash_hex = format!("{:x}", hasher.finalize());

    let evidence_dir = hunt_dir.join("evidence");
    fs::create_dir_all(&evidence_dir).map_err(|e| e.to_string())?;
    let enc_dest_path = evidence_dir.join(format!("{}.enc", hash_hex));
    let db_path = hunt_dir.join("metadata.db");
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
    let already_logged: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM evidence WHERE sha256_hash = ?1",
            rusqlite::params![hash_hex],
            |row| row.get::<_, i64>(0),
        )
        .map(|n| n > 0)
        .unwrap_or(false);

    // Content-addressed filename. A second add of the same bytes must never
    // overwrite or delete the existing .enc (often the only sealed copy).
    if enc_dest_path.exists() || already_logged {
        return Ok("already_present".to_string());
    }

    let (encrypted_bytes, nonce) = crypto::encrypt_data(&scrubbed_bytes, key)?;
    match crate::sandbox::create_new_file(&enc_dest_path) {
        Ok(mut file) => {
            use std::io::Write;
            file.write_all(&encrypted_bytes).map_err(|e| e.to_string())?;
        }
        Err(err)
            if err.to_lowercase().contains("already") || err.to_lowercase().contains("exists") =>
        {
            return Ok("already_present".to_string());
        }
        Err(err) => return Err(err),
    }

    conn.execute(
        "INSERT INTO evidence (description, file_path, encrypted_key_nonce, sha256_hash) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![description, filename, nonce, hash_hex],
    )
    .map_err(|e| e.to_string())?;
    Ok("added".to_string())
}

#[tauri::command]
pub fn add_hunt_evidence(
    app: AppHandle,
    state: State<'_, AppState>,
    hunt_id: String,
    file_path: String,
    description: String,
    password: Option<String>,
) -> Result<String, String> {
    let key = state.get_key().ok_or("Vault Locked")?;
    sealed_guard(&app, &hunt_id, password.as_deref())?;
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err("Source file does not exist".to_string());
    }
    let file_bytes = fs::read(&path).map_err(|e| format!("Failed to read source file: {}", e))?;
    let original_filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    let hunt_dir = resolve_hunt_dir(&app, &hunt_id)?;
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let salt = fs::read_to_string(root.join("master_salt.bin")).ok();
    let verifier = fs::read(root.join("master_verifier.bin")).ok();
    add_hunt_evidence_bytes_at(
        &hunt_dir,
        &key,
        &original_filename,
        &file_bytes,
        &description,
        password.as_deref(),
        salt.as_deref(),
        verifier.as_deref(),
    )
}

#[tauri::command]
pub fn add_hunt_evidence_bytes(
    app: AppHandle,
    state: State<'_, AppState>,
    hunt_id: String,
    filename: String,
    file_bytes: Vec<u8>,
    description: String,
    password: Option<String>,
) -> Result<String, String> {
    let key = state.get_key().ok_or("Vault Locked")?;
    sealed_guard(&app, &hunt_id, password.as_deref())?;
    let hunt_dir = resolve_hunt_dir(&app, &hunt_id)?;
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let salt = fs::read_to_string(root.join("master_salt.bin")).ok();
    let verifier = fs::read(root.join("master_verifier.bin")).ok();
    add_hunt_evidence_bytes_at(
        &hunt_dir,
        &key,
        &filename,
        &file_bytes,
        &description,
        password.as_deref(),
        salt.as_deref(),
        verifier.as_deref(),
    )
}

pub fn delete_hunt_evidence_at(
    hunt_dir: &std::path::Path,
    evidence_id: i64,
    password: Option<&str>,
    salt: Option<&str>,
    verifier: Option<&[u8]>,
) -> Result<(), String> {
    crate::sandbox::require_password_if_sealed(hunt_dir, password, salt, verifier)?;
    let db_path = hunt_dir.join("metadata.db");
    delete_hunt_evidence_files(hunt_dir, &db_path, evidence_id)
}

fn delete_hunt_evidence_files(
    hunt_dir: &std::path::Path,
    db_path: &std::path::Path,
    evidence_id: i64,
) -> Result<(), String> {
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;

    // 1. Get SHA-256 hash from DB to delete the file
    let mut stmt = conn.prepare("SELECT sha256_hash FROM evidence WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    let hash_opt: Option<String> = stmt.query_row(rusqlite::params![evidence_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    // 2. Delete file if it exists
    if let Some(hash_hex) = hash_opt {
        if !crate::sandbox::is_hex_sha256(&hash_hex) {
            return Err("Evidence hash is not a SHA-256 hex value. File was not deleted.".to_string());
        }
        let enc_filename = format!("{}.enc", hash_hex);
        let enc_path = hunt_dir.join("evidence").join(&enc_filename);
        let evidence_root = hunt_dir.join("evidence");
        if enc_path.exists() {
            let canon = enc_path.canonicalize().map_err(|e| e.to_string())?;
            let evidence_canon = evidence_root.canonicalize().map_err(|e| e.to_string())?;
            if !canon.starts_with(&evidence_canon) {
                return Err("Evidence path escaped the hunt folder.".to_string());
            }
            let _ = fs::remove_file(canon);
        }
    }

    // 3. Delete from DB
    conn.execute("DELETE FROM evidence WHERE id = ?1", rusqlite::params![evidence_id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_hunt_evidence(
    app: AppHandle,
    hunt_id: String,
    evidence_id: i64,
    password: Option<String>,
) -> Result<(), String> {
    let hunt_dir = resolve_hunt_dir(&app, &hunt_id)?;
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let salt = fs::read_to_string(root.join("master_salt.bin")).ok();
    let verifier = fs::read(root.join("master_verifier.bin")).ok();
    delete_hunt_evidence_at(
        &hunt_dir,
        evidence_id,
        password.as_deref(),
        salt.as_deref(),
        verifier.as_deref(),
    )
}

#[tauri::command]
pub fn purge_vault_cache(
    app: AppHandle,
    password: Option<String>,
    include_sealed: Option<bool>,
) -> Result<String, String> {
    let root = app.path().app_local_data_dir()
        .map_err(|e| e.to_string())?;
    let vaults = root.join("vaults");
    let salt = fs::read_to_string(root.join("master_salt.bin")).ok();
    let verifier = fs::read(root.join("master_verifier.bin")).ok();
    let include_sealed = include_sealed.unwrap_or(false);
    let report = crate::sandbox::purge_vaults_checked(
        &vaults,
        password.as_deref(),
        salt.as_deref(),
        verifier.as_deref(),
        include_sealed,
    )?;
    Ok(format!(
        "Deleted {} hunt folder(s). Skipped {} sealed Confidential vault(s). Sealed vaults are the only encrypted copy and are not deleted unless you re-enter the vault password.",
        report.deleted, report.skipped_sealed
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::HuntDatabase;
    use crate::seal;
    use std::fs;

    fn sealed_hunt_with_evidence(password: &str) -> (PathBuf, i64, String, Vec<u8>, PathBuf) {
        let dir = std::env::temp_dir().join(format!("os-evdel-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(dir.join("evidence")).unwrap();
        let salt = crypto::generate_salt();
        let key = crypto::derive_key(password, &salt).unwrap();
        let verifier = crypto::create_password_verifier(&key).unwrap();
        seal::write_seal_marker(&dir, "00000000-0000-4000-8000-000000000099", "2024-01-01T00:00:00Z")
            .unwrap();
        let hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        fs::write(dir.join("evidence").join(format!("{}.enc", hash)), b"only-copy").unwrap();
        let db = HuntDatabase::open(dir.join("metadata.db")).unwrap();
        let id = db.insert_evidence("exhibit", "photo.jpg", &[0u8; 24], hash).unwrap();
        drop(db);
        let enc = dir.join("evidence").join(format!("{}.enc", hash));
        (dir, id, salt, verifier, enc)
    }

    #[test]
    fn sealed_evidence_delete_refused_without_password() {
        let (dir, id, salt, verifier, enc) = sealed_hunt_with_evidence("correct-ev-password");
        let err = delete_hunt_evidence_at(&dir, id, None, Some(&salt), Some(&verifier)).unwrap_err();
        assert!(
            err.to_lowercase().contains("password") || err.to_lowercase().contains("sealed"),
            "sealed evidence delete without password must be refused: {err}"
        );
        assert!(enc.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sealed_evidence_delete_refused_with_wrong_password() {
        let (dir, id, salt, verifier, enc) = sealed_hunt_with_evidence("correct-ev-password");
        let err = delete_hunt_evidence_at(
            &dir,
            id,
            Some("wrong-password"),
            Some(&salt),
            Some(&verifier),
        )
        .unwrap_err();
        assert!(
            err.to_lowercase().contains("wrong password"),
            "wrong password must not delete sealed evidence: {err}"
        );
        assert!(enc.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sealed_evidence_delete_allowed_with_correct_password() {
        let (dir, id, salt, verifier, enc) = sealed_hunt_with_evidence("correct-ev-password");
        delete_hunt_evidence_at(
            &dir,
            id,
            Some("correct-ev-password"),
            Some(&salt),
            Some(&verifier),
        )
        .unwrap();
        assert!(!enc.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    fn sealed_hunt_for_add(password: &str) -> (PathBuf, crypto::SessionKey, String, Vec<u8>) {
        let dir = std::env::temp_dir().join(format!("os-evadd-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(dir.join("evidence")).unwrap();
        let salt = crypto::generate_salt();
        let key = crypto::derive_key(password, &salt).unwrap();
        let verifier = crypto::create_password_verifier(&key).unwrap();
        seal::write_seal_marker(&dir, "00000000-0000-4000-8000-000000000088", "2024-01-01T00:00:00Z")
            .unwrap();
        HuntDatabase::open(dir.join("metadata.db")).unwrap();
        (dir, key, salt, verifier)
    }

    #[test]
    fn sealed_evidence_add_refused_without_password() {
        let (dir, key, salt, verifier) = sealed_hunt_for_add("correct-add-password");
        let err = add_hunt_evidence_bytes_at(
            &dir,
            &key,
            "photo.jpg",
            b"new exhibit bytes",
            "sealed add",
            None,
            Some(&salt),
            Some(&verifier),
        )
        .unwrap_err();
        assert!(
            err.to_lowercase().contains("password") || err.to_lowercase().contains("sealed"),
            "sealed evidence add without password must be refused: {err}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sealed_evidence_add_refused_with_wrong_password() {
        let (dir, key, salt, verifier) = sealed_hunt_for_add("correct-add-password");
        let err = add_hunt_evidence_bytes_at(
            &dir,
            &key,
            "photo.jpg",
            b"new exhibit bytes",
            "sealed add",
            Some("wrong-password"),
            Some(&salt),
            Some(&verifier),
        )
        .unwrap_err();
        assert!(
            err.to_lowercase().contains("wrong password"),
            "wrong password must not add sealed evidence: {err}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sealed_evidence_add_allowed_with_correct_password() {
        let (dir, key, salt, verifier) = sealed_hunt_for_add("correct-add-password");
        let status = add_hunt_evidence_bytes_at(
            &dir,
            &key,
            "photo.jpg",
            b"new exhibit bytes",
            "sealed add",
            Some("correct-add-password"),
            Some(&salt),
            Some(&verifier),
        )
        .unwrap();
        assert_eq!(status, "added");
        let entries: Vec<_> = fs::read_dir(dir.join("evidence"))
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(
            entries.iter().any(|e| e.path().extension().and_then(|s| s.to_str()) == Some("enc")),
            "correct password must write an encrypted exhibit"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    fn evidence_row_count(dir: &std::path::Path) -> i64 {
        let conn = rusqlite::Connection::open(dir.join("metadata.db")).unwrap();
        conn.query_row("SELECT COUNT(*) FROM evidence", [], |row| row.get(0))
            .unwrap()
    }

    fn only_enc_file(dir: &std::path::Path) -> std::path::PathBuf {
        let mut encs: Vec<_> = fs::read_dir(dir.join("evidence"))
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("enc"))
            .collect();
        assert_eq!(encs.len(), 1, "expected exactly one .enc, got {encs:?}");
        encs.remove(0)
    }

    #[test]
    fn sealed_evidence_duplicate_hash_does_not_overwrite() {
        let (dir, key, salt, verifier) = sealed_hunt_for_add("correct-add-password");
        let payload = b"identical sealed exhibit bytes";
        assert_eq!(
            add_hunt_evidence_bytes_at(
                &dir,
                &key,
                "photo.jpg",
                payload,
                "first add",
                Some("correct-add-password"),
                Some(&salt),
                Some(&verifier),
            )
            .unwrap(),
            "added"
        );
        let enc = only_enc_file(&dir);
        let before = fs::read(&enc).unwrap();
        assert!(!before.is_empty());
        assert_eq!(
            add_hunt_evidence_bytes_at(
                &dir,
                &key,
                "photo-copy.jpg",
                payload,
                "second add of the same bytes",
                Some("correct-add-password"),
                Some(&salt),
                Some(&verifier),
            )
            .unwrap(),
            "already_present"
        );
        assert_eq!(
            fs::read(&enc).unwrap(),
            before,
            "re-adding identical content must not rewrite the sealed .enc"
        );
        assert_eq!(only_enc_file(&dir), enc);
        assert_eq!(evidence_row_count(&dir), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unsealed_evidence_duplicate_hash_is_noop() {
        let dir = std::env::temp_dir().join(format!("os-evdup-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(dir.join("evidence")).unwrap();
        HuntDatabase::open(dir.join("metadata.db")).unwrap();
        let salt = crypto::generate_salt();
        let key = crypto::derive_key("unsealed-add-password", &salt).unwrap();
        let payload = b"same unsealed exhibit twice";
        assert_eq!(
            add_hunt_evidence_bytes_at(&dir, &key, "a.bin", payload, "one", None, None, None).unwrap(),
            "added"
        );
        let enc = only_enc_file(&dir);
        let before = fs::read(&enc).unwrap();
        assert_eq!(
            add_hunt_evidence_bytes_at(&dir, &key, "b.bin", payload, "two", None, None, None).unwrap(),
            "already_present"
        );
        assert_eq!(fs::read(&enc).unwrap(), before);
        assert_eq!(evidence_row_count(&dir), 1);
        let _ = fs::remove_dir_all(&dir);
    }
}

