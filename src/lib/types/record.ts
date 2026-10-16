export type EvidentiaryTier =
  | "official_court_record"
  | "hearing_and_evidentiary_record"
  | "personal_case_archive"
  | "potential_additional_evidence";

export interface DocumentProvenance {
  origin: string;
  original_path?: string | null;
  version: number;
  submission_channel?: string | null;
  submission_date?: string | null;
  submission_status?: string | null;
  rejection_reason?: string | null;
  accepted_date?: string | null;
  docket_sequence?: number | null;
  docket_code?: string | null;
  filing_stamp_verified: boolean;
  served_recipients: string[];
  service_method?: string | null;
  service_date?: string | null;
  certificate_of_service_ref?: string | null;
  evidentiary_tier: EvidentiaryTier;
  completeness: string;
}

export interface RecordEntry {
  id: string;
  filename: string;
  relative_path: string;
  sha256: string;
  size_bytes: number;
  tier: EvidentiaryTier;
  docket_date?: string | null;
  filing_stamp_detected: boolean;
  source_provenance: string;
  title: string;
  notes: string;
  verified: boolean;
  provenance?: DocumentProvenance;
}


export interface MissingRecordItem {
  id: string;
  title: string;
  record_type: "transcript" | "order" | "opposition" | "exhibit" | "docket_sheet" | string;
  reason: string;
  docket_entry_ref?: string | null;
  recommended_action: string;
  estimated_cost_notice: string;
  request_form?: string | null;
}

export interface DocketEntry {
  sequence_num?: number | null;
  date: string;
  title: string;
  description: string;
  filed_by?: string | null;
  matched_record_id?: string | null;
}

export interface RecordInventoryReport {
  case_id: string;
  court_name: string;
  docket_number: string;
  total_files_scanned: number;
  tier1_official_count: number;
  tier2_hearing_count: number;
  tier3_archive_count: number;
  tier4_unsubmitted_count: number;
  entries: RecordEntry[];
  missing_records: MissingRecordItem[];
  reconciliation_summary: string;
}

export interface CourtRecordsRequestPacket {
  state: string;
  court_name: string;
  division: string;
  docket_number: string;
  caption: string;
  requester_name: string;
  requester_address: string;
  requester_phone: string;
  requester_email: string;
  records_requested: string[];
  preferred_format: string;
  statutory_fee_notice: string;
  submission_method: string;
  instructions: string;
  rendered_request_text: string;
}

export type RecordSupportStatus =
  | "unsupported_in_record"
  | "contradicted_by_record"
  | "unverified_oral_colloquy"
  | "verified_in_record";

export interface FindingSource {
  document_id: string;
  document_name: string;
  page?: number | null;
  quoted_text: string;
}

export type RouteKind =
  | "reconsideration_interlocutory"
  | "reconsideration_final"
  | "appellate_review"
  | "record_correction"
  | "judicial_conduct_grievance"
  | "attorney_ethics_grievance"
  | "public_records_request"
  | "ada_accommodation";

export interface AccountabilityRoute {
  route_kind: RouteKind;
  title: string;
  rule_citation: string;
  governing_standard: string;
  deadline_notice?: string | null;
  is_disciplinary: boolean;
  warning_notice: string;
  official_form_name?: string | null;
  submission_portal_url?: string | null;
}

export interface IssueItem {
  id: string;
  title: string;
  date_identified: string;
  finding_source: FindingSource;
  record_support_status: RecordSupportStatus;
  contradicting_evidence_local: string[];
  contradicting_evidence_filed: string[];
  analysis_notes: string;
  available_routes: AccountabilityRoute[];
  resolved: boolean;
}

export interface DiscrepancyReport {
  case_id: string;
  court_name: string;
  issues: IssueItem[];
  non_tolling_warning: string;
  summary: string;
}

export interface AccountabilityResource {
  id: string;
  title: string;
  agency: string;
  category: "judicial" | "attorney" | "records" | "ada" | string;
  state: string;
  form_number?: string | null;
  direct_pdf_url?: string | null;
  portal_url: string;
  fee_schedule: string;
  jurisdictional_scope: string;
  non_appeal_warning: string;
}

export interface FactualPacketReport {
  case_id: string;
  title: string;
  issue_id: string;
  date_generated: string;
  factual_chronology: string[];
  record_provenance: string[];
  rendered_markdown: string;
  notice: string;
}

export type AccommodationCategory =
  | "remote_telephonic_appearance"
  | "physical_mobility_access"
  | "visual_hearing_assistance"
  | "extended_reading_time"
  | "support_person_accompaniment"
  | "document_magnification_accessible_pdf"
  | "communication_interpreter";

export interface AdaRequestPayload {
  case_id: string;
  requester_name: string;
  requester_role: string;
  contact_phone: string;
  contact_email: string;
  accommodations: AccommodationCategory[];
  specific_needs_narrative: string;
  upcoming_date?: string | null;
}

export interface AdaPetitionReport {
  state: string;
  court_name: string;
  docket_number: string;
  ada_coordinator_title: string;
  statutory_authorities: string[];
  rendered_petition_text: string;
  submission_guidelines: string;
}

export const TIER_LABELS: Record<EvidentiaryTier, { label: string; desc: string; badge: string; color: string }> = {
  official_court_record: {
    label: "Tier 1: Official Court Record",
    desc: "Docketed, filed, and served documents bearing clerk stamps or filing confirmation.",
    badge: "Official Record",
    color: "bg-emerald-500/10 text-emerald-700 dark:text-emerald-400 border-emerald-500/30",
  },
  hearing_and_evidentiary_record: {
    label: "Tier 2: Hearing / Transcript Record",
    desc: "Certified hearing transcripts, sound recordings, and admitted trial exhibits.",
    badge: "Certified Record",
    color: "bg-blue-500/10 text-blue-700 dark:text-blue-400 border-blue-500/30",
  },
  personal_case_archive: {
    label: "Tier 3: Personal Case Archive",
    desc: "Internal drafts, correspondence, unserved letters, and working files.",
    badge: "Internal Archive",
    color: "bg-amber-500/10 text-amber-700 dark:text-amber-400 border-amber-500/30",
  },
  potential_additional_evidence: {
    label: "Tier 4: Potential Additional Evidence (Unsubmitted)",
    desc: "Privately held evidence not yet submitted or admitted below. Cannot be cited on appeal without formal record supplementation motion.",
    badge: "Unsubmitted / Extrinsic",
    color: "bg-rose-500/10 text-rose-700 dark:text-rose-400 border-rose-500/30",
  },
};
