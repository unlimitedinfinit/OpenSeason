use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordSupportStatus {
    UnsupportedInRecord,
    ContradictedByRecord,
    UnverifiedOralColloquy,
    VerifiedInRecord,
}

impl RecordSupportStatus {
    pub fn label(&self) -> &'static str {
        match self {
            RecordSupportStatus::UnsupportedInRecord => "Unsupported in Record (No record citation found)",
            RecordSupportStatus::ContradictedByRecord => "Contradicted by Record Evidence",
            RecordSupportStatus::UnverifiedOralColloquy => "Unverified Oral Colloquy (Transcript required)",
            RecordSupportStatus::VerifiedInRecord => "Supported by Verified Record",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FindingSource {
    pub document_id: String,
    pub document_name: String,
    pub page: Option<u32>,
    pub quoted_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteKind {
    ReconsiderationInterlocutory,
    ReconsiderationFinal,
    AppellateReview,
    RecordCorrection,
    JudicialConductGrievance,
    AttorneyEthicsGrievance,
    PublicRecordsRequest,
    AdaAccommodation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountabilityRoute {
    pub route_kind: RouteKind,
    pub title: String,
    pub rule_citation: String,
    pub governing_standard: String,
    pub deadline_notice: Option<String>,
    pub is_disciplinary: bool,
    pub warning_notice: String,
    pub official_form_name: Option<String>,
    pub submission_portal_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IssueItem {
    pub id: String,
    pub title: String,
    pub date_identified: String,
    pub finding_source: FindingSource,
    pub record_support_status: RecordSupportStatus,
    pub contradicting_evidence_local: Vec<String>,
    pub contradicting_evidence_filed: Vec<String>,
    pub analysis_notes: String,
    pub available_routes: Vec<AccountabilityRoute>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscrepancyReport {
    pub case_id: String,
    pub court_name: String,
    pub issues: Vec<IssueItem>,
    pub non_tolling_warning: String,
    pub summary: String,
}
