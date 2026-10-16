use serde::{Deserialize, Serialize};

/// The 4 distinct evidentiary tiers to prevent unfiled drafts from masquerading as the court record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidentiaryTier {
    /// Tier 1: Documents actually filed, entered, served, or docketed by the court clerk.
    OfficialCourtRecord,
    /// Tier 2: Certified hearing transcripts, audio recordings, admitted trial exhibits, testimony.
    HearingAndEvidentiaryRecord,
    /// Tier 3: Personal drafts, client notes, internal work product, unserved letters, emails.
    PersonalCaseArchive,
    /// Tier 4: Extrinsic documents that exist privately but were NEVER submitted or admitted below.
    PotentialAdditionalEvidence,
}

impl EvidentiaryTier {
    pub fn label(&self) -> &'static str {
        match self {
            EvidentiaryTier::OfficialCourtRecord => "Tier 1: Official Court Record",
            EvidentiaryTier::HearingAndEvidentiaryRecord => "Tier 2: Hearing / Transcript Record",
            EvidentiaryTier::PersonalCaseArchive => "Tier 3: Personal Case Archive",
            EvidentiaryTier::PotentialAdditionalEvidence => "Tier 4: Potential Additional Evidence (Unsubmitted)",
        }
    }

    pub fn is_official(&self) -> bool {
        matches!(self, EvidentiaryTier::OfficialCourtRecord | EvidentiaryTier::HearingAndEvidentiaryRecord)
    }
}

/// The 7 independent provenance dimensions for document integrity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentProvenance {
    /// 1. Origin: "court", "litigant", "opposing_counsel", "third_party", "agency"
    #[serde(default = "default_provenance_origin")]
    pub origin: String,
    /// 2. Identity: original path and version count
    #[serde(default)]
    pub original_path: Option<String>,
    #[serde(default = "default_version_one")]
    pub version: u32,
    /// 3. Submission: sent date, channel ("eCourts", "mail", "hand"), confirmation ID, rejection note
    #[serde(default)]
    pub submission_channel: Option<String>,
    #[serde(default)]
    pub submission_date: Option<String>,
    #[serde(default)]
    pub submission_status: Option<String>,
    #[serde(default)]
    pub rejection_reason: Option<String>,
    /// 4. Filing: accepted date, docket sequence number, docket code, clerk stamp
    #[serde(default)]
    pub accepted_date: Option<String>,
    #[serde(default)]
    pub docket_sequence: Option<u32>,
    #[serde(default)]
    pub docket_code: Option<String>,
    #[serde(default)]
    pub filing_stamp_verified: bool,
    /// 5. Service: served parties, method, date served, certificate of service reference
    #[serde(default)]
    pub served_recipients: Vec<String>,
    #[serde(default)]
    pub service_method: Option<String>,
    #[serde(default)]
    pub service_date: Option<String>,
    #[serde(default)]
    pub certificate_of_service_ref: Option<String>,
    /// 6. Evidentiary status: position in proceeding (official, transcript, archive, unsubmitted)
    #[serde(default = "default_provenance_tier")]
    pub evidentiary_tier: EvidentiaryTier,
    /// 7. Completeness: "complete", "partial", "missing_pages", "unverified"
    #[serde(default = "default_provenance_completeness")]
    pub completeness: String,
}

fn default_provenance_origin() -> String {
    "unknown".into()
}

fn default_version_one() -> u32 {
    1
}

fn default_provenance_tier() -> EvidentiaryTier {
    EvidentiaryTier::PersonalCaseArchive
}

fn default_provenance_completeness() -> String {
    "unverified".into()
}

impl Default for DocumentProvenance {
    fn default() -> Self {
        Self {
            origin: default_provenance_origin(),
            original_path: None,
            version: 1,
            submission_channel: None,
            submission_date: None,
            submission_status: None,
            rejection_reason: None,
            accepted_date: None,
            docket_sequence: None,
            docket_code: None,
            filing_stamp_verified: false,
            served_recipients: Vec::new(),
            service_method: None,
            service_date: None,
            certificate_of_service_ref: None,
            evidentiary_tier: default_provenance_tier(),
            completeness: default_provenance_completeness(),
        }
    }
}

/// An entry in the reconstructed case record inventory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecordEntry {
    pub id: String,
    pub filename: String,
    pub relative_path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub tier: EvidentiaryTier,
    pub docket_date: Option<String>,
    pub filing_stamp_detected: bool,
    pub source_provenance: String,
    pub title: String,
    pub notes: String,
    pub verified: bool,
    #[serde(default)]
    pub provenance: Option<DocumentProvenance>,
}


/// A missing document or record needed to complete/perfect the case record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissingRecordItem {
    pub id: String,
    pub title: String,
    pub record_type: String, // "transcript", "order", "opposition", "exhibit", "docket_sheet"
    pub reason: String,
    pub docket_entry_ref: Option<String>,
    pub recommended_action: String,
    pub estimated_cost_notice: String,
    pub request_form: Option<String>,
}

/// Docket entry extracted from court register of actions / docket sheet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocketEntry {
    pub sequence_num: Option<u32>,
    pub date: String,
    pub title: String,
    pub description: String,
    pub filed_by: Option<String>,
    pub matched_record_id: Option<String>,
}

/// Full inventory and reconciliation report for a case workspace.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecordInventoryReport {
    pub case_id: String,
    pub court_name: String,
    pub docket_number: String,
    pub total_files_scanned: usize,
    pub tier1_official_count: usize,
    pub tier2_hearing_count: usize,
    pub tier3_archive_count: usize,
    pub tier4_unsubmitted_count: usize,
    pub entries: Vec<RecordEntry>,
    pub missing_records: Vec<MissingRecordItem>,
    pub reconciliation_summary: String,
}
