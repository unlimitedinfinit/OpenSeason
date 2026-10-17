//! Plain-language field help and the agent playbook. Offline. Never files.

use serde::Serialize;
use serde_json::{json, Value};

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

## Fast Onboarding for an Existing Folder (Legal OS)

1. `case_context` (`justlegalos-cli --json case context <dir>`): One call returns the complete case profile, 3-tier classification (`transactional`, `administrative`, `litigation`), governing venue rules (`NJ`, `CO`, `FEDERAL`, `SCOTUS`), timeline events, indexed exhibits, filings, Guided Intake Wizard status, and Predictive Next Steps.
2. `intake_scan` (`justlegalos-cli --json intake scan <dir>`): Extracts text offline from PDFs in `orders/` and `evidence/` and surfaces candidate docket numbers, dates, and courts with provenance.
3. `intake_status` / `intake_answer` (`justlegalos-cli --json intake status <dir>`): Drives a 1-question-at-a-time wizard (`next_question`). Supports filing-scoped sections (`--filing-id <id>`) so multiple motions never overwrite each other.
4. `case_next_steps` (`justlegalos-cli --json case next-steps <dir>`): Computes downstream procedural branches from logged events/orders, including **NJ Pause/Toll (`pause_remaining` under R. 2:4-3(e))** vs. **CO/Federal Full Clock Reset (`reset_full` under FRAP 4(a)(4) / C.R.C.P. 59)** and **Mail Service Offset (+3 days)**.
5. `check_venue` (`justlegalos-cli --json check venue <dir>`): Audits venue-specific rules (e.g. NJ R. 1:4-4(b) mandatory certification oath rejecting `"to the best of my knowledge"`, CO conferral/declaration labels marked `needs_verification` because C.A.R. may apply on appeal and a UUDA declaration may need date and place, SCOTUS Rule 14.1 Questions Presented, or Will 2-witness self-proving requirements).
6. `case_escalate` (`justlegalos-cli --json case escalate <dir> --target trial_lawsuit|appellate_appeal|mandamus|scotus_cert`): Graduates a Pre-Suit matter to a Lawsuit or escalates a Trial case to Appeal/Mandamus/SCOTUS while copying lower-court orders/filings into the new case's `evidence/` appendix.

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

/// One-page contract for a local AI: tools, case folder, MCP launch, starter prompt.
pub fn agent_manifest() -> Value {
    let tools = vec![
        json!({"name": "playbook", "cli": "docs playbook", "mcp": "playbook", "summary": "Read the offline motion playbook first. Never invent cites. Never file."}),
        json!({"name": "case_create", "cli": "case create", "mcp": "case_create", "summary": "Create a local case folder from a title and kind."}),
        json!({"name": "import_folder", "cli": "case import-folder", "mcp": "import_folder", "summary": "Copy drafts, notes, and orders into a case. Proposes names and dates. Does not auto-commit guesses."}),
        json!({"name": "draft_import", "cli": "draft import", "mcp": "draft_import", "summary": "Import the user's Markdown draft. Does not invent body text."}),
        json!({"name": "draft_preview", "cli": "draft preview --md FILE", "mcp": "draft_preview", "summary": "Preview caption and body from Markdown. Does not write files."}),
        json!({"name": "check_deadline", "cli": "check deadline", "mcp": "check_deadline", "summary": "Compute a deadline only from a user-entered trigger date. Weekend/holiday roll."}),
        json!({"name": "check_preflight", "cli": "check preflight", "mcp": "check_preflight", "summary": "Court-readiness audit. Outer JSON ok is false when the case is not ready."}),
        json!({"name": "export_court", "cli": "export court-md / export pleading", "mcp": "export_court", "summary": "Render Word and PDF. Never files or serves."}),
        json!({"name": "guide_find", "cli": "guide find", "mcp": "guide_find", "summary": "Recommend only forms the pack lists. Do not invent a deadline."}),
        json!({"name": "mcp", "cli": "mcp", "mcp": "(server)", "summary": "Stdio JSON-RPC. Accepts Content-Length framing or newline-delimited JSON."}),
    ];
    json!({
        "ok": true,
        "never_files": true,
        "never_serves": true,
        "notice": NEVER_FILE_NOTICE,
        "case_folder": "Documents/JustLegal/Cases/{caseId}/",
        "mcp_launch": {
            "command": "justlegalos-cli",
            "args": ["mcp"],
            "cwd": "(repository root or the folder that contains JustLegalOS-CLI)"
        },
        "tools": tools,
        "starter_prompt": "You are helping with JustLegal OS, an offline legal operating system. Read AGENTS.md. Never invent facts, citations, or deadlines. Never file or serve. Call `justlegalos-cli --json docs playbook` (or MCP playbook) first. If a tool returns ok:false, stop and ask the user. Use only values from the case folder the user points you at."
    })
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

    #[test]
    fn manifest_lists_tools_and_forbids_filing() {
        let m = agent_manifest();
        assert_eq!(m["never_files"], true);
        assert!(m["tools"].as_array().unwrap().len() >= 5);
        assert!(m["starter_prompt"].as_str().unwrap().contains("Never invent"));
        assert!(m["mcp_launch"]["args"][0] == "mcp");
    }
}
