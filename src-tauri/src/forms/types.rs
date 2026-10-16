//! Types, structures, and notices for the forms and packs engine.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::rule_pack::FormEntry;

pub const FIXTURE_CATALOG_JSON: &str = include_str!("../../../templates/forms-catalog/fixture.json");
pub const LEGAL_INFO_NOTICE: &str =
    "This is legal information, not legal advice. OpenSeason never files or serves. Confirm every form and deadline on the court's own site.";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PackSource {
    pub id: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub repo: String,
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CatalogPack {
    pub id: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub court: String,
    #[serde(default)]
    pub situations: Vec<String>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub size_bytes: u64,
    #[serde(default)]
    pub download_url: String,
    #[serde(default)]
    pub local_dir: String,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub rule_pack_id: String,
    #[serde(default)]
    pub source_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FormsCatalog {
    #[serde(default)]
    pub schema: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub generated_at: String,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub source_note: String,
    #[serde(default)]
    pub never_files: bool,
    #[serde(default)]
    pub never_serves: bool,
    #[serde(default)]
    pub sources: Vec<PackSource>,
    #[serde(default)]
    pub packs: Vec<CatalogPack>,
    #[serde(default)]
    pub forms: Vec<FormEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackListItem {
    pub id: String,
    pub state: String,
    pub court: String,
    pub title: String,
    pub situations: Vec<String>,
    pub size_bytes: u64,
    pub size_label: String,
    pub sha256: String,
    pub installed: bool,
    pub install_path: Option<String>,
    pub rule_pack_id: String,
    pub source_note: String,
    /// False when the catalog row has no download, local folder, or sha256.
    #[serde(default)]
    pub installable: bool,
    #[serde(default)]
    pub unavailable_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackList {
    pub never_files: bool,
    pub never_serves: bool,
    pub notice: String,
    pub packs_root: String,
    pub catalog_id: String,
    pub available: Vec<PackListItem>,
    pub installed: Vec<PackListItem>,
    pub rule_packs: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstallReport {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub notice: String,
    pub pack_id: String,
    pub state: String,
    pub situation: Option<String>,
    pub bytes_planned: u64,
    pub size_label: String,
    pub dest: String,
    pub files: Vec<String>,
    pub sha256_ok: bool,
    pub offline_after: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerifyReport {
    pub ok: bool,
    pub never_files: bool,
    pub pack_id: String,
    pub dest: String,
    pub checked: Vec<VerifyRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerifyRow {
    pub path: String,
    pub expected: String,
    pub actual: String,
    pub ok: bool,
}

#[derive(Debug, Clone, Default)]
pub struct InstallOpts {
    pub state: String,
    pub situation: Option<String>,
    pub catalog: Option<PathBuf>,
    pub file: Option<PathBuf>,
    pub root: Option<PathBuf>,
    pub allow_network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormFieldInfo {
    pub key: String,
    pub label: String,
    pub field_type: String,
    pub value_from_case: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormInspection {
    pub form_name: String,
    pub form_path: String,
    pub detected_form_number: Option<String>,
    pub fields: Vec<FormFieldInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormFillResult {
    pub ok: bool,
    pub never_files: bool,
    pub form_path: String,
    pub output_pdf_path: String,
    pub output_json_path: String,
    pub mapped_fields: usize,
    pub notice: String,
}
