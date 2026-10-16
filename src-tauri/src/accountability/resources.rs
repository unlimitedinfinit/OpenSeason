use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountabilityResource {
    pub id: String,
    pub title: String,
    pub agency: String,
    pub category: String, // "judicial", "attorney", "records", "ada"
    pub state: String,
    pub form_number: Option<String>,
    pub direct_pdf_url: Option<String>,
    pub portal_url: String,
    pub fee_schedule: String,
    pub jurisdictional_scope: String,
    pub non_appeal_warning: String,
}

pub fn list_accountability_resources(state: Option<&str>, category: Option<&str>) -> Vec<AccountabilityResource> {
    let mut all = vec![
        AccountabilityResource {
            id: "nj-acjc-11166".into(),
            title: "New Jersey ACJC Judicial Misconduct Complaint".into(),
            agency: "Advisory Committee on Judicial Conduct".into(),
            category: "judicial".into(),
            state: "NJ".into(),
            form_number: Some("CN 11166".into()),
            direct_pdf_url: Some("https://www.njcourts.gov/sites/default/files/forms/11166_acjc_complaint_form.pdf".into()),
            portal_url: "https://www.njcourts.gov/public/acjc".into(),
            fee_schedule: "No filing fee.".into(),
            jurisdictional_scope: "Investigates ethical violations by NJ state judges (bias, conflict of interest, harassment, improper ex parte communications). Cannot change rulings or overturn orders.".into(),
            non_appeal_warning: "DOES NOT TOLL COURT APPEAL DEADLINES. An adverse ruling or factual error is an appellate issue, NOT judicial misconduct.".into(),
        },
        AccountabilityResource {
            id: "nj-oae-ethics".into(),
            title: "New Jersey Attorney Ethics Grievance".into(),
            agency: "Office of Attorney Ethics (OAE) / District Ethics Committees".into(),
            category: "attorney".into(),
            state: "NJ".into(),
            form_number: Some("OAE Grievance Form".into()),
            direct_pdf_url: Some("https://www.njcourts.gov/attorneys/oae".into()),
            portal_url: "https://www.njcourts.gov/attorneys/oae".into(),
            fee_schedule: "No fee.".into(),
            jurisdictional_scope: "Investigates RPC violations by licensed attorneys (dishonesty, deceit, client neglect, trust accounting).".into(),
            non_appeal_warning: "Grievances do not stay pending civil or criminal litigation.".into(),
        },
        AccountabilityResource {
            id: "nj-court-records-10200".into(),
            title: "NJ Judiciary Court Records Request Form".into(),
            agency: "Clerk of Superior Court / Records Management".into(),
            category: "records".into(),
            state: "NJ".into(),
            form_number: Some("CN 10200".into()),
            direct_pdf_url: Some("https://www.njcourts.gov/sites/default/files/10200recordsreq_0.pdf".into()),
            portal_url: "https://www.njcourts.gov/faq/what-electronic-case-jacket".into(),
            fee_schedule: "5¢ per letter-size page, 7¢ per legal-size page. Electronic delivery incurs no media fee.".into(),
            jurisdictional_scope: "Access to filed case jackets, orders, pleadings, and docket indices under Rule 1:38.".into(),
            non_appeal_warning: "Obtaining records does not automatically extend missed court filing deadlines.".into(),
        },
        AccountabilityResource {
            id: "co-cjd-discipline".into(),
            title: "Colorado Judicial Discipline Complaint".into(),
            agency: "Colorado Commission on Judicial Discipline".into(),
            category: "judicial".into(),
            state: "CO".into(),
            form_number: Some("CJD Complaint".into()),
            direct_pdf_url: Some("https://www.coloradojudicialdiscipline.com/file-a-complaint".into()),
            portal_url: "https://www.coloradojudicialdiscipline.com".into(),
            fee_schedule: "No filing fee.".into(),
            jurisdictional_scope: "Investigates violations of the Colorado Code of Judicial Conduct. Does not review legal or factual correctness of rulings.".into(),
            non_appeal_warning: "Cannot substitute for an appeal. Rule 59 / CAR 4 deadlines must be filed separately.".into(),
        },
        AccountabilityResource {
            id: "fed-judicial-conduct-351".into(),
            title: "Federal Judicial Conduct & Disability Complaint".into(),
            agency: "Circuit Executive / Judicial Council of the Circuit".into(),
            category: "judicial".into(),
            state: "FED".into(),
            form_number: Some("28 U.S.C. § 351 Form".into()),
            direct_pdf_url: Some("https://www.uscourts.gov/judges-judgeships/judicial-conduct-disability".into()),
            portal_url: "https://www.uscourts.gov".into(),
            fee_schedule: "No fee.".into(),
            jurisdictional_scope: "Conduct prejudicial to effective and expeditious administration of court business. Explicitly excludes merits-related claims.".into(),
            non_appeal_warning: "Strict statutory rule: Claims directly related to the merits of a decision or procedural ruling are dismissed as merits-related under 28 U.S.C. § 352(b)(1)(A)(ii).".into(),
        },
    ];

    if let Some(st) = state {
        if !st.is_empty() && !st.eq_ignore_ascii_case("ALL") {
            all.retain(|r| r.state.eq_ignore_ascii_case(st) || r.state == "FED");
        }
    }

    if let Some(cat) = category {
        if !cat.is_empty() && !cat.eq_ignore_ascii_case("ALL") {
            all.retain(|r| r.category.eq_ignore_ascii_case(cat));
        }
    }

    all
}
