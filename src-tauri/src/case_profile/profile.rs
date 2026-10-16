//! CaseProfile struct and core business logic methods.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseProfile {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub mode: CaseMode,
    #[serde(default)]
    pub document_kind: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    /// Once set, the case can never be treated as syncable again.
    #[serde(default)]
    pub sealed_at: Option<String>,
    #[serde(default)]
    pub court: CourtInfo,
    #[serde(default)]
    pub docket_number: String,
    #[serde(default)]
    pub caption: Caption,
    #[serde(default)]
    pub filer: Filer,
    #[serde(default)]
    pub notice_of_appeal: NoticeOfAppealDraft,
    #[serde(default)]
    pub complaint: PleadingDraft,
    #[serde(default)]
    pub motion: PleadingDraft,
    #[serde(default)]
    pub timeline: Vec<TimelineEvent>,
    /// Rule pack id: generic-us-appellate, nj-app-div, or co-mandamus.
    #[serde(default)]
    pub rule_pack_id: String,
    /// reconsideration, enlargement, stay, amend_findings, or empty.
    #[serde(default)]
    pub motion_kind: String,
    #[serde(default)]
    pub brief: PleadingDraft,
    #[serde(default)]
    pub appellate_motion: PleadingDraft,
    #[serde(default)]
    pub mandamus: PleadingDraft,
    /// Full court-markdown draft (md2court subset) when imported.
    #[serde(default)]
    pub court_markdown: String,
    #[serde(default)]
    pub cloud_matter_id: Option<String>,
    #[serde(default)]
    pub cloud_synced_at: Option<String>,
    #[serde(default)]
    pub filings: Vec<CaseFilingItem>,
    /// Matter tier: "litigation" (default), "administrative", or "transactional"
    #[serde(default)]
    pub matter_tier: String,
    /// Procedural stage: "pre_filing", "active_docket", "post_judgment", "appellate", "supreme_court"
    #[serde(default)]
    pub procedural_stage: String,
    /// Governing state/venue code: e.g. "NJ", "CO", "FEDERAL", "SCOTUS"
    #[serde(default)]
    pub governing_state: String,
    /// Optional parent matter id when graduated from Tier 2 -> Tier 3 or escalated Trial -> Appeal -> SCOTUS
    #[serde(default)]
    pub parent_matter_id: Option<String>,
    /// Conferral status for venues requiring conferral certificates (e.g. CO C.R.C.P. 121 § 1-15(8))
    #[serde(default)]
    pub conferral_status: String,
    /// Default service method ("electronic", "mail", "personal", "certified_mail") for deadline mailbox offsets
    #[serde(default)]
    pub service_method: String,
    /// Caption layout: "boxed" (default for motions) or "classic" (legacy centered stack).
    #[serde(default)]
    pub caption_layout: String,
    /// Verified personnel roster (counsel, parties, court staff, substitution history)
    #[serde(default)]
    pub roster: Vec<RosterEntry>,
    /// Issue ledger (factual questions, competing assertions, contrary authorities)
    #[serde(default)]
    pub issues: Vec<IssueItem>,
    /// Action tracker (pending tasks, obligations, inquiries)
    #[serde(default)]
    pub tasks: Vec<ActionTask>,
    /// Agent and tool audit trail receipts
    #[serde(default)]
    pub receipts: Vec<OperationReceipt>,
}


pub fn default_schema_version() -> u32 {
    1
}

