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
