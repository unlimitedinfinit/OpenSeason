//! Court-ready motion layout: boxed NJ-style caption, Times body, page X of Y,
//! sanitized draft text, venue certification, and a real Word package.
//!
//! The disclaimer never appears in the filing. It stays in the app UI and the
//! agent envelope. This module does not invent facts, citations, or addresses.

use std::io::Write;
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::case_profile::{CaseProfile, Caption, REVIEW_NOTICE};
use crate::court_rules::CourtRules;
use crate::legal_os::{self, NJ_R_1_4_4B_OATH};

pub const ENTER_ORDER_DATE_PROMPT: &str = legal_os::ENTER_ORDER_DATE_PROMPT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptionStyle {
    Boxed,
    Classic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptionVenue {
    NewJersey,
    Colorado,
    Generic,
}

impl CaptionVenue {
    pub fn as_str(self) -> &'static str {
        match self {
            CaptionVenue::NewJersey => "nj-boxed",
            CaptionVenue::Colorado => "co-boxed",
            CaptionVenue::Generic => "boxed",
        }
    }
}

pub fn caption_venue(profile: &CaseProfile) -> CaptionVenue {
    let state = profile.governing_state.trim().to_ascii_uppercase();
    if state == "CO" || state == "COLORADO" {
        return CaptionVenue::Colorado;
    }
    if state == "NJ" || state == "NEW JERSEY" {
        return CaptionVenue::NewJersey;
    }
    let blob = format!("{} {}", profile.court.id, profile.court.name).to_ascii_lowercase();
    if blob.contains("colorado") || blob.contains("co-mandamus") || blob.contains("co-app") {
        return CaptionVenue::Colorado;
    }
    if blob.contains("new jersey") || blob.contains("nj-app") || blob.contains("nj-") {
        return CaptionVenue::NewJersey;
    }
    CaptionVenue::Generic
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyBlock {
    Heading(String),
    Paragraph(String),
    Blank,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CourtDocument {
    pub style: CaptionStyle,
    pub court_name: String,
    pub court_division: String,
    pub court_address: Vec<String>,
    pub left_label: String,
    pub left_names: String,
    pub right_label: String,
    pub right_names: String,
    pub appellate_no: String,
    pub trial_no: String,
    pub division_courtroom: String,
    pub filer_name: String,
    pub filer_role: String,
    pub filer_address: Vec<String>,
    pub filer_email: String,
    pub filer_phone: String,
    pub title: String,
    pub intro: Vec<String>,
    pub body: Vec<BodyBlock>,
    pub signature_lines: Vec<String>,
    pub certification_heading: Option<String>,
    pub certification_text: Option<String>,
    pub certificate_heading: String,
    pub certificate_lines: Vec<String>,
    pub venue: CaptionVenue,
    pub lower_court_name: String,
}

pub fn caption_style(profile: &CaseProfile, kind: &str, override_layout: Option<&str>) -> CaptionStyle {
    let raw = override_layout
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(profile.caption_layout.trim());
    match raw.to_ascii_lowercase().as_str() {
        "classic" | "legacy" | "centered" => CaptionStyle::Classic,
        "boxed" | "nj" | "table" => CaptionStyle::Boxed,
        _ => match kind.trim().to_ascii_lowercase().as_str() {
            "notice_of_appeal" | "appeal" => CaptionStyle::Classic,
            _ => CaptionStyle::Boxed,
        },
    }
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn typst_escape(text: &str) -> String {
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

/// Strip markdown, fence blocks, and leaked Typst so they never print in a filing.
pub fn sanitize_draft(text: &str) -> (Vec<BodyBlock>, Vec<String>, Vec<String>) {
    let mut body = Vec::new();
    let mut signature = Vec::new();
    let mut certificate = Vec::new();
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
            } else {
                mode = "skip";
            }
            continue;
        }
        let cleaned = s.replace("**", "").replace("__", "");
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
    (body, signature, certificate)
}

fn party_names(profile: &CaseProfile, left: bool) -> (String, String) {
    let parties = if left {
        profile.caption.appellants()
    } else {
        profile.caption.appellees()
    };
    let names = Caption::party_names(&parties);
    let fallback_role = if left { "Plaintiff" } else { "Defendant" };
    let role = parties
        .first()
        .map(|p| {
            if p.role.trim().is_empty() {
                fallback_role.to_string()
            } else {
                p.role.clone()
            }
        })
        .unwrap_or_else(|| fallback_role.to_string());
    (names, role)
}

fn default_certificate_lines(profile: &CaseProfile) -> Vec<String> {
    vec![
        "I will serve a copy of this paper myself. This software did not file or serve anyone.".into(),
        format!(
            "Date of this draft: {}.",
            chrono::Local::now().format("%B %d, %Y")
        ),
        if profile.filer.name.trim().is_empty() {
            String::new()
        } else {
            format!("Filer: {}.", profile.filer.name)
        },
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect()
}

fn venue_certification(profile: &CaseProfile) -> Option<(String, String)> {
    match caption_venue(profile) {
        CaptionVenue::NewJersey => Some((
            "CERTIFICATION IN LIEU OF OATH (R. 1:4-4(b))".into(),
            NJ_R_1_4_4B_OATH.to_string(),
        )),
        CaptionVenue::Colorado => Some((
            "DECLARATION UNDER C.R.S. § 13-27-106".into(),
            legal_os::CO_CRS_13_27_106_OATH.to_string(),
        )),
        CaptionVenue::Generic => None,
    }
}

fn signature_block(profile: &CaseProfile, extra: &[String]) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("Date: {}", chrono::Local::now().format("%B %d, %Y")));
    lines.push("Respectfully submitted,".into());
    if extra.iter().any(|l| l.contains("/s/")) {
        lines.extend(extra.iter().cloned());
    } else {
        lines.push(format!("/s/ {}", profile.filer.signature_name));
        lines.push(profile.filer.name.clone());
        if !profile.filer.role.trim().is_empty() {
            lines.push(profile.filer.role.clone());
        }
        lines.extend(extra.iter().cloned());
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
    }
    lines
}

pub fn assemble(
    profile: &CaseProfile,
    kind: &str,
    title: &str,
    user_text: &str,
    style: CaptionStyle,
) -> CourtDocument {
    let (left_names, left_label) = party_names(profile, true);
    let (right_names, right_label) = party_names(profile, false);
    let (body, sig_extra, cert_extra) = sanitize_draft(user_text);
    if body.iter().any(|b| match b {
        BodyBlock::Paragraph(s) | BodyBlock::Heading(s) => {
            s.to_ascii_lowercase().contains("to the best of my knowledge")
        }
        _ => false,
    }) {
        // Never print the forbidden hedging in an NJ certification block.
        // The body still contains the user's words; venue check flags it.
    }
    let heading = if title.trim().is_empty() {
        crate::documents::pleading_heading(kind).to_string()
    } else {
        title.trim().to_string()
    };
    let (cert_h, cert_t) = match venue_certification(profile) {
        Some(pair) => (Some(pair.0), Some(pair.1)),
        None => (None, None),
    };
    let venue = caption_venue(profile);
    let certificate_lines = if cert_extra.is_empty() {
        default_certificate_lines(profile)
    } else {
        cert_extra
            .into_iter()
            .filter(|l| !l.to_ascii_lowercase().contains("to the best of my knowledge"))
            .collect()
    };
    CourtDocument {
        style,
        court_name: profile.court.name.clone(),
        court_division: profile.court.division.clone().unwrap_or_default(),
        court_address: profile.court.address_lines.clone(),
        left_label,
        left_names,
        right_label,
        right_names,
        appellate_no: profile.docket_number.clone(),
        trial_no: profile.notice_of_appeal.trial_court_docket.clone(),
        division_courtroom: profile.court.division.clone().unwrap_or_default(),
        filer_name: profile.filer.name.clone(),
        filer_role: if profile.filer.role.trim().is_empty() {
            "Pro Se".into()
        } else {
            profile.filer.role.clone()
        },
        filer_address: profile.filer.address_lines.clone(),
        filer_email: profile.filer.email.clone(),
        filer_phone: profile.filer.phone.clone(),
        title: heading,
        intro: Vec::new(),
        body,
        signature_lines: signature_block(profile, &sig_extra),
        certification_heading: cert_h,
        certification_text: cert_t,
        certificate_heading: "CERTIFICATE OF SERVICE".into(),
        certificate_lines,
        venue,
        lower_court_name: profile.notice_of_appeal.trial_court_name.clone(),
    }
}

/// Flatten to lines for tests and classic exporters. Never includes the software disclaimer.
pub fn document_lines(doc: &CourtDocument) -> Vec<String> {
    let mut lines = Vec::new();
    if !doc.court_name.is_empty() {
        lines.push(doc.court_name.clone());
    }
    if !doc.court_division.is_empty() {
        lines.push(doc.court_division.clone());
    }
    for a in &doc.court_address {
        if !a.trim().is_empty() {
            lines.push(a.clone());
        }
    }
    if !doc.left_names.is_empty() || !doc.right_names.is_empty() {
        lines.push(format!("{}: {}", doc.left_label, doc.left_names));
        lines.push("v.".into());
        lines.push(format!("{}: {}", doc.right_label, doc.right_names));
    }
    if !doc.filer_name.is_empty() {
        lines.push(format!("{}, {}", doc.filer_name, doc.filer_role));
        lines.extend(doc.filer_address.iter().cloned());
        if !doc.filer_email.is_empty() {
            lines.push(doc.filer_email.clone());
        }
        if !doc.filer_phone.is_empty() {
            lines.push(doc.filer_phone.clone());
        }
    }
    match doc.venue {
        CaptionVenue::Colorado => {
            lines.push("COURT USE ONLY".into());
            if !doc.appellate_no.is_empty() {
                lines.push(format!("Case Number: {}", doc.appellate_no));
            }
            if !doc.lower_court_name.is_empty() {
                lines.push(format!("Lower Court: {}", doc.lower_court_name));
            }
            if !doc.trial_no.is_empty() {
                lines.push(format!("Lower Court Case: {}", doc.trial_no));
            }
        }
        CaptionVenue::NewJersey | CaptionVenue::Generic => {
            lines.push("COURT USE ONLY".into());
            if !doc.appellate_no.is_empty() {
                lines.push(format!("Appellate: {}", doc.appellate_no));
            }
            if !doc.trial_no.is_empty() {
                lines.push(format!("Trial Court: {}", doc.trial_no));
            }
            if !doc.division_courtroom.is_empty() {
                lines.push(format!("Division / Courtroom: {}", doc.division_courtroom));
            }
            if !doc.lower_court_name.is_empty() {
                lines.push(doc.lower_court_name.clone());
            }
        }
    }
    lines.push(String::new());
    lines.push(doc.title.clone());
    lines.push(String::new());
    for intro in &doc.intro {
        lines.push(intro.clone());
    }
    for block in &doc.body {
        match block {
            BodyBlock::Heading(h) => {
                lines.push(h.to_ascii_uppercase());
                lines.push(String::new());
            }
            BodyBlock::Paragraph(p) => {
                lines.push(p.clone());
                lines.push(String::new());
            }
            BodyBlock::Blank => lines.push(String::new()),
        }
    }
    lines.extend(doc.signature_lines.iter().cloned());
    if let (Some(h), Some(t)) = (&doc.certification_heading, &doc.certification_text) {
        lines.push(String::new());
        lines.push(h.clone());
        lines.push(t.clone());
    }
    lines.push(String::new());
    lines.push(doc.certificate_heading.clone());
    lines.extend(doc.certificate_lines.iter().cloned());
    lines
}

pub fn contains_forbidden_markup(text: &str) -> bool {
    text.contains(":::") || text.contains("#v(") || text.lines().any(|l| {
        let t = l.trim();
        t.starts_with('#') && !t.starts_with("#set") // leftover markdown heading
            && !t.starts_with("#align")
            && t.chars().nth(1).map(|c| c == ' ' || c.is_ascii_alphabetic()).unwrap_or(false)
    })
}

pub fn to_typst(doc: &CourtDocument, rules: &CourtRules) -> String {
    let fonts = rules.typst_font_list();
    let size = rules.font_size_pt;
    let leading = if rules.line_spacing >= 1.8 { 1.5 } else { rules.line_spacing.max(0.8) };
    let caption = match doc.style {
        CaptionStyle::Boxed => boxed_caption_typst(doc),
        CaptionStyle::Classic => classic_caption_typst(doc),
    };
    let mut body = String::new();
    for block in &doc.body {
        match block {
            BodyBlock::Heading(h) => {
                body.push_str(&format!(
                    "\n#block(above: 0.9em, below: 0.35em)[#text(weight: \"bold\")[{}]]\n",
                    typst_escape(&h.to_ascii_uppercase())
                ));
            }
            BodyBlock::Paragraph(p) => {
                body.push_str(&format!("{}\n\n", typst_escape(p)));
            }
            BodyBlock::Blank => body.push('\n'),
        }
    }
    let mut closing = String::new();
    closing.push_str("\n#v(1.1em)\n");
    for line in &doc.signature_lines {
        closing.push_str(&format!("#text[{}] \\\n", typst_escape(line)));
    }
    if let (Some(h), Some(t)) = (&doc.certification_heading, &doc.certification_text) {
        closing.push_str(&format!(
            "\n#v(1.2em)\n#align(center)[#text(weight: \"bold\")[{}]]\n#v(0.4em)\n{}\n",
            typst_escape(h),
            typst_escape(t)
        ));
    }
    closing.push_str(&format!(
        "\n#pagebreak(weak: true)\n#align(center)[#text(weight: \"bold\")[{}]]\n#v(0.5em)\n",
        typst_escape(&doc.certificate_heading)
    ));
    for line in &doc.certificate_lines {
        closing.push_str(&format!("{}\n\n", typst_escape(line)));
    }

    format!(
        r##"
#set page(
  paper: "{paper}",
  margin: (top: 0.75in, right: 0.85in, bottom: 0.85in, left: 0.85in),
  footer: context {{
    align(center)[#text(size: 10pt)[Page #counter(page).display() of #counter(page).final().first()]]
  }}
)
#set text(font: ({fonts}), size: {size}pt)
#set par(leading: {leading}em, justify: true, first-line-indent: 0em)

{caption}

#v(0.55em)
#block(stroke: 0.8pt, width: 100%, inset: 8pt)[
  #align(center)[#text(weight: "bold", size: {title_size}pt)[{title}]]
]
#v(0.7em)

{body}
{closing}
"##,
        paper = rules.paper,
        fonts = fonts,
        size = size,
        leading = leading,
        caption = caption,
        title_size = size,
        title = typst_escape(&doc.title),
        body = body,
        closing = closing,
    )
}

