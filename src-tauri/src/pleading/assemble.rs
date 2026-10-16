use crate::case_profile::{Caption, CaseProfile};
use super::certificate::{
    date_line_has_value, default_certificate_lines, is_date_label_line, polish_certificate,
    venue_certification,
};
use super::sanitize::sanitize_draft;
use super::types::{
    caption_venue, division_if_distinct, paper_noun, BodyBlock, CaptionStyle, CaptionVenue,
    CourtDocument, DATE_BLANK,
};

pub fn party_names(profile: &CaseProfile, left: bool) -> (String, String) {
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
    let paper = paper_noun(&heading);
    let certificate_lines = polish_certificate(
        profile,
        if cert_extra.is_empty() {
            default_certificate_lines(paper)
        } else {
            cert_extra
                .into_iter()
                .filter(|l| !l.to_ascii_lowercase().contains("to the best of my knowledge"))
                .collect()
        },
        paper,
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
