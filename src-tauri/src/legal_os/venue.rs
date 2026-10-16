//! Venue resolution and competence validation for state, federal, and SCOTUS practice.

use crate::case_profile::{CaseProfile, ValidationIssue};
use super::types::{
    VenueCompetenceReport, VenueProfile, CO_CRS_13_27_106_OATH, FED_28_USC_1746_OATH, NJ_R_1_4_4B_OATH,
};

/// A 45-day notice of appeal clock does not apply once the case is already in a reviewing court.
pub fn already_in_reviewing_court(profile: &CaseProfile) -> bool {
    let court = profile.court.name.to_ascii_lowercase();
    let stage = profile.procedural_stage.to_ascii_lowercase();
    let div = profile
        .court
        .division
        .clone()
        .unwrap_or_default()
        .to_ascii_lowercase();
    stage == "appellate"
        || stage == "supreme_court"
        || court.contains("appellate")
        || court.contains("court of appeals")
        || court.contains("supreme court of")
        || div.contains("appellate")
}

/// Detect the active venue profile from CaseProfile fields.
pub fn resolve_venue(profile: &CaseProfile) -> VenueProfile {
    let state_raw = profile.governing_state.trim().to_uppercase();
    let pack_raw = profile.rule_pack_id.trim().to_lowercase();
    let court_raw = profile.court.name.trim().to_lowercase();
    let title_raw = profile.title.trim().to_lowercase();

    let code = if state_raw == "SCOTUS"
        || court_raw.contains("supreme court of the united states")
        || title_raw.contains("scotus")
    {
        "SCOTUS"
    } else if state_raw == "CO" || pack_raw.starts_with("co") || court_raw.contains("colorado") {
        "CO"
    } else if state_raw == "NJ"
        || pack_raw.starts_with("nj")
        || court_raw.contains("new jersey")
        || court_raw.contains("surrogate")
    {
        "NJ"
    } else if state_raw == "FEDERAL"
        || state_raw == "FED"
        || court_raw.contains("united states district")
        || court_raw.contains("united states court of appeals")
        || profile.docket_number.contains("-cv-")
        || profile.docket_number.contains("-CV-")
    {
        "FEDERAL"
    } else {
        "GENERIC"
    };

    match code {
        "NJ" => {
            let is_appellate = already_in_reviewing_court(profile)
                || court_raw.contains("appellate");
            VenueProfile {
                venue_code: "NJ".into(),
                display_name: if is_appellate {
                    "New Jersey Superior Court, Appellate Division".into()
                } else {
                    "New Jersey Superior / Surrogate Court".into()
                },
                sworn_statement_title: "Certification in Lieu of Oath".into(),
                sworn_statement_citation: "N.J. Ct. R. 1:4-4(b)".into(),
                mandatory_oath_text: NJ_R_1_4_4B_OATH.into(),
                requires_conferral_certificate: false,
                conferral_rule: "N.J. Ct. R. 1:6-2(c) (discovery motions)".into(),
                appeal_days: 45,
                reconsideration_days: if is_appellate { 10 } else { 20 },
                reconsideration_trigger: if is_appellate { "entry".into() } else { "service".into() },
                tolling_mode: "pause_remaining".into(),
                mail_service_adder_days: 3,
                packet_manifest: vec![
                    "Notice of Motion (R. 1:6-2 / R. 2:8-1)".into(),
                    "Certification of Facts & Exhibits (R. 1:6-6 & R. 1:4-4(b))".into(),
                    "Brief / Statement in Support".into(),
                    "Proposed Form of Order".into(),
                    "Certification of Service (R. 1:5-3)".into(),
                ],
            }
        }
        "CO" => {
            let is_appellate = already_in_reviewing_court(profile)
                || court_raw.contains("court of appeals");
            VenueProfile {
                venue_code: "CO".into(),
                display_name: "Colorado State Courts (C.R.C.P. / C.A.R.)".into(),
                sworn_statement_title: "Unsworn Declaration Under Penalty of Perjury".into(),
                sworn_statement_citation: "C.R.S. § 13-27-106 (needs_verification)".into(),
                mandatory_oath_text: CO_CRS_13_27_106_OATH.into(),
                requires_conferral_certificate: !is_appellate,
                conferral_rule: if is_appellate {
                    "C.A.R. may apply; C.R.C.P. 121 § 1-15(8) is trial-court civil practice (needs_verification)".into()
                } else {
                    "C.R.C.P. 121 § 1-15(8) (needs_verification)".into()
                },
                appeal_days: 49,
                reconsideration_days: 14,
                reconsideration_trigger: "entry".into(),
                tolling_mode: "reset_full".into(),
                mail_service_adder_days: 3,
                packet_manifest: vec![
                    "Combined Motion & Incorporated Legal Authority".into(),
                    "User-supplied conferral paragraph if the governing rule requires one (C.R.C.P. 121 § 1-15(8) vs C.A.R. — needs_verification)".into(),
                    "User-supplied unsworn declaration with signing date and place if used (C.R.S. § 13-27-106 — needs_verification)".into(),
                    "Proposed Order".into(),
                    "Certificate of Service".into(),
                ],
            }
        }
        "SCOTUS" => VenueProfile {
            venue_code: "SCOTUS".into(),
            display_name: "Supreme Court of the United States".into(),
            sworn_statement_title: "Declaration Under 28 U.S.C. § 1746".into(),
            sworn_statement_citation: "28 U.S.C. § 1746 / S. Ct. R. 33 & 39".into(),
            mandatory_oath_text: FED_28_USC_1746_OATH.into(),
            requires_conferral_certificate: false,
            conferral_rule: "S. Ct. R. 21".into(),
            appeal_days: 90,
            reconsideration_days: 25,
            reconsideration_trigger: "entry".into(),
            tolling_mode: "reset_full".into(),
            mail_service_adder_days: 0,
            packet_manifest: vec![
                "Motion for Leave to Proceed In Forma Pauperis & Affidavit (S. Ct. R. 39, if applicable)".into(),
                "Questions Presented (Page i - Mandatory First Page)".into(),
                "List of Parties & Corporate Disclosure / Table of Contents & Authorities".into(),
                "Petition for a Writ of Certiorari / Extraordinary Writ".into(),
                "Appendix (Lower Court Orders & Judgments)".into(),
                "Proof of Service (S. Ct. R. 29.5)".into(),
            ],
        },
        "FEDERAL" => VenueProfile {
            venue_code: "FEDERAL".into(),
            display_name: "United States District / Appellate Court (FRCP / FRAP)".into(),
            sworn_statement_title: "Unsworn Declaration Under Penalty of Perjury".into(),
            sworn_statement_citation: "28 U.S.C. § 1746".into(),
            mandatory_oath_text: FED_28_USC_1746_OATH.into(),
            requires_conferral_certificate: false,
            conferral_rule: "FRCP 26 / Local Rule 7.1".into(),
            appeal_days: 30,
            reconsideration_days: 28,
            reconsideration_trigger: "entry".into(),
            tolling_mode: "reset_full".into(),
            mail_service_adder_days: 3,
            packet_manifest: vec![
                "Notice of Motion / Motion".into(),
                "Memorandum of Law in Support".into(),
                "Declaration Under 28 U.S.C. § 1746 & Exhibits".into(),
                "Proposed Order".into(),
                "Certificate of Service".into(),
            ],
        },
        _ => VenueProfile {
            venue_code: "GENERIC".into(),
            display_name: "General U.S. Court Practice".into(),
            sworn_statement_title: "Declaration Under Penalty of Perjury".into(),
            sworn_statement_citation: "28 U.S.C. § 1746".into(),
            mandatory_oath_text: FED_28_USC_1746_OATH.into(),
            requires_conferral_certificate: false,
            conferral_rule: String::new(),
            appeal_days: 30,
            reconsideration_days: 14,
            reconsideration_trigger: "entry".into(),
            tolling_mode: "reset_full".into(),
            mail_service_adder_days: 3,
            packet_manifest: vec![
                "Notice of Motion".into(),
                "Motion & Supporting Memorandum".into(),
                "Sworn Declaration / Certification".into(),
                "Proposed Order".into(),
                "Certificate of Service".into(),
            ],
        },
    }
}

