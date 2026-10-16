export type CaseMode = "standard" | "confidential";

export interface Party {
  name: string;
  role: string;
  address?: string;
  email?: string;
  phone?: string;
  counsel?: string;
}

export interface Caption {
  plaintiffs: Party[];
  defendants: Party[];
  appellants: Party[];
  appellees: Party[];
}

export interface CourtInfo {
  id: string;
  name: string;
  division?: string;
}

export interface Filer {
  name: string;
  role: string;
  address_lines: string[];
  phone: string;
  email: string;
  signature_name: string;
}

export interface NoticeOfAppealDraft {
  judgment_date: string;
  judgment_description: string;
  trial_court_name: string;
  trial_court_docket: string;
  user_text: string;
}

export interface PleadingDraft {
  title: string;
  user_text: string;
}

export interface TimelineEvent {
  id: string;
  title: string;
  date: string;
  notes: string;
}

export interface CaseProfile {
  schema_version: number;
  id: string;
  title: string;
  mode: CaseMode;
  document_kind: string;
  created_at: string;
  updated_at: string;
  sealed_at?: string | null;
  court: CourtInfo;
  docket_number: string;
  caption: Caption;
  filer: Filer;
  notice_of_appeal: NoticeOfAppealDraft;
  complaint: PleadingDraft;
  motion: PleadingDraft;
  timeline: TimelineEvent[];
  rule_pack_id?: string;
  motion_kind?: string;
  brief: PleadingDraft;
  appellate_motion: PleadingDraft;
  mandamus: PleadingDraft;
  court_markdown?: string;
  cloud_matter_id?: string | null;
  cloud_synced_at?: string | null;
  filings?: CaseFilingItem[];
  matter_tier?: string;
  procedural_stage?: string;
  governing_state?: string;
  parent_matter_id?: string | null;
  conferral_status?: string;
  service_method?: string;
}

export interface CaseFilingItem {
  id: string;
  title: string;
  kind: string;
  rule: string;
  status: string;
  user_text: string;
  relief_sought: string;
  created_at: string;
  updated_at: string;
}

export interface WorkspaceCase {
  profile: CaseProfile;
  source: string;
  has_vault: boolean;
  sealed: boolean;
  folder: string;
}

export interface CaseArtifact {
  kind: string;
  name: string;
  path: string;
}

export interface GuidedSection {
  id: string;
  heading: string;
  hint: string;
  user_text: string;
}

export interface GuidedDraft {
  kind: string;
  sections: GuidedSection[];
}

export interface ExhibitRow {
  letter: string;
  filename: string;
  description: string;
  path: string;
}

export interface ValidationIssue {
  field: string;
  message: string;
}

export interface ValidationReport {
  ok: boolean;
  issues: ValidationIssue[];
}

export interface SyncRefusal {
  action: string;
  mode: string;
  reason: string;
  kind: string;
}

export interface ExportResult {
  docx: string;
  pdf: string;
  notice: string;
}

export const REVIEW_NOTICE =
  "This document was assembled by OpenSeason from the user's own case profile and draft text. It is a formatting aid, not legal advice. Review every line, caption, date, and citation before filing. Consider consulting a licensed attorney.";

export const PARTY_ROLES = [
  "Plaintiff",
  "Defendant",
  "Appellant",
  "Appellee",
  "Petitioner",
  "Respondent",
  "Intervenor",
  "Witness",
] as const;

export function emptyProfile(title = "", kind = "notice_of_appeal"): CaseProfile {
  const now = new Date().toISOString();
  return {
    schema_version: 1,
    id: crypto.randomUUID(),
    title,
    mode: "standard",
    document_kind: kind,
    created_at: now,
    updated_at: now,
    sealed_at: null,
    court: { id: "generic-us-appellate", name: "", division: "" },
    docket_number: "",
    caption: { plaintiffs: [], defendants: [], appellants: [], appellees: [] },
    filer: {
      name: "",
      role: "Proceeding without an attorney",
      address_lines: [],
      phone: "",
      email: "",
      signature_name: "",
    },
    notice_of_appeal: {
      judgment_date: "",
      judgment_description: "",
      trial_court_name: "",
      trial_court_docket: "",
      user_text: "",
    },
    complaint: { title: "", user_text: "" },
    motion: { title: "", user_text: "" },
    timeline: [],
    rule_pack_id: "generic-us-appellate",
    motion_kind: "",
    brief: { title: "", user_text: "" },
    appellate_motion: { title: "", user_text: "" },
    mandamus: { title: "", user_text: "" },
    court_markdown: "",
  };
}

