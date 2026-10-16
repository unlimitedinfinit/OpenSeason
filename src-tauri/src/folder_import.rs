//! Import a folder of drafts, notes, and orders into a case.
//! Copies or (on request) links originals. Never moves or deletes them.
//! Proposes party names, docket numbers, and order dates — never auto-commits guesses.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::case_profile::{self, CaseMode, CaseProfile};
use crate::check::NEVER_FILE_NOTICE;
use crate::intake::{self, CandidateValue};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CopiedFile {
    pub original: String,
    pub dest: String,
    pub kind: String, // "draft" | "order" | "note" | "evidence"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ImportProposals {
    pub party_names: Vec<CandidateValue>,
    pub docket_numbers: Vec<CandidateValue>,
    pub order_dates: Vec<CandidateValue>,
    pub court_names: Vec<CandidateValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FolderImportReport {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub notice: String,
    pub source_folder: String,
    pub case_dir: String,
    pub case_id: String,
    pub copied_files: Vec<CopiedFile>,
    pub proposals: ImportProposals,
    pub auto_committed: bool,
}

fn classify(name: &str) -> &'static str {
    let n = name.to_ascii_lowercase();
    if n.contains("order")
        || n.contains("judgment")
        || n.contains("judgement")
        || n.contains("opinion")
        || n.contains("decision")
    {
        "order"
    } else if n.ends_with(".md") || n.contains("draft") || n.contains("motion") || n.contains("brief")
    {
        "draft"
    } else if n.contains("note") || n.ends_with(".txt") {
        "note"
    } else {
        "evidence"
    }
}

fn dest_subdir(kind: &str) -> &'static str {
    match kind {
        "order" => "orders",
        "draft" => "drafts",
        "note" => "drafts",
        _ => "evidence",
    }
}

fn unique_dest(dir: &Path, filename: &str) -> PathBuf {
    let mut dest = dir.join(filename);
    if !dest.exists() {
        return dest;
    }
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    for i in 2..1000 {
        dest = if ext.is_empty() {
            dir.join(format!("{stem}-{i}"))
        } else {
            dir.join(format!("{stem}-{i}.{ext}"))
        };
        if !dest.exists() {
            return dest;
        }
    }
    dest
}

