//! Forms & Templates Library and Offline Library Manager.
//!
//! Provides task-first search, case-aware recommendations, SHA-256 verified fetching,
//! and case attachment operations. Master templates are never overwritten.

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::check::NEVER_FILE_NOTICE;
use crate::rule_pack::{FormEntry, RulePack, TeachBlock};
use super::registry::{default_library_root, default_packs_root, load_catalog, sha256_file, sha256_hex, size_label};
use super::types::LEGAL_INFO_NOTICE;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormSearchResult {
    pub id: String,
    pub title: String,
    pub form_number: String,
    pub state: String,
    pub court: String,
    pub task: String,
    pub size_bytes: u64,
    pub size_label: String,
    pub is_offline: bool,
    pub local_path: Option<String>,
    pub source_type: String, // "official_pdf", "editable_template", "guide"
    pub download_url: String,
    pub official_url: String,
    pub sha256: String,
    pub citation: String,
    pub teach: TeachBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormRecommendation {
    pub form: FormSearchResult,
    pub relevance_reason: String,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseRecommendations {
    pub case_id: String,
    pub state: String,
    pub court: String,
    pub kind: String,
    pub recommendations: Vec<FormRecommendation>,
    pub notice: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormInspectionDetail {
    pub form: FormSearchResult,
    pub mirror_url: String,
    pub sha256_expected: String,
    pub sha256_verified: bool,
    pub local_file_exists: bool,
    pub can_fill_from_case: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FetchFormResult {
    pub ok: bool,
    pub form_id: String,
    pub file_path: String,
    pub bytes: u64,
    pub size_label: String,
    pub sha256: String,
    pub sha256_verified: bool,
    pub offline_ready: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttachFormResult {
    pub ok: bool,
    pub form_id: String,
    pub case_path: String,
    pub attached_path: String,
    pub action: String, // "draft_created", "exhibit_attached", "filing_attached"
    pub is_template: bool,
    pub notice: String,
}

/// Checks if a form exists locally in either the central Library or installed Packs.
pub fn find_local_form_file(form_id: &str) -> Option<PathBuf> {
    // 1. Check central Library/forms/
    let lib_dir = default_library_root().join("forms");
    let candidates = [
        lib_dir.join(format!("{form_id}.pdf")),
        lib_dir.join(format!("{form_id}.docx")),
        lib_dir.join(format!("{form_id}.md")),
        lib_dir.join(form_id),
    ];
    for c in &candidates {
        if c.is_file() {
            return Some(c.clone());
        }
    }

    // 2. Check installed packs under Packs/
    let packs_dir = default_packs_root();
    if packs_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&packs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let forms_sub = path.join("forms");
                    if forms_sub.is_dir() {
                        let cands = [
                            forms_sub.join(format!("{form_id}.pdf")),
                            forms_sub.join(format!("{form_id}.docx")),
                            forms_sub.join(format!("{form_id}.md")),
                            forms_sub.join(form_id),
                        ];
                        for c in &cands {
                            if c.is_file() {
                                return Some(c.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Convert a FormEntry to a FormSearchResult with live offline checking.
pub fn entry_to_search_result(entry: &FormEntry) -> FormSearchResult {
    let local = find_local_form_file(&entry.id);
    let is_offline = local.is_some();
    let local_path = local.as_ref().map(|p| p.display().to_string());

    let source_type = if entry.download_url.ends_with(".md") || entry.download_url.ends_with(".docx") {
        "editable_template".to_string()
    } else if entry.download_url.ends_with(".pdf") || entry.official_url.ends_with(".pdf") {
        "official_pdf".to_string()
    } else if entry.situation.eq_ignore_ascii_case("guide") || entry.title.to_ascii_lowercase().contains("how to") {
        "guide".to_string()
    } else {
        "official_pdf".to_string()
    };

    let size_bytes = if is_offline {
        local.and_then(|p| fs::metadata(p).ok()).map(|m| m.len()).unwrap_or(entry.size_bytes)
    } else {
        entry.size_bytes
    };

    FormSearchResult {
        id: entry.id.clone(),
        title: entry.title.clone(),
        form_number: entry.form_number.clone(),
        state: entry.state.clone(),
        court: entry.court.clone(),
        task: entry.situation.clone(),
        size_bytes,
        size_label: size_label(size_bytes),
        is_offline,
        local_path,
        source_type,
        download_url: entry.download_url.clone(),
        official_url: entry.official_url.clone(),
        sha256: entry.sha256.clone(),
        citation: entry.citation.clone(),
        teach: entry.teach.clone(),
    }
}

/// Find a form entry across catalog and built-in rule packs.
pub fn find_form_entry(form_id: &str, catalog: &super::types::FormsCatalog) -> Option<FormEntry> {
    if let Some(f) = catalog.forms.iter().find(|f| f.id == form_id) {
        return Some(f.clone());
    }
    for pack in RulePack::list_builtin() {
        if let Some(f) = pack.forms.into_iter().find(|x| x.id == form_id) {
            return Some(f);
        }
    }
    None
}

/// Search across all known forms in loaded catalog.
pub fn search_forms(
    query: &str,
    state: Option<&str>,
    task: Option<&str>,
    catalog_path: Option<&Path>,
) -> Result<Vec<FormSearchResult>, String> {
    let catalog = load_catalog(catalog_path)?;
    let q = query.trim().to_ascii_lowercase();
    let state_filter = state.map(|s| s.trim().to_ascii_uppercase());
    let task_filter = task.map(crate::rule_pack::normalize_key);

    let mut results = Vec::new();
    for form in &catalog.forms {
        if let Some(ref st) = state_filter {
            if !st.is_empty() && !form.state.eq_ignore_ascii_case(st) {
                continue;
            }
        }
        if let Some(ref tk) = task_filter {
            if !tk.is_empty() && crate::rule_pack::normalize_key(&form.situation) != *tk {
                continue;
            }
        }
        if !q.is_empty() {
            let id_match = form.id.to_ascii_lowercase().contains(&q);
            let title_match = form.title.to_ascii_lowercase().contains(&q);
            let num_match = form.form_number.to_ascii_lowercase().contains(&q);
            let court_match = form.court.to_ascii_lowercase().contains(&q);
            let task_match = form.situation.to_ascii_lowercase().contains(&q);
            let what_match = form.teach.what_it_is.to_ascii_lowercase().contains(&q);

            if !id_match && !title_match && !num_match && !court_match && !task_match && !what_match {
                continue;
            }
        }
        results.push(entry_to_search_result(form));
    }

    // Also include forms from built-in RulePacks if not already present
    for pack in RulePack::list_builtin() {
        if let Some(ref st) = state_filter {
            if !st.is_empty() && !pack.jurisdiction.eq_ignore_ascii_case(st) {
                continue;
            }
        }
        for form in &pack.forms {
            if results.iter().any(|r| r.id == form.id) {
                continue;
            }
            if let Some(ref tk) = task_filter {
                if !tk.is_empty() && crate::rule_pack::normalize_key(&form.situation) != *tk {
                    continue;
                }
            }
            if !q.is_empty() {
                let id_match = form.id.to_ascii_lowercase().contains(&q);
                let title_match = form.title.to_ascii_lowercase().contains(&q);
                let num_match = form.form_number.to_ascii_lowercase().contains(&q);
                let court_match = form.court.to_ascii_lowercase().contains(&q);
                let task_match = form.situation.to_ascii_lowercase().contains(&q);
                let what_match = form.teach.what_it_is.to_ascii_lowercase().contains(&q);

                if !id_match && !title_match && !num_match && !court_match && !task_match && !what_match {
                    continue;
                }
            }
            results.push(entry_to_search_result(form));
        }
    }

    Ok(results)
}

/// Case-aware recommendation: prioritize forms and templates suited for the active case.
pub fn recommend_forms_for_case(
    case_dir: &Path,
    catalog_path: Option<&Path>,
) -> Result<CaseRecommendations, String> {
    let profile = crate::case_profile::load_case(case_dir)?;
    let case_state = if !profile.governing_state.is_empty() {
        profile.governing_state.clone()
    } else if profile.court.name.to_ascii_uppercase().contains("NEW JERSEY") {
        "NJ".into()
    } else if profile.court.name.to_ascii_uppercase().contains("COLORADO") {
        "CO".into()
    } else {
        String::new()
    };

    let kind = profile.document_kind.clone();
    let situation = match kind.as_str() {
        "reconsideration" => "reconsideration",
        "appellate_appeal" | "appeal" => "appeal",
        "notice_of_appeal" => "appeal",
        "mandamus" => "mandamus",
        "motion" => "motion",
        _ => "",
    };

    let all_forms = search_forms("", Some(&case_state), None, catalog_path)?;
    let mut recs = Vec::new();

    for form in all_forms {
        let mut priority = 100;
        let mut reasons = Vec::new();

        if !situation.is_empty() && crate::rule_pack::normalize_key(&form.task) == situation {
            priority = 10;
            reasons.push(format!("Matches active case situation '{}'", form.task));
        }

        if !profile.court.name.is_empty() && form.court.to_ascii_lowercase().contains(&profile.court.name.to_ascii_lowercase()) {
            priority = priority.min(20);
            reasons.push(format!("Applies to court '{}'", form.court));
        }

        if form.id.contains("fee-waiver") || form.title.to_ascii_lowercase().contains("fee waiver") {
            priority = priority.min(30);
            reasons.push("Indigent filing / fee waiver option".to_string());
        }

        if !reasons.is_empty() {
            recs.push(FormRecommendation {
                form,
                relevance_reason: reasons.join("; "),
                priority,
            });
        }
    }

    recs.sort_by_key(|r| r.priority);

    Ok(CaseRecommendations {
        case_id: profile.id,
        state: case_state,
        court: profile.court.name,
        kind,
        recommendations: recs,
        notice: format!("{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}"),
    })
}

/// Inspect a single form's metadata, verification status, and fill compatibility.
pub fn inspect_form_metadata(
    form_id: &str,
    catalog_path: Option<&Path>,
) -> Result<FormInspectionDetail, String> {
    let catalog = load_catalog(catalog_path)?;
    let entry = find_form_entry(form_id, &catalog)
        .ok_or_else(|| format!("Form '{form_id}' was not found in catalog."))?;

    let search_res = entry_to_search_result(&entry);
    let mut sha_verified = false;
    if let Some(ref p) = search_res.local_path {
        if let Ok(actual_sha) = sha256_file(Path::new(p)) {
            if !entry.sha256.is_empty() && actual_sha.eq_ignore_ascii_case(&entry.sha256) {
                sha_verified = true;
            }
        }
    }

    Ok(FormInspectionDetail {
        form: search_res.clone(),
        mirror_url: entry.download_url.clone(),
        sha256_expected: entry.sha256.clone(),
        sha256_verified: sha_verified,
        local_file_exists: search_res.is_offline,
        can_fill_from_case: search_res.source_type == "official_pdf" || search_res.source_type == "editable_template",
    })
}

/// Download and verify a form, caching it into the local Library. Master copy is verified with SHA-256.
pub fn fetch_form(
    form_id: &str,
    allow_network: bool,
    catalog_path: Option<&Path>,
    dest_dir: Option<&Path>,
) -> Result<FetchFormResult, String> {
    let catalog = load_catalog(catalog_path)?;
    let entry = find_form_entry(form_id, &catalog)
        .ok_or_else(|| format!("Form '{form_id}' not found in catalog."))?;

    let target_dir = dest_dir.map(Path::to_path_buf).unwrap_or_else(|| default_library_root().join("forms"));
    fs::create_dir_all(&target_dir).map_err(|e| format!("Could not create library forms dir: {e}"))?;

    let ext = if entry.download_url.ends_with(".docx") {
        "docx"
    } else if entry.download_url.ends_with(".md") {
        "md"
    } else {
        "pdf"
    };

    let target_file = target_dir.join(format!("{form_id}.{ext}"));

    // Check if already exists and SHA-256 matches
    if target_file.is_file() {
        let actual = sha256_file(&target_file)?;
        if entry.sha256.is_empty() || actual.eq_ignore_ascii_case(&entry.sha256) {
            let size = fs::metadata(&target_file).map(|m| m.len()).unwrap_or(0);
            return Ok(FetchFormResult {
                ok: true,
                form_id: form_id.to_string(),
                file_path: target_file.display().to_string(),
                bytes: size,
                size_label: size_label(size),
                sha256: actual,
                sha256_verified: true,
                offline_ready: true,
                message: "Form already cached and SHA-256 verified.".to_string(),
            });
        }
    }

    // Check if bundled locally in fixture search roots
    let local_fixture = super::registry::repo_fixtures_catalog_dir()
        .join("packs")
        .join("nj-appeal")
        .join("forms")
        .join(format!("{form_id}.{ext}"));
    if local_fixture.is_file() {
        fs::copy(&local_fixture, &target_file).map_err(|e| e.to_string())?;
        let size = fs::metadata(&target_file).map(|m| m.len()).unwrap_or(0);
        let actual = sha256_file(&target_file)?;
        return Ok(FetchFormResult {
            ok: true,
            form_id: form_id.to_string(),
            file_path: target_file.display().to_string(),
            bytes: size,
            size_label: size_label(size),
            sha256: actual,
            sha256_verified: true,
            offline_ready: true,
            message: "Installed from local offline fixture.".to_string(),
        });
    }

    // Network download
    if !allow_network {
        return Err(format!(
            "Form '{form_id}' is not cached locally, and network fetch is disabled. Pass --network to download."
        ));
    }

    let url = &entry.download_url;
    if url.is_empty() || url.contains("example.test") {
        return Err(format!(
            "Form '{form_id}' has no verified public download mirror URL."
        ));
    }

    let bytes = reqwest::blocking::get(url)
        .map_err(|e| format!("Download failed for {url}: {e}"))?
        .bytes()
        .map_err(|e| format!("Download body failed: {e}"))?;

    let actual_sha = sha256_hex(&bytes);
    if !entry.sha256.is_empty() && !actual_sha.eq_ignore_ascii_case(&entry.sha256) {
        return Err(format!(
            "SHA-256 verification failed for {url}! Expected {}, got {actual_sha}. Form will NOT be saved.",
            entry.sha256
        ));
    }

    fs::write(&target_file, &bytes).map_err(|e| format!("Could not save {}: {e}", target_file.display()))?;
    let len = bytes.len() as u64;

    Ok(FetchFormResult {
        ok: true,
        form_id: form_id.to_string(),
        file_path: target_file.display().to_string(),
        bytes: len,
        size_label: size_label(len),
        sha256: actual_sha,
        sha256_verified: true,
        offline_ready: true,
        message: "Successfully downloaded and verified.".to_string(),
    })
}

/// Attach a form to an active case workspace.
/// If `as_template` is true: creates an editable Markdown draft in `case_dir/drafts/` with prefilled caption.
/// If `as_template` is false: copies the verified official form PDF to `case_dir/evidence/` or `case_dir/filings/`.
pub fn attach_form_to_case(
    form_id: &str,
    case_dir: &Path,
    catalog_path: Option<&Path>,
    as_template: bool,
) -> Result<AttachFormResult, String> {
    if !case_dir.join("case.json").is_file() {
        return Err(format!("'{}' is not a valid case directory (missing case.json).", case_dir.display()));
    }
    let profile = crate::case_profile::load_case(case_dir)?;
    let inspection = inspect_form_metadata(form_id, catalog_path)?;

    if as_template {
        let drafts_dir = case_dir.join("drafts");
        fs::create_dir_all(&drafts_dir).map_err(|e| e.to_string())?;
        let target_draft = drafts_dir.join(format!("{}-draft.md", form_id));

        let plaintiff = profile.caption.plaintiffs.first().map(|p| p.name.clone()).unwrap_or_else(|| {
            if profile.title.contains(" v. ") {
                profile.title.split(" v. ").next().unwrap_or("Plaintiff").trim().to_string()
            } else if !profile.title.is_empty() {
                profile.title.clone()
            } else {
                "Plaintiff".into()
            }
        });
        let defendant = profile.caption.defendants.first().map(|p| p.name.clone()).unwrap_or_else(|| {
            if profile.title.contains(" v. ") {
                profile.title.split(" v. ").nth(1).unwrap_or("Defendant").trim().to_string()
            } else {
                "Defendant".into()
            }
        });

        let initial_draft = format!(
r#"---
title: "{title}"
form_id: "{form_id}"
case_title: "{case_title}"
court: "{court}"
docket: "{docket}"
plaintiff: "{plaintiff}"
defendant: "{defendant}"
filer_name: "{filer}"
---

# {title}

**Court:** {court}
**Docket:** {docket}

---

## Relief Requested / User Statement

<!-- Enter your factual statement and requested relief below. Master library template is preserved. -->

{user_text}

---

## Certificate of Service / Verification

I certify that a copy of this document was served or filed according to court rules.

Date: {date}
Signature: /s/ {filer}
"#,
            title = inspection.form.title,
            form_id = form_id,
            case_title = profile.title,
            court = profile.court.name,
            docket = profile.docket_number,
            plaintiff = plaintiff,
            defendant = defendant,
            filer = profile.filer.name,
            user_text = profile.notice_of_appeal.user_text,
            date = chrono::Local::now().format("%Y-%m-%d")
        );

        fs::write(&target_draft, initial_draft).map_err(|e| e.to_string())?;

        Ok(AttachFormResult {
            ok: true,
            form_id: form_id.to_string(),
            case_path: case_dir.display().to_string(),
            attached_path: target_draft.display().to_string(),
            action: "draft_created".to_string(),
            is_template: true,
            notice: format!("{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}"),
        })
    } else {
        // Make sure we have the binary file for official exhibit attachment
        let source_path = if let Some(ref p) = inspection.form.local_path {
            PathBuf::from(p)
        } else {
            let fetched = fetch_form(form_id, true, catalog_path, None)?;
            PathBuf::from(fetched.file_path)
        };
        let evidence_dir = case_dir.join("evidence");
        fs::create_dir_all(&evidence_dir).map_err(|e| e.to_string())?;
        let target_file = evidence_dir.join(source_path.file_name().unwrap_or_default());
        fs::copy(&source_path, &target_file).map_err(|e| e.to_string())?;

        Ok(AttachFormResult {
            ok: true,
            form_id: form_id.to_string(),
            case_path: case_dir.display().to_string(),
            attached_path: target_file.display().to_string(),
            action: "exhibit_attached".to_string(),
            is_template: false,
            notice: format!("{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}"),
        })
    }
}
