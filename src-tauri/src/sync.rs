use std::fs;
use std::path::{Path, PathBuf};

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::case_profile::{self, CaseMode, CaseProfile, TimelineEvent};
use crate::seal;

/// Runtime / build-time env var for the Firebase Web API key. Never commit a real value.
pub const FIREBASE_API_KEY_ENV: &str = "OPENSEASON_FIREBASE_API_KEY";

/// Compile-time fallback from a gitignored `.env` (see `src-tauri/build.rs`). Absent in CI.
const COMPILE_TIME_FIREBASE_API_KEY: Option<&str> = option_env!("OPENSEASON_FIREBASE_API_KEY");

/// Public project id (not a secret). Used only after a key is resolved.
pub const FIREBASE_PROJECT_ID: &str = "justlegalapp1";
/// Public default bucket name (not a secret).
pub const FIREBASE_STORAGE_BUCKET: &str = "justlegalapp1.firebasestorage.app";

/// User-level config outside the repo (`%LOCALAPPDATA%/JustLegal/config.json` on Windows).
pub fn user_config_path() -> PathBuf {
    let base = dirs::data_local_dir()
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("JustLegal").join("config.json")
}

pub fn firebase_api_key_missing_message() -> String {
    format!(
        "Cloud sync is disabled: Firebase API key is not configured. Set the {FIREBASE_API_KEY_ENV} environment variable, add firebase_api_key to {}, or provide the key at build time via a gitignored .env (see .env.example). Offline features are unaffected.",
        user_config_path().display()
    )
}

