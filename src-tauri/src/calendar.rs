//! Deadline calendar export (.ics). Does not file or serve.

use std::fs;
use std::path::Path;

use crate::deadlines::DeadlineResult;

pub fn ics_text(result: &DeadlineResult) -> String {
    let ymd = result.due.replace('-', "");
    let uid = format!("{}-{}@openseason.local", result.pack_id, result.rule_id);
    let summary = format!("{} due ({})", result.rule, result.rule_id);
    let desc = format!(
        "{} This app never files or serves. Confirm the date against the current court rules before relying on it.",
        result.math.replace('\n', " ")
    );
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//OpenSeason//Document Builder//EN\r\nBEGIN:VEVENT\r\nUID:{uid}\r\nDTSTAMP:{ymd}T120000Z\r\nDTSTART;VALUE=DATE:{ymd}\r\nSUMMARY:{summary}\r\nDESCRIPTION:{desc}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

pub fn write_ics(case_dir: &Path, result: &DeadlineResult) -> Result<std::path::PathBuf, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Calendar export from the Documents pointer is refused.".into());
    }
    let dest = case_dir.join("exports");
    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    let path = dest.join(format!("deadline-{}-{}.ics", result.pack_id, result.rule_id));
    fs::write(&path, ics_text(result)).map_err(|e| e.to_string())?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deadlines::{compute_deadline, parse_iso_date};
    use crate::rule_pack::RulePack;

    #[test]
    fn ics_contains_due_date() {
        let pack = RulePack::nj_app_div();
        let rule = pack.find_deadline("reconsideration-appellate").unwrap();
        let result = compute_deadline(&pack, rule, parse_iso_date("2026-01-09").unwrap());
        let ics = ics_text(&result);
        assert!(ics.contains("DTSTART;VALUE=DATE:20260120"));
        assert!(ics.contains("BEGIN:VCALENDAR"));
        assert!(ics.contains("never files"));
    }
}
