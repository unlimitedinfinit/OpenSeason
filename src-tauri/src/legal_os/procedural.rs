//! Predictive procedural state graph, deadlines, branching, and tolling calculations.

use chrono::{Duration, NaiveDate};
use std::path::Path;

use crate::case_profile::CaseProfile;
use crate::check::NEVER_FILE_NOTICE;
use crate::deadlines::{compute_deadline, parse_iso_date};
use crate::rule_pack::{DeadlineRule, RulePack};
use super::types::{
    NextStepsReport, ProceduralBranch, DEADLINE_UNVERIFIED_LABEL, ENTER_ORDER_DATE_PROMPT,
};
use super::venue::{already_in_reviewing_court, resolve_venue};

pub fn local_today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

#[derive(Debug, Clone)]
pub struct UserTrigger {
    pub date: NaiveDate,
    pub date_iso: String,
    pub title: String,
    pub source: String,
}

/// A deadline is only real when the user typed an order/trigger date with a source.
pub fn user_order_trigger(profile: &CaseProfile) -> Option<UserTrigger> {
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

pub fn administrative_trigger(profile: &CaseProfile) -> Option<UserTrigger> {
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

/// Compute holiday-adjusted date string and days remaining from today.
pub fn compute_adjusted_date(
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

pub fn deadline_math(rule: &str, trigger: &UserTrigger, days: i64, due: &str, mail_offset: i64) -> String {
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

        // Branch A: reconsideration
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

        // Branch C: stay / extraordinary writ
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

        // Branch D: 30-day extension of time (safeguard when baseline/record is unbuilt or approaching)
        let ext_assessment = crate::consultation::assess_baseline_extension(profile, case_dir);
        let ext_rule = ext_assessment.governing_rule.clone();
        let ext_status = if ext_assessment.is_extension_recommended {
            "recommended".to_string()
        } else {
            "available".to_string()
        };
        let ext_explanation = if ext_assessment.is_extension_recommended {
            format!(
                "Recommended: Case record baseline is currently being assembled ({}). Courts routinely grant a first 30-day extension for good cause to allow self-represented litigants time to retrieve transcripts and verify facts. Secure this via consent letter under {} or formal motion.",
                ext_assessment.missing_elements.join("; "),
                ext_assessment.governing_rule
            )
        } else {
            format!(
                "Available: If you need additional time to prepare filings or confer with counsel, a 30-day extension can be requested under {}.",
                ext_assessment.governing_rule
            )
        };

        branches.push(ProceduralBranch {
            id: "branch-baseline-extension".into(),
            title: "Request 30-day extension of time (record & transcripts)".into(),
            rule_citation: ext_rule,
            trigger_event_title: "Case baseline assembly".into(),
            trigger_date: String::new(),
            trigger_kind: "extension".into(),
            base_days: 30,
            mail_offset_days: 0,
            clock_interaction: "independent".into(),
            tolled_by_event: None,
            days_elapsed_before_toll: None,
            days_remaining_after_toll: None,
            computed_deadline: None,
            days_until_due: None,
            status: ext_status,
            template_kind: "enlargement".into(),
            escalation_target: None,
            explanation: ext_explanation,
            math: String::new(),
            trigger_source: "Case baseline audit".into(),
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
