//! Mail-gap audit engine and jurisdictional trap detector.
//!
//! Analyzes discrepancies between order entry dates, mailing postmarks,
//! physical delivery, and court clerk receipt timestamps. Detects traps
//! such as Colorado's physical clerk receipt rule (Dobler v. CDOC, 2026).

use chrono::NaiveDate;
use super::types::MailAuditReport;

/// Audits the timeline of an order or filing for mail delays and jurisdictional traps.
pub fn audit_mail_gap(
    case_id: &str,
    order_entry_date: &str,
    mailing_postmark: Option<&str>,
    delivery_receipt_date: Option<&str>,
    clerk_receipt_date: Option<&str>,
    governing_state: Option<&str>,
) -> MailAuditReport {
    let state = governing_state.unwrap_or("CO").to_ascii_uppercase();
    let mut jurisdictional_traps = Vec::new();
    let mut recommended_actions = Vec::new();
    let mut status = "verified_timely".to_string();

    let entry = NaiveDate::parse_from_str(order_entry_date, "%Y-%m-%d").ok();
    let postmark = mailing_postmark.and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let delivery = delivery_receipt_date.and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let clerk = clerk_receipt_date.and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    let mail_delay_days = match (entry, delivery, postmark) {
        (Some(e), Some(d), _) => (d - e).num_days(),
        (Some(e), None, Some(p)) => (p - e).num_days(),
        _ => 0,
    };


    let holiday_or_weekend_impact = mail_delay_days >= 4;

    // Colorado Jurisdiction Traps
    if state == "CO" || state == "COLORADO" {
        jurisdictional_traps.push(
            "Colorado Physical Clerk Receipt Rule: Colorado courts require physical clerk receipt, not postmark dates. A filing postmarked before the deadline but received after is untimely."
                .to_string(),
        );

        jurisdictional_traps.push(
            "Dobler v. CDOC (Colo. App. 2026) Tolling Trap: An untimely Rule 59 postjudgment motion fails to toll the 49-day notice of appeal clock under C.A.R. 4(a)."
                .to_string(),
        );

        jurisdictional_traps.push(
            "Unique Circumstances Doctrine Strictly Limited (Estate of Ybarra, 2024): Discretionary exceptions for mailing delay or reliance on clerk statements cannot salvage missed jurisdictional deadlines."
                .to_string(),
        );

        jurisdictional_traps.push(
            "Gestner v. Gestner: Appealing the denial of a Rule 60(b) motion does not bring up the underlying judgment for appellate review."
                .to_string(),
        );

        if let (Some(p), Some(c)) = (postmark, clerk) {
            let gap = (c - p).num_days();
            if gap > 3 {
                status = "flagged_risk".to_string();
                recommended_actions.push(format!(
                    "CRITICAL: {gap}-day gap detected between postmark ({}) and clerk receipt ({}). Verify whether district court docket shows timely receipt before statutory deadline.",
                    p.format("%Y-%m-%d"),
                    c.format("%Y-%m-%d")
                ));
            }
        }

        if mail_delay_days > 5 {
            status = "flagged_risk".to_string();
            recommended_actions.push(format!(
                "Order reached litigant {mail_delay_days} days after entry. Promptly file a Motion for Enlargement of Time under C.A.R. 26(b) citing mail transmission delay."
            ));
        }

        recommended_actions.push("Check whether the order is from the screening division ('BY THE COURT') allowing response by written show-cause explanation.".to_string());
    } else if state == "NJ" || state == "NEW JERSEY" {
        jurisdictional_traps.push(
            "NJ Court Rule 4:49-2 Reconsideration Bar: The 20-day reconsideration deadline for final orders is non-relaxable under R. 1:3-4(c)."
                .to_string(),
        );
        jurisdictional_traps.push(
            "Lawson v. Dewar (App. Div. 2021): Interlocutory orders are governed by R. 4:42-2 in the interest of justice and are NOT subject to the 20-day jurisdictional bar."
                .to_string(),
        );

        recommended_actions.push("Check whether an informal 30-day adversary consent letter to the Case Manager is available under appellate practice.".to_string());
    }

    MailAuditReport {
        case_id: case_id.to_string(),
        order_entry_date: order_entry_date.to_string(),
        mailing_postmark: mailing_postmark.map(Into::into),
        delivery_receipt_date: delivery_receipt_date.map(Into::into),
        clerk_receipt_date: clerk_receipt_date.map(Into::into),
        jurisdiction_receipt_rule: if state == "CO" { "physical_clerk_receipt".into() } else { "court_rules_specific".into() },
        mail_delay_days,
        holiday_or_weekend_impact,
        jurisdictional_traps,
        recommended_actions,
        status,
    }
}
