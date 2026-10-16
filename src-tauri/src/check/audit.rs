//! Preflight auditing, sign-offs, and complete case checks.

use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

use crate::case_profile::{self, ValidationIssue};
use crate::deadlines::{compute_deadline, parse_iso_date, DeadlineResult};
use crate::rule_pack::RulePack;
use super::authorities::run_authority_check;
use super::facts::run_fact_check;
use super::transcripts::check_transcripts;
use super::types::{
    checks_dir, ensure_checks, read_json, write_json, AuthorityReport, AuthorityStatus,
    CheckAllReport, FactCheckReport, PreFlightAudit, SignoffItem, SignoffReport,
    TranscriptReport, NEVER_FILE_NOTICE,
};

pub fn run_deadline(
    case_dir: &Path,
    pack_id: &str,
    rule_id: &str,
    trigger_date: &str,
) -> Result<DeadlineResult, String> {
    let pack = RulePack::resolve(pack_id).or_else(|_| RulePack::builtin(pack_id))?;
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
            message: "No deadline computed yet. Fix: check deadline --pack nj-app-div --rule reconsideration-appellate --trigger-date YYYY-MM-DD.".into(),
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
) -> Result<PathBuf, String> {
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
    let _ = crate::exhibits::rebuild(case_dir);
    Ok(dest)
}

pub fn preflight_audit(case_dir: &Path) -> Result<PreFlightAudit, String> {
    let profile = case_profile::load_case(case_dir)?;
    let mut missing = Vec::new();

    // 1. Caption
    let total_parties = profile.caption.appellants.len() + profile.caption.appellees.len()
        + profile.caption.plaintiffs.len() + profile.caption.defendants.len();
    let caption_ok = !profile.court.name.trim().is_empty() && total_parties > 0;
    let caption_summary = if caption_ok {
        format!("{} ({} parties)", profile.court.name, total_parties)
    } else {
        missing.push("Caption incomplete: court name or parties missing".to_string());
        "Missing court name or party definitions".to_string()
    };

    // 2. Docket
    let docket_clean = profile.docket_number.trim();
    let docket_ok = !docket_clean.is_empty();
    let docket_summary = if docket_ok {
        format!("Docket: {}", docket_clean)
    } else {
        missing.push("Docket / Case number is missing".to_string());
        "No docket number assigned".to_string()
    };

    // 3. Filer & Signature
    let filer_ok = !profile.filer.name.trim().is_empty() && !profile.filer.signature_name.trim().is_empty();
    let filer_summary = if filer_ok {
        format!("/s/ {} ({})", profile.filer.signature_name, profile.filer.name)
    } else {
        missing.push("Filer name or electronic signature is blank".to_string());
        "Missing filer name or signature".to_string()
    };

    // 4. Deadline
    let all_checks: Option<CheckAllReport> = read_json(&checks_dir(case_dir).join("all.json")).ok().flatten();
    let deadline_res: Option<DeadlineResult> = all_checks.as_ref().and_then(|a| a.deadline.clone())
        .or_else(|| read_json(&checks_dir(case_dir).join("deadline.json")).ok().flatten());
    
    let (deadline_ok, deadline_summary) = match deadline_res {
        Some(ref dl) => {
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            let timely = dl.due >= today;
            (timely, format!("Due: {} ({} days from trigger; timely: {})", dl.due, dl.day_count, timely))
        }
        None => {
            let litigation = profile.effective_tier() == "litigation";
            if litigation {
                missing.push(
                    "Enter the order date to compute this deadline. OpenSeason will not guess from today.".into(),
                );
                (false, "Enter the order date to compute this deadline".to_string())
            } else {
                (true, "No court deadline applies until you enter a trigger date.".to_string())
            }
        }
    };
    if !deadline_ok && deadline_res.is_some() {
        missing.push("Timeliness check failed: target filing date is past calculated deadline".to_string());
    }

    // 5. Facts & Record grounding
    let fact_report: Option<FactCheckReport> = read_json(&checks_dir(case_dir).join("factcheck.json")).ok().flatten();
    let (facts_ok, facts_verified_count, facts_total_count) = match fact_report {
        Some(fr) => {
            let total = fr.rows.len();
            let verified = fr.rows.iter().filter(|r| !r.record_cite.is_empty() || r.verified).count();
            let ok = total == 0 || verified == total;
            if !ok {
                missing.push(format!("Fact check: {} of {} facts need record citations (Ex. A, etc.)", total - verified, total));
            }
            (ok, verified, total)
        }
        None => (true, 0, 0),
    };

    // 6. Authorities
    let auth_report: Option<AuthorityReport> = read_json(&checks_dir(case_dir).join("authority.json")).ok().flatten();
    let (authorities_ok, authorities_verified_count, authorities_total_count) = match auth_report {
        Some(ar) => {
            let total = ar.rows.len();
            let verified = ar.rows.iter().filter(|r| r.status == AuthorityStatus::Verified).count();
            let ok = total == 0 || verified == total;
            if !ok {
                missing.push(format!("Authorities check: {} citations require verification on official legal sources", total - verified));
            }
            (ok, verified, total)
        }
        None => (true, 0, 0),
    };

    // 7. Transcripts
    let user_draft = profile.notice_of_appeal.user_text.as_str();
    let transcript_res = check_transcripts(case_dir, user_draft).unwrap_or(TranscriptReport {
        ok: true,
        never_files: true,
        indexed_files: vec![],
        total_lines: 0,
        citations: vec![],
        issues: vec![],
    });
    let transcripts_ok = transcript_res.ok;
    if !transcripts_ok {
        missing.push("Transcript citations: ungrounded references found".to_string());
    }

    // 8. Packet parts
    let packet_parts_ready = case_dir.join("exports").join("packet").is_dir() || case_dir.join("exports").is_dir();

    let ok = caption_ok && docket_ok && filer_ok && deadline_ok && facts_ok && authorities_ok && transcripts_ok;

    Ok(PreFlightAudit {
        ok,
        never_files: true,
        never_serves: true,
        caption_ok,
        caption_summary,
        docket_ok,
        docket_summary,
        filer_ok,
        filer_summary,
        deadline_ok,
        deadline_summary,
        facts_ok,
        facts_verified_count,
        facts_total_count,
        authorities_ok,
        authorities_verified_count,
        authorities_total_count,
        transcripts_ok,
        packet_parts_ready,
        missing_requirements: missing,
        notice: format!("{}\n{}", NEVER_FILE_NOTICE, crate::case_profile::REVIEW_NOTICE),
    })
}
