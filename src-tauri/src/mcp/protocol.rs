use std::io::{BufRead, Write};
use serde_json::{json, Value};

use super::handlers::call_tool;
use super::tools::tool_list;
use crate::agent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framing {
    ContentLength,
    Newline,
}

pub fn write_message(out: &mut impl Write, body: &Value, framing: Framing) -> Result<(), String> {
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

pub fn read_message(reader: &mut impl BufRead, framing: &mut Option<Framing>) -> Result<Option<Value>, String> {
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
        return serde_json::from_slice(&buf).map(Some).map_err(|e| e.to_string());
    }
    if trimmed.starts_with('{') {
        *framing = Some(Framing::Newline);
        return serde_json::from_str(trimmed).map(Some).map_err(|e| e.to_string());
    }
    Err(format!("Unrecognized MCP frame: {}", trimmed.chars().take(80).collect::<String>()))
}

pub fn handle(msg: &Value) -> Option<Value> {
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
