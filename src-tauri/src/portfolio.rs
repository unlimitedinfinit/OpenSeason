//! Cross-Venue Case Escalation, Multi-State Portfolio Deadline Radar,
//! and Tauri IPC command endpoints for the Legal Operating System.

use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

use crate::case_profile::{
    self, create_case_folder, default_cases_root, load_case, resolve_case_dir, save_case, CaseMode,
    Filer, TimelineEvent,
};
use crate::exhibits;
use crate::guided::{self, GuidedDraft};
use crate::intake::{
    case_context, intake_answer, intake_scan, intake_status, load_global_filer, save_global_filer,
    CaseContextBundle, EscalationResult, IntakeScanReport, IntakeStatusReport, PortfolioCaseItem,
    PortfolioStatusReport,
};
use crate::legal_os::{
    check_venue_competence, predict_next_steps, NextStepsReport, VenueCompetenceReport,
};
use crate::pdf_tools::{self, PdfTextExtractionResult};

/// Graduate a Tier 2 Pre-Suit Demand -> Tier 3 Lawsuit, OR Escalate Trial -> Appeal / Mandamus / SCOTUS.
/// Copies all `orders/`, `filings/`, and `evidence/` from the parent case into the new case's `evidence/`
/// as the pre-indexed Lower Court Record / Appendix!
pub fn escalate_case(
    source_case_dir: &Path,
    target_stage: &str, // "litigation" | "appellate" | "mandamus" | "scotus"
    new_folder_path: Option<&Path>,
) -> Result<EscalationResult, String> {
    let parent = load_case(source_case_dir)?;
    let stage_key = target_stage.trim().to_ascii_lowercase();

    let (new_kind, new_tier, new_stage, new_court, new_pack) = match stage_key.as_str() {
        "litigation" | "complaint" | "trial" => (
            "complaint",
            "litigation",
            "pre_filing",
            if parent.governing_state == "CO" {
                "District Court, State of Colorado".to_string()
            } else {
                "Superior Court of New Jersey, Law Division".to_string()
            },
            if parent.governing_state == "CO" {
                "co-mandamus".to_string()
            } else {
                "nj-app-div".to_string()
            },
        ),
        "mandamus" | "car21" => (
            "mandamus",
            "litigation",
            "appellate",
            "Supreme Court of Colorado".to_string(),
            "co-mandamus".to_string(),
        ),
        "scotus" | "certiorari" => (
            "appellate_brief",
            "litigation",
            "supreme_court",
            "Supreme Court of the United States".to_string(),
            "generic-us-appellate".to_string(),
        ),
        _ => (
            "notice_of_appeal",
            "litigation",
            "appellate",
            if parent.governing_state == "CO" || parent.rule_pack_id.contains("co") {
                "Colorado Court of Appeals".to_string()
            } else {
                "Superior Court of New Jersey, Appellate Division".to_string()
            },
            if parent.governing_state == "CO" || parent.rule_pack_id.contains("co") {
                "co-mandamus".to_string()
            } else {
                "nj-app-div".to_string()
            },
        ),
    };

    let new_title = match stage_key.as_str() {
        "scotus" | "certiorari" => format!("SCOTUS — {}", parent.title),
        "mandamus" => format!("Mandamus — {}", parent.title),
        "litigation" | "complaint" => format!("Lawsuit — {}", parent.title),
        _ => format!("Appeal — {}", parent.title),
    };

    let dest_root = match new_folder_path {
        Some(p) => p.to_path_buf(),
        None => default_cases_root(),
    };

    let (new_dir, mut new_profile) =
        create_case_folder(&dest_root, &new_title, CaseMode::Standard, new_kind)?;

    new_profile.matter_tier = new_tier.into();
    new_profile.procedural_stage = new_stage.into();
    new_profile.governing_state = if stage_key == "scotus" {
        "SCOTUS".into()
    } else {
        parent.governing_state.clone()
    };
    new_profile.rule_pack_id = new_pack;
    new_profile.court.name = new_court;
    new_profile.parent_matter_id = Some(parent.id.clone());
    new_profile.caption = parent.caption.clone();
    new_profile.filer = parent.filer.clone();
    new_profile.notice_of_appeal.trial_court_name = parent.court.name.clone();
    new_profile.notice_of_appeal.trial_court_docket = parent.docket_number.clone();
    new_profile.timeline = parent.timeline.clone();
    new_profile.timeline.push(TimelineEvent {
        id: uuid::Uuid::new_v4().to_string(),
        title: format!("Escalated from '{}' ({})", parent.title, parent.docket_number),
        date: Utc::now().date_naive().format("%Y-%m-%d").to_string(),
        notes: format!("Parent Matter ID: {}", parent.id),
    });

    // Copy orders/, filings/, and evidence/ from parent into new case's evidence/ as Lower Court Appendix
    let mut copied_count = 0usize;
    let target_ev = new_dir.join("evidence");
    fs::create_dir_all(&target_ev).map_err(|e| e.to_string())?;

    for sub in ["orders", "filings", "evidence"] {
        let src_sub = source_case_dir.join(sub);
        if !src_sub.is_dir() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&src_sub) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if fname.ends_with(".json") || fname.starts_with('.') {
                        continue;
                    }
                    let prefixed = format!("record-{sub}-{fname}");
                    if fs::copy(&p, target_ev.join(&prefixed)).is_ok() {
                        copied_count += 1;
                    }
                }
            }
        }
    }

    save_case(&new_dir, &new_profile)?;
    let _ = exhibits::rebuild(&new_dir);

    Ok(EscalationResult {
        ok: true,
        never_files: true,
        never_serves: true,
        parent_case_id: parent.id,
        new_case_dir: new_dir.display().to_string(),
        new_profile,
        copied_record_files: copied_count,
        message: format!(
            "Escalated matter to '{}' with {} record/appendix file(s) automatically indexed.",
            new_title, copied_count
        ),
    })
}

