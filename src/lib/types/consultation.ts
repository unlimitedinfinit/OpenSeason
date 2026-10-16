export type VulnerabilitySeverity = "Critical" | "Significant" | "Advisory";

export type VulnerabilityType =
  | "StandardOfReview"
  | "PreservationIssue"
  | "HarmlessError"
  | "MissingRecordGrounding"
  | "InterlocutoryFinality"
  | "NonTolledClock"
  | "HearsayWithoutException";

export interface StressTestVulnerability {
  vulnerability_type: VulnerabilityType;
  severity: VulnerabilitySeverity;
  title: string;
  description: string;
  opposing_argument_prediction: string;
  suggested_cure_or_inquiry: string;
  citation_anchor?: string | null;
}

export interface IssueStressTestResult {
  issue_id: string;
  issue_title: string;
  asserted_by: string;
  vulnerabilities: StressTestVulnerability[];
  overall_risk_rating: string;
  procedural_posture_summary: string;
}

export interface DossierExhibitItem {
  letter: string;
  description: string;
  filename: string;
  sha256_short: string;
  tier: string;
  relevancy_tag: string;
}

export interface ConsultationDossier {
  case_id: string;
  title: string;
  jurisdiction: string;
  docket_number: string;
  executive_summary: string;
  timeline_highlights: string[];
  issue_stress_tests: IssueStressTestResult[];
  exhibits_inventory: DossierExhibitItem[];
  targeted_questions_for_counsel: string[];
  rendered_markdown: string;
  exported_docx_path?: string | null;
  exported_pdf_path?: string | null;
}

export type ExtensionRoute =
  | "ConsentStipulationLetter"
  | "FormalMotionForEnlargement"
  | "ProhibitedByRule";

export interface ExtensionProposal {
  case_id: string;
  route: ExtensionRoute;
  governing_rule: string;
  maximum_days_allowed: number;
  requested_days: number;
  jurisdiction: string;
  consent_obtained: boolean;
  good_cause_required: boolean;
  non_alterable_warning?: string | null;
  rendered_document_text: string;
  recommended_form_number?: string | null;
  step_instructions: string[];
}

export interface BaselineExtensionAssessment {
  case_id: string;
  is_extension_recommended: boolean;
  urgency_level: string;
  missing_elements: string[];
  recommended_target: string;
  recommended_days: number;
  governing_rule: string;
  recommended_form_number?: string | null;
  good_cause_statement: string;
  conferral_letter_draft: string;
  formal_motion_draft: string;
  conferral_instructions: string[];
  filing_instructions: string[];
}

