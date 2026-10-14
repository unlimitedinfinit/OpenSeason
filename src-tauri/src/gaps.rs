//! Justice-gap notes. Structured capture for later content use. Fake names only.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GapNote {
    pub id: String,
    pub created_at: String,
    pub title: String,
    pub court: String,
    pub rule: String,
    pub what_happened: String,
    pub what_was_hard: String,
    pub never_identifies_real_people: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GapLog {
    pub notes: Vec<GapNote>,
}

pub fn json_path(case_dir: &Path) -> std::path::PathBuf {
    case_dir.join("gaps").join("notes.json")
}

pub fn load(case_dir: &Path) -> Result<GapLog, String> {
    Ok(crate::check::read_json(&json_path(case_dir))?.unwrap_or(GapLog { notes: Vec::new() }))
}

pub fn add(
    case_dir: &Path,
    title: &str,
    court: &str,
    rule: &str,
    what_happened: &str,
    what_was_hard: &str,
) -> Result<GapLog, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Gap notes cannot be written on the Documents pointer.".into());
    }
    let mut log = load(case_dir)?;
    log.notes.push(GapNote {
        id: Uuid::new_v4().to_string(),
        created_at: Utc::now().to_rfc3339(),
        title: title.trim().to_string(),
        court: court.trim().to_string(),
        rule: rule.trim().to_string(),
        what_happened: what_happened.trim().to_string(),
        what_was_hard: what_was_hard.trim().to_string(),
        never_identifies_real_people: true,
    });
    fs::create_dir_all(case_dir.join("gaps")).map_err(|e| e.to_string())?;
    crate::check::write_json(&json_path(case_dir), &log)?;
    Ok(log)
}

pub fn export_markdown(case_dir: &Path) -> Result<std::path::PathBuf, String> {
    let log = load(case_dir)?;
    let mut md = String::from("# Justice gap notes\n\nFake names only. Do not paste real litigants.\n\n");
    for note in &log.notes {
        md.push_str(&format!(
            "## {}\n\n- Court: {}\n- Rule: {}\n- When: {}\n\n### What happened\n\n{}\n\n### What was hard\n\n{}\n\n",
            note.title, note.court, note.rule, note.created_at, note.what_happened, note.what_was_hard
        ));
    }
    fs::create_dir_all(case_dir.join("gaps")).map_err(|e| e.to_string())?;
    let path = case_dir.join("gaps").join("notes.md");
    fs::write(&path, md).map_err(|e| e.to_string())?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn add_and_export_uses_fake_names_flag() {
        let dir = std::env::temp_dir().join(format!("os-gap-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let log = add(
            &dir,
            "Clerk would not stamp a motion",
            "Sample County",
            "R. 2:11-6",
            "Lucille Example brought a fake motion to the clerk window.",
            "No plain-language form for reconsideration.",
        )
        .unwrap();
        assert!(log.notes[0].never_identifies_real_people);
        let md = export_markdown(&dir).unwrap();
        assert!(fs::read_to_string(md).unwrap().contains("What was hard"));
        let _ = fs::remove_dir_all(dir);
    }
}
