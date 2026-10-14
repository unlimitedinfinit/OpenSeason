//! Auto-numbered exhibit index. Facts link to Ex. A, Ex. B, never invented cites.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExhibitRow {
    pub letter: String,
    pub filename: String,
    pub description: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExhibitIndex {
    pub never_files: bool,
    pub items: Vec<ExhibitRow>,
}

pub fn path(case_dir: &Path) -> std::path::PathBuf {
    case_dir.join("evidence").join("index.json")
}

fn letter_at(i: usize) -> String {
    if i < 26 {
        ((b'A' + i as u8) as char).to_string()
    } else {
        format!("{}", i + 1)
    }
}

pub fn rebuild(case_dir: &Path) -> Result<ExhibitIndex, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. The exhibit index cannot be rebuilt on the Documents pointer.".into());
    }
    let evidence = case_dir.join("evidence");
    fs::create_dir_all(&evidence).map_err(|e| e.to_string())?;
    let saved = crate::check::read_json::<ExhibitIndex>(&path(case_dir))?
        .map(|i| i.items)
        .unwrap_or_default();
    let mut items = Vec::new();
    if evidence.is_dir() {
        let mut files: Vec<_> = fs::read_dir(&evidence)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                if !p.is_file() {
                    return false;
                }
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name == "index.json" {
                    return false;
                }
                if let Some(stem) = name.strip_suffix(".txt") {
                    if evidence.join(stem).is_file() {
                        return false;
                    }
                }
                true
            })
            .collect();
        files.sort();
        for (i, file) in files.into_iter().enumerate() {
            let name = file
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("exhibit")
                .to_string();
            let prior = saved.iter().find(|r| r.filename == name);
            let desc_file = evidence.join(format!("{name}.txt"));
            let description = if desc_file.is_file() {
                fs::read_to_string(desc_file).unwrap_or_default()
            } else {
                prior.map(|p| p.description.clone()).unwrap_or_default()
            };
            items.push(ExhibitRow {
                letter: prior
                    .map(|p| p.letter.clone())
                    .unwrap_or_else(|| letter_at(i)),
                filename: name,
                description: description.trim().to_string(),
                path: file.display().to_string(),
            });
        }
    }
    let index = ExhibitIndex {
        never_files: true,
        items,
    };
    crate::check::write_json(&path(case_dir), &index)?;
    Ok(index)
}

/// Attach a fact row to an exhibit letter. Does not invent a cite.
pub fn link_fact(case_dir: &Path, fact_id: u32, letter: &str) -> Result<crate::check::FactCheckReport, String> {
    let index = rebuild(case_dir)?;
    let row = index
        .items
        .iter()
        .find(|r| r.letter.eq_ignore_ascii_case(letter.trim()))
        .ok_or_else(|| {
            format!(
                "No exhibit '{}'. Rebuild the index after adding a file. Known letters: {}.",
                letter,
                index.items.iter().map(|r| r.letter.as_str()).collect::<Vec<_>>().join(", ")
            )
        })?;
    let cite = format!("Ex. {}", row.letter);
    crate::check::set_fact_cite(case_dir, fact_id, &cite, &row.filename, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_profile::{ensure_case_layout, sample_appeal_profile, save_case};
    use crate::check::{add_exhibit, run_fact_check};
    use uuid::Uuid;

    #[test]
    fn first_exhibit_is_letter_a() {
        let dir = std::env::temp_dir().join(format!("os-ex-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        ensure_case_layout(&dir).unwrap();
        save_case(&dir, &sample_appeal_profile()).unwrap();
        add_exhibit(&dir, "order.txt", b"fake order", "Dkt. 12").unwrap();
        let index = rebuild(&dir).unwrap();
        assert_eq!(index.items[0].letter, "A");
        run_fact_check(&dir, "The trial court entered an order dismissing the complaint.").unwrap();
        let report = link_fact(&dir, 1, "A").unwrap();
        assert_eq!(report.rows[0].record_cite, "Ex. A");
        let _ = fs::remove_dir_all(dir);
    }
}
