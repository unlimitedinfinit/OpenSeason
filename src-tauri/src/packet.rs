//! Filing packet assembler. Formats papers only. Never files or serves.

use serde::{Deserialize, Serialize};
use std::fs;
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
    let venue = crate::legal_os::resolve_venue(profile);
    let oath_text = venue.mandatory_oath_text.as_str();
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
                oath_text,
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
                "I certify that on ____________ I served a copy of this paper on ____________ by ____________.",
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExhibitPacketItem {
    pub letter: String,
    pub description: String,
    pub filename: String,
    pub bates_start: String,
    pub bates_end: String,
    pub page_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExhibitPacketReport {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub notice: String,
    pub table_of_exhibits_pdf: String,
    pub exhibit_packet_pdf: String,
    pub bates_start: String,
    pub bates_end: String,
    pub total_exhibits: usize,
    pub total_pages: usize,
    pub exhibits: Vec<ExhibitPacketItem>,
}

pub fn assemble_exhibit_packet(
    case_dir: &Path,
    bates_prefix: Option<&str>,
    starting_number: Option<u32>,
) -> Result<ExhibitPacketReport, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Exhibit packet export from the Documents pointer is refused.".into());
    }
    let profile = case_profile::load_case(case_dir)?;
    let index = crate::exhibits::rebuild(case_dir)?;

    let prefix = bates_prefix
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            let stem = profile.caption.appellants.first()
                .or_else(|| profile.caption.plaintiffs.first())
                .map(|p| p.name.as_str())
                .unwrap_or("EX");
            stem.chars().filter(|c| c.is_ascii_alphabetic()).take(6).collect::<String>().to_uppercase()
        });

    let mut current_bates = starting_number.unwrap_or(1);
    let mut items = Vec::new();
    let evidence_dir = case_dir.join("evidence");

    for row in &index.items {
        let p = evidence_dir.join(&row.filename);
        let pages = if p.is_file() {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
            if ext == "txt" || ext == "md" {
                let lines = fs::read_to_string(&p).map(|s| s.lines().count()).unwrap_or(1);
                std::cmp::max(1, (lines + 44) / 45)
            } else {
                1
            }
        } else {
            1
        };

        let bstart = format!("{}-{:06}", prefix, current_bates);
        let bend = format!("{}-{:06}", prefix, current_bates + pages as u32 - 1);
        current_bates += pages as u32;

        items.push(ExhibitPacketItem {
            letter: row.letter.clone(),
            description: if row.description.is_empty() { row.filename.clone() } else { row.description.clone() },
            filename: row.filename.clone(),
            bates_start: bstart,
            bates_end: bend,
            page_count: pages,
        });
    }

    let exports_dir = case_dir.join("exports");
    fs::create_dir_all(&exports_dir).map_err(|e| e.to_string())?;

    // 1. Build Table of Exhibits Typst source
    let mut table_rows = String::new();
    for it in &items {
        table_rows.push_str(&format!(
            "[Ex. {}], [{}], [{}], [{}]\n",
            it.letter, it.bates_start, it.description.replace('[', "\\[").replace(']', "\\]"), it.filename
        ));
    }

    let court_name = profile.court.name.clone();
    let docket = profile.docket_number.clone();
    let filer_sig = profile.filer.signature_name.clone();
    let filer_name = profile.filer.name.clone();

    let table_typst = format!(
r##"
#set page(paper: "us-letter", margin: (x: 1in, y: 1in))
#set text(font: "Times New Roman", size: 12pt)
#set par(justify: false, leading: 0.8em)

#align(center)[
  #strong("{court_name}") \
  #v(0.5em)
  Docket No. {docket} \
  #v(1em)
  #strong([TABLE OF EXHIBITS])
]

#v(1.5em)

#table(
  columns: (auto, auto, 1fr, auto),
  inset: 7pt,
  align: (center, center, left, left),
  table.header(
    [*Exhibit*], [*Bates Range*], [*Description*], [*Source File*]
  ),
  {table_rows}
)

#v(2em)
#align(left)[
  *CERTIFICATION OF EXHIBITS* \
  I certify that the foregoing exhibits are true and authentic copies of the documents referenced in the accompanying papers.
  #v(1.5em)
  /s/ {filer_sig} \
  {filer_name}
]
"##,
        court_name = court_name,
        docket = docket,
        table_rows = table_rows,
        filer_sig = filer_sig,
        filer_name = filer_name,
    );

    let table_pdf_bytes = crate::pdf::compile_typst(table_typst)?;
    let table_pdf_path = exports_dir.join("table-of-exhibits.pdf");
    fs::write(&table_pdf_path, table_pdf_bytes).map_err(|e| e.to_string())?;

    // 2. Build Exhibit Packet Typst source with Bates stamps
    let mut packet_typst = format!(
r##"
#set page(
  paper: "us-letter",
  margin: (top: 1in, bottom: 1in, left: 1in, right: 1in),
  header: locate(loc => [
    #text(size: 9pt, fill: luma(80))[
      {court_name} | Docket No. {docket}
    ]
  ]),
  footer: locate(loc => [
    #text(size: 9pt, fill: luma(80))[
      #h(1fr) BATES: {prefix}-#numbering("000001", counter(page).at(loc).first())
    ]
  ])
)
#set text(font: "Times New Roman", size: 12pt)

