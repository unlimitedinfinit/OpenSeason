//! File persistence, directory layouts, and case discovery on disk.

use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

use crate::court_rules::{CourtRules, GENERIC_RULES_JSON};
use super::profile::CaseProfile;
use super::types::{CaseMode, Caption, CourtInfo, Filer, NoticeOfAppealDraft, Party, PleadingDraft, CASE_SUBDIRS};

pub fn default_cases_root() -> PathBuf {
    dirs::document_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("JustLegal")
        .join("Cases")
}

pub fn ensure_case_layout(case_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(case_dir)
        .map_err(|e| format!("Could not create case folder {}: {}", case_dir.display(), e))?;
    for sub in CASE_SUBDIRS {
        fs::create_dir_all(case_dir.join(sub))
            .map_err(|e| format!("Could not create {} folder: {}", sub, e))?;
    }
    let rules_path = case_dir.join("court-rules").join("generic.json");
    if !rules_path.exists() {
        fs::write(&rules_path, GENERIC_RULES_JSON)
            .map_err(|e| format!("Could not write default court rules: {}", e))?;
    }
    Ok(())
}

pub fn case_json_path(case_dir: &Path) -> PathBuf {
    case_dir.join("case.json")
}

pub fn load_case(case_dir: &Path) -> Result<CaseProfile, String> {
    let path = case_json_path(case_dir);
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("Could not read {}: {}", path.display(), e))?;
    let mut profile: CaseProfile =
        serde_json::from_str(&text).map_err(|e| format!("case.json is not valid: {}", e))?;
    crate::seal::apply_disk_seal(&mut profile, case_dir);
    Ok(profile)
}

pub fn save_case(case_dir: &Path, profile: &CaseProfile) -> Result<(), String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err(
            "This case is sealed. The Documents folder is a pointer only and cannot be edited.".to_string(),
        );
    }
    ensure_case_layout(case_dir)?;
    let mut to_write = profile.clone();
    to_write.updated_at = Utc::now().to_rfc3339();
    if crate::seal::is_dir_sealed(case_dir) {
        to_write.mode = CaseMode::Confidential;
        if to_write.sealed_at.is_none() {
            to_write.sealed_at = crate::seal::read_marker_sealed_at(case_dir);
        }
    }
    let text = serde_json::to_string_pretty(&to_write)
        .map_err(|e| format!("Could not serialize case.json: {}", e))?;
    fs::write(case_json_path(case_dir), text)
        .map_err(|e| format!("Could not write case.json: {}", e))?;
    if to_write.sealed_at.is_some() {
        let when = to_write
            .sealed_at
            .clone()
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        crate::seal::write_seal_marker(case_dir, &to_write.id, &when)?;
    }
    Ok(())
}

pub fn create_case_folder(
    parent_or_path: &Path,
    title: &str,
    mode: CaseMode,
    document_kind: &str,
) -> Result<(PathBuf, CaseProfile), String> {
    let mut profile = CaseProfile::new_standard(title, document_kind);
    if title.trim().is_empty() {
        return Err("A case title is required.".to_string());
    }
    let dest_name = parent_or_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if crate::sandbox::is_reserved_marker_name(dest_name) {
        return Err("Refusing to create a case folder named .sealed.".to_string());
    }
    let app_data = crate::sandbox::default_app_data_root();
    let resolved = crate::sandbox::canonicalize_existing_prefix(parent_or_path);
    if app_data.exists() {
        if let Ok(app_canon) = app_data.canonicalize() {
            if resolved == app_canon || resolved.starts_with(&app_canon) {
                return Err("Refusing to create a case folder inside the app data directory.".to_string());
            }
        }
    }
    if mode == CaseMode::Confidential {
        profile
            .convert_to_confidential()
            .expect("new case can be sealed");
    }

    let case_dir = if parent_or_path.join("case.json").exists() {
        return Err(format!(
            "A case already exists at {}",
            parent_or_path.display()
        ));
    } else if parent_or_path.exists()
        && parent_or_path.is_dir()
        && fs::read_dir(parent_or_path)
            .map_err(|e| e.to_string())?
            .next()
            .is_none()
    {
        parent_or_path.to_path_buf()
    } else if parent_or_path.extension().is_none() && !parent_or_path.exists() {
        parent_or_path.to_path_buf()
    } else {
        parent_or_path.join(&profile.id)
    };

    ensure_case_layout(&case_dir)?;
    save_case(&case_dir, &profile)?;
    let _ = crate::guided::load_or_init(&case_dir, &profile.document_kind);
    let loaded = load_case(&case_dir)?;
    Ok((case_dir, loaded))
}

