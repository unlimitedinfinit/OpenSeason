export interface HelpLink {
  id: string;
  kind: string;
  label: string;
  description: string;
  url: string;
  verified: boolean;
}

export interface HelpLinks {
  confirm: string;
  offline_note: string;
  source_note: string;
  verified_on: string;
  opens_browser: boolean;
  links: HelpLink[];
}

export interface TeachBlock {
  what_it_is: string;
  when_used: string;
  common_mistakes: string[];
  learn_more: string;
}

export interface FormEntry {
  id: string;
  title: string;
  form_number: string;
  state: string;
  court: string;
  situation: string;
  size_bytes: number;
  download_url: string;
  official_url: string;
  sha256: string;
  citation: string;
  citation_status: string;
  teach: TeachBlock;
}

export interface SituationStep {
  id: string;
  title: string;
  why: string;
  form_ids: string[];
  citation: string;
  citation_status: string;
  teach: TeachBlock;
}

export interface GuideChoice {
  id: string;
  label: string;
  detail: string;
}

export interface GuideQuestion {
  id: string;
  prompt: string;
  why: string;
  choices: GuideChoice[];
}

export interface GuideQuestions {
  disclaimer: string;
  legal_information_not_advice: boolean;
  questions: GuideQuestion[];
  help_links: HelpLink[];
  confirm_before_open: string;
}

export interface GuideResult {
  disclaimer: string;
  never_files: boolean;
  legal_information_not_advice: boolean;
  state: string;
  court: string;
  situation: string;
  situation_label: string;
  pack_id: string;
  why: string;
  forms: FormEntry[];
  steps: SituationStep[];
  deadline: { due: string; rule: string; math: string } | null;
  citation: string;
  citation_status: string;
  unverified: string[];
  stuck: boolean;
  teach: TeachBlock;
  help_links: HelpLink[];
  confirm_before_open: string;
  notice: string;
}

export interface PackListItem {
  id: string;
  state: string;
  court: string;
  title: string;
  situations: string[];
  size_bytes: number;
  size_label: string;
  sha256: string;
  installed: boolean;
  install_path?: string | null;
  rule_pack_id: string;
  source_note: string;
}

export interface PackList {
  packs_root: string;
  catalog_id: string;
  notice: string;
  available: PackListItem[];
  installed: PackListItem[];
  rule_packs: { id: string; name: string; jurisdiction: string; forms: number; situations: number }[];
}

export interface InstallReport {
  ok: boolean;
  pack_id: string;
  state: string;
  bytes_planned: number;
  size_label: string;
  dest: string;
  files: string[];
  sha256_ok: boolean;
  offline_after: boolean;
  notice: string;
}
