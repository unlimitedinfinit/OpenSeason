use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use serde::{Deserialize, Serialize};

use crate::case_profile::{validate_case, CaseProfile, Caption, REVIEW_NOTICE};
use crate::court_rules::CourtRules;
use crate::pdf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPaths {
    pub docx: PathBuf,
    pub pdf: PathBuf,
}

struct Para {
    text: String,
    bold: bool,
    center: bool,
    size_half_points: u32,
}

impl Para {
    fn centered(text: impl Into<String>, size: u32, bold: bool) -> Self {
        Self {
            text: text.into(),
            bold,
            center: true,
            size_half_points: size,
        }
    }
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn typst_escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('@', "\\@")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('<', "\\<")
        .replace('>', "\\>")
}

fn party_side_label(kind: &str, left: bool) -> &'static str {
    match (kind, left) {
        ("complaint", true) | ("motion", true) => "Plaintiff(s),",
        ("complaint", false) | ("motion", false) => "Defendant(s).",
        (_, true) => "Appellant(s),",
        (_, false) => "Appellee(s).",
    }
}

fn assembled_caption_lines(profile: &CaseProfile, kind: &str) -> Vec<String> {
    let left = profile.caption.appellants();
    let right = profile.caption.appellees();
    let mut lines = Vec::new();
    lines.push(profile.court.name.clone());
    if let Some(div) = &profile.court.division {
        if !div.trim().is_empty() {
            lines.push(div.clone());
        }
    }
    lines.push(String::new());
    lines.push(Caption::party_names(&left));
    lines.push(party_side_label(kind, true).to_string());
    lines.push(String::new());
    lines.push("v.".to_string());
    lines.push(String::new());
    lines.push(Caption::party_names(&right));
    lines.push(party_side_label(kind, false).to_string());
    lines.push(String::new());
    lines.push(format!("Case No. {}", profile.docket_number));
    lines.push(String::new());
    lines
}

fn assembled_signature_lines(profile: &CaseProfile) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!(
        "Date: {}",
        chrono::Local::now().format("%B %d, %Y")
    ));
    lines.push(String::new());
    lines.push("Respectfully submitted,".to_string());
    lines.push(String::new());
    lines.push(format!("/s/ {}", profile.filer.signature_name));
    lines.push(profile.filer.name.clone());
    if !profile.filer.role.trim().is_empty() {
        lines.push(profile.filer.role.clone());
    }
    for line in &profile.filer.address_lines {
        if !line.trim().is_empty() {
            lines.push(line.clone());
        }
    }
    if !profile.filer.phone.trim().is_empty() {
        lines.push(profile.filer.phone.clone());
    }
    if !profile.filer.email.trim().is_empty() {
        lines.push(profile.filer.email.clone());
    }
    lines.push(String::new());
    lines.push(REVIEW_NOTICE.to_string());
    lines
}

fn push_user_paragraphs(lines: &mut Vec<String>, text: &str) {
    for paragraph in text.split("\n\n") {
        let trimmed = paragraph.trim();
        if !trimmed.is_empty() {
            lines.push(trimmed.to_string());
            lines.push(String::new());
        }
    }
}

pub fn pleading_heading(kind: &str) -> &'static str {
    match kind.trim().to_ascii_lowercase().as_str() {
        "complaint" => "COMPLAINT",
        "appellate_brief" | "brief" => "APPELLATE BRIEF",
        "appellate_motion" => "MOTION IN THE APPELLATE COURT",
        "mandamus" => "PETITION FOR RULE TO SHOW CAUSE",
        "motion" | "reconsideration" | "enlargement" | "stay" | "amend_findings" => "MOTION",
        _ => "NOTICE OF APPEAL",
    }
}

