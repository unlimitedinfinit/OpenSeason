use serde::{Deserialize, Serialize};

/// Comprehensive directory metadata for court access, clerk counters, and records.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CourtAccessProfile {
    pub court_id: String,
    pub court_name: String,
    pub jurisdiction_state: String,
    pub physical_address: String,
    pub clerk_counter_hours: String,
    pub phone: String,
    pub electronic_portal_name: String,
    pub electronic_portal_url: String,
    pub pro_se_efiling_allowed: bool,
    pub docket_search_url: String,
    pub records_access_rule: String,
    pub transcript_request_form: String,
    pub transcript_deposit_rules: String,
    pub statutory_fees_summary: String,
    pub indigent_fee_waiver_available: bool,
    pub special_instructions: Vec<String>,
}

/// An item needed from the court record during retrieval.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissingRecordRetrievalItem {
    pub title: String,
    pub reason: String,
    pub record_tier: String,
    pub target_agency_or_court: String,
    pub required_form: String,
}

/// Guided wizard plan for retrieving missing court records.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecordRetrievalWizardPlan {
    pub case_id: String,
    pub court_id: String,
    pub court_name: String,
    pub missing_records: Vec<MissingRecordRetrievalItem>,
    pub recommended_form: String,
    pub recommended_form_number: String,
    pub submission_channels: Vec<String>,
    pub estimated_cost_explanation: String,
    pub fee_waiver_rule: Option<String>,
    pub followup_reminder_days: u32,
    pub step_by_step_instructions: Vec<String>,
}
