use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VulnerabilitySeverity {
    Critical,
    Significant,
    Advisory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VulnerabilityType {
    StandardOfReview,
    PreservationIssue,
    HarmlessError,
    MissingRecordGrounding,
    InterlocutoryFinality,
    NonTolledClock,
    HearsayWithoutException,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestVulnerability {
    pub vulnerability_type: VulnerabilityType,
    pub severity: VulnerabilitySeverity,
    pub title: String,
    pub description: String,
    pub opposing_argument_prediction: String,
    pub suggested_cure_or_inquiry: String,
    pub citation_anchor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueStressTestResult {
    pub issue_id: String,
    pub issue_title: String,
    pub asserted_by: String,
    pub vulnerabilities: Vec<StressTestVulnerability>,
    pub overall_risk_rating: String,
    pub procedural_posture_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DossierExhibitItem {
    pub letter: String,
    pub description: String,
    pub filename: String,
    pub sha256_short: String,
    pub tier: String,
    pub relevancy_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsultationDossier {
    pub case_id: String,
    pub title: String,
    pub jurisdiction: String,
    pub docket_number: String,
    pub executive_summary: String,
    pub timeline_highlights: Vec<String>,
    pub issue_stress_tests: Vec<IssueStressTestResult>,
    pub exhibits_inventory: Vec<DossierExhibitItem>,
    pub targeted_questions_for_counsel: Vec<String>,
    pub rendered_markdown: String,
    pub exported_docx_path: Option<String>,
    pub exported_pdf_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExtensionRoute {
    ConsentStipulationLetter,
    FormalMotionForEnlargement,
    ProhibitedByRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionProposal {
    pub case_id: String,
    pub route: ExtensionRoute,
    pub governing_rule: String,
    pub maximum_days_allowed: u32,
    pub requested_days: u32,
    pub jurisdiction: String,
    pub consent_obtained: bool,
    pub good_cause_required: bool,
    pub non_alterable_warning: Option<String>,
    pub rendered_document_text: String,
    pub recommended_form_number: Option<String>,
    pub step_instructions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineExtensionAssessment {
    pub case_id: String,
    pub is_extension_recommended: bool,
    pub urgency_level: String,
    pub missing_elements: Vec<String>,
    pub recommended_target: String,
    pub recommended_days: u32,
    pub governing_rule: String,
    pub recommended_form_number: Option<String>,
    pub good_cause_statement: String,
    pub conferral_letter_draft: String,
    pub formal_motion_draft: String,
    pub conferral_instructions: Vec<String>,
    pub filing_instructions: Vec<String>,
}