fn motion_kind_label(kind: &str, profile: &CaseProfile) -> String {
    let key = if kind == "motion" && !profile.motion_kind.is_empty() {
        profile.motion_kind.as_str()
    } else {
        kind
    };
    match key {
        "reconsideration" => "Motion for reconsideration (basic template)".into(),
        "enlargement" => "Motion for enlargement of time (basic template)".into(),
        "stay" => "Motion for a stay (basic template)".into(),
        "amend_findings" => "Motion to amend findings (basic template)".into(),
        "appellate_motion" => "Motion in the appellate court (basic template)".into(),
        "appellate_brief" | "brief" => "Appellate brief scaffold (basic)".into(),
        "mandamus" => "Colorado C.A.R. 21 petition (basic template)".into(),
        "complaint" => "Complaint (basic template)".into(),
        _ => "Motion (basic template)".into(),
    }
}

fn pleading_body<'a>(profile: &'a CaseProfile, kind: &str) -> (&'a str, &'a str) {
    match kind {
        "complaint" => (profile.complaint.title.as_str(), profile.complaint.user_text.as_str()),
        "appellate_brief" | "brief" => (profile.brief.title.as_str(), profile.brief.user_text.as_str()),
        "appellate_motion" => (
            profile.appellate_motion.title.as_str(),
            profile.appellate_motion.user_text.as_str(),
        ),
        "mandamus" => (profile.mandamus.title.as_str(), profile.mandamus.user_text.as_str()),
        _ => (profile.motion.title.as_str(), profile.motion.user_text.as_str()),
    }
}

fn scaffold_sections(kind: &str) -> &'static [&'static str] {
    match kind {
        "appellate_brief" | "brief" => &[
            "Table of contents (user supplies entries)",
            "Table of authorities (filled from cites the user wrote; unverified cites stay tagged)",
            "Preliminary statement",
            "Procedural history",
            "Statement of facts (each fact needs a record cite)",
            "Legal argument (user-supplied; this app does not invent points)",
            "Conclusion",
        ],
        "mandamus" => &[
            "Jurisdiction (C.A.R. 21)",
            "Parties",
            "Issues presented",
            "Facts from the record",
            "Reasons relief should issue (user-supplied)",
            "Prayer for relief",
        ],
        "reconsideration" => &[
            "The order challenged",
            "The rule relied on (user must verify)",
            "Grounds as written by the user",
            "Relief requested",
        ],
        "enlargement" => &[
            "The deadline that needs more time",
            "The reason for the request (user-supplied)",
            "The new date requested",
        ],
        "stay" => &[
            "The order to be stayed",
            "Why a stay is requested (user-supplied)",
            "The terms of the stay requested",
        ],
        "amend_findings" => &[
            "The finding to be amended",
            "The proposed amendment (user-supplied)",
            "Record support",
        ],
        _ => &[],
    }
}

pub fn assembled_pleading_paragraphs(profile: &CaseProfile, kind: &str) -> Vec<String> {
    let key = kind.trim().to_ascii_lowercase();
    if key.is_empty() || key == "notice_of_appeal" || key == "appeal" {
        return assembled_notice_paragraphs(profile);
    }
    if !profile.court_markdown.trim().is_empty() && key == "court_md" {
        return crate::court_md::parse_markdown(&profile.court_markdown).lines;
    }

    let caption_kind = match key.as_str() {
        "complaint" | "mandamus" => "complaint",
        _ => "notice_of_appeal",
    };
    let mut lines = assembled_caption_lines(profile, caption_kind);
    let heading = pleading_heading(&key);
    lines.push(heading.to_string());
    lines.push(String::new());
    let (subtitle, body) = pleading_body(profile, &key);
    if !subtitle.trim().is_empty() {
        lines.push(subtitle.trim().to_string());
        lines.push(String::new());
    }
    lines.push(motion_kind_label(&key, profile));
    lines.push(
        "The builder filled the caption, parties, and signature block from the case profile. The body below is the user's own text. This app does not invent facts, grounds, or citations.".into(),
    );
    lines.push(String::new());
    for section in scaffold_sections(&key) {
        lines.push(section.to_string());
        lines.push(String::new());
    }
    push_user_paragraphs(&mut lines, body);
    lines.extend(assembled_signature_lines(profile));
    lines.push(String::new());
    lines.push("CERTIFICATE OF SERVICE".into());
    lines.push(
        "I will serve a copy of this paper myself. This software did not file or serve anything."
            .into(),
    );
    lines
}

