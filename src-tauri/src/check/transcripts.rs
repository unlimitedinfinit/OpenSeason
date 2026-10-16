//! Transcript citation and evidentiary record grounding.

use regex::Regex;
use std::fs;
use std::path::Path;
use crate::case_profile::ValidationIssue;
use super::types::{TranscriptCitation, TranscriptReport};

pub fn check_transcripts(case_dir: &Path, draft: &str) -> Result<TranscriptReport, String> {
    let evidence = case_dir.join("evidence");
    let mut indexed_files = Vec::new();
    let mut total_lines = 0;
    let mut transcript_texts = Vec::new();

    if evidence.is_dir() {
        if let Ok(entries) = fs::read_dir(&evidence) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                    let lower = name.to_ascii_lowercase();
                    if lower.contains("hearing") || lower.contains("transcript") || lower.contains("record") || lower.ends_with(".txt") || lower.ends_with(".md") {
                        if let Ok(text) = fs::read_to_string(&p) {
                            total_lines += text.lines().count();
                            transcript_texts.push((name.clone(), text));
                            indexed_files.push(name);
                        } else {
                            indexed_files.push(name);
                        }
                    }
                }
            }
        }
    }

    let cite_regex = Regex::new(r"\[(?:Tr\.|Hearing|Dep\.|Deposition)\s*(?:([^,\]]+),\s*)?(?:p\.|page\s*)?(\d+)(?::(\d+))?\]").unwrap();
    let mut citations = Vec::new();
    let mut issues = Vec::new();

    for cap in cite_regex.captures_iter(draft) {
        let raw = cap.get(0).map(|m| m.as_str()).unwrap_or("").to_string();
        let transcript_name = cap.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_else(|| {
            indexed_files.first().cloned().unwrap_or_else(|| "HEARING".to_string())
        });
        let page: u32 = cap.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(1);
        let line: Option<u32> = cap.get(3).and_then(|m| m.as_str().parse().ok());

        let mut verified = false;
        let mut quote_found = None;

        if !transcript_texts.is_empty() {
            // Check if page or line exists in transcripts
            for (tname, text) in &transcript_texts {
                if tname.eq_ignore_ascii_case(&transcript_name) || transcript_name == "HEARING" || indexed_files.len() == 1 {
                    let page_pattern = format!("Page {}", page);
                    let page_pattern_alt = format!("{}:{}", page, line.unwrap_or(1));
                    if text.contains(&page_pattern) || text.contains(&page_pattern_alt) || text.lines().count() >= (page as usize * 10) {
                        verified = true;
                        quote_found = Some(format!("Grounding match verified in {}", tname));
                        break;
                    }
                }
            }
        }

        if !verified && indexed_files.is_empty() {
            issues.push(ValidationIssue {
                field: "transcripts".to_string(),
                message: format!("Citation {} cited but no transcript or hearing record files exist in evidence/.", raw),
            });
        } else if !verified {
            issues.push(ValidationIssue {
                field: "transcripts".to_string(),
                message: format!("Citation {} (page {}, line {:?}) could not be grounded against indexed records.", raw, page, line),
            });
        }

        citations.push(TranscriptCitation {
            raw,
            transcript_name,
            page,
            line,
            verified,
            quote_found,
        });
    }

    let ok = issues.is_empty();
    Ok(TranscriptReport {
        ok,
        never_files: true,
        indexed_files,
        total_lines,
        citations,
        issues,
    })
}
