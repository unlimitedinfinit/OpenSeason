//! Jurisdiction-Specific Public Records & Licensing FOIA Request Generator.

use std::path::Path;
use super::types::*;
use crate::case_profile::CaseProfile;

pub fn generate_records_pack(
    _case_dir: &Path,
    profile: &CaseProfile,
    target_state: Option<&str>,
) -> Result<RecordsPackRequest, String> {
    let state_code = target_state
        .unwrap_or(if !profile.governing_state.is_empty() {
            &profile.governing_state
        } else {
            "FEDERAL"
        })
        .to_uppercase();

    let target_name = if !profile.caption.defendants.is_empty() {
        profile.caption.defendants[0].name.clone()
    } else {
        profile.title.clone()
    };

    let target_addr = if !profile.caption.defendants.is_empty() && !profile.caption.defendants[0].address.is_empty() {
        profile.caption.defendants[0].address.clone()
    } else {
        "[Target Facility Address]".to_string()
    };

    let filer_name = if !profile.filer.name.is_empty() {
        &profile.filer.name
    } else {
        "[Requester Name]"
    };

    let filer_email = if !profile.filer.email.is_empty() {
        &profile.filer.email
    } else {
        "[requester@example.com]"
    };

    let (statute_title, agency, deadline_days) = match state_code.as_str() {
        "MN" | "MINNESOTA" => (
            "Minnesota Government Data Practices Act (Minn. Stat. § 13.01 et seq.)",
            "Minnesota Department of Health / Department of Human Services (Data Practices Compliance Official)",
            "reasonable time (typically 10 business days)",
        ),
        "NJ" | "NEW JERSEY" => (
            "New Jersey Open Public Records Act (N.J.S.A. 47:1A-1 et seq.)",
            "New Jersey Department of Health / Custodian of Records",
            "7 business days",
        ),
        "CO" | "COLORADO" => (
            "Colorado Open Records Act (C.R.S. § 24-72-201 et seq.)",
            "Colorado Department of Public Health and Environment (CDPHE) Records Custodian",
            "3 working days",
        ),
        "CA" | "CALIFORNIA" => (
            "California Public Records Act (Cal. Gov. Code § 7920.000 et seq.)",
            "California Department of Public Health (CDPH) / Department of Social Services",
            "10 calendar days",
        ),
        _ => (
            "Freedom of Information Act (5 U.S.C. § 552)",
            "U.S. Department of Health and Human Services (HHS FOIA Officer) / State Oversight Agency",
            "20 working days",
        ),
    };

    let records_items = vec![
        format!(
            "1. FACILITY LICENSING APPLICATION & BLUEPRINTS: The initial and all renewal facility licensing applications, approved floor plans, authorized bed/client capacity, and certified operating hours for '{}' at {}.",
            target_name, target_addr
        ),
        format!(
            "2. ANNUAL FIRE & SAFETY OCCUPANCY INSPECTIONS: All annual fire marshal, building safety, and environmental health inspection reports, citations, or occupancy permits issued for the premises at {} from 2020 to present.",
            target_addr
        ),
        format!(
            "3. REGULATORY AUDITS & DEFICIENCY NOTICES: All state licensing surveys, statements of deficiencies (e.g. CMS Form 2567), plans of correction, and compliance audits pertaining to '{}'.",
            target_name
        ),
        format!(
            "4. PUBLIC CLAIMS & AGGREGATE ENROLLMENT SUMMARIES: Monthly public summary rosters of state Medicaid or Child Nutrition program payments and participant enrollment counts issued to '{}' from 2021 to present (excluding patient-identifying private health data).",
            target_name
        ),
    ];

    let letter_text = format!(
        "PUBLIC RECORDS REQUEST UNDER {statute_title}\n\n\
        DATE: [Current Date]\n\
        TO: Records Access Officer / Custodian of Government Records\n\
        AGENCY: {agency}\n\n\
        RE: Public Records Request Concerning Facility Licensing and Public Payments for:\n\
            Target Entity: {target_name}\n\
            Physical Location: {target_addr}\n\n\
        Dear Records Custodian,\n\n\
        Pursuant to the {statute_title}, I hereby request access to and electronic copies of the following public government records in your possession or control:\n\n\
        {items}\n\n\
        STATUTORY FEE WAIVER REQUEST:\n\
        This request is submitted in the public interest to shed light on government operations and public health compliance under Executive Order 14395 and related transparency initiatives. It is not submitted for private commercial gain. I request that all reproduction or search fees be waived. If fees are anticipated to exceed $25.00, please notify me in advance.\n\n\
        TIMEFRAME & FORMAT:\n\
        Under the {statute_title}, a response is required within {deadline_days}. Please transmit all responsive documents electronically in searchable PDF format to {filer_email}.\n\n\
        Thank you for your assistance in maintaining public integrity and government transparency.\n\n\
        Respectfully submitted,\n\n\
        /s/ {filer_name}\n\
        {filer_name}\n\
        Email: {filer_email}\n",
        statute_title = statute_title,
        agency = agency,
        target_name = target_name,
        target_addr = target_addr,
        items = records_items.join("\n\n"),
        deadline_days = deadline_days,
        filer_name = filer_name,
        filer_email = filer_email
    );

    Ok(RecordsPackRequest {
        state: state_code,
        statute_title: statute_title.to_string(),
        target_agency: agency.to_string(),
        target_entity_name: target_name,
        target_entity_address: target_addr,
        records_items,
        formal_letter_text: letter_text,
    })
}