pub fn assembled_notice_paragraphs(profile: &CaseProfile) -> Vec<String> {
    let appellants = profile.caption.appellants();
    let appellant_names = Caption::party_names(&appellants);

    let mut lines = assembled_caption_lines(profile, "notice_of_appeal");
    if !profile.notice_of_appeal.trial_court_docket.trim().is_empty() {
        // Insert trial docket after Case No. (last non-empty caption line before blank).
        if let Some(blank) = lines.iter().rposition(|l| l.is_empty()) {
            lines.insert(
                blank,
                format!(
                    "Trial court docket: {}",
                    profile.notice_of_appeal.trial_court_docket
                ),
            );
        }
    }
    lines.push("NOTICE OF APPEAL".to_string());
    lines.push(String::new());

    let mut lead = format!(
        "Notice is given that {} appeal(s) to {} from the {} entered on {}.",
        appellant_names,
        profile.court.name,
        profile.notice_of_appeal.judgment_description.trim(),
        profile.notice_of_appeal.judgment_date.trim()
    );
    if !profile.notice_of_appeal.trial_court_name.trim().is_empty() {
        lead.push_str(&format!(
            " The order was entered in {}.",
            profile.notice_of_appeal.trial_court_name.trim()
        ));
    }
    lines.push(lead);
    lines.push(String::new());
    push_user_paragraphs(&mut lines, &profile.notice_of_appeal.user_text);
    lines.extend(assembled_signature_lines(profile));
    lines
}

fn build_docx_bytes(profile: &CaseProfile, rules: &CourtRules) -> Result<Vec<u8>, String> {
    build_docx_bytes_from_lines(&assembled_notice_paragraphs(profile), "NOTICE OF APPEAL", rules)
}

fn build_docx_bytes_from_lines(
    lines: &[String],
    heading: &str,
    rules: &CourtRules,
) -> Result<Vec<u8>, String> {
    let size = rules.font_size_half_points();
    let mut paras = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let is_title = line == heading;
        let is_court = index == 0;
        let center = is_title
            || is_court
            || line == "v."
            || line.ends_with("Appellant(s),")
            || line.ends_with("Appellee(s).")
            || rules.caption_align == "center" && index < 12;
        paras.push(if is_title {
            Para::centered(line.clone(), size + 4, true)
        } else {
            Para {
                text: line.clone(),
                bold: is_court || is_title,
                center,
                size_half_points: size,
            }
        });
    }

    let (top, right, bottom, left) = rules.margin_twips();
    let spacing = rules.line_spacing_twips();
    let font = xml_escape(&rules.font_family);

    let mut body = String::new();
    for para in paras {
        let align = if para.center { "center" } else { "both" };
        let bold = if para.bold { "<w:b/>" } else { "" };
        let text = xml_escape(&para.text);
        body.push_str(&format!(
            r#"<w:p>
  <w:pPr>
    <w:jc w:val="{align}"/>
    <w:spacing w:line="{spacing}" w:lineRule="auto"/>
  </w:pPr>
  <w:r>
    <w:rPr>
      <w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:cs="{font}"/>
      <w:sz w:val="{sz}"/>
      <w:szCs w:val="{sz}"/>
      {bold}
    </w:rPr>
    <w:t xml:space="preserve">{text}</w:t>
  </w:r>
</w:p>"#,
            align = align,
            spacing = spacing,
            font = font,
            sz = para.size_half_points,
            bold = bold,
            text = text
        ));
    }

    let document_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    {body}
    <w:sectPr>
      <w:pgSz w:w="{pw}" w:h="{ph}"/>
      <w:pgMar w:top="{top}" w:right="{right}" w:bottom="{bottom}" w:left="{left}"/>
    </w:sectPr>
  </w:body>
