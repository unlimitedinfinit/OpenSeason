use serde::{Deserialize, Serialize};

pub fn firebase_api_key() -> String {
    std::env::var("JUSTLEGAL_FIREBASE_API_KEY")
        .or_else(|_| std::env::var("FIREBASE_API_KEY"))
        .unwrap_or_default()
}

pub fn firebase_project_id() -> String {
    std::env::var("JUSTLEGAL_FIREBASE_PROJECT_ID")
        .unwrap_or_else(|_| "justlegalapp1".to_string())
}

pub fn firebase_storage_bucket() -> String {
    std::env::var("JUSTLEGAL_FIREBASE_STORAGE_BUCKET")
        .unwrap_or_else(|_| "justlegalapp1.firebasestorage.app".to_string())
}

/// Network-facing actions that must never receive a confidential case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncAction {
    Backup,
    Publish,
    PullAccount,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "kind", content = "message")]
pub enum SyncError {
    #[error("Confidential cases cannot be sent through the sync, backup, or publish layer. Use a deliberate local export if you need a copy.")]
    ConfidentialForbidden,
    #[error("Account sync is not implemented in this build.")]
    NotImplemented,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncRefusal {
    pub action: String,
    pub mode: String,
    pub reason: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSession {
    pub id_token: String,
    pub refresh_token: String,
    pub uid: String,
    pub email: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncFileRecord {
    pub doc_id: String,
    pub title: String,
    pub filename: String,
    pub subfolder: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub synced_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CaseSyncMetadata {
    pub cloud_matter_id: String,
    pub last_synced_at: String,
    pub files: Vec<SyncFileRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PullReport {
    pub ok: bool,
    pub cases_pulled: usize,
    pub cases_updated: usize,
    pub files_downloaded: usize,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PushReport {
    pub ok: bool,
    pub cloud_matter_id: String,
    pub files_uploaded: usize,
    pub message: String,
}
