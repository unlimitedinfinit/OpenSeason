use std::path::Path;

use super::types::IssueItem;
use crate::case_profile::CaseProfile;
use crate::record::inventory::load_record_inventory;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FactualPacketReport {
    pub case_id: String,
    pub title: String,
    pub issue_id: String,
    pub date_generated: String,
    pub factual_chronology: Vec<String>,
    pub record_provenance: Vec<String>,
    pub rendered_markdown: String,
    pub notice: String,
}

/// Assembles a factual, exhibit-grounded submission packet for an identified issue.
pub fn assemble_factual_packet(
    case_dir: &Path,
    profile: &CaseProfile,
    issue: &IssueItem,
) -> Result<FactualPacketReport, String> {
    let inv = load_record_inventory(case_dir);
    let mut chrono_items = Vec::new();
    let mut provenance_items = Vec::new();

    chrono_items.push(format!("Matter: {} (Docket No. {})", profile.title, profile.docket_number));
    chrono_items.push(format!("Court: {}", profile.court.name));
    chrono_items.push(format!("Operative Order/Finding: {}", issue.finding_source.document_name));
    chrono_items.push(format!("Finding Text: \"{}\"", issue.finding_source.quoted_text));

    for ev in &profile.timeline {
        chrono_items.push(format!("Event [{}]: {} ({})", ev.date, ev.title, ev.notes));
    }

    if let Some(r) = &inv {
        for entry in &r.entries {
            provenance_items.push(format!(
                "- {} [{}] - SHA-256: {} (Status: {})",
                entry.filename,
                entry.tier.label(),
                &entry.sha256[..12],
                if entry.verified { "Verified in docket" } else { "Unverified filing status" }
            ));
        }
    }

    let mut md = String::new();
    md.push_str(&format!("# Factual Record Summary: {}\n\n", issue.title));
    md.push_str("> [!IMPORTANT]\n");
    md.push_str("> This document contains an objective factual chronology assembled from the case workspace. ");
    md.push_str("It does NOT contain legal arguments or emotional accusations. Review every date and citation before filing or submitting.\n\n");

    md.push_str("## 1. Operative Finding Under Review\n");
    md.push_str(&format!("- **Document**: {}\n", issue.finding_source.document_name));
    md.push_str(&format!("- **Quoted Clause**: \"{}\"\n", issue.finding_source.quoted_text));
    md.push_str(&format!("- **Record Support Status**: {}\n\n", issue.record_support_status.label()));

    md.push_str("## 2. Contradicting or Missing Evidence in Case Record\n");
    if issue.contradicting_evidence_filed.is_empty() && issue.contradicting_evidence_local.is_empty() {
        md.push_str("No conflicting local evidence cataloged.\n\n");
    } else {
        for filed in &issue.contradicting_evidence_filed {
            md.push_str(&format!("- **Filed Evidence**: {}\n", filed));
        }
        for local in &issue.contradicting_evidence_local {
            md.push_str(&format!("- **Local Unfiled Material**: {}\n", local));
        }
        md.push('\n');
    }

    md.push_str("## 3. Procedural Analysis & Recommended Standards\n");
    for route in &issue.available_routes {
        md.push_str(&format!("### {}\n", route.title));
        md.push_str(&format!("- **Rule**: {}\n", route.rule_citation));
        md.push_str(&format!("- **Standard**: {}\n", route.governing_standard));
        if let Some(dl) = &route.deadline_notice {
            md.push_str(&format!("- **Deadline**: {}\n", dl));
        }
        if route.is_disciplinary {
            md.push_str(&format!("- **Warning**: {}\n", route.warning_notice));
        }
        md.push('\n');
    }

    md.push_str("## 4. Evidentiary Provenance Log\n");
    for prov in &provenance_items {
        md.push_str(&format!("{}\n", prov));
    }

    Ok(FactualPacketReport {
        case_id: profile.id.clone(),
        title: issue.title.clone(),
        issue_id: issue.id.clone(),
        date_generated: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        factual_chronology: chrono_items,
        record_provenance: provenance_items,
        rendered_markdown: md,
        notice: "Packet assembled. Factually grounded in case workspace without hallucinated citations.".to_string(),
    })
}