</w:document>"#,
        body = body,
        pw = rules.page_width_twips,
        ph = rules.page_height_twips,
        top = top,
        right = right,
        bottom = bottom,
        left = left
    );

    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;

    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

    let doc_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#;

    let mut buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("[Content_Types].xml", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(content_types.as_bytes())
            .map_err(|e| e.to_string())?;
        zip.start_file("_rels/.rels", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(rels.as_bytes()).map_err(|e| e.to_string())?;
        zip.start_file("word/_rels/document.xml.rels", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(doc_rels.as_bytes())
            .map_err(|e| e.to_string())?;
        zip.start_file("word/document.xml", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(document_xml.as_bytes())
            .map_err(|e| e.to_string())?;
        zip.finish().map_err(|e| e.to_string())?;
    }
    Ok(buffer)
}

fn build_typst_source(profile: &CaseProfile, rules: &CourtRules) -> String {
    build_typst_source_from_lines(&assembled_notice_paragraphs(profile), "NOTICE OF APPEAL", rules)
}

fn build_typst_source_from_lines(lines: &[String], heading: &str, rules: &CourtRules) -> String {
    let fonts = rules.typst_font_list();
    let size = rules.font_size_pt;
    let leading = rules.line_spacing;
    let mut blocks = String::new();
    for (index, line) in lines.iter().enumerate() {
        let escaped = typst_escape(line);
        let is_title = line == heading;
        if line.is_empty() {
            blocks.push_str("#v(0.6em)\n");
            continue;
        }
        if is_title {
            blocks.push_str(&format!(
                "#align(center)[#text(size: {}pt, weight: \"bold\")[{}]]\n",
                size + 2,
                escaped
            ));
        } else if index == 0 {
            blocks.push_str(&format!(
                "#align(center)[#text(weight: \"bold\")[{}]]\n",
                escaped
            ));
        } else {
            blocks.push_str(&format!("#align(center)[{}]\n", escaped));
        }
    }

    format!(
        r#"
#set page(
  paper: "{paper}",
  margin: (top: {top}in, right: {right}in, bottom: {bottom}in, left: {left}in),
  footer: align(center)[
    #text(size: 8pt)[{notice}]
  ]
)
#set text(font: ({fonts}), size: {size}pt)
#set par(leading: {leading}em, justify: true)

{blocks}
"#,
        paper = rules.paper,
        top = rules.margins_inches.top,
        right = rules.margins_inches.right,
        bottom = rules.margins_inches.bottom,
        left = rules.margins_inches.left,
        notice = typst_escape(REVIEW_NOTICE),
        fonts = fonts,
        size = size,
        leading = leading,
        blocks = blocks
    )
}

fn assert_notice_out_dir(out: &Path) -> Result<(), String> {
    let name = out.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if crate::sandbox::is_reserved_marker_name(name) {
        return Err("Export cannot write to a path named .sealed.".to_string());
    }
    let app_data = crate::sandbox::default_app_data_root();
    let resolved = crate::sandbox::canonicalize_existing_prefix(out);
    if app_data.exists() {
        if let Ok(app_canon) = app_data.canonicalize() {
            if resolved == app_canon || resolved.starts_with(&app_canon) {
                return Err("Export cannot write inside the app data folder.".to_string());
            }
        }
    }
    Ok(())
}

pub fn export_notice_of_appeal(
    case_dir: &Path,
    profile: &CaseProfile,
    out_dir: Option<&Path>,
) -> Result<ExportPaths, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Export from the Documents pointer is refused.".to_string());
    }
    if let Some(out) = out_dir {
        assert_notice_out_dir(out)?;
    }
    let report = validate_case(profile, Some(case_dir));
    if !report.ok {
        let details = report
            .issues
            .iter()
            .map(|i| format!("{}: {}", i.field, i.message))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!("Case is not ready to export. {}", details));
    }

    let rules = CourtRules::load_for_case(case_dir);
    let dest = out_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| case_dir.join("exports"));
    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;

    let stem = format!(
        "notice-of-appeal-{}",
        profile
            .docket_number
            .replace('/', "-")
            .replace(' ', "_")
            .replace('\\', "-")
    );
    let docx_path = dest.join(format!("{}.docx", stem));
    let pdf_path = dest.join(format!("{}.pdf", stem));

    let docx_bytes = build_docx_bytes(profile, &rules)?;
    fs::write(&docx_path, &docx_bytes).map_err(|e| e.to_string())?;

    let typst_src = build_typst_source(profile, &rules);
    let mut pdf_bytes = pdf::compile_typst(typst_src)?;
    // Typst embeds page text as glyphs, so keep a plaintext PDF comment
    // with the same notice that already appears in the footer.
    pdf_bytes.extend_from_slice(b"\n% ");
    pdf_bytes.extend_from_slice(REVIEW_NOTICE.as_bytes());
    pdf_bytes.extend_from_slice(b"\n");
    fs::write(&pdf_path, &pdf_bytes).map_err(|e| e.to_string())?;

    Ok(ExportPaths {
        docx: docx_path,
        pdf: pdf_path,
    })
}

