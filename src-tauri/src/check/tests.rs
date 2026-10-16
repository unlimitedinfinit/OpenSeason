//! Tests for check module.

use std::fs;
use uuid::Uuid;
use crate::case_profile::{ensure_case_layout, sample_appeal_profile, save_case};
use super::audit::*;
use super::authorities::*;
use super::facts::*;
use super::types::*;

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

::: certificate
I certify that on October 16, 2026 I served a copy of this motion on Sample County Clerk by electronic mail.
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
    let profile = crate::case_profile::load_case(&dir).unwrap();
    let exported = crate::documents::export_court_markdown(&dir, &profile, None).unwrap();
    assert!(exported.docx.is_file());
    assert!(exported.pdf.is_file());
    assert!(crate::documents::inspect_docx_contains(&exported.docx, "Jordan Example").unwrap());
    assert!(!crate::documents::inspect_docx_contains(&exported.docx, "not legal advice").unwrap());
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