#align(center + horizon)[
  #text(size: 20pt, weight: "bold")[EXHIBIT PACKET] \
  #v(1em)
  #text(size: 14pt)[{court_name}] \
  #v(0.5em)
  #text(size: 12pt)[Docket No. {docket}] \
  #v(2em)
  #text(size: 11pt)[Prepared: {date}]
]

#pagebreak()
#counter(page).update({start})
"##,
        court_name = court_name,
        docket = docket,
        prefix = prefix,
        start = starting_number.unwrap_or(1),
        date = chrono::Local::now().format("%B %d, %Y")
    );

    for it in &items {
        let p = evidence_dir.join(&it.filename);
        let content = if p.is_file() {
            fs::read_to_string(&p).unwrap_or_else(|_| format!("[Binary or PDF Attachment: {}]", it.filename))
        } else {
            "[Attached file]".to_string()
        };

        packet_typst.push_str(&format!(
r##"
#align(center)[
  #v(2em)
  #text(size: 16pt, weight: "bold")[EXHIBIT {letter}] \
  #v(0.5em)
  #text(size: 12pt, style: "italic")[{desc}] \
  #v(0.3em)
  #text(size: 10pt)[Bates: {bstart} - {bend}]
]
#v(1.5em)
#line(length: 100%, stroke: 0.5pt)
#v(1em)

#raw("{content}")

#pagebreak()
"##,
            letter = it.letter,
            desc = it.description.replace('"', "\\\""),
            bstart = it.bates_start,
            bend = it.bates_end,
            content = content.replace('\\', "\\\\").replace('"', "\\\"")
        ));
    }

    let packet_pdf_bytes = crate::pdf::compile_typst(packet_typst)?;
    let packet_pdf_path = exports_dir.join("exhibit-packet.pdf");
    fs::write(&packet_pdf_path, packet_pdf_bytes).map_err(|e| e.to_string())?;

    let bates_start_str = items.first().map(|i| i.bates_start.clone()).unwrap_or_default();
    let bates_end_str = items.last().map(|i| i.bates_end.clone()).unwrap_or_default();
    let total_pages: usize = items.iter().map(|i| i.page_count).sum();

    let report = ExhibitPacketReport {
        ok: true,
        never_files: true,
        never_serves: true,
        notice: format!("{NEVER_FILE_NOTICE}\n{REVIEW_NOTICE}"),
        table_of_exhibits_pdf: table_pdf_path.display().to_string(),
        exhibit_packet_pdf: packet_pdf_path.display().to_string(),
        bates_start: bates_start_str,
        bates_end: bates_end_str,
        total_exhibits: items.len(),
        total_pages,
        exhibits: items,
    };

    crate::check::write_json(&exports_dir.join("exhibit-packet.json"), &report)?;
    Ok(report)
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
        for item in &report.items {
            let xml = std::fs::read_to_string(&item.docx).unwrap_or_default();
            // Real Word files are zip archives, not plain text.
            assert!(crate::pleading::is_valid_docx(std::path::Path::new(&item.docx)).unwrap());
            let _ = xml;
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn exhibit_packet_bates_starts_at_requested_number() {
        let dir = std::env::temp_dir().join(format!("os-bates-{}", Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("evidence")).unwrap();
        ensure_case_layout(&dir).unwrap();
        let profile = sample_appeal_profile();
        save_case(&dir, &profile).unwrap();
        std::fs::write(dir.join("evidence").join("order.txt"), "Order granting dismissal.\n").unwrap();
        let report = assemble_exhibit_packet(&dir, Some("EX"), Some(1)).unwrap();
        assert_eq!(report.exhibits[0].bates_start, "EX-000001");
        assert!(!report.exhibits[0].bates_start.ends_with("000002"));
        let pdf = std::fs::read(&report.exhibit_packet_pdf).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        let text = std::process::Command::new("pdftotext")
            .args(["-layout", &report.exhibit_packet_pdf, "-"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();
        if !text.is_empty() {
            assert!(
                text.contains("EX-000001") || text.contains("BATES: EX-000001"),
                "cover must not consume Bates 000001: {text}"
            );
            assert!(!text.contains("not legal advice"));
        }
        let _ = std::fs::remove_dir_all(dir);
    }
}