pub fn export_pleading(
    case_dir: &Path,
    profile: &CaseProfile,
    kind: &str,
    out_dir: Option<&Path>,
) -> Result<ExportPaths, String> {
    let key = kind.trim().to_ascii_lowercase();
    if key.is_empty() || key == "notice_of_appeal" || key == "appeal" {
        return export_notice_of_appeal(case_dir, profile, out_dir);
    }
    if key == "court_md" || key == "markdown" {
        return export_court_markdown(case_dir, profile, out_dir);
    }
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Export from the Documents pointer is refused.".to_string());
    }
    if let Some(out) = out_dir {
        assert_notice_out_dir(out)?;
    }
    let report = crate::case_profile::validate_pleading(profile, Some(case_dir), &key);
    if !report.ok {
        let details = report
            .issues
            .iter()
            .map(|i| format!("{}: {}", i.field, i.message))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!("Case is not ready to export. {}", details));
    }

    let heading = pleading_heading(&key);
    let lines = assembled_pleading_paragraphs(profile, &key);
    let rules = CourtRules::load_for_case(case_dir);
    let dest = out_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| case_dir.join("exports"));
    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;

    let stem = format!(
        "{}-{}",
        key.replace('_', "-"),
        profile
            .docket_number
            .replace('/', "-")
            .replace(' ', "_")
            .replace('\\', "-")
    );
    let docx_path = dest.join(format!("{}.docx", stem));
    let pdf_path = dest.join(format!("{}.pdf", stem));

    let docx_bytes = build_docx_bytes_from_lines(&lines, heading, &rules)?;
    fs::write(&docx_path, &docx_bytes).map_err(|e| e.to_string())?;

    let typst_src = build_typst_source_from_lines(&lines, heading, &rules);
    let mut pdf_bytes = pdf::compile_typst(typst_src)?;
    pdf_bytes.extend_from_slice(b"\n% ");
    pdf_bytes.extend_from_slice(REVIEW_NOTICE.as_bytes());
    pdf_bytes.extend_from_slice(b"\n");
    fs::write(&pdf_path, &pdf_bytes).map_err(|e| e.to_string())?;

    Ok(ExportPaths {
        docx: docx_path,
        pdf: pdf_path,
    })
}

