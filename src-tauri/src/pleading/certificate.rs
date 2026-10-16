use crate::case_profile::CaseProfile;
use crate::legal_os::NJ_R_1_4_4B_OATH;
use super::types::{caption_venue, CaptionVenue, CourtDocument, CERTIFICATE_BLANK};

pub fn certificate_sentence(date: &str, paper: &str, recipient: &str, method: &str) -> String {
    format!("I certify that on {date} I served a copy of this {paper} on {recipient} by {method}.")
}

fn find_ci(haystack: &str, needle: &str) -> Option<usize> {
    haystack.to_ascii_lowercase().find(&needle.to_ascii_lowercase())
}

/// Parse "I certify that on [date] I served a copy of this [paper] on [recipient] by [method]."
pub fn parse_served_sentence(line: &str) -> Option<(String, String, String, String)> {
    let lower = line.to_ascii_lowercase();
    let prefix = "i certify that on ";
    let mid = " i served a copy of this ";
    let start = find_ci(&lower, prefix)?;
    let after_on = start + prefix.len();
    let mid_at = find_ci(&lower[after_on..], mid)? + after_on;
    let date = line[after_on..mid_at].trim().trim_end_matches(',').trim();
    let after_this = mid_at + mid.len();
    let on_at = find_ci(&lower[after_this..], " on ")? + after_this;
    let paper = line[after_this..on_at].trim();
    let after_recip = on_at + " on ".len();
    let by_at = find_ci(&lower[after_recip..], " by ")? + after_recip;
    let recipient = line[after_recip..by_at].trim();
    let method_raw = line[by_at + " by ".len()..].trim();
    let method = method_raw
        .split('.')
        .next()
        .unwrap_or(method_raw)
        .trim()
        .trim_end_matches(',');
    Some((
        date.to_string(),
        paper.to_string(),
        recipient.to_string(),
        method.to_string(),
    ))
}

pub fn is_fill_in_blank(value: &str) -> bool {
    let t = value.trim();
    t.is_empty() || t.chars().all(|c| c == '_' || c == '—' || c == '-' || c == ' ')
}

pub fn labeled_value<'a>(line: &'a str, labels: &[&str]) -> Option<&'a str> {
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

pub fn is_date_label_line(line: &str) -> bool {
    let t = line.trim().to_ascii_lowercase();
    t.starts_with("date:") || t.starts_with("date ") || t.starts_with("service date")
}

pub fn date_line_has_value(line: &str) -> bool {
    labeled_value(line, &["Date", "Service date", "Date of service"]).is_some()
        && line.chars().any(|c| c.is_ascii_digit())
}

pub fn filled_cert_value(value: &str) -> Option<String> {
    let t = value.trim();
    if is_fill_in_blank(t) {
        return None;
    }
    Some(t.to_string())
}

#[derive(Debug, Clone, Default)]
pub struct CertificateFacts {
    pub recipient: Option<String>,
    pub method: Option<String>,
    pub date: Option<String>,
    pub extras: Vec<String>,
}

pub fn is_future_service_line(line: &str) -> bool {
    let t = line.to_ascii_lowercase();
    t.contains("i will serve") || t.contains("will serve a copy")
}

pub fn is_engine_disclaimer_line(line: &str) -> bool {
    let t = line.to_ascii_lowercase();
    t.contains("this software did not") || t.contains("this app did not")
}

pub fn is_blank_cert_label(line: &str) -> bool {
    let t = line.trim().to_ascii_lowercase();
    (t.starts_with("recipient") || t.starts_with("method") || is_date_label_line(line))
        && labeled_value(
            line,
            &[
                "Recipient",
                "Served on",
                "Party served",
                "To",
                "Method",
                "Manner",
                "By",
                "Date",
                "Service date",
                "Date of service",
            ],
        )
        .is_none()
}

