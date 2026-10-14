//! Plain-language field help and the agent playbook. Offline. Never files.

use serde::Serialize;

use crate::check::NEVER_FILE_NOTICE;

pub const PLAYBOOK: &str = r#"# Agent playbook: court-ready motion (offline)

Read AGENTS.md first. Never invent facts, arguments, or citations. Never file or serve.

## One full reconsideration run (fake names only)

1. case_create path=./tmp-motion title="Lucille Example v. Sample County Board" kind=reconsideration pack=nj-app-div
2. draft_sections path=./tmp-motion  (read the hints; do not invent text)
3. draft_fill path=./tmp-motion id=order text="<user-supplied order date and what it did>"
4. draft_fill id=rule text="<user-supplied rule, or leave blank>"
5. draft_fill id=grounds text="<user-supplied grounds>"
6. draft_fill id=relief text="<user-supplied request>"
7. evidence_add path=./tmp-motion file=<user exhibit> description="Dkt. 12"
8. evidence_index path=./tmp-motion   (first file is Ex. A)
9. check_fact path=./tmp-motion
10. For each fact with no cite: evidence_link path=./tmp-motion fact_id=N letter=A  OR ask the user
11. check_authority path=./tmp-motion  (status starts unverified; do not mark verified unless the user directed a source)
12. check_deadline pack=nj-app-div rule=reconsideration-appellate trigger_date=YYYY-MM-DD
13. export_ics if the user wants a calendar reminder
14. check_all  (sign-off stays unchecked until a human checks every box)
15. export_packet  (notice of motion, motion, certification, proposed order, certificate of service)
16. Stop. Tell the user they must read every page, sign, file, and serve.

If a tool returns ok=false, read issues[].message. It says how to fix the field. Do not guess a cite to clear an issue.

## What do I need to file?

Call `guide_questions` then `guide_find` with the user's state, court, situation, and optional trigger date. Recommend only forms the pack lists. If citation_status is not pack_source, tell the user it needs verification. Do not invent a deadline. `help_links` is data only — never open a browser. The window confirms before opening JustLegal.
"#;

#[derive(Debug, Clone, Serialize)]
pub struct FieldHelp {
    pub id: String,
    pub label: String,
    pub help: String,
}

pub fn field_help() -> Vec<FieldHelp> {
    vec![
        FieldHelp { id: "title".into(), label: "Case title".into(), help: "Write it like Lucille Example v. Sample County Board. Use the names on the papers you have.".into() },
        FieldHelp { id: "court".into(), label: "Court name".into(), help: "Copy the court name from the caption of an existing paper. Do not guess a court.".into() },
        FieldHelp { id: "docket".into(), label: "Docket or case number".into(), help: "Copy the number exactly. If you do not have it, leave it blank and the checker will say so.".into() },
        FieldHelp { id: "parties".into(), label: "Parties".into(), help: "Add yourself and the other side. Role is Plaintiff, Defendant, Appellant, or Appellee. Counsel can be Pro se.".into() },
        FieldHelp { id: "order".into(), label: "The order".into(), help: "Date and what the order did. Look at the paper. This app will not invent an order.".into() },
        FieldHelp { id: "grounds".into(), label: "Grounds".into(), help: "Why you think the order is wrong, in your own words. Every fact needs an exhibit letter.".into() },
        FieldHelp { id: "exhibit".into(), label: "Exhibits".into(), help: "Add the papers you already have. The first file becomes Ex. A. Link each fact to a letter.".into() },
        FieldHelp { id: "deadline".into(), label: "Deadline".into(), help: "Enter the date the order was entered (YYYY-MM-DD). The NJ pack counts court days and rolls weekends and holidays. Confirm the date yourself.".into() },
        FieldHelp { id: "signoff".into(), label: "Sign-off".into(), help: "You must check every box. The app will not file or serve for you.".into() },
    ]
}

pub fn playbook() -> String {
    format!("{NEVER_FILE_NOTICE}\n\n{PLAYBOOK}")
}

pub fn how_to_fix(field: &str) -> String {
    let f = field.to_ascii_lowercase();
    if f.starts_with("fact.") {
        return "Fix: add an exhibit, run evidence index, then evidence_link --fact-id N --letter A. Or check fact --id N --cite \"Ex. A\" --exhibit A. Do not invent a cite.".into();
    }
    if f.starts_with("authority.") {
        return "Fix: verify the cite on a live source the user names, then check authority --id N --status verified --source URL. Or leave it unverified and tell the user.".into();
    }
    if f == "deadline" {
        return "Fix: check deadline --pack nj-app-div --rule reconsideration-appellate --trigger-date YYYY-MM-DD".into();
    }
    if f == "signoff" {
        return "Fix: a human must check every box. check signoff --id read-pdf --checked. The app still will not file.".into();
    }
    if f.contains("user_text") || f.contains("court_markdown") {
        return "Fix: draft fill --id <section> --text \"user words\", or import Markdown. Do not invent the body.".into();
    }
    if f.contains("docket") {
        return "Fix: copy the case number from a court paper into case.json docket_number.".into();
    }
    "Fix: fill the field with a value from the user's papers. Leave it blank rather than guessing.".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playbook_forbids_inventing_cites() {
        assert!(PLAYBOOK.contains("Never invent"));
        assert!(PLAYBOOK.contains("export_packet"));
        assert!(how_to_fix("fact.1").contains("Do not invent"));
    }
}