/// Scan all cases in `cases_root` and build a multi-state Portfolio Deadline Radar.
pub fn portfolio_status(cases_root: &Path) -> Result<PortfolioStatusReport, String> {
    let profiles = case_profile::list_cases_in(cases_root)?;
    let global_filer = load_global_filer();
    let mut matters = Vec::new();

    for p in profiles {
        let cdir = cases_root.join(&p.id);
        let actual_dir = if cdir.join("case.json").is_file() {
            cdir
        } else {
            let mut found = cases_root.join(&p.id);
            if let Ok(entries) = fs::read_dir(cases_root) {
                for e in entries.flatten() {
                    if let Ok(loaded) = load_case(&e.path()) {
                        if loaded.id == p.id {
                            found = e.path();
                            break;
                        }
                    }
                }
            }
            found
        };

        let next_steps = predict_next_steps(&p, Some(&actual_dir));
        let intake = intake_status(&actual_dir).ok();

        let urgent = next_steps
            .branches
            .iter()
            .filter(|b| b.computed_deadline.is_some())
            .min_by_key(|b| b.days_until_due.unwrap_or(9999))
            .or_else(|| next_steps.branches.first());

        matters.push(PortfolioCaseItem {
            case_id: p.id.clone(),
            title: p.title.clone(),
            matter_tier: p.effective_tier().to_string(),
            document_kind: p.document_kind.clone(),
            venue_code: next_steps.venue_code,
            docket_number: p.docket_number.clone(),
            completion_percent: intake.as_ref().map(|i| i.completion_percent).unwrap_or(0),
            next_question_prompt: intake.and_then(|i| i.next_question).map(|q| q.prompt),
            urgent_branch_title: urgent.map(|b| b.title.clone()),
            urgent_deadline: urgent.and_then(|b| b.computed_deadline.clone()),
            days_until_due: urgent.and_then(|b| b.days_until_due),
            parent_matter_id: p.parent_matter_id.clone(),
        });
    }

    matters.sort_by_key(|m| m.days_until_due.unwrap_or(99999));

    Ok(PortfolioStatusReport {
        ok: true,
        never_files: true,
        never_serves: true,
        total_matters: matters.len(),
        global_filer,
        matters,
    })
}

// ============================================================================
// Tauri IPC Commands
// ============================================================================

