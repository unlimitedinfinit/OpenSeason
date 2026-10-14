//! Minimal MCP stdio server. Same engine as the CLI. Offline. Never files.

use std::io::{BufRead, BufReader, Write};

use serde_json::{json, Value};

use crate::agent;
use crate::case_profile;
use crate::check;
use crate::court_md;
use crate::documents;
use crate::rule_pack::RulePack;

fn tool(name: &str, description: &str, props: Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {"type": "object", "properties": props, "required": required}
    })
}

fn tool_list() -> Value {
    let path = json!({"type": "string", "description": "Absolute path to the case folder (contains case.json)."});
    json!([
        tool("playbook", "Return the offline motion playbook. Read this first. Never invent cites. Never file.", json!({}), &[]),
        tool("case_create", "Create a local case folder. kind: reconsideration|enlargement|stay|amend_findings|appellate_motion|appellate_brief|notice_of_appeal. Never files.", json!({"path": path, "title": {"type": "string", "description": "Caption like Lucille Example v. Sample County Board"}, "kind": {"type": "string"}, "pack": {"type": "string", "description": "nj-app-div, co-mandamus, or generic-us-appellate"}}), &["path", "title"]),
        tool("case_seed", "Copy the fake Lucille Example reconsideration case into path. For tests only.", json!({"path": path}), &["path"]),
        tool("draft_import", "Import a court-markdown draft (YAML caption, ::: signature, ::: certificate).", json!({"path": path, "markdown": {"type": "string"}}), &["path", "markdown"]),
        tool("draft_sections", "List guided drafting sections and hints for this case kind.", json!({"path": path}), &["path"]),
        tool("draft_fill", "Fill one guided section with the user's own words. Do not invent text.", json!({"path": path, "id": {"type": "string", "description": "Section id from draft_sections"}, "text": {"type": "string"}}), &["path", "id", "text"]),
        tool("evidence_add", "Copy an exhibit into evidence/. First file becomes Ex. A after evidence_index.", json!({"path": path, "file": {"type": "string"}, "description": {"type": "string"}}), &["path", "file"]),
        tool("evidence_index", "Rebuild the exhibit index (Ex. A, Ex. B).", json!({"path": path}), &["path"]),
        tool("evidence_link", "Link a fact-check row to an exhibit letter. Does not invent a cite.", json!({"path": path, "fact_id": {"type": "integer"}, "letter": {"type": "string"}}), &["path", "fact_id", "letter"]),
        tool("check_fact", "Split the draft into sentences. Missing cites stay issues. Optional id/cite/exhibit to attach a user cite.", json!({"path": path, "id": {"type": "integer"}, "cite": {"type": "string"}, "exhibit": {"type": "string"}, "verified": {"type": "boolean"}}), &["path"]),
        tool("check_authority", "List cites already in the draft. Status starts unverified. Never invents a cite.", json!({"path": path, "id": {"type": "integer"}, "status": {"type": "string", "description": "verified|unverified|not_found"}, "source": {"type": "string"}}), &["path"]),
        tool("check_deadline", "Compute a deadline from a rule pack. Weekend/holiday roll. Does not file.", json!({"path": path, "pack": {"type": "string"}, "rule": {"type": "string"}, "trigger_date": {"type": "string", "description": "YYYY-MM-DD"}}), &["path", "pack", "rule", "trigger_date"]),
        tool("check_all", "Run fact, authority, deadline, and sign-off. ok=false until a human checks every box.", json!({"path": path, "pack": {"type": "string"}, "rule": {"type": "string"}, "trigger_date": {"type": "string"}}), &["path"]),
        tool("export_court", "Render Word and PDF. Never files or serves.", json!({"path": path, "kind": {"type": "string"}}), &["path"]),
        tool("export_packet", "Assemble notice of motion, motion, certification, proposed order, certificate of service.", json!({"path": path}), &["path"]),
        tool("export_ics", "Write a .ics reminder for a computed deadline.", json!({"path": path, "pack": {"type": "string"}, "rule": {"type": "string"}, "trigger_date": {"type": "string"}}), &["path", "pack", "rule", "trigger_date"]),
        tool("packs_list", "List bundled rule packs and installable form packs. Shows download sizes. Never files.", json!({"catalog": {"type": "string"}}), &[]),
        tool("packs_validate", "Validate a rule-pack JSON file. New states are data, not code.", json!({"file": {"type": "string"}}), &["file"]),
        tool("packs_install", "Install a forms pack for a state (optional situation). Uses a local file or catalog local_dir. Does not open a browser. Never files.", json!({"state": {"type": "string"}, "situation": {"type": "string"}, "file": {"type": "string"}, "catalog": {"type": "string"}}), &["state"]),
        tool("packs_remove", "Remove an installed forms pack.", json!({"state": {"type": "string"}}), &["state"]),
        tool("packs_verify", "Verify SHA-256 hashes of installed form files.", json!({"state": {"type": "string"}}), &[]),
        tool("guide_questions", "What do I need to file? Return the plain-language questions. Legal information, not advice.", json!({}), &[]),
        tool("guide_find", "Recommend forms or a packet from pack data. Never invents a rule. Includes help_links as data and never opens a browser.", json!({"state": {"type": "string"}, "court": {"type": "string"}, "situation": {"type": "string"}, "trigger_date": {"type": "string", "description": "YYYY-MM-DD"}}), &["state", "situation"]),
        tool("help_links", "JustLegal Get legal help / Find resources URLs as data. Never opens a browser. UI must confirm first.", json!({}), &[]),
        tool("gaps_add", "Capture a justice-gap note. Fake names only.", json!({"path": path, "title": {"type": "string"}, "court": {"type": "string"}, "rule": {"type": "string"}, "happened": {"type": "string"}, "hard": {"type": "string"}}), &["path", "title"]),
    ])
}

