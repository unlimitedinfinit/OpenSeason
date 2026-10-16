use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::case_profile::CaseProfile;
use crate::court_rules::CourtRules;
use super::assembly::assembled_notice_paragraphs;
use super::types::{typst_escape, xml_escape, Para};

pub(crate) fn build_docx_bytes(profile: &CaseProfile, rules: &CourtRules) -> Result<Vec<u8>, String> {
    build_docx_bytes_from_lines(&assembled_notice_paragraphs(profile), "NOTICE OF APPEAL", rules)
}

pub(crate) fn build_docx_bytes_from_lines(
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
        let align = if para.center { "center" } else { "left" };
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

pub(crate) fn build_typst_source(profile: &CaseProfile, rules: &CourtRules) -> String {
    build_typst_source_from_lines(&assembled_notice_paragraphs(profile), "NOTICE OF APPEAL", rules)
}

pub(crate) fn build_typst_source_from_lines(lines: &[String], heading: &str, rules: &CourtRules) -> String {
    let fonts = rules.typst_font_list();
    let size = rules.font_size_pt;
    let leading = if rules.line_spacing >= 1.8 {
        1.5
    } else {
        rules.line_spacing.max(0.8)
    };
    let mut blocks = String::new();
    let mut in_caption = true;
    for (index, line) in lines.iter().enumerate() {
        let escaped = typst_escape(line);
        let is_title = line.eq_ignore_ascii_case(heading) || (line == heading);
        if is_title {
            in_caption = false;
        }
        if line.is_empty() {
            blocks.push_str("#v(0.45em)\n");
            continue;
        }
        if is_title {
            blocks.push_str(&format!(
                "#align(center)[#text(size: {}pt, weight: \"bold\")[{}]]\n",
                size + 1,
                escaped
            ));
        } else if index == 0 || in_caption {
            blocks.push_str(&format!(
                "#align(center)[#text(weight: \"bold\")[{}]]\n",
                escaped
            ));
        } else {
            blocks.push_str(&format!("{escaped}\n\n"));
        }
    }

    format!(
        r#"
#set page(
  paper: "{paper}",
  margin: (top: {top}in, right: {right}in, bottom: {bottom}in, left: {left}in),
  footer: context {{
    align(center)[#text(size: 10pt)[Page #counter(page).display() of #counter(page).final().first()]]
  }}
)
#set text(font: ({fonts}), size: {size}pt)
#set par(leading: {leading}em, justify: false)

{blocks}
"#,
        paper = rules.paper,
        top = rules.margins_inches.top,
        right = rules.margins_inches.right,
        bottom = rules.margins_inches.bottom,
        left = rules.margins_inches.left,
        fonts = fonts,
        size = size,
        leading = leading,
        blocks = blocks
    )
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