/// First non-empty candidate wins: env, user config file, compile-time `option_env!`.
pub fn resolve_firebase_api_key(
    env_value: Option<&str>,
    config_value: Option<&str>,
    compile_time_value: Option<&str>,
) -> Option<String> {
    for candidate in [env_value, config_value, compile_time_value] {
        if let Some(raw) = candidate {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

pub fn load_firebase_api_key_from_config_path(path: &Path) -> Option<String> {
    let data = fs::read_to_string(path).ok()?;
    let value: Value = serde_json::from_str(&data).ok()?;
    let key = value.get("firebase_api_key").and_then(|v| v.as_str())?;
    let trimmed = key.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn env_firebase_api_key() -> Option<String> {
    std::env::var(FIREBASE_API_KEY_ENV)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Resolve the Firebase Web API key, or `None` if cloud sync must stay disabled.
pub fn firebase_api_key() -> Option<String> {
    let config = load_firebase_api_key_from_config_path(&user_config_path());
    resolve_firebase_api_key(
        env_firebase_api_key().as_deref(),
        config.as_deref(),
        COMPILE_TIME_FIREBASE_API_KEY,
    )
}

pub fn require_firebase_api_key() -> Result<String, String> {
    firebase_api_key().ok_or_else(firebase_api_key_missing_message)
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
    #[error("Cloud sync is disabled: Firebase API key is not configured. Set OPENSEASON_FIREBASE_API_KEY, add firebase_api_key to the JustLegal config file, or provide the key at build time via a gitignored .env. Offline features are unaffected.")]
    NotConfigured,
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

pub fn assert_sync_allowed(profile: &CaseProfile) -> Result<(), SyncError> {
    if profile.is_sync_forbidden() {
        Err(SyncError::ConfidentialForbidden)
    } else {
        Ok(())
    }
}

pub fn request_sync(profile: &CaseProfile, action: SyncAction) -> Result<(), SyncError> {
    assert_sync_allowed(profile)?;
    let _ = action;
    let _ = CaseMode::Standard;
    if firebase_api_key().is_none() {
        return Err(SyncError::NotConfigured);
    }
    Err(SyncError::NotImplemented)
}

pub fn request_sync_for_dir(case_dir: &Path, action: SyncAction) -> Result<(), SyncError> {
    if seal::is_dir_sealed(case_dir) {
        return Err(SyncError::ConfidentialForbidden);
    }
    let profile = crate::case_profile::load_case(case_dir)
        .map_err(|_| SyncError::NotImplemented)?;
    request_sync(&profile, action)
}

pub fn describe_sync_for_dir(case_dir: &Path, action: SyncAction) -> Result<SyncRefusal, String> {
    let mode = crate::case_profile::load_case(case_dir)
        .map(|p| p.mode.as_str().to_string())
        .unwrap_or_else(|_| {
            if seal::is_dir_sealed(case_dir) {
                "confidential".to_string()
            } else {
                "unknown".to_string()
            }
        });
    match request_sync_for_dir(case_dir, action) {
        Ok(()) => Ok(SyncRefusal {
            action: format!("{:?}", action).to_ascii_lowercase(),
            mode,
            reason: "ok".to_string(),
            kind: "ok".to_string(),
        }),
        Err(SyncError::ConfidentialForbidden) => Ok(SyncRefusal {
            action: format!("{:?}", action).to_ascii_lowercase(),
            mode,
            reason: SyncError::ConfidentialForbidden.to_string(),
            kind: "confidential_forbidden".to_string(),
        }),
        Err(SyncError::NotImplemented) => Ok(SyncRefusal {
            action: format!("{:?}", action).to_ascii_lowercase(),
            mode,
            reason: SyncError::NotImplemented.to_string(),
            kind: "not_implemented".to_string(),
        }),
        Err(SyncError::NotConfigured) => Ok(SyncRefusal {
            action: format!("{:?}", action).to_ascii_lowercase(),
            mode,
            reason: SyncError::NotConfigured.to_string(),
            kind: "not_configured".to_string(),
        }),
    }
}

pub fn describe_sync_error(profile: &CaseProfile, action: SyncAction) -> SyncRefusal {
    match request_sync(profile, action) {
        Ok(()) => SyncRefusal {
            action: format!("{:?}", action).to_ascii_lowercase(),
            mode: profile.mode.as_str().to_string(),
            reason: "ok".to_string(),
            kind: "ok".to_string(),
        },
        Err(SyncError::ConfidentialForbidden) => SyncRefusal {
            action: format!("{:?}", action).to_ascii_lowercase(),
            mode: profile.mode.as_str().to_string(),
            reason: SyncError::ConfidentialForbidden.to_string(),
            kind: "confidential_forbidden".to_string(),
        },
        Err(SyncError::NotImplemented) => SyncRefusal {
            action: format!("{:?}", action).to_ascii_lowercase(),
            mode: profile.mode.as_str().to_string(),
            reason: SyncError::NotImplemented.to_string(),
            kind: "not_implemented".to_string(),
        },
        Err(SyncError::NotConfigured) => SyncRefusal {
            action: format!("{:?}", action).to_ascii_lowercase(),
            mode: profile.mode.as_str().to_string(),
            reason: SyncError::NotConfigured.to_string(),
            kind: "not_configured".to_string(),
        },
    }
}

// ---------------------------------------------------------------------------
// Cloud Routing & Storage Helpers
// ---------------------------------------------------------------------------

pub fn route_doctype_to_subfolder(doc_type: &str) -> &'static str {
    let clean = doc_type.trim().to_ascii_lowercase();
    match clean.as_str() {
        "order" => "orders",
        "motion" | "answer" | "notice" | "petition" | "complaint" | "pleading" | "brief"
        | "court filing" | "service of process" => "filings",
        "evidence" | "receipt" | "lease" => "evidence",
        _ => "evidence",
    }
}

pub fn session_file_path() -> PathBuf {
    let base = dirs::data_local_dir()
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("JustLegal").join("cloud_session.json")
}

pub fn load_cloud_session() -> Option<CloudSession> {
    let p = session_file_path();
    if !p.exists() {
        return None;
    }
    let data = fs::read_to_string(&p).ok()?;
    serde_json::from_str::<CloudSession>(&data).ok()
}

pub fn save_cloud_session(session: &CloudSession) -> Result<(), String> {
    let p = session_file_path();
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(session).map_err(|e| e.to_string())?;
    fs::write(p, data).map_err(|e| e.to_string())
}

pub fn clear_cloud_session() -> Result<(), String> {
    let p = session_file_path();
    if p.exists() {
        fs::remove_file(p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn calculate_file_sha256(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn load_case_sync_metadata(case_dir: &Path) -> CaseSyncMetadata {
    let sync_path = case_dir.join(".openseason").join("sync.json");
    if sync_path.exists() {
        if let Ok(data) = fs::read_to_string(&sync_path) {
            if let Ok(meta) = serde_json::from_str::<CaseSyncMetadata>(&data) {
                return meta;
            }
        }
    }
    CaseSyncMetadata::default()
}

pub fn save_case_sync_metadata(case_dir: &Path, meta: &CaseSyncMetadata) -> Result<(), String> {
    let dir = case_dir.join(".openseason");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let data = serde_json::to_string_pretty(meta).map_err(|e| e.to_string())?;
    fs::write(dir.join("sync.json"), data).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Firebase Auth & Cloud Sync Implementation
// ---------------------------------------------------------------------------

pub fn cloud_sign_in_password(email: &str, password: &str) -> Result<CloudSession, String> {
    let api_key = require_firebase_api_key()?;
    let client = Client::builder().build().map_err(|e| e.to_string())?;
    let url = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={}",
        api_key
    );
    let payload = json!({
        "email": email.trim(),
        "password": password,
        "returnSecureToken": true
    });

    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|e| format!("Network request failed: {e}"))?;

    let status = resp.status();
    let body: Value = resp.json().map_err(|e| format!("Invalid JSON response: {e}"))?;

    if !status.is_success() {
        let msg = body
            .pointer("/error/message")
            .and_then(|v| v.as_str())
            .unwrap_or("Sign-in failed");
        return Err(format!("Firebase Authentication error: {msg}"));
    }

    let id_token = body
        .get("idToken")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing idToken".to_string())?
        .to_string();
    let refresh_token = body
        .get("refreshToken")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let uid = body
        .get("localId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing uid".to_string())?
        .to_string();
    let email_resp = body
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or(email)
        .to_string();
    let expires_in: u64 = body
        .get("expiresIn")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(3600);

    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + expires_in;

    let session = CloudSession {
        id_token,
        refresh_token,
        uid,
        email: email_resp,
        expires_at,
    };

    save_cloud_session(&session)?;
    Ok(session)
}

fn extract_string_field(doc: &Value, name: &str) -> String {
    doc.pointer(&format!("/fields/{}/stringValue", name))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

pub fn cloud_pull_all(
    session: &CloudSession,
    target_root: &Path,
    filter_case_id: Option<&str>,
) -> Result<PullReport, String> {
    require_firebase_api_key()?;
    let client = Client::builder().build().map_err(|e| e.to_string())?;
    let run_query_url = format!(
        "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents:runQuery",
        FIREBASE_PROJECT_ID
    );

    let query_payload = json!({
        "structuredQuery": {
            "from": [{ "collectionId": "cases" }],
            "where": {
                "fieldFilter": {
                    "field": { "fieldPath": "userId" },
                    "op": "EQUAL",
                    "value": { "stringValue": session.uid }
                }
            }
        }
    });

    let resp = client
        .post(&run_query_url)
        .bearer_auth(&session.id_token)
        .json(&query_payload)
        .send()
        .map_err(|e| format!("Firestore query failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Firestore returned HTTP {}", resp.status()));
    }

    let items: Vec<Value> = resp.json().map_err(|e| format!("Query response parse error: {e}"))?;
    let mut report = PullReport::default();
    report.ok = true;

    // Load existing local cases to check matches
    let local_cases = case_profile::list_cases_in(target_root).unwrap_or_default();

    for item in items {
        let doc = match item.get("document") {
            Some(d) => d,
            None => continue,
        };

        let full_name = doc.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let cloud_case_id = full_name.rsplit('/').next().unwrap_or("").to_string();
        if cloud_case_id.is_empty() {
            continue;
        }

        if let Some(target_id) = filter_case_id {
            if cloud_case_id != target_id {
                continue;
            }
        }

        let title = extract_string_field(doc, "title");
        let case_number = extract_string_field(doc, "caseNumber");
        let venue = extract_string_field(doc, "venue");
        let state = extract_string_field(doc, "state");

        // Match existing local case: either by cloud_matter_id OR (if empty) by docket_number
        let mut matched_dir: Option<PathBuf> = None;
        for c in &local_cases {
            if c.cloud_matter_id.as_deref() == Some(&cloud_case_id) {
                matched_dir = Some(target_root.join(&c.id));
                break;
            }
            if !case_number.is_empty() && c.docket_number.trim() == case_number.trim() {
                matched_dir = Some(target_root.join(&c.id));
                break;
            }
        }

        let case_dir = match matched_dir {
            Some(dir) => {
                report.cases_updated += 1;
                dir
            }
            None => {
                // Generate a clean slug or UUID
                let slug = if !title.is_empty() {
                    let s: String = title
                        .to_ascii_lowercase()
                        .chars()
                        .map(|c| if c.is_alphanumeric() { c } else { '-' })
                        .collect();
                    s.split('-').filter(|x| !x.is_empty()).collect::<Vec<&str>>().join("-")
                } else {
                    cloud_case_id.clone()
                };
                let new_dir = target_root.join(&slug);
                let (dir, _) = case_profile::create_case_folder(
                    &new_dir,
                    if !title.is_empty() { &title } else { "Untitled Case" },
                    CaseMode::Standard,
                    "motion",
                )?;
                report.cases_pulled += 1;
                dir
            }
        };

        // Strict guard: skip if sealed or confidential
        if seal::is_dir_sealed(&case_dir) {
            report.issues.push(format!("Skipping sealed case folder: {}", case_dir.display()));
            continue;
        }

        let mut profile = case_profile::load_case(&case_dir)?;
        if profile.is_sync_forbidden() {
            report.issues.push(format!("Skipping confidential case: {}", profile.title));
            continue;
        }

        // Update profile metadata
        profile.cloud_matter_id = Some(cloud_case_id.clone());
        profile.cloud_synced_at = Some(chrono::Utc::now().to_rfc3339());
        if profile.docket_number.is_empty() && !case_number.is_empty() {
            profile.docket_number = case_number;
        }
        if profile.court.name.is_empty() && !venue.is_empty() {
            profile.court.name = venue;
        }
        if profile.rule_pack_id == "generic-us-appellate" && state.to_ascii_lowercase().contains("new jersey") {
            profile.rule_pack_id = "nj-app-div".into();
        }

        // Pull Events (Timeline)
        let events_url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/cases/{}/events?pageSize=100",
            FIREBASE_PROJECT_ID, cloud_case_id
        );
        if let Ok(ev_resp) = client.get(&events_url).bearer_auth(&session.id_token).send() {
            if ev_resp.status().is_success() {
                if let Ok(ev_body) = ev_resp.json::<Value>() {
                    if let Some(docs) = ev_body.get("documents").and_then(|v| v.as_array()) {
                        for edoc in docs {
                            let etitle = extract_string_field(edoc, "title");
                            let edate = extract_string_field(edoc, "dateText");
                            let edesc = extract_string_field(edoc, "description");
                            let ename = edoc.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let eid = format!("cloud:{}", ename.rsplit('/').next().unwrap_or(""));

                            if !profile.timeline.iter().any(|x| x.id == eid || (x.date == edate && x.title == etitle)) {
                                profile.timeline.push(TimelineEvent {
                                    id: eid,
                                    title: etitle,
                                    date: edate,
                                    notes: edesc,
                                });
                            }
                        }
                    }
                }
            }
        }

        case_profile::save_case(&case_dir, &profile)?;

        // Pull Documents & Download Attached Storage Files
        let mut sync_meta = load_case_sync_metadata(&case_dir);
        sync_meta.cloud_matter_id = cloud_case_id.clone();
        sync_meta.last_synced_at = chrono::Utc::now().to_rfc3339();

        let docs_url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/cases/{}/documents?pageSize=100",
            FIREBASE_PROJECT_ID, cloud_case_id
        );
        if let Ok(doc_resp) = client.get(&docs_url).bearer_auth(&session.id_token).send() {
            if doc_resp.status().is_success() {
                if let Ok(doc_body) = doc_resp.json::<Value>() {
                    if let Some(docs) = doc_body.get("documents").and_then(|v| v.as_array()) {
                        for ddoc in docs {
                            let dtitle = extract_string_field(ddoc, "title");
                            let doc_type = extract_string_field(ddoc, "docType");
                            let mut orig_name = extract_string_field(ddoc, "originalFilename");
                            if orig_name.is_empty() {
                                orig_name = if dtitle.is_empty() {
                                    "unnamed-record.pdf".into()
                                } else if dtitle.ends_with(".pdf") {
                                    dtitle.clone()
                                } else {
                                    format!("{}.pdf", dtitle)
                                };
                            }

                            let dname = ddoc.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let doc_id = dname.rsplit('/').next().unwrap_or("").to_string();
                            let subfolder = route_doctype_to_subfolder(&doc_type);
                            let target_folder = case_dir.join(subfolder);
                            let _ = fs::create_dir_all(&target_folder);
                            let dest_file = target_folder.join(&orig_name);

                            // Check if already synced with same doc_id and exists
                            let already_synced = sync_meta.files.iter().any(|f| f.doc_id == doc_id && dest_file.exists());
                            if already_synced {
                                continue;
                            }

                            // Download from Firebase Storage
                            // Primary path: cases/{caseId}/documents/{docId}/{orig_name}
                            let storage_path = format!("cases/{}/documents/{}/{}", cloud_case_id, doc_id, orig_name);
                            let encoded_path = urlencoding::encode(&storage_path);
                            let download_url = format!(
                                "https://firebasestorage.googleapis.com/v0/b/{}/o/{}?alt=media",
                                FIREBASE_STORAGE_BUCKET, encoded_path
                            );

                            let mut file_downloaded = false;
                            if let Ok(file_resp) = client.get(&download_url).bearer_auth(&session.id_token).send() {
                                if file_resp.status().is_success() {
                                    if let Ok(bytes) = file_resp.bytes() {
                                        if fs::write(&dest_file, &bytes).is_ok() {
                                            file_downloaded = true;
                                        }
                                    }
                                }
                            }

                            // Try fallback path if primary failed: documents/{docId}/{orig_name}
                            if !file_downloaded {
                                let fallback_path = format!("documents/{}/{}", doc_id, orig_name);
                                let encoded_fallback = urlencoding::encode(&fallback_path);
                                let fallback_url = format!(
                                    "https://firebasestorage.googleapis.com/v0/b/{}/o/{}?alt=media",
                                    FIREBASE_STORAGE_BUCKET, encoded_fallback
                                );
                                if let Ok(fb_resp) = client.get(&fallback_url).bearer_auth(&session.id_token).send() {
                                    if fb_resp.status().is_success() {
                                        if let Ok(bytes) = fb_resp.bytes() {
                                            if fs::write(&dest_file, &bytes).is_ok() {
                                                file_downloaded = true;
                                            }
                                        }
                                    }
                                }
                            }

                            if file_downloaded {
                                report.files_downloaded += 1;
                                let hash = calculate_file_sha256(&dest_file).unwrap_or_default();
                                let size = fs::metadata(&dest_file).map(|m| m.len()).unwrap_or(0);
                                sync_meta.files.push(SyncFileRecord {
                                    doc_id,
                                    title: dtitle,
                                    filename: orig_name,
                                    subfolder: subfolder.to_string(),
                                    sha256: hash,
                                    size_bytes: size,
                                    synced_at: chrono::Utc::now().to_rfc3339(),
                                });
                            }
                        }
                    }
                }
            }
        }

        let _ = save_case_sync_metadata(&case_dir, &sync_meta);
    }

    Ok(report)
}

pub fn cloud_push_case(case_dir: &Path, session: &CloudSession) -> Result<PushReport, String> {
    if seal::is_dir_sealed(case_dir) {
        return Err(SyncError::ConfidentialForbidden.to_string());
    }
    let mut profile = case_profile::load_case(case_dir)
        .map_err(|e| format!("Could not load case profile: {e}"))?;
    if profile.is_sync_forbidden() {
        return Err(SyncError::ConfidentialForbidden.to_string());
    }
    require_firebase_api_key()?;

    let client = Client::builder().build().map_err(|e| e.to_string())?;

    // Create cloud case if missing
    let cloud_matter_id = match &profile.cloud_matter_id {
        Some(id) if !id.trim().is_empty() => id.clone(),
        _ => {
            let create_url = format!(
                "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/cases",
                FIREBASE_PROJECT_ID
            );
            let payload = json!({
                "fields": {
                    "userId": { "stringValue": session.uid },
                    "title": { "stringValue": profile.title },
                    "caseNumber": { "stringValue": profile.docket_number },
                    "venue": { "stringValue": profile.court.name },
                    "jurisdiction": { "stringValue": profile.court.name },
                    "status": { "stringValue": "Active" },
                    "caseType": { "stringValue": "Civil" },
                    "createdAt": { "timestampValue": profile.created_at },
                    "updatedAt": { "timestampValue": chrono::Utc::now().to_rfc3339() }
                }
            });

            let resp = client
                .post(&create_url)
                .bearer_auth(&session.id_token)
                .json(&payload)
                .send()
                .map_err(|e| format!("Case creation on cloud failed: {e}"))?;

            if !resp.status().is_success() {
                return Err(format!("Cloud returned HTTP {}", resp.status()));
            }

            let body: Value = resp.json().map_err(|e| format!("Invalid JSON: {e}"))?;
            let full_name = body.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let new_id = full_name.rsplit('/').next().unwrap_or("").to_string();
            if new_id.is_empty() {
                return Err("Failed to obtain created matter ID".into());
            }

            profile.cloud_matter_id = Some(new_id.clone());
            case_profile::save_case(case_dir, &profile)?;
            new_id
        }
    };

    let mut sync_meta = load_case_sync_metadata(case_dir);
    sync_meta.cloud_matter_id = cloud_matter_id.clone();
    let mut files_uploaded = 0;

    // Check exports/ folder for compiled documents to push
    let exports_dir = case_dir.join("exports");
    if exports_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&exports_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                    let hash = calculate_file_sha256(&path).unwrap_or_default();

                    // If already synced with same SHA-256, skip
                    if sync_meta.files.iter().any(|f| f.filename == fname && f.sha256 == hash) {
                        continue;
                    }

                    // 1. Create document record in Firestore
                    let doc_create_url = format!(
                        "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/cases/{}/documents",
                        FIREBASE_PROJECT_ID, cloud_matter_id
                    );
                    let doc_payload = json!({
                        "fields": {
                            "title": { "stringValue": fname },
                            "docType": { "stringValue": "Court Filing" },
                            "originalFilename": { "stringValue": fname },
                            "contentType": { "stringValue": if fname.ends_with(".docx") { "application/vnd.openxmlformats-officedocument.wordprocessingml.document" } else { "application/pdf" } },
                            "sizeBytes": { "integerValue": fs::metadata(&path).map(|m| m.len()).unwrap_or(0).to_string() },
                            "uploadedBy": { "stringValue": session.uid },
                            "createdAt": { "timestampValue": chrono::Utc::now().to_rfc3339() },
                            "updatedAt": { "timestampValue": chrono::Utc::now().to_rfc3339() }
                        }
                    });

                    if let Ok(dresp) = client.post(&doc_create_url).bearer_auth(&session.id_token).json(&doc_payload).send() {
                        if dresp.status().is_success() {
                            if let Ok(dbody) = dresp.json::<Value>() {
                                let doc_id = dbody.get("name").and_then(|v| v.as_str()).unwrap_or("").rsplit('/').next().unwrap_or("").to_string();
                                
                                // 2. Upload file bytes to Firebase Storage
                                let storage_path = format!("cases/{}/documents/{}/{}", cloud_matter_id, doc_id, fname);
                                let upload_url = format!(
                                    "https://firebasestorage.googleapis.com/v0/b/{}/o?name={}",
                                    FIREBASE_STORAGE_BUCKET,
                                    urlencoding::encode(&storage_path)
                                );

                                if let Ok(bytes) = fs::read(&path) {
                                    let content_type = if fname.ends_with(".docx") {
                                        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                                    } else {
                                        "application/pdf"
                                    };
                                    let _ = client
                                        .post(&upload_url)
                                        .bearer_auth(&session.id_token)
                                        .header("Content-Type", content_type)
                                        .body(bytes)
                                        .send();
                                }

                                files_uploaded += 1;
                                sync_meta.files.push(SyncFileRecord {
                                    doc_id,
                                    title: fname.clone(),
                                    filename: fname,
                                    subfolder: "exports".to_string(),
                                    sha256: hash,
                                    size_bytes: fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
                                    synced_at: chrono::Utc::now().to_rfc3339(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    sync_meta.last_synced_at = chrono::Utc::now().to_rfc3339();
    let _ = save_case_sync_metadata(case_dir, &sync_meta);

    Ok(PushReport {
        ok: true,
        cloud_matter_id,
        files_uploaded,
        message: format!("Successfully synced with cloud (uploaded {} files)", files_uploaded),
    })
}

// Simple URL encoder helper to avoid external crate requirement if needed
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut out = String::new();
        for b in s.bytes() {
            if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
                out.push(b as char);
            } else {
                out.push_str(&format!("%{:02X}", b));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_profile::sample_appeal_profile;

    #[test]
    fn confidential_case_cannot_backup_publish_or_pull() {
        let mut profile = sample_appeal_profile();
        profile.convert_to_confidential().unwrap();

        for action in [SyncAction::Backup, SyncAction::Publish, SyncAction::PullAccount] {
            let err = request_sync(&profile, action).unwrap_err();
            assert_eq!(err, SyncError::ConfidentialForbidden);
            let described = describe_sync_error(&profile, action);
            assert_eq!(described.kind, "confidential_forbidden");
        }
    }

    #[test]
    fn sealed_case_stays_blocked_even_if_mode_flag_is_flipped() {
        let mut profile = sample_appeal_profile();
        profile.convert_to_confidential().unwrap();
        profile.mode = CaseMode::Standard;
        let err = request_sync(&profile, SyncAction::Publish).unwrap_err();
        assert_eq!(err, SyncError::ConfidentialForbidden);
    }

    #[test]
    fn standard_case_hits_stub_not_confidential_error() {
        let profile = sample_appeal_profile();
        assert!(!profile.is_sync_forbidden());
        let err = request_sync(&profile, SyncAction::Backup).unwrap_err();
        assert!(
            matches!(err, SyncError::NotImplemented | SyncError::NotConfigured),
            "standard cases must not be treated as confidential; got {err:?}"
        );
        let described = describe_sync_error(&profile, SyncAction::Backup);
        assert!(
            described.kind == "not_implemented" || described.kind == "not_configured",
            "unexpected kind {}",
            described.kind
        );
    }

    #[test]
    fn firebase_api_key_resolver_prefers_env_then_config_then_build() {
        assert!(resolve_firebase_api_key(None, None, None).is_none());
        assert!(resolve_firebase_api_key(Some("   "), Some(""), Some("")).is_none());
        assert_eq!(
            resolve_firebase_api_key(Some(" from-env "), Some("from-file"), Some("from-build"))
                .as_deref(),
            Some("from-env")
        );
        assert_eq!(
            resolve_firebase_api_key(None, Some("from-file"), Some("from-build")).as_deref(),
            Some("from-file")
        );
        assert_eq!(
            resolve_firebase_api_key(None, None, Some("from-build")).as_deref(),
            Some("from-build")
        );
    }

    #[test]
    fn user_config_json_supplies_api_key() {
        let dir = std::env::temp_dir().join(format!("os-fb-cfg-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        fs::write(&path, r#"{ "firebase_api_key": " test-key-value " }"#).unwrap();
        assert_eq!(
            load_firebase_api_key_from_config_path(&path).as_deref(),
            Some("test-key-value")
        );
        fs::write(&path, r#"{}"#).unwrap();
        assert!(load_firebase_api_key_from_config_path(&path).is_none());
        fs::write(&path, "not-json").unwrap();
        assert!(load_firebase_api_key_from_config_path(&path).is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_api_key_message_is_actionable_and_does_not_crash() {
        let msg = firebase_api_key_missing_message();
        assert!(msg.contains(FIREBASE_API_KEY_ENV));
        assert!(msg.contains("config.json"));
        assert!(msg.to_ascii_lowercase().contains("disabled"));
        assert!(msg.to_ascii_lowercase().contains("offline"));
        let sign_in = cloud_sign_in_password("user@example.test", "not-a-real-password");
        if firebase_api_key().is_none() {
            let err = sign_in.expect_err("sign-in must not panic or succeed without a key");
            assert!(err.contains(FIREBASE_API_KEY_ENV));
            assert!(err.to_ascii_lowercase().contains("disabled"));
        }
    }

    #[test]
    fn local_export_is_not_a_sync_action() {
        let mut profile = sample_appeal_profile();
        profile.convert_to_confidential().unwrap();
        assert!(profile.is_sync_forbidden());
        assert!(matches!(
            request_sync(&profile, SyncAction::Publish),
            Err(SyncError::ConfidentialForbidden)
        ));
    }

    #[test]
    fn doctype_routing_maps_real_values_correctly() {
        assert_eq!(route_doctype_to_subfolder("Order"), "orders");
        assert_eq!(route_doctype_to_subfolder("Motion"), "filings");
        assert_eq!(route_doctype_to_subfolder("Brief"), "filings");
        assert_eq!(route_doctype_to_subfolder("Court Filing"), "filings");
        assert_eq!(route_doctype_to_subfolder("Evidence"), "evidence");
        assert_eq!(route_doctype_to_subfolder("Receipt"), "evidence");
    }

    fn contains_firebase_web_api_key(text: &str) -> bool {
        let bytes = text.as_bytes();
        let prefix = b"AIza";
        let mut i = 0;
        while i + 39 <= bytes.len() {
            if &bytes[i..i + 4] == prefix
                && bytes[i + 4..i + 39]
                    .iter()
                    .all(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'-')
            {
                return true;
            }
            i += 1;
        }
        false
    }

    #[test]
    fn firebase_key_pattern_rejects_placeholders() {
        assert!(!contains_firebase_web_api_key("OPENSEASON_FIREBASE_API_KEY="));
        assert!(!contains_firebase_web_api_key("AIza[0-9A-Za-z_-]{35}"));
        assert!(!contains_firebase_web_api_key("your-web-api-key"));
        let fake = format!("AIza{}", "x".repeat(35));
        assert!(contains_firebase_web_api_key(&fake));
    }

    #[test]
    fn tracked_files_must_not_contain_firebase_web_api_keys() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let output = std::process::Command::new("git")
            .args(["ls-files", "-z"])
            .current_dir(&root)
            .output()
            .expect("git ls-files must run so committed secrets can be detected");
        assert!(
            output.status.success(),
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let mut hits = Vec::new();
        for path in output.stdout.split(|b| *b == 0).filter(|p| !p.is_empty()) {
            let rel = String::from_utf8_lossy(path);
            let full = root.join(rel.as_ref());
            let Ok(bytes) = std::fs::read(&full) else {
                continue;
            };
            let Ok(text) = String::from_utf8(bytes) else {
                continue;
            };
            if contains_firebase_web_api_key(&text) {
                hits.push(rel.into_owned());
            }
        }

        assert!(
            hits.is_empty(),
            "Firebase Web API key pattern found in tracked files: {hits:?}. \
             Remove the literal and load the key at runtime."
        );
    }
}
