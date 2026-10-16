use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

use crate::crypto::{self, AppState};
use super::helpers::{hunt_db_path, resolve_hunt_dir, sealed_guard, EvidenceEntry};

#[tauri::command]
pub fn get_hunt_evidence(app: AppHandle, hunt_id: String) -> Result<Vec<EvidenceEntry>, String> {
    let db_path = hunt_db_path(&app, &hunt_id)?;
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, description, file_path, sha256_hash, created_at FROM evidence ORDER BY created_at ASC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(EvidenceEntry {
                id: row.get(0)?,
                description: row.get(1)?,
                file_path: row.get(2)?,
                sha256_hash: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

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
    let mut stmt = conn
        .prepare("SELECT sha256_hash FROM evidence WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    let hash_opt: Option<String> = stmt
        .query_row(rusqlite::params![evidence_id], |row| row.get(0))
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