fn call_tool(name: &str, args: &Value) -> Result<Value, String> {
    let s = |k: &str| args.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
    match name {
        "case_create" => {
            let path = std::path::PathBuf::from(s("path"));
            let kind = if s("kind").is_empty() { "motion".into() } else { s("kind") };
            let (dir, mut profile) = case_profile::create_case_folder(
                &path,
                &s("title"),
                case_profile::CaseMode::Standard,
                &kind,
            )?;
            if !s("pack").is_empty() {
                profile.rule_pack_id = s("pack");
                profile.court.id = s("pack");
                case_profile::save_case(&dir, &profile)?;
            }
            Ok(json!({"id": profile.id, "folder": dir, "never_files": true}))
        }
        "draft_import" => {
            let doc = court_md::import_markdown(std::path::Path::new(&s("path")), &s("markdown"))?;
            Ok(json!({"lines": doc.lines.len(), "case_no": doc.meta.case_no, "never_files": true}))
        }
        "evidence_add" => {
            let file = s("file");
            let bytes = std::fs::read(&file).map_err(|e| e.to_string())?;
            let dest = check::add_exhibit(
                std::path::Path::new(&s("path")),
                std::path::Path::new(&file)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("exhibit.bin"),
                &bytes,
                &s("description"),
            )?;
            Ok(json!({"path": dest, "never_files": true}))
        }
        "check_fact" => {
            let draft = check::draft_text_for_case(std::path::Path::new(&s("path")))?;
            let report = check::run_fact_check(std::path::Path::new(&s("path")), &draft)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "check_authority" => {
            let draft = check::draft_text_for_case(std::path::Path::new(&s("path")))?;
            let report = check::run_authority_check(std::path::Path::new(&s("path")), &draft)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "check_deadline" => {
            let report = check::run_deadline(
                std::path::Path::new(&s("path")),
                &s("pack"),
                &s("rule"),
                &s("trigger_date"),
            )?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "check_all" => {
            let pack = s("pack");
            let rule = s("rule");
            let trigger = s("trigger_date");
            let report = check::run_all(
                std::path::Path::new(&s("path")),
                if pack.is_empty() { None } else { Some(pack.as_str()) },
                if rule.is_empty() { None } else { Some(rule.as_str()) },
                if trigger.is_empty() { None } else { Some(trigger.as_str()) },
            )?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "export_court" => {
            let path = std::path::PathBuf::from(s("path"));
            let profile = case_profile::load_case(&path)?;
            let kind = if s("kind").is_empty() { "court_md".into() } else { s("kind") };
            let exported = documents::export_pleading(&path, &profile, &kind, None)?;
            Ok(json!({"docx": exported.docx, "pdf": exported.pdf, "never_files": true}))
        }
        "playbook" => Ok(json!({"playbook": crate::help::playbook()})),
        "case_seed" => {
            let (dir, profile) = crate::seed::seed_lucille(std::path::Path::new(&s("path")))?;
            Ok(json!({"folder": dir, "id": profile.id, "title": profile.title, "never_files": true}))
        }
        "draft_sections" => {
            let profile = case_profile::load_case(std::path::Path::new(&s("path")))?;
            let draft = crate::guided::load_or_init(std::path::Path::new(&s("path")), &profile.document_kind)?;
            Ok(serde_json::to_value(draft).unwrap())
        }
        "draft_fill" => {
            let draft = crate::guided::set_section(std::path::Path::new(&s("path")), &s("id"), &s("text"))?;
            Ok(serde_json::to_value(draft).unwrap())
        }
        "evidence_index" => {
            Ok(serde_json::to_value(crate::exhibits::rebuild(std::path::Path::new(&s("path")))?).unwrap())
        }
        "evidence_link" => {
            let fact_id = args.get("fact_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            Ok(serde_json::to_value(crate::exhibits::link_fact(
                std::path::Path::new(&s("path")),
                fact_id,
                &s("letter"),
            )?)
            .unwrap())
        }
        "export_packet" => {
            Ok(serde_json::to_value(crate::packet::assemble_for_case(std::path::Path::new(&s("path")))?).unwrap())
        }
        "export_ics" => {
            let dir = std::path::PathBuf::from(s("path"));
            let result = check::run_deadline(&dir, &s("pack"), &s("rule"), &s("trigger_date"))?;
            let ics = crate::calendar::write_ics(&dir, &result)?;
            Ok(json!({"ics": ics, "due": result.due, "never_files": true}))
        }
        "packs_list" => {
            let catalog = if s("catalog").is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(s("catalog")))
            };
            Ok(serde_json::to_value(crate::forms::list_packs(catalog.as_deref(), None)?).unwrap())
        }
        "packs_validate" => {
            Ok(serde_json::to_value(RulePack::validate_file(std::path::Path::new(&s("file")))?).unwrap())
        }
        "packs_install" => {
            let file = if s("file").is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(s("file")))
            };
            let catalog = if s("catalog").is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(s("catalog")))
            };
            let situation = if s("situation").is_empty() {
                None
            } else {
                Some(s("situation"))
            };
            Ok(serde_json::to_value(crate::forms::install(crate::forms::InstallOpts {
                state: s("state"),
                situation,
                catalog,
                file,
                root: None,
                allow_network: false,
            })?)
            .unwrap())
        }
        "packs_remove" => Ok(json!({"removed": crate::forms::remove_pack(&s("state"), None)?})),
        "packs_verify" => {
            let state = if s("state").is_empty() {
                None
            } else {
                Some(s("state"))
            };
            Ok(serde_json::to_value(crate::forms::verify_pack(state.as_deref(), None)?).unwrap())
        }
        "guide_questions" => Ok(serde_json::to_value(crate::guide::questions()?).unwrap()),
        "guide_find" => {
            let court = if s("court").is_empty() {
                None
            } else {
                Some(s("court"))
            };
            let trigger = if s("trigger_date").is_empty() {
                None
            } else {
                Some(s("trigger_date"))
            };
            Ok(serde_json::to_value(crate::guide::find(&s("state"), court.as_deref(), &s("situation"), trigger.as_deref())?).unwrap())
        }
        "help_links" => Ok(serde_json::to_value(crate::help_links::HelpLinks::load()?).unwrap()),
        "gaps_add" => Ok(serde_json::to_value(crate::gaps::add(
            std::path::Path::new(&s("path")),
            &s("title"),
            &s("court"),
            &s("rule"),
            &s("happened"),
            &s("hard"),
        )?)
        .unwrap()),
        other => Err(format!(
            "Unknown tool '{other}'. Call playbook or packs_list. Known tools are listed by tools/list."
        )),
    }
}

