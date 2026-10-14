//! Minimal MCP stdio server. Same engine as the CLI. Offline. Never files.

use std::io::{BufRead, BufReader, Write};

use serde_json::{json, Value};

use crate::agent;
use crate::case_profile;
use crate::check;
use crate::court_md;
use crate::documents;
use crate::rule_pack::RulePack;

fn tool_list() -> Value {
    json!([
        {"name": "case_create", "description": "Create a local case folder. Never files.", "inputSchema": {"type": "object", "properties": {
            "path": {"type": "string"}, "title": {"type": "string"}, "kind": {"type": "string"}, "pack": {"type": "string"}
        }, "required": ["path", "title"]}},
        {"name": "draft_import", "description": "Import a court-markdown draft into the case.", "inputSchema": {"type": "object", "properties": {
            "path": {"type": "string"}, "markdown": {"type": "string"}
        }, "required": ["path", "markdown"]}},
        {"name": "evidence_add", "description": "Copy an exhibit into the case evidence folder.", "inputSchema": {"type": "object", "properties": {
            "path": {"type": "string"}, "file": {"type": "string"}, "description": {"type": "string"}
        }, "required": ["path", "file"]}},
        {"name": "check_fact", "description": "Record fact-check. Does not invent cites.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}, "required": ["path"]}},
        {"name": "check_authority", "description": "List cites already in the draft. Status starts unverified.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}, "required": ["path"]}},
        {"name": "check_deadline", "description": "Compute a deadline from a rule pack.", "inputSchema": {"type": "object", "properties": {
            "path": {"type": "string"}, "pack": {"type": "string"}, "rule": {"type": "string"}, "trigger_date": {"type": "string"}
        }, "required": ["path", "pack", "rule", "trigger_date"]}},
        {"name": "check_all", "description": "Run fact, authority, deadline, and sign-off.", "inputSchema": {"type": "object", "properties": {
            "path": {"type": "string"}, "pack": {"type": "string"}, "rule": {"type": "string"}, "trigger_date": {"type": "string"}
        }, "required": ["path"]}},
        {"name": "export_court", "description": "Render Word and PDF. Never files or serves.", "inputSchema": {"type": "object", "properties": {
            "path": {"type": "string"}, "kind": {"type": "string"}
        }, "required": ["path"]}},
        {"name": "packs_list", "description": "List bundled rule packs.", "inputSchema": {"type": "object", "properties": {}}}
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
        "packs_list" => Ok(json!(RulePack::list_builtin()
            .iter()
            .map(|p| json!({"id": p.id, "name": p.name, "jurisdiction": p.jurisdiction}))
            .collect::<Vec<_>>())),
        other => Err(format!("Unknown tool '{other}'")),
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
