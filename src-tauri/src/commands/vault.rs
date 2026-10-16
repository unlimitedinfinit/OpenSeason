use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

use crate::bundle;
use crate::crypto::{self, AppState};
use crate::db::HuntDatabase;
use crate::pdf;
use crate::usaspending::{self, AwardSummary};
use super::helpers::{
    get_vault_root, hunt_db_path, resolve_hunt_dir, sealed_guard, EventEntry, HuntMetadata,
    PartyEntry, SectionEntry,
};

#[tauri::command]
pub fn save_disclosure_cmd(
    app: AppHandle,
    hunt_id: String,
    target: String,
    _count: usize,
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
                let clean_title = r.1.replace('[', "\\[").replace(']', "\\]");
                let clean_desc = r.2.replace('[', "\\[").replace(']', "\\]");
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
                let clean_path = r.1.replace('[', "\\[").replace(']', "\\]");
                let clean_desc = r.0.replace('[', "\\[").replace(']', "\\]");
                let clean_hash = r.2.replace('[', "\\[").replace(']', "\\]");
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
                let clean_content = r.1.replace('[', "\\[").replace(']', "\\]");
                complaint_markup.push_str(&format!("== {}\n{}\n\n", title, clean_content));
            }
        }
    }

    let pdf_bytes = pdf::compile_report(&target, value, &timeline_markup, &evidence_markup, &complaint_markup)?;

    // 1. Save to Vault (Archive)
    let vault_path = hunt_dir.join("disclosure_statement.pdf");
    fs::write(&vault_path, &pdf_bytes).map_err(|e| e.to_string())?;

    // 2. Save to User Downloads (User Request)
    let download_dir = app.path().download_dir().map_err(|e| e.to_string())?;

    let stem = crate::sandbox::sanitize_download_stem(&format!(
        "Disclosure_{}",
        target.replace(' ', "_")
    ))?;
    let desired = download_dir.join(format!("{}.pdf", stem));
    let app_root = app
        .path()
        .app_local_data_dir()
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
    tauri::async_runtime::spawn_blocking(move || usaspending::check_target(&name))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn export_hunt_cmd(app: AppHandle, hunt_id: String, target_path: String) -> Result<String, String> {
    let hunt_path = resolve_hunt_dir(&app, &hunt_id)?;

    if !hunt_path.exists() {
        return Err("Hunt not found".to_string());
    }

    let output_path = if target_path == "DOWNLOADS" {
        let download_dir = app.path().download_dir().map_err(|e| e.to_string())?;

        // Get hunt name from database for filename
        let db_path = hunt_path.join("metadata.db");
        let mut name = hunt_id.clone();

        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let stmt = conn.prepare("SELECT name FROM info LIMIT 1").ok();
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

    let app_root = app
        .path()
        .app_local_data_dir()
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
    let input_path = PathBuf::from(&osb_path);

    if !input_path.exists() {
        return Err("OSB file not found".to_string());
    }

    bundle::import_hunt(&input_path, &vaults_root)
}

#[tauri::command]
pub fn get_salt(app: AppHandle) -> Result<String, String> {
    let root = app
        .path()
        .app_local_data_dir()
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
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let root = app
        .path()
        .app_local_data_dir()
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

            let db_path = entry.path().join("metadata.db");
            if db_path.exists() {
                if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                    let stmt = conn.prepare("SELECT name FROM info LIMIT 1").ok();
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
                created: "Unknown".to_string(),
            });
        }
    }
    Ok(hunts)
}

#[tauri::command]
pub fn create_new_hunt(
    app: AppHandle,
    name: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    use uuid::Uuid;
    if state.get_key().is_none() {
        return Err("Vault Locked".to_string());
    }

    let uuid = Uuid::new_v4();
    let vaults_root = get_vault_root(&app)?;
    let hunt_dir = crate::sandbox::resolve_id_under_root(&vaults_root, &uuid.to_string())?;

    fs::create_dir_all(hunt_dir.join("evidence"))
        .map_err(|e| format!("Failed to create dir: {}", e))?;

    let db_path = hunt_dir.join("metadata.db");
    let db = HuntDatabase::open(&db_path).map_err(|e| e.to_string())?;

    let created_at = chrono::Utc::now().to_rfc3339();
    db.conn
        .execute(
            "INSERT INTO info (name, created_at, status, mode) VALUES (?1, ?2, 'Draft', 'confidential')",
            [&name, &created_at],
        )
        .map_err(|e| e.to_string())?;

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

    conn.execute("UPDATE info SET name = ?1", rusqlite::params![name])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_hunt(
    app: AppHandle,
    hunt_id: String,
    password: Option<String>,
) -> Result<(), String> {
    let vault_path = get_vault_root(&app)?;
    let root = app
        .path()
        .app_local_data_dir()
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

#[tauri::command]
pub fn get_hunt_timeline(app: AppHandle, hunt_id: String) -> Result<Vec<EventEntry>, String> {
    let db_path = hunt_db_path(&app, &hunt_id)?;
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, title, description, event_date, event_type FROM events ORDER BY event_date ASC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(EventEntry {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                event_date: row.get(3)?,
                event_type: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

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
    )
    .map_err(|e| e.to_string())?;
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

    let mut stmt = conn
        .prepare("SELECT id, name, role, email, phone, notes FROM parties ORDER BY name ASC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(PartyEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                role: row.get(2)?,
                email: row.get(3)?,
                phone: row.get(4)?,
                notes: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

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
    )
    .map_err(|e| e.to_string())?;
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

    let mut stmt = conn
        .prepare("SELECT section_id, content FROM complaint_sections")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(SectionEntry {
                section_id: row.get(0)?,
                content: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;

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
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn purge_vault_cache(
    app: AppHandle,
    password: Option<String>,
    include_sealed: Option<bool>,
) -> Result<String, String> {
    let root = app
        .path()
        .app_local_data_dir()
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
