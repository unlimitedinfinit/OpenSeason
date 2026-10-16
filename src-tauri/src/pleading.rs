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

/// Omit a division line when it is empty or already named in the court title
/// (e.g. "Appellate Division" on "Superior Court of New Jersey, Appellate Division").
pub fn division_if_distinct(court_name: &str, division: &str) -> String {
    let d = division.trim();
    if d.is_empty() {
        return String::new();
    }
    let name = court_name.to_ascii_lowercase();
    let div = d.to_ascii_lowercase();
    if name.contains(&div) {
        return String::new();
    }
    d.to_string()
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
        let cleaned = strip_verification_flags(&s.replace("**", "").replace("__", ""));
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

pub const DATE_BLANK: &str = "Date: ____________";
pub const CERTIFICATE_EXPORT_BLOCKED: &str = "Cannot export: the certificate of service is missing a recipient, a method, or a service date. Add them in the ::: certificate block (Recipient, Method, Date). Preview may show a blank fill-in form; export will not invent those facts. This software did not serve anyone.";

fn strip_verification_flags(s: &str) -> String {
    let mut t = s.to_string();
    loop {
        let lower = t.to_ascii_lowercase();
        let pat = if lower.contains("(needs_verification)") {
            "(needs_verification)"
        } else if lower.contains("needs_verification") {
            "needs_verification"
        } else {
            break;
        };
        if let Some(idx) = lower.find(pat) {
            t.replace_range(idx..idx + pat.len(), "");
        } else {
            break;
        }
    }
    while t.contains("  ") {
        t = t.replace("  ", " ");
    }
    t.replace(" .", ".").replace(" ,", ",").trim().to_string()
}

fn is_fill_in_blank(value: &str) -> bool {
    let t = value.trim();
    t.is_empty() || t.chars().all(|c| c == '_' || c == '—' || c == '-' || c == ' ')
}

fn labeled_value<'a>(line: &'a str, labels: &[&str]) -> Option<&'a str> {
    let trimmed = line.trim();
    let lower = trimmed.to_ascii_lowercase();
    for label in labels {
        let needle = label.to_ascii_lowercase();
        if lower.starts_with(&needle) {
            let rest = trimmed[label.len()..].trim();
            let rest = rest.strip_prefix(':').unwrap_or(rest).trim();
            if is_fill_in_blank(rest) {
                return None;
            }
            return Some(rest);
        }
    }
    None
}

fn is_date_label_line(line: &str) -> bool {
    let t = line.trim().to_ascii_lowercase();
    t.starts_with("date:") || t.starts_with("date ") || t.starts_with("service date")
}

fn date_line_has_value(line: &str) -> bool {
    labeled_value(line, &["Date", "Service date", "Date of service"]).is_some()
        && line.chars().any(|c| c.is_ascii_digit())
}

#[derive(Debug, Clone, Default)]
pub struct CertificateGaps {
    pub recipient: bool,
    pub method: bool,
    pub date: bool,
}

impl CertificateGaps {
    pub fn is_empty(&self) -> bool {
        !self.recipient && !self.method && !self.date
    }

    pub fn missing_labels(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if self.recipient {
            v.push("recipient");
        }
        if self.method {
            v.push("method");
        }
        if self.date {
            v.push("service date");
        }
        v
    }
}

fn line_has_service_method(line: &str) -> bool {
    let t = line.to_ascii_lowercase();
    [
        "electronic mail",
        "e-mail",
        "email",
        "certified mail",
        "first-class mail",
        "u.s. mail",
        "us mail",
        "personal service",
        "hand delivery",
        "hand-delivery",
        "e-file",
        "efile",
        "electronic filing",
        "overnight",
        "facsimile",
        "fax",
        "regular mail",
    ]
    .iter()
    .any(|m| t.contains(m))
}

pub fn certificate_gaps(lines: &[String]) -> CertificateGaps {
    let has_recipient = lines.iter().any(|l| {
        labeled_value(l, &["Recipient", "Served on", "Party served", "To"]).is_some()
    });
    let has_method = lines.iter().any(|l| {
        labeled_value(l, &["Method", "Manner", "By"]).is_some() || line_has_service_method(l)
    });
    let has_date = lines.iter().any(|l| date_line_has_value(l));
    CertificateGaps {
        recipient: !has_recipient,
        method: !has_method,
        date: !has_date,
    }
}

