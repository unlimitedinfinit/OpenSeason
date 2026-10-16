//! Fact checking and record citation verification.

use regex::Regex;
use std::path::Path;
use crate::case_profile::ValidationIssue;
use super::types::{checks_dir, ensure_checks, read_json, write_json, FactCheckReport, FactRow};

/// Split user draft into sentences. Does not invent facts or cites.
pub fn factual_sentences(draft: &str) -> Vec<String> {
    let cleaned = draft.replace('\n', " ");
    let re = Regex::new(r"[^.!?]+[.!?]?").unwrap();
    re.find_iter(&cleaned)
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| s.len() > 8 && !s.eq_ignore_ascii_case("respectfully submitted."))
        .collect()
}

pub fn load_saved_facts(case_dir: &Path) -> Vec<FactRow> {
    read_json::<FactCheckReport>(&checks_dir(case_dir).join("factcheck.json"))
        .ok()
        .flatten()
        .map(|r| r.rows)
        .unwrap_or_default()
}

pub fn run_fact_check(case_dir: &Path, draft: &str) -> Result<FactCheckReport, String> {
    let saved = load_saved_facts(case_dir);
    let sentences = factual_sentences(draft);
    let mut rows = Vec::new();
    let mut issues = Vec::new();
    for (i, assertion) in sentences.into_iter().enumerate() {
        let prior = saved.iter().find(|r| r.assertion == assertion);
        let row = FactRow {
            id: (i + 1) as u32,
            record_cite: prior.map(|p| p.record_cite.clone()).unwrap_or_default(),
            exhibit: prior.map(|p| p.exhibit.clone()).unwrap_or_default(),
            verified: prior.map(|p| p.verified).unwrap_or(false),
            note: prior.map(|p| p.note.clone()).unwrap_or_default(),
            assertion,
        };
        if row.record_cite.trim().is_empty() {
            issues.push(ValidationIssue {
                field: format!("fact.{}", row.id),
                message: "Factual sentence has no record cite. The checker will not invent one. Fix: add an exhibit, then evidence link --fact-id N --letter A (or check fact --id N --cite \"Ex. A\" --exhibit A)."
                    .into(),
            });
        } else if !row.verified {
            issues.push(ValidationIssue {
                field: format!("fact.{}", row.id),
                message: "Record cite is present but not marked verified.".into(),
            });
        }
        rows.push(row);
    }
    let report = FactCheckReport {
        ok: issues.is_empty(),
        never_files: true,
        rows,
        issues,
    };
    let dir = ensure_checks(case_dir)?;
    write_json(&dir.join("factcheck.json"), &report)?;
    Ok(report)
}

/// Attach a user-supplied cite to an existing fact row. Never creates a new assertion.
pub fn set_fact_cite(
    case_dir: &Path,
    fact_id: u32,
    record_cite: &str,
    exhibit: &str,
    verified: bool,
) -> Result<FactCheckReport, String> {
    let path = checks_dir(case_dir).join("factcheck.json");
    let mut report = read_json::<FactCheckReport>(&path)?
        .ok_or_else(|| "Run fact-check first so sentences exist.".to_string())?;
    let row = report
        .rows
        .iter_mut()
        .find(|r| r.id == fact_id)
        .ok_or_else(|| format!("No fact row {fact_id}."))?;
    row.record_cite = record_cite.trim().to_string();
    row.exhibit = exhibit.trim().to_string();
    row.verified = verified && !row.record_cite.is_empty();
    report.issues = report
        .rows
        .iter()
        .filter_map(|r| {
            if r.record_cite.trim().is_empty() {
                Some(ValidationIssue {
                    field: format!("fact.{}", r.id),
                    message: "Factual sentence has no record cite. The checker will not invent one. Fix: evidence link --fact-id N --letter A."
                        .into(),
                })
            } else if !r.verified {
                Some(ValidationIssue {
                    field: format!("fact.{}", r.id),
                    message: "Record cite is present but not marked verified.".into(),
                })
            } else {
                None
            }
        })
        .collect();
    report.ok = report.issues.is_empty();
    write_json(&path, &report)?;
    Ok(report)
}
