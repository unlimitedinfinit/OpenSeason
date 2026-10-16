use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::case_profile::CaseProfile;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CourtFrontMatter {
    #[serde(default)]
    pub court: String,
    #[serde(default)]
    pub county_or_district: String,
    #[serde(default)]
    pub plaintiff: String,
    #[serde(default)]
    pub plaintiff_label: String,
    #[serde(default)]
    pub defendant: String,
    #[serde(default)]
    pub defendant_label: String,
    #[serde(default)]
    pub case_no: String,
    #[serde(default)]
    pub case_no_label: String,
    #[serde(default)]
    pub judge: String,
    #[serde(default)]
    pub doc_title: String,
    #[serde(default)]
    pub extra_caption: String,
    #[serde(default)]
    pub trial_court: String,
    #[serde(default)]
    pub trial_docket: String,
    #[serde(default)]
    pub court_address: String,
    #[serde(default)]
    pub governing_state: String,
    #[serde(default)]
    pub caption_layout: String,
    #[serde(default)]
    pub filer_email: String,
    #[serde(default)]
    pub filer_phone: String,
    #[serde(default)]
    pub filer_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CourtMarkdownDoc {
    pub meta: CourtFrontMatter,
    pub body: String,
    pub lines: Vec<String>,
}

pub fn parse_front_matter(text: &str) -> (CourtFrontMatter, String) {
    let mut meta = CourtFrontMatter::default();
    let trimmed = text.trim_start_matches('\u{feff}');
    if !trimmed.starts_with("---") {
        return (meta, trimmed.to_string());
    }
    let mut parts = trimmed.splitn(3, '\n');
    let _ = parts.next();
    let rest = trimmed[3..].trim_start_matches('\n');
    if let Some(end) = rest.find("\n---") {
        let yaml = &rest[..end];
        let body = rest[end + 4..].trim_start_matches('\n').to_string();
        for line in yaml.lines() {
            if let Some((k, v)) = line.split_once(':') {
                let key = k.trim();
                let val = v.trim().trim_matches('"').to_string();
                match key {
                    "court" => meta.court = val,
                    "county_or_district" => meta.county_or_district = val,
                    "plaintiff" => meta.plaintiff = val,
                    "plaintiff_label" => meta.plaintiff_label = val,
                    "defendant" => meta.defendant = val,
                    "defendant_label" => meta.defendant_label = val,
                    "case_no" => meta.case_no = val,
                    "case_no_label" => meta.case_no_label = val,
                    "judge" => meta.judge = val,
                    "doc_title" => meta.doc_title = val,
                    "extra_caption" => meta.extra_caption = val,
                    "trial_court" | "lower_court" => meta.trial_court = val,
                    "trial_docket" | "lower_court_case" | "trial_court_docket" => {
                        meta.trial_docket = val
                    }
                    "court_address" => meta.court_address = val,
                    "governing_state" | "state" => meta.governing_state = val,
                    "caption_layout" => meta.caption_layout = val,
                    "filer_email" | "email" => meta.filer_email = val,
                    "filer_phone" | "phone" => meta.filer_phone = val,
                    "filer_address" | "address" => meta.filer_address = val,
                    _ => {}
                }
            }
        }
        return (meta, body);
    }
    (meta, trimmed.to_string())
}

fn apply_inline_markers(line: &str) -> String {
    line.replace("**", "")
        .replace("\\pagebreak", "")
        .trim()
        .to_string()
}

pub fn assemble_lines(meta: &CourtFrontMatter, body: &str) -> Vec<String> {
    let mut lines = Vec::new();
    if !meta.court.is_empty() {
        for part in meta.court.split(" | ") {
            lines.push(part.trim().to_string());
        }
    }
    let distinct_div = crate::pleading::division_if_distinct(&meta.court, &meta.county_or_district);
    if !distinct_div.is_empty() {
        lines.push(distinct_div);
    }
    if !meta.plaintiff.is_empty() || !meta.defendant.is_empty() {
        lines.push(String::new());
        lines.push(format!("{},", meta.plaintiff));
        lines.push(format!(
            "    {},",
            if meta.plaintiff_label.is_empty() {
                "Plaintiff"
            } else {
                meta.plaintiff_label.as_str()
            }
        ));
        lines.push(String::new());
        lines.push("v.".into());
        lines.push(String::new());
        lines.push(format!("{},", meta.defendant));
        lines.push(format!(
            "    {}.",
            if meta.defendant_label.is_empty() {
                "Defendant"
            } else {
                meta.defendant_label.as_str()
            }
        ));
        lines.push(String::new());
        if !meta.case_no.is_empty() {
            let label = if meta.case_no_label.is_empty() {
                "Case No."
            } else {
                meta.case_no_label.as_str()
            };
            lines.push(format!("{label} {}", meta.case_no));
        }
        if !meta.judge.is_empty() {
            lines.push(meta.judge.clone());
        }
        for extra in meta.extra_caption.split(" | ").filter(|s| !s.is_empty()) {
            lines.push(extra.to_string());
        }
        lines.push(String::new());
    }
    if !meta.doc_title.is_empty() {
        lines.push(meta.doc_title.to_ascii_uppercase());
        lines.push(String::new());
    }

    let mut mode = "body";
    for raw in body.lines() {
        let s = raw.trim();
        if s.starts_with("::: ") || s == ":::" {
            let name = s.trim_start_matches(":::").trim();
            if name.is_empty() {
                mode = "body";
            } else {
                mode = name;
                if mode == "certificate" {
                    lines.push(String::new());
                    lines.push("CERTIFICATE OF SERVICE".into());
                    lines.push(String::new());
                }
            }
            continue;
        }
        if s == "\\pagebreak" {
            lines.push(String::new());
            continue;
        }
        if mode == "signature" {
            if s.is_empty() {
                lines.push(String::new());
            } else {
                lines.push(apply_inline_markers(s));
            }
            continue;
        }
        if s.is_empty() {
            lines.push(String::new());
            continue;
        }
        if let Some(rest) = s.strip_prefix("### ") {
            lines.push(rest.to_string());
        } else if let Some(rest) = s.strip_prefix("## ") {
            lines.push(rest.to_string());
        } else if let Some(rest) = s.strip_prefix("# ") {
            lines.push(rest.to_ascii_uppercase());
        } else if let Some(rest) = s.strip_prefix("> ") {
            lines.push(rest.to_string());
        } else if let Some(rest) = s.strip_prefix("- ") {
            lines.push(format!("• {rest}"));
        } else {
            lines.push(apply_inline_markers(s));
        }
    }
    lines
}

pub fn parse_markdown(text: &str) -> CourtMarkdownDoc {
    let (meta, body) = parse_front_matter(text);
    let lines = assemble_lines(&meta, &body);
    CourtMarkdownDoc { meta, body, lines }
}

pub fn import_markdown(case_dir: &Path, markdown: &str) -> Result<CourtMarkdownDoc, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Import into the Documents pointer is refused.".into());
    }
    let doc = parse_markdown(markdown);
    fs::create_dir_all(case_dir.join("drafts")).map_err(|e| e.to_string())?;
    fs::write(case_dir.join("drafts").join("draft.md"), markdown).map_err(|e| e.to_string())?;
    let mut profile = crate::case_profile::load_case(case_dir)?;
    apply_front_matter(&mut profile, &doc.meta);
    profile.court_markdown = markdown.to_string();
    if profile.motion.user_text.trim().is_empty() {
        profile.motion.user_text = doc.body.clone();
    }
    crate::case_profile::save_case(case_dir, &profile)?;
    Ok(doc)
}

