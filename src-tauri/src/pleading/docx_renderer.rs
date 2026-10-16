use std::io::Write;
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::case_profile::REVIEW_NOTICE;
use crate::court_rules::CourtRules;
use super::types::{xml_escape, BodyBlock, CaptionStyle, CaptionVenue, CourtDocument};

fn blank_or(value: &str) -> String {
    if value.trim().is_empty() {
        "_______________".into()
    } else {
        value.to_string()
    }
}

fn para_xml(text: &str, bold: bool, center: bool, size: u32, spacing: u32, font: &str) -> String {
    para_xml_ex(text, bold, center, size, spacing, font, 0, false)
}

fn para_xml_ex(
    text: &str,
    bold: bool,
    center: bool,
    size: u32,
    spacing: u32,
    font: &str,
    after: u32,
    keep_together: bool,
) -> String {
    let align = if center { "center" } else { "left" };
    let b = if bold { "<w:b/>" } else { "" };
    let keep = if keep_together {
        "<w:keepNext/>\n    <w:keepLines/>"
    } else {
        ""
    };
    format!(
        r#"<w:p>
  <w:pPr>
    <w:jc w:val="{align}"/>
    <w:spacing w:after="{after}" w:line="{spacing}" w:lineRule="auto"/>
    {keep}
  </w:pPr>
  <w:r>
    <w:rPr>
      <w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:cs="{font}"/>
      <w:sz w:val="{size}"/>
      <w:szCs w:val="{size}"/>
      {b}
    </w:rPr>
    <w:t xml:space="preserve">{text}</w:t>
  </w:r>
</w:p>"#,
        align = align,
        after = after,
        spacing = spacing,
        keep = keep,
        font = font,
        size = size,
        b = b,
        text = xml_escape(text)
    )
}

fn boxed_caption_docx(doc: &CourtDocument, font: &str) -> String {
    let mut left_text = String::new();
    left_text.push_str(&doc.court_name);
    if !doc.court_division.is_empty() {
        left_text.push('\n');
        left_text.push_str(&doc.court_division);
    }
    for a in &doc.court_address {
        if !a.trim().is_empty() {
            left_text.push('\n');
            left_text.push_str(a);
        }
    }
    left_text.push_str(&format!(
        "\n\n{}: {}\nv.\n{}: {}\n\n{}, {}",
        doc.left_label, doc.left_names, doc.right_label, doc.right_names, doc.filer_name, doc.filer_role
    ));
    for a in &doc.filer_address {
        if !a.trim().is_empty() {
            left_text.push('\n');
            left_text.push_str(a);
        }
    }
    if !doc.filer_email.is_empty() {
        left_text.push('\n');
        left_text.push_str(&doc.filer_email);
    }
    if !doc.filer_phone.is_empty() {
        left_text.push('\n');
        left_text.push_str(&doc.filer_phone);
    }
    let appellate = blank_or(&doc.appellate_no);
    let trial = blank_or(&doc.trial_no);
    let lower = blank_or(&doc.lower_court_name);
    let division_line = if doc.division_courtroom.trim().is_empty() {
        String::new()
    } else {
        format!("\nDivision / Courtroom: {}", doc.division_courtroom)
    };
    let right_text = match doc.venue {
        CaptionVenue::Colorado => format!(
            "COURT USE ONLY\n\nCase Number: {appellate}\nLower Court: {lower}\nLower Court Case: {trial}"
        ),
        CaptionVenue::NewJersey | CaptionVenue::Generic => format!(
            "COURT USE ONLY\n\nCase Number: {appellate}\nTrial Court: {trial}{division_line}"
        ),
    };

    let cell = |text: &str, width: u32| {
        let grid_span = "";
        let mut paras = String::new();
        for line in text.split('\n') {
            paras.push_str(&para_xml(line, false, false, 20, 240, font));
        }
        format!(
            r#"<w:tc>
  <w:tcPr>
    <w:tcW w:w="{width}" w:type="dxa"/>
    <w:tcBorders>
      <w:top w:val="single" w:sz="12" w:space="0" w:color="000000"/>
      <w:left w:val="single" w:sz="12" w:space="0" w:color="000000"/>
      <w:bottom w:val="single" w:sz="12" w:space="0" w:color="000000"/>
      <w:right w:val="single" w:sz="12" w:space="0" w:color="000000"/>
    </w:tcBorders>
    {grid_span}
  </w:tcPr>
  {paras}
</w:tc>"#
        )
    };

    let title_cell = format!(
        r#"<w:tc>
  <w:tcPr>
    <w:tcW w:w="10800" w:type="dxa"/>
    <w:gridSpan w:val="2"/>
    <w:tcBorders>
      <w:top w:val="single" w:sz="12" w:space="0" w:color="000000"/>
      <w:left w:val="single" w:sz="12" w:space="0" w:color="000000"/>
      <w:bottom w:val="single" w:sz="12" w:space="0" w:color="000000"/>
      <w:right w:val="single" w:sz="12" w:space="0" w:color="000000"/>
    </w:tcBorders>
  </w:tcPr>
  {}
</w:tc>"#,
        para_xml(&doc.title, true, true, 24, 240, font)
    );

    format!(
        r#"<w:tbl>
  <w:tblPr>
    <w:tblW w:w="10800" w:type="dxa"/>
    <w:tblLayout w:type="fixed"/>
  </w:tblPr>
  <w:tblGrid>
    <w:gridCol w:w="6840"/>
    <w:gridCol w:w="3960"/>
  </w:tblGrid>
  <w:tr>
    {left}
    {right}
  </w:tr>
  <w:tr>
    {title}
  </w:tr>
</w:tbl>
<w:p><w:pPr><w:spacing w:line="240" w:lineRule="auto"/></w:pPr></w:p>"#,
        left = cell(&left_text, 6840),
        right = cell(&right_text, 3960),
        title = title_cell
    )
}

