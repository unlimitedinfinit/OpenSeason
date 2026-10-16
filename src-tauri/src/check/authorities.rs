//! Legal authorities extraction and verification.

use regex::Regex;
use std::path::Path;
use crate::case_profile::ValidationIssue;
use super::types::{checks_dir, ensure_checks, read_json, write_json, AuthorityReport, AuthorityRow, AuthorityStatus};

pub fn extract_authorities(draft: &str) -> Vec<String> {
    let patterns = [
        r"R\.\s*\d+:\d+(?:-\d+(?:\([a-z0-9]+\))?)?",
        r"C\.A\.R\.\s*\d+",
        r"\d+\s+U\.S\.C\.\s*§\s*\d+[a-z0-9-]*",
        r"Fed\.\s*R\.\s*(?:App|Civ|Evid)\.\s*P\.\s*\d+[a-z0-9()]*",
        r"\d+\s+[A-Z][A-Za-z.&']+(?:\s+[A-Z][A-Za-z.&']+)*\s+\d+(?:,\s*\d+)?",
    ];
    let mut found = Vec::new();
    for pat in patterns {
        let re = Regex::new(pat).unwrap();
        for m in re.find_iter(draft) {
            let cite = collapse_ws(m.as_str());
            if !found.contains(&cite) {
                found.push(cite);
            }
        }
    }
    found
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn load_saved_authorities(case_dir: &Path) -> Vec<AuthorityRow> {
    read_json::<AuthorityReport>(&checks_dir(case_dir).join("authority.json"))
        .ok()
        .flatten()
        .map(|r| r.rows)
        .unwrap_or_default()
}

pub fn run_authority_check(case_dir: &Path, draft: &str) -> Result<AuthorityReport, String> {
    let saved = load_saved_authorities(case_dir);
    let cites = extract_authorities(draft);
    let mut rows = Vec::new();
    let mut issues = Vec::new();
    for (i, cite) in cites.into_iter().enumerate() {
        let prior = saved.iter().find(|r| r.cite == cite);
        let status = prior.map(|p| p.status).unwrap_or(AuthorityStatus::Unverified);
        let row = AuthorityRow {
            id: (i + 1) as u32,
            source_url: prior.map(|p| p.source_url.clone()).unwrap_or_default(),
            note: prior.map(|p| p.note.clone()).unwrap_or_default(),
            status,
            cite,
        };
        if row.status != AuthorityStatus::Verified {
            issues.push(ValidationIssue {
                field: format!("authority.{}", row.id),
                message: format!(
                    "Cite '{}' is {}. The checker does not invent or verify citations on its own.",
                    row.cite,
                    match row.status {
                        AuthorityStatus::NotFound => "not found",
                        _ => "unverified",
                    }
                ),
            });
        }
        rows.push(row);
    }
    let report = AuthorityReport {
        ok: issues.is_empty(),
        never_files: true,
        invented: false,
        rows,
        issues,
    };
    let dir = ensure_checks(case_dir)?;
    write_json(&dir.join("authority.json"), &report)?;
    Ok(report)
}

pub fn set_authority_status(
    case_dir: &Path,
    authority_id: u32,
    status: AuthorityStatus,
    source_url: &str,
    note: &str,
) -> Result<AuthorityReport, String> {
    let path = checks_dir(case_dir).join("authority.json");
    let mut report = read_json::<AuthorityReport>(&path)?
        .ok_or_else(|| "Run authority check first.".to_string())?;
    let row = report
        .rows
        .iter_mut()
        .find(|r| r.id == authority_id)
        .ok_or_else(|| format!("No authority row {authority_id}."))?;
    row.status = status;
    row.source_url = source_url.trim().to_string();
    row.note = note.trim().to_string();
    report.invented = false;
    report.issues = report
        .rows
        .iter()
        .filter(|r| r.status != AuthorityStatus::Verified)
        .map(|r| ValidationIssue {
            field: format!("authority.{}", r.id),
            message: format!("Cite '{}' is not verified.", r.cite),
        })
        .collect();
    report.ok = report.issues.is_empty();
    write_json(&path, &report)?;
    Ok(report)
}
