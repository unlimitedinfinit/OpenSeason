use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::case_profile::{CaseProfile, REVIEW_NOTICE};

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
    if !meta.county_or_district.is_empty() {
        lines.push(meta.county_or_district.clone());
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
    if !lines.iter().any(|l| l.contains("not legal advice")) {
        lines.push(String::new());
        lines.push(REVIEW_NOTICE.to_string());
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
    format!(
        "---\ncourt: {court}\ncounty_or_district: {div}\nplaintiff: {plaintiff}\nplaintiff_label: {plabel}\ndefendant: {defendant}\ndefendant_label: {dlabel}\ncase_no: {docket}\ncase_no_label: Docket No.\ndoc_title: {doc_title}\n---\n\n{body}\n\n::: signature\n/s/ {sig}\n{name}\n{role}\n{addr}\n{phone}\n{email}\n:::\n\n::: certificate\nI certify that on this date I served a copy of this paper on the parties listed below by the method I will use when I file. This app did not serve anyone.\n:::\n",
        court = profile.court.name,
        div = profile.court.division.clone().unwrap_or_default(),
        plabel = left.first().map(|p| p.role.as_str()).unwrap_or("Plaintiff"),
        dlabel = right.first().map(|p| p.role.as_str()).unwrap_or("Defendant"),
        docket = profile.docket_number,
        sig = profile.filer.signature_name,
        name = profile.filer.name,
        role = profile.filer.role,
        phone = profile.filer.phone,
        email = profile.filer.email,
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
        assert!(blob.contains("not legal advice"));
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
}
