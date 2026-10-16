use crate::case_profile::{Caption, CaseProfile};
use super::types::{party_side_label, pleading_body, pleading_heading};

pub(crate) fn assembled_caption_lines(profile: &CaseProfile, kind: &str) -> Vec<String> {
    let left = profile.caption.appellants();
    let right = profile.caption.appellees();
    let tier = profile.effective_tier();
    let key = kind.trim().to_ascii_lowercase();
    let mut lines = Vec::new();

    if tier == "transactional" {
        let state_label = if !profile.governing_state.trim().is_empty() {
            format!("STATE OF {}", profile.governing_state.trim().to_uppercase())
        } else {
            "TRANSACTIONAL LEGAL INSTRUMENT".to_string()
        };
        lines.push(state_label);
        lines.push(String::new());
        if key == "will" || key == "power_of_attorney" {
            lines.push(format!("DECLARANT / TESTATOR: {}", Caption::party_names(&left)));
            if !right.is_empty() {
                lines.push(format!("FIDUCIARY / BENEFICIARY: {}", Caption::party_names(&right)));
            }
        } else {
            lines.push(format!("FIRST PARTY: {}", Caption::party_names(&left)));
            lines.push(format!("COUNTERPARTY: {}", Caption::party_names(&right)));
        }
        lines.push(format!("INSTRUMENT: {}", profile.title));
        lines.push(String::new());
        return lines;
    }

    if tier == "administrative" {
        lines.push("FORMAL STATUTORY NOTICE & PRE-SUIT DEMAND".to_string());
        if !profile.court.name.trim().is_empty() {
            lines.push(format!("AGENCY / VENUE: {}", profile.court.name));
        }
        lines.push(String::new());
        lines.push(format!("FROM (CLAIMANT / SENDER): {}", Caption::party_names(&left)));
        lines.push(format!("TO (RESPONDENT / RECIPIENT): {}", Caption::party_names(&right)));
        lines.push(format!("RE: {}", profile.title));
        if !profile.docket_number.trim().is_empty() {
            lines.push(format!("REFERENCE / CLAIM NO.: {}", profile.docket_number));
        }
        lines.push(String::new());
        return lines;
    }

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
    let docket_display = if profile.docket_number.trim().is_empty() {
        "__________ (To Be Assigned)".to_string()
    } else {
        profile.docket_number.clone()
    };
    lines.push(format!("Case No. {}", docket_display));
    lines.push(String::new());
    lines
}

pub(crate) fn assembled_signature_lines(profile: &CaseProfile) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(crate::pleading::DATE_BLANK.to_string());
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
    lines
}

pub(crate) fn push_user_paragraphs(lines: &mut Vec<String>, text: &str) {
    for paragraph in text.split("\n\n") {
        let trimmed = paragraph.trim();
        if !trimmed.is_empty() {
            lines.push(trimmed.to_string());
            lines.push(String::new());
        }
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

    let (subtitle, body) = pleading_body(profile, &key);
    let title = if subtitle.trim().is_empty() {
        pleading_heading(&key).to_string()
    } else {
        subtitle.trim().to_string()
    };
    let style = crate::pleading::caption_style(profile, &key, None);
    let doc = crate::pleading::assemble(profile, &key, &title, body, style);
    crate::pleading::document_lines(&doc)
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
