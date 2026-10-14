//! Filing packet assembler. Formats papers only. Never files or serves.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::case_profile::{self, CaseProfile, REVIEW_NOTICE};
use crate::check::NEVER_FILE_NOTICE;
use crate::documents::{self, ExportPaths};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PacketItem {
    pub id: String,
    pub label: String,
    pub docx: String,
    pub pdf: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PacketReport {
    pub never_files: bool,
    pub never_serves: bool,
    pub notice: String,
    pub items: Vec<PacketItem>,
    pub checklist: Vec<String>,
}

fn part_lines(profile: &CaseProfile, id: &str) -> (String, Vec<String>) {
    let caption = documents::assembled_pleading_paragraphs(profile, "reconsideration");
    // Reuse caption header (court through case number) when present.
    let header: Vec<String> = caption
        .iter()
        .take_while(|l| !l.eq_ignore_ascii_case("MOTION") && !l.contains("basic template"))
        .cloned()
        .collect();
    let mut lines = header;
    if lines.last().map(|l| !l.is_empty()).unwrap_or(false) {
        lines.push(String::new());
    }
    let (heading, body): (&str, Vec<&str>) = match id {
        "notice" => (
            "NOTICE OF MOTION",
            vec![
                "Please take notice that the undersigned will move for the relief in the accompanying motion.",
                "This software did not notice, file, or serve anyone.",
            ],
        ),
        "certification" => (
            "CERTIFICATION",
            vec![
                "I certify that the facts in the accompanying motion are true to the best of my knowledge.",
                "I certify that this paper is the user's own draft. OpenSeason did not invent facts.",
                "I understand this app did not file or serve this paper.",
            ],
        ),
        "order" => (
            "PROPOSED ORDER",
            vec![
                "It is ORDERED that the relief requested in the accompanying motion is granted / denied.",
                "The court fills the blank. This app does not decide the motion.",
            ],
        ),
        "certificate" => (
            "CERTIFICATE OF SERVICE",
            vec![
                "I certify that on the date I actually serve this packet, I will serve a copy on the parties listed in the caption by a method the court rules allow.",
                "This software did not serve anyone.",
            ],
        ),
        _ => (
            "MOTION",
            vec![],
        ),
    };
    lines.push(heading.to_string());
    lines.push(String::new());
    if id == "motion" {
        return ("MOTION".into(), documents::assembled_pleading_paragraphs(profile, &profile.document_kind));
    }
    for para in body {
        lines.push(para.to_string());
        lines.push(String::new());
    }
    lines.push(format!("/s/ {}", profile.filer.signature_name));
    lines.push(profile.filer.name.clone());
    lines.push(REVIEW_NOTICE.to_string());
    (heading.to_string(), lines)
}

pub fn assemble(case_dir: &Path, profile: &CaseProfile) -> Result<PacketReport, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Packet export from the Documents pointer is refused.".into());
    }
    let dest = case_dir.join("exports").join("packet");
    std::fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    let parts = [
        ("notice", "Notice of motion"),
        ("motion", "Motion"),
        ("certification", "Certification"),
        ("order", "Proposed order"),
        ("certificate", "Certificate of service"),
    ];
    let mut items = Vec::new();
    for (id, label) in parts {
        let (heading, lines) = part_lines(profile, id);
        let exported: ExportPaths =
            documents::write_lines_as_docx_pdf(case_dir, &format!("packet-{id}"), &heading, &lines, Some(&dest))?;
        items.push(PacketItem {
            id: id.into(),
            label: label.into(),
            docx: exported.docx.display().to_string(),
            pdf: exported.pdf.display().to_string(),
        });
    }
    let checklist = vec![
        "Read every page of every PDF.".into(),
        "Every fact has an exhibit letter you checked.".into(),
        "Every citation is verified on a live source or removed.".into(),
        "Caption, signature, and certificate of service are complete.".into(),
        "You will file and serve this yourself. The app did not.".into(),
    ];
    let report = PacketReport {
        never_files: true,
        never_serves: true,
        notice: format!("{NEVER_FILE_NOTICE}\n{REVIEW_NOTICE}"),
        items,
        checklist,
    };
    crate::check::write_json(&dest.join("packet.json"), &report)?;
    Ok(report)
}

pub fn assemble_for_case(case_dir: &Path) -> Result<PacketReport, String> {
    let profile = case_profile::load_case(case_dir)?;
    assemble(case_dir, &profile)
}

pub fn packet_dir(case_dir: &Path) -> PathBuf {
    case_dir.join("exports").join("packet")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_profile::{ensure_case_layout, sample_appeal_profile, save_case};
    use uuid::Uuid;

    #[test]
    fn packet_writes_five_parts() {
        let dir = std::env::temp_dir().join(format!("os-pkt-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        ensure_case_layout(&dir).unwrap();
        let mut profile = sample_appeal_profile();
        profile.document_kind = "reconsideration".into();
        profile.motion.user_text = "The clerk mailed the order on March 1.".into();
        save_case(&dir, &profile).unwrap();
        let report = assemble(&dir, &profile).unwrap();
        assert_eq!(report.items.len(), 5);
        assert!(report.never_files && report.never_serves);
        assert!(std::path::Path::new(&report.items[0].docx).is_file());
        let _ = std::fs::remove_dir_all(dir);
    }
}
