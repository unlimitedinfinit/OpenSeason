//! Guided drafting sections. Same prompts for the window and the agent CLI.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::case_profile;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuidedSection {
    pub id: String,
    pub heading: String,
    pub hint: String,
    #[serde(default)]
    pub user_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuidedDraft {
    pub kind: String,
    pub sections: Vec<GuidedSection>,
}

pub fn templates_for(kind: &str) -> Vec<GuidedSection> {
    let key = kind.trim().to_ascii_lowercase();
    let rows: &[(&str, &str, &str)] = match key.as_str() {
        "enlargement" => &[
            ("deadline", "The deadline that needs more time", "What paper is due, and on what date? Write only what you know."),
            ("reason", "Why you need more time", "Your own reason. Do not invent facts. If you do not know, leave this blank."),
            ("new_date", "The new date you are asking for", "A date you are requesting. This app does not decide if the court will grant it."),
        ],
        "stay" => &[
            ("order", "The order you want paused", "Name the order and the date it was entered. Use the record, not a guess."),
            ("why", "Why a stay is needed", "Your own words. This app will not invent harm or urgency."),
            ("terms", "What you want the stay to do", "What should be paused, and for how long, in your own words."),
        ],
        "amend_findings" => &[
            ("finding", "The finding you want changed", "Quote or point to the finding. Do not invent a finding the court did not make."),
            ("amendment", "The change you are asking for", "The words you want the court to use instead."),
            ("record", "Where the record supports the change", "Exhibit letter or transcript page. This checker will not invent a cite."),
        ],
        "appellate_brief" | "brief" => &[
            ("prelim", "Preliminary statement", "Two or three sentences in your own words about what this appeal is."),
            ("history", "Procedural history", "What was filed and what the trial court did. Dates from the record only."),
            ("facts", "Statement of facts", "Each fact needs a record cite. Leave a fact out rather than guessing."),
            ("argument", "Legal argument", "Points you researched. Do not invent case citations."),
            ("conclusion", "Conclusion", "The relief you want, in your own words."),
        ],
        "appellate_motion" => &[
            ("relief", "What you want the appellate court to do", "One sentence. This is a basic R. 2:8-1 form, not a brief."),
            ("grounds", "Your grounds", "Your own reasons. Do not invent rules or cases."),
            ("record", "Record support", "Exhibits or docket entries you already have."),
        ],
        "mandamus" => &[
            ("jurisdiction", "Why this court can hear this", "C.A.R. 21 is a public rule number, not legal advice. Write only what you know."),
            ("issues", "Issues presented", "The questions you want answered. Do not invent them."),
            ("facts", "Facts from the record", "Each fact needs a cite. The Colorado pack will not invent the showing."),
            ("reasons", "Reasons relief should issue", "Your own words."),
            ("prayer", "Prayer for relief", "What you want the court to order."),
        ],
        "complaint" => &[
            ("parties", "Who is suing whom", "Names you already entered in the caption."),
            ("claim", "What happened", "Your own statement of the claim. This is not a full interview."),
            ("relief", "What you want the court to do", "Your own request."),
        ],
        "notice_of_appeal" => &[
            ("order", "The order or judgment you are appealing", "Date and a short description from the record."),
            ("notice", "Your notice text", "Your own words. This app will not invent grounds of appeal."),
        ],
        "will" => &[
            ("declaration", "Declaration of Testator & Family Status", "State your full legal name, county/state of residence, and revoke prior wills in your own words."),
            ("bequests", "Specific Bequests & Residuary Distribution", "List specific gifts and who receives the remainder of your estate. Do not leave placeholders."),
            ("executor", "Appointment of Executor / Personal Representative", "Name your primary and alternate Personal Representative / Executor."),
            ("attestation", "Witness & Self-Proving Affidavit Instructions", "State the governing state (e.g., NJ N.J.S.A. 3B:3-4 or CO C.R.S. § 15-11-504) for two-witness and notary execution."),
        ],
        "lease" => &[
            ("premises_term", "Premises, Lease Term & Monthly Rent", "Property address, start/end dates, monthly rent amount, and due date."),
            ("security_deposit", "Security Deposit & Statutory Trust Terms", "Deposit amount, bank/trust holding details, and statutory return period."),
            ("habitability", "Maintenance, Utilities & Habitability", "Allocation of utilities, repairs, and statutory warranty of habitability."),
            ("default_notice", "Default, Cure Period & Termination Notice", "Notice period required before termination or eviction filing."),
        ],
        "contractor" => &[
            ("scope", "Scope of Services & Deliverables", "Concrete description of services and milestones to be performed."),
            ("compensation", "Compensation, Invoicing & Payment Schedule", "Rates, milestone payment triggers, and late-fee terms."),
            ("ip_ownership", "Intellectual Property & Confidentiality", "Who owns work product upon full payment and confidentiality obligations."),
            ("termination", "Independent Contractor Relationship & Termination", "Notice period to terminate and non-employment tax status."),
        ],
        "power_of_attorney" => &[
            ("principal_agent", "Principal & Designated Agent (Attorney-in-Fact)", "Your full name and the name/address of your chosen Agent and Successor Agent."),
            ("powers_granted", "Specific Financial or Legal Powers Granted", "List the exact powers granted in your own words."),
            ("durability", "Effective Date, Durability & Revocation Terms", "Whether this power remains effective upon disability/incapacity."),
        ],
        "demand_letter" | "cure_notice" | "security_deposit_demand" => &[
            ("obligation", "Underlying Agreement or Statutory Duty", "Identify the lease, contract, or statute (e.g., 7-day / 30-day demand rule)."),
            ("breach_chronology", "Chronology of Default or Non-Payment", "Dates and exact amounts or violations supported by your records."),
            ("cure_demand", "Exact Cure / Payment Required & Statutory Deadline", "Exact dollar amount or action demanded and the date the cure clock expires."),
            ("escalation_notice", "Delivery Method & Notice of Legal Escalation", "Certified Mail RRR tracking info and statement that failure to cure will result in a Tier 3 court action."),
        ],
        "records_request" | "agency_appeal" => &[
            ("authority", "Governing Statute or Agency Rule", "e.g., NJ OPRA (N.J.S.A. 47:1A-1), CO CORA (C.R.S. § 24-72-201), FOIA, or Unemployment Rule."),
            ("records_or_ruling", "Specific Records Requested or Agency Determination Challenged", "Date Range, record custodian, or agency docket/decision date."),
            ("relief", "Fee Waiver / Production Format or Administrative Relief Sought", "Electronic PDF delivery request or reversal of agency determination."),
        ],
        _ => &[
            ("order", "The order you want the court to look at again", "Date of the order and what it did. Copy from the paper you have."),
            ("rule", "The rule you are using", "If you do not have a rule, leave this blank. The app will not invent one. NJ reconsideration is often R. 2:11-6 or R. 4:49-2. Confirm the current rule yourself."),
            ("grounds", "Why the order is wrong, in your own words", "Write only facts you can point to in an exhibit. Do not invent."),
            ("relief", "What you want the court to do", "For example, vacate the order and restore the case. Your words, not ours."),
        ],
    };
    rows.iter()
        .map(|(id, heading, hint)| GuidedSection {
            id: (*id).into(),
            heading: (*heading).into(),
            hint: (*hint).into(),
            user_text: String::new(),
        })
        .collect()
}