export function sampleProfile(): CaseProfile {
  const p = emptyProfile("Example v. Sample County Clerk", "notice_of_appeal");
  p.id = "00000000-0000-4000-8000-000000000001";
  p.court.name = "United States Court of Appeals for the Sample Circuit";
  p.docket_number = "24-01000";
  p.caption.appellants = [
    {
      name: "Jordan Example",
      role: "Appellant",
      address: "100 Sample Street, Example City, ST 00000",
      email: "jordan.example@example.test",
      phone: "555-0100",
      counsel: "Pro se",
    },
  ];
  p.caption.plaintiffs = p.caption.appellants;
  p.caption.appellees = [
    {
      name: "Sample County Clerk",
      role: "Appellee",
      address: "1 Courthouse Square, Example City, ST 00000",
      counsel: "Counsel of record",
    },
  ];
  p.caption.defendants = p.caption.appellees;
  p.filer = {
    name: "Jordan Example",
    role: "Appellant, proceeding without an attorney",
    address_lines: ["100 Sample Street", "Example City, ST 00000"],
    phone: "555-0100",
    email: "jordan.example@example.test",
    signature_name: "Jordan Example",
  };
  p.notice_of_appeal = {
    judgment_date: "2024-03-01",
    judgment_description: "Order granting the motion to dismiss",
    trial_court_name: "United States District Court for the Sample District",
    trial_court_docket: "1:23-cv-01000",
    user_text:
      "Appellant appeals from the order entered on March 1, 2024, granting the motion to dismiss. This paragraph is the user's own draft. OpenSeason did not write the grounds of appeal.",
  };
  p.complaint = {
    title: "Complaint for declaratory and injunctive relief",
    user_text:
      "This paragraph is the user's own statement of the claim. OpenSeason did not invent the facts or the legal theory.",
  };
  p.motion = {
    title: "Motion for an extension of time",
    user_text:
      "This paragraph is the user's own motion text. OpenSeason did not invent the request or the grounds.",
  };
  p.timeline = [
    {
      id: "t1",
      title: "Order entered",
      date: "2024-03-01",
      notes: "Trial court granted the motion to dismiss.",
    },
  ];
  return p;
}

// ---------------------------------------------------------------------------
// Cloud Sync & Engine Upgrades Types
// ---------------------------------------------------------------------------

export interface CloudSession {
  id_token: string;
  refresh_token: string;
  uid: string;
  email: string;
  expires_at: number;
}

export interface SyncFileRecord {
  doc_id: string;
  title: string;
  filename: string;
  subfolder: string;
  sha256: string;
  size_bytes: number;
  synced_at: string;
}

export interface CaseSyncMetadata {
  cloud_matter_id: string;
  last_synced_at: string;
  files: SyncFileRecord[];
}

export interface PullReport {
  ok: boolean;
  cases_pulled: number;
  cases_updated: number;
  files_downloaded: number;
  issues: string[];
}

export interface PushReport {
  ok: boolean;
  cloud_matter_id: string;
  files_uploaded: number;
  message: string;
}

export interface PreFlightAudit {
  ok: boolean;
  never_files: boolean;
  never_serves: boolean;
  caption_ok: boolean;
  caption_summary: string;
  docket_ok: boolean;
  docket_summary: string;
  filer_ok: boolean;
  filer_summary: string;
  deadline_ok: boolean;
  deadline_summary: string;
  facts_ok: boolean;
  facts_verified_count: number;
  facts_total_count: number;
  authorities_ok: boolean;
  authorities_verified_count: number;
  authorities_total_count: number;
  transcripts_ok: boolean;
  packet_parts_ready: boolean;
  missing_requirements: string[];
  notice: string;
}

export interface ExhibitPacketItem {
  letter: string;
  description: string;
  filename: string;
  bates_start: string;
  bates_end: string;
  page_count: number;
}

export interface ExhibitPacketReport {
  ok: boolean;
  never_files: boolean;
  never_serves: boolean;
  notice: string;
  table_of_exhibits_pdf: string;
  exhibit_packet_pdf: string;
  bates_start: string;
  bates_end: string;
  total_exhibits: number;
  total_pages: number;
  exhibits: ExhibitPacketItem[];
}

export interface FormFieldInfo {
  key: string;
  label: string;
  field_type: string;
  value_from_case?: string | null;
  required: boolean;
}

export interface FormInspection {
  form_name: string;
  form_path: string;
  detected_form_number?: string | null;
  fields: FormFieldInfo[];
}

export interface FormFillResult {
  ok: boolean;
  never_files: boolean;
  form_path: string;
  output_pdf_path: string;
  output_json_path: string;
  mapped_fields: number;
  notice: string;
}

export interface TranscriptCitation {
  raw: string;
  transcript_name: string;
  page: number;
  line?: number | null;
  verified: boolean;
  quote_found?: string | null;
}

export interface TranscriptReport {
  ok: boolean;
  never_files: boolean;
  indexed_files: string[];
  total_lines: number;
  citations: TranscriptCitation[];
  issues: ValidationIssue[];
}

export interface ServiceRecipient {
  name: string;
  role: string;
  address: string;
  email: string;
}

export interface ServiceCertParams {
  filing_title: string;
  service_date: string;
  method: string;
  recipients: ServiceRecipient[];
  signature_name: string;
}

export interface ServiceCertResult {
  ok: boolean;
  never_files: boolean;
  notice: string;
  cert_pdf: string;
  cert_text: string;
}

