use super::certificate::require_complete_certificate;
use super::assemble::document_lines;
use super::types::{BodyBlock, CourtDocument};

#[derive(Debug, Clone)]
pub struct VerificationLeak {
    pub block: &'static str,
    pub line_number: usize,
    pub snippet: String,
}

pub fn body_without_front_matter(text: &str) -> &str {
    let trimmed = text.trim_start_matches('\u{feff}');
    if !trimmed.starts_with("---") {
        return trimmed;
    }
    let rest = trimmed[3..].trim_start_matches(|c| c == '\r' || c == '\n');
    if let Some(end) = rest.find("\n---") {
        rest[end + 4..].trim_start_matches(|c| c == '\r' || c == '\n')
    } else {
        trimmed
    }
}

pub fn find_verification_leaks(user_text: &str) -> Vec<VerificationLeak> {
    let body = body_without_front_matter(user_text);
    let mut leaks = Vec::new();
    let mut block = "body";
    for (i, raw) in body.replace('\r', "").lines().enumerate() {
        let s = raw.trim();
        if s.starts_with(":::") {
            let name = s.trim_start_matches(':').trim();
            block = if name.is_empty() {
                "body"
            } else if name.starts_with("signature") {
                "signature"
            } else if name.starts_with("certificate") {
                "certificate"
            } else if name.starts_with("declaration") || name.starts_with("certification") {
                "declaration"
            } else {
                "body"
            };
            continue;
        }
        if s.to_ascii_lowercase().contains("needs_verification") {
            let snippet = if s.chars().count() > 80 {
                let cut: String = s.chars().take(77).collect();
                format!("{cut}…")
            } else {
                s.to_string()
            };
            leaks.push(VerificationLeak {
                block,
                line_number: i + 1,
                snippet,
            });
        }
    }
    leaks
}

pub fn format_verification_leaks(leaks: &[VerificationLeak]) -> String {
    leaks
        .iter()
        .map(|l| format!("{} line {}: \"{}\"", l.block, l.line_number, l.snippet))
        .collect::<Vec<_>>()
        .join("; ")
}

pub fn require_no_verification_leak(user_text: &str) -> Result<(), String> {
    let leaks = find_verification_leaks(user_text);
    if leaks.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Cannot export: user-authored text contains 'needs_verification' ({}). Keep that flag in template YAML metadata, not in the paper. Export will not rewrite your sentences.",
            format_verification_leaks(&leaks)
        ))
    }
}

pub fn require_exportable(doc: &CourtDocument, user_text: &str) -> Result<(), String> {
    require_no_verification_leak(user_text)?;
    let assembled = document_lines(doc).join("\n");
    if assembled.to_ascii_lowercase().contains("needs_verification") {
        return Err(
            "Cannot export: assembled paper contains 'needs_verification'. Keep that flag in template YAML metadata, not in the paper. Export will not rewrite your sentences."
                .into(),
        );
    }
    require_complete_certificate(doc)
}

pub fn contains_forbidden_markup(text: &str) -> bool {
    text.contains(":::") || text.contains("#v(") || text.lines().any(|l| {
        let t = l.trim();
        t.starts_with('#') && !t.starts_with("#set") // leftover markdown heading
            && !t.starts_with("#align")
            && t.chars().nth(1).map(|c| c == ' ' || c.is_ascii_alphabetic()).unwrap_or(false)
    })
}

/// Strip markdown, fence blocks, and leaked Typst so they never print in a filing.
/// Returns body, signature lines, certificate-of-service lines, and an optional
/// user-supplied declaration / certification (not auto-asserted for Colorado).
pub fn sanitize_draft(text: &str) -> (Vec<BodyBlock>, Vec<String>, Vec<String>, Vec<String>) {
    let mut body = Vec::new();
    let mut signature = Vec::new();
    let mut certificate = Vec::new();
    let mut declaration = Vec::new();
    let mut mode = "body";
    for raw in text.replace('\r', "").lines() {
        let s = raw.trim();
        if s.contains("#v(")
            || s.contains("#h(")
            || s.starts_with("#set ")
            || s.starts_with("#align")
            || s.starts_with("#grid")
            || s.starts_with("#box")
            || s.starts_with("#table")
            || s.starts_with("#pagebreak")
        {
            continue;
        }
        if s.starts_with(":::") {
            let name = s.trim_start_matches(':').trim();
            if name.is_empty() {
                mode = "body";
            } else if name.starts_with("signature") {
                mode = "signature";
            } else if name.starts_with("certificate") {
                mode = "certificate";
            } else if name.starts_with("declaration") || name.starts_with("certification") {
                mode = "declaration";
            } else {
                mode = "skip";
            }
            continue;
        }
        let cleaned = s.replace("**", "").replace("__", "");
        let cleaned = cleaned.trim().to_string();
        match mode {
            "signature" => {
                if !cleaned.is_empty() {
                    signature.push(cleaned);
                }
            }
            "certificate" => {
                if !cleaned.is_empty() {
                    certificate.push(cleaned);
                }
            }
            "declaration" => {
                if !cleaned.is_empty() {
                    declaration.push(cleaned);
                }
            }
            "skip" => {}
            _ => {
                if cleaned.is_empty() {
                    if !matches!(body.last(), Some(BodyBlock::Blank)) {
                        body.push(BodyBlock::Blank);
                    }
                } else if let Some(rest) = cleaned.strip_prefix("### ") {
                    body.push(BodyBlock::Heading(rest.trim().to_string()));
                } else if let Some(rest) = cleaned.strip_prefix("## ") {
                    body.push(BodyBlock::Heading(rest.trim().to_string()));
                } else if let Some(rest) = cleaned.strip_prefix("# ") {
                    body.push(BodyBlock::Heading(rest.trim().to_string()));
                } else if cleaned.starts_with('#') && cleaned.len() > 1 && cleaned.as_bytes()[1].is_ascii_alphabetic()
                {
                    body.push(BodyBlock::Heading(cleaned.trim_start_matches('#').trim().to_string()));
                } else {
                    body.push(BodyBlock::Paragraph(cleaned));
                }
            }
        }
    }
    (body, signature, certificate, declaration)
}