pub fn path(case_dir: &Path) -> std::path::PathBuf {
    case_dir.join("drafts").join("sections.json")
}

pub fn scoped_path(case_dir: &Path, filing_id: Option<&str>) -> std::path::PathBuf {
    match filing_id.map(|s| s.trim()).filter(|s| !s.is_empty()) {
        Some(fid) => case_dir.join("drafts").join(format!("sections-{fid}.json")),
        None => path(case_dir),
    }
}

pub fn load_or_init(case_dir: &Path, kind: &str) -> Result<GuidedDraft, String> {
    load_or_init_scoped(case_dir, None, kind)
}

pub fn load_or_init_scoped(
    case_dir: &Path,
    filing_id: Option<&str>,
    kind: &str,
) -> Result<GuidedDraft, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Guided drafts cannot be written on the Documents pointer.".into());
    }
    let target = scoped_path(case_dir, filing_id);
    if let Some(existing) = crate::check::read_json::<GuidedDraft>(&target)? {
        if existing.kind.eq_ignore_ascii_case(kind) || kind.trim().is_empty() {
            return Ok(existing);
        }
    }
    let draft = GuidedDraft {
        kind: kind.to_string(),
        sections: templates_for(kind),
    };
    fs::create_dir_all(case_dir.join("drafts")).map_err(|e| e.to_string())?;
    crate::check::write_json(&target, &draft)?;
    Ok(draft)
}