fn boxed_caption_typst(doc: &CourtDocument) -> String {
    let mut left = String::new();
    left.push_str(&format!(
        "#text(weight: \"bold\", size: 11pt)[{}]\\ \n",
        typst_escape(&doc.court_name)
    ));
    if !doc.court_division.is_empty() {
        left.push_str(&format!("{}\\ \n", typst_escape(&doc.court_division)));
    }
    for a in &doc.court_address {
        if !a.trim().is_empty() {
            left.push_str(&format!("#text(size: 9pt)[{}]\\ \n", typst_escape(a)));
        }
    }
    left.push_str("#v(0.35em)\n");
    left.push_str(&format!(
        "{}: {}\\ \nv.\\ \n{}: {}\\ \n",
        typst_escape(&doc.left_label),
        typst_escape(&doc.left_names),
        typst_escape(&doc.right_label),
        typst_escape(&doc.right_names)
    ));
    left.push_str("#v(0.3em)\n#set text(size: 9pt)\n");
    left.push_str(&format!(
        "{}, {}\\ \n",
        typst_escape(&doc.filer_name),
        typst_escape(&doc.filer_role)
    ));
    for a in &doc.filer_address {
        if !a.trim().is_empty() {
            left.push_str(&format!("{}\\ \n", typst_escape(a)));
        }
    }
    let mut contact = Vec::new();
    if !doc.filer_email.is_empty() {
        contact.push(doc.filer_email.clone());
    }
    if !doc.filer_phone.is_empty() {
        contact.push(doc.filer_phone.clone());
    }
    if !contact.is_empty() {
        left.push_str(&format!("{}\\ \n", typst_escape(&contact.join(" · "))));
    }

    let right = boxed_right_typst(doc);

    format!(
        r##"
#table(
  columns: (1fr, 2.35in),
  stroke: 0.8pt,
  inset: 6pt,
  [
    #set par(leading: 0.38em, justify: false)
    #set text(size: 9.5pt)
    {left}
  ],
  [
    #set par(leading: 0.38em, justify: false)
    #set text(size: 9pt)
    {right}
  ]
)
"##,
        left = left,
        right = right,
    )
}

