//! Unit tests for Legal OS venue competence, procedural graph, and tolling.

use super::procedural::*;
use super::types::*;
use super::venue::*;
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
    assert_eq!(appeal.days_remaining_after_toll, Some(38));
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
    assert!(
        (18..=24).contains(&days),
        "days_until_due {days} should be ~20 local days, not a UTC-skewed value"
    );
}