pub fn save(case_dir: &Path, draft: &GuidedDraft) -> Result<(), String> {
    fs::create_dir_all(case_dir.join("drafts")).map_err(|e| e.to_string())?;
    crate::check::write_json(&path(case_dir), draft)
}

pub fn assemble(sections: &[GuidedSection]) -> String {
    let mut out = String::new();
    for section in sections {
        if section.user_text.trim().is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(&section.heading);
        out.push('\n');
        out.push_str(section.user_text.trim());
    }
    out
}

/// Write one section and copy the assembled body onto the matching profile field (or scoped filing).
pub fn set_section(case_dir: &Path, id: &str, text: &str) -> Result<GuidedDraft, String> {
    set_section_scoped(case_dir, None, id, text)
}

pub fn set_section_scoped(
    case_dir: &Path,
    filing_id: Option<&str>,
    id: &str,
    text: &str,
) -> Result<GuidedDraft, String> {
    let mut profile = case_profile::load_case(case_dir)?;
    let kind = if let Some(fid) = filing_id.filter(|s| !s.trim().is_empty()) {
        profile
            .filings
            .iter()
            .find(|f| f.id == fid)
            .map(|f| f.kind.clone())
            .unwrap_or_else(|| profile.document_kind.clone())
    } else if profile.document_kind.is_empty() {
        "reconsideration".to_string()
    } else {
        profile.document_kind.clone()
    };
    let mut draft = load_or_init_scoped(case_dir, filing_id, &kind)?;
    let known: Vec<String> = draft.sections.iter().map(|s| s.id.clone()).collect();
    let section = draft.sections.iter_mut().find(|s| s.id == id).ok_or_else(|| {
        format!(
            "No section '{id}'. Known ids: {}. Use draft sections to list them.",
            known.join(", ")
        )
    })?;
    section.user_text = text.to_string();
    let target = scoped_path(case_dir, filing_id);
    fs::create_dir_all(case_dir.join("drafts")).map_err(|e| e.to_string())?;
    crate::check::write_json(&target, &draft)?;
    let body = assemble(&draft.sections);

    if let Some(fid) = filing_id.filter(|s| !s.trim().is_empty()) {
        if let Some(filing) = profile.filings.iter_mut().find(|f| f.id == fid) {
            filing.user_text = body.clone();
            let md_path = case_dir.join("drafts").join(format!("motion-{fid}.md"));
            let _ = fs::write(md_path, &body);
        }
    } else {
        match draft.kind.to_ascii_lowercase().as_str() {
            "appellate_brief" | "brief" => profile.brief.user_text = body,
            "appellate_motion" => profile.appellate_motion.user_text = body,
            "mandamus" => profile.mandamus.user_text = body,
            "complaint" => profile.complaint.user_text = body,
            "notice_of_appeal" => profile.notice_of_appeal.user_text = body,
            _ => profile.motion.user_text = body,
        }
    }
    case_profile::save_case(case_dir, &profile)?;
    Ok(draft)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_profile::{ensure_case_layout, sample_appeal_profile, save_case};
    use uuid::Uuid;

    #[test]
    fn reconsideration_has_order_rule_grounds_relief() {
        let ids: Vec<_> = templates_for("reconsideration").into_iter().map(|s| s.id).collect();
        assert_eq!(ids, vec!["order", "rule", "grounds", "relief"]);
    }

    #[test]
    fn filling_a_section_writes_the_motion_body() {
        let dir = std::env::temp_dir().join(format!("os-guided-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        ensure_case_layout(&dir).unwrap();
        let mut profile = sample_appeal_profile();
        profile.document_kind = "reconsideration".into();
        save_case(&dir, &profile).unwrap();
        set_section(&dir, "grounds", "The clerk mailed the order on March 1.").unwrap();
        let loaded = case_profile::load_case(&dir).unwrap();
        assert!(loaded.motion.user_text.contains("The clerk mailed the order on March 1."));
        let _ = fs::remove_dir_all(dir);
    }
}