pub fn export_custom_filing(
    case_dir: &Path,
    profile: &CaseProfile,
    title: &str,
    user_text: &str,
    out_dir: Option<&Path>,
) -> Result<ExportPaths, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Export from the Documents pointer is refused.".to_string());
    }
    if let Some(out) = out_dir {
        assert_notice_out_dir(out)?;
    }

    let mut lines = assembled_caption_lines(profile, "motion");
    let heading = if title.trim().is_empty() { "MOTION" } else { title.trim() };
    lines.push(heading.to_string());
    lines.push(String::new());

    push_user_paragraphs(&mut lines, user_text);
    lines.extend(assembled_signature_lines(profile));

    let rules = CourtRules::load_for_case(case_dir);
    let dest = out_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| case_dir.join("exports"));
    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;

    let safe_title = heading
        .to_ascii_lowercase()
        .replace(' ', "-")
        .replace('/', "-")
        .replace('\\', "-");
    let stem = format!(
        "filing-{}-{}",
        safe_title,
        profile
            .docket_number
            .replace('/', "-")
            .replace(' ', "_")
            .replace('\\', "-")
    );
    let docx_path = dest.join(format!("{}.docx", stem));
    let pdf_path = dest.join(format!("{}.pdf", stem));

    let docx_bytes = build_docx_bytes_from_lines(&lines, heading, &rules)?;
    fs::write(&docx_path, &docx_bytes).map_err(|e| e.to_string())?;

    let typst_src = build_typst_source_from_lines(&lines, heading, &rules);
    let mut pdf_bytes = pdf::compile_typst(typst_src)?;
    pdf_bytes.extend_from_slice(b"\n% ");
    pdf_bytes.extend_from_slice(REVIEW_NOTICE.as_bytes());
    pdf_bytes.extend_from_slice(b"\n");
    fs::write(&pdf_path, &pdf_bytes).map_err(|e| e.to_string())?;

    Ok(ExportPaths {
        docx: docx_path,
        pdf: pdf_path,
    })
}

pub fn export_court_markdown(
    case_dir: &Path,
    profile: &CaseProfile,
    out_dir: Option<&Path>,
) -> Result<ExportPaths, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Export from the Documents pointer is refused.".to_string());
    }
    if let Some(out) = out_dir {
        assert_notice_out_dir(out)?;
    }
    let markdown = if !profile.court_markdown.trim().is_empty() {
        profile.court_markdown.clone()
    } else {
        let path = case_dir.join("drafts").join("draft.md");
        if path.is_file() {
            fs::read_to_string(path).map_err(|e| e.to_string())?
        } else {
            crate::court_md::markdown_from_profile(profile, &profile.document_kind)
        }
    };
    let doc = crate::court_md::parse_markdown(&markdown);
    if doc.lines.is_empty() {
        return Err("Markdown draft is empty.".into());
    }
    let stem = format!(
        "court-md-{}",
        profile
            .docket_number
            .replace('/', "-")
            .replace(' ', "_")
            .replace('\\', "-")
    );
    let heading = if doc.meta.doc_title.is_empty() {
        "MOTION"
    } else {
        doc.meta.doc_title.as_str()
    };
    write_lines_as_docx_pdf(case_dir, &stem, heading, &doc.lines, out_dir)
}

/// Write court-shaped Word and PDF from already-assembled lines.
pub fn write_lines_as_docx_pdf(
    case_dir: &Path,
    stem: &str,
    heading: &str,
    lines: &[String],
    out_dir: Option<&Path>,
) -> Result<ExportPaths, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Export from the Documents pointer is refused.".to_string());
    }
    if let Some(out) = out_dir {
        assert_notice_out_dir(out)?;
    }
    if lines.is_empty() {
        return Err("Nothing to export. Fill a guided section or import Markdown first.".into());
    }
    let rules = CourtRules::load_for_case(case_dir);
    let dest = out_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| case_dir.join("exports"));
    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    let docx_path = dest.join(format!("{stem}.docx"));
    let pdf_path = dest.join(format!("{stem}.pdf"));
    let docx_bytes = build_docx_bytes_from_lines(lines, heading, &rules)?;
    fs::write(&docx_path, &docx_bytes).map_err(|e| e.to_string())?;
    let typst_src = build_typst_source_from_lines(lines, heading, &rules);
    let mut pdf_bytes = crate::pdf::compile_typst(typst_src)?;
    pdf_bytes.extend_from_slice(b"\n% ");
    pdf_bytes.extend_from_slice(REVIEW_NOTICE.as_bytes());
    pdf_bytes.extend_from_slice(b"\n");
    fs::write(&pdf_path, &pdf_bytes).map_err(|e| e.to_string())?;
    Ok(ExportPaths {
        docx: docx_path,
        pdf: pdf_path,
    })
}

