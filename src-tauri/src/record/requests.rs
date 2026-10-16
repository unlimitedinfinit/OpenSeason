use std::path::Path;

use super::inventory::load_record_inventory;
use crate::case_profile::CaseProfile;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CourtRecordsRequestPacket {
    pub state: String,
    pub court_name: String,
    pub division: String,
    pub docket_number: String,
    pub caption: String,
    pub requester_name: String,
    pub requester_address: String,
    pub requester_phone: String,
    pub requester_email: String,
    pub records_requested: Vec<String>,
    pub preferred_format: String, // "Electronic (Email / PDF link)"
    pub statutory_fee_notice: String,
    pub submission_method: String,
    pub instructions: String,
    pub rendered_request_text: String,
}

/// Generates prefilled court records request data (e.g. NJ Form CN 10200 or PACER/Federal Jacket request).
pub fn generate_records_request(
    case_dir: &Path,
    profile: &CaseProfile,
    state_override: Option<&str>,
) -> Result<CourtRecordsRequestPacket, String> {
    let report = load_record_inventory(case_dir);
    let mut requested_items = Vec::new();

    if let Some(r) = &report {
        for m in &r.missing_records {
            requested_items.push(format!("{} ({})", m.title, m.reason));
        }
    }

    if requested_items.is_empty() {
        requested_items.push("Complete Case Jacket & Official Register of Actions".to_string());
        requested_items.push("All entered Orders and Memoranda of Decision".to_string());
        requested_items.push("Certified hearing recordings or transcripts for recorded oral arguments".to_string());
    }

    let state = state_override
        .map(String::from)
        .or_else(|| {
            if !profile.governing_state.is_empty() {
                Some(profile.governing_state.clone())
            } else if profile.court.name.to_ascii_uppercase().contains("NEW JERSEY") {
                Some("NJ".into())
            } else if profile.court.name.to_ascii_uppercase().contains("COLORADO") {
                Some("CO".into())
            } else {
                Some("FED".into())
            }
        })
        .unwrap_or_else(|| "NJ".into());

    let (fee_notice, submit_method, form_ref, instructions) = match state.to_ascii_uppercase().as_str() {
        "NJ" => (
            "New Jersey Judiciary Form CN 10200 Copy Fees: 5¢ per letter-size page; 7¢ per legal-size page. Electronic delivery via email/link incurs no media charges unless special formatting is required.".to_string(),
            "Submit to the Trial Court Administrator / Division Clerk where the matter is pending, or upload via JEDS.".to_string(),
            "NJ Judiciary Form CN 10200 (Records Request Form)".to_string(),
            "Under NJ Court Rules, court records are presumed open for public inspection unless sealed or impounded by court order. Specify exact docket number and request electronic delivery to avoid paper copy surcharges.".to_string(),
        ),
        "CO" => (
            "Colorado Judicial Branch Records Request: Actual cost of copying. Self-represented litigants may inspect files in clerk's office at no charge; copies provided under Chief Justice Directive 05-01 fee schedule.".to_string(),
            "Submit to the Clerk of Court for the relevant District or County Court.".to_string(),
            "Colorado Judicial Records Request Form (CJD 05-01)".to_string(),
            "Submit to the supervising court clerk. Request email transmission of PDF copies if available.".to_string(),
        ),
        _ => (
            "Federal Court Records / PACER: General charge is $0.10 per page, capped at $3.00 for any single document. PACER fees are completely waived if usage is under $30.00 in a quarterly billing cycle.".to_string(),
            "Access via PACER portal or request physical file jacket from Clerk of Court.".to_string(),
            "AO Form / Clerk Docket Inspection".to_string(),
            "Check whether quarterly PACER $30 exemption applies to your account before requesting clerk copies.".to_string(),
        ),
    };

    let filer_name = if !profile.filer.name.is_empty() {
        profile.filer.name.clone()
    } else {
        profile.caption.plaintiffs.first().map(|p| p.name.clone()).unwrap_or_else(|| "Requester (Self-Represented)".to_string())
    };

    let filer_addr = profile.filer.address_lines.join(", ");
    let filer_phone = profile.filer.phone.clone();
    let filer_email = profile.filer.email.clone();

    let mut body = String::new();
    body.push_str(&format!("OFFICIAL COURT RECORDS REQUEST ({})\n", form_ref));
    body.push_str(&format!("TO: Clerk of the Court / Records Custodian, {}\n", profile.court.name));
    body.push_str(&format!("RE: Case Caption: {}\n", profile.title));
    body.push_str(&format!("    Docket Number: {}\n\n", profile.docket_number));
    body.push_str("REQUESTER INFORMATION:\n");
    body.push_str(&format!("  Name: {}\n", filer_name));
    body.push_str(&format!("  Address: {}\n", filer_addr));
    body.push_str(&format!("  Telephone: {}\n", filer_phone));
    body.push_str(&format!("  Email: {}\n\n", filer_email));
    body.push_str("RECORDS SOUGHT:\n");
    for (i, item) in requested_items.iter().enumerate() {
        body.push_str(&format!("  {}. {}\n", i + 1, item));
    }
    body.push_str("\nDELIVERY METHOD PREFERRED: Electronic Delivery via Email / Secure Link\n");
    body.push_str(&format!("FEE NOTICE: {}\n", fee_notice));
    body.push_str("Please provide an estimate of any copying fees exceeding $15.00 prior to processing.\n\n");
    body.push_str(&format!("Respectfully submitted,\n/s/ {}\nDate: {}\n", filer_name, chrono::Utc::now().format("%Y-%m-%d")));

    Ok(CourtRecordsRequestPacket {
        state,
        court_name: profile.court.name.clone(),
        division: profile.court.division.clone().unwrap_or_default(),
        docket_number: profile.docket_number.clone(),
        caption: profile.title.clone(),
        requester_name: filer_name,
        requester_address: filer_addr,
        requester_phone: filer_phone,
        requester_email: filer_email,
        records_requested: requested_items,
        preferred_format: "Electronic (Email / PDF link)".to_string(),
        statutory_fee_notice: fee_notice,
        submission_method: submit_method,
        instructions,
        rendered_request_text: body,
    })
}