fn write_message(out: &mut impl Write, body: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec(body).map_err(|e| e.to_string())?;
    write!(out, "Content-Length: {}\r\n\r\n", bytes.len()).map_err(|e| e.to_string())?;
    out.write_all(&bytes).map_err(|e| e.to_string())?;
    out.flush().map_err(|e| e.to_string())
}

fn read_message(reader: &mut impl BufRead) -> Result<Option<Value>, String> {
    let mut headers = String::new();
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
    serde_json::from_slice(&buf).map(Some).map_err(|e| e.to_string())
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

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("os-mcp-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn tools_list_names_the_agent_surface() {
        let reply = handle(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }))
        .unwrap();
        let names: Vec<&str> = reply["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        for required in [
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
            assert!(names.contains(&required), "missing {required} in {names:?}");
        }
    }

    #[test]
    fn content_length_roundtrip() {
        let mut buf = Vec::new();
        write_message(&mut buf, &json!({"hello": "world"})).unwrap();
        let mut reader = BufReader::new(Cursor::new(buf));
        let msg = read_message(&mut reader).unwrap().unwrap();
        assert_eq!(msg["hello"], "world");
    }

    #[test]
    fn mcp_tools_create_import_check_export() {
        let dir = temp_dir();
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
        assert_eq!(created["never_files"], true);
        let md = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("fixtures")
                .join("sample-reconsideration.md"),
        )
        .unwrap_or_else(|_| {
            "---\ncourt: Sample\nplaintiff: Jordan Example\ndefendant: Sample County Clerk\ncase_no: A-01000-24\n---\nRelief is sought under R. 2:11-6.\n".into()
        });
        call_tool(
            "draft_import",
            &json!({"path": dir, "markdown": md}),
        )
        .unwrap();
        let exhibit = dir.join("order-excerpt.txt");
        std::fs::write(&exhibit, b"Order granting dismissal (fake exhibit)\n").unwrap();
        call_tool(
            "evidence_add",
            &json!({"path": dir, "file": exhibit, "description": "Dkt. 12"}),
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
        let exported = call_tool("export_court", &json!({"path": dir, "kind": "court_md"})).unwrap();
        let docx = exported["docx"].as_str().unwrap();
        let pdf = exported["pdf"].as_str().unwrap();
        assert!(std::path::Path::new(docx).is_file());
        assert!(std::path::Path::new(pdf).is_file());
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
        assert!(guide["help_links"].as_array().unwrap().iter().any(|l| l["id"] == "find-legal-help"));
        let links = call_tool("help_links", &json!({})).unwrap();
        assert_eq!(links["opens_browser"], false);
        let _ = std::fs::remove_dir_all(dir);
    }
}

pub fn serve_stdio() -> Result<(), String> {
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = std::io::stdout();
    eprintln!("openseason mcp: offline stdio server. Never files or serves.");
    loop {
        match read_message(&mut reader)? {
            None => break,
            Some(msg) => {
                if let Some(reply) = handle(&msg) {
                    write_message(&mut stdout, &reply)?;
                }
            }
        }
    }
    Ok(())
}