pub fn extract_certificate_facts(lines: &[String]) -> CertificateFacts {
    let mut facts = CertificateFacts::default();
    for l in lines {
        if is_engine_disclaimer_line(l) {
            continue;
        }
        if let Some((date, _paper, recipient, method)) = parse_served_sentence(l) {
            if let Some(v) = filled_cert_value(&date) {
                if v.chars().any(|c| c.is_ascii_digit()) {
                    facts.date = Some(v);
                }
            }
            if let Some(v) = filled_cert_value(&recipient) {
                facts.recipient = Some(v);
            }
            if let Some(v) = filled_cert_value(&method) {
                facts.method = Some(v);
            }
            continue;
        }
        if is_future_service_line(l) || is_blank_cert_label(l) {
            continue;
        }
        if let Some(v) = labeled_value(l, &["Recipient", "Served on", "Party served", "To"]) {
            facts.recipient = Some(v.to_string());
            continue;
        }
        if let Some(v) = labeled_value(l, &["Method", "Manner", "By"]) {
            facts.method = Some(v.to_string());
            continue;
        }
        if date_line_has_value(l) {
            if let Some(v) = labeled_value(l, &["Date", "Service date", "Date of service"]) {
                facts.date = Some(v.to_string());
            }
            continue;
        }
        if line_has_service_method(l) && facts.method.is_none() {
            facts.method = Some(l.trim().to_string());
            continue;
        }
        if !l.trim().is_empty() {
            facts.extras.push(l.clone());
        }
    }
    facts
}

pub fn line_has_service_method(line: &str) -> bool {
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

pub fn certificate_gaps(lines: &[String]) -> CertificateGaps {
    let facts = extract_certificate_facts(lines);
    CertificateGaps {
        recipient: facts.recipient.is_none(),
        method: facts.method.is_none(),
        date: facts.date.is_none(),
    }
}

pub fn join_en(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [a] => (*a).to_string(),
        [a, b] => format!("{a} and {b}"),
        [first, rest @ ..] => {
            let last = rest[rest.len() - 1];
            let mid = &rest[..rest.len() - 1];
            if mid.is_empty() {
                format!("{first} and {last}")
            } else {
                format!("{first}, {} and {last}", mid.join(", "))
            }
        }
    }
}

pub fn require_complete_certificate(doc: &CourtDocument) -> Result<(), String> {
    let gaps = certificate_gaps(&doc.certificate_lines);
    if gaps.is_empty() {
        if doc
            .certificate_lines
            .iter()
            .any(|l| is_future_service_line(l))
        {
            return Err(
                "Cannot export: the certificate of service uses future tense (\"I will serve\"). State completed service: \"I certify that on [date] I served a copy of this [paper] on [recipient] by [method].\""
                    .into(),
            );
        }
        Ok(())
    } else {
        let list = join_en(&gaps.missing_labels());
        Err(format!(
            "Cannot export: the certificate of service is missing the {list}. Add the {list} in the ::: certificate block. Preview may show a blank fill-in form; export will not invent those facts."
        ))
    }
}

pub fn default_certificate_lines(paper: &str) -> Vec<String> {
    vec![certificate_sentence(
        CERTIFICATE_BLANK,
        paper,
        CERTIFICATE_BLANK,
        CERTIFICATE_BLANK,
    )]
}

pub fn polish_certificate(profile: &CaseProfile, lines: Vec<String>, paper: &str) -> Vec<String> {
    let facts = extract_certificate_facts(&lines);
    let sentence = certificate_sentence(
        facts.date.as_deref().unwrap_or(CERTIFICATE_BLANK),
        paper,
        facts.recipient.as_deref().unwrap_or(CERTIFICATE_BLANK),
        facts.method.as_deref().unwrap_or(CERTIFICATE_BLANK),
    );
    let mut out = vec![sentence];
    for extra in facts.extras {
        if extra.contains("/s/") || extra.to_ascii_lowercase().starts_with("/s/") {
            out.push(extra);
        }
    }
    if !out.iter().any(|l| l.contains("/s/")) {
        let sig = if profile.filer.signature_name.trim().is_empty() {
            profile.filer.name.trim()
        } else {
            profile.filer.signature_name.trim()
        };
        if !sig.is_empty() {
            out.push(format!("/s/ {sig}"));
        }
    }
    out
}

/// NJ R. 1:4-4(b) is pack_source and is inserted by the engine.
/// Colorado UUDA / C.R.S. § 13-27-106 is needs_verification: never auto-assert it.
pub fn venue_certification(profile: &CaseProfile) -> Option<(String, String)> {
    match caption_venue(profile) {
        CaptionVenue::NewJersey => Some((
            "CERTIFICATION IN LIEU OF OATH (R. 1:4-4(b))".into(),
            NJ_R_1_4_4B_OATH.to_string(),
        )),
        CaptionVenue::Colorado | CaptionVenue::Generic => None,
    }
}
