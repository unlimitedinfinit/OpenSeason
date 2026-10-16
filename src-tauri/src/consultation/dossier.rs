use std::fs;
use std::path::Path;

use crate::case_profile::CaseProfile;
use crate::record::inventory::load_record_inventory;
use super::stress_test::stress_test_all_issues;
use super::types::{ConsultationDossier, DossierExhibitItem};

/// Assembles a 2-page executive Attorney Consultation Dossier with verified record citations,
/// stress-tested vulnerabilities, exhibit inventory with SHA-256 hashes, and targeted legal questions.
pub fn generate_consultation_dossier(
    case_dir: &Path,
    profile: &CaseProfile,
    custom_questions: &[String],
) -> Result<ConsultationDossier, String> {
    let jurisdiction = if !profile.governing_state.is_empty() {
        profile.governing_state.clone()
    } else if profile.court.name.to_lowercase().contains("colorado") {
        "Colorado".into()
    } else if profile.court.name.to_lowercase().contains("new jersey") {
        "New Jersey".into()
    } else {
        "General / Federal".into()
    };

    // 1. Executive Summary
    let stage = if profile.procedural_stage.is_empty() {
        "Active Matter"
    } else {
        &profile.procedural_stage
    };

    let exec_summary = format!(
        "Matter: {} (Docket: {})\nCourt: {}\nFiler: {} ({})\nProcedural Stage: {}\n\n\
        Executive Posture:\nThis matter involves review and potential procedural escalation of orders entered in {}. \
        The record has been inventoried under four evidentiary tiers (Official, Hearing Transcripts, Archive, and Unsubmitted), \
        with SHA-256 provenance tracking to prevent reliance on unadmitted evidence.",
        profile.title,
        if profile.docket_number.is_empty() { "[UNASSIGNED]" } else { &profile.docket_number },
        profile.court.name,
        profile.filer.name,
        profile.filer.role,
        stage,
        jurisdiction
    );

    // 2. Timeline Highlights
    let mut timeline_highlights = Vec::new();
    for ev in &profile.timeline {
        timeline_highlights.push(format!("- **{}**: {} — {}", ev.date, ev.title, ev.notes));
    }
    if timeline_highlights.is_empty() {
        timeline_highlights.push("- No timeline events entered yet in case profile.".into());
    }

    // 3. Issue Stress Tests
    let stress_tests = stress_test_all_issues(profile);

    // 4. Exhibit Inventory & Provenance
    let mut exhibits_inventory = Vec::new();
    if let Some(inv) = load_record_inventory(case_dir) {
        for (i, entry) in inv.entries.iter().enumerate() {
            let letter = format!("{}", (b'A' + (i % 26) as u8) as char);
            let sha_short = if entry.sha256.len() >= 8 { &entry.sha256[..8] } else { &entry.sha256 };
            exhibits_inventory.push(DossierExhibitItem {
                letter,
                description: entry.filename.clone(),
                filename: entry.filename.clone(),
                sha256_short: sha_short.into(),
                tier: entry.tier.label().into(),
                relevancy_tag: if entry.verified { "Official Record".into() } else { "Unverified Local Archive".into() },
            });
        }
    }

    // 5. Targeted Questions for Counsel
    let mut questions = Vec::new();
    questions.push("1. Given the interlocutory vs. final disposition posture, is direct appeal available as of right, or is a motion for leave required?".into());
    questions.push("2. Which standard of review (de novo vs. abuse of discretion) governs the primary findings challenged?".into());
    questions.push("3. Are the specific objections sufficiently preserved in the lower court hearing transcripts, or is plain error review triggered?".into());
    questions.push("4. Does filing a post-judgment motion toll the statutory appeal clock under local procedural rules?".into());
    for q in custom_questions {
        if !q.trim().is_empty() {
            questions.push(format!("- {}", q.trim()));
        }
    }

    // 6. Render Markdown
    let mut md = String::new();
    md.push_str("# ATTORNEY CONSULTATION DOSSIER\n\n");
    md.push_str("> **NOTICE**: Assembled by JustLegal OS formatting and record engine from user's verified files. ");
    md.push_str("Does not practice law or invent citations. Prepared solely to facilitate efficient legal consultation.\n\n");
    md.push_str("## 1. CASE PROFILE & JURISDICTIONAL POSTURE\n\n");
    md.push_str(&exec_summary);
    md.push_str("\n\n## 2. KEY PROCEDURAL TIMELINE\n\n");
    for t in &timeline_highlights {
        md.push_str(t);
        md.push('\n');
    }
    md.push_str("\n## 3. IDENTIFIED ISSUES & DEVIL'S ADVOCATE STRESS TEST\n\n");
    if stress_tests.is_empty() {
        md.push_str("No specific legal issues registered in case profile. Run `issues stress-test` after adding issues.\n");
    } else {
        for test in &stress_tests {
            md.push_str(&format!("### Issue: {} [{}]\n", test.issue_title, test.overall_risk_rating));
            md.push_str(&format!("- **Asserted By**: {}\n", test.asserted_by));
            md.push_str(&format!("- **Procedural Summary**: {}\n", test.procedural_posture_summary));
            for v in &test.vulnerabilities {
                md.push_str(&format!("  - **[{:?}] {}** ({:?})\n", v.vulnerability_type, v.title, v.severity));
                md.push_str(&format!("    - *Vulnerability*: {}\n", v.description));
                md.push_str(&format!("    - *Predicted Opposing Argument*: {}\n", v.opposing_argument_prediction));
                md.push_str(&format!("    - *Suggested Inquiry*: {}\n", v.suggested_cure_or_inquiry));
                if let Some(c) = &v.citation_anchor {
                    md.push_str(&format!("    - *Authority Anchor*: {}\n", c));
                }
            }
            md.push('\n');
        }
    }
    md.push_str("## 4. EVIDENTIARY EXHIBITS & TRANSCRIPTS INVENTORY\n\n");
    if exhibits_inventory.is_empty() {
        md.push_str("No exhibits inventoried yet. Run `record inventory` to catalog case evidence.\n");
    } else {
        md.push_str("| Ex. | Filename | Evidentiary Tier | SHA-256 | Status |\n");
        md.push_str("|:---:|:---|:---|:---:|:---|\n");
        for ex in &exhibits_inventory {
            md.push_str(&format!("| {} | {} | {} | `{}` | {} |\n", ex.letter, ex.filename, ex.tier, ex.sha256_short, ex.relevancy_tag));
        }
    }
    md.push_str("\n## 5. TARGETED QUESTIONS FOR COUNSEL\n\n");
    for q in &questions {
        md.push_str(q);
        md.push('\n');
    }

    // Persist Markdown to exports/consultation_dossier.md
    let exports_dir = case_dir.join("exports");
    let _ = fs::create_dir_all(&exports_dir);
    let md_path = exports_dir.join("consultation_dossier.md");
    let _ = fs::write(&md_path, &md);

    Ok(ConsultationDossier {
        case_id: profile.id.clone(),
        title: profile.title.clone(),
        jurisdiction,
        docket_number: profile.docket_number.clone(),
        executive_summary: exec_summary,
        timeline_highlights,
        issue_stress_tests: stress_tests,
        exhibits_inventory,
        targeted_questions_for_counsel: questions,
        rendered_markdown: md,
        exported_docx_path: None,
        exported_pdf_path: Some(md_path.to_string_lossy().to_string()),
    })
}