pub fn require_complete_certificate(doc: &CourtDocument) -> Result<(), String> {
    let gaps = certificate_gaps(&doc.certificate_lines);
    if gaps.is_empty() {
        Ok(())
    } else if gaps.missing_labels().len() == 3 {
        Err(CERTIFICATE_EXPORT_BLOCKED.into())
    } else {
        Err(format!(
            "Cannot export: the certificate of service is missing {}. Add them in the ::: certificate block (Recipient, Method, Date). Preview may show a blank fill-in form; export will not invent those facts. This software did not serve anyone.",
            gaps.missing_labels().join(", ")
        ))
    }
}

fn default_certificate_lines() -> Vec<String> {
    vec!["I will serve a copy of this paper myself. This software did not file or serve anyone.".into()]
}

fn polish_certificate(profile: &CaseProfile, mut lines: Vec<String>) -> Vec<String> {
    let gaps = certificate_gaps(&lines);
    if gaps.recipient && !lines.iter().any(|l| l.to_ascii_lowercase().starts_with("recipient")) {
        lines.push("Recipient: ____________".into());
    }
    if gaps.method && !lines.iter().any(|l| l.to_ascii_lowercase().starts_with("method")) {
        lines.push("Method: ____________".into());
    }
    if gaps.date && !lines.iter().any(|l| is_date_label_line(l)) {
        lines.push(DATE_BLANK.into());
    }
    if !lines.iter().any(|l| l.contains("/s/")) {
        let sig = if profile.filer.signature_name.trim().is_empty() {
            profile.filer.name.trim()
        } else {
            profile.filer.signature_name.trim()
        };
        if !sig.is_empty() {
            lines.push(format!("/s/ {sig}"));
        }
    }
    lines
}

/// NJ R. 1:4-4(b) is pack_source and is inserted by the engine.
/// Colorado UUDA / C.R.S. § 13-27-106 is needs_verification: never auto-assert it.
fn venue_certification(profile: &CaseProfile) -> Option<(String, String)> {
    match caption_venue(profile) {
        CaptionVenue::NewJersey => Some((
            "CERTIFICATION IN LIEU OF OATH (R. 1:4-4(b))".into(),
            NJ_R_1_4_4B_OATH.to_string(),
        )),
        CaptionVenue::Colorado | CaptionVenue::Generic => None,
    }
}

fn looks_like_phone(s: &str) -> bool {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.len() >= 10 && digits.len() <= 15
}

fn looks_like_address(s: &str) -> bool {
    let l = s.to_ascii_lowercase();
    if l.contains('@') || l.starts_with("/s/") {
        return false;
    }
    l.contains("street")
        || l.contains(" st.")
        || l.contains(" avenue")
        || l.contains(" ave")
        || l.contains("p.o.")
        || l.contains("po box")
        || (s.contains(',') && s.chars().any(|c| c.is_ascii_digit()))
}

fn merge_filer_contact(profile: &CaseProfile, sig: &[String]) -> (Vec<String>, String, String) {
    let mut addr = profile.filer.address_lines.clone();
    let mut email = profile.filer.email.trim().to_string();
    let mut phone = profile.filer.phone.trim().to_string();
    for line in sig {
        let t = line.trim();
        if t.is_empty() || t.contains("/s/") {
            continue;
        }
        if t.contains('@') {
            if email.is_empty() {
                email = t.to_string();
            }
        } else if looks_like_phone(t) {
            if phone.is_empty() {
                phone = t.to_string();
            }
        } else if looks_like_address(t) && !addr.iter().any(|a| a.trim() == t) {
            addr.push(t.to_string());
        }
    }
    (
        addr.into_iter().filter(|a| !a.trim().is_empty()).collect(),
        email,
        phone,
    )
}

