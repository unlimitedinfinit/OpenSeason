use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::case_profile::{ensure_case_layout, sample_appeal_profile, save_case, CaseProfile, REVIEW_NOTICE};
use super::assembly::{assembled_notice_paragraphs, assembled_pleading_paragraphs};
use super::exporter::{
    export_court_markdown, export_notice_of_appeal, export_notice_of_appeal_docx_only, export_pleading,
};
use super::writers::{inspect_docx_contains, inspect_pdf_contains};

fn temp_case() -> (PathBuf, CaseProfile) {
    let dir = std::env::temp_dir().join(format!("os-doc-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    ensure_case_layout(&dir).unwrap();
    let profile = sample_appeal_profile();
    save_case(&dir, &profile).unwrap();
    (dir, profile)
}

#[test]
fn notice_pulls_caption_and_user_text() {
    let profile = sample_appeal_profile();
    let lines = assembled_notice_paragraphs(&profile);
    let blob = lines.join("\n");
    assert!(blob.contains("Jordan Example"));
    assert!(blob.contains("Sample County Clerk"));
    assert!(blob.contains("24-01000"));
    assert!(blob.contains("United States Court of Appeals for the Sample Circuit"));
    assert!(blob.contains("user's own draft"));
    assert!(!blob.contains(REVIEW_NOTICE));
    assert!(!blob.contains("Jane Doe"));
}

#[test]
fn docx_export_embeds_profile_fields() {
    let (dir, profile) = temp_case();
    let path = export_notice_of_appeal_docx_only(&dir, &profile, None).unwrap();
    assert!(path.exists());
    assert!(inspect_docx_contains(&path, "24-01000").unwrap());
    assert!(inspect_docx_contains(&path, "Jordan Example").unwrap());
    assert!(!inspect_docx_contains(&path, REVIEW_NOTICE).unwrap());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn export_refuses_incomplete_case() {
    let (dir, mut profile) = temp_case();
    profile.docket_number.clear();
    let err = export_notice_of_appeal_docx_only(&dir, &profile, None).unwrap_err();
    assert!(err.contains("not ready"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn notice_pdf_compiles_for_sample_case() {
    let (dir, profile) = temp_case();
    let result = export_notice_of_appeal(&dir, &profile, None);
    match result {
        Ok(paths) => {
            let bytes = fs::read(&paths.pdf).unwrap();
            assert!(bytes.starts_with(b"%PDF"), "expected a PDF header");
            assert!(paths.docx.exists());
        }
        Err(e) => panic!("PDF export should work when a system font exists: {}", e),
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn exported_filing_does_not_embed_software_disclaimer() {
    let (dir, profile) = temp_case();
    let paths = export_notice_of_appeal(&dir, &profile, None)
        .expect("sample case should export Word and PDF");
    assert!(
        !inspect_docx_contains(&paths.docx, REVIEW_NOTICE).unwrap(),
        "the filing must not include the software disclaimer"
    );
    assert!(
        !inspect_docx_contains(&paths.docx, "not legal advice").unwrap(),
        "the filing must not say it is not legal advice"
    );
    assert!(
        !inspect_pdf_contains(&paths.pdf, "not legal advice").unwrap(),
        "the PDF filing must not include the software disclaimer"
    );
    assert!(dir.join("exports").join("REVIEW-NOTICE.txt").is_file());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn complaint_and_motion_templates_use_profile() {
    let profile = sample_appeal_profile();
    let complaint = assembled_pleading_paragraphs(&profile, "complaint").join("\n");
    assert!(complaint.contains("COMPLAINT") || complaint.contains("Complaint"));
    assert!(complaint.contains("Jordan Example"));
    assert!(complaint.contains("user's own statement"));
    assert!(!complaint.to_ascii_lowercase().contains("not legal advice"));
    assert!(!complaint.contains(REVIEW_NOTICE));
    let motion = assembled_pleading_paragraphs(&profile, "motion").join("\n");
    assert!(motion.contains("MOTION") || motion.contains("Motion"));
    assert!(motion.contains("user's own motion text"));
    assert!(motion.contains("Sample County Clerk"));
}

#[test]
fn complaint_docx_export_embeds_profile_fields() {
    let (dir, mut profile) = temp_case();
    profile.complaint.user_text.push_str(
        "\n::: certificate\nRecipient: Sample County Clerk\nMethod: electronic mail\nDate: October 1, 2026\n:::\n",
    );
    let paths = export_pleading(&dir, &profile, "complaint", None).unwrap();
    assert!(paths.docx.exists());
    assert!(
        inspect_docx_contains(&paths.docx, "COMPLAINT").unwrap()
            || inspect_docx_contains(&paths.docx, "Complaint").unwrap()
    );
    assert!(inspect_docx_contains(&paths.docx, "Jordan Example").unwrap());
    assert!(!inspect_docx_contains(&paths.docx, REVIEW_NOTICE).unwrap());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn motion_pdf_and_docx_are_court_ready() {
    let (dir, mut profile) = temp_case();
    profile.rule_pack_id = "nj-app-div".into();
    profile.governing_state = "NJ".into();
    profile.court.name = "Superior Court of New Jersey, Appellate Division".into();
    profile.court.address_lines = vec![
        "Richard J. Hughes Justice Complex, 25 W. Market Street".into(),
        "P.O. Box 006, Trenton, NJ 08625-0006".into(),
    ];
    profile.docket_number = "A-01000-24".into();
    profile.notice_of_appeal.trial_court_docket = "L-01000-24".into();
    profile.document_kind = "reconsideration".into();
    profile.motion.title = "Motion for reconsideration".into();
    let fixture = include_str!("../../../fixtures/sample-reconsideration.md").replace("\r\n", "\n");
    profile.motion.user_text = fixture
        .split_once("---\n\n")
        .map(|(_, body)| body.to_string())
        .unwrap_or_else(|| {
            "# Preliminary statement\nThis is a fake draft for tests. The trial court entered an order dismissing the complaint.\n::: signature\n/s/ Jordan Example\n:::".into()
        });
    let paths = export_pleading(&dir, &profile, "reconsideration", None).unwrap();
    assert!(crate::pleading::is_valid_docx(&paths.docx).unwrap());
    let xml_ok = inspect_docx_contains(&paths.docx, "word/document.xml");
    let _ = xml_ok;
    let text = pdf_text(&paths.pdf);
    assert!(!text.contains('#'), "raw markdown leaked: {text}");
    assert!(!text.contains(":::"), "fence leaked: {text}");
    assert!(!text.contains("#v("), "typst leaked: {text}");
    assert!(!text.to_ascii_lowercase().contains("to the best of my knowledge"));
    assert!(text.contains("I certify that the foregoing statements made by me are true"));
    assert!(text.contains("COURT USE ONLY") || text.contains("Court Use Only"));
    assert!(text.contains("Appellate") || text.contains("A-01000"));
    assert!(text.contains("CERTIFICATE OF SERVICE") || text.contains("Certificate of Service"));
    assert!(text.contains("Page") && text.contains("of"));
    assert!(!text.to_ascii_lowercase().contains("not legal advice"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn nj_and_co_markdown_exports_have_venue_captions() {
    let (nj_dir, mut nj) = temp_case();
    nj.court_markdown = include_str!("../../../fixtures/sample-reconsideration.md").replace("\r\n", "\n");
    nj.filer.signature_name = "Jordan Example".into();
    nj.filer.name = "Jordan Example".into();
    let nj_paths = export_court_markdown(&nj_dir, &nj, None).unwrap();
    assert!(crate::pleading::is_valid_docx(&nj_paths.docx).unwrap());
    let nj_text = pdf_text(&nj_paths.pdf);
    assert!(!nj_text.contains('#'), "NJ markup leaked: {nj_text}");
    assert!(!nj_text.contains(":::"));
    assert!(!nj_text.contains("#v("));
    assert!(nj_text.contains("A-01000-24"));
    assert!(nj_text.contains("L-01000-24") || nj_text.contains("Trial Court"));
    assert!(nj_text.contains("COURT USE ONLY") || nj_text.contains("Court Use Only"));
    assert!(nj_text.contains("I certify that the foregoing statements made by me are true"));
    assert!(nj_text.contains("CERTIFICATE OF SERVICE") || nj_text.contains("Certificate of Service"));
    assert!(nj_text.contains("/s/") || nj_text.contains("Jordan Example"));

    let (co_dir, mut co) = temp_case();
    co.court_markdown = include_str!("../../../fixtures/sample-co-reconsideration.md").replace("\r\n", "\n");
    co.filer.signature_name = "Jordan Example".into();
    co.filer.name = "Jordan Example".into();
    let co_paths = export_court_markdown(&co_dir, &co, None).unwrap();
    assert!(crate::pleading::is_valid_docx(&co_paths.docx).unwrap());
    let zip = std::fs::File::open(&co_paths.docx).unwrap();
    let mut archive = zip::ZipArchive::new(zip).unwrap();
    assert!(archive.by_name("word/document.xml").is_ok());
    let co_text = pdf_text(&co_paths.pdf);
    assert!(!co_text.contains('#'), "CO markup leaked: {co_text}");
    assert!(!co_text.contains(":::"));
    assert!(!co_text.contains("#v("));
    assert!(co_text.contains("24CA1000"));
    assert!(co_text.contains("23CV1000") || co_text.contains("Lower Court"));
    assert!(co_text.contains("COURT USE ONLY") || co_text.contains("Court Use Only"));
    assert!(co_text.contains("I declare under penalty of perjury under the law of Colorado"));
    assert!(co_text.contains("CERTIFICATE OF SERVICE") || co_text.contains("Certificate of Service"));
    assert!(!nj_text.to_ascii_lowercase().contains("needs_verification"));
    assert!(!co_text.to_ascii_lowercase().contains("needs_verification"));
    assert!(nj_text.contains("Case Number"));
    assert!(co_text.contains("Case Number"));
    assert!(nj_text.contains("I served a copy"));
    assert!(nj_text.contains("Sample County Clerk"));
    assert!(
        nj_text.contains("electronic mail")
            || (nj_text.contains("electronic") && nj_text.contains("mail")),
        "NJ COS method missing: {nj_text}"
    );
    assert!(co_text.contains("I served a copy"));
    assert!(co_text.contains("Sample County Clerk"));
    assert!(
        co_text.contains("electronic mail")
            || (co_text.contains("electronic") && co_text.contains("mail")),
        "CO COS method missing: {co_text}"
    );
    assert!(!nj_text.to_ascii_lowercase().contains("i will serve"));
    assert!(!co_text.to_ascii_lowercase().contains("i will serve"));
    let _ = fs::remove_dir_all(&nj_dir);
    let _ = fs::remove_dir_all(&co_dir);
}

fn pdf_text(path: &Path) -> String {
    let out = std::process::Command::new("pdftotext")
        .args(["-layout", &path.display().to_string(), "-"])
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => {
            if let Ok(res) = crate::pdf_tools::extract_pdf_text(path) {
                if !res.full_text.trim().is_empty() {
                    return res.full_text;
                }
            }
            let bytes = fs::read(path).unwrap_or_default();
            String::from_utf8_lossy(&bytes).into_owned()
        }
    }
}

#[test]
fn appellate_brief_scaffold_uses_profile_and_says_basic() {
    let mut profile = sample_appeal_profile();
    profile.brief.user_text = "This paragraph is the user's own brief point.".into();
    let blob = assembled_pleading_paragraphs(&profile, "appellate_brief").join("\n");
    assert!(blob.contains("APPELLATE BRIEF") || blob.contains("Appellate brief"));
    assert!(blob.contains("user's own brief point"));
    assert!(blob.contains("CERTIFICATE OF SERVICE"));
    assert!(blob.contains("I certify that on ____________ I served a copy of this"));
    assert!(!blob.to_ascii_lowercase().contains("i will serve"));
    assert!(blob.contains("Jordan Example"));
}
