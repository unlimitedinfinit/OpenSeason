//! Guided drafting sections. Same prompts for the window and the agent CLI.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::case_profile;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuidedSection {
    pub id: String,
    pub heading: String,
    pub hint: String,
    #[serde(default)]
    pub user_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuidedDraft {
    pub kind: String,
    pub sections: Vec<GuidedSection>,
}

pub fn templates_for(kind: &str) -> Vec<GuidedSection> {
    let key = kind.trim().to_ascii_lowercase();
    let rows: &[(&str, &str, &str)] = match key.as_str() {
        "enlargement" => &[
            ("deadline", "The deadline that needs more time", "What paper is due, and on what date? Write only what you know."),
            ("reason", "Why you need more time", "Your own reason. Do not invent facts. If you do not know, leave this blank."),
            ("new_date", "The new date you are asking for", "A date you are requesting. This app does not decide if the court will grant it."),
        ],
        "stay" => &[
            ("order", "The order you want paused", "Name the order and the date it was entered. Use the record, not a guess."),
            ("why", "Why a stay is needed", "Your own words. This app will not invent harm or urgency."),
            ("terms", "What you want the stay to do", "What should be paused, and for how long, in your own words."),
        ],
        "amend_findings" => &[
            ("finding", "The finding you want changed", "Quote or point to the finding. Do not invent a finding the court did not make."),
            ("amendment", "The change you are asking for", "The words you want the court to use instead."),
            ("record", "Where the record supports the change", "Exhibit letter or transcript page. This checker will not invent a cite."),
        ],
        "appellate_brief" | "brief" => &[
            ("prelim", "Preliminary statement", "Two or three sentences in your own words about what this appeal is."),
            ("history", "Procedural history", "What was filed and what the trial court did. Dates from the record only."),
            ("facts", "Statement of facts", "Each fact needs a record cite. Leave a fact out rather than guessing."),
            ("argument", "Legal argument", "Points you researched. Do not invent case citations."),
            ("conclusion", "Conclusion", "The relief you want, in your own words."),
        ],
        "appellate_motion" => &[
            ("relief", "What you want the appellate court to do", "One sentence. This is a basic R. 2:8-1 form, not a brief."),
            ("grounds", "Your grounds", "Your own reasons. Do not invent rules or cases."),
            ("record", "Record support", "Exhibits or docket entries you already have."),
        ],
        "mandamus" => &[
            ("jurisdiction", "Why this court can hear this", "C.A.R. 21 is a public rule number, not legal advice. Write only what you know."),
            ("issues", "Issues presented", "The questions you want answered. Do not invent them."),
            ("facts", "Facts from the record", "Each fact needs a cite. The Colorado pack will not invent the showing."),
            ("reasons", "Reasons relief should issue", "Your own words."),
            ("prayer", "Prayer for relief", "What you want the court to order."),
        ],
        "complaint" => &[
            ("parties", "Who is suing whom", "Names you already entered in the caption."),
            ("claim", "What happened", "Your own statement of the claim. This is not a full interview."),
            ("relief", "What you want the court to do", "Your own request."),
        ],
        "notice_of_appeal" => &[
            ("order", "The order or judgment you are appealing", "Date and a short description from the record."),
            ("notice", "Your notice text", "Your own words. This app will not invent grounds of appeal."),
        ],
        _ => &[
            ("order", "The order you want the court to look at again", "Date of the order and what it did. Copy from the paper you have."),
            ("rule", "The rule you are using", "If you do not have a rule, leave this blank. The app will not invent one. NJ reconsideration is often R. 2:11-6 or R. 4:49-2. Confirm the current rule yourself."),
            ("grounds", "Why the order is wrong, in your own words", "Write only facts you can point to in an exhibit. Do not invent."),
            ("relief", "What you want the court to do", "For example, vacate the order and restore the case. Your words, not ours."),
        ],
    };
    rows.iter()
        .map(|(id, heading, hint)| GuidedSection {
            id: (*id).into(),
            heading: (*heading).into(),
            hint: (*hint).into(),
            user_text: String::new(),
        })
        .collect()
}

pub fn path(case_dir: &Path) -> std::path::PathBuf {
    case_dir.join("drafts").join("sections.json")
}

pub fn load_or_init(case_dir: &Path, kind: &str) -> Result<GuidedDraft, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Guided drafts cannot be written on the Documents pointer.".into());
    }
    if let Some(existing) = crate::check::read_json::<GuidedDraft>(&path(case_dir))? {
        return Ok(existing);
    }
    let draft = GuidedDraft {
        kind: kind.to_string(),
        sections: templates_for(kind),
    };
    save(case_dir, &draft)?;
    Ok(draft)
}

pub fn save(case_dir: &Path, draft: &GuidedDraft) -> Result<(), String> {
    fs::create_dir_all(case_dir.join("drafts")).map_err(|e| e.to_string())?;
    crate::check::write_json(&path(case_dir), draft)
}

pub fn assemble(sections: &[GuidedSection]) -> String {
    let mut out = String::new();
    for section in sections {
        if section.user_text.trim().is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(&section.heading);
        out.push('\n');
        out.push_str(section.user_text.trim());
    }
    out
}

/// Write one section and copy the assembled body onto the matching profile field.
pub fn set_section(case_dir: &Path, id: &str, text: &str) -> Result<GuidedDraft, String> {
    let profile = case_profile::load_case(case_dir)?;
    let kind = if profile.document_kind.is_empty() {
        "reconsideration"
    } else {
        profile.document_kind.as_str()
    };
    let mut draft = load_or_init(case_dir, kind)?;
    let known: Vec<String> = draft.sections.iter().map(|s| s.id.clone()).collect();
    let section = draft.sections.iter_mut().find(|s| s.id == id).ok_or_else(|| {
        format!(
            "No section '{id}'. Known ids: {}. Use draft sections to list them.",
            known.join(", ")
        )
    })?;
    section.user_text = text.to_string();
    save(case_dir, &draft)?;
    let body = assemble(&draft.sections);
    let mut profile = profile;
    match draft.kind.to_ascii_lowercase().as_str() {
        "appellate_brief" | "brief" => profile.brief.user_text = body,
        "appellate_motion" => profile.appellate_motion.user_text = body,
        "mandamus" => profile.mandamus.user_text = body,
        "complaint" => profile.complaint.user_text = body,
        "notice_of_appeal" => profile.notice_of_appeal.user_text = body,
        _ => profile.motion.user_text = body,
    }
    case_profile::save_case(case_dir, &profile)?;
    Ok(draft)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_profile::{ensure_case_layout, sample_appeal_profile, save_case};
    use uuid::Uuid;

    #[test]
    fn reconsideration_has_order_rule_grounds_relief() {
        let ids: Vec<_> = templates_for("reconsideration").into_iter().map(|s| s.id).collect();
        assert_eq!(ids, vec!["order", "rule", "grounds", "relief"]);
    }

    #[test]
    fn filling_a_section_writes_the_motion_body() {
        let dir = std::env::temp_dir().join(format!("os-guided-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        ensure_case_layout(&dir).unwrap();
        let mut profile = sample_appeal_profile();
        profile.document_kind = "reconsideration".into();
        save_case(&dir, &profile).unwrap();
        set_section(&dir, "grounds", "The clerk mailed the order on March 1.").unwrap();
        let loaded = case_profile::load_case(&dir).unwrap();
        assert!(loaded.motion.user_text.contains("The clerk mailed the order on March 1."));
        let _ = fs::remove_dir_all(dir);
    }
}
