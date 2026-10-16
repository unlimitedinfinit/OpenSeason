use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::case_profile::CaseProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPaths {
    pub docx: std::path::PathBuf,
    pub pdf: std::path::PathBuf,
}

pub(crate) struct Para {
    pub text: String,
    pub bold: bool,
    pub center: bool,
    pub size_half_points: u32,
}

impl Para {
    pub fn centered(text: impl Into<String>, size: u32, bold: bool) -> Self {
        Self {
            text: text.into(),
            bold,
            center: true,
            size_half_points: size,
        }
    }
}

pub(crate) fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn typst_escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('@', "\\@")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('<', "\\<")
        .replace('>', "\\>")
}

pub(crate) fn party_side_label(kind: &str, left: bool) -> &'static str {
    match (kind, left) {
        ("complaint", true) | ("motion", true) => "Plaintiff(s),",
        ("complaint", false) | ("motion", false) => "Defendant(s).",
        (_, true) => "Appellant(s),",
        (_, false) => "Appellee(s).",
    }
}

pub fn pleading_heading(kind: &str) -> &'static str {
    match kind.trim().to_ascii_lowercase().as_str() {
        "complaint" => "COMPLAINT",
        "appellate_brief" | "brief" => "APPELLATE BRIEF",
        "appellate_motion" => "MOTION IN THE APPELLATE COURT",
        "mandamus" => "PETITION FOR RULE TO SHOW CAUSE",
        "will" => "LAST WILL AND TESTAMENT",
        "lease" => "RESIDENTIAL LEASE AGREEMENT",
        "contractor" => "INDEPENDENT CONTRACTOR AGREEMENT",
        "power_of_attorney" => "DURABLE POWER OF ATTORNEY",
        "demand_letter" => "FORMAL DEMAND LETTER",
        "cure_notice" => "STATUTORY NOTICE TO CURE",
        "security_deposit_demand" => "STATUTORY SECURITY DEPOSIT DEMAND",
        "records_request" => "PUBLIC RECORDS REQUEST (OPRA / CORA / FOIA)",
        "agency_appeal" => "ADMINISTRATIVE AGENCY APPEAL",
        "motion" | "reconsideration" | "enlargement" | "stay" | "amend_findings" => "MOTION",
        _ => "NOTICE OF APPEAL",
    }
}

#[allow(dead_code)]
pub(crate) fn motion_kind_label(kind: &str, profile: &CaseProfile) -> String {
    let key = if kind == "motion" && !profile.motion_kind.is_empty() {
        profile.motion_kind.as_str()
    } else {
        kind
    };
    match key {
        "reconsideration" => "Motion for reconsideration (basic template)".into(),
        "enlargement" => "Motion for enlargement of time (basic template)".into(),
        "stay" => "Motion for a stay (basic template)".into(),
        "amend_findings" => "Motion to amend findings (basic template)".into(),
        "appellate_motion" => "Motion in the appellate court (basic template)".into(),
        "appellate_brief" | "brief" => "Appellate brief scaffold (basic)".into(),
        "mandamus" => "Colorado C.A.R. 21 petition (basic template)".into(),
        "complaint" => "Complaint (basic template)".into(),
        "will" => "Last Will & Testament with Self-Proving Affidavit scaffold".into(),
        "lease" => "Residential Lease Agreement scaffold".into(),
        "contractor" => "Independent Contractor / MSA Agreement scaffold".into(),
        "power_of_attorney" => "Durable Power of Attorney scaffold".into(),
        "demand_letter" => "Pre-Suit Demand Letter scaffold".into(),
        "cure_notice" => "Statutory Notice to Cure scaffold".into(),
        "security_deposit_demand" => "Statutory Security Deposit Demand scaffold".into(),
        "records_request" => "Open Public Records Request (OPRA / CORA / FOIA) scaffold".into(),
        "agency_appeal" => "Administrative Agency Appeal scaffold".into(),
        _ => "Motion (basic template)".into(),
    }
}

