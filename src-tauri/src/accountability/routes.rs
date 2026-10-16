use super::types::{AccountabilityRoute, RecordSupportStatus, RouteKind};
use crate::case_profile::CaseProfile;

pub fn determine_routes_for_issue(
    profile: &CaseProfile,
    status: RecordSupportStatus,
    is_final_judgment: bool,
) -> Vec<AccountabilityRoute> {
    let mut routes = Vec::new();

    let state = if !profile.governing_state.is_empty() {
        profile.governing_state.to_ascii_uppercase()
    } else if profile.court.name.to_ascii_uppercase().contains("NEW JERSEY") {
        "NJ".to_string()
    } else if profile.court.name.to_ascii_uppercase().contains("COLORADO") {
        "CO".to_string()
    } else {
        "FED".to_string()
    };

    // 1. Interlocutory Reconsideration Route
    if !is_final_judgment && state == "NJ" {
        routes.push(AccountabilityRoute {
            route_kind: RouteKind::ReconsiderationInterlocutory,
            title: "Interlocutory Reconsideration in the Interest of Justice".to_string(),
            rule_citation: "NJ Court Rule 4:42-2 (Civil Practice)".to_string(),
            governing_standard: "Broad judicial discretion to revise any interlocutory order prior to entry of final judgment in the interests of justice (Lawson v. Dewar, 368 N.J. Super. 5). Not subject to the strict 20-day bar of R. 4:49-2.".to_string(),
            deadline_notice: Some("Prior to entry of final judgment (bring promptly upon discovering factual error).".to_string()),
            is_disciplinary: false,
            warning_notice: "Standard court motion. File in trial court with supporting certification citing actual record.".to_string(),
            official_form_name: Some("Notice of Motion (Civil/Chancery)".to_string()),
            submission_portal_url: Some("https://www.njcourts.gov/self-help/civil-case".to_string()),
        });
    }

    // 2. Final Judgment Reconsideration Route
    if is_final_judgment || state != "NJ" {
        let (rule_cit, std_text, dl_text) = if state == "NJ" {
            (
                "NJ Court Rule 4:49-2",
                "Reconsideration of final order: granted only if court acted in an arbitrary, capricious, or palpably incorrect manner, or failed to consider evidence on the record (D'Atria v. D'Atria).",
                "STRICT: Served not later than 20 days after service of the judgment or order. Non-extendable by court or parties.",
            )
        } else if state == "CO" {
            (
                "C.R.C.P. Rule 59 / 121 § 1-15",
                "Post-trial relief / amendment of findings. Court made clear error of law or fact.",
                "STRICT: Filed within 14 days of entry of order unless extended.",
            )
        } else {
            (
                "Fed. R. Civ. P. 59(e) / 60(b)",
                "Motion to Alter or Amend a Judgment based on manifest error of law or fact.",
                "Within 28 days of judgment entry (Rule 59) or reasonable time (Rule 60).",
            )
        };

        routes.push(AccountabilityRoute {
            route_kind: RouteKind::ReconsiderationFinal,
            title: "Motion for Reconsideration of Final Order".to_string(),
            rule_citation: rule_cit.to_string(),
            governing_standard: std_text.to_string(),
            deadline_notice: Some(dl_text.to_string()),
            is_disciplinary: false,
            warning_notice: "Filing deadline is non-extendable in many venues. Check service date immediately.".to_string(),
            official_form_name: Some("Motion for Reconsideration".to_string()),
            submission_portal_url: None,
        });
    }

    // 3. Record Correction / Supplementation on Appeal
    if state == "NJ" {
        routes.push(AccountabilityRoute {
            route_kind: RouteKind::RecordCorrection,
            title: "Motion to Supplement or Correct the Record on Appeal".to_string(),
            rule_citation: "NJ Court Rule 2:5-5 (Appellate Practice)".to_string(),
            governing_standard: "If anything material to either party is omitted from or misstated in the record below, the trial judge or Appellate Division may direct that the omission or misstatement be corrected.".to_string(),
            deadline_notice: Some("Promptly upon filing Notice of Appeal or discovering record omission.".to_string()),
            is_disciplinary: false,
            warning_notice: "Does not allow introducing brand-new evidence that was never before the trial court. Strictly for correcting what was omitted from the transmission.".to_string(),
            official_form_name: Some("Motion for Leave to Correct/Supplement Record (Part II)".to_string()),
            submission_portal_url: Some("https://www.njcourts.gov/sites/default/files/forms/10837_appl_prose_kit.pdf".to_string()),
        });
    }

    // 4. Judicial Conduct Grievance (Oversight Track)
    let (body_name, acjc_url, acjc_form) = if state == "NJ" {
        (
            "NJ Advisory Committee on Judicial Conduct (ACJC)",
            "https://www.njcourts.gov/public/acjc",
            "ACJC Form CN 11166 (Complaint Against a Judge)",
        )
    } else {
        (
            "Colorado Commission on Judicial Discipline",
            "https://www.coloradojudicialdiscipline.com",
            "CJD Complaint Form",
        )
    };

    routes.push(AccountabilityRoute {
        route_kind: RouteKind::JudicialConductGrievance,
        title: format!("Formal Inquiry to {}", body_name),
        rule_citation: "Code of Judicial Conduct (Canon 2 & Canon 3) / Court Disciplinary Rules".to_string(),
        governing_standard: "Appropriate ONLY for documented ethical violations (willful bias, ex parte communications, harassment, conflict of interest). Unfavorable rulings or judicial legal/factual errors DO NOT constitute misconduct.".to_string(),
        deadline_notice: None,
        is_disciplinary: true,
        warning_notice: "CRITICAL ALERT: Filing an ACJC/CJD complaint DOES NOT stay or overturn court orders, and DOES NOT toll your 20-day or 45-day court appeal deadlines. Proceed in court first.".to_string(),
        official_form_name: Some(acjc_form.to_string()),
        submission_portal_url: Some(acjc_url.to_string()),
    });

    // 5. Public Records / FOIA Route
    routes.push(AccountabilityRoute {
        route_kind: RouteKind::PublicRecordsRequest,
        title: if state == "NJ" { "OPRA & Court Records Request (Form CN 10200)" } else { "Open Records Request (CORA / FOIA)" }.to_string(),
        rule_citation: if state == "NJ" { "N.J.S.A. 47:1A-1 (OPRA) & Court Rule 1:38" } else { "Open Records Act" }.to_string(),
        governing_standard: "Obtain government communications, agency reports, or official clerk files not provided in discovery.".to_string(),
        deadline_notice: None,
        is_disciplinary: false,
        warning_notice: "Subject to statutory exemptions. Request electronic delivery to avoid copying surcharges.".to_string(),
        official_form_name: Some("Court Records Request Form CN 10200".to_string()),
        submission_portal_url: Some("https://www.njcourts.gov/faq/what-electronic-case-jacket".to_string()),
    });

    routes
}
