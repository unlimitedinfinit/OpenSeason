use std::path::PathBuf;
use serde::Serialize;
use crate::case_profile::{self, CaseProfile};

pub(crate) fn cases_root() -> PathBuf {
    case_profile::default_cases_root()
}

pub(crate) fn case_dir_from_id(case_id: &str) -> Result<PathBuf, String> {
    crate::sandbox::resolve_id_under_root(&cases_root(), case_id)
}

#[derive(Serialize)]
pub struct WorkspaceCase {
    pub profile: CaseProfile,
    pub source: String,
    pub has_vault: bool,
    pub sealed: bool,
    pub folder: String,
}

#[derive(Debug, Serialize)]
pub struct CaseArtifact {
    pub kind: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct ExportResult {
    pub docx: String,
    pub pdf: String,
    pub notice: String,
}
