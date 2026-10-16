use super::types::{AdaPetitionReport, AdaRequestPayload};
use crate::case_profile::CaseProfile;

pub fn generate_ada_petition(
    profile: &CaseProfile,
    payload: &AdaRequestPayload,
) -> Result<AdaPetitionReport, String> {
    let state = if !profile.governing_state.is_empty() {
        profile.governing_state.to_ascii_uppercase()
    } else if profile.court.name.to_ascii_uppercase().contains("NEW JERSEY") {
        "NJ".to_string()
    } else if profile.court.name.to_ascii_uppercase().contains("COLORADO") {
        "CO".to_string()
    } else {
        "FED".to_string()
    };

    let (coordinator_title, state_policy, submission_guide) = match state.as_str() {
        "NJ" => (
            "Trial Court Administrator / Vicinage ADA Coordinator".to_string(),
            "New Jersey Judiciary Policy on Court Access for Persons with Disabilities (Directive #01-19) & N.J. Court Rule 1:33".to_string(),
            "Submit directly to the Trial Court Administrator or Municipal Division Manager at least 10 days prior to the proceeding. Requests are maintained confidentially and do not require disclosing underlying medical diagnoses to opposing counsel.".to_string(),
        ),
        "CO" => (
            "Managing Court Clerk / District ADA Coordinator".to_string(),
            "Colorado Judicial Branch ADA Policy & Chief Justice Directive 06-01".to_string(),
            "Submit to the District Court Administrator / ADA Coordinator at least 5 business days before the scheduled proceeding.".to_string(),
        ),
        _ => (
            "Federal Court Clerk / Circuit ADA Coordinator".to_string(),
            "Title II of the Americans with Disabilities Act (42 U.S.C. § 12132) & 28 C.F.R. § 35.130".to_string(),
            "Submit to the Clerk of Court / Court Operations Division.".to_string(),
        ),
    };

    let authorities = vec![
        "Title II of the Americans with Disabilities Act (42 U.S.C. § 12132)".to_string(),
        "U.S. Department of Justice Title II Regulations (28 C.F.R. Part 35)".to_string(),
        state_policy.clone(),
    ];

    let mut body = String::new();
    body.push_str("CONFIDENTIAL REQUEST FOR ADA ADMINISTRATIVE ACCOMMODATION\n");
    body.push_str("PRIVILEGED MEDICAL/DISABILITY COMMUNICATION — NOT FOR PUBLIC FILING\n\n");
    body.push_str(&format!("TO: {}\n", coordinator_title));
    body.push_str(&format!("    {}\n", profile.court.name));
    body.push_str(&format!("RE: Case Caption: {}\n", profile.title));
    body.push_str(&format!("    Docket Number: {}\n", profile.docket_number));
    if let Some(date) = &payload.upcoming_date {
        body.push_str(&format!("    Upcoming Proceeding Date: {}\n", date));
    }
    body.push('\n');

    body.push_str("1. REQUESTER IDENTIFICATION\n");
    body.push_str(&format!("   Name: {}\n", payload.requester_name));
    body.push_str(&format!("   Role in Proceeding: {}\n", payload.requester_role));
    body.push_str(&format!("   Phone: {}\n", payload.contact_phone));
    body.push_str(&format!("   Email: {}\n\n", payload.contact_email));

    body.push_str("2. STATUTORY BASIS & EQUAL ACCESS MANDATE\n");
    body.push_str("   This administrative request is submitted pursuant to Title II of the Americans with Disabilities\n");
    body.push_str("   Act (42 U.S.C. § 12132), 28 C.F.R. § 35.130(b)(7) (reasonable modifications of policies),\n");
    body.push_str(&format!("   and the {}.\n\n", state_policy));

    body.push_str("3. SPECIFIC ACCOMMODATIONS SOUGHT\n");
    for (i, acc) in payload.accommodations.iter().enumerate() {
        body.push_str(&format!("   {}. {}\n", i + 1, acc.label()));
    }
    body.push('\n');

    body.push_str("4. FUNCTIONAL LIMITATION & PRACTICAL NEED\n");
    body.push_str(&format!("   {}\n\n", payload.specific_needs_narrative.trim()));

    body.push_str("5. CONFIDENTIALITY NOTICE\n");
    body.push_str("   Under judicial administrative guidelines, this request and any accompanying medical documentation\n");
    body.push_str("   are confidential administrative records and must remain sealed from the public docket and\n");
    body.push_str("   protected from disclosure to opposing counsel without the requester's consent.\n\n");

    body.push_str(&format!("Respectfully submitted,\n/s/ {}\nDate: {}\n", payload.requester_name, chrono::Utc::now().format("%Y-%m-%d")));

    Ok(AdaPetitionReport {
        state,
        court_name: profile.court.name.clone(),
        docket_number: profile.docket_number.clone(),
        ada_coordinator_title: coordinator_title,
        statutory_authorities: authorities,
        rendered_petition_text: body,
        submission_guidelines: submission_guide,
    })
}
