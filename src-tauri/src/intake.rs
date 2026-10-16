//! Intake & Interview State Machine, Consolidated Case Context, Offline PDF Ingestion,
//! Cross-Venue Case Escalation, Global Filer Profile, and Portfolio Deadline Radar.

use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::case_profile::{
    self, create_case_folder, default_cases_root, load_case, save_case, CaseMode, CaseProfile,
    Filer, Party, TimelineEvent,
};
use crate::check::NEVER_FILE_NOTICE;
use crate::exhibits::{self, ExhibitIndex};
use crate::guided;
use crate::legal_os::{
    check_venue_competence, predict_next_steps, resolve_venue, NextStepsReport,
    VenueCompetenceReport,
};
use crate::pdf_tools;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScannedFileSummary {
    pub folder: String,
    pub filename: String,
    pub page_count: usize,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CandidateValue {
    pub value: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct IntakeScanReport {
    pub scanned_files: Vec<ScannedFileSummary>,
    pub candidate_dockets: Vec<CandidateValue>,
    pub candidate_dates: Vec<CandidateValue>,
    pub candidate_courts: Vec<CandidateValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntakeQuestion {
    pub id: String,
    pub step_number: usize,
    pub category: String, // "profile" | "venue" | "parties" | "draft_section"
    pub prompt: String,
    pub why: String,
    pub current_value: String,
    pub answered: bool,
    pub choices: Vec<String>,
    pub candidates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntakeStatusReport {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub case_id: String,
    pub case_title: String,
    pub matter_tier: String,
    pub document_kind: String,
    pub venue_code: String,
    pub completed_steps: usize,
    pub total_steps: usize,
    pub completion_percent: u32,
    pub next_question: Option<IntakeQuestion>,
    pub questions: Vec<IntakeQuestion>,
    pub scan: IntakeScanReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseContextBundle {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub notice: String,
    pub case_dir: String,
    pub profile: CaseProfile,
    pub venue_competence: VenueCompetenceReport,
    pub next_steps: NextStepsReport,
    pub intake: IntakeStatusReport,
    pub exhibits: ExhibitIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EscalationResult {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub parent_case_id: String,
    pub new_case_dir: String,
    pub new_profile: CaseProfile,
    pub copied_record_files: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortfolioCaseItem {
    pub case_id: String,
    pub title: String,
    pub matter_tier: String,
    pub document_kind: String,
    pub venue_code: String,
    pub docket_number: String,
    pub completion_percent: u32,
    pub next_question_prompt: Option<String>,
    pub urgent_branch_title: Option<String>,
    pub urgent_deadline: Option<String>,
    pub days_until_due: Option<i64>,
    pub parent_matter_id: Option<String>,
    /// "needs_trigger_date" | "active" | "expired" | "paused_tolled" | "action_ready"
    #[serde(default)]
    pub urgent_status: Option<String>,
    #[serde(default)]
    pub urgent_rule: Option<String>,
    #[serde(default)]
    pub urgent_trigger_date: Option<String>,
    #[serde(default)]
    pub urgent_math: Option<String>,
    #[serde(default)]
    pub unverified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortfolioStatusReport {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub total_matters: usize,
    pub global_filer: Filer,
    pub matters: Vec<PortfolioCaseItem>,
}

/// Path to the user's global filer profile (`Documents/JustLegal/filer.json`).
pub fn global_filer_path() -> PathBuf {
    default_cases_root()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("filer.json")
}

pub fn load_global_filer() -> Filer {
    let p = global_filer_path();
    if p.is_file() {
        if let Ok(text) = fs::read_to_string(&p) {
            if let Ok(filer) = serde_json::from_str::<Filer>(&text) {
                return filer;
            }
        }
    }
    Filer::default()
}

pub fn save_global_filer(filer: &Filer) -> Result<Filer, String> {
    let p = global_filer_path();
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(filer).map_err(|e| e.to_string())?;
    fs::write(&p, json).map_err(|e| e.to_string())?;
    Ok(filer.clone())
}

/// Scan `orders/`, `evidence/`, `filings/`, and `drafts/` for PDFs and text files,
/// extracting text offline and detecting candidate dates, dockets, and courts with provenance.
pub fn intake_scan(case_dir: &Path) -> Result<IntakeScanReport, String> {
    let mut report = IntakeScanReport::default();
    let docket_re = Regex::new(
        r"\b(?:\d+:\d{2}-(?i:cv|cr|mc)-\d{3,6}|[A-Z]{1,3}-\d{3,6}-\d{2}|\d{2}-\d{4,6}|SC\d{4,7}|[0-9]{2}SA[0-9]{2,5})\b",
    )
    .unwrap();
    let iso_date_re = Regex::new(r"\b(20\d{2}-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12]\d|3[01]))\b").unwrap();
    let court_re = Regex::new(
        r"(?i)\b((?:SUPERIOR|SUPREME|DISTRICT|COUNTY|MUNICIPAL|APPELLATE|SURROGATE)\s+COURT[^\n,;]{0,65})",
    )
    .unwrap();

    for folder in ["orders", "evidence", "filings", "drafts"] {
        let dir = case_dir.join(folder);
        if !dir.is_dir() {
            continue;
        }
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if filename.ends_with(".json") || filename.starts_with('.') {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            let (page_count, extracted) = if ext == "pdf" {
                match pdf_tools::extract_pdf_text(&path) {
                    Ok(res) => (res.page_count, res.full_text),
                    Err(_) => (1, String::new()),
                }
            } else if ext == "md" || ext == "txt" {
                let txt = fs::read_to_string(&path).unwrap_or_default();
                (1, txt)
            } else {
                continue;
            };

            let source_tag = format!("{folder}/{filename}");
            for cap in docket_re.find_iter(&extracted) {
                let val = cap.as_str().trim().to_string();
                if !report.candidate_dockets.iter().any(|c| c.value == val) {
                    report.candidate_dockets.push(CandidateValue {
                        value: val,
                        source: source_tag.clone(),
                    });
                }
            }
            // Also check filename for docket patterns
            for cap in docket_re.find_iter(&filename) {
                let val = cap.as_str().trim().to_string();
                if !report.candidate_dockets.iter().any(|c| c.value == val) {
                    report.candidate_dockets.push(CandidateValue {
                        value: val,
                        source: source_tag.clone(),
                    });
                }
            }

            for cap in iso_date_re.find_iter(&extracted) {
                let val = cap.as_str().trim().to_string();
                if !report.candidate_dates.iter().any(|c| c.value == val) {
                    report.candidate_dates.push(CandidateValue {
                        value: val,
                        source: source_tag.clone(),
                    });
                }
            }

            for cap in court_re.find_iter(&extracted) {
                let val = cap.as_str().split_whitespace().collect::<Vec<_>>().join(" ");
                if !report.candidate_courts.iter().any(|c| c.value.eq_ignore_ascii_case(&val)) {
                    report.candidate_courts.push(CandidateValue {
                        value: val,
                        source: source_tag.clone(),
                    });
                }
            }

            let snippet: String = extracted
                .chars()
                .take(320)
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");

            report.scanned_files.push(ScannedFileSummary {
                folder: folder.to_string(),
                filename,
                page_count,
                snippet,
            });
        }
    }

    Ok(report)
}

/// Build the 1-question-at-a-time Guided Intake state for a case folder.
pub fn intake_status(case_dir: &Path) -> Result<IntakeStatusReport, String> {
    let profile = load_case(case_dir)?;
    let scan = intake_scan(case_dir).unwrap_or_default();
    let global_filer = load_global_filer();
    let tier = profile.effective_tier().to_string();
    let venue = resolve_venue(&profile);
    let kind = if profile.document_kind.trim().is_empty() {
        "reconsideration".to_string()
    } else {
        profile.document_kind.clone()
    };

    let mut questions = Vec::new();
    let mut step = 1usize;

    // Question 1: Matter Tier & Instrument Kind
    questions.push(IntakeQuestion {
        id: "matter_tier".into(),
        step_number: step,
        category: "profile".into(),
        prompt: "Which tier of legal matter are we working on?".into(),
        why: "OpenSeason adapts validation and headers across Tier 1 (Wills/Contracts/Leases), Tier 2 (Pre-Suit Demands/Cure Notices), and Tier 3 (Court Litigation/Appeals).".into(),
        current_value: tier.clone(),
        answered: !profile.matter_tier.trim().is_empty() || !profile.document_kind.trim().is_empty(),
        choices: vec![
            "litigation".into(),
            "administrative".into(),
            "transactional".into(),
        ],
        candidates: vec![tier.clone()],
    });
    step += 1;

    // Question 2: Specific Document / Instrument Type
    let kind_choices = match tier.as_str() {
        "transactional" => vec![
            "will".into(),
            "lease".into(),
            "contractor".into(),
            "power_of_attorney".into(),
        ],
        "administrative" => vec![
            "demand_letter".into(),
            "cure_notice".into(),
            "security_deposit_demand".into(),
            "records_request".into(),
            "agency_appeal".into(),
        ],
        _ => vec![
            "reconsideration".into(),
            "notice_of_appeal".into(),
            "stay".into(),
            "enlargement".into(),
            "complaint".into(),
            "mandamus".into(),
            "appellate_brief".into(),
        ],
    };
    questions.push(IntakeQuestion {
        id: "document_kind".into(),
        step_number: step,
        category: "profile".into(),
        prompt: "Which specific document or instrument are we preparing?".into(),
        why: "Selects the exact guided sections, statutory checks, and court/instrument layout.".into(),
        current_value: kind.clone(),
        answered: !profile.document_kind.trim().is_empty(),
        choices: kind_choices,
        candidates: vec![kind.clone()],
    });
    step += 1;

    // Question 3: Governing State / Court Venue
    let court_or_state_val = if tier == "litigation" {
        profile.court.name.clone()
    } else {
        profile.governing_state.clone()
    };
    questions.push(IntakeQuestion {
        id: if tier == "litigation" { "court.name".into() } else { "governing_state".into() },
        step_number: step,
        category: "venue".into(),
        prompt: if tier == "litigation" {
            "What is the exact name of the Court (and Division)?".into()
        } else {
            "Which state's laws govern this instrument or notice? (e.g., NJ, CO)".into()
        },
        why: "Enforces state-specific oath formulas (NJ R. 1:4-4(b) vs CO C.R.S. § 13-27-106 vs 28 U.S.C. § 1746) and statutory deadlines.".into(),
        current_value: court_or_state_val.clone(),
        answered: !court_or_state_val.trim().is_empty(),
        choices: if tier == "litigation" {
            vec![
                "Superior Court of New Jersey, Appellate Division".into(),
                "Superior Court of New Jersey, Law Division".into(),
                "Superior Court of New Jersey, Chancery Division — Probate Part".into(),
                "Supreme Court of Colorado".into(),
                "District Court, State of Colorado".into(),
                "United States District Court".into(),
                "Supreme Court of the United States".into(),
            ]
        } else {
            vec!["NJ".into(), "CO".into(), "FEDERAL".into()]
        },
        candidates: scan.candidate_courts.iter().map(|c| c.value.clone()).collect(),
    });
    step += 1;

    // Question 4: Docket Number (only for Litigation)
    if tier == "litigation" {
        let has_docket = !profile.docket_number.trim().is_empty() || profile.is_pre_filing();
        questions.push(IntakeQuestion {
            id: "docket_number".into(),
            step_number: step,
            category: "profile".into(),
            prompt: "What is the case Docket / Case Number? (Or answer 'PRE_FILING' if filing a new complaint)".into(),
            why: "Placed in the running court caption. If the clerk has not assigned a number yet, PRE_FILING prints 'Docket No. __________ (To Be Assigned)'.".into(),
            current_value: if profile.is_pre_filing() && profile.docket_number.is_empty() {
                "PRE_FILING".into()
            } else {
                profile.docket_number.clone()
            },
            answered: has_docket,
            choices: vec!["PRE_FILING".into()],
            candidates: scan.candidate_dockets.iter().map(|c| c.value.clone()).collect(),
        });
        step += 1;
    }

    // Question 5: Primary Party (Plaintiff / Appellant / Testator / Landlord / Claimant)
    let primary_party = profile
        .caption
        .appellants()
        .first()
        .map(|p| p.name.clone())
        .unwrap_or_default();
    let mut party_candidates = Vec::new();
    if !global_filer.name.trim().is_empty() {
        party_candidates.push(global_filer.name.clone());
    }
    questions.push(IntakeQuestion {
        id: "primary_party".into(),
        step_number: step,
        category: "parties".into(),
        prompt: match tier.as_str() {
            "transactional" => "Who is the First Party (Testator, Landlord, or Client)?".into(),
            "administrative" => "Who is the Claimant / Sender of this notice?".into(),
            _ => "Who is the Plaintiff / Appellant / Petitioner?".into(),
        },
        why: "Identifies the moving or declaring party.".into(),
        current_value: primary_party.clone(),
        answered: !primary_party.trim().is_empty(),
        choices: vec![],
        candidates: party_candidates.clone(),
    });
    step += 1;

    // Question 6: Counterparty / Fiduciary (Defendant / Appellee / Executor / Tenant / Respondent)
    let counterparty = profile
        .caption
        .appellees()
        .first()
        .map(|p| p.name.clone())
        .unwrap_or_default();
    questions.push(IntakeQuestion {
        id: "counterparty".into(),
        step_number: step,
        category: "parties".into(),
        prompt: match (tier.as_str(), kind.as_str()) {
            ("transactional", "will") => "Who are you naming as Executor / Personal Representative?".into(),
            ("transactional", "power_of_attorney") => "Who are you naming as your Agent (Attorney-in-Fact)?".into(),
            ("transactional", _) => "Who is the Counterparty (Tenant or Contractor)?".into(),
            ("administrative", _) => "Who is the Respondent / Recipient of this notice?".into(),
            _ => "Who is the Defendant / Appellee / Respondent?".into(),
        },
        why: "Identifies the opposing party, recipient, or fiduciary.".into(),
        current_value: counterparty.clone(),
        answered: !counterparty.trim().is_empty(),
        choices: vec![],
        candidates: vec![],
    });
    step += 1;

    // Question 7: Filer / Signer Identity
    questions.push(IntakeQuestion {
        id: "filer.name".into(),
        step_number: step,
        category: "parties".into(),
        prompt: "What is the full legal name of the person signing this document?".into(),
        why: "Used for the signature block, certification/declaration under penalty of perjury, and Certificate of Service.".into(),
        current_value: profile.filer.name.clone(),
        answered: !profile.filer.name.trim().is_empty() && !profile.filer.signature_name.trim().is_empty(),
        choices: vec![],
        candidates: party_candidates,
    });
    step += 1;

    // Question 8: Colorado Rule 121 Conferral (when venue == CO and filing a motion)
    if venue.venue_code == "CO" && tier == "litigation" {
        questions.push(IntakeQuestion {
            id: "conferral_status".into(),
            step_number: step,
            category: "venue".into(),
            prompt: "Colorado C.R.C.P. 121 § 1-15(8) Conferral: Did you confer with the opposing party before filing?".into(),
            why: "Colorado courts require every motion to begin with a Certificate of Conferral stating whether the relief is opposed.".into(),
            current_value: profile.conferral_status.clone(),
            answered: !profile.conferral_status.trim().is_empty(),
            choices: vec![
                "Conferred in good faith; opposing party opposes the relief requested.".into(),
                "Conferred in good faith; opposing party does not oppose this motion.".into(),
                "Attempted to confer in good faith via email/phone; no response received prior to filing.".into(),
                "Exempt from conferral under C.R.C.P. 121 § 1-15(8) (unrepresented / pro se prisoner or dispositive rule).".into(),
            ],
            candidates: vec![],
        });
        step += 1;
    }

    // Question 9+: Guided Draft Sections for the active document kind
    if let Ok(draft) = guided::load_or_init(case_dir, &kind) {
        for sec in draft.sections {
            questions.push(IntakeQuestion {
                id: format!("section.{}", sec.id),
                step_number: step,
                category: "draft_section".into(),
                prompt: format!("{}: {}", sec.heading, sec.hint),
                why: "Your own words only. OpenSeason formats your statement into the final instrument without inventing facts.".into(),
                current_value: sec.user_text.clone(),
                answered: !sec.user_text.trim().is_empty(),
                choices: vec![],
                candidates: scan.candidate_dates.iter().map(|d| d.value.clone()).collect(),
            });
            step += 1;
        }
    }

    let total_steps = questions.len();
    let completed_steps = questions.iter().filter(|q| q.answered).count();
    let completion_percent = if total_steps == 0 {
        100
    } else {
        ((completed_steps as f64 / total_steps as f64) * 100.0).round() as u32
    };
    let next_question = questions.iter().find(|q| !q.answered).cloned();

    Ok(IntakeStatusReport {
        ok: true,
        never_files: true,
        never_serves: true,
        case_id: profile.id,
        case_title: profile.title,
        matter_tier: tier,
        document_kind: kind,
        venue_code: venue.venue_code,
        completed_steps,
        total_steps,
        completion_percent,
        next_question,
        questions,
        scan,
    })
}

/// Answer one question in the Guided Intake state machine and advance to the next step.
pub fn intake_answer(
    case_dir: &Path,
    question_id: &str,
    answer: &str,
) -> Result<IntakeStatusReport, String> {
    let mut profile = load_case(case_dir)?;
    let val = answer.trim().to_string();
    let qid = question_id.trim();

    if let Some(sec_id) = qid.strip_prefix("section.") {
        guided::set_section(case_dir, sec_id, &val)?;
        return intake_status(case_dir);
    }

    match qid {
        "matter_tier" => {
            profile.matter_tier = val.to_ascii_lowercase();
        }
        "document_kind" => {
            profile.document_kind = val.to_ascii_lowercase();
            profile.matter_tier = profile.effective_tier().to_string();
            let _ = guided::load_or_init(case_dir, &profile.document_kind);
        }
        "court.name" => {
            profile.court.name = val.clone();
            let lower = val.to_ascii_lowercase();
            if lower.contains("new jersey") || lower.contains("surrogate") {
                profile.governing_state = "NJ".into();
                profile.rule_pack_id = "nj-app-div".into();
            } else if lower.contains("colorado") {
                profile.governing_state = "CO".into();
                profile.rule_pack_id = "co-mandamus".into();
            } else if lower.contains("supreme court of the united states") {
                profile.governing_state = "SCOTUS".into();
            } else if lower.contains("united states") {
                profile.governing_state = "FEDERAL".into();
            }
        }
        "governing_state" => {
            profile.governing_state = val.to_uppercase();
            if profile.governing_state == "NJ" {
                profile.rule_pack_id = "nj-app-div".into();
            } else if profile.governing_state == "CO" {
                profile.rule_pack_id = "co-mandamus".into();
            }
        }
        "docket_number" => {
            if val.eq_ignore_ascii_case("PRE_FILING") || val.eq_ignore_ascii_case("TO_BE_ASSIGNED") {
                profile.procedural_stage = "pre_filing".into();
                profile.docket_number.clear();
            } else {
                profile.docket_number = val;
                if profile.procedural_stage == "pre_filing" {
                    profile.procedural_stage = "active_docket".into();
                }
            }
        }
        "primary_party" => {
            let party = Party {
                name: val.clone(),
                role: match profile.effective_tier() {
                    "transactional" => "First Party / Declarant".into(),
                    "administrative" => "Claimant".into(),
                    _ => "Appellant / Plaintiff".into(),
                },
                counsel: "Pro se".into(),
                ..Default::default()
            };
            profile.caption.appellants = vec![party.clone()];
            profile.caption.plaintiffs = vec![party];
        }
        "counterparty" => {
            let party = Party {
                name: val,
                role: match profile.effective_tier() {
                    "transactional" => "Counterparty / Fiduciary".into(),
                    "administrative" => "Respondent".into(),
                    _ => "Appellee / Defendant".into(),
                },
                ..Default::default()
            };
            profile.caption.appellees = vec![party.clone()];
            profile.caption.defendants = vec![party];
        }
        "filer.name" => {
            profile.filer.name = val.clone();
            profile.filer.signature_name = val.clone();
            // If global filer has address/email/phone and case is empty, inherit them
            let gf = load_global_filer();
            if profile.filer.email.is_empty() && !gf.email.is_empty() {
                profile.filer.email = gf.email;
            }
            if profile.filer.phone.is_empty() && !gf.phone.is_empty() {
                profile.filer.phone = gf.phone;
            }
            if profile.filer.address_lines.is_empty() && !gf.address_lines.is_empty() {
                profile.filer.address_lines = gf.address_lines;
            }
        }
        "conferral_status" => {
            profile.conferral_status = val;
        }
        other => {
            return Err(format!("Unknown intake question id '{other}'."));
        }
    }

    save_case(case_dir, &profile)?;
    intake_status(case_dir)
}

/// Build the complete, consolidated Case Context Bundle in one call for AI agents and UI dashboards.
pub fn case_context(case_dir: &Path) -> Result<CaseContextBundle, String> {
    let profile = load_case(case_dir)?;
    let venue_competence = check_venue_competence(&profile);
    let next_steps = predict_next_steps(&profile, Some(case_dir));
    let intake = intake_status(case_dir)?;
    let exhibits = exhibits::rebuild(case_dir).unwrap_or(ExhibitIndex {
        never_files: true,
        items: Vec::new(),
    });

    Ok(CaseContextBundle {
        ok: true,
        never_files: true,
        never_serves: true,
        notice: NEVER_FILE_NOTICE.into(),
        case_dir: case_dir.display().to_string(),
        profile,
        venue_competence,
        next_steps,
        intake,
        exhibits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_profile::{create_case_folder, ensure_case_layout, CaseMode};
    use uuid::Uuid;

    #[test]
    fn guided_intake_walks_and_updates_tier1_will() {
        let dir = std::env::temp_dir().join(format!("os-intake-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let (case_dir, _) =
            create_case_folder(&dir, "Last Will of Jordan Example", CaseMode::Standard, "will")
                .unwrap();

        let status1 = intake_status(&case_dir).unwrap();
        assert_eq!(status1.matter_tier, "transactional");
        assert!(status1.next_question.is_some());

        let status2 = intake_answer(&case_dir, "governing_state", "CO").unwrap();
        assert_eq!(status2.venue_code, "CO");

        let _ = intake_answer(&case_dir, "primary_party", "Jordan Example").unwrap();
        let _ = intake_answer(&case_dir, "counterparty", "Alex Example (Executor)").unwrap();
        let status3 = intake_answer(&case_dir, "filer.name", "Jordan Example").unwrap();
        assert!(status3.completed_steps >= 4);

        let _ = fs::remove_dir_all(dir);
    }
}

