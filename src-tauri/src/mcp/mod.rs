//! Minimal MCP stdio server. Same engine as the CLI. Offline. Never files.

pub mod handlers;
pub mod protocol;
pub mod tools;

use std::io::BufReader;

use protocol::{handle, read_message, write_message, Framing};

pub fn serve_stdio() -> Result<(), String> {
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = std::io::stdout();
    let mut framing = None;
    eprintln!("openseason mcp: offline stdio server. Never files or serves.");
    loop {
        match read_message(&mut reader, &mut framing)? {
            None => break,
            Some(msg) => {
                if let Some(reply) = handle(&msg) {
                    write_message(&mut stdout, &reply, framing.unwrap_or(Framing::ContentLength))?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Cursor};
    use serde_json::{json, Value};
    use super::handlers::call_tool;
    use super::protocol::{handle, read_message, write_message, Framing};

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
            "record_inventory",
            "accountability_audit",
            "ada_petition",
            "court_access_directory",
            "record_retrieval_plan",
        ] {
            assert!(names.contains(&required), "missing {required} in {names:?}");
        }
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
        write_message(&mut buf, &json!({"jsonrpc": "2.0", "id": 1, "method": "ping"}), Framing::Newline).unwrap();
        let mut reader = BufReader::new(Cursor::new(buf));
        let mut framing = None;
        let msg = read_message(&mut reader, &mut framing).unwrap().unwrap();
        assert_eq!(msg["method"], "ping");
        assert_eq!(framing, Some(Framing::Newline));
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