fn signature_block(profile: &CaseProfile, extra: &[String]) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(date) = extra.iter().find(|l| is_date_label_line(l)) {
        if date_line_has_value(date) {
            lines.push(date.clone());
        } else {
            lines.push(DATE_BLANK.into());
        }
    } else {
        lines.push(DATE_BLANK.into());
    }
    lines.push("Respectfully submitted,".into());
    let extra: Vec<String> = extra
        .iter()
        .filter(|l| !is_date_label_line(l))
        .cloned()
        .collect();
    if extra.iter().any(|l| l.contains("/s/")) {
        lines.extend(extra);
    } else {
        lines.push(format!("/s/ {}", profile.filer.signature_name));
        lines.push(profile.filer.name.clone());
        if !profile.filer.role.trim().is_empty() {
            lines.push(profile.filer.role.clone());
        }
        lines.extend(extra);
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
    let (body, sig_extra, cert_extra, decl_extra) = sanitize_draft(user_text);
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
    let venue = caption_venue(profile);
    let (cert_h, cert_t) = match venue {
        CaptionVenue::NewJersey => match venue_certification(profile) {
            Some(pair) => (Some(pair.0), Some(pair.1)),
            None => (None, None),
        },
        _ if !decl_extra.is_empty() => (
            Some("Declaration".into()),
            Some(decl_extra.join(" ")),
        ),
        _ => (None, None),
    };
    let certificate_lines = polish_certificate(
        profile,
        if cert_extra.is_empty() {
            default_certificate_lines()
        } else {
            cert_extra
                .into_iter()
                .filter(|l| !l.to_ascii_lowercase().contains("to the best of my knowledge"))
                .collect()
        },
    );
    let distinct_div = division_if_distinct(
        &profile.court.name,
        profile.court.division.as_deref().unwrap_or(""),
    );
    let (filer_address, filer_email, filer_phone) = merge_filer_contact(profile, &sig_extra);
    CourtDocument {
        style,
        court_name: profile.court.name.clone(),
        court_division: distinct_div.clone(),
        court_address: profile.court.address_lines.clone(),
        left_label,
        left_names,
        right_label,
        right_names,
        appellate_no: profile.docket_number.clone(),
        trial_no: profile.notice_of_appeal.trial_court_docket.clone(),
        division_courtroom: distinct_div,
        filer_name: profile.filer.name.clone(),
        filer_role: if profile.filer.role.trim().is_empty() {
            "Pro Se".into()
        } else {
            profile.filer.role.clone()
        },
        filer_address,
        filer_email,
        filer_phone,
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
                lines.push(format!("Case Number: {}", doc.appellate_no));
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
    // Certification / declaration sits above the signature so one date and /s/
    // close the paper. The certificate of service is its own last page.
    if let (Some(h), Some(t)) = (&doc.certification_heading, &doc.certification_text) {
        lines.push(String::new());
        lines.push(h.clone());
        lines.push(t.clone());
        lines.push(String::new());
    }
    lines.extend(doc.signature_lines.iter().cloned());
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
    // Typst `leading` is the gap between line boxes. Map Word multipliers onto a
    // consistent gap so wrapped lines match other lines in the same paragraph.
    let leading = if rules.line_spacing >= 1.8 {
        0.85
    } else if rules.line_spacing >= 1.4 {
        0.55
    } else {
        0.35
    };
    let caption = match doc.style {
        CaptionStyle::Boxed => boxed_caption_typst(doc),
        CaptionStyle::Classic => classic_caption_typst(doc),
    };
    let mut body = String::new();
    for block in &doc.body {
        match block {
            BodyBlock::Heading(h) => {
                body.push_str(&format!(
                    "\n#block(above: 1.1em, below: 0.85em)[#text(weight: \"bold\")[{}]]\n",
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
    closing.push_str("\n#block(breakable: false)[\n#set par(leading: 0.35em, justify: false, first-line-indent: 0em)\n#v(0.7em)\n");
    if let (Some(h), Some(t)) = (&doc.certification_heading, &doc.certification_text) {
        closing.push_str(&format!(
            "#align(center)[#text(weight: \"bold\")[{}]]\n#v(0.4em)\n#set par(leading: 0.55em, justify: true)\n{}\n#v(0.75em)\n#set par(leading: 0.35em, justify: false)\n",
            typst_escape(h),
            typst_escape(t)
        ));
    }
    for line in &doc.signature_lines {
        closing.push_str(&format!("{} \\\n", typst_escape(line)));
    }
    closing.push_str("]\n");
    closing.push_str(&format!(
        "\n#pagebreak()\n#align(center)[#text(weight: \"bold\")[{}]]\n#v(0.5em)\n",
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
#set par(leading: {leading}em, spacing: {leading}em, justify: true, first-line-indent: 0em)

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
    let case_no = typst_escape(&blank_or(&doc.appellate_no));
    match doc.venue {
        CaptionVenue::Colorado => {
            let lower = typst_escape(&blank_or(&doc.lower_court_name));
            let trial = typst_escape(&blank_or(&doc.trial_no));
            format!(
                r##"#align(center)[#text(weight: "bold", size: 8pt)[COURT USE ONLY]]
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
            let trial = typst_escape(&blank_or(&doc.trial_no));
            let division_line = if doc.division_courtroom.trim().is_empty() {
                String::new()
            } else {
                format!(
                    " \\\n      Division / Courtroom: {}",
                    typst_escape(&doc.division_courtroom)
                )
            };
            format!(
                r##"#align(center)[#text(weight: "bold", size: 8pt)[COURT USE ONLY]]
    #v(0.25em)
    #box(stroke: 0.5pt, width: 100%, inset: 6pt)[
      *Case Number:* {case_no} \
      Trial Court: {trial}{division_line}
    ]"##,
                case_no = case_no,
                trial = trial,
                division_line = division_line,
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
    let align = if center { "center" } else { "both" };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_profile::sample_appeal_profile;

    #[test]
    fn sanitize_strips_markdown_fences_and_typst() {
        let md = "# Preliminary statement\nHello.\n::: signature\n/s/ Jordan Example\n:::\n#v(0.5em)\n::: certificate\nI will serve this myself.\n:::";
        let (body, sig, cert, _decl) = sanitize_draft(md);
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
        let cert_at = lines.find("CERTIFICATION IN LIEU OF OATH").unwrap();
        let sig_at = lines.find("/s/ Jordan Example").unwrap();
        let cos_at = lines.find("CERTIFICATE OF SERVICE").unwrap();
        assert!(cert_at < sig_at, "certification belongs above the signature");
        assert!(sig_at < cos_at, "certificate of service follows the signature");
        assert!(!lines.contains("Division / Courtroom"));
        let appellate_hits = lines.matches("Appellate Division").count();
        assert_eq!(appellate_hits, 1, "court name already says Appellate Division");
        let typst = to_typst(&doc, &crate::court_rules::CourtRules::generic());
        assert!(typst.contains("#pagebreak()"));
        assert!(!typst.contains("#pagebreak(weak"));
        assert!(typst.contains("breakable: false"));
        assert!(typst.contains("below: 0.85em"));
        assert!(typst.contains("leading: 0.85em"));
        assert!(typst.contains("spacing: 0.85em"));
        assert!(typst.contains("leading: 0.35em"));
        assert!(lines.contains("Case Number:"));
        assert!(lines.contains(DATE_BLANK));
        assert!(!lines.to_ascii_lowercase().contains("needs_verification"));
    }

    #[test]
    fn filer_address_and_email_appear_in_caption() {
        let mut p = sample_appeal_profile();
        p.governing_state = "NJ".into();
        p.court.name = "Superior Court of New Jersey, Appellate Division".into();
        p.court.division = Some("Appellate Division".into());
        p.filer.address_lines = vec!["100 Sample Street".into(), "Example City, ST 00000".into()];
        p.filer.email = "jordan.example@example.test".into();
        p.filer.phone = "555-0100".into();
        p.motion.user_text = include_str!("../../fixtures/sample-reconsideration.md")
            .split("\n---\n")
            .last()
            .unwrap_or("The trial court entered an order.")
            .to_string();
        let doc = assemble(
            &p,
            "reconsideration",
            "Motion for reconsideration",
            &p.motion.user_text,
            CaptionStyle::Boxed,
        );
        assert!(doc.filer_address.iter().any(|a| a.contains("100 Sample Street")));
        assert_eq!(doc.filer_email, "jordan.example@example.test");
        let lines = document_lines(&doc).join("\n");
        assert!(lines.contains("100 Sample Street"));
        assert!(lines.contains("jordan.example@example.test"));
        assert!(!lines.contains("Division / Courtroom"));
        assert_eq!(doc.court_division, "");
        let typst = to_typst(&doc, &crate::court_rules::CourtRules::generic());
        assert!(typst.contains("100 Sample Street"));
        assert!(
            typst.contains("jordan.example\\@example.test")
                || typst.contains("jordan.example@example.test")
        );
        assert!(!typst.contains("Division / Courtroom"));
    }

    #[test]
    fn co_motion_uses_lower_court_box_and_user_declaration() {
        let mut p = sample_appeal_profile();
        p.governing_state = "CO".into();
        p.court.name = "Colorado Court of Appeals".into();
        p.court.division = Some("Court of Appeals".into());
        p.court.address_lines = vec!["2 East 14th Avenue".into(), "Denver, CO 80203".into()];
        p.docket_number = "24CA1000".into();
        p.notice_of_appeal.trial_court_name = "Denver District Court".into();
        p.notice_of_appeal.trial_court_docket = "23CV1000".into();
        p.caption.appellants[0].role = "Appellant".into();
        p.caption.appellees[0].role = "Appellee".into();
        p.filer.address_lines = vec!["100 Sample Street".into(), "Example City, ST 00000".into()];
        p.filer.email = "jordan.example@example.test".into();
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
        assert!(lines.contains("I declare under penalty of perjury under the law of Colorado"));
        assert!(!lines.to_ascii_lowercase().contains("needs_verification"));
        assert!(!lines.contains("DECLARATION UNDER C.R.S."));
        assert!(!lines.contains("Certificate of Conferral (C.R.C.P. 121"));
        assert!(lines.contains("CERTIFICATE OF SERVICE"));
        assert!(lines.contains("/s/ Jordan Example"));
        assert!(lines.contains("100 Sample Street"));
        assert!(lines.contains("jordan.example@example.test"));
        assert_eq!(doc.court_division, "");
        let decl_at = lines.find("I declare under penalty of perjury").unwrap();
        let sig_at = lines.find("/s/ Jordan Example").unwrap();
        let cos_at = lines.find("CERTIFICATE OF SERVICE").unwrap();
        assert!(decl_at < sig_at);
        assert!(sig_at < cos_at);
        assert!(!contains_forbidden_markup(&lines));
        assert!(!lines.to_ascii_lowercase().contains("to the best of my knowledge"));
        assert!(!lines.contains(REVIEW_NOTICE));
        let typst = to_typst(&doc, &crate::court_rules::CourtRules::generic());
        assert!(typst.contains("#pagebreak()"));
        assert!(!typst.contains("#pagebreak(weak"));
        assert!(typst.contains("leading: 0.85em"));
        assert!(lines.contains("Case Number:"));
        assert!(lines.contains("Recipient: Sample County Clerk"));
        assert!(lines.contains("Method: electronic mail"));
        assert!(lines.contains("Date: October 1, 2026"));
        assert!(doc.signature_lines.iter().any(|l| l == DATE_BLANK));
    }

    #[test]
    fn rendered_paper_omits_needs_verification() {
        for (state, court, md) in [
            (
                "NJ",
                "Superior Court of New Jersey, Appellate Division",
                include_str!("../../fixtures/sample-reconsideration.md"),
            ),
            (
                "CO",
                "Colorado Court of Appeals",
                include_str!("../../fixtures/sample-co-reconsideration.md"),
            ),
        ] {
            let mut p = sample_appeal_profile();
            p.governing_state = state.into();
            p.court.name = court.into();
            p.motion.user_text = md.split("\n---\n").last().unwrap_or(md).to_string();
            let doc = assemble(
                &p,
                "motion",
                "Motion for reconsideration",
                &p.motion.user_text,
                CaptionStyle::Boxed,
            );
            let lines = document_lines(&doc).join("\n").to_ascii_lowercase();
            assert!(
                !lines.contains("needs_verification"),
                "{state} document_lines leaked needs_verification"
            );
            let typst = to_typst(&doc, &crate::court_rules::CourtRules::generic()).to_ascii_lowercase();
            assert!(
                !typst.contains("needs_verification"),
                "{state} typst leaked needs_verification"
            );
        }
    }

    #[test]
    fn signature_date_is_blank_unless_user_entered_one() {
        let mut p = sample_appeal_profile();
        p.governing_state = "NJ".into();
        p.motion.user_text = "# Body\nHello.\n::: signature\n/s/ Jordan Example\n:::".into();
        let doc = assemble(&p, "motion", "Motion", &p.motion.user_text, CaptionStyle::Boxed);
        assert!(doc.signature_lines.iter().any(|l| l == DATE_BLANK));
        let today = chrono::Local::now().format("%B %d, %Y").to_string();
        assert!(
            !doc.signature_lines.iter().any(|l| l.contains(&today)),
            "export date must not be used as the signing date"
        );

        p.motion.user_text =
            "# Body\nHello.\n::: signature\nDate: January 2, 2026\n/s/ Jordan Example\n:::".into();
        let doc = assemble(&p, "motion", "Motion", &p.motion.user_text, CaptionStyle::Boxed);
        assert!(doc.signature_lines.iter().any(|l| l.contains("January 2, 2026")));
        assert!(!doc.signature_lines.iter().any(|l| l == DATE_BLANK));
    }

    #[test]
    fn nj_and_co_share_court_use_only_and_case_number_label() {
        let mut nj = sample_appeal_profile();
        nj.governing_state = "NJ".into();
        nj.court.name = "Superior Court of New Jersey, Appellate Division".into();
        nj.docket_number = "A-01000-24".into();
        let nj_doc = assemble(&nj, "motion", "Motion", "Body.\n::: signature\n/s/ Jordan Example\n:::", CaptionStyle::Boxed);
        let nj_typst = to_typst(&nj_doc, &crate::court_rules::CourtRules::generic());
        assert!(nj_typst.contains("[COURT USE ONLY]"));
        assert!(!nj_typst.contains("COURT USE ONLY▲"));
        assert!(!nj_typst.contains("▲ COURT USE ONLY"));
        assert!(nj_typst.contains("Case Number:"));

        let mut co = sample_appeal_profile();
        co.governing_state = "CO".into();
        co.court.name = "Colorado Court of Appeals".into();
        co.docket_number = "24CA1000".into();
        let co_doc = assemble(&co, "motion", "Motion", "Body.\n::: signature\n/s/ Jordan Example\n:::", CaptionStyle::Boxed);
        let co_typst = to_typst(&co_doc, &crate::court_rules::CourtRules::generic());
        assert!(co_typst.contains("[COURT USE ONLY]"));
        assert!(!co_typst.contains("▲ COURT USE ONLY"));
        assert!(co_typst.contains("Case Number:"));
    }

    #[test]
    fn incomplete_certificate_is_fill_in_for_preview_and_blocks_export() {
        let mut p = sample_appeal_profile();
        p.governing_state = "NJ".into();
        p.motion.user_text = "# Body\nHello.\n::: signature\n/s/ Jordan Example\n:::\n::: certificate\nI will serve this myself.\n:::".into();
        let doc = assemble(&p, "motion", "Motion", &p.motion.user_text, CaptionStyle::Boxed);
        let lines = document_lines(&doc).join("\n");
        assert!(lines.contains("Recipient: ____________"));
        assert!(lines.contains("Method: ____________"));
        assert!(lines.contains(DATE_BLANK));
        let err = require_complete_certificate(&doc).unwrap_err();
        assert!(err.contains("recipient"));
        assert!(err.contains("method"));
        assert!(err.contains("service date"));

        p.motion.user_text = include_str!("../../fixtures/sample-reconsideration.md")
            .split("\n---\n")
            .last()
            .unwrap()
            .to_string();
        let complete = assemble(&p, "motion", "Motion", &p.motion.user_text, CaptionStyle::Boxed);
        require_complete_certificate(&complete).unwrap();
    }

    #[test]
    fn venue_language_marks_co_labels_needs_verification() {
        let raw = include_str!("../../templates/venue-language.json");
        let v: serde_json::Value = serde_json::from_str(raw).unwrap();
        assert_eq!(v["venues"]["NJ"]["oath"]["citation_status"], "pack_source");
        assert_eq!(v["venues"]["CO"]["oath"]["citation_status"], "needs_verification");
        assert_eq!(
            v["venues"]["CO"]["conferral"]["citation_status"],
            "needs_verification"
        );
        assert!(v["venues"]["CO"]["conferral"]["note"]
            .as_str()
            .unwrap()
            .contains("C.A.R."));
        assert!(v["venues"]["CO"]["oath"]["note"]
            .as_str()
            .unwrap()
            .contains("date and place"));
    }
}
