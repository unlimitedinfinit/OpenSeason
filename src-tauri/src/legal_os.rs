//! Legal Operating System core:
//! 1. 3-Tier Matter Architecture (Transactional, Administrative/Pre-Suit, Litigation)
//! 2. Multi-Venue Competence Matrix (NJ, CO, Federal, SCOTUS, Generic)
//! 3. Predictive Procedural State Graph (Event -> Branches with Pause/Toll vs Reset & Mail Offsets)

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::case_profile::{CaseProfile, ValidationIssue};
use crate::check::NEVER_FILE_NOTICE;
use crate::deadlines::{compute_deadline, parse_iso_date};
use crate::rule_pack::{DeadlineRule, RulePack};

pub const NJ_R_1_4_4B_OATH: &str = "I certify that the foregoing statements made by me are true. I am aware that if any of the foregoing statements made by me are willfully false, I am subject to punishment.";

pub const CO_CRS_13_27_106_OATH: &str = "I declare under penalty of perjury under the law of Colorado that the foregoing is true and correct.";

pub const FED_28_USC_1746_OATH: &str = "I declare under penalty of perjury under the laws of the United States of America that the foregoing is true and correct.";

/// Suggested conferral sentence. citation_status is needs_verification.
/// Appellate papers in Colorado may be governed by C.A.R. rather than C.R.C.P. 121.
pub const CO_RULE_121_CONFERRAL_TEMPLATE: &str = "I conferred with the opposing party on [date]. The opposing party [agrees / takes no position / opposes]. Confirm whether C.R.C.P. 121 § 1-15(8) or the Colorado Appellate Rules govern this paper before filing.";

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
    pub status: String, // "active" | "paused_tolled" | "expired" | "action_ready" | "needs_trigger_date"
    pub template_kind: String,
    pub escalation_target: Option<String>,
    pub explanation: String,
    /// How the due date was calculated. Empty when no user trigger date exists.
    #[serde(default)]
    pub math: String,
    /// Provenance of the trigger date (timeline event, judgment_date field, etc.).
    #[serde(default)]
    pub trigger_source: String,
    /// Displayed deadlines are always unverified until the user checks the order and rule.
    #[serde(default)]
    pub unverified: bool,
}

pub const ENTER_ORDER_DATE_PROMPT: &str = "Enter the order date to compute this deadline";
pub const DEADLINE_UNVERIFIED_LABEL: &str = "unverified, check the order and rule";

fn local_today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

#[derive(Debug, Clone)]
struct UserTrigger {
    date: NaiveDate,
    date_iso: String,
    title: String,
    source: String,
}

/// A deadline is only real when the user typed an order/trigger date with a source.
fn user_order_trigger(profile: &CaseProfile) -> Option<UserTrigger> {
    let mut from_timeline: Option<UserTrigger> = None;
    for ev in &profile.timeline {
        let raw = ev.date.trim();
        if raw.is_empty() {
            continue;
        }
        let Ok(parsed) = parse_iso_date(raw) else {
            continue;
        };
        let t = ev.title.to_ascii_lowercase();
        let looks_like_order = t.contains("order")
            || t.contains("judgment")
            || t.contains("judgement")
            || t.contains("denied")
            || t.contains("dismiss")
            || t.contains("entered")
            || t.contains("decision")
            || t.contains("opinion");
        if !looks_like_order {
            continue;
        }
        from_timeline = Some(UserTrigger {
            date: parsed,
            date_iso: parsed.to_string(),
            title: if ev.title.trim().is_empty() {
                "Order / judgment date entered by user".into()
            } else {
                ev.title.clone()
            },
            source: format!("timeline:{}", ev.title),
        });
    }
    if from_timeline.is_some() {
        return from_timeline;
    }
    let jd = profile.notice_of_appeal.judgment_date.trim();
    if jd.is_empty() {
        return None;
    }
    let Ok(parsed) = parse_iso_date(jd) else {
        return None;
    };
    Some(UserTrigger {
        date: parsed,
        date_iso: parsed.to_string(),
        title: if profile.notice_of_appeal.judgment_description.trim().is_empty() {
            "Judgment / order date from the case profile".into()
        } else {
            profile.notice_of_appeal.judgment_description.clone()
        },
        source: "notice_of_appeal.judgment_date".into(),
    })
}

