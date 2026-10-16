//! Minimal MCP stdio server. Same engine as the CLI. Offline. Never files.
//!
//! Write tools default to dry-run. Set `dry_run: false` to apply. Cloud sync,
//! PDF redaction, form fill, and other legal-content generators are not exposed.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::agent;
use crate::case_profile;
use crate::check;
use crate::court_md;
use crate::documents;
use crate::rule_pack::RulePack;

const WRITE_TOOLS: &[&str] = &[
    "case_create",
    "case_seed",
    "draft_import",
    "draft_fill",
    "evidence_add",
    "evidence_index",
    "evidence_link",
    "export_court",
    "export_packet",
    "export_ics",
    "export_exhibit_packet",
    "packs_install",
    "packs_remove",
    "gaps_add",
    "motion_create",
    "motion_export",
    "pdf_merge",
    "pdf_split",
    "pdf_rotate",
    "case_escalate",
    "intake_answer",
    "import_folder",
];

const BLOCKED_TOOLS: &[&str] = &[
    "cloud_pull",
    "cloud_push",
    "pdf_redact",
    "form_fill",
    "sign_pdf",
    "service_cert",
    "service_order",
];

fn tool(name: &str, description: &str, props: Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {"type": "object", "properties": props, "required": required}
    })
}

fn dry_run_schema() -> Value {
    json!({
        "type": "boolean",
        "description": "Default true: report the planned action and change nothing on disk. Set false to apply."
    })
}

fn write_tool(name: &str, description: &str, mut props: Value, required: &[&str]) -> Value {
    if let Some(map) = props.as_object_mut() {
        map.insert("dry_run".into(), dry_run_schema());
    }
    tool(name, description, props, required)
}

fn is_write_tool(name: &str) -> bool {
    WRITE_TOOLS.contains(&name)
}

fn is_explicit_apply(args: &Value) -> bool {
    matches!(args.get("dry_run"), Some(v) if v.as_bool() == Some(false))
}

fn arg_str(args: &Value, k: &str) -> String {
    args.get(k)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn cases_root() -> PathBuf {
    case_profile::default_cases_root()
}

fn resolve_under_cases_root(raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("A path is required.".into());
    }
    let root = cases_root();
    let path = PathBuf::from(trimmed);
    let candidate = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    crate::sandbox::assert_under_root(&root, &candidate)
}

fn case_path(args: &Value) -> Result<PathBuf, String> {
    resolve_under_cases_root(&arg_str(args, "path"))
}

fn optional_case_path(args: &Value) -> Result<Option<PathBuf>, String> {
    let raw = arg_str(args, "path");
    if raw.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(resolve_under_cases_root(&raw)?))
}

fn dry_run_report(name: &str, args: &Value) -> Value {
    let path = arg_str(args, "path");
    let would = match name {
        "case_create" => format!(
            "Create a case folder at {} with title {:?}. Write nothing until dry_run is false.",
            if path.is_empty() { "<path>" } else { path.as_str() },
            arg_str(args, "title")
        ),
        "case_seed" => format!("Seed the fake Lucille Example case at {path}. Write nothing until dry_run is false."),
        "draft_import" => format!("Import the user-supplied Markdown into {path}/drafts/draft.md. Write nothing until dry_run is false."),
        "draft_fill" => format!(
            "Fill guided section {:?} in {} with the user-supplied text. Write nothing until dry_run is false.",
            arg_str(args, "id"),
            path
        ),
        "evidence_add" => format!(
            "Copy exhibit {:?} into {}/evidence/. Write nothing until dry_run is false.",
            arg_str(args, "file"),
            path
        ),
        "evidence_index" => format!("Rebuild the exhibit index in {path}. Write nothing until dry_run is false."),
        "evidence_link" => format!("Link a fact row to an exhibit letter in {path}. Write nothing until dry_run is false."),
        "export_court" => format!("Export Word and PDF for {path} into the case exports folder. Write nothing until dry_run is false."),
        "export_packet" => format!("Assemble the motion packet for {path}. Write nothing until dry_run is false."),
        "export_ics" => format!("Write a deadline calendar file for {path}. Write nothing until dry_run is false."),
        "export_exhibit_packet" => format!("Assemble a Bates-stamped exhibit packet for {path}. Write nothing until dry_run is false."),
        "packs_install" => format!(
            "Install the forms pack for state {:?}. Write nothing until dry_run is false.",
            arg_str(args, "state")
        ),
        "packs_remove" => format!(
            "Remove the forms pack for state {:?}. Write nothing until dry_run is false.",
            arg_str(args, "state")
        ),
        "gaps_add" => format!("Append a justice-gap note in {path}. Write nothing until dry_run is false."),
        "motion_create" => format!(
            "Add a docket motion titled {:?} in {}. Write nothing until dry_run is false.",
            arg_str(args, "title"),
            path
        ),
        "motion_export" => format!("Export motion {:?} in {} to Word and PDF. Write nothing until dry_run is false.", arg_str(args, "id"), path),
        "pdf_merge" => "Merge PDFs into an output path under the cases root. Write nothing until dry_run is false.".into(),
        "pdf_split" => "Split a PDF into an output path under the cases root. Write nothing until dry_run is false.".into(),
        "pdf_rotate" => "Rotate a PDF into an output path under the cases root. Write nothing until dry_run is false.".into(),
        "case_escalate" => format!(
            "Escalate {} to target {:?}. Write nothing until dry_run is false.",
            path,
            arg_str(args, "target")
        ),
        "intake_answer" => format!(
            "Record the user-supplied answer for question {:?} in {}. Write nothing until dry_run is false.",
            arg_str(args, "question_id"),
            path
        ),
        "import_folder" => format!(
            "Copy (never move) files from {:?} into {}. Write nothing until dry_run is false.",
            arg_str(args, "from"),
            path
        ),
        other => format!("Run {other}. Write nothing until dry_run is false."),
    };
    json!({
        "dry_run": true,
        "tool": name,
        "would": would,
        "changed": false,
        "never_files": true,
        "never_serves": true
    })
}

