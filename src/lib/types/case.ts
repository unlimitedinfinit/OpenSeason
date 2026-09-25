export type CaseMode = "standard" | "confidential";

export interface Party {
  name: string;
  role: string;
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
  division?: string | null;
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

export const REVIEW_NOTICE =
  "This document was assembled by OpenSeason from the user's own case profile and draft text. It is a formatting aid, not legal advice. Review every line, caption, date, and citation before filing. Consider consulting a licensed attorney.";
