use std::fs;
use std::path::{Path, PathBuf};

use crate::case_profile::{validate_case, CaseProfile};
use crate::court_rules::CourtRules;
use crate::pdf;
use super::types::{assert_notice_out_dir, pleading_body, pleading_heading, ExportPaths};
use super::writers::{
    build_docx_bytes, build_docx_bytes_from_lines, build_typst_source, build_typst_source_from_lines,
};

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
    let _ = crate::pleading::write_review_sidecar(&dest);

    let typst_src = build_typst_source(profile, &rules);
    let pdf_bytes = pdf::compile_typst(typst_src)?;
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
    let (subtitle, body) = pleading_body(profile, &key);
    let title = if subtitle.trim().is_empty() {
        heading.to_string()
    } else {
        subtitle.trim().to_string()
    };
    let style = crate::pleading::caption_style(profile, &key, None);
    let doc = crate::pleading::assemble(profile, &key, &title, body, style);
    crate::pleading::require_exportable(&doc, body)?;
    let rules = CourtRules::load_for_case(case_dir);
    let dest = out_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| case_dir.join("exports"));
    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    let _ = crate::pleading::write_review_sidecar(&dest);

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

    let docx_bytes = crate::pleading::to_docx(&doc, &rules)?;
    fs::write(&docx_path, &docx_bytes).map_err(|e| e.to_string())?;

    let typst_src = crate::pleading::to_typst(&doc, &rules);
    let pdf_bytes = pdf::compile_typst(typst_src)?;
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

    let heading = if title.trim().is_empty() { "MOTION" } else { title.trim() };
    let style = crate::pleading::caption_style(profile, "motion", None);
    let court_doc = crate::pleading::assemble(profile, "motion", heading, user_text, style);
    crate::pleading::require_exportable(&court_doc, user_text)?;
    let rules = CourtRules::load_for_case(case_dir);
    let dest = out_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| case_dir.join("exports"));
    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    let _ = crate::pleading::write_review_sidecar(&dest);

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

    fs::write(&docx_path, crate::pleading::to_docx(&court_doc, &rules)?)
        .map_err(|e| e.to_string())?;
    fs::write(
        &pdf_path,
        pdf::compile_typst(crate::pleading::to_typst(&court_doc, &rules))?,
    )
    .map_err(|e| e.to_string())?;

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
    let parsed = crate::court_md::parse_markdown(&markdown);
    if parsed.body.trim().is_empty() && parsed.lines.is_empty() {
        return Err("Markdown draft is empty.".into());
    }
    let working = crate::court_md::overlay_profile(profile, &parsed.meta);
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
    let court_doc = crate::pleading::assemble(&working, "motion", heading, &parsed.body, style);
    crate::pleading::require_exportable(&court_doc, &parsed.body)?;
    let rules = CourtRules::load_for_case(case_dir);
    let dest = out_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| case_dir.join("exports"));
    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    let _ = crate::pleading::write_review_sidecar(&dest);
    let stem = format!(
        "court-md-{}",
        profile
            .docket_number
            .replace('/', "-")
            .replace(' ', "_")
            .replace('\\', "-")
    );
    let docx_path = dest.join(format!("{stem}.docx"));
    let pdf_path = dest.join(format!("{stem}.pdf"));
    fs::write(&docx_path, crate::pleading::to_docx(&court_doc, &rules)?)
        .map_err(|e| e.to_string())?;
    fs::write(&pdf_path, crate::pdf::compile_typst(crate::pleading::to_typst(&court_doc, &rules))?)
        .map_err(|e| e.to_string())?;
    Ok(ExportPaths {
        docx: docx_path,
        pdf: pdf_path,
    })
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
    let _ = crate::pleading::write_review_sidecar(&dest);
    let typst_src = build_typst_source_from_lines(lines, heading, &rules);
    let pdf_bytes = crate::pdf::compile_typst(typst_src)?;
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
