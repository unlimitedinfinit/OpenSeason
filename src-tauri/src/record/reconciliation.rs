use std::path::Path;

use super::inventory::{load_record_inventory, save_record_inventory};
use super::types::{DocketEntry, EvidentiaryTier, MissingRecordItem, RecordInventoryReport};
use crate::case_profile::CaseProfile;

/// Parses a simple text docket listing (e.g. copied from JEDS, PACER, or court docket sheet)
pub fn parse_text_docket(text: &str) -> Vec<DocketEntry> {
    let mut entries = Vec::new();
    let mut seq = 1;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Try to find a date in YYYY-MM-DD or MM/DD/YYYY format
        let parts: Vec<&str> = trimmed.split(['\t', '|']).map(|s| s.trim()).collect();
        if parts.len() >= 2 {
            entries.push(DocketEntry {
                sequence_num: Some(seq),
                date: parts[0].to_string(),
                title: parts[1].to_string(),
                description: if parts.len() > 2 { parts[2].to_string() } else { parts[1].to_string() },
                filed_by: if parts.len() > 3 { Some(parts[3].to_string()) } else { None },
                matched_record_id: None,
            });
            seq += 1;
        } else if trimmed.len() > 10 {
            // Single line fallback
            entries.push(DocketEntry {
                sequence_num: Some(seq),
                date: String::new(),
                title: trimmed.to_string(),
                description: trimmed.to_string(),
                filed_by: None,
                matched_record_id: None,
            });
            seq += 1;
        }
    }

    entries
}

/// Reconciles the local case inventory against official docket entries or profile timeline.
pub fn reconcile_docket(
    case_dir: &Path,
    profile: &CaseProfile,
    docket_entries: &[DocketEntry],
) -> Result<RecordInventoryReport, String> {
    let mut report = load_record_inventory(case_dir)
        .unwrap_or_else(|| super::inventory::scan_case_record(case_dir, profile).unwrap_or_default());

    let mut missing = Vec::new();
    let mut missing_index = 1;

    // 1. Check docket entries against local inventory
    for doc in docket_entries {
        let doc_title_lower = doc.title.to_ascii_lowercase();
        let mut found_match = false;

        for entry in &mut report.entries {
            let entry_name_lower = entry.filename.to_ascii_lowercase();
            if (entry_name_lower.contains(&doc_title_lower) || doc_title_lower.contains(&entry_name_lower))
                || (!doc.date.is_empty() && entry.filename.contains(&doc.date))
            {
                entry.verified = true;
                if entry.tier == EvidentiaryTier::PersonalCaseArchive && entry.filing_stamp_detected {
                    entry.tier = EvidentiaryTier::OfficialCourtRecord;
                }
                if let Some(ref mut prov) = entry.provenance {
                    prov.docket_sequence = doc.sequence_num;
                    if !doc.date.is_empty() {
                        prov.accepted_date = Some(doc.date.clone());
                    }
                    prov.filing_stamp_verified = true;
                    prov.evidentiary_tier = entry.tier;
                    prov.completeness = "complete".into();
                }
                found_match = true;
                break;
            }

        }

        if !found_match {
            let is_transcript = doc_title_lower.contains("hearing")
                || doc_title_lower.contains("oral argument")
                || doc_title_lower.contains("proceeding")
                || doc_title_lower.contains("trial");

            let is_order = doc_title_lower.contains("order") || doc_title_lower.contains("judgment") || doc_title_lower.contains("dismiss");
            let is_opposition = doc_title_lower.contains("opposition") || doc_title_lower.contains("response") || doc_title_lower.contains("reply");

            let (record_type, rec_action, cost_est, req_form) = if is_transcript {
                (
                    "transcript".to_string(),
                    "Request certified hearing transcript from court reporting agency or Appellate Division".to_string(),
                    "Cost varies by page count ($3.00 - $4.50/page estimated for expedited; ordinary delivery lower)".to_string(),
                    Some("NJ Appellate / Superior Court Transcript Request Form".to_string()),
                )
            } else if is_order {
                (
                    "order".to_string(),
                    "Request signed/entered copy from court clerk or download from JEDS/eCourts".to_string(),
                    "NJ Judiciary Form CN 10200: 5¢ per letter-size page, 7¢ per legal-size page".to_string(),
                    Some("NJ Judiciary Court Records Request Form (CN 10200)".to_string()),
                )
            } else if is_opposition {
                (
                    "opposition".to_string(),
                    "Request copy of opposing party's filed submission from clerk or demand service from counsel".to_string(),
                    "Available via Court Records Request or PACER ($0.10/page capped at $3.00)".to_string(),
                    Some("NJ Judiciary Court Records Request Form (CN 10200)".to_string()),
                )
            } else {
                (
                    "court_filing".to_string(),
                    "Obtain official stamped filing to confirm procedural completeness".to_string(),
                    "Standard statutory copy fees apply".to_string(),
                    Some("NJ Judiciary Court Records Request Form (CN 10200)".to_string()),
                )
            };

            missing.push(MissingRecordItem {
                id: format!("mis-{missing_index:03}"),
                title: doc.title.clone(),
                record_type,
                reason: format!("Listed on court docket for date '{}' but no verified copy found locally in case workspace.", doc.date),
                docket_entry_ref: Some(doc.title.clone()),
                recommended_action: rec_action,
                estimated_cost_notice: cost_est,
                request_form: req_form,
            });
            missing_index += 1;
        }
    }

    // 2. Also check Timeline Events in case profile for missing orders or hearings
    for ev in &profile.timeline {
        let ev_lower = ev.title.to_ascii_lowercase();
        if (ev_lower.contains("hearing") || ev_lower.contains("oral argument"))
            && !report.entries.iter().any(|e| e.tier == EvidentiaryTier::HearingAndEvidentiaryRecord)
        {
            let already_flagged = missing.iter().any(|m| m.record_type == "transcript");
            if !already_flagged {
                missing.push(MissingRecordItem {
                    id: format!("mis-{missing_index:03}"),
                    title: format!("Certified Transcript for '{}' ({})", ev.title, ev.date),
                    record_type: "transcript".to_string(),
                    reason: "Case timeline notes a hearing/oral argument took place, but no certified transcript is in the case record. A court order based on oral colloquy cannot be effectively evaluated or appealed without the transcript.".to_string(),
                    docket_entry_ref: Some(ev.title.clone()),
                    recommended_action: "Order official transcript before filing reconsideration or appeal based on oral statements.".to_string(),
                    estimated_cost_notice: "Standard transcript rates apply. Deposit usually required.".to_string(),
                    request_form: Some("NJ Transcript Request Form".to_string()),
                });
                missing_index += 1;
            }
        }
    }

    report.missing_records = missing;
    report.reconciliation_summary = format!(
        "Reconciled case record against docket: {} files matched, {} missing records identified (transcripts/orders/filings needed to perfect the record).",
        report.entries.iter().filter(|e| e.verified).count(),
        report.missing_records.len()
    );

    save_record_inventory(case_dir, &report)?;
    Ok(report)
}