/// Used by tests that only need the Word file (no font-dependent PDF).
pub fn export_notice_of_appeal_docx_only(
    case_dir: &Path,
    profile: &CaseProfile,
    out_dir: Option<&Path>,
) -> Result<PathBuf, String> {
    let report = validate_case(profile, Some(case_dir));
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Export from the Documents pointer is refused.".to_string());
    }
    if let Some(out) = out_dir {
        assert_notice_out_dir(out)?;
    }
    if !report.ok {
        return Err("Case is not ready to export.".to_string());
    }
    let rules = CourtRules::load_for_case(case_dir);
    let dest = out_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| case_dir.join("exports"));
    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    let path = dest.join("notice-of-appeal.docx");
    let bytes = build_docx_bytes(profile, &rules)?;
    fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn inspect_pdf_contains(path: &Path, needle: &str) -> Result<bool, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    if bytes.windows(needle.len()).any(|w| w == needle.as_bytes()) {
        return Ok(true);
    }
    if String::from_utf8_lossy(&bytes).contains(needle) {
        return Ok(true);
    }
    // Typst often stores page text in Flate streams.
    let mut i = 0;
    while i + 10 < bytes.len() {
        if bytes[i..].starts_with(b"stream") {
            let start = i + 6;
            let rest = &bytes[start..];
            let end = rest
                .windows(9)
                .position(|w| w == b"endstream")
                .map(|p| start + p);
            if let Some(end) = end {
                let mut payload = &bytes[start..end];
                if payload.starts_with(b"\r\n") {
                    payload = &payload[2..];
                } else if payload.starts_with(b"\n") {
                    payload = &payload[1..];
                }
                let mut decoded = Vec::new();
                let mut decoder = flate2::read::ZlibDecoder::new(payload);
                if std::io::Read::read_to_end(&mut decoder, &mut decoded).is_ok()
                    && String::from_utf8_lossy(&decoded).contains(needle)
                {
                    return Ok(true);
                }
                i = end + 9;
                continue;
            }
        }
        i += 1;
    }
    Ok(false)
}

