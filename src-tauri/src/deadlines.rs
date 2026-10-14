use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::holidays::{holidays_for_set, next_business_day};
use crate::rule_pack::{DeadlineRule, RulePack};

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
}