#[tauri::command]
pub fn case_context_cmd(id_or_path: String) -> Result<CaseContextBundle, String> {
    let dir = resolve_case_dir(&id_or_path);
    case_context(&dir)
}

#[tauri::command]
pub fn intake_status_cmd(id_or_path: String) -> Result<IntakeStatusReport, String> {
    let dir = resolve_case_dir(&id_or_path);
    intake_status(&dir)
}

#[tauri::command]
pub fn intake_answer_cmd(
    id_or_path: String,
    question_id: String,
    answer: String,
) -> Result<IntakeStatusReport, String> {
    let dir = resolve_case_dir(&id_or_path);
    intake_answer(&dir, &question_id, &answer)
}

#[tauri::command]
pub fn intake_scan_cmd(id_or_path: String) -> Result<IntakeScanReport, String> {
    let dir = resolve_case_dir(&id_or_path);
    intake_scan(&dir)
}

#[tauri::command]
pub fn case_next_steps_cmd(id_or_path: String) -> Result<NextStepsReport, String> {
    let dir = resolve_case_dir(&id_or_path);
    let profile = load_case(&dir)?;
    Ok(predict_next_steps(&profile, Some(&dir)))
}

#[tauri::command]
pub fn check_venue_competence_cmd(id_or_path: String) -> Result<VenueCompetenceReport, String> {
    let dir = resolve_case_dir(&id_or_path);
    let profile = load_case(&dir)?;
    Ok(check_venue_competence(&profile))
}

#[tauri::command]
pub fn escalate_case_cmd(
    id_or_path: String,
    target_stage: String,
) -> Result<EscalationResult, String> {
    let dir = resolve_case_dir(&id_or_path);
    escalate_case(&dir, &target_stage, None)
}

#[tauri::command]
pub fn portfolio_status_cmd() -> Result<PortfolioStatusReport, String> {
    portfolio_status(&default_cases_root())
}

#[tauri::command]
pub fn get_global_filer_cmd() -> Result<Filer, String> {
    Ok(load_global_filer())
}

#[tauri::command]
pub fn save_global_filer_cmd(filer: Filer) -> Result<Filer, String> {
    save_global_filer(&filer)
}

#[tauri::command]
pub fn pdf_extract_text_cmd(pdf_path: String) -> Result<PdfTextExtractionResult, String> {
    pdf_tools::extract_pdf_text(Path::new(&pdf_path))
}

#[tauri::command]
pub fn scoped_sections_cmd(
    id_or_path: String,
    filing_id: Option<String>,
    kind: String,
) -> Result<GuidedDraft, String> {
    let dir = resolve_case_dir(&id_or_path);
    guided::load_or_init_scoped(&dir, filing_id.as_deref(), &kind)
}

#[tauri::command]
pub fn scoped_fill_cmd(
    id_or_path: String,
    filing_id: Option<String>,
    section_id: String,
    text: String,
) -> Result<GuidedDraft, String> {
    let dir = resolve_case_dir(&id_or_path);
    guided::set_section_scoped(&dir, filing_id.as_deref(), &section_id, &text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn escalate_copies_record_into_appendix() {
        let root = std::env::temp_dir().join(format!("os-esc-{}", Uuid::new_v4()));
        let src = root.join("trial-case");
        fs::create_dir_all(&src).unwrap();
        let (_, mut p) =
            create_case_folder(&src, "Example v. Surrogate", CaseMode::Standard, "reconsideration")
                .unwrap();
        p.docket_number = "L-8578-25".into();
        p.governing_state = "NJ".into();
        save_case(&src, &p).unwrap();
        fs::write(
            src.join("orders").join("Order_Denied.txt"),
            "Order entered 2026-10-10 in L-8578-25",
        )
        .unwrap();

        let esc_dest = root.join("appeal-case");
        let res = escalate_case(&src, "appellate", Some(&esc_dest)).unwrap();
        assert!(res.ok);
        assert_eq!(res.copied_record_files, 1);
        assert_eq!(res.new_profile.notice_of_appeal.trial_court_docket, "L-8578-25");

        let _ = fs::remove_dir_all(root);
    }
}
