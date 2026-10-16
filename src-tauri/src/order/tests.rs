//! Tests for order decoding, mail gap auditing, and case bootstrap.

use super::bootstrap::*;
use super::decoder::*;
use super::mail_audit::*;
use super::types::*;

#[test]
fn test_decode_colorado_show_cause_order() {
    let order_text = r#"
COLORADO COURT OF APPEALS
2 East 14th Avenue
Denver, CO 80203

Court of Appeals Case No. 2026CA765
Pueblo County District Court Case No. 24CV30000

Jordan Example,
Plaintiff-Appellant,
v.
Sample Agency,
Defendant-Appellee.

ORDER TO SHOW CAUSE

Upon consideration of the notice of appeal filed herein, it appears that the final judgment was entered on December 19, 2025. 
The notice of appeal was received on October 6, 2026.

IT IS HEREBY ORDERED that appellant shall show cause in writing within 21 days of the date of this order why this appeal should not be dismissed for lack of appellate jurisdiction under C.A.R. 4(a).

Dated: October 6, 2026.

BY THE COURT
"#;

    let decoded = decode_order_text(order_text, Some("2026CA765-Show-Cause.pdf"));

    assert_eq!(decoded.court_name.as_deref(), Some("Colorado Court of Appeals"));
    assert_eq!(decoded.docket_number.as_deref(), Some("2026CA765"));
    assert_eq!(decoded.title, "Order to Show Cause");
    assert!(decoded.signer.is_screening_division());
    assert!(!decoded.is_boilerplate_notice);
    assert_eq!(decoded.candidate_response_window_days, Some(21));
    assert!(decoded.governing_rules_cited.iter().any(|r| r.contains("C.A.R. 4")));
    assert_eq!(decoded.status, "needs_verification");
    assert!(decoded.uncertainty_notes.iter().any(|n| n.contains("BY THE COURT")));
    assert!(decoded.suggested_actions.iter().any(|a| a.contains("Review governing rule")));
}

#[test]
fn test_decode_new_jersey_notice_of_docketing_is_boilerplate() {
    let notice_text = r#"
SUPERIOR COURT OF NEW JERSEY
APPELLATE DIVISION
DOCKET NO. A-001234-24

NOTICE OF DOCKETING

To all parties:
The appeal in the above matter has been docketed. Transcripts must be ordered within 30 days.
Paragraph 8: Requests for oral argument under R. 2:11-1(b) must be submitted in accordance with the court rules.
Paragraph 9: This notice does not schedule oral argument or set a briefing date.

Dated: January 15, 2026
Joseph H. Orlando, Clerk
"#;

    let decoded = decode_order_text(notice_text, Some("Notice-of-Docketing.pdf"));

    assert_eq!(decoded.court_name.as_deref(), Some("New Jersey Superior Court, Appellate Division"));
    assert!(decoded.is_boilerplate_notice);
    assert!(decoded.uncertainty_notes.iter().any(|n| n.contains("boilerplate")));
    assert!(decoded.suggested_actions.iter().any(|a| a.contains("transcript delivery date")));
}

#[test]
fn test_colorado_mail_delay_flags_dobler_trap() {
    let report = audit_mail_gap(
        "example-case",
        "2026-10-06",
        Some("2026-10-08"),
        Some("2026-10-15"),
        Some("2026-10-16"),
        Some("CO"),
    );

    assert_eq!(report.jurisdiction_receipt_rule, "physical_clerk_receipt");
    assert!(report.mail_delay_days >= 7);
    assert_eq!(report.status, "flagged_risk");
    assert!(report.jurisdictional_traps.iter().any(|t| t.contains("Dobler v. CDOC")));
    assert!(report.jurisdictional_traps.iter().any(|t| t.contains("Physical Clerk Receipt")));
    assert!(report.jurisdictional_traps.iter().any(|t| t.contains("Estate of Ybarra")));
}

#[test]
fn test_case_bootstrap_organizes_folder_without_moving_originals() {
    let temp_src = std::env::temp_dir().join(format!("src-raw-{}", uuid::Uuid::new_v4()));
    let temp_dest = std::env::temp_dir().join(format!("dest-cases-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_src).unwrap();
    std::fs::create_dir_all(&temp_dest).unwrap();

    // Create fake original raw files
    let order_file = temp_src.join("2026CA765-Order-Dismissal.pdf");
    let motion_file = temp_src.join("Motion-Rule59-Postjudgment.pdf");
    let transcript_file = temp_src.join("Hearing-Transcript-Exhibits.pdf");
    let notes_file = temp_src.join("My-Case-Notes.md");

    std::fs::write(&order_file, b"%PDF-1.4 fake order").unwrap();
    std::fs::write(&motion_file, b"%PDF-1.4 fake motion").unwrap();
    std::fs::write(&transcript_file, b"%PDF-1.4 fake transcript").unwrap();
    std::fs::write(&notes_file, b"# Notes").unwrap();

    let res = bootstrap_case_from_folder(&temp_src, &temp_dest, Some("Jordan Example Appeal")).unwrap();

    assert_eq!(res.files_scanned, 4);
    assert_eq!(res.orders_imported, 1);
    assert_eq!(res.filings_imported, 1);
    assert_eq!(res.evidence_imported, 1);
    assert_eq!(res.drafts_imported, 1);
    assert_eq!(res.detected_docket_number.as_deref(), Some("2026CA765"));

    // Verify originals still exist untouched
    assert!(order_file.exists());
    assert!(motion_file.exists());
    assert!(transcript_file.exists());
    assert!(notes_file.exists());

    let _ = std::fs::remove_dir_all(&temp_src);
    let _ = std::fs::remove_dir_all(&temp_dest);
}
