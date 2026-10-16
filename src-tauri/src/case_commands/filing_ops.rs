use std::path::Path;

use crate::case_profile::{self, CaseMode, REVIEW_NOTICE};
use crate::documents::{self, ExportPaths};
use crate::sync::{self, SyncAction, SyncRefusal};
use super::types::{case_dir_from_id, cases_root, ExportResult};

#[tauri::command]
pub fn export_pleading_cmd(case_id: String, kind: String) -> Result<ExportResult, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = case_profile::load_case(&dir)?;
    let paths: ExportPaths = documents::export_pleading(&dir, &profile, &kind, None)?;
    Ok(ExportResult {
        docx: paths.docx.to_string_lossy().into_owned(),
        pdf: paths.pdf.to_string_lossy().into_owned(),
        notice: REVIEW_NOTICE.to_string(),
    })
}

#[tauri::command]
pub fn export_notice_of_appeal_cmd(case_id: String) -> Result<ExportResult, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = case_profile::load_case(&dir)?;
    let paths: ExportPaths = documents::export_notice_of_appeal(&dir, &profile, None)?;
    Ok(ExportResult {
        docx: paths.docx.to_string_lossy().into_owned(),
        pdf: paths.pdf.to_string_lossy().into_owned(),
        notice: REVIEW_NOTICE.to_string(),
    })
}

#[tauri::command]
pub fn sync_case_cmd(case_id: String, action: String) -> Result<SyncRefusal, String> {
    let dir = case_dir_from_id(&case_id)?;
    let action = match action.as_str() {
        "backup" => SyncAction::Backup,
        "publish" => SyncAction::Publish,
        "pull_account" | "pull" => SyncAction::PullAccount,
        other => {
            return Err(format!(
                "Unknown sync action '{}'. Use backup, publish, or pull_account.",
                other
            ))
        }
    };
    sync::describe_sync_for_dir(&dir, action)
}

#[tauri::command]
pub fn import_markdown_cmd(case_id: String, markdown: String) -> Result<crate::court_md::CourtMarkdownDoc, String> {
    crate::court_md::import_markdown(&case_dir_from_id(&case_id)?, &markdown)
}

#[tauri::command]
pub fn preview_markdown_cmd(
    markdown: String,
    case_id: Option<String>,
) -> Result<crate::court_md::CourtPreview, String> {
    let profile = match case_id {
        Some(id) if !id.trim().is_empty() => case_profile::load_case(&case_dir_from_id(&id)?)?,
        _ => case_profile::CaseProfile::new_standard("Preview", "motion"),
    };
    Ok(crate::court_md::preview_markdown(&profile, &markdown))
}

/// Same steps as the desktop Draft-to-PDF export (`export_markdown_draft_cmd`).
pub fn export_markdown_draft_at(
    cases_root: &Path,
    markdown: String,
    case_id: Option<String>,
    title: Option<String>,
) -> Result<ExportResult, String> {
    let dir = match case_id {
        Some(id) if !id.trim().is_empty() => {
            crate::sandbox::resolve_id_under_root(cases_root, &id)?
        }
        _ => {
            let parsed = crate::court_md::parse_markdown(&markdown);
            let caption_title = title.unwrap_or_else(|| {
                if !parsed.meta.plaintiff.is_empty() && !parsed.meta.defendant.is_empty() {
                    format!("{} v. {}", parsed.meta.plaintiff, parsed.meta.defendant)
                } else {
                    "Imported draft".into()
                }
            });
            let (dir, _profile) = case_profile::create_case_folder(
                cases_root,
                &caption_title,
                CaseMode::Standard,
                "reconsideration",
            )?;
            dir
        }
    };
    crate::court_md::import_markdown(&dir, &markdown)?;
    let profile = case_profile::load_case(&dir)?;
    let paths = documents::export_court_markdown(&dir, &profile, None)?;
    Ok(ExportResult {
        docx: paths.docx.to_string_lossy().into_owned(),
        pdf: paths.pdf.to_string_lossy().into_owned(),
        notice: REVIEW_NOTICE.to_string(),
    })
}

#[tauri::command]
pub fn export_markdown_draft_cmd(
    markdown: String,
    case_id: Option<String>,
    title: Option<String>,
) -> Result<ExportResult, String> {
    export_markdown_draft_at(&cases_root(), markdown, case_id, title)
}

#[tauri::command]
pub fn list_case_filings_cmd(case_id: String) -> Result<Vec<crate::case_profile::CaseFilingItem>, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = case_profile::load_case(&dir)?;
    Ok(profile.filings)
}

#[tauri::command]
pub fn save_case_filing_cmd(case_id: String, mut filing: crate::case_profile::CaseFilingItem) -> Result<crate::case_profile::CaseFilingItem, String> {
    let dir = case_dir_from_id(&case_id)?;
    let mut profile = case_profile::load_case(&dir)?;
    if filing.id.trim().is_empty() {
        filing.id = uuid::Uuid::new_v4().to_string();
    }
    let now = chrono::Local::now().to_rfc3339();
    if filing.created_at.is_empty() {
        filing.created_at = now.clone();
    }
    filing.updated_at = now;

    if let Some(pos) = profile.filings.iter().position(|f| f.id == filing.id) {
        profile.filings[pos] = filing.clone();
    } else {
        profile.filings.push(filing.clone());
    }
    case_profile::save_case(&dir, &profile)?;
    Ok(filing)
}

#[tauri::command]
pub fn delete_case_filing_cmd(case_id: String, filing_id: String) -> Result<bool, String> {
    let dir = case_dir_from_id(&case_id)?;
    let mut profile = case_profile::load_case(&dir)?;
    profile.filings.retain(|f| f.id != filing_id);
    case_profile::save_case(&dir, &profile)?;
    Ok(true)
}

#[tauri::command]
pub fn export_case_filing_cmd(case_id: String, filing_id: String) -> Result<ExportPaths, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = case_profile::load_case(&dir)?;
    let filing = profile.filings.iter().find(|f| f.id == filing_id)
        .ok_or_else(|| format!("Filing {filing_id} not found in case"))?;
    crate::documents::export_custom_filing(&dir, &profile, &filing.title, &filing.user_text, None)
}

#[tauri::command]
pub fn generate_certificate_of_service_cmd(
    case_id: String,
    params: crate::service::ServiceCertParams,
) -> Result<crate::service::ServiceCertResult, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = case_profile::load_case(&dir)?;
    crate::service::generate_certificate_of_service(&dir, &profile, &params)
}

#[tauri::command]
pub fn generate_proposed_order_cmd(
    case_id: String,
    motion_title: String,
    relief: String,
) -> Result<crate::service::ProposedOrderResult, String> {
    let dir = case_dir_from_id(&case_id)?;
    let profile = case_profile::load_case(&dir)?;
    crate::service::generate_proposed_order(&dir, &profile, &motion_title, &relief)
}