fn administrative_trigger(profile: &CaseProfile) -> Option<UserTrigger> {
    for ev in profile.timeline.iter().rev() {
        let raw = ev.date.trim();
        if raw.is_empty() {
            continue;
        }
        let Ok(parsed) = parse_iso_date(raw) else {
            continue;
        };
        return Some(UserTrigger {
            date: parsed,
            date_iso: parsed.to_string(),
            title: if ev.title.trim().is_empty() {
                "Notice / demand date entered by user".into()
            } else {
                ev.title.clone()
            },
            source: format!("timeline:{}", ev.title),
        });
    }
    None
}

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
            // The pack id is teaching data. Court name and stage decide whether a 10-day
            // appellate reconsideration clock applies — never the pack id alone.
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
    let today = local_today();
    let due_date = parse_iso_date(&res.due).unwrap_or(start + Duration::days(total_days));
    let remaining = (due_date - today).num_days();
    (res.due, remaining)
}

fn deadline_math(rule: &str, trigger: &UserTrigger, days: i64, due: &str, mail_offset: i64) -> String {
    format!(
        "Trigger date {date} ({title}; source: {source}) + {days} days{mail} under {rule} = due {due}. {unverified}.",
        date = trigger.date_iso,
        title = trigger.title,
        source = trigger.source,
        days = days,
        mail = if mail_offset > 0 {
            format!(" + {mail_offset} mail-service days")
        } else {
            String::new()
        },
        rule = rule,
        due = due,
        unverified = DEADLINE_UNVERIFIED_LABEL
    )
}