/// Validate venue-specific and tier-specific competence rules.
pub fn check_venue_competence(profile: &CaseProfile) -> VenueCompetenceReport {
    let venue = resolve_venue(profile);
    let tier = profile.effective_tier().to_string();
    let kind = profile.document_kind.trim().to_ascii_lowercase();
    let mut issues = Vec::new();
    let mut recommendations = Vec::new();

    let combined_text = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        profile.motion.user_text,
        profile.complaint.user_text,
        profile.brief.user_text,
        profile.mandamus.user_text,
        profile.notice_of_appeal.user_text,
        profile.court_markdown
    );
    let lower_text = combined_text.to_ascii_lowercase();

    if tier == "transactional" {
        match kind.as_str() {
            "will" => {
                if !lower_text.contains("witness") && !lower_text.contains("affidavit") {
                    issues.push(ValidationIssue {
                        field: "transactional.will_attestation".into(),
                        message: "Will draft is missing two-witness attestation / self-proving affidavit language (NJ N.J.S.A. 3B:3-4 / CO C.R.S. § 15-11-504).".into(),
                    });
                }
                if !lower_text.contains("executor") && !lower_text.contains("personal representative") {
                    issues.push(ValidationIssue {
                        field: "transactional.will_executor".into(),
                        message: "Will draft does not name an Executor or Personal Representative.".into(),
                    });
                }
                recommendations.push("Execute the Will in the physical presence of two disinterested adult witnesses and a Notary Public for the Self-Proving Affidavit.".into());
            }
            "lease" => {
                if !lower_text.contains("deposit") && !lower_text.contains("rent") {
                    issues.push(ValidationIssue {
                        field: "transactional.lease_terms".into(),
                        message: "Residential Lease should state monthly rent and security deposit terms.".into(),
                    });
                }
                recommendations.push(format!(
                    "Verify {} statutory security deposit caps and mandatory habitability disclosures.",
                    venue.venue_code
                ));
            }
            "contractor" => {
                if !lower_text.contains("intellectual property") && !lower_text.contains("ownership") && !lower_text.contains("work product") {
                    issues.push(ValidationIssue {
                        field: "transactional.contractor_ip".into(),
                        message: "Contractor Agreement should specify Intellectual Property / Work Product ownership and payment milestones.".into(),
                    });
                }
            }
            _ => {}
        }
    } else if tier == "administrative" {
        if !lower_text.contains("day") && !lower_text.contains("date") && !lower_text.contains("deadline") {
            issues.push(ValidationIssue {
                field: "administrative.cure_clock".into(),
                message: "Pre-suit demand or notice should state a specific cure deadline or statutory response window.".into(),
            });
        }
        recommendations.push("Send via Certified Mail, Return Receipt Requested (and save the tracking receipt into evidence/) so the cure clock can be proven if this matter graduates to a Tier 3 lawsuit.".into());
    } else {
        // Tier 3 Litigation Venue Competence Checks
        if venue.venue_code == "NJ" {
            if lower_text.contains("to the best of my knowledge")
                || lower_text.contains("information and belief")
            {
                issues.push(ValidationIssue {
                    field: "venue.nj_r_1_4_4b".into(),
                    message: "NJ Court Rule 1:4-4(b) & 1:6-6 warning: Certifications containing 'to the best of my knowledge' or 'information and belief' are defective in NJ. Facts must be stated on personal knowledge with the verbatim R. 1:4-4(b) punishment clause.".into(),
                });
            }
            recommendations.push(format!(
                "New Jersey requires a Certification in Lieu of Oath (R. 1:4-4(b)): \"{}\"",
                NJ_R_1_4_4B_OATH
            ));
        }

        if venue.venue_code == "CO" {
            let is_motion = kind.is_empty()
                || kind == "motion"
                || kind == "reconsideration"
                || kind == "enlargement"
                || kind == "stay";
            let is_appellate = already_in_reviewing_court(profile)
                || profile.court.name.to_ascii_lowercase().contains("court of appeals");
            if is_motion
                && !is_appellate
                && profile.conferral_status.trim().is_empty()
                && !lower_text.contains("confer")
                && !lower_text.contains("121")
            {
                issues.push(ValidationIssue {
                    field: "venue.co_rule_121_conferral".into(),
                    message: "C.R.C.P. 121 § 1-15(8) conferral is trial-court civil-motion practice (needs_verification). Confirm the rule that governs this paper, then add a conferral paragraph or set conferral_status. Do not treat the C.R.C.P. 121 label as verified.".into(),
                });
            }
            if is_appellate {
                recommendations.push(
                    "Colorado Court of Appeals papers may be governed by C.A.R. rather than C.R.C.P. 121 § 1-15(8). A UUDA declaration (C.R.S. § 13-27-106) often needs a signing date and place. Both labels are needs_verification — edit the user's own text; the engine will not assert them.".into(),
                );
            } else {
                recommendations.push(
                    "Colorado trial-court civil motions often combine the motion and legal authority and open with a conferral statement. Confirm whether C.R.C.P. 121 § 1-15(8) applies (needs_verification).".into(),
                );
            }
        }

        if venue.venue_code == "SCOTUS" {
            if !lower_text.contains("question presented") && !lower_text.contains("questions presented") {
                issues.push(ValidationIssue {
                    field: "venue.scotus_rule_14".into(),
                    message: "U.S. Supreme Court Rule 14.1(a) requires 'Questions Presented' on the very first page of a Petition before the Table of Contents.".into(),
                });
            }
            recommendations.push("If filing In Forma Pauperis under S. Ct. Rule 39, include the Motion for Leave to Proceed IFP and Form Affidavit on 8.5 x 11 inch paper.".into());
        }
    }

    VenueCompetenceReport {
        ok: issues.is_empty(),
        never_files: true,
        never_serves: true,
        matter_tier: tier,
        document_kind: profile.document_kind.clone(),
        venue,
        issues,
        recommendations,
    }
}
