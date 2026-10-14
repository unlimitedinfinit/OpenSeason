use serde::Serialize;

use crate::case_profile::{REVIEW_NOTICE, ValidationIssue};
use crate::check::NEVER_FILE_NOTICE;

#[derive(Debug, Clone, Serialize)]
pub struct AgentEnvelope<T: Serialize> {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub notice: String,
    pub issues: Vec<ValidationIssue>,
    pub data: T,
}

impl<T: Serialize> AgentEnvelope<T> {
    pub fn ok(data: T) -> Self {
        Self {
            ok: true,
            never_files: true,
            never_serves: true,
            notice: format!("{NEVER_FILE_NOTICE}\n{REVIEW_NOTICE}"),
            issues: Vec::new(),
            data,
        }
    }

    pub fn with_issues(data: T, issues: Vec<ValidationIssue>) -> Self {
        Self {
            ok: issues.is_empty(),
            never_files: true,
            never_serves: true,
            notice: format!("{NEVER_FILE_NOTICE}\n{REVIEW_NOTICE}"),
            issues,
            data,
        }
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

/// Wrap an engine JSON value in the stable agent envelope.
/// If `value` already has an `issues` array, those become envelope issues.
pub fn wrap_value(value: serde_json::Value) -> serde_json::Value {
    let issues: Vec<ValidationIssue> = value
        .get("issues")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    serde_json::to_value(AgentEnvelope::with_issues(value, issues)).unwrap_or_else(|_| {
        serde_json::json!({
            "ok": false,
            "never_files": true,
            "never_serves": true,
            "notice": NEVER_FILE_NOTICE,
            "issues": [],
            "data": null
        })
    })
}

pub fn wrap_error(message: String) -> serde_json::Value {
    serde_json::to_value(AgentEnvelope::with_issues(
        serde_json::Value::Null,
        vec![ValidationIssue {
            field: "mcp".into(),
            message,
        }],
    ))
    .unwrap_or_else(|_| serde_json::json!({"ok": false, "never_files": true, "never_serves": true}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_always_declares_it_never_files() {
        let env = AgentEnvelope::with_issues(
            serde_json::json!({"due": "2026-01-20"}),
            vec![ValidationIssue {
                field: "fact.1".into(),
                message: "Factual sentence has no record cite.".into(),
            }],
        );
        assert!(!env.ok);
        assert!(env.never_files && env.never_serves);
        let text = env.to_json().unwrap();
        assert!(text.contains("\"never_files\": true"));
        assert!(text.contains("\"never_serves\": true"));
        assert!(text.contains("fact.1"));
    }

    #[test]
    fn wrap_value_lifts_nested_issues() {
        let wrapped = wrap_value(serde_json::json!({
            "ok": false,
            "issues": [{"field": "authority.1", "message": "unverified"}]
        }));
        assert_eq!(wrapped["ok"], false);
        assert_eq!(wrapped["never_files"], true);
        assert_eq!(wrapped["issues"][0]["field"], "authority.1");
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatedCase {
    pub id: String,
    pub folder: String,
    pub mode: String,
    pub document_kind: String,
    pub rule_pack_id: String,
}
