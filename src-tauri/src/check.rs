use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::case_profile::{self, ValidationIssue};
use crate::deadlines::{compute_deadline, parse_iso_date, DeadlineResult};
use crate::rule_pack::RulePack;

pub const NEVER_FILE_NOTICE: &str =
    "This app never files or serves. Josh (or the user) signs, files, and serves by hand.";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FactRow {
    pub id: u32,
    pub assertion: String,
    #[serde(default)]
    pub record_cite: String,
    #[serde(default)]
    pub exhibit: String,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FactCheckReport {
    pub ok: bool,
    pub never_files: bool,
    pub rows: Vec<FactRow>,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityStatus {
    Verified,
    Unverified,
    NotFound,
}

impl Default for AuthorityStatus {
    fn default() -> Self {
        AuthorityStatus::Unverified
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthorityRow {
    pub id: u32,
    pub cite: String,
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub status: AuthorityStatus,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthorityReport {
    pub ok: bool,
    pub never_files: bool,
    pub invented: bool,
    pub rows: Vec<AuthorityRow>,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignoffItem {
    pub id: String,
    pub label: String,
    pub checked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignoffReport {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub items: Vec<SignoffItem>,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckAllReport {
    pub never_files: bool,
    pub never_serves: bool,
    pub notice: String,
    pub fact: FactCheckReport,
    pub authority: AuthorityReport,
    pub deadline: Option<DeadlineResult>,
    pub signoff: SignoffReport,
    pub ok: bool,
    pub issues: Vec<ValidationIssue>,
}

pub fn checks_dir(case_dir: &Path) -> std::path::PathBuf {
    case_dir.join("checks")
}

fn ensure_checks(case_dir: &Path) -> Result<std::path::PathBuf, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Checks cannot write into the Documents pointer.".into());
    }
    let dir = checks_dir(case_dir);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

pub fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map(Some).map_err(|e| e.to_string())
}

/// Split user draft into sentences. Does not invent facts or cites.
pub fn factual_sentences(draft: &str) -> Vec<String> {
    let cleaned = draft.replace('\n', " ");
    let re = Regex::new(r"[^.!?]+[.!?]?").unwrap();
    re.find_iter(&cleaned)
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| s.len() > 8 && !s.eq_ignore_ascii_case("respectfully submitted."))
        .collect()
}

fn load_saved_facts(case_dir: &Path) -> Vec<FactRow> {
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
                message: "Factual sentence has no record cite. The checker will not invent one."
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
                    message: "Factual sentence has no record cite. The checker will not invent one."
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

/// Extract citation-like tokens already present in the draft. Never invents a cite.
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

fn load_saved_authorities(case_dir: &Path) -> Vec<AuthorityRow> {
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

pub fn run_deadline(
    case_dir: &Path,
    pack_id: &str,
    rule_id: &str,
    trigger_date: &str,
) -> Result<DeadlineResult, String> {
    let pack = RulePack::builtin(pack_id)?;
    let rule = pack.find_deadline(rule_id)?.clone();
    let date = parse_iso_date(trigger_date)?;
    let result = compute_deadline(&pack, &rule, date);
    let dir = ensure_checks(case_dir)?;
    write_json(&dir.join("deadline.json"), &result)?;
    Ok(result)
}

fn default_signoff_items() -> Vec<SignoffItem> {
    [
        ("read-pdf", "I read the whole PDF, every page."),
        ("facts", "Every fact has a record cite and I checked each one."),
        ("authority", "Every case or rule cite was verified on a live source."),
        ("deadline", "I agree with the deadline math and the effect on appeal time."),
        ("jurisdiction", "I know whether a notice of appeal has been filed and whether this court can rule."),
        ("format", "Caption, signature block, and certificate of service are complete."),
        ("clean", "No placeholders, UNVERIFIED tags, or agent notes remain in the PDF."),
        ("no-advice", "I understand this tool gives no legal advice and did not file or serve anything."),
        ("i-file", "I will file and serve this myself."),
    ]
    .into_iter()
    .map(|(id, label)| SignoffItem {
        id: id.into(),
        label: label.into(),
        checked: false,
    })
    .collect()
}

pub fn load_or_init_signoff(case_dir: &Path) -> Result<SignoffReport, String> {
    let path = checks_dir(case_dir).join("signoff.json");
    if let Some(existing) = read_json::<SignoffReport>(&path)? {
        return Ok(existing);
    }
    let report = SignoffReport {
        ok: false,
        never_files: true,
        never_serves: true,
        items: default_signoff_items(),
        issues: vec![ValidationIssue {
            field: "signoff".into(),
            message: "Sign-off is incomplete. The user must check every box. The app will not file."
                .into(),
        }],
    };
    let dir = ensure_checks(case_dir)?;
    write_json(&dir.join("signoff.json"), &report)?;
    Ok(report)
}

pub fn set_signoff(case_dir: &Path, id: &str, checked: bool) -> Result<SignoffReport, String> {
    let mut report = load_or_init_signoff(case_dir)?;
    let item = report
        .items
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| format!("Unknown sign-off item '{id}'."))?;
    item.checked = checked;
    report.never_files = true;
    report.never_serves = true;
    report.ok = report.items.iter().all(|i| i.checked);
    report.issues = if report.ok {
        Vec::new()
    } else {
        vec![ValidationIssue {
            field: "signoff".into(),
            message: "Sign-off is incomplete. The user must check every box. The app will not file."
                .into(),
        }]
    };
    write_json(&checks_dir(case_dir).join("signoff.json"), &report)?;
    Ok(report)
}

pub fn draft_text_for_case(case_dir: &Path) -> Result<String, String> {
    let md = case_dir.join("drafts").join("draft.md");
    if md.is_file() {
        return fs::read_to_string(md).map_err(|e| e.to_string());
    }
    let profile = case_profile::load_case(case_dir)?;
    let kind = profile.document_kind.to_ascii_lowercase();
    let body = match kind.as_str() {
        "complaint" => profile.complaint.user_text,
        "appellate_brief" | "brief" => profile.brief.user_text,
        "appellate_motion" => profile.appellate_motion.user_text,
        "mandamus" => profile.mandamus.user_text,
        "motion" => profile.motion.user_text,
        _ => {
            if !profile.court_markdown.trim().is_empty() {
                profile.court_markdown
            } else if !profile.motion.user_text.trim().is_empty() {
                profile.motion.user_text
            } else {
                profile.notice_of_appeal.user_text
            }
        }
    };
    Ok(body)
}

pub fn run_all(
    case_dir: &Path,
    pack_id: Option<&str>,
    rule_id: Option<&str>,
    trigger_date: Option<&str>,
) -> Result<CheckAllReport, String> {
    let draft = draft_text_for_case(case_dir)?;
    let fact = run_fact_check(case_dir, &draft)?;
    let authority = run_authority_check(case_dir, &draft)?;
    let deadline = match (pack_id, rule_id, trigger_date) {
        (Some(p), Some(r), Some(d)) => Some(run_deadline(case_dir, p, r, d)?),
        _ => read_json(&checks_dir(case_dir).join("deadline.json"))?,
    };
    let signoff = load_or_init_signoff(case_dir)?;
    let mut issues = Vec::new();
    issues.extend(fact.issues.clone());
    issues.extend(authority.issues.clone());
    issues.extend(signoff.issues.clone());
    if deadline.is_none() {
        issues.push(ValidationIssue {
            field: "deadline".into(),
            message: "No deadline computed yet. Pass --pack, --rule, and --trigger-date.".into(),
        });
    }
    Ok(CheckAllReport {
        never_files: true,
        never_serves: true,
        notice: format!(
            "{}\nChecked at {}.",
            NEVER_FILE_NOTICE,
            Utc::now().to_rfc3339()
        ),
        ok: issues.is_empty(),
        fact,
        authority,
        deadline,
        signoff,
        issues,
    })
}

pub fn add_exhibit(
    case_dir: &Path,
    filename: &str,
    bytes: &[u8],
    description: &str,
) -> Result<std::path::PathBuf, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Add evidence in the unlocked vault, with the vault password.".into());
    }
    let evidence = case_dir.join("evidence");
    fs::create_dir_all(&evidence).map_err(|e| e.to_string())?;
    let safe = crate::sandbox::sanitize_download_stem(filename).unwrap_or_else(|_| "exhibit".into());
    let dest = evidence.join(&safe);
    if dest.exists() {
        return Err("An exhibit with that name already exists. Rename the file and try again.".into());
    }
    fs::write(&dest, bytes).map_err(|e| e.to_string())?;
    if !description.trim().is_empty() {
        let _ = fs::write(evidence.join(format!("{safe}.txt")), description);
    }
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_profile::{ensure_case_layout, sample_appeal_profile, save_case};
    use uuid::Uuid;

    fn temp_case() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("os-check-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        ensure_case_layout(&dir).unwrap();
        save_case(&dir, &sample_appeal_profile()).unwrap();
        dir
    }

    #[test]
    fn fact_check_does_not_invent_cites() {
        let dir = temp_case();
        let draft = "The order dismissed the complaint. The clerk mailed the order on March 1.";
        let report = run_fact_check(&dir, draft).unwrap();
        assert!(!report.ok);
        assert!(report.rows.len() >= 2);
        assert!(report.rows.iter().all(|r| r.record_cite.is_empty()));
        assert!(report.issues.iter().all(|i| i.message.contains("will not invent")));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn authority_extracts_only_draft_cites_and_starts_unverified() {
        let dir = temp_case();
        let draft = "Relief is sought under R. 2:11-6 and R. 1:3-4(a). No other authorities.";
        let report = run_authority_check(&dir, draft).unwrap();
        assert_eq!(report.invented, false);
        assert!(report.rows.iter().any(|r| r.cite.contains("2:11-6")));
        assert!(report.rows.iter().all(|r| r.status == AuthorityStatus::Unverified));
        assert!(!report.ok);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn signoff_blocks_until_every_box_is_checked() {
        let dir = temp_case();
        let first = load_or_init_signoff(&dir).unwrap();
        assert!(!first.ok);
        assert!(first.never_files && first.never_serves);
        let last_id = first.items.last().unwrap().id.clone();
        for item in &first.items {
            if item.id != last_id {
                set_signoff(&dir, &item.id, true).unwrap();
            }
        }
        assert!(!set_signoff(&dir, &last_id, false).unwrap().ok);
        assert!(set_signoff(&dir, &last_id, true).unwrap().ok);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn agent_e2e_import_check_deadline() {
        let dir = temp_case();
        let md = r#"---
court: Superior Court of New Jersey, Appellate Division
plaintiff: Jordan Example
plaintiff_label: Appellant
defendant: Sample County Clerk
defendant_label: Respondent
case_no: A-01000-24
doc_title: Motion for reconsideration
---

The trial court entered the order on March 1. Relief is sought under R. 2:11-6.

::: signature
/s/ Jordan Example
:::
"#;
        crate::court_md::import_markdown(&dir, md).unwrap();
        let exhibit = add_exhibit(&dir, "order.txt", b"Order granting dismissal", "Dkt. 12").unwrap();
        assert!(exhibit.exists());
        let all = run_all(
            &dir,
            Some("nj-app-div"),
            Some("reconsideration-appellate"),
            Some("2026-01-09"),
        )
        .unwrap();
        assert!(all.never_files && all.never_serves);
        assert!(!all.fact.ok, "facts start without cites");
        assert!(all.authority.rows.iter().any(|r| r.cite.contains("2:11-6")));
        assert_eq!(all.deadline.as_ref().unwrap().due, "2026-01-20");
        let profile = case_profile::load_case(&dir).unwrap();
        let exported = crate::documents::export_court_markdown(&dir, &profile, None).unwrap();
        assert!(exported.docx.is_file());
        assert!(exported.pdf.is_file());
        assert!(crate::documents::inspect_docx_contains(&exported.docx, "Jordan Example").unwrap());
        assert!(crate::documents::inspect_docx_contains(&exported.docx, "not legal advice").unwrap());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn authority_does_not_invent_cites_absent_from_draft() {
        let dir = temp_case();
        let report = run_authority_check(&dir, "The clerk mailed the order.").unwrap();
        assert!(!report.invented);
        assert!(report.rows.is_empty());
        let _ = fs::remove_dir_all(dir);
    }
}