export interface ProposedOrderResult {
  ok: boolean;
  never_files: boolean;
  notice: string;
  order_pdf: string;
  order_docx: string;
}

export interface PdfOperationResult {
  ok: boolean;
  never_files: boolean;
  operation: string;
  output_path: string;
  page_count: number;
  message: string;
}

export interface RedactOperationResult {
  ok: boolean;
  never_files: boolean;
  output_path: string;
  redactions_count: number;
  message: string;
}

export interface SignatureProfile {
  full_name: string;
  initials: string;
  style: string;
  signature_data?: string | null;
  court_s_block: string;
  updated_at: string;
}

export interface SignatureStampResult {
  ok: boolean;
  never_files: boolean;
  output_path: string;
  page_number: number;
  stamped_text: string;
  message: string;
}

// ---------------------------------------------------------------------------
// Legal Operating System (3-Tier, Multi-Venue, Predictive Graph, Intake Wizard, Portfolio)
// ---------------------------------------------------------------------------

export interface VenueProfile {
  venue_code: string;
  display_name: string;
  sworn_statement_title: string;
  sworn_statement_citation: string;
  mandatory_oath_text: string;
  requires_conferral_certificate: boolean;
  conferral_rule: string;
  appeal_days: number;
  reconsideration_days: number;
  reconsideration_trigger: string;
  tolling_mode: "pause_remaining" | "reset_full" | "none" | string;
  mail_service_adder_days: number;
  packet_manifest: string[];
}

export interface VenueCompetenceReport {
  ok: boolean;
  never_files: boolean;
  never_serves: boolean;
  matter_tier: string;
  document_kind: string;
  venue: VenueProfile;
  issues: ValidationIssue[];
  recommendations: string[];
}

export interface ProceduralBranch {
  id: string;
  title: string;
  rule_citation: string;
  trigger_event_title: string;
  trigger_date: string;
  trigger_kind: string;
  base_days: number;
  mail_offset_days: number;
  clock_interaction: string;
  tolled_by_event?: string | null;
  days_elapsed_before_toll?: number | null;
  days_remaining_after_toll?: number | null;
  computed_deadline?: string | null;
  days_until_due?: number | null;
  status: string;
  template_kind: string;
  escalation_target?: string | null;
  explanation: string;
}

export interface NextStepsReport {
  ok: boolean;
  never_files: boolean;
  never_serves: boolean;
  notice: string;
  case_id: string;
  case_title: string;
  matter_tier: string;
  venue_code: string;
  detected_triggers: string[];
  branches: ProceduralBranch[];
}

export interface ScannedFileSummary {
  folder: string;
  filename: string;
  page_count: number;
  snippet: string;
}

export interface CandidateValue {
  value: string;
  source: string;
}

export interface IntakeScanReport {
  scanned_files: ScannedFileSummary[];
  candidate_dockets: CandidateValue[];
  candidate_dates: CandidateValue[];
  candidate_courts: CandidateValue[];
}

export interface IntakeQuestion {
  id: string;
  step_number: number;
  category: string;
  prompt: string;
  why: string;
  current_value: string;
  answered: boolean;
  choices: string[];
  candidates: string[];
}

export interface IntakeStatusReport {
  ok: boolean;
  never_files: boolean;
  never_serves: boolean;
  case_id: string;
  case_title: string;
  matter_tier: string;
  document_kind: string;
  venue_code: string;
  completed_steps: number;
  total_steps: number;
  completion_percent: number;
  next_question?: IntakeQuestion | null;
  questions: IntakeQuestion[];
  scan: IntakeScanReport;
}

export interface CaseContextBundle {
  ok: boolean;
  never_files: boolean;
  never_serves: boolean;
  notice: string;
  case_dir: string;
  profile: CaseProfile;
  venue_competence: VenueCompetenceReport;
  next_steps: NextStepsReport;
  intake: IntakeStatusReport;
  exhibits: { never_files: boolean; items: ExhibitRow[] };
}

export interface EscalationResult {
  ok: boolean;
  never_files: boolean;
  never_serves: boolean;
  parent_case_id: string;
  new_case_dir: string;
  new_profile: CaseProfile;
  copied_record_files: number;
  message: string;
}

export interface PortfolioCaseItem {
  case_id: string;
  title: string;
  matter_tier: string;
  document_kind: string;
  venue_code: string;
  docket_number: string;
  completion_percent: number;
  next_question_prompt?: string | null;
  urgent_branch_title?: string | null;
  urgent_deadline?: string | null;
  days_until_due?: number | null;
  parent_matter_id?: string | null;
}

export interface PortfolioStatusReport {
  ok: boolean;
  never_files: boolean;
  never_serves: boolean;
  total_matters: number;
  global_filer: Filer;
  matters: PortfolioCaseItem[];
}

export interface PdfTextPage {
  page_number: number;
  text: string;
}

export interface PdfTextExtractionResult {
  ok: boolean;
  never_files: boolean;
  path: string;
  page_count: number;
  total_chars: number;
  pages: PdfTextPage[];
  full_text: string;
}