pub fn resolve_case_dir(id_or_path: &str) -> PathBuf {
    let as_path = PathBuf::from(id_or_path);
    if as_path.join("case.json").exists() {
        return as_path;
    }
    default_cases_root().join(id_or_path)
}

pub fn list_cases_in(root: &Path) -> Result<Vec<CaseProfile>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut cases = Vec::new();
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() && path.join("case.json").exists() {
            if let Ok(profile) = load_case(&path) {
                cases.push(profile);
            }
        }
    }
    cases.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(cases)
}

pub fn layout_missing(case_dir: &Path) -> Vec<String> {
    CASE_SUBDIRS
        .iter()
        .filter(|sub| !case_dir.join(sub).is_dir())
        .map(|s| (*s).to_string())
        .collect()
}

pub fn sample_appeal_profile() -> CaseProfile {
    CaseProfile {
        schema_version: 1,
        id: "00000000-0000-4000-8000-000000000001".to_string(),
        title: "Example v. Sample County Clerk".to_string(),
        mode: CaseMode::Standard,
        document_kind: "notice_of_appeal".to_string(),
        created_at: "2024-06-01T12:00:00Z".to_string(),
        updated_at: "2024-06-01T12:00:00Z".to_string(),
        sealed_at: None,
        court: CourtInfo {
            id: "generic-us-appellate".to_string(),
            name: "United States Court of Appeals for the Sample Circuit".to_string(),
            division: None,
            address_lines: Vec::new(),
        },
        docket_number: "24-01000".to_string(),
        caption: Caption {
            plaintiffs: vec![Party {
                name: "Jordan Example".to_string(),
                role: "Appellant".to_string(),
                counsel: "Pro se".to_string(),
                ..Default::default()
            }],
            defendants: vec![Party {
                name: "Sample County Clerk".to_string(),
                role: "Appellee".to_string(),
                counsel: "Counsel of record".to_string(),
                ..Default::default()
            }],
            appellants: vec![Party {
                name: "Jordan Example".to_string(),
                role: "Appellant".to_string(),
                counsel: "Pro se".to_string(),
                ..Default::default()
            }],
            appellees: vec![Party {
                name: "Sample County Clerk".to_string(),
                role: "Appellee".to_string(),
                counsel: "Counsel of record".to_string(),
                ..Default::default()
            }],
        },
        filer: Filer {
            name: "Jordan Example".to_string(),
            role: "Appellant, proceeding without an attorney".to_string(),
            address_lines: vec![
                "100 Sample Street".to_string(),
                "Example City, ST 00000".to_string(),
            ],
            phone: "555-0100".to_string(),
            email: "jordan.example@example.test".to_string(),
            signature_name: "Jordan Example".to_string(),
        },
        notice_of_appeal: NoticeOfAppealDraft {
            judgment_date: "2024-03-01".to_string(),
            judgment_description: "Order granting the motion to dismiss".to_string(),
            trial_court_name: "United States District Court for the Sample District".to_string(),
            trial_court_docket: "1:23-cv-01000".to_string(),
            user_text: "Appellant appeals from the order entered on March 1, 2024, granting the motion to dismiss. This paragraph is the user's own draft. OpenSeason did not write the grounds of appeal.".to_string(),
        },
        complaint: PleadingDraft {
            title: "Complaint for declaratory and injunctive relief".to_string(),
            user_text: "This paragraph is the user's own statement of the claim. OpenSeason did not invent the facts or the legal theory.".to_string(),
        },
        motion: PleadingDraft {
            title: "Motion for an extension of time".to_string(),
            user_text: "This paragraph is the user's own motion text. OpenSeason did not invent the request or the grounds.".to_string(),
        },
        timeline: Vec::new(),
        rule_pack_id: "generic-us-appellate".to_string(),
        motion_kind: String::new(),
        brief: PleadingDraft::default(),
        appellate_motion: PleadingDraft::default(),
        mandamus: PleadingDraft::default(),
        court_markdown: String::new(),
        cloud_matter_id: None,
        cloud_synced_at: None,
        filings: Vec::new(),
        matter_tier: "litigation".to_string(),
        procedural_stage: String::new(),
        governing_state: String::new(),
        parent_matter_id: None,
        conferral_status: String::new(),
        service_method: "electronic".to_string(),
        caption_layout: String::new(),
        roster: Vec::new(),
        issues: Vec::new(),
        tasks: Vec::new(),
        receipts: Vec::new(),
    }
}


pub fn write_sample_fixture(case_dir: &Path) -> Result<CaseProfile, String> {
    let profile = sample_appeal_profile();
    ensure_case_layout(case_dir)?;
    save_case(case_dir, &profile)?;
    let _rules = CourtRules::generic();
    Ok(profile)
}
