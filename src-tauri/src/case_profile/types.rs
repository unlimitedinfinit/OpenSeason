//! Data types, party definitions, and structures for case profiles.

use serde::{Deserialize, Serialize};

pub const CASE_SUBDIRS: &[&str] = &[
    "orders",
    "filings",
    "evidence",
    "drafts",
    "court-rules",
    "exports",
];

pub const REVIEW_NOTICE: &str = "This document was assembled by OpenSeason from the user's own case profile and draft text. It is a formatting aid, not legal advice. Review every line, caption, date, and citation before filing. Consider consulting a licensed attorney.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaseMode {
    Standard,
    Confidential,
}

impl CaseMode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "standard" => Ok(CaseMode::Standard),
            "confidential" => Ok(CaseMode::Confidential),
            other => Err(format!(
                "Unknown case mode '{}'. Use standard or confidential.",
                other
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CaseMode::Standard => "standard",
            CaseMode::Confidential => "confidential",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Party {
    pub name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone: String,
    /// Attorney name, or "Pro se" when the party is representing themselves.
    #[serde(default)]
    pub counsel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Caption {
    #[serde(default)]
    pub plaintiffs: Vec<Party>,
    #[serde(default)]
    pub defendants: Vec<Party>,
    #[serde(default)]
    pub appellants: Vec<Party>,
    #[serde(default)]
    pub appellees: Vec<Party>,
}

impl Caption {
    pub fn appellants(&self) -> Vec<&Party> {
        if !self.appellants.is_empty() {
            self.appellants.iter().collect()
        } else {
            self.plaintiffs.iter().collect()
        }
    }

    pub fn appellees(&self) -> Vec<&Party> {
        if !self.appellees.is_empty() {
            self.appellees.iter().collect()
        } else {
            self.defendants.iter().collect()
        }
    }

    pub fn party_names(parties: &[&Party]) -> String {
        let names: Vec<&str> = parties.iter().map(|p| p.name.as_str()).collect();
        match names.as_slice() {
            [] => String::new(),
            [one] => (*one).to_string(),
            [first, second] => format!("{} and {}", first, second),
            _ => {
                let (last, rest) = names.split_last().unwrap();
                format!("{} and {}", rest.join(", "), last)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CourtInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub division: Option<String>,
    /// Optional court mailing address. Never invent an address.
    #[serde(default)]
    pub address_lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Filer {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub address_lines: Vec<String>,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub signature_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NoticeOfAppealDraft {
    #[serde(default)]
    pub judgment_date: String,
    #[serde(default)]
    pub judgment_description: String,
    #[serde(default)]
    pub trial_court_name: String,
    #[serde(default)]
    pub trial_court_docket: String,
    #[serde(default)]
    pub user_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CaseFilingItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub rule: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub user_text: String,
    #[serde(default)]
    pub relief_sought: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PleadingDraft {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub user_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TimelineEvent {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationIssue {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationReport {
    pub ok: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SubstitutionRecord {
    pub date: String,
    pub substituted_by: String,
    #[serde(default)]
    pub notice_of_appearance_ref: Option<String>,
    #[serde(default)]
    pub notes: String,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RosterEntry {
    pub id: String,
    pub name: String,
    /// e.g. "plaintiff", "defendant", "appellant", "appellee", "opposing_counsel", "co_counsel", "judge", "clerk"
    pub role: String,
    #[serde(default)]
    pub represented_party_id: Option<String>,
    #[serde(default)]
    pub bar_number: Option<String>,
    #[serde(default)]
    pub firm_name: Option<String>,
    #[serde(default)]
    pub address_lines: Vec<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default)]
    pub service_preference: Option<String>,
    #[serde(default)]
    pub substitution_history: Vec<SubstitutionRecord>,
}

fn default_unresolved() -> String {
    "unresolved".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct IssueItem {
    pub id: String,
    pub title: String,
    /// e.g. "jurisdictional", "procedural", "factual", "statutory"
    pub category: String,
    /// e.g. "court", "adversary", "user"
    pub asserted_by: String,
    #[serde(default)]
    pub record_citations: Vec<String>,
    #[serde(default)]
    pub contrary_authorities: Vec<String>,
    /// "unresolved", "verified", "contradicted", "moot"
    #[serde(default = "default_unresolved")]
    pub status: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ActionTask {
    pub id: String,
    pub title: String,
    /// "pending", "in_progress", "completed", "dismissed"
    pub status: String,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub assigned_to: Option<String>,
    /// "deadline_response", "records_request", "service_proof", "filing", "inquiry"
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub target_ref: Option<String>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct OperationReceipt {
    pub id: String,
    pub timestamp: String,
    /// e.g. "order.decode", "draft.fill", "filing.validate", "consultation.prepare"
    pub operation: String,
    /// "agent", "user", "gui", "cli"
    pub invoked_by: String,
    /// "success", "needs_verification", "rejected", "error"
    pub status: String,
    #[serde(default)]
    pub files_modified: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