fn blank_or(value: &str) -> String {
    if value.trim().is_empty() {
        "_______________".into()
    } else {
        value.to_string()
    }
}

fn boxed_right_typst(doc: &CourtDocument) -> String {
    match doc.venue {
        CaptionVenue::Colorado => {
            let case_no = typst_escape(&blank_or(&doc.appellate_no));
            let lower = typst_escape(&blank_or(&doc.lower_court_name));
            let trial = typst_escape(&blank_or(&doc.trial_no));
            format!(
                r##"#align(center)[#text(weight: "bold", size: 8pt)[▲ COURT USE ONLY ▲]]
    #v(0.25em)
    #box(stroke: 0.5pt, width: 100%, inset: 6pt)[
      *Case Number:* {case_no} \
      Lower Court: {lower} \
      Lower Court Case: {trial}
    ]"##,
                case_no = case_no,
                lower = lower,
                trial = trial,
            )
        }
        CaptionVenue::NewJersey | CaptionVenue::Generic => {
            let appellate = typst_escape(&blank_or(&doc.appellate_no));
            let trial = typst_escape(&blank_or(&doc.trial_no));
            let div = typst_escape(&blank_or(&doc.division_courtroom));
            let heading = if doc.venue == CaptionVenue::NewJersey {
                "COURT USE ONLY▲"
            } else {
                "COURT USE ONLY"
            };
            format!(
                r##"#align(center)[#text(weight: "bold", size: 8pt)[{heading}]]
    #v(0.25em)
    #box(stroke: 0.5pt, width: 100%, inset: 6pt)[
      *Case Number* \
      Appellate: {appellate} \
      Trial Court: {trial} \
      Division / Courtroom: {div}
    ]"##,
                heading = heading,
                appellate = appellate,
                trial = trial,
                div = div,
            )
        }
    }
}

