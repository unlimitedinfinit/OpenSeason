use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::holidays::{holidays_for_set, next_business_day};
use crate::rule_pack::{DeadlineRule, RulePack};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeadlineVerificationStatus {
    Verified,
    Provisional,
    Disputed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourcedDeadline {
    pub pack_id: String,
    pub rule_id: String,
    pub governing_rule: String,
    pub governing_rule_citation: String,
    pub trigger_event: String,
    pub trigger_date: String,
    pub trigger_source: String,
    pub day_count: i64,
    pub raw_due_date: String,
    pub due_date: String,
    pub days_remaining: i64,
    pub status: DeadlineVerificationStatus,
    pub calculation_basis: String,
    pub filing_method: String,
    pub extensions_verified: String,
    pub rolled_over_weekend_or_holiday: bool,
    pub enlargeable_under: Option<String>,
    pub tolls_appeal_under: Option<String>,
    pub verification_checklist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeadlineResult {
    pub pack_id: String,
    pub rule_id: String,
    pub rule: String,
    pub trigger: String,
    pub trigger_date: String,
    pub day_count: i64,
    pub raw_due: String,
    pub due: String,
    pub rolled: bool,
    pub enlargeable_under: Option<String>,
    pub tolls_appeal_under: Option<String>,
    pub notes: String,
    pub never_files: bool,
    pub math: String,
}

/// R. 1:3-1 style: exclude the trigger day, include the last day, then
/// roll Saturday / Sunday / holiday to the next business day.
pub fn add_court_days(trigger: NaiveDate, days: i64, holidays: &[NaiveDate]) -> (NaiveDate, bool) {
    let raw = trigger + chrono::Duration::days(days);
    let due = next_business_day(raw, holidays);
    (due, due != raw)
}

pub fn compute_deadline(
    pack: &RulePack,
    rule: &DeadlineRule,
    trigger_date: NaiveDate,
) -> DeadlineResult {
    let holidays = holidays_for_set(&pack.holiday_set, trigger_date.year());
    let next_year = holidays_for_set(&pack.holiday_set, trigger_date.year() + 1);
    let mut all = holidays;
    all.extend(next_year);
    all.sort();
    all.dedup();

    let (due, rolled) = if rule.days <= 0 {
        (trigger_date, false)
    } else {
        add_court_days(trigger_date, rule.days, &all)
    };
    let raw = trigger_date + chrono::Duration::days(rule.days.max(0));
    let math = format!(
        "Trigger ({}) {} + {} calendar days = {}. Weekend/holiday roll ({}) => {}.",
        rule.trigger,
        trigger_date,
        rule.days,
        raw,
        rule.weekend_holiday_roll,
        due
    );
    DeadlineResult {
        pack_id: pack.id.clone(),
        rule_id: rule.id.clone(),
        rule: rule.rule.clone(),
        trigger: rule.trigger.clone(),
        trigger_date: trigger_date.to_string(),
        day_count: rule.days,
        raw_due: raw.to_string(),
        due: due.to_string(),
        rolled,
        enlargeable_under: rule.enlargeable_under.clone(),
        tolls_appeal_under: rule.tolls_appeal_under.clone(),
        notes: rule.notes.clone(),
        never_files: true,
        math,
    }
}

pub fn compute_sourced_deadline(
    pack: &RulePack,
    rule: &DeadlineRule,
    trigger_date: NaiveDate,
    trigger_source: Option<&str>,
    status: Option<DeadlineVerificationStatus>,
    filing_method: Option<&str>,
) -> SourcedDeadline {
    let base_dl = compute_deadline(pack, rule, trigger_date);
    let today = chrono::Local::now().date_naive();
    let due_naive = NaiveDate::parse_from_str(&base_dl.due, "%Y-%m-%d").unwrap_or(today);
    let days_remaining = (due_naive - today).num_days();

    let ver_status = status.unwrap_or(DeadlineVerificationStatus::Provisional);
    let trigger_src = trigger_source
        .unwrap_or("User-entered trigger date")
        .to_string();
    let filing_meth = filing_method
        .unwrap_or("Court electronic filing system / Clerk counter")
        .to_string();

    let mut checklist = Vec::new();
    checklist.push(format!("Verify exact date of '{}' from official docket sheet or stamped copy", rule.trigger));
    checklist.push("Confirm whether service was electronic (same-day) or via postal mail (+ additional days)".into());
    if base_dl.rolled {
        checklist.push("Holiday/weekend roll applied: confirm local court holiday closure calendar".into());
    }
    if let Some(ref enl) = rule.enlargeable_under {
        checklist.push(format!("Extension possibility: review motion for enlargement under {}", enl));
    }

    SourcedDeadline {
        pack_id: pack.id.clone(),
        rule_id: rule.id.clone(),
        governing_rule: rule.notes.clone(),
        governing_rule_citation: rule.rule.clone(),
        trigger_event: rule.trigger.clone(),
        trigger_date: trigger_date.to_string(),
        trigger_source: trigger_src,
        day_count: rule.days,
        raw_due_date: base_dl.raw_due,
        due_date: base_dl.due,
        days_remaining,
        status: ver_status,
        calculation_basis: base_dl.math,
        filing_method: filing_meth,
        extensions_verified: "None verified".to_string(),
        rolled_over_weekend_or_holiday: base_dl.rolled,
        enlargeable_under: rule.enlargeable_under.clone(),
        tolls_appeal_under: rule.tolls_appeal_under.clone(),
        verification_checklist: checklist,
    }
}

pub fn parse_iso_date(value: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .map_err(|_| format!("Date '{value}' must be YYYY-MM-DD."))
}

trait YearExt {
    fn year(&self) -> i32;
}
impl YearExt for NaiveDate {
    fn year(&self) -> i32 {
        chrono::Datelike::year(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule_pack::RulePack;

    #[test]
    fn nj_reconsideration_10_days_rolls_over_mlk_2026() {
        let pack = RulePack::nj_app_div();
        let rule = pack.find_deadline("reconsideration-appellate").unwrap();
        // Entry Friday 2026-01-09 + 10 days = 2026-01-19 (MLK) -> 2026-01-20.
        let result = compute_deadline(
            &pack,
            rule,
            NaiveDate::from_ymd_opt(2026, 1, 9).unwrap(),
        );
        assert_eq!(result.raw_due, "2026-01-19");
        assert_eq!(result.due, "2026-01-20");
        assert!(result.rolled);
        assert_eq!(result.rule, "R. 2:11-6");
        assert_eq!(result.enlargeable_under.as_deref(), Some("R. 1:3-4(a)"));
        assert_eq!(result.tolls_appeal_under.as_deref(), Some("R. 2:4-3(e)"));
    }

    #[test]
    fn trial_motion_20_days_from_service() {
        let pack = RulePack::nj_app_div();
        let rule = pack.find_deadline("reconsideration-trial").unwrap();
        let result = compute_deadline(
            &pack,
            rule,
            NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(),
        );
        assert_eq!(result.trigger, "service");
        assert_eq!(result.day_count, 20);
        assert_eq!(result.raw_due, "2026-03-22");
        // 2026-03-22 is Sunday -> Monday 23.
        assert_eq!(result.due, "2026-03-23");
    }

    #[test]
    fn test_sourced_deadline_carries_provenance_and_verification_checklist() {
        let pack = RulePack::nj_app_div();
        let rule = pack.find_deadline("reconsideration-appellate").unwrap();
        let sourced = compute_sourced_deadline(
            &pack,
            rule,
            NaiveDate::from_ymd_opt(2026, 1, 9).unwrap(),
            Some("Order PDF: orders/order-2026-01-09.pdf#page=2"),
            Some(DeadlineVerificationStatus::Provisional),
            Some("NJ eCourts Appellate (11:59 PM EST)"),
        );
        assert_eq!(sourced.status, DeadlineVerificationStatus::Provisional);
        assert_eq!(sourced.trigger_source, "Order PDF: orders/order-2026-01-09.pdf#page=2");
        assert_eq!(sourced.filing_method, "NJ eCourts Appellate (11:59 PM EST)");
        assert_eq!(sourced.due_date, "2026-01-20");
        assert!(sourced.rolled_over_weekend_or_holiday);
        assert!(sourced.calculation_basis.contains("R. 1:3-1") || sourced.calculation_basis.contains("Weekend/holiday roll"));
        assert!(sourced.verification_checklist.iter().any(|c| c.contains("Holiday/weekend roll")));
    }
}
