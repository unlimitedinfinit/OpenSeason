use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::court_rules::{CourtRules, GENERIC_RULES_JSON};

pub const CASE_SUBDIRS: &[&str] = &[
    "orders",
    "filings",
    "evidence",
    "drafts",
    "court-rules",
    "exports",
];

pub const REVIEW_NOTICE: &str = "This document was assembled by OpenSeason from the user's own case profile and draft text. It is a formatting aid, not legal advice. Review every line, caption, date, and citation before filing. Consider consulting a licensed attorney.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaseMode {
    Standard,
    Confidential,
}

impl CaseMode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "standard" => Ok(CaseMode::Standard),
            "confidential" => Ok(CaseMode::Confidential),
            other => Err(format!(
                "Unknown case mode '{}'. Use standard or confidential.",
                other
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CaseMode::Standard => "standard",
            CaseMode::Confidential => "confidential",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Party {
    pub name: String,
    #[serde(default)]
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Caption {
    #[serde(default)]
    pub plaintiffs: Vec<Party>,
    #[serde(default)]
    pub defendants: Vec<Party>,
    #[serde(default)]
    pub appellants: Vec<Party>,
    #[serde(default)]
    pub appellees: Vec<Party>,
}

impl Caption {
    pub fn appellants(&self) -> Vec<&Party> {
        if !self.appellants.is_empty() {
            self.appellants.iter().collect()
        } else {
            self.plaintiffs.iter().collect()
        }
    }

    pub fn appellees(&self) -> Vec<&Party> {
        if !self.appellees.is_empty() {
            self.appellees.iter().collect()
        } else {
            self.defendants.iter().collect()
        }
    }

    pub fn party_names(parties: &[&Party]) -> String {
        let names: Vec<&str> = parties.iter().map(|p| p.name.as_str()).collect();
        match names.as_slice() {
            [] => String::new(),
            [one] => (*one).to_string(),
            [first, second] => format!("{} and {}", first, second),
            _ => {
                let (last, rest) = names.split_last().unwrap();
                format!("{} and {}", rest.join(", "), last)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CourtInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub division: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Filer {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub address_lines: Vec<String>,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub signature_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NoticeOfAppealDraft {
    #[serde(default)]
    pub judgment_date: String,
    #[serde(default)]
    pub judgment_description: String,
    #[serde(default)]
    pub trial_court_name: String,
    #[serde(default)]
    pub trial_court_docket: String,
    #[serde(default)]
    pub user_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseProfile {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub mode: CaseMode,
    #[serde(default)]
    pub document_kind: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    /// Once set, the case can never be treated as syncable again.
    #[serde(default)]
    pub sealed_at: Option<String>,
    #[serde(default)]
    pub court: CourtInfo,
    #[serde(default)]
    pub docket_number: String,
    #[serde(default)]
    pub caption: Caption,
    #[serde(default)]
    pub filer: Filer,
    #[serde(default)]
    pub notice_of_appeal: NoticeOfAppealDraft,
}

fn default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationIssue {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationReport {
    pub ok: bool,
    pub issues: Vec<ValidationIssue>,
}

impl CaseProfile {
    pub fn new_standard(title: &str, document_kind: &str) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            schema_version: 1,
            id: Uuid::new_v4().to_string(),
            title: title.trim().to_string(),
            mode: CaseMode::Standard,
            document_kind: document_kind.to_string(),
            created_at: now.clone(),
            updated_at: now,
            sealed_at: None,
            court: CourtInfo {
                id: "generic-us-appellate".to_string(),
                name: String::new(),
                division: None,
            },
            docket_number: String::new(),
            caption: Caption::default(),
            filer: Filer::default(),
            notice_of_appeal: NoticeOfAppealDraft::default(),
        }
    }

    pub fn is_sync_forbidden(&self) -> bool {
        self.mode == CaseMode::Confidential || self.sealed_at.is_some()
    }

    pub fn convert_to_confidential(&mut self) -> Result<(), String> {
        if self.mode == CaseMode::Confidential && self.sealed_at.is_some() {
            return Err("This case is already confidential.".to_string());
        }
        let now = Utc::now().to_rfc3339();
        self.mode = CaseMode::Confidential;
        self.sealed_at = Some(now.clone());
        self.updated_at = now;
        Ok(())
    }

    pub fn reopen_as_standard(&self) -> Result<CaseProfile, String> {
        Err("Confidential cases cannot be made syncable. Use a deliberate local export if you need a copy on disk.".to_string())
    }
}

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
    serde_json::from_str(&text).map_err(|e| format!("case.json is not valid: {}", e))
}

pub fn save_case(case_dir: &Path, profile: &CaseProfile) -> Result<(), String> {
    ensure_case_layout(case_dir)?;
    let mut to_write = profile.clone();
    to_write.updated_at = Utc::now().to_rfc3339();
    let text = serde_json::to_string_pretty(&to_write)
        .map_err(|e| format!("Could not serialize case.json: {}", e))?;
    fs::write(case_json_path(case_dir), text)
        .map_err(|e| format!("Could not write case.json: {}", e))?;
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

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn placeholder_hits(text: &str) -> Vec<String> {
    let mut hits = Vec::new();
    let bracket = Regex::new(r"\[[A-Z0-9_][A-Z0-9_ \-/]{1,80}\]").unwrap();
    for cap in bracket.find_iter(text) {
        hits.push(cap.as_str().to_string());
    }
    let mustache = Regex::new(r"\{\{[^}]+\}\}").unwrap();
    for cap in mustache.find_iter(text) {
        hits.push(cap.as_str().to_string());
    }
    let chevron = Regex::new(r"<<[^>]+>>").unwrap();
    for cap in chevron.find_iter(text) {
        hits.push(cap.as_str().to_string());
    }
    for token in ["TODO", "TBD", "FIXME", "___"] {
        if text.to_ascii_uppercase().contains(token) {
            hits.push(token.to_string());
        }
    }
    hits.sort();
    hits.dedup();
    hits
}

pub fn validate_case(profile: &CaseProfile, case_dir: Option<&Path>) -> ValidationReport {
    let mut issues = Vec::new();

    if is_blank(&profile.title) {
        issues.push(ValidationIssue {
            field: "title".into(),
            message: "Case title is required.".into(),
        });
    }
    if is_blank(&profile.court.name) {
        issues.push(ValidationIssue {
            field: "court.name".into(),
            message: "Court name is required so the caption can be filled.".into(),
        });
    }
    if is_blank(&profile.docket_number) {
        issues.push(ValidationIssue {
            field: "docket_number".into(),
            message: "Docket number is required.".into(),
        });
    }
    if profile.caption.appellants().is_empty() {
        issues.push(ValidationIssue {
            field: "caption.appellants".into(),
            message: "Add at least one appellant (or plaintiff).".into(),
        });
    }
    if profile.caption.appellees().is_empty() {
        issues.push(ValidationIssue {
            field: "caption.appellees".into(),
            message: "Add at least one appellee (or defendant).".into(),
        });
    }
    if is_blank(&profile.filer.name) {
        issues.push(ValidationIssue {
            field: "filer.name".into(),
            message: "Filer name is required for the signature block.".into(),
        });
    }
    if is_blank(&profile.filer.signature_name) {
        issues.push(ValidationIssue {
            field: "filer.signature_name".into(),
            message: "Signature name is required.".into(),
        });
    }

    if let Some(dir) = case_dir {
        for missing in layout_missing(dir) {
            issues.push(ValidationIssue {
                field: format!("folder.{}", missing),
                message: format!("Expected folder '{}' is missing.", missing),
            });
        }
    }

    let kind = profile.document_kind.to_ascii_lowercase();
    if kind.is_empty() || kind == "notice_of_appeal" || kind == "appeal" {
        if is_blank(&profile.notice_of_appeal.judgment_date) {
            issues.push(ValidationIssue {
                field: "notice_of_appeal.judgment_date".into(),
                message: "Judgment or order date is required for a Notice of Appeal.".into(),
            });
        }
        if is_blank(&profile.notice_of_appeal.judgment_description) {
            issues.push(ValidationIssue {
                field: "notice_of_appeal.judgment_description".into(),
                message: "Describe the order or judgment being appealed, in the user's own words."
                    .into(),
            });
        }
        if is_blank(&profile.notice_of_appeal.user_text) {
            issues.push(ValidationIssue {
                field: "notice_of_appeal.user_text".into(),
                message: "Paste the user's own Notice of Appeal text. OpenSeason will not invent it.".into(),
            });
        }
        let combined = format!(
            "{} {} {} {}",
            profile.notice_of_appeal.user_text,
            profile.notice_of_appeal.judgment_description,
            profile.court.name,
            profile.title
        );
        for hit in placeholder_hits(&combined) {
            issues.push(ValidationIssue {
                field: "placeholders".into(),
                message: format!("Unfilled placeholder remains: {}", hit),
            });
        }
    }

    ValidationReport {
        ok: issues.is_empty(),
        issues,
    }
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
        },
        docket_number: "24-01000".to_string(),
        caption: Caption {
            plaintiffs: vec![Party {
                name: "Jordan Example".to_string(),
                role: "Appellant".to_string(),
            }],
            defendants: vec![Party {
                name: "Sample County Clerk".to_string(),
                role: "Appellee".to_string(),
            }],
            appellants: vec![Party {
                name: "Jordan Example".to_string(),
                role: "Appellant".to_string(),
            }],
            appellees: vec![Party {
                name: "Sample County Clerk".to_string(),
                role: "Appellee".to_string(),
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
    }
}

pub fn write_sample_fixture(case_dir: &Path) -> Result<CaseProfile, String> {
    let profile = sample_appeal_profile();
    ensure_case_layout(case_dir)?;
    save_case(case_dir, &profile)?;
    let _rules = CourtRules::generic();
    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("os-case-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn creates_expected_layout() {
        let dir = temp_dir();
        ensure_case_layout(&dir).unwrap();
        for sub in CASE_SUBDIRS {
            assert!(dir.join(sub).is_dir(), "missing {}", sub);
        }
        assert!(dir.join("court-rules").join("generic.json").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sample_case_validates() {
        let profile = sample_appeal_profile();
        let report = validate_case(&profile, None);
        assert!(report.ok, "{:?}", report.issues);
    }

    #[test]
    fn missing_fields_and_placeholders_fail() {
        let mut profile = sample_appeal_profile();
        profile.docket_number.clear();
        profile.notice_of_appeal.user_text = "See [PLAINTIFF] and {{docket}} TODO".into();
        let report = validate_case(&profile, None);
        assert!(!report.ok);
        let fields: Vec<_> = report.issues.iter().map(|i| i.field.as_str()).collect();
        assert!(fields.contains(&"docket_number"));
        assert!(fields.contains(&"placeholders"));
    }

    #[test]
    fn fixture_sample_case_validates() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/sample-appeal-case");
        let profile = load_case(&fixture).expect("sample fixture should load");
        let report = validate_case(&profile, Some(&fixture));
        assert!(report.ok, "{:?}", report.issues);
        assert_eq!(profile.filer.name, "Jordan Example");
    }

    #[test]
    fn convert_is_one_way() {
        let mut profile = sample_appeal_profile();
        profile.convert_to_confidential().unwrap();
        assert_eq!(profile.mode, CaseMode::Confidential);
        assert!(profile.sealed_at.is_some());
        assert!(profile.is_sync_forbidden());
        assert!(profile.reopen_as_standard().is_err());
        // Even if someone flips the flag, sealed_at still blocks sync.
        profile.mode = CaseMode::Standard;
        assert!(profile.is_sync_forbidden());
    }
}