fn apply_front_matter(profile: &mut CaseProfile, meta: &CourtFrontMatter) {
    if !meta.court.is_empty() && profile.court.name.trim().is_empty() {
        profile.court.name = meta.court.clone();
    }
    if !meta.case_no.is_empty() && profile.docket_number.trim().is_empty() {
        profile.docket_number = meta.case_no.clone();
    }
    if !meta.doc_title.is_empty() && profile.motion.title.trim().is_empty() {
        profile.motion.title = meta.doc_title.clone();
    }
    if !meta.plaintiff.is_empty() && profile.caption.appellants().is_empty() {
        let party = crate::case_profile::Party {
            name: meta.plaintiff.clone(),
            role: if meta.plaintiff_label.is_empty() {
                "Appellant".into()
            } else {
                meta.plaintiff_label.clone()
            },
            ..Default::default()
        };
        profile.caption.plaintiffs = vec![party.clone()];
        profile.caption.appellants = vec![party];
    }
    if !meta.defendant.is_empty() && profile.caption.appellees().is_empty() {
        let party = crate::case_profile::Party {
            name: meta.defendant.clone(),
            role: if meta.defendant_label.is_empty() {
                "Respondent".into()
            } else {
                meta.defendant_label.clone()
            },
            ..Default::default()
        };
        profile.caption.defendants = vec![party.clone()];
        profile.caption.appellees = vec![party];
    }
    if profile.filer.name.trim().is_empty() && !meta.plaintiff.is_empty() {
        profile.filer.name = meta.plaintiff.clone();
        if profile.filer.signature_name.trim().is_empty() {
            profile.filer.signature_name = meta.plaintiff.clone();
        }
    }
    if profile.notice_of_appeal.trial_court_name.trim().is_empty() && !meta.trial_court.is_empty() {
        profile.notice_of_appeal.trial_court_name = meta.trial_court.clone();
    }
    if profile.notice_of_appeal.trial_court_docket.trim().is_empty() && !meta.trial_docket.is_empty()
    {
        profile.notice_of_appeal.trial_court_docket = meta.trial_docket.clone();
    }
    if profile.court.address_lines.is_empty() && !meta.court_address.is_empty() {
        profile.court.address_lines = split_address(&meta.court_address);
    }
    if profile.governing_state.trim().is_empty() && !meta.governing_state.is_empty() {
        profile.governing_state = meta.governing_state.clone();
    }
    if profile.caption_layout.trim().is_empty() && !meta.caption_layout.is_empty() {
        profile.caption_layout = meta.caption_layout.clone();
    }
    fill_filer_from_meta(profile, meta, false);
}