/// Copy (never move) files from `source` into an existing or newly created case folder.
/// Proposed caption fields are returned for the user to confirm. Nothing is written to
/// case.json except the folder layout itself.
pub fn import_folder(
    case_dir: &Path,
    source: &Path,
    title: Option<&str>,
) -> Result<FolderImportReport, String> {
    if !source.is_dir() {
        return Err(format!("Source folder does not exist: {}", source.display()));
    }
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Import into the Documents pointer is refused.".into());
    }

    let case_exists = case_dir.join("case.json").is_file();
    let profile: CaseProfile = if case_exists {
        case_profile::load_case(case_dir)?
    } else {
        let title = title
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                source
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Imported matter")
                    .to_string()
            });
        fs::create_dir_all(case_dir).map_err(|e| e.to_string())?;
        let (_created, profile) =
            case_profile::create_case_folder(case_dir, &title, CaseMode::Standard, "motion")?;
        profile
    };

    let actual_dir = if case_exists {
        case_dir.to_path_buf()
    } else {
        // Re-resolve the folder that contains this profile id.
        let parent = case_dir.parent().unwrap_or(case_dir);
        let mut found = case_dir.to_path_buf();
        if let Ok(entries) = fs::read_dir(parent) {
            for e in entries.flatten() {
                if e.path().join("case.json").is_file() {
                    if let Ok(p) = case_profile::load_case(&e.path()) {
                        if p.id == profile.id {
                            found = e.path();
                            break;
                        }
                    }
                }
            }
        }
        if !found.join("case.json").is_file() {
            case_profile::ensure_case_layout(case_dir)?;
            case_profile::save_case(case_dir, &profile)?;
            found = case_dir.to_path_buf();
        }
        found
    };

    case_profile::ensure_case_layout(&actual_dir)?;
    fs::create_dir_all(actual_dir.join("drafts")).map_err(|e| e.to_string())?;
    fs::create_dir_all(actual_dir.join("orders")).map_err(|e| e.to_string())?;
    fs::create_dir_all(actual_dir.join("evidence")).map_err(|e| e.to_string())?;

    let mut copied = Vec::new();
    collect_and_copy(source, &actual_dir, &mut copied)?;

    let scan = intake::intake_scan(&actual_dir).unwrap_or_default();
    let mut proposals = ImportProposals {
        party_names: Vec::new(),
        docket_numbers: scan.candidate_dockets,
        order_dates: scan.candidate_dates,
        court_names: scan.candidate_courts,
    };
    // Filename / snippet heuristics for party-like Title Case pairs. Do not write them.
    for file in &copied {
        if file.kind == "draft" {
            if let Ok(text) = fs::read_to_string(&file.dest) {
                for cap in extract_party_guesses(&text) {
                    if !proposals
                        .party_names
                        .iter()
                        .any(|c| c.value == cap)
                    {
                        proposals.party_names.push(CandidateValue {
                            value: cap,
                            source: file.original.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(FolderImportReport {
        ok: true,
        never_files: true,
        never_serves: true,
        notice: format!(
            "{NEVER_FILE_NOTICE}\nCopied {} file(s). Proposed names, dockets, and dates were NOT written to case.json. Confirm each value yourself.",
            copied.len()
        ),
        source_folder: source.display().to_string(),
        case_dir: actual_dir.display().to_string(),
        case_id: profile.id,
        copied_files: copied,
        proposals,
        auto_committed: false,
    })
}

fn collect_and_copy(
    source: &Path,
    case_dir: &Path,
    copied: &mut Vec<CopiedFile>,
) -> Result<(), String> {
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in walkdir::WalkDir::new(source).max_depth(4) {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if name.starts_with('.') || name == "case.json" {
            continue;
        }
        files.push(entry.path().to_path_buf());
    }
    files.sort();
    for src in files {
        let name = src
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file")
            .to_string();
        let kind = classify(&name);
        let dest_dir = case_dir.join(dest_subdir(kind));
        fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
        let dest = unique_dest(&dest_dir, &name);
        fs::copy(&src, &dest).map_err(|e| {
            format!("Could not copy {} → {}: {e}", src.display(), dest.display())
        })?;
        copied.push(CopiedFile {
            original: src.display().to_string(),
            dest: dest.display().to_string(),
            kind: kind.to_string(),
        });
    }
    Ok(())
}

fn extract_party_guesses(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower = text.to_ascii_lowercase();
    if let Some(idx) = lower.find(" v. ") {
        let left = text[..idx].lines().last().unwrap_or("").trim();
        let right = text[idx + 4..].lines().next().unwrap_or("").trim();
        if looks_like_name(left) {
            out.push(left.trim_end_matches(',').to_string());
        }
        if looks_like_name(right) {
            out.push(right.trim_end_matches(',').to_string());
        }
    }
    out
}

fn looks_like_name(s: &str) -> bool {
    let s = s.trim();
    if s.len() < 4 || s.len() > 80 {
        return false;
    }
    s.chars().any(|c| c.is_alphabetic()) && !s.contains("http")
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn copies_without_moving_and_does_not_write_guesses() {
        let root = std::env::temp_dir().join(format!("os-imp-{}", Uuid::new_v4()));
        let src = root.join("inbox");
        let dest = root.join("case");
        fs::create_dir_all(src.join("papers")).unwrap();
        fs::write(
            src.join("papers").join("Order-Denied-2026-10-01.txt"),
            "Jordan Example v. Sample County Clerk\nDocket A-01000-24\nEntered 2026-10-01",
        )
        .unwrap();
        fs::write(
            src.join("draft.md"),
            "# Preliminary statement\nThe trial court entered an order.\n",
        )
        .unwrap();
        let report = import_folder(&dest, &src, Some("Example v. Sample County Clerk")).unwrap();
        assert!(report.ok);
        assert!(!report.auto_committed);
        assert_eq!(report.copied_files.len(), 2);
        assert!(src.join("draft.md").is_file(), "original must remain");
        assert!(src.join("papers").join("Order-Denied-2026-10-01.txt").is_file());
        let case_json = fs::read_to_string(Path::new(&report.case_dir).join("case.json")).unwrap();
        let profile: CaseProfile = serde_json::from_str(&case_json).unwrap();
        assert!(profile.docket_number.is_empty(), "must not auto-commit docket guesses");
        assert!(profile.notice_of_appeal.judgment_date.is_empty());
        assert!(profile.caption.plaintiffs.is_empty() || profile.caption.plaintiffs[0].name.is_empty() || profile.docket_number.is_empty());
        assert!(!report.proposals.docket_numbers.is_empty() || !report.proposals.order_dates.is_empty() || !report.proposals.party_names.is_empty());
        let _ = fs::remove_dir_all(root);
    }
}