pub fn inspect_docx_contains(path: &Path, needle: &str) -> Result<bool, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut document = archive
        .by_name("word/document.xml")
        .map_err(|e| e.to_string())?;
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut document, &mut xml).map_err(|e| e.to_string())?;
    Ok(xml.contains(&xml_escape(needle)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_profile::{ensure_case_layout, sample_appeal_profile, save_case};
    use uuid::Uuid;

    fn temp_case() -> (PathBuf, CaseProfile) {
        let dir = std::env::temp_dir().join(format!("os-doc-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        ensure_case_layout(&dir).unwrap();
        let profile = sample_appeal_profile();
        save_case(&dir, &profile).unwrap();
        (dir, profile)
    }

    #[test]
    fn notice_pulls_caption_and_user_text() {
        let profile = sample_appeal_profile();
        let lines = assembled_notice_paragraphs(&profile);
        let blob = lines.join("\n");
        assert!(blob.contains("Jordan Example"));
        assert!(blob.contains("Sample County Clerk"));
        assert!(blob.contains("24-01000"));
        assert!(blob.contains("United States Court of Appeals for the Sample Circuit"));
        assert!(blob.contains("user's own draft"));
        assert!(blob.contains(REVIEW_NOTICE));
        assert!(!blob.contains("Jane Doe"));
    }

    #[test]
    fn docx_export_embeds_profile_fields() {
        let (dir, profile) = temp_case();
        let path = export_notice_of_appeal_docx_only(&dir, &profile, None).unwrap();
        assert!(path.exists());
        assert!(inspect_docx_contains(&path, "24-01000").unwrap());
        assert!(inspect_docx_contains(&path, "Jordan Example").unwrap());
        assert!(inspect_docx_contains(&path, REVIEW_NOTICE).unwrap());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_refuses_incomplete_case() {
        let (dir, mut profile) = temp_case();
        profile.docket_number.clear();
        let err = export_notice_of_appeal_docx_only(&dir, &profile, None).unwrap_err();
        assert!(err.contains("not ready"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn notice_pdf_compiles_for_sample_case() {
        let (dir, profile) = temp_case();
        let result = export_notice_of_appeal(&dir, &profile, None);
        match result {
            Ok(paths) => {
                let bytes = fs::read(&paths.pdf).unwrap();
                assert!(bytes.starts_with(b"%PDF"), "expected a PDF header");
                assert!(paths.docx.exists());
            }
            Err(e) => panic!("PDF export should work when a system font exists: {}", e),
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn exported_docx_and_pdf_contain_review_notice() {
        let (dir, profile) = temp_case();
        let paths = export_notice_of_appeal(&dir, &profile, None)
            .expect("sample case should export Word and PDF");
        assert!(
            inspect_docx_contains(&paths.docx, REVIEW_NOTICE).unwrap(),
            "DOCX must include the review / not-legal-advice notice"
        );
        assert!(
            inspect_docx_contains(&paths.docx, "not legal advice").unwrap(),
            "DOCX must say it is not legal advice"
        );
        assert!(
            inspect_pdf_contains(&paths.pdf, "not legal advice").unwrap(),
            "PDF must say it is not legal advice"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn complaint_and_motion_templates_use_profile() {
        let profile = sample_appeal_profile();
        let complaint = assembled_pleading_paragraphs(&profile, "complaint").join("\n");
        assert!(complaint.contains("COMPLAINT"));
        assert!(complaint.contains("Jordan Example"));
        assert!(complaint.to_ascii_lowercase().contains("basic"));
        assert!(complaint.contains("COMPLAINT") || complaint.contains("Complaint"));
        assert!(complaint.contains("user's own statement"));
        assert!(complaint.contains(REVIEW_NOTICE));
        let motion = assembled_pleading_paragraphs(&profile, "motion").join("\n");
        assert!(motion.contains("MOTION"));
        assert!(motion.contains("user's own motion text"));
        assert!(motion.contains("Sample County Clerk"));
    }

    #[test]
    fn complaint_docx_export_embeds_profile_fields() {
        let (dir, profile) = temp_case();
        let paths = export_pleading(&dir, &profile, "complaint", None).unwrap();
        assert!(paths.docx.exists());
        assert!(inspect_docx_contains(&paths.docx, "COMPLAINT").unwrap());
        assert!(inspect_docx_contains(&paths.docx, "Jordan Example").unwrap());
        assert!(inspect_docx_contains(&paths.docx, REVIEW_NOTICE).unwrap());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn appellate_brief_scaffold_uses_profile_and_says_basic() {
        let mut profile = sample_appeal_profile();
        profile.brief.user_text = "This paragraph is the user's own brief point.".into();
        let blob = assembled_pleading_paragraphs(&profile, "appellate_brief").join("\n");
        assert!(blob.contains("APPELLATE BRIEF"));
        assert!(blob.contains("basic"));
        assert!(blob.contains("user's own brief point"));
        assert!(blob.contains("CERTIFICATE OF SERVICE"));
        assert!(blob.contains("did not file or serve"));
        assert!(blob.contains("Jordan Example"));
    }
}