pub fn to_docx(doc: &CourtDocument, rules: &CourtRules) -> Result<Vec<u8>, String> {
    let size = rules.font_size_half_points();
    let spacing = rules.line_spacing_twips();
    let font = xml_escape(&rules.font_family);
    let (top, right, bottom, left) = rules.margin_twips();
    let mut body = String::new();
    match doc.style {
        CaptionStyle::Boxed => body.push_str(&boxed_caption_docx(doc, &font)),
        CaptionStyle::Classic => {
            body.push_str(&para_xml(&doc.court_name, true, true, size, spacing, &font));
            if !doc.court_division.is_empty() {
                body.push_str(&para_xml(&doc.court_division, false, true, size, spacing, &font));
            }
            body.push_str(&para_xml(&doc.left_names, false, true, size, spacing, &font));
            body.push_str(&para_xml("v.", false, true, size, spacing, &font));
            body.push_str(&para_xml(&doc.right_names, false, true, size, spacing, &font));
            if !doc.appellate_no.is_empty() {
                body.push_str(&para_xml(
                    &format!("Case No. {}", doc.appellate_no),
                    false,
                    true,
                    size,
                    spacing,
                    &font,
                ));
            }
            body.push_str(&para_xml(&doc.title, true, true, size + 4, spacing, &font));
        }
    }
    for block in &doc.body {
        match block {
            BodyBlock::Heading(h) => {
                body.push_str(&para_xml_ex(
                    &h.to_ascii_uppercase(),
                    true,
                    false,
                    size,
                    spacing,
                    &font,
                    200,
                    false,
                ));
            }
            BodyBlock::Paragraph(p) => {
                body.push_str(&para_xml(p, false, false, size, spacing, &font));
            }
            BodyBlock::Blank => {
                body.push_str(&para_xml("", false, false, size, spacing, &font));
            }
        }
    }
    if let (Some(h), Some(t)) = (&doc.certification_heading, &doc.certification_text) {
        body.push_str(&para_xml_ex(h, true, true, size, spacing, &font, 120, true));
        body.push_str(&para_xml_ex(t, false, false, size, spacing, &font, 160, true));
    }
    let sig_count = doc.signature_lines.len();
    for (i, line) in doc.signature_lines.iter().enumerate() {
        body.push_str(&para_xml_ex(
            line,
            false,
            false,
            size,
            240,
            &font,
            0,
            i + 1 < sig_count,
        ));
    }
    body.push_str(r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#);
    body.push_str(&para_xml(&doc.certificate_heading, true, true, size, spacing, &font));
    for line in &doc.certificate_lines {
        body.push_str(&para_xml(line, false, false, size, spacing, &font));
    }

    let document_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
    {body}
    <w:sectPr>
      <w:footerReference w:type="default" r:id="rId2"/>
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

    let footer_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:p>
    <w:pPr><w:jc w:val="center"/></w:pPr>
    <w:r><w:t xml:space="preserve">Page </w:t></w:r>
    <w:r><w:fldChar w:fldCharType="begin"/></w:r>
    <w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r>
    <w:r><w:fldChar w:fldCharType="end"/></w:r>
    <w:r><w:t xml:space="preserve"> of </w:t></w:r>
    <w:r><w:fldChar w:fldCharType="begin"/></w:r>
    <w:r><w:instrText xml:space="preserve"> NUMPAGES </w:instrText></w:r>
    <w:r><w:fldChar w:fldCharType="end"/></w:r>
  </w:p>
</w:ftr>"#;

    pack_docx(&document_xml, footer_xml)
}

pub fn pack_docx(document_xml: &str, footer_xml: &str) -> Result<Vec<u8>, String> {
    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/footer1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/>
</Types>"#;
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;
    let doc_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/>
</Relationships>"#;

    let mut buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("[Content_Types].xml", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(content_types.as_bytes()).map_err(|e| e.to_string())?;
        zip.start_file("_rels/.rels", options).map_err(|e| e.to_string())?;
        zip.write_all(rels.as_bytes()).map_err(|e| e.to_string())?;
        zip.start_file("word/_rels/document.xml.rels", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(doc_rels.as_bytes()).map_err(|e| e.to_string())?;
        zip.start_file("word/document.xml", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(document_xml.as_bytes()).map_err(|e| e.to_string())?;
        zip.start_file("word/footer1.xml", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(footer_xml.as_bytes()).map_err(|e| e.to_string())?;
        zip.finish().map_err(|e| e.to_string())?;
    }
    Ok(buffer)
}

pub fn is_valid_docx(path: &Path) -> Result<bool, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let ok = archive.by_name("word/document.xml").is_ok();
    Ok(ok)
}

pub fn write_review_sidecar(exports_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(exports_dir).map_err(|e| e.to_string())?;
    std::fs::write(exports_dir.join("REVIEW-NOTICE.txt"), REVIEW_NOTICE).map_err(|e| e.to_string())
}
