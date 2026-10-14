use chrono::{Datelike, NaiveDate, Weekday};

/// Weekend or listed holiday. Used by court-day roll (next business day).
pub fn is_non_business_day(date: NaiveDate, extra: &[NaiveDate]) -> bool {
    matches!(date.weekday(), Weekday::Sat | Weekday::Sun) || extra.contains(&date)
}

pub fn next_business_day(mut date: NaiveDate, extra: &[NaiveDate]) -> NaiveDate {
    while is_non_business_day(date, extra) {
        date = date.succ_opt().expect("date stays in range");
    }
    date
}

/// US federal holidays plus New Jersey observed days for one calendar year.
/// Confirm against the court's current holiday list before relying on a date.
pub fn holidays_for_set(set: &str, year: i32) -> Vec<NaiveDate> {
    let mut days = federal_holidays(year);
    if set == "us-nj" {
        days.extend(nj_extra_holidays(year));
    }
    days.sort();
    days.dedup();
    days
}

fn observed(date: NaiveDate) -> NaiveDate {
    match date.weekday() {
        Weekday::Sat => date.pred_opt().unwrap(),
        Weekday::Sun => date.succ_opt().unwrap(),
        _ => date,
    }
}

fn nth_weekday(year: i32, month: u32, weekday: Weekday, n: u8) -> NaiveDate {
    let mut d = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let mut seen = 0u8;
    loop {
        if d.weekday() == weekday {
            seen += 1;
            if seen == n {
                return d;
            }
        }
        d = d.succ_opt().unwrap();
        if d.month() != month {
            panic!("nth weekday out of month");
        }
    }
}

fn last_weekday(year: i32, month: u32, weekday: Weekday) -> NaiveDate {
    let mut d = NaiveDate::from_ymd_opt(year, month, 1).unwrap()
        + chrono::Months::new(1)
        - chrono::Days::new(1);
    while d.weekday() != weekday {
        d = d.pred_opt().unwrap();
    }
    d
}

fn federal_holidays(year: i32) -> Vec<NaiveDate> {
    vec![
        observed(NaiveDate::from_ymd_opt(year, 1, 1).unwrap()),
        nth_weekday(year, 1, Weekday::Mon, 3), // MLK
        nth_weekday(year, 2, Weekday::Mon, 3), // Presidents
        last_weekday(year, 5, Weekday::Mon),   // Memorial
        observed(NaiveDate::from_ymd_opt(year, 6, 19).unwrap()),
        observed(NaiveDate::from_ymd_opt(year, 7, 4).unwrap()),
        nth_weekday(year, 9, Weekday::Mon, 1), // Labor
        nth_weekday(year, 10, Weekday::Mon, 2), // Columbus
        observed(NaiveDate::from_ymd_opt(year, 11, 11).unwrap()),
        nth_weekday(year, 11, Weekday::Thu, 4), // Thanksgiving
        observed(NaiveDate::from_ymd_opt(year, 12, 25).unwrap()),
    ]
}

fn nj_extra_holidays(year: i32) -> Vec<NaiveDate> {
    // Lincoln's Birthday (Feb 12), Good Friday, Election Day.
    // Good Friday is the Friday before Easter Sunday (Anonymous Gregorian).
    let easter = gregorian_easter(year);
    let good_friday = easter.pred_opt().unwrap().pred_opt().unwrap();
    let election = {
        // First Tuesday after the first Monday in November.
        let first_monday = nth_weekday(year, 11, Weekday::Mon, 1);
        first_monday.succ_opt().unwrap()
    };
    vec![
        observed(NaiveDate::from_ymd_opt(year, 2, 12).unwrap()),
        good_friday,
        election,
    ]
}

fn gregorian_easter(year: i32) -> NaiveDate {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = (h + l - 7 * m + 114) % 31 + 1;
    NaiveDate::from_ymd_opt(year, month as u32, day as u32).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mlk_2026_is_january_19() {
        let days = holidays_for_set("us-federal", 2026);
        assert!(days.contains(&NaiveDate::from_ymd_opt(2026, 1, 19).unwrap()));
    }

    #[test]
    fn saturday_rolls_to_monday() {
        let sat = NaiveDate::from_ymd_opt(2026, 1, 10).unwrap();
        assert_eq!(
            next_business_day(sat, &[]),
            NaiveDate::from_ymd_opt(2026, 1, 12).unwrap()
        );
    }
}