fn classic_caption_typst(doc: &CourtDocument) -> String {
    let mut s = String::from("#align(center)[\n#set par(leading: 0.55em, justify: false)\n");
    s.push_str(&format!(
        "#text(weight: \"bold\")[{}]\\ \n",
        typst_escape(&doc.court_name)
    ));
    if !doc.court_division.is_empty() {
        s.push_str(&format!("{}\\ \n", typst_escape(&doc.court_division)));
    }
    s.push_str("#v(0.4em)\n");
    s.push_str(&format!("{}\\ \n", typst_escape(&doc.left_names)));
    s.push_str(&format!("{}(s),\\ \n", typst_escape(&doc.left_label)));
    s.push_str("v.\\ \n");
    s.push_str(&format!("{}\\ \n", typst_escape(&doc.right_names)));
    s.push_str(&format!("{}(s).\\ \n", typst_escape(&doc.right_label)));
    if !doc.appellate_no.is_empty() {
        s.push_str(&format!("Case No. {}\\ \n", typst_escape(&doc.appellate_no)));
    }
    s.push_str("]\n");
    s
}

fn para_xml(text: &str, bold: bool, center: bool, size: u32, spacing: u32, font: &str) -> String {
    let align = if center { "center" } else { "both" };
    let b = if bold { "<w:b/>" } else { "" };
    format!(
        r#"<w:p>
  <w:pPr>
    <w:jc w:val="{align}"/>
    <w:spacing w:line="{spacing}" w:lineRule="auto"/>
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
        spacing = spacing,
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
    let div = blank_or(&doc.division_courtroom);
    let lower = blank_or(&doc.lower_court_name);
    let right_text = match doc.venue {
        CaptionVenue::Colorado => format!(
            "▲ COURT USE ONLY ▲\n\nCase Number: {appellate}\nLower Court: {lower}\nLower Court Case: {trial}"
        ),
        CaptionVenue::NewJersey => format!(
            "COURT USE ONLY▲\n\nCase Number\nAppellate: {appellate}\nTrial Court: {trial}\nDivision / Courtroom: {div}"
        ),
        CaptionVenue::Generic => format!(
            "COURT USE ONLY\n\nCase Number\nAppellate: {appellate}\nTrial Court: {trial}\nDivision / Courtroom: {div}"
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
                body.push_str(&para_xml(&h.to_ascii_uppercase(), true, false, size, spacing, &font));
            }
            BodyBlock::Paragraph(p) => {
                body.push_str(&para_xml(p, false, false, size, spacing, &font));
            }
            BodyBlock::Blank => {
                body.push_str(&para_xml("", false, false, size, spacing, &font));
            }
        }
    }
    for line in &doc.signature_lines {
        body.push_str(&para_xml(line, false, false, size, spacing, &font));
    }
    if let (Some(h), Some(t)) = (&doc.certification_heading, &doc.certification_text) {
        body.push_str(&para_xml(h, true, true, size, spacing, &font));
        body.push_str(&para_xml(t, false, false, size, spacing, &font));
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_profile::sample_appeal_profile;

    #[test]
    fn sanitize_strips_markdown_fences_and_typst() {
        let md = "# Preliminary statement\nHello.\n::: signature\n/s/ Jordan Example\n:::\n#v(0.5em)\n::: certificate\nI will serve this myself.\n:::";
        let (body, sig, cert) = sanitize_draft(md);
        let headings: Vec<_> = body
            .iter()
            .filter_map(|b| match b {
                BodyBlock::Heading(h) => Some(h.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(headings, ["Preliminary statement"]);
        assert!(sig.iter().any(|s| s.contains("/s/ Jordan Example")));
        assert!(cert.iter().any(|s| s.contains("I will serve")));
        let blob = format!("{body:?}");
        assert!(!blob.contains(":::"));
        assert!(!blob.contains("#v("));
        assert!(!blob.contains("# Preliminary"));
    }

    #[test]
    fn nj_motion_uses_verbatim_r_1_4_4b_and_omits_disclaimer() {
        let mut p = sample_appeal_profile();
        p.rule_pack_id = "nj-app-div".into();
        p.governing_state = "NJ".into();
        p.court.name = "Superior Court of New Jersey, Appellate Division".into();
        p.docket_number = "A-01000-24".into();
        p.notice_of_appeal.trial_court_docket = "L-01000-24".into();
        p.notice_of_appeal.trial_court_name =
            "Superior Court of New Jersey, Law Division, Sample County".into();
        p.motion.user_text = "# Preliminary statement\nThe trial court entered an order.\n::: signature\n/s/ Jordan Example\n:::".into();
        let doc = assemble(&p, "reconsideration", "Motion for reconsideration", &p.motion.user_text, CaptionStyle::Boxed);
        let lines = document_lines(&doc).join("\n");
        assert!(lines.contains(NJ_R_1_4_4B_OATH));
        assert!(!lines.to_ascii_lowercase().contains("to the best of my knowledge"));
        assert!(!lines.contains(REVIEW_NOTICE));
        assert!(!lines.contains("not legal advice"));
        assert!(!contains_forbidden_markup(&lines));
        assert!(lines.contains("COURT USE ONLY") || lines.contains("Appellate"));
        assert!(lines.contains("A-01000-24") || lines.contains(&p.docket_number));
        assert!(lines.contains("CERTIFICATE OF SERVICE"));
        assert!(lines.contains("/s/ Jordan Example"));
    }

    #[test]
    fn co_motion_uses_lower_court_box_and_crs_declaration() {
        let mut p = sample_appeal_profile();
        p.governing_state = "CO".into();
        p.court.name = "Colorado Court of Appeals".into();
        p.court.address_lines = vec!["2 East 14th Avenue".into(), "Denver, CO 80203".into()];
        p.docket_number = "24CA1000".into();
        p.notice_of_appeal.trial_court_name = "Denver District Court".into();
        p.notice_of_appeal.trial_court_docket = "23CV1000".into();
        p.caption.appellants[0].role = "Appellant".into();
        p.caption.appellees[0].role = "Appellee".into();
        p.motion.user_text = include_str!("../../fixtures/sample-co-reconsideration.md")
            .split("\n---\n")
            .last()
            .unwrap_or("The district court entered an order.")
            .to_string();
        let doc = assemble(
            &p,
            "motion",
            "Motion for reconsideration",
            &p.motion.user_text,
            CaptionStyle::Boxed,
        );
        assert_eq!(doc.venue, CaptionVenue::Colorado);
        let lines = document_lines(&doc).join("\n");
        assert!(lines.contains("24CA1000"));
        assert!(lines.contains("Denver District Court"));
        assert!(lines.contains("23CV1000"));
        assert!(lines.contains("Lower Court"));
        assert!(lines.contains("COURT USE ONLY"));
        assert!(lines.contains(legal_os::CO_CRS_13_27_106_OATH));
        assert!(lines.contains("CERTIFICATE OF SERVICE"));
        assert!(lines.contains("/s/ Jordan Example"));
        assert!(!contains_forbidden_markup(&lines));
        assert!(!lines.to_ascii_lowercase().contains("to the best of my knowledge"));
        assert!(!lines.contains(REVIEW_NOTICE));
    }
}
