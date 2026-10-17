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

#[test]
fn test_end_to_end_order_ingestion_and_workflow_generation() {
    let temp_case = std::env::temp_dir().join(format!("case-order-e2e-{}", uuid::Uuid::new_v4()));
    let (_created_dir, _profile) = crate::case_profile::create_case_folder(
        &temp_case,
        "Jordan Example v. Sample Agency",
        crate::case_profile::CaseMode::Standard,
        "motion",
    ).unwrap();

    // Create a real valid PDF with show cause language using Typst
    let order_pdf_path = std::env::temp_dir().join(format!("order-show-cause-{}.pdf", uuid::Uuid::new_v4()));
    let typst_markup = r#"
#set page(paper: "us-letter", margin: 1in)
#set text(font: "Liberation Serif", size: 12pt)

#align(center)[
  *COLORADO COURT OF APPEALS* \
  2 East 14th Avenue, Denver, CO 80203 \
  \
  Court of Appeals Case No. 2026CA765 \
  \
  *ORDER TO SHOW CAUSE*
]

Upon consideration of the notice of appeal filed herein, it appears that final judgment was entered on December 19, 2025.

IT IS HEREBY ORDERED that appellant shall show cause within 21 days why appeal should not be dismissed for lack of jurisdiction.

Dated: October 6, 2026.

BY THE COURT
"#;
    let pdf_bytes = crate::pdf::compile_typst(typst_markup.to_string()).unwrap();
    std::fs::write(&order_pdf_path, pdf_bytes).unwrap();

    // Ingest the order into the case folder
    let report = ingest_order_into_case(&temp_case, &order_pdf_path).unwrap();

    // Verify preservation
    assert!(temp_case.join(&report.saved_order_path).exists());
    assert!(!report.sha256.is_empty());

    // Verify order decoding
    assert_eq!(report.decoded_order.title, "Order to Show Cause");
    assert_eq!(report.decoded_order.court_name.as_deref(), Some("Colorado Court of Appeals"));
    assert_eq!(report.decoded_order.docket_number.as_deref(), Some("2026CA765"));
    assert!(report.decoded_order.signer.is_by_the_court());

    // Verify timeline update in case.json
    let updated_profile = crate::case_profile::load_case(&temp_case).unwrap();
    assert!(updated_profile.timeline.iter().any(|ev| ev.title == "Order to Show Cause"));
    assert_eq!(updated_profile.docket_number, "2026CA765");
    assert_eq!(updated_profile.court.name, "Colorado Court of Appeals");

    // Verify receipt logged
    assert!(updated_profile.receipts.iter().any(|r| r.operation == "order.ingest_and_classify"));

    // Verify workflow plan was formulated & saved
    let plan = report.workflow_plan.expect("workflow plan should be formulated");
    assert!(!plan.steps.is_empty());
    assert!(temp_case.join("workflow_plan.json").exists());

    // Clean up
    let _ = std::fs::remove_file(&order_pdf_path);
    let _ = std::fs::remove_dir_all(&temp_case);
}
