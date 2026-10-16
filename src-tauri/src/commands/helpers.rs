use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

pub(crate) fn get_vault_root(app: &AppHandle) -> Result<PathBuf, String> {
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;

    let vaults = root.join("vaults");
    fs::create_dir_all(&vaults).map_err(|e| e.to_string())?;
    Ok(vaults)
}

pub(crate) fn resolve_hunt_dir(app: &AppHandle, hunt_id: &str) -> Result<PathBuf, String> {
    let vaults = get_vault_root(app)?;
    crate::sandbox::resolve_id_under_root(&vaults, hunt_id)
}

pub(crate) fn hunt_db_path(app: &AppHandle, hunt_id: &str) -> Result<PathBuf, String> {
    Ok(resolve_hunt_dir(app, hunt_id)?.join("metadata.db"))
}

pub(crate) fn sealed_guard(
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

#[derive(Serialize, Deserialize)]
pub struct HuntMetadata {
    pub id: String,
    pub name: String,
    pub created: String,
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

#[derive(Serialize, Deserialize)]
pub struct EvidenceEntry {
    pub id: i64,
    pub description: String,
    pub file_path: String,
    pub sha256_hash: Option<String>,
    pub created_at: String,
}
