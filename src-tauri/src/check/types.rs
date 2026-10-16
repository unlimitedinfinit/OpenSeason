//! Data types, models, and notices for checks and verification.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::case_profile::ValidationIssue;
use crate::deadlines::DeadlineResult;

pub const NEVER_FILE_NOTICE: &str =
    "This app never files or serves. Josh (or the user) signs, files, and serves by hand.";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FactRow {
    pub id: u32,
    pub assertion: String,
    #[serde(default)]
    pub record_cite: String,
    #[serde(default)]
    pub exhibit: String,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FactCheckReport {
    pub ok: bool,
    pub never_files: bool,
    pub rows: Vec<FactRow>,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityStatus {
    Verified,
    Unverified,
    NotFound,
}

impl Default for AuthorityStatus {
    fn default() -> Self {
        AuthorityStatus::Unverified
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthorityRow {
    pub id: u32,
    pub cite: String,
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub status: AuthorityStatus,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthorityReport {
    pub ok: bool,
    pub never_files: bool,
    pub invented: bool,
    pub rows: Vec<AuthorityRow>,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignoffItem {
    pub id: String,
    pub label: String,
    pub checked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignoffReport {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub items: Vec<SignoffItem>,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckAllReport {
    pub never_files: bool,
    pub never_serves: bool,
    pub notice: String,
    pub fact: FactCheckReport,
    pub authority: AuthorityReport,
    pub deadline: Option<DeadlineResult>,
    pub signoff: SignoffReport,
    pub ok: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranscriptCitation {
    pub raw: String,
    pub transcript_name: String,
    pub page: u32,
    pub line: Option<u32>,
    pub verified: bool,
    pub quote_found: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranscriptReport {
    pub ok: bool,
    pub never_files: bool,
    pub indexed_files: Vec<String>,
    pub total_lines: usize,
    pub citations: Vec<TranscriptCitation>,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreFlightAudit {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub caption_ok: bool,
    pub caption_summary: String,
    pub docket_ok: bool,
    pub docket_summary: String,
    pub filer_ok: bool,
    pub filer_summary: String,
    pub deadline_ok: bool,
    pub deadline_summary: String,
    pub facts_ok: bool,
    pub facts_verified_count: usize,
    pub facts_total_count: usize,
    pub authorities_ok: bool,
    pub authorities_verified_count: usize,
    pub authorities_total_count: usize,
    pub transcripts_ok: bool,
    pub packet_parts_ready: bool,
    pub missing_requirements: Vec<String>,
    pub notice: String,
}

pub fn checks_dir(case_dir: &Path) -> PathBuf {
    case_dir.join("checks")
}

pub fn ensure_checks(case_dir: &Path) -> Result<PathBuf, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Checks cannot write into the Documents pointer.".into());
    }
    let dir = checks_dir(case_dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())
}

pub fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map(Some).map_err(|e| e.to_string())
}