fn tool_list() -> Value {
    let path = json!({"type": "string", "description": "Case folder. Must stay under OPENSEASON_CASES_ROOT (default ~/Documents/JustLegal/Cases)."});
    json!([
        tool("playbook", "Return the offline motion playbook. Read this first. Never invent cites. Never file.", json!({}), &[]),
        tool("case_list", "List cases under the cases root (OPENSEASON_CASES_ROOT or ~/Documents/JustLegal/Cases). Same scan as the window.", json!({"root": {"type": "string", "description": "Optional subdirectory of the cases root"}}), &[]),
        write_tool("case_create", "Create a local case folder. kind: reconsideration|enlargement|stay|amend_findings|appellate_motion|appellate_brief|notice_of_appeal. Defaults to dry-run. Never files.", json!({"path": path, "title": {"type": "string", "description": "Caption like Lucille Example v. Sample County Board"}, "kind": {"type": "string"}, "pack": {"type": "string", "description": "nj-app-div, co-mandamus, or generic-us-appellate"}}), &["path", "title"]),
        write_tool("case_seed", "Copy the fake Lucille Example reconsideration case into path. For tests only. Defaults to dry-run.", json!({"path": path}), &["path"]),
        write_tool("draft_import", "Import a court-markdown draft (YAML caption, ::: signature, ::: certificate). Passes the user's Markdown through. Defaults to dry-run.", json!({"path": path, "markdown": {"type": "string"}}), &["path", "markdown"]),
        tool("draft_preview", "Preview a Markdown draft as a court caption (NJ or CO boxed) plus body. Does not write files. Never files.", json!({"path": path, "markdown": {"type": "string"}}), &["markdown"]),
        tool("draft_sections", "List guided drafting sections and hints for this case kind.", json!({"path": path}), &["path"]),
        write_tool("draft_fill", "Fill one guided section with the user's own words. Do not invent text. Defaults to dry-run.", json!({"path": path, "id": {"type": "string", "description": "Section id from draft_sections"}, "text": {"type": "string"}}), &["path", "id", "text"]),
        write_tool("evidence_add", "Copy an exhibit into evidence/. First file becomes Ex. A after evidence_index. Defaults to dry-run.", json!({"path": path, "file": {"type": "string"}, "description": {"type": "string"}}), &["path", "file"]),
        write_tool("evidence_index", "Rebuild the exhibit index (Ex. A, Ex. B). Defaults to dry-run.", json!({"path": path}), &["path"]),
        write_tool("evidence_link", "Link a fact-check row to an exhibit letter. Does not invent a cite. Defaults to dry-run.", json!({"path": path, "fact_id": {"type": "integer"}, "letter": {"type": "string"}}), &["path", "fact_id", "letter"]),
        tool("check_fact", "Split the draft into sentences. Missing cites stay issues. Optional id/cite/exhibit to attach a user cite.", json!({"path": path, "id": {"type": "integer"}, "cite": {"type": "string"}, "exhibit": {"type": "string"}, "verified": {"type": "boolean"}}), &["path"]),
        tool("check_authority", "List cites already in the draft. Status starts unverified. Never invents a cite.", json!({"path": path, "id": {"type": "integer"}, "status": {"type": "string", "description": "verified|unverified|not_found"}, "source": {"type": "string"}}), &["path"]),
        tool("check_deadline", "Compute a deadline from a rule pack. Weekend/holiday roll. Does not file.", json!({"path": path, "pack": {"type": "string"}, "rule": {"type": "string"}, "trigger_date": {"type": "string", "description": "YYYY-MM-DD"}}), &["path", "pack", "rule", "trigger_date"]),
        tool("check_all", "Run fact, authority, deadline, and sign-off. ok=false until a human checks every box.", json!({"path": path, "pack": {"type": "string"}, "rule": {"type": "string"}, "trigger_date": {"type": "string"}}), &["path"]),
        write_tool("export_court", "Render Word and PDF. Never files or serves. Defaults to dry-run.", json!({"path": path, "kind": {"type": "string"}}), &["path"]),
        write_tool("export_packet", "Assemble notice of motion, motion, certification, proposed order, certificate of service. Defaults to dry-run.", json!({"path": path}), &["path"]),
        write_tool("export_ics", "Write a .ics reminder for a computed deadline. Defaults to dry-run.", json!({"path": path, "pack": {"type": "string"}, "rule": {"type": "string"}, "trigger_date": {"type": "string"}}), &["path", "pack", "rule", "trigger_date"]),
        tool("packs_list", "List bundled rule packs and installable form packs. Shows download sizes. Never files.", json!({"catalog": {"type": "string"}}), &[]),
        tool("packs_validate", "Validate a rule-pack JSON file. New states are data, not code.", json!({"file": {"type": "string"}}), &["file"]),
        write_tool("packs_install", "Install a forms pack for a state (optional situation). Uses a local file or catalog local_dir. Does not open a browser. Defaults to dry-run. Never files.", json!({"state": {"type": "string"}, "situation": {"type": "string"}, "file": {"type": "string"}, "catalog": {"type": "string"}}), &["state"]),
        write_tool("packs_remove", "Remove an installed forms pack. Defaults to dry-run.", json!({"state": {"type": "string"}}), &["state"]),
        tool("packs_verify", "Verify SHA-256 hashes of installed form files.", json!({"state": {"type": "string"}}), &[]),
        tool("guide_questions", "What do I need to file? Return the plain-language questions. Legal information, not advice.", json!({}), &[]),
        tool("guide_find", "Recommend forms or a packet from pack data. Never invents a rule. Includes help_links as data and never opens a browser.", json!({"state": {"type": "string"}, "court": {"type": "string"}, "situation": {"type": "string"}, "trigger_date": {"type": "string", "description": "YYYY-MM-DD"}}), &["state", "situation"]),
        tool("help_links", "JustLegal Get legal help / Find resources URLs as data. Never opens a browser. UI must confirm first.", json!({}), &[]),
        write_tool("gaps_add", "Capture a justice-gap note. Fake names only. Defaults to dry-run.", json!({"path": path, "title": {"type": "string"}, "court": {"type": "string"}, "rule": {"type": "string"}, "happened": {"type": "string"}, "hard": {"type": "string"}}), &["path", "title"]),
        tool("check_preflight", "Court readiness pre-flight checklist. Verifies caption, docket, filer, deadline, fact cites, authorities, transcripts, and packet parts.", json!({"path": path}), &["path"]),
        tool("check_transcripts", "Audit hearing transcripts in evidence/ and check grounding of [Tr. Page:Line] citations.", json!({"path": path}), &["path"]),
        write_tool("export_exhibit_packet", "Assemble Table of Exhibits and Bates-stamped Exhibit Packet PDF with exhibit divider sheets. Defaults to dry-run.", json!({"path": path, "prefix": {"type": "string"}, "start": {"type": "integer"}}), &["path"]),
        tool("form_inspect", "Inspect court PDF form fields (e.g. Colorado JDF series or NJ forms). Does not fill or edit the form.", json!({"pdf": {"type": "string"}, "path": path}), &["pdf"]),
        tool("motion_list", "List all motions and filings in a case docket.", json!({"path": path}), &["path"]),
        write_tool("motion_create", "Add a new motion or filing to the case docket using the user's title and text. Defaults to dry-run.", json!({"path": path, "title": {"type": "string"}, "kind": {"type": "string"}, "rule": {"type": "string"}, "text": {"type": "string"}}), &["path", "title"]),
        write_tool("motion_export", "Export a specific motion from the case docket to Word and PDF. Defaults to dry-run.", json!({"path": path, "id": {"type": "string"}}), &["path", "id"]),
        write_tool("pdf_merge", "Merge multiple PDFs into one file under the cases root. Defaults to dry-run.", json!({"files": {"type": "array", "items": {"type": "string"}}, "out": {"type": "string"}}), &["files", "out"]),
        write_tool("pdf_split", "Extract specific pages from a PDF into a file under the cases root. Defaults to dry-run.", json!({"pdf": {"type": "string"}, "pages": {"type": "string"}, "out": {"type": "string"}}), &["pdf", "pages", "out"]),
        write_tool("pdf_rotate", "Rotate PDF pages by 90, 180, or 270 degrees. Output stays under the cases root. Defaults to dry-run.", json!({"pdf": {"type": "string"}, "degrees": {"type": "integer"}, "out": {"type": "string"}}), &["pdf", "degrees", "out"]),
        tool("case_context", "Consolidated 1-call Legal OS snapshot: profile, tier, venue rules, timeline, exhibits, filings, intake wizard status, and predictive next steps.", json!({"path": path}), &["path"]),
        tool("case_next_steps", "Predictive Procedural State Graph: compute downstream branches, deadlines, and tolling (NJ pause vs CO/Fed reset) from timeline & orders.", json!({"path": path, "event": {"type": "string"}, "trigger_date": {"type": "string"}}), &["path"]),
        write_tool("case_escalate", "Graduate Tier 2 Pre-Suit -> Tier 3 Lawsuit or escalate Trial -> Appellate / Mandamus / SCOTUS, carrying over record exhibits. Defaults to dry-run.", json!({"path": path, "target": {"type": "string", "description": "trial_lawsuit|appellate_appeal|mandamus|scotus_cert"}}), &["path", "target"]),
        tool("intake_status", "1-question-at-a-time Guided Intake Wizard status, progress percentage, and next_question prompt.", json!({"path": path, "filing_id": {"type": "string"}}), &["path"]),
        write_tool("intake_answer", "Answer a single question in the Guided Intake Wizard with the user's words. Defaults to dry-run.", json!({"path": path, "question_id": {"type": "string"}, "answer": {"type": "string"}, "filing_id": {"type": "string"}}), &["path", "question_id", "answer"]),
        tool("intake_scan", "Offline PDF text extraction & candidate detection (docket numbers, dates, courts) across orders/ and evidence/.", json!({"path": path}), &["path"]),
        tool("check_venue", "Deterministic Multi-Venue Competence Matrix check (NJ R. 1:4-4(b) oath, CO conferral/declaration as needs_verification, 28 U.S.C. § 1746, SCOTUS Rule 14.1, Will witnesses).", json!({"path": path}), &["path"]),
        tool("portfolio_status", "Multi-state, multi-venue Portfolio Deadline Radar across all matters in the cases directory.", json!({"root": {"type": "string"}}), &[]),
        tool("pdf_extract_text", "Extract text page-by-page from a PDF under the cases root (never uploads).", json!({"pdf": {"type": "string"}}), &["pdf"]),
        tool("agent_manifest", "List every CLI/MCP tool with a one-line description, MCP launch config, and a copyable starter prompt. Never files.", json!({}), &[]),
        write_tool("import_folder", "Copy (never move) drafts/notes/orders from a folder into a case. Proposes parties, dockets, and order dates. Does not auto-commit guesses. Defaults to dry-run.", json!({"path": path, "from": {"type": "string", "description": "Folder of drafts, notes, and orders"}, "title": {"type": "string"}}), &["path", "from"]),
    ])
}

