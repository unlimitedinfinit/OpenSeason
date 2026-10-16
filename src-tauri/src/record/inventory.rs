use std::fs;
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};

use super::types::{DocumentProvenance, EvidentiaryTier, RecordEntry, RecordInventoryReport};
use crate::case_profile::CaseProfile;

pub fn calculate_sha256(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("Cannot open {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| format!("Hashing failed {}: {e}", path.display()))?;
    Ok(format!("{:x}", hasher.finalize()))
}

/// Detects if a PDF file contains a court filing stamp or official clerk docketing header.
pub fn detect_filing_stamp(path: &Path) -> bool {
    if !path.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("pdf")).unwrap_or(false) {
        return false;
    }

    if let Ok(doc) = lopdf::Document::load(path) {
        let pages = doc.get_pages();
        for &page_id in pages.keys().take(2) {
            if let Ok(text) = doc.extract_text(&[page_id]) {
                let lower = text.to_ascii_lowercase();
                if lower.contains("filed")
                    || lower.contains("clerk of the court")
                    || lower.contains("received")
                    || lower.contains("docket no")
                    || lower.contains("electronically filed")
                    || lower.contains("jeds")
                    || lower.contains("ecf")
                    || lower.contains("entered on")
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Scans the entire case folder and classifies every document into one of the 4 evidentiary tiers.
pub fn scan_case_record(case_dir: &Path, profile: &CaseProfile) -> Result<RecordInventoryReport, String> {
    let mut entries = Vec::new();

    let scan_dirs = [
        ("orders", EvidentiaryTier::OfficialCourtRecord),
        ("filings", EvidentiaryTier::OfficialCourtRecord),
        ("evidence", EvidentiaryTier::PotentialAdditionalEvidence),
        ("drafts", EvidentiaryTier::PersonalCaseArchive),
    ];

    let mut entry_index = 1;

    for (subfolder, default_tier) in scan_dirs {
        let dir_path = case_dir.join(subfolder);
        if !dir_path.is_dir() {
            continue;
        }

        if let Ok(dir_entries) = fs::read_dir(&dir_path) {
            for entry in dir_entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                    if fname.starts_with('.') || fname == "sync.json" || fname == "record.json" {
                        continue;
                    }

                    let size_bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    let sha256 = calculate_sha256(&path).unwrap_or_default();
                    let relative_path = format!("{subfolder}/{fname}");

                    let stamp_detected = detect_filing_stamp(&path);

                    // Refine tier classification
                    let mut tier = default_tier;
                    let mut verified = false;

                    let lower_name = fname.to_ascii_lowercase();
                    if lower_name.contains("transcript") || lower_name.contains("hearing") || lower_name.contains("oral_argument") {
                        tier = EvidentiaryTier::HearingAndEvidentiaryRecord;
                        verified = stamp_detected || lower_name.contains("certified");
                    } else if subfolder == "orders" {
                        tier = EvidentiaryTier::OfficialCourtRecord;
                        verified = stamp_detected;
                    } else if subfolder == "filings" {
                        if lower_name.contains("draft") || lower_name.contains("notes") {
                            tier = EvidentiaryTier::PersonalCaseArchive;
                            verified = false;
                        } else {
                            tier = EvidentiaryTier::OfficialCourtRecord;
                            verified = stamp_detected;
                        }
                    } else if subfolder == "evidence" {
                        // Check if exhibit is certified or referenced in docket
                        if lower_name.contains("exhibit") && stamp_detected {
                            tier = EvidentiaryTier::OfficialCourtRecord;
                            verified = true;
                        } else {
                            tier = EvidentiaryTier::PotentialAdditionalEvidence;
                            verified = false;
                        }
                    } else if subfolder == "drafts" {
                        tier = EvidentiaryTier::PersonalCaseArchive;
                        verified = false;
                    }

                    let origin = match subfolder {
                        "orders" => "court".to_string(),
                        "filings" => if stamp_detected { "court".to_string() } else { "litigant".to_string() },
                        "evidence" => "third_party".to_string(),
                        "drafts" => "litigant".to_string(),
                        _ => "unknown".to_string(),
                    };

                    let prov = DocumentProvenance {
                        origin,
                        original_path: None,
                        version: 1,
                        submission_channel: if stamp_detected { Some("eCourts / Clerk Desk".into()) } else { None },
                        submission_date: None,
                        submission_status: if stamp_detected { Some("accepted".into()) } else { Some("draft".into()) },
                        rejection_reason: None,
                        accepted_date: None,
                        docket_sequence: None,
                        docket_code: None,
                        filing_stamp_verified: stamp_detected,
                        served_recipients: Vec::new(),
                        service_method: None,
                        service_date: None,
                        certificate_of_service_ref: None,
                        evidentiary_tier: tier,
                        completeness: if verified { "complete".into() } else { "unverified".into() },
                    };

                    entries.push(RecordEntry {
                        id: format!("rec-{entry_index:03}"),
                        filename: fname.clone(),
                        relative_path,
                        sha256,
                        size_bytes,
                        tier,
                        docket_date: None,
                        filing_stamp_detected: stamp_detected,
                        source_provenance: format!("Scanned from {subfolder}/ directory"),
                        title: fname.replace(".pdf", "").replace(".docx", "").replace(".md", "").replace(['-', '_'], " "),
                        notes: if stamp_detected { "Court filing stamp or clerk header detected".into() } else { "No clerk filing stamp detected".into() },
                        verified,
                        provenance: Some(prov),
                    });
                    entry_index += 1;
                }

            }
        }
    }

    let mut tier1_count = 0;
    let mut tier2_count = 0;
    let mut tier3_count = 0;
    let mut tier4_count = 0;

    for e in &entries {
        match e.tier {
            EvidentiaryTier::OfficialCourtRecord => tier1_count += 1,
            EvidentiaryTier::HearingAndEvidentiaryRecord => tier2_count += 1,
            EvidentiaryTier::PersonalCaseArchive => tier3_count += 1,
            EvidentiaryTier::PotentialAdditionalEvidence => tier4_count += 1,
        }
    }

    let report = RecordInventoryReport {
        case_id: profile.id.clone(),
        court_name: profile.court.name.clone(),
        docket_number: profile.docket_number.clone(),
        total_files_scanned: entries.len(),
        tier1_official_count: tier1_count,
        tier2_hearing_count: tier2_count,
        tier3_archive_count: tier3_archive_count(tier3_count),
        tier4_unsubmitted_count: tier4_count,
        entries,
        missing_records: Vec::new(),
        reconciliation_summary: format!(
            "Scanned {} documents: {} official court records, {} hearing/transcript records, {} personal drafts, {} potential unsubmitted evidence.",
            tier1_count + tier2_count + tier3_count + tier4_count,
            tier1_count,
            tier2_count,
            tier3_count,
            tier4_count
        ),
    };

    save_record_inventory(case_dir, &report)?;
    Ok(report)
}

fn tier3_archive_count(c: usize) -> usize {
    c
}

pub fn load_record_inventory(case_dir: &Path) -> Option<RecordInventoryReport> {
    let p = case_dir.join(".openseason").join("record.json");
    if !p.is_file() {
        return None;
    }
    let data = fs::read_to_string(p).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_record_inventory(case_dir: &Path, report: &RecordInventoryReport) -> Result<(), String> {
    let dir = case_dir.join(".openseason");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let data = serde_json::to_string_pretty(report).map_err(|e| e.to_string())?;
    fs::write(dir.join("record.json"), data).map_err(|e| e.to_string())
}
