use super::types::CourtAccessProfile;

/// Returns bundled court access directory entries.
pub fn get_all_court_profiles() -> Vec<CourtAccessProfile> {
    vec![
        CourtAccessProfile {
            court_id: "nj-app-div".into(),
            court_name: "Superior Court of New Jersey, Appellate Division".into(),
            jurisdiction_state: "NJ".into(),
            physical_address: "Richard J. Hughes Justice Complex, 25 W. Market St., P.O. Box 006, Trenton, NJ 08625".into(),
            clerk_counter_hours: "Monday through Friday, 8:30 a.m. to 4:30 p.m. ET".into(),
            phone: "(609) 815-2950".into(),
            electronic_portal_name: "Judiciary Electronic Document Submission (JEDS) / eCourts".into(),
            electronic_portal_url: "https://www.njcourts.gov/public/jeds".into(),
            pro_se_efiling_allowed: true,
            docket_search_url: "https://www.njcourts.gov/public/case-search".into(),
            records_access_rule: "New Jersey Court Rule 1:38 (Public Access to Court and Administrative Records)".into(),
            transcript_request_form: "NJ Judiciary Appellate Transcript Request (Form CN 10200 / Transcript Request Form)".into(),
            transcript_deposit_rules: "Under R. 2:5-3(d), a non-refundable or estimated deposit is required with the transcript request (typically $300-$500 per half day of proceedings unless waived).".into(),
            statutory_fees_summary: "5¢ per letter page, 7¢ per legal page. Electronic delivery incurs zero media fee under CN 10200.".into(),
            indigent_fee_waiver_available: true,
            special_instructions: vec![
                "Pro se filers can submit records requests electronically via JEDS or mail Form CN 10200 to the Appellate Clerk.".into(),
                "Oral argument requests under R. 2:11-1(b) are separate from transcript orders under R. 2:5-3.".into(),
            ],
        },
        CourtAccessProfile {
            court_id: "nj-law-div".into(),
            court_name: "Superior Court of New Jersey, Law / Chancery Division (Trial Court)".into(),
            jurisdiction_state: "NJ".into(),
            physical_address: "County Superior Court Clerk's Office (Varies by County / Vicinage)".into(),
            clerk_counter_hours: "Monday through Friday, 8:30 a.m. to 4:30 p.m. ET".into(),
            phone: "Local County Clerk of Court".into(),
            electronic_portal_name: "JEDS / eCourts Civil".into(),
            electronic_portal_url: "https://www.njcourts.gov/public/jeds".into(),
            pro_se_efiling_allowed: true,
            docket_search_url: "https://www.njcourts.gov/public/case-search".into(),
            records_access_rule: "New Jersey Court Rule 1:38".into(),
            transcript_request_form: "County Reporting / Transcript Request Form (Form CN 10200)".into(),
            transcript_deposit_rules: "Deposit payable to the certified court transcriber per R. 2:5-3.".into(),
            statutory_fees_summary: "5¢/letter page, 7¢/legal page; free inspection in person.".into(),
            indigent_fee_waiver_available: true,
            special_instructions: vec![
                "Address requests to the Trial Court Administrator (TCA) for the specific vicinage where the matter was heard.".into(),
            ],
        },
        CourtAccessProfile {
            court_id: "co-court-appeals".into(),
            court_name: "Colorado Court of Appeals".into(),
            jurisdiction_state: "CO".into(),
            physical_address: "Ralph L. Carr Colorado Judicial Center, 2 East 14th Avenue, Denver, CO 80203".into(),
            clerk_counter_hours: "Monday through Friday, 8:00 a.m. to 5:00 p.m. MT".into(),
            phone: "(720) 625-5150".into(),
            electronic_portal_name: "Colorado Courts E-Filing (CCE) / Pro Se E-Filing Portal".into(),
            electronic_portal_url: "https://www.courts.state.co.us/Administration/Program.cfm?Program=39".into(),
            pro_se_efiling_allowed: true,
            docket_search_url: "https://www.courts.state.co.us/dockets/".into(),
            records_access_rule: "Chief Justice Directive 05-01 (Public Access to Court Records and Information)".into(),
            transcript_request_form: "JDF 4 Transcript Request Form".into(),
            transcript_deposit_rules: "Transcript requests must be filed in the trial court within 14 days of the notice of appeal under C.A.R. 10(b), accompanied by required transcriber deposit.".into(),
            statutory_fees_summary: "CJD 05-01: Cost recovery for physical duplication; electronic file inspection free at public terminals.".into(),
            indigent_fee_waiver_available: true,
            special_instructions: vec![
                "COLORADO CLERK RECEIPT RULE: In Colorado state appellate courts, the physical receipt by the clerk controls, NOT the postmark (Dobler v. CDOC 2026).".into(),
                "Trial record transmission is organized by the trial court clerk pursuant to C.A.R. 11.".into(),
            ],
        },
        CourtAccessProfile {
            court_id: "co-district-court".into(),
            court_name: "Colorado District Court (Judicial Districts 1-22)".into(),
            jurisdiction_state: "CO".into(),
            physical_address: "County Justice Center (Varies by Judicial District)".into(),
            clerk_counter_hours: "Monday through Friday, 7:30 a.m. or 8:00 a.m. to 4:30 p.m. MT".into(),
            phone: "Local Judicial District Clerk of Court".into(),
            electronic_portal_name: "CCE Pro Se Filing".into(),
            electronic_portal_url: "https://www.courts.state.co.us".into(),
            pro_se_efiling_allowed: true,
            docket_search_url: "https://www.courts.state.co.us/dockets/".into(),
            records_access_rule: "Colorado Chief Justice Directive 05-01".into(),
            transcript_request_form: "JDF 4 (Request for Transcript)".into(),
            transcript_deposit_rules: "Submitted directly to the Managing Court Reporter or Clerk with estimate deposit.".into(),
            statutory_fees_summary: "Actual copying cost; certified copies $20 per certification.".into(),
            indigent_fee_waiver_available: true,
            special_instructions: vec![
                "Audio recordings of hearings (FTR Gold or CourtCall) may be requested via JDF 4 if a certified reporter was not present.".into(),
            ],
        },
        CourtAccessProfile {
            court_id: "fed-district-court".into(),
            court_name: "United States District Court (Federal)".into(),
            jurisdiction_state: "FED".into(),
            physical_address: "United States Courthouse (Local Division)".into(),
            clerk_counter_hours: "Monday through Friday, 8:00 a.m. to 5:00 p.m. local time".into(),
            phone: "Local US District Clerk".into(),
            electronic_portal_name: "PACER / CM/ECF".into(),
            electronic_portal_url: "https://pacer.uscourts.gov".into(),
            pro_se_efiling_allowed: false, // Default requires pro se motion for e-filing privileges
            docket_search_url: "https://pcl.uscourts.gov".into(),
            records_access_rule: "Federal Rules of Civil Procedure 79 & Judicial Conference Electronic Public Access Policy".into(),
            transcript_request_form: "AO Form 435 (Transcript Order)".into(),
            transcript_deposit_rules: "AO 435 submitted to court reporter; per-page rate determined by Judicial Conference fee schedule (ordinary, 14-day, expedited, daily).".into(),
            statutory_fees_summary: "PACER $0.10/page capped at $3.00/document; quarterly fee waived if under $30.00 total.".into(),
            indigent_fee_waiver_available: true,
            special_instructions: vec![
                "Pro se litigants generally file in paper at the intake counter unless granted CM/ECF electronic filing authorization by motion.".into(),
                "PACER quarterly $30 exemption applies automatically if balance remains below the threshold.".into(),
            ],
        },
    ]
}

/// Looks up a court access profile by court id, court name substring, or state code.
pub fn find_court_access_profile(query: &str) -> Option<CourtAccessProfile> {
    let q = query.trim().to_ascii_lowercase();
    let all = get_all_court_profiles();

    // 1. Direct ID match
    if let Some(p) = all.iter().find(|p| p.court_id == q) {
        return Some(p.clone());
    }

    // 2. Name contains match
    if let Some(p) = all.iter().find(|p| p.court_name.to_ascii_lowercase().contains(&q)) {
        return Some(p.clone());
    }

    // 3. State match
    if let Some(p) = all.iter().find(|p| p.jurisdiction_state.to_ascii_lowercase() == q) {
        return Some(p.clone());
    }

    None
}