/// Predict all downstream legal branches and clocks from the case's timeline and tier.
/// Never invents a deadline from "today". A due date is returned only when the user
/// entered an order/trigger date with a source.
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

    let tolling_event = profile.timeline.iter().find(|ev| {
        let t = ev.title.to_ascii_lowercase();
        (t.contains("reconsideration")
            || t.contains("alter or amend")
            || t.contains("rule 59")
            || t.contains("4:49-2"))
            && (t.contains("filed") || t.contains("served") || t.contains("submitted"))
    });

    if tier == "transactional" {
        detected_triggers.push(format!(
            "Tier 1 Transactional Instrument ({})",
            profile.document_kind
        ));
        branches.push(ProceduralBranch {
            id: "exec-ceremony".into(),
            title: "Complete execution, witnesses, and notary block".into(),
            rule_citation: if profile.document_kind == "will" {
                "N.J.S.A. 3B:3-4 / C.R.S. § 15-11-504".into()
            } else {
                "State contract and electronic signature act".into()
            },
            trigger_event_title: "Draft initialized".into(),
            trigger_date: String::new(),
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
            explanation: "Verify every name and clause, export to PDF, and complete signatures (or two-witness plus notary self-proving affidavit for a will). This is not a court deadline.".into(),
            math: String::new(),
            trigger_source: String::new(),
            unverified: false,
        });
    } else if tier == "administrative" {
        let trigger = administrative_trigger(profile);
        let base_days = match profile.document_kind.as_str() {
            "cure_notice" => 7,
            "records_request" => 7,
            "security_deposit_demand" => 14,
            _ => 30,
        };
        let rule = "Statutory pre-suit demand clock (pack data; confirm the statute)";
        if let Some(tr) = trigger {
            detected_triggers.push(format!("{} ({}) [{}]", tr.title, tr.date_iso, tr.source));
            let (due_iso, days_left) =
                compute_adjusted_date(&profile.rule_pack_id, tr.date, base_days + mail_offset);
            let math = deadline_math(rule, &tr, base_days, &due_iso, mail_offset);
            branches.push(ProceduralBranch {
                id: "cure-window".into(),
                title: "Statutory cure / response window".into(),
                rule_citation: rule.into(),
                trigger_event_title: tr.title.clone(),
                trigger_date: tr.date_iso.clone(),
                trigger_kind: "delivery".into(),
                base_days,
                mail_offset_days: mail_offset,
                clock_interaction: "independent".into(),
                tolled_by_event: None,
                days_elapsed_before_toll: None,
                days_remaining_after_toll: None,
                computed_deadline: Some(due_iso),
                days_until_due: Some(days_left),
                status: if days_left < 0 {
                    "action_ready".into()
                } else {
                    "active".into()
                },
                template_kind: "complaint".into(),
                escalation_target: Some("litigation".into()),
                explanation: format!(
                    "{math} Once the window expires without compliance, you may graduate this folder into a court complaint. Confirm the statute yourself."
                ),
                math,
                trigger_source: tr.source,
                unverified: true,
            });
        } else {
            detected_triggers.push("No user-entered notice/demand date".into());
            branches.push(ProceduralBranch {
                id: "cure-window".into(),
                title: "Statutory cure / response window".into(),
                rule_citation: rule.into(),
                trigger_event_title: String::new(),
                trigger_date: String::new(),
                trigger_kind: "delivery".into(),
                base_days,
                mail_offset_days: mail_offset,
                clock_interaction: "independent".into(),
                tolled_by_event: None,
                days_elapsed_before_toll: None,
                days_remaining_after_toll: None,
                computed_deadline: None,
                days_until_due: None,
                status: "needs_trigger_date".into(),
                template_kind: "complaint".into(),
                escalation_target: Some("litigation".into()),
                explanation: ENTER_ORDER_DATE_PROMPT.into(),
                math: String::new(),
                trigger_source: String::new(),
                unverified: true,
            });
        }
    } else {
        let trigger = user_order_trigger(profile);
        let in_review = already_in_reviewing_court(profile);
        let recon_offset = if venue.reconsideration_trigger == "service" {
            mail_offset
        } else {
            0
        };
        let recon_rule = match venue.venue_code.as_str() {
            "NJ" => {
                if venue.reconsideration_days == 10 {
                    "N.J. Ct. R. 2:11-6"
                } else {
                    "N.J. Ct. R. 4:49-2"
                }
            }
            "CO" => "C.R.C.P. 59 / C.A.R. 40",
            "SCOTUS" => "S. Ct. R. 44",
            _ => "Fed. R. Civ. P. 59(e)",
        };
        let appeal_rule = match venue.venue_code.as_str() {
            "NJ" => "N.J. Ct. R. 2:4-1 & R. 2:4-3(e)",
            "CO" => "C.A.R. 4(a)",
            "SCOTUS" => "S. Ct. R. 13.1",
            _ => "Fed. R. App. P. 4(a)(1)",
        };

        match &trigger {
            Some(tr) => detected_triggers.push(format!("{} ({}) [{}]", tr.title, tr.date_iso, tr.source)),
            None => detected_triggers.push("No user-entered order/trigger date".into()),
        }

        // Branch A: reconsideration — always listed, but due date only with a user trigger.
        let mut recon = ProceduralBranch {
            id: "branch-reconsideration".into(),
            title: format!("Motion for reconsideration ({})", venue.venue_code),
            rule_citation: recon_rule.into(),
            trigger_event_title: trigger.as_ref().map(|t| t.title.clone()).unwrap_or_default(),
            trigger_date: trigger.as_ref().map(|t| t.date_iso.clone()).unwrap_or_default(),
            trigger_kind: venue.reconsideration_trigger.clone(),
            base_days: venue.reconsideration_days,
            mail_offset_days: recon_offset,
            clock_interaction: venue.tolling_mode.clone(),
            tolled_by_event: None,
            days_elapsed_before_toll: None,
            days_remaining_after_toll: None,
            computed_deadline: None,
            days_until_due: None,
            status: "needs_trigger_date".into(),
            template_kind: "reconsideration".into(),
            escalation_target: None,
            explanation: ENTER_ORDER_DATE_PROMPT.into(),
            math: String::new(),
            trigger_source: trigger.as_ref().map(|t| t.source.clone()).unwrap_or_default(),
            unverified: true,
        };
        if let Some(tr) = &trigger {
            let (recon_due, recon_days_left) = compute_adjusted_date(
                &profile.rule_pack_id,
                tr.date,
                venue.reconsideration_days + recon_offset,
            );
            recon.computed_deadline = Some(recon_due.clone());
            recon.days_until_due = Some(recon_days_left);
            recon.status = if recon_days_left < 0 {
                "expired".into()
            } else {
                "active".into()
            };
            recon.math = deadline_math(
                recon_rule,
                tr,
                venue.reconsideration_days,
                &recon_due,
                recon_offset,
            );
            recon.explanation = format!(
                "{} Ask the same court to reconsider within {} days of {} (plus {} mail-offset days if mail service applies). {}",
                recon.math,
                venue.reconsideration_days,
                venue.reconsideration_trigger,
                recon_offset,
                DEADLINE_UNVERIFIED_LABEL
            );
        }
        branches.push(recon);

        // Branch B: notice of appeal — omitted when the case is already in a reviewing court.
        if !in_review {
            let mut appeal = ProceduralBranch {
                id: "branch-appeal".into(),
                title: if venue.venue_code == "SCOTUS" {
                    "Petition for a writ of certiorari (SCOTUS)".into()
                } else {
                    format!("Notice of appeal ({})", venue.venue_code)
                },
                rule_citation: appeal_rule.into(),
                trigger_event_title: trigger.as_ref().map(|t| t.title.clone()).unwrap_or_default(),
                trigger_date: trigger.as_ref().map(|t| t.date_iso.clone()).unwrap_or_default(),
                trigger_kind: "entry".into(),
                base_days: venue.appeal_days,
                mail_offset_days: 0,
                clock_interaction: venue.tolling_mode.clone(),
                tolled_by_event: None,
                days_elapsed_before_toll: None,
                days_remaining_after_toll: None,
                computed_deadline: None,
                days_until_due: None,
                status: "needs_trigger_date".into(),
                template_kind: "notice_of_appeal".into(),
                escalation_target: Some(if venue.venue_code == "SCOTUS" {
                    "scotus".into()
                } else {
                    "appellate".into()
                }),
                explanation: ENTER_ORDER_DATE_PROMPT.into(),
                math: String::new(),
                trigger_source: trigger.as_ref().map(|t| t.source.clone()).unwrap_or_default(),
                unverified: true,
            };
            if let Some(tr) = &trigger {
                let (appeal_due, appeal_days_left) =
                    compute_adjusted_date(&profile.rule_pack_id, tr.date, venue.appeal_days);
                let mut tolled_by = None;
                let mut elapsed_before_toll = None;
                let mut remaining_after_toll = None;
                let mut appeal_status = if appeal_days_left < 0 {
                    "expired".to_string()
                } else {
                    "active".to_string()
                };
                if let Some(tev) = tolling_event {
                    if let Ok(t_date) = parse_iso_date(&tev.date) {
                        let elapsed = std::cmp::max(0, (t_date - tr.date).num_days());
                        if elapsed <= venue.reconsideration_days + recon_offset {
                            tolled_by = Some(format!("{} ({})", tev.title, tev.date));
                            elapsed_before_toll = Some(elapsed);
                            appeal_status = "paused_tolled".into();
                            remaining_after_toll = Some(if venue.tolling_mode == "pause_remaining" {
                                std::cmp::max(0, venue.appeal_days - elapsed)
                            } else {
                                venue.appeal_days
                            });
                        }
                    }
                }
                appeal.computed_deadline = Some(appeal_due.clone());
                appeal.days_until_due = Some(appeal_days_left);
                appeal.status = appeal_status;
                appeal.tolled_by_event = tolled_by.clone();
                appeal.days_elapsed_before_toll = elapsed_before_toll;
                appeal.days_remaining_after_toll = remaining_after_toll;
                appeal.math = deadline_math(appeal_rule, tr, venue.appeal_days, &appeal_due, 0);
                appeal.explanation = if let Some(ref tb) = tolled_by {
                    if venue.tolling_mode == "pause_remaining" {
                        format!(
                            "{} Clock paused (NJ R. 2:4-3(e)): tolling motion {} filed after {} days. {} days remain once the motion is decided. {}",
                            appeal.math,
                            tb,
                            elapsed_before_toll.unwrap_or(0),
                            remaining_after_toll.unwrap_or(0),
                            DEADLINE_UNVERIFIED_LABEL
                        )
                    } else {
                        format!(
                            "{} Clock reset pending: a timely post-judgment motion {} tolls appeal; the full {}-day period restarts when the motion is decided. {}",
                            appeal.math, tb, venue.appeal_days, DEADLINE_UNVERIFIED_LABEL
                        )
                    }
                } else {
                    format!(
                        "{} Escalate to appellate review within {} days of judgment entry. {}",
                        appeal.math, venue.appeal_days, DEADLINE_UNVERIFIED_LABEL
                    )
                };
            }
            branches.push(appeal);
        }

        // Branch C: stay / extraordinary writ — no automatic day count.
        branches.push(ProceduralBranch {
            id: "branch-stay-or-mandamus".into(),
            title: "Motion for stay pending appeal / extraordinary writ".into(),
            rule_citation: match venue.venue_code.as_str() {
                "NJ" => "N.J. Ct. R. 2:9-5".into(),
                "CO" => "C.A.R. 8 / C.A.R. 21 (Mandamus)".into(),
                "SCOTUS" => "S. Ct. R. 23 (Stay)".into(),
                _ => "Fed. R. App. P. 8(a)".into(),
            },
            trigger_event_title: trigger.as_ref().map(|t| t.title.clone()).unwrap_or_default(),
            trigger_date: trigger.as_ref().map(|t| t.date_iso.clone()).unwrap_or_default(),
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
            explanation: "Pause enforcement while reconsideration or appellate review is pending, or seek an extraordinary writ if a ministerial duty was refused. This branch has no automatic day count in the pack.".into(),
            math: String::new(),
            trigger_source: trigger.as_ref().map(|t| t.source.clone()).unwrap_or_default(),
            unverified: false,
        });
    }

    if let Some(dir) = case_dir {
        if let Ok(entries) = std::fs::read_dir(dir.join("orders")) {
            let count = entries.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).count();
            if count > 0 {
                detected_triggers.push(format!(
                    "{count} order file(s) in orders/ — dates from those files are not used until you enter them"
                ));
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
    fn co_trial_court_flags_missing_conferral_as_needs_verification() {
        let mut p = sample_appeal_profile();
        p.governing_state = "CO".into();
        p.document_kind = "motion".into();
        p.court.name = "Denver District Court".into();
        p.court.division = None;
        p.procedural_stage = "active_docket".into();
        p.motion.user_text = "Movant requests an extension of 14 days.".into();
        let report = check_venue_competence(&p);
        assert!(!report.ok);
        let issue = report
            .issues
            .iter()
            .find(|i| i.field == "venue.co_rule_121_conferral")
            .expect("trial-court CO motion still surfaces conferral");
        assert!(issue.message.contains("needs_verification"));

        p.conferral_status = "Conferred on Oct 12; opposing party does not oppose.".into();
        let report_ok = check_venue_competence(&p);
        assert!(report_ok.ok);
    }

    #[test]
    fn co_appellate_does_not_assert_rule_121_as_verified() {
        let mut p = sample_appeal_profile();
        p.governing_state = "CO".into();
        p.document_kind = "motion".into();
        p.court.name = "Colorado Court of Appeals".into();
        p.procedural_stage = "appellate".into();
        p.motion.user_text = "Movant requests an extension of 14 days.".into();
        let report = check_venue_competence(&p);
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.field == "venue.co_rule_121_conferral"),
            "C.R.C.P. 121 is not asserted for Court of Appeals papers"
        );
        assert!(report
            .recommendations
            .iter()
            .any(|r| r.contains("needs_verification") && r.contains("C.A.R.")));
        assert!(report.venue.conferral_rule.contains("needs_verification"));
        assert!(report.venue.sworn_statement_citation.contains("needs_verification"));
    }

    fn trial_court_profile() -> crate::case_profile::CaseProfile {
        let mut p = sample_appeal_profile();
        p.rule_pack_id = "nj-app-div".into();
        p.governing_state = "NJ".into();
        p.court.name = "Superior Court of New Jersey, Law Division".into();
        p.court.division = Some("Sample vicinage".into());
        p.procedural_stage = "post_judgment".into();
        p.notice_of_appeal.judgment_date.clear();
        p.timeline.clear();
        p
    }

    #[test]
    fn nj_tolling_pauses_remaining_appeal_days() {
        let mut p = trial_court_profile();
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
        assert!(appeal.unverified);
        assert!(appeal.math.contains("2026-10-01"));
        assert!(appeal.math.contains("unverified"));
    }

    #[test]
    fn no_trigger_date_does_not_invent_a_deadline() {
        let p = trial_court_profile();
        let next = predict_next_steps(&p, None);
        assert!(
            next.detected_triggers
                .iter()
                .any(|t| t.contains("No user-entered")),
            "{:?}",
            next.detected_triggers
        );
        for b in &next.branches {
            assert!(
                b.computed_deadline.is_none(),
                "{} leaked a deadline {:?}",
                b.id,
                b.computed_deadline
            );
            if b.id == "branch-reconsideration" || b.id == "branch-appeal" {
                assert_eq!(b.status, "needs_trigger_date");
                assert!(b.explanation.contains(ENTER_ORDER_DATE_PROMPT));
            }
        }
        let blob = format!("{next:?}");
        assert!(!blob.contains("Using Today as Preview"));
    }

    #[test]
    fn user_entered_order_date_computes_with_basis() {
        let mut p = trial_court_profile();
        p.timeline.push(TimelineEvent {
            id: "1".into(),
            title: "Order Denying Motion".into(),
            date: "2026-10-01".into(),
            notes: "Dkt. 12".into(),
        });
        let next = predict_next_steps(&p, None);
        let recon = next
            .branches
            .iter()
            .find(|b| b.id == "branch-reconsideration")
            .unwrap();
        assert_eq!(recon.computed_deadline.as_deref(), Some("2026-10-21"));
        assert_eq!(recon.trigger_date, "2026-10-01");
        assert!(recon.trigger_source.contains("timeline"));
        assert!(recon.unverified);
        assert!(recon.math.contains("N.J. Ct. R. 4:49-2") || recon.math.contains("4:49-2"));
        assert!(recon.explanation.contains(DEADLINE_UNVERIFIED_LABEL));
    }

    #[test]
    fn appellate_division_does_not_offer_trial_notice_of_appeal_clock() {
        let mut p = trial_court_profile();
        p.court.name = "Superior Court of New Jersey, Appellate Division".into();
        p.procedural_stage = "appellate".into();
        p.timeline.push(TimelineEvent {
            id: "1".into(),
            title: "Appellate Order Entered".into(),
            date: "2026-10-01".into(),
            notes: String::new(),
        });
        let next = predict_next_steps(&p, None);
        assert!(
            next.branches.iter().all(|b| b.id != "branch-appeal"),
            "appellate cases must not show a 45-day trial notice-of-appeal clock: {:?}",
            next.branches.iter().map(|b| &b.id).collect::<Vec<_>>()
        );
        let recon = next
            .branches
            .iter()
            .find(|b| b.id == "branch-reconsideration")
            .unwrap();
        assert!(recon.computed_deadline.is_some());
        assert_eq!(recon.rule_citation, "N.J. Ct. R. 2:11-6");
    }

    #[test]
    fn uses_local_today_not_utc_for_days_remaining() {
        let mut p = trial_court_profile();
        p.timeline.push(TimelineEvent {
            id: "1".into(),
            title: "Order Entered".into(),
            date: chrono::Local::now().date_naive().to_string(),
            notes: String::new(),
        });
        let next = predict_next_steps(&p, None);
        let recon = next
            .branches
            .iter()
            .find(|b| b.id == "branch-reconsideration")
            .unwrap();
        let days = recon.days_until_due.expect("user trigger should compute");
        // 20 calendar days from today, rolled to a business day, should be in a tight window.
        assert!(
            (18..=24).contains(&days),
            "days_until_due {days} should be ~20 local days, not a UTC-skewed value"
        );
    }
}
