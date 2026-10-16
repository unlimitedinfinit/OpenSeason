use std::fs;
use std::path::Path;

use super::routes::determine_routes_for_issue;
use super::types::{DiscrepancyReport, FindingSource, IssueItem, RecordSupportStatus};
use crate::case_profile::CaseProfile;
use crate::record::inventory::load_record_inventory;
use crate::record::types::EvidentiaryTier;

pub const MANDATORY_NON_TOLLING_WARNING: &str =
    "IMPORTANT: Filing a judicial misconduct complaint (e.g. with the ACJC) or an attorney ethics grievance \
    DOES NOT overturn, modify, or stay any court order, and DOES NOT toll, extend, or pause your statutory \
    deadlines to file a Motion for Reconsideration or Notice of Appeal. Always protect your court deadlines first.";

/// Audits judicial orders against the case record inventory to surface unsupported premises or discrepancies.
pub fn audit_record_discrepancies(
    case_dir: &Path,
    profile: &CaseProfile,
) -> Result<DiscrepancyReport, String> {
    let mut issues = Vec::new();
    let record_inv = load_record_inventory(case_dir);

    let orders_dir = case_dir.join("orders");
    if orders_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&orders_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                    let text = extract_order_text(&path).unwrap_or_default();

                    // Check for key factual findings that frequently lack record support
                    let lower = text.to_ascii_lowercase();

                    // 1. Jurisdictional / Residency findings (e.g. Florida vs NJ scenario)
                    if lower.contains("jurisdiction") || lower.contains("reside") || lower.contains("florida") || lower.contains("dismiss") {
                        let has_residence_local = record_inv
                            .as_ref()
                            .map(|inv| inv.entries.iter().any(|e| e.filename.to_ascii_lowercase().contains("lease") || e.filename.to_ascii_lowercase().contains("residence")))
                            .unwrap_or(false);

                        let has_residence_filed = record_inv
                            .as_ref()
                            .map(|inv| inv.entries.iter().any(|e| e.tier == EvidentiaryTier::OfficialCourtRecord && (e.filename.to_ascii_lowercase().contains("lease") || e.filename.to_ascii_lowercase().contains("residence"))))
                            .unwrap_or(false);

                        let mut contradicting_local = Vec::new();
                        let mut contradicting_filed = Vec::new();

                        if let Some(inv) = &record_inv {
                            for e in &inv.entries {
                                let l = e.filename.to_ascii_lowercase();
                                if l.contains("lease") || l.contains("residence") || l.contains("nj") || l.contains("address") {
                                    if e.tier == EvidentiaryTier::OfficialCourtRecord {
                                        contradicting_filed.push(format!("{} (Filed Tier 1)", e.filename));
                                    } else {
                                        contradicting_local.push(format!("{} (Unfiled Tier 4)", e.filename));
                                    }
                                }
                            }
                        }

                        let status = if has_residence_filed {
                            RecordSupportStatus::ContradictedByRecord
                        } else if has_residence_local {
                            RecordSupportStatus::UnsupportedInRecord
                        } else {
                            RecordSupportStatus::UnverifiedOralColloquy
                        };

                        let quoted = extract_snippet(&text, &["jurisdiction", "reside", "florida", "dismiss"]);

                        let issue = IssueItem {
                            id: format!("iss-{:03}", issues.len() + 1),
                            title: format!("Factual premise in {} regarding residency or jurisdiction", fname),
                            date_identified: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                            finding_source: FindingSource {
                                document_id: fname.clone(),
                                document_name: fname,
                                page: Some(1),
                                quoted_text: quoted,
                            },
                            record_support_status: status,
                            contradicting_evidence_local: contradicting_local,
                            contradicting_evidence_filed: contradicting_filed,
                            analysis_notes: "The court order addresses residency or jurisdictional authority. If the finding rests on statements made during oral colloquy rather than filed evidence, obtain the hearing transcript to verify whether an evidentiary basis exists.".to_string(),
                            available_routes: determine_routes_for_issue(profile, status, false),
                            resolved: false,
                        };
                        issues.push(issue);
                    }
                }
            }
        }
    }

    let summary = if issues.is_empty() {
        "Audit completed: no glaring jurisdictional or residency discrepancies detected between orders and verified record.".to_string()
    } else {
        format!(
            "Audit identified {} potential record discrepancy issue(s). Review each issue's evidentiary support and procedural routes.",
            issues.len()
        )
    };

    let report = DiscrepancyReport {
        case_id: profile.id.clone(),
        court_name: profile.court.name.clone(),
        issues,
        non_tolling_warning: MANDATORY_NON_TOLLING_WARNING.to_string(),
        summary,
    };

    save_issues_report(case_dir, &report)?;
    Ok(report)
}

fn extract_snippet(text: &str, keywords: &[&str]) -> String {
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        for kw in keywords {
            if lower.contains(kw) {
                return line.trim().to_string();
            }
        }
    }
    "Relevant operative clause from entered order".to_string()
}

fn extract_order_text(path: &Path) -> Result<String, String> {
    if path.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("pdf")).unwrap_or(false) {
        if let Ok(doc) = lopdf::Document::load(path) {
            let pages = doc.get_pages();
            let mut out = String::new();
            for &page_id in pages.keys().take(5) {
                if let Ok(t) = doc.extract_text(&[page_id]) {
                    out.push_str(&t);
                    out.push('\n');
                }
            }
            return Ok(out);
        }
    } else if let Ok(t) = fs::read_to_string(path) {
        return Ok(t);
    }
    Ok(String::new())
}

pub fn load_issues_report(case_dir: &Path) -> Option<DiscrepancyReport> {
    let p = case_dir.join(".openseason").join("issues.json");
    if !p.is_file() {
        return None;
    }
    let data = fs::read_to_string(p).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_issues_report(case_dir: &Path, report: &DiscrepancyReport) -> Result<(), String> {
    let dir = case_dir.join(".openseason");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let data = serde_json::to_string_pretty(report).map_err(|e| e.to_string())?;
    fs::write(dir.join("issues.json"), data).map_err(|e| e.to_string())
}