fn split_address(raw: &str) -> Vec<String> {
    raw.split(" | ")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Overlay markdown front matter onto a working copy of the profile for preview/export.
/// Does not write case.json. The user's draft is the source for this paper's caption.
pub fn overlay_profile(profile: &CaseProfile, meta: &CourtFrontMatter) -> CaseProfile {
    let mut p = profile.clone();
    if !meta.court.is_empty() {
        p.court.name = meta.court.clone();
    }
    if !meta.county_or_district.is_empty() {
        let d = crate::pleading::division_if_distinct(&p.court.name, &meta.county_or_district);
        p.court.division = if d.is_empty() { None } else { Some(d) };
    }
    if !meta.case_no.is_empty() {
        p.docket_number = meta.case_no.clone();
    }
    if !meta.doc_title.is_empty() {
        p.motion.title = meta.doc_title.clone();
    }
    if !meta.plaintiff.is_empty() {
        let party = crate::case_profile::Party {
            name: meta.plaintiff.clone(),
            role: if meta.plaintiff_label.is_empty() {
                "Appellant".into()
            } else {
                meta.plaintiff_label.clone()
            },
            ..Default::default()
        };
        p.caption.plaintiffs = vec![party.clone()];
        p.caption.appellants = vec![party];
    }
    if !meta.defendant.is_empty() {
        let party = crate::case_profile::Party {
            name: meta.defendant.clone(),
            role: if meta.defendant_label.is_empty() {
                "Respondent".into()
            } else {
                meta.defendant_label.clone()
            },
            ..Default::default()
        };
        p.caption.defendants = vec![party.clone()];
        p.caption.appellees = vec![party];
    }
    if !meta.trial_court.is_empty() {
        p.notice_of_appeal.trial_court_name = meta.trial_court.clone();
    }
    if !meta.trial_docket.is_empty() {
        p.notice_of_appeal.trial_court_docket = meta.trial_docket.clone();
    }
    if !meta.court_address.is_empty() {
        p.court.address_lines = split_address(&meta.court_address);
    }
    if !meta.governing_state.is_empty() {
        p.governing_state = meta.governing_state.clone();
    }
    if !meta.caption_layout.is_empty() {
        p.caption_layout = meta.caption_layout.clone();
    }
    if p.filer.name.trim().is_empty() && !meta.plaintiff.is_empty() {
        p.filer.name = meta.plaintiff.clone();
        if p.filer.signature_name.trim().is_empty() {
            p.filer.signature_name = meta.plaintiff.clone();
        }
    }
    fill_filer_from_meta(&mut p, meta, true);
    p
}

fn fill_filer_from_meta(profile: &mut CaseProfile, meta: &CourtFrontMatter, overwrite: bool) {
    if (overwrite || profile.filer.email.trim().is_empty()) && !meta.filer_email.is_empty() {
        profile.filer.email = meta.filer_email.clone();
    }
    if (overwrite || profile.filer.phone.trim().is_empty()) && !meta.filer_phone.is_empty() {
        profile.filer.phone = meta.filer_phone.clone();
    }
    if (overwrite || profile.filer.address_lines.is_empty()) && !meta.filer_address.is_empty() {
        profile.filer.address_lines = split_address(&meta.filer_address);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CourtPreview {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub title: String,
    pub caption_kind: String,
    pub court_name: String,
    pub docket_number: String,
    pub trial_court: String,
    pub trial_docket: String,
    pub lines: Vec<String>,
    pub has_signature: bool,
    pub has_certificate: bool,
    pub markup_stripped: bool,
    pub notice: String,
}

pub fn preview_markdown(profile: &CaseProfile, markdown: &str) -> CourtPreview {
    let parsed = parse_markdown(markdown);
    let working = overlay_profile(profile, &parsed.meta);
    let heading = if parsed.meta.doc_title.is_empty() {
        if working.motion.title.trim().is_empty() {
            "MOTION"
        } else {
            working.motion.title.as_str()
        }
    } else {
        parsed.meta.doc_title.as_str()
    };
    let style = crate::pleading::caption_style(&working, "motion", None);
    let doc = crate::pleading::assemble(&working, "motion", heading, &parsed.body, style);
    let lines = crate::pleading::document_lines(&doc);
    let blob = lines.join("\n");
    let leaks = crate::pleading::find_verification_leaks(&parsed.body);
    let notice = if leaks.is_empty() {
        "Preview only. Nothing was filed or served. Review every line before you export.".into()
    } else {
        format!(
            "Warning: user-authored text contains 'needs_verification' ({}). Export will refuse until you remove it from the paper (YAML metadata is fine). Preview does not rewrite your sentences.",
            crate::pleading::format_verification_leaks(&leaks)
        )
    };
    CourtPreview {
        ok: !parsed.body.trim().is_empty() || !working.motion.user_text.trim().is_empty(),
        never_files: true,
        never_serves: true,
        title: heading.to_string(),
        caption_kind: doc.venue.as_str().to_string(),
        court_name: working.court.name.clone(),
        docket_number: working.docket_number.clone(),
        trial_court: working.notice_of_appeal.trial_court_name.clone(),
        trial_docket: working.notice_of_appeal.trial_court_docket.clone(),
        has_signature: blob.contains("/s/"),
        has_certificate: blob.contains("CERTIFICATE OF SERVICE"),
        markup_stripped: !crate::pleading::contains_forbidden_markup(&blob),
        notice,
        lines,
    }
}

pub fn markdown_from_profile(profile: &CaseProfile, kind: &str) -> String {
    let left = profile.caption.appellants();
    let right = profile.caption.appellees();
    let plaintiff = crate::case_profile::Caption::party_names(&left);
    let defendant = crate::case_profile::Caption::party_names(&right);
    let doc_title = match kind {
        "appellate_brief" => "APPELLATE BRIEF (BASIC SCAFFOLD)",
        "appellate_motion" => "MOTION IN THE APPELLATE COURT (BASIC TEMPLATE)",
        "mandamus" => "PETITION FOR RULE TO SHOW CAUSE (BASIC TEMPLATE)",
        "complaint" => "COMPLAINT (BASIC TEMPLATE)",
        "notice_of_appeal" => "NOTICE OF APPEAL",
        other if other.starts_with("motion") => "MOTION",
        _ => "MOTION",
    };
    let body = match kind {
        "complaint" => profile.complaint.user_text.as_str(),
        "appellate_brief" => profile.brief.user_text.as_str(),
        "appellate_motion" => profile.appellate_motion.user_text.as_str(),
        "mandamus" => profile.mandamus.user_text.as_str(),
        "notice_of_appeal" => profile.notice_of_appeal.user_text.as_str(),
        _ => {
            if !profile.court_markdown.trim().is_empty() {
                // already full markdown
            }
            profile.motion.user_text.as_str()
        }
    };
    let addr = profile.filer.address_lines.join("\n");
    let filer_addr = profile.filer.address_lines.join(" | ");
    let distinct_div = crate::pleading::division_if_distinct(
        &profile.court.name,
        profile.court.division.as_deref().unwrap_or(""),
    );
    let div_line = if distinct_div.is_empty() {
        String::new()
    } else {
        format!("county_or_district: {distinct_div}\n")
    };
    format!(
        "---\ncourt: {court}\n{div_line}plaintiff: {plaintiff}\nplaintiff_label: {plabel}\ndefendant: {defendant}\ndefendant_label: {dlabel}\ncase_no: {docket}\ncase_no_label: Docket No.\ndoc_title: {doc_title}\nfiler_email: {email}\nfiler_phone: {phone}\nfiler_address: {filer_addr}\n---\n\n{body}\n\n::: signature\n/s/ {sig}\n{name}\n{role}\n{addr}\n{phone}\n{email}\n:::\n\n::: certificate\nI certify that on ____________ I served a copy of this paper on ____________ by ____________.\n:::\n",
        court = profile.court.name,
        div_line = div_line,
        plabel = left.first().map(|p| p.role.as_str()).unwrap_or("Plaintiff"),
        dlabel = right.first().map(|p| p.role.as_str()).unwrap_or("Defendant"),
        docket = profile.docket_number,
        sig = profile.filer.signature_name,
        name = profile.filer.name,
        role = profile.filer.role,
        phone = profile.filer.phone,
        email = profile.filer.email,
        filer_addr = filer_addr,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_reads_caption_and_blocks() {
        let md = r#"---
court: Superior Court of New Jersey, Appellate Division
plaintiff: Jordan Example
plaintiff_label: Appellant
defendant: Sample County Clerk
defendant_label: Respondent
case_no: A-01000-24
doc_title: Motion for enlargement of time
---

# Preliminary statement
This sentence is the user's own draft.

::: signature
/s/ Jordan Example
Pro se
:::

::: certificate
Served by the user, not by this app.
:::
"#;
        let doc = parse_markdown(md);
        assert_eq!(doc.meta.case_no, "A-01000-24");
        let blob = doc.lines.join("\n");
        assert!(blob.contains("JORDAN EXAMPLE") || blob.contains("Jordan Example"));
        assert!(blob.contains("CERTIFICATE OF SERVICE"));
        assert!(blob.contains("MOTION FOR ENLARGEMENT OF TIME"));
        assert!(!blob.contains(":::"));
        assert!(!blob.contains("# Preliminary"));
        assert!(!blob.contains("not legal advice"));
    }

    #[test]
    fn import_fills_caption_from_front_matter() {
        let dir = std::env::temp_dir().join(format!("os-md-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        crate::case_profile::ensure_case_layout(&dir).unwrap();
        let profile = crate::case_profile::CaseProfile::new_standard(
            "Example v. Sample County Clerk",
            "reconsideration",
        );
        crate::case_profile::save_case(&dir, &profile).unwrap();
        let md = r#"---
court: Superior Court of New Jersey, Appellate Division
plaintiff: Jordan Example
plaintiff_label: Appellant
defendant: Sample County Clerk
defendant_label: Respondent
case_no: A-01000-24
doc_title: Motion for reconsideration
---

The trial court entered the order on March 1.
"#;
        import_markdown(&dir, md).unwrap();
        let loaded = crate::case_profile::load_case(&dir).unwrap();
        assert_eq!(loaded.docket_number, "A-01000-24");
        assert_eq!(loaded.caption.appellants[0].name, "Jordan Example");
        assert_eq!(loaded.caption.appellees[0].name, "Sample County Clerk");
        assert!(dir.join("drafts").join("draft.md").is_file());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn preview_nj_and_co_markdown_uses_venue_captions() {
        let empty = crate::case_profile::CaseProfile::new_standard("Preview", "motion");
        let nj = preview_markdown(&empty, include_str!("../../fixtures/sample-reconsideration.md"));
        assert_eq!(nj.caption_kind, "nj-boxed");
        assert!(nj.docket_number.contains("A-01000-24"));
        assert!(nj.trial_docket.contains("L-01000-24"));
        let blob = nj.lines.join("\n");
        assert!(blob.contains("COURT USE ONLY"));
        assert!(blob.contains("Appellate"));
        assert!(blob.contains("CERTIFICATE OF SERVICE"));
        assert!(blob.contains("I certify that on October 16, 2026 I served a copy of this motion on Sample County Clerk by electronic mail."));
        assert!(!blob.to_ascii_lowercase().contains("i will serve"));
        assert!(blob.contains("/s/ Jordan Example"));
        assert!(blob.contains("100 Sample Street"));
        assert!(blob.contains("jordan.example@example.test"));
        assert!(!blob.contains("Division / Courtroom"));
        assert_eq!(blob.matches("Appellate Division").count(), 1);
        assert!(blob.contains("Case Number:"));
        assert!(!blob.to_ascii_lowercase().contains("needs_verification"));
        assert!(nj.markup_stripped);
        assert!(!blob.contains(":::"));
        assert!(!blob.contains("# Preliminary"));

        let co = preview_markdown(&empty, include_str!("../../fixtures/sample-co-reconsideration.md"));
        assert_eq!(co.caption_kind, "co-boxed");
        assert_eq!(co.docket_number, "24CA1000");
        let cob = co.lines.join("\n");
        assert!(cob.contains("Lower Court"));
        assert!(cob.contains("24CA1000"));
        assert!(cob.contains("I declare under penalty of perjury under the law of Colorado"));
        assert!(cob.contains("100 Sample Street"));
        assert!(cob.contains("jordan.example@example.test"));
        assert!(!cob.contains("DECLARATION UNDER C.R.S."));
        assert!(!cob.contains("Certificate of Conferral (C.R.C.P. 121"));
        assert!(!cob.to_ascii_lowercase().contains("needs_verification"));
        assert!(cob.contains("I certify that on October 2, 2026 I served a copy of this motion on Sample County Clerk by electronic mail."));
        assert!(!cob.to_ascii_lowercase().contains("i will serve"));
        assert!(co.has_certificate);
        assert!(co.has_signature);
    }

    #[test]
    fn preview_fills_blank_certificate_and_export_refuses_incomplete() {
        let empty = crate::case_profile::CaseProfile::new_standard("Preview", "motion");
        let md = r#"---
court: Superior Court of New Jersey, Appellate Division
plaintiff: Jordan Example
defendant: Sample County Clerk
case_no: A-01000-24
governing_state: NJ
---
# Body
Hello.
::: signature
/s/ Jordan Example
:::
::: certificate
I will serve a copy of this paper myself. This software did not serve anyone.
:::
"#;
        let preview = preview_markdown(&empty, md);
        let blob = preview.lines.join("\n");
        assert!(blob.contains("I certify that on ____________ I served a copy of this motion on ____________ by ____________."));
        assert!(!blob.to_ascii_lowercase().contains("i will serve"));
        assert!(!blob.contains("parties listed below"));

        let dir = std::env::temp_dir().join(format!("os-cos-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        crate::case_profile::ensure_case_layout(&dir).unwrap();
        let profile = crate::case_profile::CaseProfile::new_standard("Example v. Sample County Clerk", "motion");
        crate::case_profile::save_case(&dir, &profile).unwrap();
        import_markdown(&dir, md).unwrap();
        let loaded = crate::case_profile::load_case(&dir).unwrap();
        let err = crate::documents::export_court_markdown(&dir, &loaded, None).unwrap_err();
        assert!(err.contains("recipient") || err.contains("certificate of service"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