fn call_tool(name: &str, args: &Value) -> Result<Value, String> {
    if BLOCKED_TOOLS.contains(&name) {
        return Err(format!(
            "Tool '{name}' is not available on the MCP surface. Cloud sync, PDF redaction, form fill, and generated legal papers stay off this server."
        ));
    }
    if is_write_tool(name) && !is_explicit_apply(args) {
        return Ok(dry_run_report(name, args));
    }
    dispatch_tool(name, args)
}

fn dispatch_tool(name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "case_list" => {
            let root = if arg_str(args, "root").is_empty() {
                cases_root()
            } else {
                resolve_under_cases_root(&arg_str(args, "root"))?
            };
            let listed = case_profile::list_cases_with_folders(&root)?;
            let cases: Vec<Value> = listed
                .into_iter()
                .map(|(folder, profile)| {
                    json!({
                        "id": profile.id,
                        "title": profile.title,
                        "document_kind": profile.document_kind,
                        "mode": profile.mode,
                        "docket_number": profile.docket_number,
                        "rule_pack_id": profile.rule_pack_id,
                        "updated_at": profile.updated_at,
                        "folder": folder,
                        "sealed_at": profile.sealed_at,
                    })
                })
                .collect();
            Ok(json!({
                "root": root,
                "cases": cases,
                "never_files": true,
                "never_serves": true
            }))
        }
        "case_create" => {
            let path = case_path(args)?;
            let kind = if arg_str(args, "kind").is_empty() {
                "motion".into()
            } else {
                arg_str(args, "kind")
            };
            let (dir, mut profile) = case_profile::create_case_folder(
                &path,
                &arg_str(args, "title"),
                case_profile::CaseMode::Standard,
                &kind,
            )?;
            if !arg_str(args, "pack").is_empty() {
                profile.rule_pack_id = arg_str(args, "pack");
                profile.court.id = arg_str(args, "pack");
                case_profile::save_case(&dir, &profile)?;
            }
            Ok(json!({"id": profile.id, "folder": dir, "never_files": true, "dry_run": false}))
        }
        "draft_import" => {
            let path = case_path(args)?;
            let doc = court_md::import_markdown(&path, &arg_str(args, "markdown"))?;
            Ok(json!({"lines": doc.lines.len(), "case_no": doc.meta.case_no, "never_files": true, "dry_run": false}))
        }
        "draft_preview" => {
            let profile = match optional_case_path(args)? {
                None => case_profile::CaseProfile::new_standard("Preview", "motion"),
                Some(path) => case_profile::load_case(&path)?,
            };
            let preview = court_md::preview_markdown(&profile, &arg_str(args, "markdown"));
            Ok(serde_json::to_value(preview).unwrap())
        }
        "evidence_add" => {
            let path = case_path(args)?;
            let file = arg_str(args, "file");
            let bytes = std::fs::read(&file).map_err(|e| e.to_string())?;
            let dest = check::add_exhibit(
                &path,
                Path::new(&file)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("exhibit.bin"),
                &bytes,
                &arg_str(args, "description"),
            )?;
            Ok(json!({"path": dest, "never_files": true, "dry_run": false}))
        }
        "check_fact" => {
            let path = case_path(args)?;
            let draft = check::draft_text_for_case(&path)?;
            let report = check::run_fact_check(&path, &draft)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "check_authority" => {
            let path = case_path(args)?;
            let draft = check::draft_text_for_case(&path)?;
            let report = check::run_authority_check(&path, &draft)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "check_deadline" => {
            let path = case_path(args)?;
            let report = check::run_deadline(
                &path,
                &arg_str(args, "pack"),
                &arg_str(args, "rule"),
                &arg_str(args, "trigger_date"),
            )?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "check_all" => {
            let path = case_path(args)?;
            let pack = arg_str(args, "pack");
            let rule = arg_str(args, "rule");
            let trigger = arg_str(args, "trigger_date");
            let report = check::run_all(
                &path,
                if pack.is_empty() { None } else { Some(pack.as_str()) },
                if rule.is_empty() { None } else { Some(rule.as_str()) },
                if trigger.is_empty() {
                    None
                } else {
                    Some(trigger.as_str())
                },
            )?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "export_court" => {
            let path = case_path(args)?;
            let profile = case_profile::load_case(&path)?;
            let kind = if arg_str(args, "kind").is_empty() {
                "court_md".into()
            } else {
                arg_str(args, "kind")
            };
            let exported = documents::export_pleading(&path, &profile, &kind, None)?;
            Ok(json!({"docx": exported.docx, "pdf": exported.pdf, "never_files": true, "dry_run": false}))
        }
        "playbook" => Ok(json!({"playbook": crate::help::playbook()})),
        "case_seed" => {
            let path = case_path(args)?;
            let (dir, profile) = crate::seed::seed_lucille(&path)?;
            Ok(json!({"folder": dir, "id": profile.id, "title": profile.title, "never_files": true, "dry_run": false}))
        }
        "draft_sections" => {
            let path = case_path(args)?;
            let profile = case_profile::load_case(&path)?;
            let draft = crate::guided::load_or_init(&path, &profile.document_kind)?;
            Ok(serde_json::to_value(draft).unwrap())
        }
        "draft_fill" => {
            let path = case_path(args)?;
            let draft = crate::guided::set_section(&path, &arg_str(args, "id"), &arg_str(args, "text"))?;
            Ok(serde_json::to_value(draft).unwrap())
        }
        "evidence_index" => {
            Ok(serde_json::to_value(crate::exhibits::rebuild(&case_path(args)?)?).unwrap())
        }
        "evidence_link" => {
            let path = case_path(args)?;
            let fact_id = args.get("fact_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            Ok(serde_json::to_value(crate::exhibits::link_fact(
                &path,
                fact_id,
                &arg_str(args, "letter"),
            )?)
            .unwrap())
        }
        "export_packet" => {
            Ok(serde_json::to_value(crate::packet::assemble_for_case(&case_path(args)?)?).unwrap())
        }
        "export_ics" => {
            let dir = case_path(args)?;
            let result = check::run_deadline(
                &dir,
                &arg_str(args, "pack"),
                &arg_str(args, "rule"),
                &arg_str(args, "trigger_date"),
            )?;
            let ics = crate::calendar::write_ics(&dir, &result)?;
            Ok(json!({"ics": ics, "due": result.due, "never_files": true, "dry_run": false}))
        }
        "packs_list" => {
            let catalog = if arg_str(args, "catalog").is_empty() {
                None
            } else {
                Some(PathBuf::from(arg_str(args, "catalog")))
            };
            Ok(serde_json::to_value(crate::forms::list_packs(catalog.as_deref(), None)?).unwrap())
        }
        "packs_validate" => {
            Ok(serde_json::to_value(RulePack::validate_file(Path::new(&arg_str(args, "file")))?).unwrap())
        }
        "packs_install" => {
            let file = if arg_str(args, "file").is_empty() {
                None
            } else {
                Some(PathBuf::from(arg_str(args, "file")))
            };
            let catalog = if arg_str(args, "catalog").is_empty() {
                None
            } else {
                Some(PathBuf::from(arg_str(args, "catalog")))
            };
            let situation = if arg_str(args, "situation").is_empty() {
                None
            } else {
                Some(arg_str(args, "situation"))
            };
            Ok(serde_json::to_value(crate::forms::install(crate::forms::InstallOpts {
                state: arg_str(args, "state"),
                situation,
                catalog,
                file,
                root: None,
                allow_network: false,
            })?)
            .unwrap())
        }
        "packs_remove" => Ok(json!({"removed": crate::forms::remove_pack(&arg_str(args, "state"), None)?, "dry_run": false})),
        "packs_verify" => {
            let state = if arg_str(args, "state").is_empty() {
                None
            } else {
                Some(arg_str(args, "state"))
            };
            Ok(serde_json::to_value(crate::forms::verify_pack(state.as_deref(), None)?).unwrap())
        }
        "guide_questions" => Ok(serde_json::to_value(crate::guide::questions()?).unwrap()),
        "guide_find" => {
            let court = if arg_str(args, "court").is_empty() {
                None
            } else {
                Some(arg_str(args, "court"))
            };
            let trigger = if arg_str(args, "trigger_date").is_empty() {
                None
            } else {
                Some(arg_str(args, "trigger_date"))
            };
            Ok(serde_json::to_value(crate::guide::find(
                &arg_str(args, "state"),
                court.as_deref(),
                &arg_str(args, "situation"),
                trigger.as_deref(),
            )?)
            .unwrap())
        }
        "help_links" => Ok(serde_json::to_value(crate::help_links::HelpLinks::load()?).unwrap()),
        "gaps_add" => Ok(serde_json::to_value(crate::gaps::add(
            &case_path(args)?,
            &arg_str(args, "title"),
            &arg_str(args, "court"),
            &arg_str(args, "rule"),
            &arg_str(args, "happened"),
            &arg_str(args, "hard"),
        )?)
        .unwrap()),
        "check_preflight" => {
            let report = check::preflight_audit(&case_path(args)?)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "check_transcripts" => {
            let dir = case_path(args)?;
            let profile = case_profile::load_case(&dir)?;
            let draft = profile.notice_of_appeal.user_text.as_str();
            let report = check::check_transcripts(&dir, draft)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "export_exhibit_packet" => {
            let dir = case_path(args)?;
            let prefix_s = arg_str(args, "prefix");
            let prefix = if prefix_s.is_empty() {
                None
            } else {
                Some(prefix_s.as_str())
            };
            let start = args.get("start").and_then(|v| v.as_u64()).map(|n| n as u32);
            let report = crate::packet::assemble_exhibit_packet(&dir, prefix, start)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "form_inspect" => {
            let pdf = PathBuf::from(arg_str(args, "pdf"));
            let case = optional_case_path(args)?;
            let report = crate::forms::inspect_court_form(&pdf, case.as_deref())?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "motion_list" => {
            let profile = case_profile::load_case(&case_path(args)?)?;
            Ok(serde_json::to_value(profile.filings).unwrap())
        }
        "motion_create" => {
            let dir = case_path(args)?;
            let mut profile = case_profile::load_case(&dir)?;
            let now = chrono::Local::now().to_rfc3339();
            let filing = case_profile::CaseFilingItem {
                id: uuid::Uuid::new_v4().to_string(),
                title: arg_str(args, "title"),
                kind: if arg_str(args, "kind").is_empty() {
                    "motion".into()
                } else {
                    arg_str(args, "kind")
                },
                rule: arg_str(args, "rule"),
                status: "draft".into(),
                user_text: arg_str(args, "text"),
                relief_sought: String::new(),
                created_at: now.clone(),
                updated_at: now,
            };
            profile.filings.push(filing.clone());
            case_profile::save_case(&dir, &profile)?;
            Ok(serde_json::to_value(filing).unwrap())
        }
        "motion_export" => {
            let dir = case_path(args)?;
            let profile = case_profile::load_case(&dir)?;
            let id = arg_str(args, "id");
            let filing = profile
                .filings
                .iter()
                .find(|f| f.id == id)
                .ok_or_else(|| format!("Filing ID '{id}' not found in case."))?;
            let paths = crate::documents::export_custom_filing(
                &dir,
                &profile,
                &filing.title,
                &filing.user_text,
                None,
            )?;
            Ok(serde_json::to_value(paths).unwrap())
        }
        "pdf_merge" => {
            let files_arr = args
                .get("files")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let mut paths = Vec::new();
            for v in &files_arr {
                let raw = v.as_str().unwrap_or("");
                paths.push(resolve_under_cases_root(raw)?);
            }
            let out = resolve_under_cases_root(&arg_str(args, "out"))?;
            let result = crate::pdf_tools::merge_pdfs(&paths, &out)?;
            Ok(serde_json::to_value(result).unwrap())
        }
        "pdf_split" => {
            let pdf = resolve_under_cases_root(&arg_str(args, "pdf"))?;
            let out = resolve_under_cases_root(&arg_str(args, "out"))?;
            let pages_str = arg_str(args, "pages");
            let mut page_list = Vec::new();
            for part in pages_str.split([',', ' ']) {
                let trimmed = part.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Some((s_str, e_str)) = trimmed.split_once('-') {
                    if let (Ok(start), Ok(end)) =
                        (s_str.trim().parse::<u32>(), e_str.trim().parse::<u32>())
                    {
                        for p in start..=end {
                            page_list.push(p);
                        }
                    }
                } else if let Ok(num) = trimmed.parse::<u32>() {
                    page_list.push(num);
                }
            }
            let result = crate::pdf_tools::split_pdf(&pdf, &page_list, &out)?;
            Ok(serde_json::to_value(result).unwrap())
        }
        "pdf_rotate" => {
            let pdf = resolve_under_cases_root(&arg_str(args, "pdf"))?;
            let out = resolve_under_cases_root(&arg_str(args, "out"))?;
            let degrees = args.get("degrees").and_then(|v| v.as_i64()).unwrap_or(90);
            let result = crate::pdf_tools::rotate_pdf(&pdf, degrees, None, &out)?;
            Ok(serde_json::to_value(result).unwrap())
        }
        "case_context" => {
            let dir = case_path(args)?;
            Ok(serde_json::to_value(crate::intake::case_context(&dir)?).unwrap())
        }
        "case_next_steps" => {
            let dir = case_path(args)?;
            let profile = case_profile::load_case(&dir)?;
            let report = crate::legal_os::predict_next_steps(&profile, Some(&dir));
            Ok(serde_json::to_value(report).unwrap())
        }
        "case_escalate" => {
            let dir = case_path(args)?;
            let report = crate::portfolio::escalate_case(&dir, &arg_str(args, "target"), None)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "intake_status" => {
            let dir = case_path(args)?;
            let report = crate::intake::intake_status(&dir)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "intake_answer" => {
            let dir = case_path(args)?;
            let report =
                crate::intake::intake_answer(&dir, &arg_str(args, "question_id"), &arg_str(args, "answer"))?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "intake_scan" => {
            let dir = case_path(args)?;
            Ok(serde_json::to_value(crate::intake::intake_scan(&dir)).unwrap())
        }
        "check_venue" => {
            let dir = case_path(args)?;
            let profile = case_profile::load_case(&dir)?;
            Ok(serde_json::to_value(crate::legal_os::check_venue_competence(&profile)).unwrap())
        }
        "portfolio_status" => {
            let root = if arg_str(args, "root").is_empty() {
                cases_root()
            } else {
                resolve_under_cases_root(&arg_str(args, "root"))?
            };
            Ok(serde_json::to_value(crate::portfolio::portfolio_status(&root)?).unwrap())
        }
        "pdf_extract_text" => {
            let pdf = resolve_under_cases_root(&arg_str(args, "pdf"))?;
            Ok(serde_json::to_value(crate::pdf_tools::extract_pdf_text(&pdf)?).unwrap())
        }
        "agent_manifest" => Ok(crate::help::agent_manifest()),
        "import_folder" => {
            let dest = case_path(args)?;
            let from = arg_str(args, "from");
            let title = arg_str(args, "title");
            let report = crate::folder_import::import_folder(
                &dest,
                Path::new(&from),
                if title.is_empty() {
                    None
                } else {
                    Some(title.as_str())
                },
            )?;
            Ok(serde_json::to_value(report).unwrap())
        }
        other => Err(format!(
            "Unknown tool '{other}'. Call playbook or packs_list. Known tools are listed by tools/list."
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Framing {
    ContentLength,
    Newline,
}

fn write_message(out: &mut impl Write, body: &Value, framing: Framing) -> Result<(), String> {
    let bytes = serde_json::to_vec(body).map_err(|e| e.to_string())?;
    match framing {
        Framing::ContentLength => {
            write!(out, "Content-Length: {}\r\n\r\n", bytes.len()).map_err(|e| e.to_string())?;
            out.write_all(&bytes).map_err(|e| e.to_string())?;
        }
        Framing::Newline => {
            out.write_all(&bytes).map_err(|e| e.to_string())?;
            out.write_all(b"\n").map_err(|e| e.to_string())?;
        }
    }
    out.flush().map_err(|e| e.to_string())
}

fn read_message(
    reader: &mut impl BufRead,
    framing: &mut Option<Framing>,
) -> Result<Option<Value>, String> {
    let mut first = String::new();
    let n = reader.read_line(&mut first).map_err(|e| e.to_string())?;
    if n == 0 {
        return Ok(None);
    }
    let trimmed = first.trim();
    if trimmed.is_empty() {
        return read_message(reader, framing);
    }
    if trimmed.to_ascii_lowercase().starts_with("content-length:") {
        *framing = Some(Framing::ContentLength);
        let mut headers = first;
        loop {
            let mut line = String::new();
            let n = reader.read_line(&mut line).map_err(|e| e.to_string())?;
            if n == 0 {
                return Ok(None);
            }
            if line == "\r\n" || line == "\n" {
                break;
            }
            headers.push_str(&line);
        }
        let mut len = 0usize;
        for line in headers.lines() {
            if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                len = v.trim().parse().map_err(|e| format!("{e}"))?;
            }
        }
        if len == 0 {
            return Ok(None);
        }
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
        return serde_json::from_slice(&buf)
            .map(Some)
            .map_err(|e| e.to_string());
    }
    if trimmed.starts_with('{') {
        *framing = Some(Framing::Newline);
        return serde_json::from_str(trimmed)
            .map(Some)
            .map_err(|e| e.to_string());
    }
    Err(format!(
        "Unrecognized MCP frame: {}",
        trimmed.chars().take(80).collect::<String>()
    ))
}

fn handle(msg: &Value) -> Option<Value> {
    let method = msg.get("method")?.as_str()?;
    let id = msg.get("id").cloned();
    let result = match method {
        "initialize" => json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "openseason", "version": "0.1.0"}
        }),
        "notifications/initialized" | "initialized" => return None,
        "tools/list" => json!({"tools": tool_list()}),
        "tools/call" => {
            let params = msg.get("params").cloned().unwrap_or(json!({}));
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            match call_tool(name, &args) {
                Ok(value) => {
                    let enveloped = agent::wrap_value(value);
                    json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string_pretty(&enveloped).unwrap_or_else(|_| "{}".into())
                        }]
                    })
                }
                Err(err) => json!({
                    "isError": true,
                    "content": [{
                        "type": "text",
                        "text": agent::wrap_error(err).to_string()
                    }]
                }),
            }
        }
        other => {
            return Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": format!("Unknown method {other}")}
            }))
        }
    };
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::{Mutex, MutexGuard};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct CasesRootGuard {
        _lock: MutexGuard<'static, ()>,
        root: PathBuf,
    }

    impl CasesRootGuard {
        fn new() -> Self {
            let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let root = std::env::temp_dir().join(format!("os-mcp-root-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&root).unwrap();
            std::env::set_var("OPENSEASON_CASES_ROOT", &root);
            Self { _lock: lock, root }
        }

        fn case_dir(&self, name: &str) -> PathBuf {
            self.root.join(name)
        }
    }

    impl Drop for CasesRootGuard {
        fn drop(&mut self) {
            std::env::remove_var("OPENSEASON_CASES_ROOT");
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn tool_names() -> Vec<String> {
        let reply = handle(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }))
        .unwrap();
        reply["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect()
    }

    fn folder_fingerprint(dir: &Path) -> Vec<(String, Vec<u8>)> {
        fn walk(base: &Path, current: &Path, out: &mut Vec<(String, Vec<u8>)>) {
            if !current.exists() {
                return;
            }
            let mut entries: Vec<_> = std::fs::read_dir(current)
                .unwrap()
                .map(|e| e.unwrap())
                .collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                let path = entry.path();
                let rel = path
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                if path.is_dir() {
                    walk(base, &path, out);
                } else {
                    out.push((rel, std::fs::read(&path).unwrap()));
                }
            }
        }
        let mut out = Vec::new();
        walk(dir, dir, &mut out);
        out
    }

    fn sample_markdown() -> String {
        std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("fixtures")
                .join("sample-reconsideration.md"),
        )
        .unwrap_or_else(|_| {
            "---\ncourt: Sample\nplaintiff: Jordan Example\ndefendant: Sample County Clerk\ncase_no: A-01000-24\n---\nRelief is sought under R. 2:11-6.\n".into()
        })
    }

    /// JSON-RPC `tools/call` envelope (`ok`, `never_files`, `data`, `issues`).
    fn mcp_envelope(name: &str, args: &Value) -> Value {
        let reply = handle(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {"name": name, "arguments": args}
        }))
        .expect("MCP tools/call should reply");
        assert_ne!(
            reply["result"]["isError"], true,
            "MCP tool {name} returned isError: {reply}"
        );
        let text = reply["result"]["content"][0]["text"]
            .as_str()
            .expect("MCP content text");
        serde_json::from_str(text)
            .unwrap_or_else(|e| panic!("envelope JSON for {name}: {e}; {text}"))
    }

    #[test]
    fn initialize_and_tools_list() {
        let init = handle(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize"
        }))
        .unwrap();
        assert_eq!(init["jsonrpc"], "2.0");
        assert_eq!(init["id"], 1);
        assert_eq!(init["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(init["result"]["serverInfo"]["name"], "openseason");
        assert!(init["result"]["capabilities"]["tools"].is_object());

        let names = tool_names();
        for required in [
            "case_list",
            "case_create",
            "draft_import",
            "evidence_add",
            "check_fact",
            "check_authority",
            "check_deadline",
            "check_all",
            "export_court",
            "packs_list",
            "packs_install",
            "packs_verify",
            "guide_find",
            "help_links",
        ] {
            assert!(
                names.iter().any(|n| n == required),
                "missing {required} in {names:?}"
            );
        }
        for blocked in BLOCKED_TOOLS {
            assert!(
                !names.iter().any(|n| n == blocked),
                "blocked tool {blocked} must be absent from tools/list, got {names:?}"
            );
        }
        assert!(!names.iter().any(|n| n.contains("cloud")));
    }

    #[test]
    fn tools_list_names_the_agent_surface() {
        let names = tool_names();
        assert!(names.contains(&"case_list".to_string()));
        assert!(names.contains(&"case_create".to_string()));
        assert!(!names.contains(&"cloud_pull".to_string()));
        assert!(!names.contains(&"cloud_push".to_string()));
        assert!(!names.contains(&"pdf_redact".to_string()));
        assert!(!names.contains(&"form_fill".to_string()));
    }

    #[test]
    fn content_length_roundtrip() {
        let mut buf = Vec::new();
        write_message(&mut buf, &json!({"hello": "world"}), Framing::ContentLength).unwrap();
        let mut reader = BufReader::new(Cursor::new(buf));
        let mut framing = None;
        let msg = read_message(&mut reader, &mut framing).unwrap().unwrap();
        assert_eq!(msg["hello"], "world");
        assert_eq!(framing, Some(Framing::ContentLength));
    }

    #[test]
    fn newline_delimited_jsonrpc_roundtrip() {
        let mut buf = Vec::new();
        write_message(
            &mut buf,
            &json!({"jsonrpc": "2.0", "id": 1, "method": "ping"}),
            Framing::Newline,
        )
        .unwrap();
        let mut reader = BufReader::new(Cursor::new(buf));
        let mut framing = None;
        let msg = read_message(&mut reader, &mut framing).unwrap().unwrap();
        assert_eq!(msg["method"], "ping");
        assert_eq!(framing, Some(Framing::Newline));
    }

    #[test]
    fn case_list_reuses_engine_listing() {
        let guard = CasesRootGuard::new();
        let empty = call_tool("case_list", &json!({})).unwrap();
        assert_eq!(empty["never_files"], true);
        assert!(empty["cases"].as_array().unwrap().is_empty());

        let dir = guard.case_dir("listed-example");
        let (_created, profile) = case_profile::create_case_folder(
            &dir,
            "Example v. Sample County Clerk",
            case_profile::CaseMode::Standard,
            "reconsideration",
        )
        .unwrap();

        let listed = call_tool("case_list", &json!({})).unwrap();
        let cases = listed["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0]["id"], profile.id);
        assert_eq!(cases[0]["title"], "Example v. Sample County Clerk");
        assert_eq!(cases[0]["document_kind"], "reconsideration");
        let folder = PathBuf::from(cases[0]["folder"].as_str().unwrap());
        assert!(folder.join("case.json").is_file());
    }

    #[test]
    fn write_tools_dry_run_leaves_folder_byte_identical() {
        let guard = CasesRootGuard::new();
        let dir = guard.case_dir("dry-run-example");
        case_profile::create_case_folder(
            &dir,
            "Example v. Sample County Clerk",
            case_profile::CaseMode::Standard,
            "reconsideration",
        )
        .unwrap();
        let before = folder_fingerprint(&dir);

        let created = call_tool(
            "case_create",
            &json!({
                "path": dir,
                "title": "Example v. Sample County Clerk",
                "kind": "reconsideration",
                "pack": "nj-app-div"
            }),
        )
        .unwrap();
        assert_eq!(created["dry_run"], true);
        assert_eq!(created["changed"], false);
        assert!(created["would"].as_str().unwrap().contains("Write nothing"));

        let imported = call_tool(
            "draft_import",
            &json!({"path": dir, "markdown": sample_markdown()}),
        )
        .unwrap();
        assert_eq!(imported["dry_run"], true);
        assert_eq!(imported["changed"], false);

        let exhibit = guard.root.join("order-excerpt.txt");
        std::fs::write(&exhibit, b"Order granting dismissal (fake exhibit)\n").unwrap();
        let added = call_tool(
            "evidence_add",
            &json!({"path": dir, "file": exhibit, "description": "Dkt. 12"}),
        )
        .unwrap();
        assert_eq!(added["dry_run"], true);

        let exported =
            call_tool("export_court", &json!({"path": dir, "kind": "court_md"})).unwrap();
        assert_eq!(exported["dry_run"], true);
        assert_eq!(exported["changed"], false);

        let after = folder_fingerprint(&dir);
        assert_eq!(
            before, after,
            "dry-run must leave the case folder byte-identical"
        );
        assert!(!dir.join("drafts").join("draft.md").is_file());
    }

    #[test]
    fn mcp_refuses_writes_outside_cases_root() {
        let _guard = CasesRootGuard::new();
        let outside = std::env::temp_dir().join(format!("os-mcp-outside-{}", uuid::Uuid::new_v4()));
        let err = call_tool(
            "case_create",
            &json!({
                "path": outside,
                "title": "Example v. Sample County Clerk",
                "kind": "reconsideration",
                "dry_run": false
            }),
        )
        .unwrap_err();
        assert!(
            err.to_lowercase().contains("outside") || err.to_lowercase().contains("cases root"),
            "expected sandbox refusal, got {err}"
        );
        assert!(!outside.join("case.json").exists());
        let _ = std::fs::remove_dir_all(outside);
    }

    #[test]
    fn mcp_tools_create_import_check_export() {
        let guard = CasesRootGuard::new();
        let dir = guard.case_dir("roundtrip-example");
        let created = call_tool(
            "case_create",
            &json!({
                "path": dir,
                "title": "Example v. Sample County Clerk",
                "kind": "reconsideration",
                "pack": "nj-app-div",
                "dry_run": false
            }),
        )
        .unwrap();
        assert_eq!(created["never_files"], true);
        assert_eq!(created["dry_run"], false);
        assert!(dir.join("case.json").is_file());

        let md = sample_markdown();
        call_tool(
            "draft_import",
            &json!({"path": dir, "markdown": md, "dry_run": false}),
        )
        .unwrap();
        let exhibit = dir.join("order-excerpt.txt");
        std::fs::write(&exhibit, b"Order granting dismissal (fake exhibit)\n").unwrap();
        call_tool(
            "evidence_add",
            &json!({"path": dir, "file": exhibit, "description": "Dkt. 12", "dry_run": false}),
        )
        .unwrap();
        let fact = call_tool("check_fact", &json!({"path": dir})).unwrap();
        assert_eq!(fact["never_files"], true);
        assert_eq!(fact["ok"], false);
        let authority = call_tool("check_authority", &json!({"path": dir})).unwrap();
        assert_eq!(authority["invented"], false);
        let deadline = call_tool(
            "check_deadline",
            &json!({
                "path": dir,
                "pack": "nj-app-div",
                "rule": "reconsideration-appellate",
                "trigger_date": "2026-01-09"
            }),
        )
        .unwrap();
        assert_eq!(deadline["due"], "2026-01-20");
        let all = call_tool(
            "check_all",
            &json!({
                "path": dir,
                "pack": "nj-app-div",
                "rule": "reconsideration-appellate",
                "trigger_date": "2026-01-09"
            }),
        )
        .unwrap();
        assert_eq!(all["never_files"], true);
        assert_eq!(all["never_serves"], true);
        assert_eq!(
            all["ok"], false,
            "check_all stays not ok until a human signs off"
        );
        assert_eq!(all["signoff"]["ok"], false);
        assert!(all["signoff"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["checked"] == false));
        let exported = call_tool(
            "export_court",
            &json!({"path": dir, "kind": "court_md", "dry_run": false}),
        )
        .unwrap();
        let docx = exported["docx"].as_str().unwrap();
        let pdf = exported["pdf"].as_str().unwrap();
        assert!(Path::new(docx).is_file());
        assert!(Path::new(pdf).is_file());
        let wrapped = handle(&json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/call",
            "params": {"name": "packs_list", "arguments": {}}
        }))
        .unwrap();
        let text = wrapped["result"]["content"][0]["text"].as_str().unwrap();
        let env: Value = serde_json::from_str(text).unwrap();
        assert_eq!(env["never_files"], true);
        assert_eq!(env["ok"], true);
        let guide = call_tool(
            "guide_find",
            &json!({"state": "NJ", "situation": "reconsideration", "trigger_date": "2026-01-09"}),
        )
        .unwrap();
        assert_eq!(guide["never_files"], true);
        assert_eq!(guide["citation"], "R. 2:11-6");
        assert_eq!(guide["deadline"]["due"], "2026-01-20");
        assert!(guide["help_links"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l["id"] == "find-legal-help"));
        let links = call_tool("help_links", &json!({})).unwrap();
        assert_eq!(links["opens_browser"], false);
    }

    #[test]
    fn check_all_stays_not_ok_until_human_signoff_and_cloud_tools_absent() {
        let names = tool_names();
        assert!(
            !names.contains(&"cloud_pull".to_string()),
            "cloud_pull must be absent from tools/list, got {names:?}"
        );
        assert!(
            !names.contains(&"cloud_push".to_string()),
            "cloud_push must be absent from tools/list, got {names:?}"
        );
        assert!(
            !names.iter().any(|n| n.contains("cloud")),
            "no cloud* tool on the MCP surface, got {names:?}"
        );
        assert!(
            !names
                .iter()
                .any(|n| { n.contains("signoff") || n.contains("sign_off") || n == "set_signoff" }),
            "sign-off is a human gate; MCP must not expose a sign-off tool, got {names:?}"
        );

        let guard = CasesRootGuard::new();
        let dir = guard.case_dir("signoff-example");
        call_tool(
            "case_create",
            &json!({
                "path": dir,
                "title": "Example v. Sample County Clerk",
                "kind": "reconsideration",
                "pack": "nj-app-div",
                "dry_run": false
            }),
        )
        .unwrap();
        call_tool(
            "draft_import",
            &json!({"path": dir, "markdown": sample_markdown(), "dry_run": false}),
        )
        .unwrap();
        let exhibit = dir.join("order-excerpt.txt");
        std::fs::write(&exhibit, b"Order granting dismissal (fake exhibit)\n").unwrap();
        call_tool(
            "evidence_add",
            &json!({"path": dir, "file": exhibit, "description": "Dkt. 12", "dry_run": false}),
        )
        .unwrap();

        let check_args = json!({
            "path": dir,
            "pack": "nj-app-div",
            "rule": "reconsideration-appellate",
            "trigger_date": "2026-01-09"
        });
        let before = mcp_envelope("check_all", &check_args);
        assert_eq!(before["never_files"], true);
        assert_eq!(before["never_serves"], true);
        assert_eq!(
            before["ok"], false,
            "MCP check_all envelope ok=false before human sign-off: {before}"
        );
        assert_eq!(before["data"]["ok"], false);
        assert_eq!(before["data"]["signoff"]["ok"], false);

        // User-supplied fixture cites (not invented by MCP). Isolates the sign-off gate.
        let fact_rows = before["data"]["fact"]["rows"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert!(
            !fact_rows.is_empty(),
            "sample draft should produce fact rows: {before}"
        );
        for row in &fact_rows {
            let id = row["id"].as_u64().unwrap() as u32;
            check::set_fact_cite(
                &dir,
                id,
                "Ex. A (order-excerpt.txt, Dkt. 12)",
                "order-excerpt.txt",
                true,
            )
            .unwrap();
        }
        let authority_rows = before["data"]["authority"]["rows"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        for row in &authority_rows {
            let id = row["id"].as_u64().unwrap() as u32;
            check::set_authority_status(
                &dir,
                id,
                check::AuthorityStatus::Verified,
                "fixture://sample-reconsideration.md",
                "test fixture: user-supplied verification",
            )
            .unwrap();
        }

        let unsigned = mcp_envelope("check_all", &check_args);
        assert_eq!(
            unsigned["ok"], false,
            "MCP check_all stays ok=false while sign-off boxes are unchecked: {unsigned}"
        );
        assert_eq!(unsigned["data"]["ok"], false);
        assert_eq!(unsigned["data"]["fact"]["ok"], true);
        assert_eq!(unsigned["data"]["authority"]["ok"], true);
        assert_eq!(unsigned["data"]["signoff"]["ok"], false);
        assert!(unsigned["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["field"] == "signoff"));

        let items = unsigned["data"]["signoff"]["items"]
            .as_array()
            .cloned()
            .unwrap();
        assert!(!items.is_empty());
        for item in &items {
            check::set_signoff(&dir, item["id"].as_str().unwrap(), true).unwrap();
        }

        let signed = mcp_envelope("check_all", &check_args);
        assert_eq!(signed["never_files"], true);
        assert_eq!(signed["never_serves"], true);
        assert_eq!(
            signed["data"]["signoff"]["ok"], true,
            "engine sign-off should check every box: {signed}"
        );
        assert_eq!(
            signed["ok"], true,
            "MCP check_all ok=true only after the human sign-off gate: {signed}"
        );
        assert_eq!(signed["data"]["ok"], true);
    }
}

pub fn serve_stdio() -> Result<(), String> {
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = std::io::stdout();
    let mut framing = None;
    eprintln!(
        "openseason mcp: offline stdio server. Never files or serves. Cases root: {}. Write tools default to dry-run.",
        cases_root().display()
    );
    loop {
        match read_message(&mut reader, &mut framing)? {
            None => break,
            Some(msg) => {
                if let Some(reply) = handle(&msg) {
                    write_message(
                        &mut stdout,
                        &reply,
                        framing.unwrap_or(Framing::ContentLength),
                    )?;
                }
            }
        }
    }
    Ok(())
}