impl CaseProfile {
    pub fn effective_tier(&self) -> &'static str {
        match self.matter_tier.trim().to_ascii_lowercase().as_str() {
            "transactional" | "tier1" | "everyday" => "transactional",
            "administrative" | "tier2" | "pre_suit" | "presuit" => "administrative",
            "litigation" | "tier3" => "litigation",
            _ => match self.document_kind.trim().to_ascii_lowercase().as_str() {
                "will" | "lease" | "contractor" | "power_of_attorney" | "bill_of_sale" | "nda" => {
                    "transactional"
                }
                "demand_letter"
                | "cure_notice"
                | "security_deposit_demand"
                | "records_request"
                | "agency_appeal" => "administrative",
                _ => "litigation",
            },
        }
    }

    pub fn is_pre_filing(&self) -> bool {
        self.procedural_stage.trim().eq_ignore_ascii_case("pre_filing")
    }

    pub fn new_standard(title: &str, document_kind: &str) -> Self {
        let now = Utc::now().to_rfc3339();
        let kind_lower = document_kind.trim().to_ascii_lowercase();
        let inferred_tier = match kind_lower.as_str() {
            "will" | "lease" | "contractor" | "power_of_attorney" | "bill_of_sale" | "nda" => {
                "transactional"
            }
            "demand_letter"
            | "cure_notice"
            | "security_deposit_demand"
            | "records_request"
            | "agency_appeal" => "administrative",
            _ => "litigation",
        };
        Self {
            schema_version: 1,
            id: Uuid::new_v4().to_string(),
            title: title.trim().to_string(),
            mode: CaseMode::Standard,
            document_kind: document_kind.to_string(),
            created_at: now.clone(),
            updated_at: now,
            sealed_at: None,
            court: CourtInfo {
                id: "generic-us-appellate".to_string(),
                name: String::new(),
                division: None,
                address_lines: Vec::new(),
            },
            docket_number: String::new(),
            caption: Caption::default(),
            filer: Filer::default(),
            notice_of_appeal: NoticeOfAppealDraft::default(),
            complaint: PleadingDraft::default(),
            motion: PleadingDraft::default(),
            timeline: Vec::new(),
            rule_pack_id: "generic-us-appellate".to_string(),
            motion_kind: String::new(),
            brief: PleadingDraft::default(),
            appellate_motion: PleadingDraft::default(),
            mandamus: PleadingDraft::default(),
            court_markdown: String::new(),
            cloud_matter_id: None,
            cloud_synced_at: None,
            filings: Vec::new(),
            matter_tier: inferred_tier.to_string(),
            procedural_stage: String::new(),
            governing_state: String::new(),
            parent_matter_id: None,
            conferral_status: String::new(),
            service_method: "electronic".to_string(),
            caption_layout: String::new(),
            roster: Vec::new(),
            issues: Vec::new(),
            tasks: Vec::new(),
            receipts: Vec::new(),
        }
    }

    pub fn is_sync_forbidden(&self) -> bool {
        self.mode == CaseMode::Confidential || self.sealed_at.is_some()
    }

    pub fn convert_to_confidential(&mut self) -> Result<(), String> {
        if self.mode == CaseMode::Confidential && self.sealed_at.is_some() {
            return Err("This case is already confidential.".to_string());
        }
        let now = Utc::now().to_rfc3339();
        self.mode = CaseMode::Confidential;
        self.sealed_at = Some(now.clone());
        self.updated_at = now;
        Ok(())
    }

    pub fn reopen_as_standard(&self) -> Result<CaseProfile, String> {
        if !self.is_sync_forbidden() {
            return Err("This case is not sealed.".to_string());
        }
        Err("Confidential cases cannot be made syncable. Use a deliberate local export if you need a copy on disk.".to_string())
    }

    /// Add or update a person/entity in the roster
    pub fn upsert_roster_entry(&mut self, entry: RosterEntry) {
        if let Some(existing) = self.roster.iter_mut().find(|r| r.id == entry.id) {
            *existing = entry;
        } else {
            self.roster.push(entry);
        }
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Record an attorney substitution, archiving former counsel without losing service history
    pub fn substitute_counsel(
        &mut self,
        former_counsel_id: &str,
        new_counsel: RosterEntry,
        substitution_date: &str,
        appearance_ref: Option<String>,
    ) -> Result<(), String> {
        let former_name = if let Some(former) = self.roster.iter_mut().find(|r| r.id == former_counsel_id) {
            former.is_active = false;
            former.substitution_history.push(SubstitutionRecord {
                date: substitution_date.to_string(),
                substituted_by: new_counsel.name.clone(),
                notice_of_appearance_ref: appearance_ref,
                notes: format!("Substituted by {} on {}", new_counsel.name, substitution_date),
            });
            former.name.clone()
        } else {
            return Err(format!("Former counsel id '{}' not found in roster.", former_counsel_id));
        };

        // Add the new counsel as active
        self.upsert_roster_entry(new_counsel);

        // Add a timeline event to maintain verified chronological integrity
        self.timeline.push(TimelineEvent {
            id: format!("subst-{}", Uuid::new_v4().simple()),
            title: format!("Substitution of Counsel: {} replaced by active counsel", former_name),
            date: substitution_date.to_string(),
            notes: "Recorded in personnel roster with substitution history preserved.".to_string(),
        });

        self.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    /// Add or update an issue in the issue ledger
    pub fn upsert_issue(&mut self, issue: IssueItem) {
        if let Some(existing) = self.issues.iter_mut().find(|i| i.id == issue.id) {
            *existing = issue;
        } else {
            self.issues.push(issue);
        }
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Log an operation receipt for agent and tool audit trails
    pub fn log_receipt(&mut self, receipt: OperationReceipt) {
        self.receipts.push(receipt);
        self.updated_at = Utc::now().to_rfc3339();
    }
}

