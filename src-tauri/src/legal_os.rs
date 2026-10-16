//! Legal Operating System core:
//! 1. 3-Tier Matter Architecture (Transactional, Administrative/Pre-Suit, Litigation)
//! 2. Multi-Venue Competence Matrix (NJ, CO, Federal, SCOTUS, Generic)
//! 3. Predictive Procedural State Graph (Event -> Branches with Pause/Toll vs Reset & Mail Offsets)

use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::case_profile::{CaseProfile, ValidationIssue};
use crate::check::NEVER_FILE_NOTICE;
use crate::deadlines::{compute_deadline, parse_iso_date};
use crate::rule_pack::{DeadlineRule, RulePack};

pub const NJ_R_1_4_4B_OATH: &str = "I certify that the foregoing statements made by me are true. I am aware that if any of the foregoing statements made by me are willfully false, I am subject to punishment.";

pub const CO_CRS_13_27_106_OATH: &str = "I declare under penalty of perjury under the law of Colorado that the foregoing is true and correct.";

pub const FED_28_USC_1746_OATH: &str = "I declare under penalty of perjury under the laws of the United States of America that the foregoing is true and correct.";

pub const CO_RULE_121_CONFERRAL_TEMPLATE: &str = "Certificate of Conferral (C.R.C.P. 121 § 1-15(8)): Undersigned certifies that they have conferred (or attempted to confer) in good faith with opposing counsel/party regarding the relief requested in this motion.";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VenueProfile {
    pub venue_code: String,
    pub display_name: String,
    pub sworn_statement_title: String,
    pub sworn_statement_citation: String,
    pub mandatory_oath_text: String,
    pub requires_conferral_certificate: bool,
    pub conferral_rule: String,
    pub appeal_days: i64,
    pub reconsideration_days: i64,
    pub reconsideration_trigger: String,
    pub tolling_mode: String, // "pause_remaining" (NJ) vs "reset_full" (CO/Federal)
    pub mail_service_adder_days: i64,
    pub packet_manifest: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VenueCompetenceReport {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub matter_tier: String,
    pub document_kind: String,
    pub venue: VenueProfile,
    pub issues: Vec<ValidationIssue>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProceduralBranch {
    pub id: String,
    pub title: String,
    pub rule_citation: String,
    pub trigger_event_title: String,
    pub trigger_date: String,
    pub trigger_kind: String, // "entry" | "service" | "delivery"
    pub base_days: i64,
    pub mail_offset_days: i64,
    pub clock_interaction: String, // "independent" | "pause_remaining" | "reset_full"
    pub tolled_by_event: Option<String>,
    pub days_elapsed_before_toll: Option<i64>,
    pub days_remaining_after_toll: Option<i64>,
    pub computed_deadline: Option<String>,
    pub days_until_due: Option<i64>,
    pub status: String, // "active" | "paused_tolled" | "expired" | "action_ready"
    pub template_kind: String,
    pub escalation_target: Option<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NextStepsReport {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub notice: String,
    pub case_id: String,
    pub case_title: String,
    pub matter_tier: String,
    pub venue_code: String,
    pub detected_triggers: Vec<String>,
    pub branches: Vec<ProceduralBranch>,
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
            let is_appellate = court_raw.contains("appellate") || pack_raw.contains("app-div");
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
        "CO" => VenueProfile {
            venue_code: "CO".into(),
            display_name: "Colorado State Courts (C.R.C.P. / C.A.R.)".into(),
            sworn_statement_title: "Unsworn Declaration Under Penalty of Perjury".into(),
            sworn_statement_citation: "C.R.S. § 13-27-106".into(),
            mandatory_oath_text: CO_CRS_13_27_106_OATH.into(),
            requires_conferral_certificate: true,
            conferral_rule: "C.R.C.P. 121 § 1-15(8)".into(),
            appeal_days: 49,
            reconsideration_days: 14,
            reconsideration_trigger: "entry".into(),
            tolling_mode: "reset_full".into(),
            mail_service_adder_days: 3,
            packet_manifest: vec![
                "Combined Motion & Incorporated Legal Authority (with C.R.C.P. 121 § 1-15(8) Conferral)".into(),
                "Unsworn Declaration & Exhibit Index (C.R.S. § 13-27-106)".into(),
                "Proposed Order".into(),
                "Certificate of Service".into(),
            ],
        },
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
            if is_motion
                && profile.conferral_status.trim().is_empty()
                && !lower_text.contains("confer")
                && !lower_text.contains("121")
            {
                issues.push(ValidationIssue {
                    field: "venue.co_rule_121_conferral".into(),
                    message: "Colorado C.R.C.P. 121 § 1-15(8) requires a Certificate of Conferral at the beginning of every civil motion stating whether the movant conferred in good faith with opposing parties (or why conferral is exempt). Set conferral_status or add a Rule 121 conferral paragraph.".into(),
                });
            }
            recommendations.push(
                "Colorado motions combine the motion and legal authority into a single document and open with a C.R.C.P. 121 § 1-15(8) conferral statement.".into(),
            );
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

/// Compute holiday-adjusted date string and days remaining from today.
fn compute_adjusted_date(
    pack_id: &str,
    start: NaiveDate,
    total_days: i64,
) -> (String, i64) {
    let pack = RulePack::resolve(pack_id).unwrap_or_else(|_| RulePack::generic());
    let temp_rule = DeadlineRule {
        id: "branch-calc".into(),
        label: "Branch deadline".into(),
        rule: String::new(),
        trigger: "entry".into(),
        days: total_days,
        count: "calendar".into(),
        weekend_holiday_roll: "next_business".into(),
        enlargeable_under: None,
        tolls_appeal_under: None,
        notes: String::new(),
    };
    let res = compute_deadline(&pack, &temp_rule, start);
    let today = Utc::now().date_naive();
    let due_date = parse_iso_date(&res.due).unwrap_or(start + Duration::days(total_days));
    let remaining = (due_date - today).num_days();
    (res.due, remaining)
}

/// Predict all downstream legal branches and clocks from the case's timeline and tier.
pub fn predict_next_steps(profile: &CaseProfile, case_dir: Option<&Path>) -> NextStepsReport {
    let venue = resolve_venue(profile);
    let tier = profile.effective_tier().to_string();
    let mail_offset = if profile.service_method.trim().eq_ignore_ascii_case("mail") {
        venue.mail_service_adder_days
    } else {
        0
    };

    let mut detected_triggers = Vec::new();
    let mut branches = Vec::new();
    let today_iso = Utc::now().date_naive().format("%Y-%m-%d").to_string();

    // Detect if a tolling motion (e.g. reconsideration) was filed on the timeline
    let tolling_event = profile.timeline.iter().find(|ev| {
        let t = ev.title.to_ascii_lowercase();
        (t.contains("reconsideration") || t.contains("alter or amend") || t.contains("rule 59") || t.contains("4:49-2"))
            && (t.contains("filed") || t.contains("served") || t.contains("submitted"))
    });

    if tier == "transactional" {
        detected_triggers.push(format!("Tier 1 Transactional Instrument ({})", profile.document_kind));
        branches.push(ProceduralBranch {
            id: "exec-ceremony".into(),
            title: "Complete Statutory Execution & Witness / Notary Block".into(),
            rule_citation: if profile.document_kind == "will" {
                "N.J.S.A. 3B:3-4 / C.R.S. § 15-11-504".into()
            } else {
                "State Contract & Electronic Signature Act".into()
            },
            trigger_event_title: "Draft Initialized".into(),
            trigger_date: today_iso.clone(),
            trigger_kind: "execution".into(),
            base_days: 0,
            mail_offset_days: 0,
            clock_interaction: "independent".into(),
            tolled_by_event: None,
            days_elapsed_before_toll: None,
            days_remaining_after_toll: None,
            computed_deadline: None,
            days_until_due: None,
            status: "action_ready".into(),
            template_kind: profile.document_kind.clone(),
            escalation_target: Some("administrative".into()),
            explanation: "Verify all party names and clauses, export to PDF, and complete signatures (or two-witness + notary self-proving affidavit for Wills). If a counterparty breaches this instrument later, you can 1-click escalate to a Tier 2 Statutory Cure Demand.".into(),
        });
    } else if tier == "administrative" {
        let trigger_date_str = profile
            .timeline
            .last()
            .and_then(|e| if !e.date.trim().is_empty() { Some(e.date.clone()) } else { None })
            .unwrap_or_else(|| today_iso.clone());
        let base_days = match profile.document_kind.as_str() {
            "cure_notice" => 7,
            "records_request" => 7,
            "security_deposit_demand" => 14,
            _ => 30,
        };
        let start = parse_iso_date(&trigger_date_str).unwrap_or_else(|_| Utc::now().date_naive());
        let (due_iso, days_left) = compute_adjusted_date(&profile.rule_pack_id, start, base_days + mail_offset);

        detected_triggers.push(format!("Tier 2 Statutory Notice / Demand ({trigger_date_str})"));
        branches.push(ProceduralBranch {
            id: "cure-window".into(),
            title: "Statutory Cure / Response Window Expires".into(),
            rule_citation: "Statutory Pre-Suit Demand Clock".into(),
            trigger_event_title: "Demand / Notice Sent".into(),
            trigger_date: trigger_date_str.clone(),
            trigger_kind: "delivery".into(),
            base_days,
            mail_offset_days: mail_offset,
            clock_interaction: "independent".into(),
            tolled_by_event: None,
            days_elapsed_before_toll: None,
            days_remaining_after_toll: None,
            computed_deadline: Some(due_iso),
            days_until_due: Some(days_left),
            status: if days_left < 0 { "action_ready".into() } else { "active".into() },
            template_kind: "complaint".into(),
            escalation_target: Some("litigation".into()),
            explanation: "Once the statutory cure or response window expires without compliance, 1-click graduate this folder into a Tier 3 Court Complaint with your demand letter and Certified Mail receipt automatically indexed as Exhibit A.".into(),
        });
    } else {
        // Tier 3 Litigation Procedural Graph
        // Find trigger date from timeline, notice_of_appeal.judgment_date, or orders/
        let mut order_date_str = profile.notice_of_appeal.judgment_date.trim().to_string();
        let mut order_event_title = "Order / Judgment Entered".to_string();

        for ev in &profile.timeline {
            let t = ev.title.to_ascii_lowercase();
            if t.contains("order") || t.contains("judgment") || t.contains("denied") || t.contains("dismiss") {
                if !ev.date.trim().is_empty() {
                    order_date_str = ev.date.trim().to_string();
                    order_event_title = ev.title.clone();
                }
            }
        }

        if order_date_str.is_empty() {
            order_date_str = today_iso.clone();
            order_event_title = "Adverse Order / Trigger Date (Using Today as Preview)".to_string();
        }
        detected_triggers.push(format!("{order_event_title} ({order_date_str})"));

        let order_date = parse_iso_date(&order_date_str).unwrap_or_else(|_| Utc::now().date_naive());

        // Branch A: Motion for Reconsideration / Alter or Amend
        let recon_offset = if venue.reconsideration_trigger == "service" { mail_offset } else { 0 };
        let (recon_due, recon_days_left) = compute_adjusted_date(
            &profile.rule_pack_id,
            order_date,
            venue.reconsideration_days + recon_offset,
        );
        branches.push(ProceduralBranch {
            id: "branch-reconsideration".into(),
            title: format!("Branch A: Motion for Reconsideration ({})", venue.venue_code),
            rule_citation: match venue.venue_code.as_str() {
                "NJ" => if venue.reconsideration_days == 10 { "N.J. Ct. R. 2:11-6".into() } else { "N.J. Ct. R. 4:49-2".into() },
                "CO" => "C.R.C.P. 59 / C.A.R. 40".into(),
                "SCOTUS" => "S. Ct. R. 44".into(),
                _ => "Fed. R. Civ. P. 59(e)".into(),
            },
            trigger_event_title: order_event_title.clone(),
            trigger_date: order_date_str.clone(),
            trigger_kind: venue.reconsideration_trigger.clone(),
            base_days: venue.reconsideration_days,
            mail_offset_days: recon_offset,
            clock_interaction: venue.tolling_mode.clone(),
            tolled_by_event: None,
            days_elapsed_before_toll: None,
            days_remaining_after_toll: None,
            computed_deadline: Some(recon_due),
            days_until_due: Some(recon_days_left),
            status: if recon_days_left < 0 { "expired".into() } else { "active".into() },
            template_kind: "reconsideration".into(),
            escalation_target: None,
            explanation: format!(
                "Ask the same court to reconsider or amend its order within {} days of {} (plus {} mail offset days if applicable). A timely motion tolls/affects the appeal clock ({}).",
                venue.reconsideration_days, venue.reconsideration_trigger, recon_offset, venue.tolling_mode
            ),
        });

        // Branch B: Notice of Appeal / Higher Court Review (with Pause vs Reset Tolling Math!)
        let mut tolled_by = None;
        let mut elapsed_before_toll = None;
        let mut remaining_after_toll = None;
        let mut appeal_status = "active".to_string();
        let (appeal_due, appeal_days_left) = compute_adjusted_date(
            &profile.rule_pack_id,
            order_date,
            venue.appeal_days,
        );

        if let Some(tev) = tolling_event {
            if let Ok(t_date) = parse_iso_date(&tev.date) {
                let elapsed = std::cmp::max(0, (t_date - order_date).num_days());
                if elapsed <= venue.reconsideration_days + recon_offset {
                    tolled_by = Some(format!("{} ({})", tev.title, tev.date));
                    elapsed_before_toll = Some(elapsed);
                    if venue.tolling_mode == "pause_remaining" {
                        remaining_after_toll = Some(std::cmp::max(0, venue.appeal_days - elapsed));
                        appeal_status = "paused_tolled".into();
                    } else {
                        remaining_after_toll = Some(venue.appeal_days);
                        appeal_status = "paused_tolled".into();
                    }
                }
            }
        } else if appeal_days_left < 0 {
            appeal_status = "expired".into();
        }

        branches.push(ProceduralBranch {
            id: "branch-appeal".into(),
            title: if venue.venue_code == "SCOTUS" {
                "Branch B: Petition for a Writ of Certiorari (SCOTUS)".into()
            } else {
                format!("Branch B: Notice of Appeal ({})", venue.venue_code)
            },
            rule_citation: match venue.venue_code.as_str() {
                "NJ" => "N.J. Ct. R. 2:4-1 & R. 2:4-3(e)".into(),
                "CO" => "C.A.R. 4(a)".into(),
                "SCOTUS" => "S. Ct. R. 13.1".into(),
                _ => "Fed. R. App. P. 4(a)(1)".into(),
            },
            trigger_event_title: order_event_title.clone(),
            trigger_date: order_date_str.clone(),
            trigger_kind: "entry".into(),
            base_days: venue.appeal_days,
            mail_offset_days: 0,
            clock_interaction: venue.tolling_mode.clone(),
            tolled_by_event: tolled_by.clone(),
            days_elapsed_before_toll: elapsed_before_toll,
            days_remaining_after_toll: remaining_after_toll,
            computed_deadline: Some(appeal_due),
            days_until_due: Some(appeal_days_left),
            status: appeal_status,
            template_kind: "notice_of_appeal".into(),
            escalation_target: Some(if venue.venue_code == "SCOTUS" { "scotus".into() } else { "appellate".into() }),
            explanation: if let Some(ref tb) = tolled_by {
                if venue.tolling_mode == "pause_remaining" {
                    format!(
                        "CLOCK PAUSED (NJ R. 2:4-3(e)): Tolling motion {} filed after {} days. {} days remain on the 45-day appeal clock once the motion is decided.",
                        tb,
                        elapsed_before_toll.unwrap_or(0),
                        remaining_after_toll.unwrap_or(0)
                    )
                } else {
                    format!(
                        "CLOCK RESET PENDING: Timely post-judgment motion {} tolls appeal; full {}-day appeal period restarts upon entry of the order deciding the motion.",
                        tb, venue.appeal_days
                    )
                }
            } else {
                format!(
                    "Escalate to appellate review within {} days of judgment entry. Use 1-click Escalate to bundle trial orders and filings into the Appellate Appendix.",
                    venue.appeal_days
                )
            },
        });

        // Branch C: Motion for Stay Pending Appeal / Extraordinary Mandamus
        branches.push(ProceduralBranch {
            id: "branch-stay-or-mandamus".into(),
            title: "Branch C: Motion for Stay Pending Appeal / Extraordinary Writ".into(),
            rule_citation: match venue.venue_code.as_str() {
                "NJ" => "N.J. Ct. R. 2:9-5".into(),
                "CO" => "C.A.R. 8 / C.A.R. 21 (Mandamus)".into(),
                "SCOTUS" => "S. Ct. R. 23 (Stay)".into(),
                _ => "Fed. R. App. P. 8(a)".into(),
            },
            trigger_event_title: order_event_title,
            trigger_date: order_date_str,
            trigger_kind: "entry".into(),
            base_days: 0,
            mail_offset_days: 0,
            clock_interaction: "independent".into(),
            tolled_by_event: None,
            days_elapsed_before_toll: None,
            days_remaining_after_toll: None,
            computed_deadline: None,
            days_until_due: None,
            status: "action_ready".into(),
            template_kind: "stay".into(),
            escalation_target: Some("scotus".into()),
            explanation: "Pause enforcement of the order while reconsideration or appellate review is pending, or seek an extraordinary Rule to Show Cause / Mandamus if a ministerial duty (such as docketing with an approved fee waiver) was refused.".into(),
        });
    }

    // Check if there are PDFs in orders/ that haven't been logged on the timeline yet
    if let Some(dir) = case_dir {
        if let Ok(entries) = std::fs::read_dir(dir.join("orders")) {
            let count = entries.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).count();
            if count > 0 {
                detected_triggers.push(format!("{count} order file(s) present in orders/ directory"));
            }
        }
    }

    NextStepsReport {
        ok: true,
        never_files: true,
        never_serves: true,
        notice: NEVER_FILE_NOTICE.into(),
        case_id: profile.id.clone(),
        case_title: profile.title.clone(),
        matter_tier: tier,
        venue_code: venue.venue_code,
        detected_triggers,
        branches,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_profile::{sample_appeal_profile, TimelineEvent};

    #[test]
    fn nj_venue_rejects_best_of_knowledge_hedging() {
        let mut p = sample_appeal_profile();
        p.rule_pack_id = "nj-app-div".into();
        p.document_kind = "reconsideration".into();
        p.motion.user_text = "I certify these facts are true to the best of my knowledge.".into();
        let report = check_venue_competence(&p);
        assert!(!report.ok);
        assert!(report.issues.iter().any(|i| i.field == "venue.nj_r_1_4_4b"));
    }

    #[test]
    fn co_venue_enforces_rule_121_conferral() {
        let mut p = sample_appeal_profile();
        p.governing_state = "CO".into();
        p.document_kind = "motion".into();
        p.motion.user_text = "Movant requests an extension of 14 days.".into();
        let report = check_venue_competence(&p);
        assert!(!report.ok);
        assert!(report.issues.iter().any(|i| i.field == "venue.co_rule_121_conferral"));

        p.conferral_status = "Conferred on Oct 12; opposing party does not oppose.".into();
        let report_ok = check_venue_competence(&p);
        assert!(report_ok.ok);
    }

    #[test]
    fn nj_tolling_pauses_remaining_appeal_days() {
        let mut p = sample_appeal_profile();
        p.rule_pack_id = "nj-app-div".into();
        p.timeline.push(TimelineEvent {
            id: "1".into(),
            title: "Trial Order Denied".into(),
            date: "2026-10-01".into(),
            notes: String::new(),
        });
        p.timeline.push(TimelineEvent {
            id: "2".into(),
            title: "Motion for Reconsideration Filed".into(),
            date: "2026-10-08".into(),
            notes: String::new(),
        });
        let next = predict_next_steps(&p, None);
        let appeal = next.branches.iter().find(|b| b.id == "branch-appeal").unwrap();
        assert_eq!(appeal.status, "paused_tolled");
        assert_eq!(appeal.days_elapsed_before_toll, Some(7));
        assert_eq!(appeal.days_remaining_after_toll, Some(38)); // 45 - 7 = 38 days remaining!
    }
}