pub(crate) fn pleading_body<'a>(profile: &'a CaseProfile, kind: &str) -> (&'a str, &'a str) {
    match kind {
        "complaint" => (profile.complaint.title.as_str(), profile.complaint.user_text.as_str()),
        "appellate_brief" | "brief" => (profile.brief.title.as_str(), profile.brief.user_text.as_str()),
        "appellate_motion" => (
            profile.appellate_motion.title.as_str(),
            profile.appellate_motion.user_text.as_str(),
        ),
        "mandamus" => (profile.mandamus.title.as_str(), profile.mandamus.user_text.as_str()),
        _ => (profile.motion.title.as_str(), profile.motion.user_text.as_str()),
    }
}

#[allow(dead_code)]
pub(crate) fn scaffold_sections(kind: &str) -> &'static [&'static str] {
    match kind {
        "appellate_brief" | "brief" => &[
            "Table of contents (user supplies entries)",
            "Table of authorities (filled from cites the user wrote; unverified cites stay tagged)",
            "Preliminary statement",
            "Procedural history",
            "Statement of facts (each fact needs a record cite)",
            "Legal argument (user-supplied; this app does not invent points)",
            "Conclusion",
        ],
        "mandamus" => &[
            "Jurisdiction (C.A.R. 21)",
            "Parties",
            "Issues presented",
            "Facts from the record",
            "Reasons relief should issue (user-supplied)",
            "Prayer for relief",
        ],
        "reconsideration" => &[
            "The order challenged",
            "The rule relied on (user must verify)",
            "Grounds as written by the user",
            "Relief requested",
        ],
        "enlargement" => &[
            "The deadline that needs more time",
            "The reason for the request (user-supplied)",
            "The new date requested",
        ],
        "stay" => &[
            "The order to be stayed",
            "Why a stay is requested (user-supplied)",
            "The terms of the stay requested",
        ],
        "amend_findings" => &[
            "The finding to be amended",
            "The proposed amendment (user-supplied)",
            "Record support",
        ],
        "will" => &[
            "Declaration of Testator & Revocation of Prior Wills",
            "Specific Bequests & Residuary Estate Distribution",
            "Appointment of Executor / Personal Representative",
            "Two-Witness Attestation & Statutory Self-Proving Affidavit",
        ],
        "lease" => &[
            "Premises, Term & Monthly Rent",
            "Security Deposit & Statutory Trust Notice",
            "Maintenance, Habitability & Utilities",
            "Default, Cure Period & Termination Notice",
        ],
        "contractor" => &[
            "Scope of Services & Deliverables",
            "Compensation & Payment Milestones",
            "Intellectual Property Ownership & Confidentiality",
            "Independent Contractor Status & Termination",
        ],
        "demand_letter" | "cure_notice" | "security_deposit_demand" => &[
            "Summary of Obligation or Statutory Violation",
            "Chronology of Events & Supporting Proof",
            "Specific Cure / Payment Demand & Statutory Deadline",
            "Reservation of Rights & Intent to Escalate",
        ],
        "records_request" | "agency_appeal" => &[
            "Statutory Authority (OPRA / CORA / FOIA / Agency Rule)",
            "Specific Records Requested or Determination Challenged",
            "Fee Waiver / Public Interest Statement",
        ],
        _ => &[],
    }
}

pub(crate) fn assert_notice_out_dir(out: &Path) -> Result<(), String> {
    let name = out.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if crate::sandbox::is_reserved_marker_name(name) {
        return Err("Export cannot write to a path named .sealed.".to_string());
    }
    let app_data = crate::sandbox::default_app_data_root();
    let resolved = crate::sandbox::canonicalize_existing_prefix(out);
    if app_data.exists() {
        if let Ok(app_canon) = app_data.canonicalize() {
            if resolved == app_canon || resolved.starts_with(&app_canon) {
                return Err("Export cannot write inside the app data folder.".to_string());
            }
        }
    }
    Ok(())
}
