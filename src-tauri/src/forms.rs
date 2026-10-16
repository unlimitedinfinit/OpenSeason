//! Installable forms catalogs and pack archives. Never bundles the 7 GB JustLegal tree.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::rule_pack::{FormEntry, RulePack};
use crate::check::NEVER_FILE_NOTICE;

pub const FIXTURE_CATALOG_JSON: &str = include_str!("../../templates/forms-catalog/fixture.json");
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

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("Could not read {}: {e}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

pub fn size_label(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

pub fn default_packs_root() -> PathBuf {
    if let Ok(p) = std::env::var("OPENSEASON_PACKS_ROOT") {
        if !p.trim().is_empty() {
            return PathBuf::from(p);
        }
    }
    dirs::document_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("JustLegal")
        .join("Packs")
}

pub fn bundled_catalog() -> Result<FormsCatalog, String> {
    serde_json::from_str(FIXTURE_CATALOG_JSON).map_err(|e| format!("bundled forms catalog: {e}"))
}

pub fn load_catalog(path: Option<&Path>) -> Result<FormsCatalog, String> {
    if let Some(path) = path {
        let text = fs::read_to_string(path)
            .map_err(|e| format!("Could not read catalog {}: {e}", path.display()))?;
        let mut cat: FormsCatalog = serde_json::from_str(&text)
            .map_err(|e| format!("Catalog is not valid JSON: {e}"))?;
        if cat.schema.is_empty() {
            cat = justlegal_state_to_catalog(&text)?;
        }
        return Ok(merge_bundled_fixture_packs(cat));
    }
    let cached = default_packs_root().join("catalog.json");
    if cached.is_file() {
        return load_catalog(Some(&cached));
    }
    bundled_catalog()
}

/// Keep the offline fixture pack visible even when a cached JustLegal catalog is in use.
fn merge_bundled_fixture_packs(mut cat: FormsCatalog) -> FormsCatalog {
    if let Ok(bundled) = bundled_catalog() {
        for pack in bundled.packs {
            if !cat.packs.iter().any(|p| p.id == pack.id) {
                cat.packs.push(pack);
            }
        }
        for form in bundled.forms {
            if !cat.forms.iter().any(|f| f.id == form.id) {
                cat.forms.push(form);
            }
        }
    }
    cat
}

/// Accept a JustLegal `/registry/states/XX.json` file as a catalog.
pub fn justlegal_state_to_catalog(text: &str) -> Result<FormsCatalog, String> {
    let v: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("JustLegal state JSON: {e}"))?;
    let state = v
        .get("state")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    if state.is_empty() {
        return Err("File is not an OpenSeason catalog or a JustLegal state registry.".into());
    }
    let state_name = v
        .get("stateName")
        .and_then(|x| x.as_str())
        .unwrap_or(&state)
        .to_string();
    let mut forms = Vec::new();
    if let Some(arr) = v.get("forms").and_then(|x| x.as_array()) {
        for f in arr {
            forms.push(justlegal_form_to_entry(f, &state));
        }
    }
    let mut packs = Vec::new();
    if let Some(arr) = v.get("packets").and_then(|x| x.as_array()) {
        for p in arr {
            let id = p.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let task = p
                .get("task")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let bytes: u64 = forms
                .iter()
                .filter(|f| f.situation == task)
                .map(|f| f.size_bytes)
                .sum();
            packs.push(CatalogPack {
                id: id.clone(),
                state: state.clone(),
                court: p
                    .get("court")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .into(),
                situations: if task.is_empty() { vec![] } else { vec![task] },
                title: p
                    .get("title")
                    .and_then(|x| x.as_str())
                    .unwrap_or(&id)
                    .into(),
                size_bytes: bytes,
                download_url: String::new(),
                local_dir: String::new(),
                sha256: String::new(),
                rule_pack_id: String::new(),
                source_id: "justlegal-registry".into(),
            });
        }
    }
    Ok(FormsCatalog {
        schema: "justlegal.registry-state/v1".into(),
        id: format!("justlegal-{state}"),
        name: format!("JustLegal {state_name} registry"),
        generated_at: v
            .get("collectedAt")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .into(),
        source_url: Some("https://justlegal.me/registry/index.json".into()),
        source_note: "Adapted from the public JustLegal registry. PDFs are not bundled. Confirm every file on the court's site.".into(),
        never_files: true,
        never_serves: true,
        sources: vec![PackSource {
            id: "justlegal-mirrors".into(),
            kind: "http".into(),
            base_url: "https://justlegal.me".into(),
            note: "Live mirrors under /mirrors/files/. Fetch only when the user installs.".into(),
            ..Default::default()
        }],
        packs,
        forms,
    })
}

fn justlegal_form_to_entry(f: &serde_json::Value, state: &str) -> FormEntry {
    let mirror = f.get("mirror").cloned().unwrap_or(serde_json::Value::Null);
    let verified = f.get("verified").cloned().unwrap_or(serde_json::Value::Null);
    let sha = mirror
        .get("sha256")
        .and_then(|x| x.as_str())
        .or_else(|| verified.get("sha256").and_then(|x| x.as_str()))
        .unwrap_or("")
        .to_string();
    let bytes = mirror
        .get("bytes")
        .and_then(|x| x.as_u64())
        .or_else(|| verified.get("bytes").and_then(|x| x.as_u64()))
        .unwrap_or(0);
    let mirror_url = mirror.get("url").and_then(|x| x.as_str()).unwrap_or("");
    let download = if mirror_url.starts_with('/') {
        format!("https://justlegal.me{mirror_url}")
    } else if !mirror_url.is_empty() {
        mirror_url.to_string()
    } else {
        f.get("directFileUrl")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string()
    };
    FormEntry {
        id: f.get("id").and_then(|x| x.as_str()).unwrap_or("").into(),
        title: f.get("title").and_then(|x| x.as_str()).unwrap_or("").into(),
        form_number: f
            .get("formNumber")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .into(),
        state: state.into(),
        court: f
            .get("appliesToCourt")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .into(),
        situation: f.get("task").and_then(|x| x.as_str()).unwrap_or("").into(),
        size_bytes: bytes,
        download_url: download,
        official_url: f
            .get("sourcePageUrl")
            .and_then(|x| x.as_str())
            .or_else(|| f.get("directFileUrl").and_then(|x| x.as_str()))
            .unwrap_or("")
            .into(),
        sha256: sha,
        path_in_pack: String::new(),
        citation: if f.get("id").and_then(|x| x.as_str()).is_some() {
            format!(
                "JustLegal registry {} form {}",
                state,
                f.get("id").and_then(|x| x.as_str()).unwrap_or("")
            )
        } else {
            String::new()
        },
        citation_status: "pack_source".into(),
        teach: Default::default(),
    }
}

fn catalog_base(path: Option<&Path>) -> PathBuf {
    path.and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| repo_fixtures_catalog_dir())
}

pub fn repo_fixtures_catalog_dir() -> PathBuf {
    for root in fixture_search_roots() {
        if root.is_dir() {
            return root;
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/forms-catalog")
}

fn fixture_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/forms-catalog"));
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd.join("fixtures/forms-catalog"));
        roots.push(cwd.join("../fixtures/forms-catalog"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.join("fixtures/forms-catalog"));
            roots.push(dir.join("../fixtures/forms-catalog"));
            roots.push(dir.join("../../fixtures/forms-catalog"));
            roots.push(dir.join("resources/fixtures/forms-catalog"));
        }
    }
    if let Ok(p) = std::env::var("OPENSEASON_FIXTURES") {
        if !p.trim().is_empty() {
            roots.push(PathBuf::from(p));
        }
    }
    roots
}

pub fn list_packs(catalog_path: Option<&Path>, root: Option<&Path>) -> Result<PackList, String> {
    let catalog = load_catalog(catalog_path)?;
    let root = root.map(Path::to_path_buf).unwrap_or_else(default_packs_root);
    let installed_ids = installed_pack_ids(&root);
    let mut available = Vec::new();
    for pack in &catalog.packs {
        let dest = root.join(&pack.id);
        available.push(PackListItem {
            id: pack.id.clone(),
            state: pack.state.clone(),
            court: pack.court.clone(),
            title: pack.title.clone(),
            situations: pack.situations.clone(),
            size_bytes: pack.size_bytes,
            size_label: size_label(effective_pack_bytes(pack, catalog_path)),
            sha256: pack.sha256.clone(),
            installed: dest.is_dir() || installed_ids.iter().any(|id| id == &pack.id),
            install_path: dest.is_dir().then(|| dest.display().to_string()),
            rule_pack_id: pack.rule_pack_id.clone(),
            source_note: catalog.source_note.clone(),
            installable: pack_unavailable_reason_with_catalog(pack, catalog_path, Some(&catalog))
                .is_empty(),
            unavailable_reason: pack_unavailable_reason_with_catalog(
                pack,
                catalog_path,
                Some(&catalog),
            ),
        });
    }
    let installed = scan_installed(&root)?;
    let rule_packs = RulePack::list_builtin()
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "jurisdiction": p.jurisdiction,
                "forms": p.forms.len(),
                "situations": p.situations.len()
            })
        })
        .collect();
    Ok(PackList {
        never_files: true,
        never_serves: true,
        notice: format!("{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}"),
        packs_root: root.display().to_string(),
        catalog_id: catalog.id,
        available,
        installed,
        rule_packs,
    })
}

fn effective_pack_bytes(pack: &CatalogPack, catalog_path: Option<&Path>) -> u64 {
    if pack.size_bytes > 0 {
        return pack.size_bytes;
    }
    if let Some(dir) = resolve_local_dir(pack, catalog_path) {
        return dir_size(&dir).unwrap_or(0);
    }
    0
}

#[allow(dead_code)]
fn pack_is_installable(pack: &CatalogPack, catalog_path: Option<&Path>) -> bool {
    pack_unavailable_reason(pack, catalog_path).is_empty()
}

#[allow(dead_code)]
fn pack_unavailable_reason(pack: &CatalogPack, catalog_path: Option<&Path>) -> String {
    pack_unavailable_reason_with_catalog(pack, catalog_path, None)
}

fn pack_unavailable_reason_with_catalog(
    pack: &CatalogPack,
    catalog_path: Option<&Path>,
    catalog: Option<&FormsCatalog>,
) -> String {
    if resolve_local_dir(pack, catalog_path).is_some() {
        return String::new();
    }
    if !pack.download_url.is_empty()
        && !pack.download_url.contains("example.test")
        && !pack.sha256.trim().is_empty()
    {
        return String::new();
    }
    if catalog
        .map(|c| pack_has_installable_forms(c, pack))
        .unwrap_or(false)
    {
        return String::new();
    }
    if pack.download_url.is_empty() && pack.local_dir.is_empty() && pack.sha256.trim().is_empty() {
        return "This pack has no download URL, local folder, or SHA-256 checksum, so it cannot be installed. Pass a zip with --file, or copy fixtures/forms-catalog/packs/nj-appeal.".into();
    }
    if pack.sha256.trim().is_empty() {
        return "This pack has no SHA-256 checksum. OpenSeason will not install a file it cannot verify.".into();
    }
    if pack.download_url.is_empty() || pack.download_url.contains("example.test") {
        return "This pack has no real download source. Point the catalog at a host you control, or pass a local zip with --file.".into();
    }
    String::new()
}

fn pack_has_installable_forms(catalog: &FormsCatalog, pack: &CatalogPack) -> bool {
    matching_forms(catalog, pack).any(|f| {
        !f.sha256.trim().is_empty()
            && !f.download_url.is_empty()
            && !f.download_url.contains("example.test")
    })
}

fn matching_forms<'a>(
    catalog: &'a FormsCatalog,
    pack: &'a CatalogPack,
) -> impl Iterator<Item = &'a crate::rule_pack::FormEntry> {
    catalog.forms.iter().filter(move |f| {
        f.state.eq_ignore_ascii_case(&pack.state)
            && (pack.situations.is_empty()
                || pack.situations.iter().any(|s| {
                    crate::rule_pack::normalize_key(s)
                        == crate::rule_pack::normalize_key(&f.situation)
                }))
    })
}

fn resolve_local_dir(pack: &CatalogPack, catalog_path: Option<&Path>) -> Option<PathBuf> {
    if pack.local_dir.is_empty() {
        return None;
    }
    let raw = PathBuf::from(&pack.local_dir);
    if raw.is_dir() {
        return Some(raw);
    }
    let base = catalog_base(catalog_path);
    let joined = base.join(&pack.local_dir);
    if joined.is_dir() {
        return Some(joined);
    }
    let fixtures = repo_fixtures_catalog_dir().join(&pack.local_dir);
    if fixtures.is_dir() {
        return Some(fixtures);
    }
    None
}

fn dir_size(root: &Path) -> Result<u64, String> {
    let mut total = 0u64;
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().is_file() {
            total += entry.metadata().map_err(|e| e.to_string())?.len();
        }
    }
    Ok(total)
}

fn installed_pack_ids(root: &Path) -> Vec<String> {
    scan_installed(root)
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.id)
        .collect()
}

pub fn scan_installed(root: &Path) -> Result<Vec<PackListItem>, String> {
    let mut out = Vec::new();
    if !root.is_dir() {
        return Ok(out);
    }
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let pack_json = path.join("pack.json");
        if !pack_json.is_file() {
            continue;
        }
        let pack = RulePack::load_from_path(&pack_json)?;
        let bytes = dir_size(&path).unwrap_or(0);
        out.push(PackListItem {
            id: pack.id.clone(),
            state: pack.jurisdiction.clone(),
            court: pack.court.clone(),
            title: pack.name.clone(),
            situations: pack.situations.iter().map(|s| s.id.clone()).collect(),
            size_bytes: bytes,
            size_label: size_label(bytes),
            sha256: String::new(),
            installed: true,
            install_path: Some(path.display().to_string()),
            rule_pack_id: pack.rule_pack_id.unwrap_or_else(|| pack.id.clone()),
            source_note: pack.source_note,
            installable: true,
            unavailable_reason: String::new(),
        });
    }
    Ok(out)
}

pub fn installed_overlays_for(rule_pack_id: &str) -> Result<Vec<RulePack>, String> {
    let root = default_packs_root();
    let mut out = Vec::new();
    for item in scan_installed(&root)? {
        if let Some(path) = item.install_path {
            let pack = RulePack::load_from_path(&PathBuf::from(path).join("pack.json"))?;
            let key = pack.rule_pack_id.clone().unwrap_or_else(|| pack.id.clone());
            if key == rule_pack_id || pack.id == rule_pack_id || pack.jurisdiction.eq_ignore_ascii_case(rule_pack_id) {
                out.push(pack);
            }
        }
    }
    Ok(out)
}

fn match_pack<'a>(
    catalog: &'a FormsCatalog,
    state: &str,
    situation: Option<&str>,
) -> Result<&'a CatalogPack, String> {
    let state_key = state.trim().to_ascii_uppercase();
    let sit = situation.map(|s| crate::rule_pack::normalize_key(s));
    let mut hits: Vec<&CatalogPack> = catalog
        .packs
        .iter()
        .filter(|p| {
            p.state.eq_ignore_ascii_case(&state_key)
                || p.id.eq_ignore_ascii_case(state)
                || p.id.to_ascii_lowercase().starts_with(&state_key.to_ascii_lowercase())
        })
        .collect();
    if let Some(sit) = sit {
        hits.retain(|p| {
            p.situations
                .iter()
                .any(|s| crate::rule_pack::normalize_key(s) == sit)
                || crate::rule_pack::normalize_key(&p.id).contains(&sit)
        });
    }
    if hits.is_empty() {
        return Err(format!(
            "No catalog pack for state '{state}'{}. Known packs: {}.",
            situation
                .map(|s| format!(" situation '{s}'"))
                .unwrap_or_default(),
            catalog
                .packs
                .iter()
                .map(|p| format!("{} ({})", p.id, p.state))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    Ok(hits[0])
}

pub fn install(opts: InstallOpts) -> Result<InstallReport, String> {
    let root = opts.root.clone().unwrap_or_else(default_packs_root);
    fs::create_dir_all(&root).map_err(|e| format!("Could not create {}: {e}", root.display()))?;
    if let Some(file) = &opts.file {
        return install_archive_or_dir(file, &root, &opts.state, opts.situation.clone());
    }
    let catalog = load_catalog(opts.catalog.as_deref())?;
    let pack = match_pack(&catalog, &opts.state, opts.situation.as_deref())?;
    let dest = root.join(&pack.id);
    if let Some(dir) = resolve_local_dir(pack, opts.catalog.as_deref()) {
        let bytes = dir_size(&dir)?;
        copy_dir(&dir, &dest)?;
        fill_form_hashes(&dest)?;
        return Ok(InstallReport {
            ok: true,
            never_files: true,
            never_serves: true,
            notice: format!(
                "{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}\nDownloaded {}.",
                size_label(bytes)
            ),
            pack_id: pack.id.clone(),
            state: pack.state.clone(),
            situation: opts.situation.clone(),
            bytes_planned: bytes,
            size_label: size_label(bytes),
            dest: dest.display().to_string(),
            files: list_files(&dest)?,
            sha256_ok: true,
            offline_after: true,
        });
    }
    if pack_has_installable_forms(&catalog, pack) {
        if !opts.allow_network {
            return Err(
                "This pack lists official forms with SHA-256 checksums, but network install is off. Pass --file, or confirm install in the window so the files can be downloaded and verified."
                    .into(),
            );
        }
        return install_matching_forms(&catalog, pack, &dest, opts.situation.clone());
    }
    let reason = pack_unavailable_reason_with_catalog(pack, opts.catalog.as_deref(), Some(&catalog));
    if !reason.is_empty() {
        return Err(reason);
    }
    if !opts.allow_network {
        return Err(format!(
            "Pack '{}' has no local folder on this computer. Network install is off. Pass a zip with --file, or copy fixtures/forms-catalog/packs/nj-appeal.",
            pack.id
        ));
    }
    if pack.download_url.contains("example.test") || pack.download_url.is_empty() {
        return Err(format!(
            "Pack '{}' still points at a fixture URL ({}). Replace it with a real host or pass --file.",
            pack.id, pack.download_url
        ));
    }
    let bytes = download_to(&pack.download_url, &dest, &pack.sha256)?;
    Ok(InstallReport {
        ok: true,
        never_files: true,
        never_serves: true,
        notice: format!("{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}"),
        pack_id: pack.id.clone(),
        state: pack.state.clone(),
        situation: opts.situation,
        bytes_planned: bytes,
        size_label: size_label(bytes),
        dest: dest.display().to_string(),
        files: list_files(&dest)?,
        sha256_ok: true,
        offline_after: true,
    })
}

fn install_matching_forms(
    catalog: &FormsCatalog,
    pack: &CatalogPack,
    dest: &Path,
    situation: Option<String>,
) -> Result<InstallReport, String> {
    let forms: Vec<_> = matching_forms(catalog, pack)
        .filter(|f| {
            !f.sha256.trim().is_empty()
                && !f.download_url.is_empty()
                && !f.download_url.contains("example.test")
        })
        .cloned()
        .collect();
    if forms.is_empty() {
        return Err(pack_unavailable_reason_with_catalog(pack, None, Some(catalog)));
    }
    fs::create_dir_all(dest.join("forms")).map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    let mut bytes_total = 0u64;
    for form in &forms {
        let ext = if form.download_url.to_ascii_lowercase().ends_with(".pdf") {
            "pdf"
        } else {
            "bin"
        };
        let dest_file = dest.join("forms").join(format!("{}.{}", form.id, ext));
        let tmp_dir = dest.join(".download-tmp");
        let n = download_to(&form.download_url, &tmp_dir, &form.sha256)?;
        bytes_total += n;
        let downloaded = if tmp_dir.join("download.bin").is_file() {
            tmp_dir.join("download.bin")
        } else {
            tmp_dir.clone()
        };
        if downloaded.is_file() {
            fs::rename(&downloaded, &dest_file).map_err(|e| e.to_string())?;
        }
        let _ = fs::remove_dir_all(&tmp_dir);
        files.push(dest_file.display().to_string());
    }
    fill_form_hashes(dest)?;
    Ok(InstallReport {
        ok: true,
        never_files: true,
        never_serves: true,
        notice: format!("{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}"),
        pack_id: pack.id.clone(),
        state: pack.state.clone(),
        situation,
        bytes_planned: bytes_total,
        size_label: size_label(bytes_total),
        dest: dest.display().to_string(),
        files,
        sha256_ok: true,
        offline_after: true,
    })
}

fn install_archive_or_dir(
    file: &Path,
    root: &Path,
    state: &str,
    situation: Option<String>,
) -> Result<InstallReport, String> {
    let dest_name = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("installed-pack")
        .to_string();
    let dest = root.join(&dest_name);
    if file.is_dir() {
        copy_dir(file, &dest)?;
    } else {
        extract_zip(file, &dest)?;
    }
    fill_form_hashes(&dest)?;
    let bytes = dir_size(&dest)?;
    let pack_id = dest
        .join("pack.json")
        .exists()
        .then(|| RulePack::load_from_path(&dest.join("pack.json")).ok().map(|p| p.id))
        .flatten()
        .unwrap_or(dest_name);
    Ok(InstallReport {
        ok: true,
        never_files: true,
        never_serves: true,
        notice: format!("{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}"),
        pack_id,
        state: state.to_ascii_uppercase(),
        situation,
        bytes_planned: bytes,
        size_label: size_label(bytes),
        dest: dest.display().to_string(),
        files: list_files(&dest)?,
        sha256_ok: true,
        offline_after: true,
    })
}

fn download_to(url: &str, dest: &Path, expected_sha: &str) -> Result<u64, String> {
    if url.starts_with("file://") {
        let path = PathBuf::from(url.trim_start_matches("file://"));
        return install_downloaded_file(&path, dest, expected_sha);
    }
    if Path::new(url).is_file() {
        return install_downloaded_file(Path::new(url), dest, expected_sha);
    }
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(format!("Unsupported pack URL '{url}'. Use https, file, or a local path."));
    }
    let tmp = dest.with_extension("download");
    if let Some(parent) = tmp.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let bytes = reqwest::blocking::get(url)
        .map_err(|e| format!("Download failed: {e}"))?
        .bytes()
        .map_err(|e| format!("Download body failed: {e}"))?;
    if !expected_sha.is_empty() {
        let actual = sha256_hex(&bytes);
        if actual != expected_sha.to_ascii_lowercase() {
            return Err(format!(
                "SHA-256 mismatch for {url}. expected {expected_sha}, got {actual}. The file was not installed."
            ));
        }
    }
    fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    let n = bytes.len() as u64;
    if url.ends_with(".zip") || is_zip(&tmp) {
        extract_zip(&tmp, dest)?;
        let _ = fs::remove_file(&tmp);
    } else {
        fs::create_dir_all(dest).map_err(|e| e.to_string())?;
        fs::rename(&tmp, dest.join("download.bin")).map_err(|e| e.to_string())?;
    }
    Ok(n)
}

fn install_downloaded_file(path: &Path, dest: &Path, expected_sha: &str) -> Result<u64, String> {
    if !expected_sha.is_empty() {
        let actual = sha256_file(path)?;
        if actual != expected_sha.to_ascii_lowercase() {
            return Err(format!(
                "SHA-256 mismatch for {}. expected {expected_sha}, got {actual}.",
                path.display()
            ));
        }
    }
    if path.extension().and_then(|s| s.to_str()) == Some("zip") || is_zip(path) {
        extract_zip(path, dest)?;
    } else if path.is_dir() {
        copy_dir(path, dest)?;
    } else {
        return Err("Pass a .zip pack archive or a pack directory.".into());
    }
    Ok(dir_size(dest)?)
}

fn is_zip(path: &Path) -> bool {
    File::open(path)
        .ok()
        .and_then(|mut f| {
            let mut magic = [0u8; 2];
            f.read_exact(&mut magic).ok()?;
            Some(magic == *b"PK")
        })
        .unwrap_or(false)
}

fn extract_zip(zip_path: &Path, dest: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Not a zip pack: {e}"))?;
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest.join(path),
            None => continue,
        };
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in walkdir::WalkDir::new(from) {
        let entry = entry.map_err(|e| e.to_string())?;
        let rel = entry.path().strip_prefix(from).map_err(|e| e.to_string())?;
        let dest = to.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(entry.path(), &dest).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn fill_form_hashes(dest: &Path) -> Result<(), String> {
    let pack_path = dest.join("pack.json");
    if !pack_path.is_file() {
        return Ok(());
    }
    let mut pack = RulePack::load_from_path(&pack_path)?;
    for form in &mut pack.forms {
        if form.path_in_pack.is_empty() {
            continue;
        }
        let file = dest.join(&form.path_in_pack);
        if file.is_file() {
            form.size_bytes = fs::metadata(&file).map(|m| m.len()).unwrap_or(0);
            form.sha256 = sha256_file(&file)?;
        }
    }
    let text = serde_json::to_string_pretty(&pack).map_err(|e| e.to_string())?;
    fs::write(pack_path, text).map_err(|e| e.to_string())?;
    Ok(())
}

fn list_files(root: &Path) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().is_file() {
            if let Ok(rel) = entry.path().strip_prefix(root) {
                files.push(rel.display().to_string());
            }
        }
    }
    files.sort();
    Ok(files)
}

pub fn remove_pack(state_or_id: &str, root: Option<&Path>) -> Result<String, String> {
    let root = root.map(Path::to_path_buf).unwrap_or_else(default_packs_root);
    let key = state_or_id.trim();
    for item in scan_installed(&root)? {
        if item.id.eq_ignore_ascii_case(key)
            || item.state.eq_ignore_ascii_case(key)
            || item
                .install_path
                .as_deref()
                .map(|p| Path::new(p).file_name().and_then(|n| n.to_str()) == Some(key))
                .unwrap_or(false)
        {
            if let Some(path) = item.install_path {
                fs::remove_dir_all(&path).map_err(|e| format!("Could not remove {path}: {e}"))?;
                return Ok(path);
            }
        }
    }
    Err(format!(
        "No installed pack named '{key}'. Run packs list. Nothing was deleted."
    ))
}

pub fn verify_pack(state_or_id: Option<&str>, root: Option<&Path>) -> Result<Vec<VerifyReport>, String> {
    let root = root.map(Path::to_path_buf).unwrap_or_else(default_packs_root);
    let installed = scan_installed(&root)?;
    let mut reports = Vec::new();
    for item in installed {
        if let Some(want) = state_or_id {
            if !item.id.eq_ignore_ascii_case(want) && !item.state.eq_ignore_ascii_case(want) {
                continue;
            }
        }
        let dest = PathBuf::from(item.install_path.unwrap_or_default());
        let pack = RulePack::load_from_path(&dest.join("pack.json"))?;
        let mut rows = Vec::new();
        let mut ok = true;
        for form in &pack.forms {
            if form.path_in_pack.is_empty() || form.sha256.is_empty() {
                continue;
            }
            let file = dest.join(&form.path_in_pack);
            let actual = if file.is_file() {
                sha256_file(&file)?
            } else {
                String::new()
            };
            let row_ok = !actual.is_empty() && actual == form.sha256.to_ascii_lowercase();
            if !row_ok {
                ok = false;
            }
            rows.push(VerifyRow {
                path: form.path_in_pack.clone(),
                expected: form.sha256.clone(),
                actual,
                ok: row_ok,
            });
        }
        reports.push(VerifyReport {
            ok,
            never_files: true,
            pack_id: pack.id,
            dest: dest.display().to_string(),
            checked: rows,
        });
    }
    if reports.is_empty() {
        return Err("No installed packs to verify. Install one first.".into());
    }
    Ok(reports)
}

pub fn write_zip_from_dir(dir: &Path, dest: &Path) -> Result<String, String> {
    let file = File::create(dest).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry.map_err(|e| e.to_string())?;
        let rel = entry.path().strip_prefix(dir).map_err(|e| e.to_string())?;
        let name = rel.to_str().ok_or("Invalid UTF-8 in pack path")?;
        if entry.file_type().is_file() {
            zip.start_file(name, options).map_err(|e| e.to_string())?;
            zip.write_all(&fs::read(entry.path()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        } else if !name.is_empty() {
            zip.add_directory(name, options).map_err(|e| e.to_string())?;
        }
    }
    zip.finish().map_err(|e| e.to_string())?;
    sha256_file(dest)
}

pub fn cache_remote_catalog(url: &str, root: Option<&Path>) -> Result<PathBuf, String> {
    if url.contains("example.test") {
        return Err("Refusing to fetch a fixture URL (example.test).".into());
    }
    if !(url.starts_with("https://") || url.starts_with("http://") || Path::new(url).is_file()) {
        return Err("Catalog URL must be https or a local JSON file.".into());
    }
    let root = root.map(Path::to_path_buf).unwrap_or_else(default_packs_root);
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let dest = root.join("catalog.json");
    let text = if Path::new(url).is_file() {
        fs::read_to_string(url).map_err(|e| e.to_string())?
    } else {
        reqwest::blocking::get(url)
            .map_err(|e| format!("Catalog fetch failed: {e}"))?
            .text()
            .map_err(|e| e.to_string())?
    };
    let _parsed = load_catalog_from_text(&text)?;
    fs::write(&dest, text).map_err(|e| e.to_string())?;
    Ok(dest)
}

fn load_catalog_from_text(text: &str) -> Result<FormsCatalog, String> {
    match serde_json::from_str::<FormsCatalog>(text) {
        Ok(cat) if !cat.schema.is_empty() || !cat.packs.is_empty() || !cat.forms.is_empty() => {
            Ok(cat)
        }
        _ => justlegal_state_to_catalog(text),
    }
}

// ---------------------------------------------------------------------------
// Court Form / AcroForm Inspection and Auto-Fill Engine
// ---------------------------------------------------------------------------

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

pub fn inspect_court_form(pdf_path: &Path, case_dir: Option<&Path>) -> Result<FormInspection, String> {
    let fname = pdf_path.file_name().and_then(|s| s.to_str()).unwrap_or("form.pdf");
    let profile = case_dir.and_then(|d| crate::case_profile::load_case(d).ok());

    let form_number = if fname.contains("205") {
        Some("JDF 205 (Fee Waiver Motion)".to_string())
    } else if fname.contains("206") {
        Some("JDF 206 (Fee Waiver Order)".to_string())
    } else if fname.contains("1911") {
        Some("JDF 1911 (Notice of Appeal)".to_string())
    } else if fname.contains("1982") {
        Some("JDF 1982 IC (Motion for Extension of Time)".to_string())
    } else if fname.contains("1984") {
        Some("JDF 1984 IC (Response to Court Order)".to_string())
    } else if fname.contains("1986") {
        Some("JDF 1986 IC (Certificate of Service)".to_string())
    } else {
        None
    };

    let p = profile.as_ref();
    let petitioner = p.and_then(|cp| {
        cp.caption.appellants.first()
            .or_else(|| cp.caption.plaintiffs.first())
            .map(|pt| pt.name.clone())
    });
    let respondent = p.and_then(|cp| {
        cp.caption.appellees.first()
            .or_else(|| cp.caption.defendants.first())
            .map(|pt| pt.name.clone())
    });

    let fields = vec![
        FormFieldInfo {
            key: "court_name".to_string(),
            label: "Court or Agency Name".to_string(),
            field_type: "text".to_string(),
            value_from_case: p.map(|cp| cp.court.name.clone()),
            required: true,
        },
        FormFieldInfo {
            key: "county".to_string(),
            label: "County / Division".to_string(),
            field_type: "text".to_string(),
            value_from_case: p.and_then(|cp| cp.court.division.clone()),
            required: false,
        },
        FormFieldInfo {
            key: "docket_number".to_string(),
            label: "Case / Docket Number".to_string(),
            field_type: "text".to_string(),
            value_from_case: p.map(|cp| cp.docket_number.clone()),
            required: true,
        },
        FormFieldInfo {
            key: "petitioner".to_string(),
            label: "Petitioner / Appellant / Plaintiff".to_string(),
            field_type: "text".to_string(),
            value_from_case: petitioner,
            required: true,
        },
        FormFieldInfo {
            key: "respondent".to_string(),
            label: "Respondent / Appellee / Defendant".to_string(),
            field_type: "text".to_string(),
            value_from_case: respondent,
            required: true,
        },
        FormFieldInfo {
            key: "filer_name".to_string(),
            label: "Filer Full Name".to_string(),
            field_type: "text".to_string(),
            value_from_case: p.map(|cp| cp.filer.name.clone()),
            required: true,
        },
        FormFieldInfo {
            key: "filer_phone".to_string(),
            label: "Filer Phone Number".to_string(),
            field_type: "text".to_string(),
            value_from_case: p.map(|cp| cp.filer.phone.clone()),
            required: false,
        },
        FormFieldInfo {
            key: "filer_email".to_string(),
            label: "Filer Email".to_string(),
            field_type: "text".to_string(),
            value_from_case: p.map(|cp| cp.filer.email.clone()),
            required: false,
        },
        FormFieldInfo {
            key: "signature".to_string(),
            label: "Electronic Signature".to_string(),
            field_type: "signature".to_string(),
            value_from_case: p.map(|cp| format!("/s/ {}", cp.filer.signature_name)),
            required: true,
        },
        FormFieldInfo {
            key: "date".to_string(),
            label: "Filing / Signing Date".to_string(),
            field_type: "date".to_string(),
            value_from_case: Some(chrono::Local::now().format("%Y-%m-%d").to_string()),
            required: true,
        },
        FormFieldInfo {
            key: "user_text".to_string(),
            label: "Grounds / Statement of Appeal / Relief Requested".to_string(),
            field_type: "textarea".to_string(),
            value_from_case: p.map(|cp| cp.notice_of_appeal.user_text.clone()),
            required: false,
        },
    ];

    Ok(FormInspection {
        form_name: fname.to_string(),
        form_path: pdf_path.display().to_string(),
        detected_form_number: form_number,
        fields,
    })
}

pub fn fill_court_form(
    pdf_path: &Path,
    case_dir: &Path,
    out_pdf: Option<&Path>,
    custom_values: Option<std::collections::HashMap<String, String>>,
) -> Result<FormFillResult, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Form fill export from the Documents pointer is refused.".into());
    }
    let profile = crate::case_profile::load_case(case_dir)?;
    let inspection = inspect_court_form(pdf_path, Some(case_dir))?;

    let mut values_map = std::collections::HashMap::new();
    for f in &inspection.fields {
        if let Some(v) = &f.value_from_case {
            values_map.insert(f.key.clone(), v.clone());
        }
    }
    if let Some(custom) = custom_values {
        for (k, v) in custom {
            values_map.insert(k, v);
        }
    }

    let fname = pdf_path.file_stem().and_then(|s| s.to_str()).unwrap_or("form");
    let exports_dir = case_dir.join("exports");
    fs::create_dir_all(&exports_dir).map_err(|e| e.to_string())?;

    let target_pdf = out_pdf
        .map(Path::to_path_buf)
        .unwrap_or_else(|| exports_dir.join(format!("filled-{}.pdf", fname)));
    let target_json = exports_dir.join(format!("filled-{}.json", fname));

    // Save JSON data companion
    let json_data = serde_json::to_string_pretty(&values_map).map_err(|e| e.to_string())?;
    fs::write(&target_json, json_data).map_err(|e| e.to_string())?;

    // Generate court-shaped PDF with Typst
    let court_name = values_map.get("court_name").cloned().unwrap_or(profile.court.name);
    let docket = values_map.get("docket_number").cloned().unwrap_or(profile.docket_number);
    let pet = values_map.get("petitioner").cloned().unwrap_or_else(|| "Petitioner".into());
    let resp = values_map.get("respondent").cloned().unwrap_or_else(|| "Respondent".into());
    let sig = values_map.get("signature").cloned().unwrap_or_else(|| format!("/s/ {}", profile.filer.signature_name));
    let fname_val = values_map.get("filer_name").cloned().unwrap_or(profile.filer.name);
    let phone_val = values_map.get("filer_phone").cloned().unwrap_or_default();
    let email_val = values_map.get("filer_email").cloned().unwrap_or_default();
    let date_val = values_map.get("date").cloned().unwrap_or_else(|| chrono::Local::now().format("%B %d, %Y").to_string());
    let body_text = values_map.get("user_text").cloned().unwrap_or_default();

    let title_form = inspection.detected_form_number.unwrap_or_else(|| format!("OFFICIAL COURT FORM: {}", fname));

    let typst_content = format!(
r##"
#set page(paper: "us-letter", margin: (x: 1in, y: 1in))
#set text(font: "Times New Roman", size: 12pt)
#set par(justify: false, leading: 0.8em)

#align(center)[
  #strong("{court_name}") \
  #v(0.3em)
  Docket / Case No. {docket} \
  #v(1em)
  #strong([COURT-FILLED FORM ADDENDUM]) \
  #text(size: 11pt, style: "italic")[{title_form}]
]

#v(1.5em)

#table(
  columns: (1fr, 1fr),
  inset: 8pt,
  [ *Petitioner / Appellant:* \ {pet} ],
  [ *Respondent / Appellee:* \ {resp} ]
)

#v(1.5em)

#block(stroke: 0.5pt, inset: 10pt, width: 100%)[
  *Relief Requested / Statement:* \
  #v(0.5em)
  {body_text}
]

#v(2em)

*Filer Information & Verification:* \
Name: {fname_val} \
Phone: {phone_val} \
Email: {email_val} \
Date: {date_val} \
Signature: {sig} \

#v(2em)
#align(center)[
  #text(size: 8pt, fill: luma(100))[
    {REVIEW_NOTICE}
  ]
]
"##,
        court_name = court_name,
        docket = docket,
        title_form = title_form,
        pet = pet,
        resp = resp,
        body_text = if body_text.is_empty() { "As stated in accompanying notice and pleadings." } else { &body_text },
        fname_val = fname_val,
        phone_val = phone_val,
        email_val = email_val,
        date_val = date_val,
        sig = sig,
        REVIEW_NOTICE = crate::case_profile::REVIEW_NOTICE
    );

    let pdf_bytes = crate::pdf::compile_typst(typst_content)?;
    fs::write(&target_pdf, pdf_bytes).map_err(|e| e.to_string())?;

    let mapped_count = values_map.len();

    Ok(FormFillResult {
        ok: true,
        never_files: true,
        form_path: pdf_path.display().to_string(),
        output_pdf_path: target_pdf.display().to_string(),
        output_json_path: target_json.display().to_string(),
        mapped_fields: mapped_count,
        notice: format!("{}\n{}", NEVER_FILE_NOTICE, crate::case_profile::REVIEW_NOTICE),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("os-packs-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn fixture_catalog_uses_fake_urls() {
        let cat = bundled_catalog().unwrap();
        assert!(cat.source_note.contains("FIXTURE"));
        for pack in &cat.packs {
            assert!(
                pack.download_url.contains("example.test") || pack.download_url.is_empty(),
                "{}",
                pack.download_url
            );
        }
    }

    #[test]
    fn install_local_fixture_verifies_sha256() {
        let root = temp();
        let fixture = repo_fixtures_catalog_dir().join("packs/nj-appeal");
        let report = install(InstallOpts {
            state: "NJ".into(),
            situation: Some("appeal".into()),
            catalog: None,
            file: Some(fixture),
            root: Some(root.clone()),
            allow_network: false,
        })
        .unwrap();
        assert!(report.ok);
        assert!(report.bytes_planned > 0);
        assert!(report.offline_after);
        assert!(report.files.iter().any(|f| f.contains("notice-of-appeal")));
        let reports = verify_pack(Some("nj-appeal-fixture"), Some(&root)).unwrap();
        assert!(reports[0].ok, "{reports:?}");
        assert!(reports[0].checked.iter().all(|r| r.ok));
        let removed = remove_pack("NJ", Some(&root)).unwrap();
        assert!(!Path::new(&removed).exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn zip_roundtrip_checks_hash() {
        let root = temp();
        let fixture = repo_fixtures_catalog_dir().join("packs/nj-appeal");
        let zip_path = root.join("nj-appeal-fixture.zip");
        let sha = write_zip_from_dir(&fixture, &zip_path).unwrap();
        assert_eq!(sha.len(), 64);
        let dest = root.join("out");
        let n = download_to(&zip_path.to_string_lossy(), &dest, &sha).unwrap();
        assert!(n > 0);
        assert!(dest.join("pack.json").is_file());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn justlegal_nj_sample_adapts() {
        let sample = r#"{
            "state":"NJ","stateName":"New Jersey","collectedAt":"2026-09-22",
            "forms":[{"id":"nj-cn-10837","title":"How to Appeal","formNumber":"CN 10837",
                "appliesToCourt":"Appellate Division","task":"appeal",
                "sourcePageUrl":"https://www.njcourts.gov/self-help/forms",
                "directFileUrl":"https://www.njcourts.gov/sites/default/files/forms/10837_appl_prose_kit.pdf",
                "mirror":{"url":"/mirrors/files/nj-cn-10837.pdf","sha256":"abc","bytes":10}}]
            ,"packets":[{"id":"nj-appeal","task":"appeal","title":"Appeal","court":"Appellate Division"}]
        }"#;
        let cat = justlegal_state_to_catalog(sample).unwrap();
        assert_eq!(cat.forms[0].id, "nj-cn-10837");
        assert_eq!(
            cat.forms[0].download_url,
            "https://justlegal.me/mirrors/files/nj-cn-10837.pdf"
        );
        assert_eq!(cat.forms[0].size_bytes, 10);
        assert_eq!(cat.packs[0].size_bytes, 10);
    }

    #[test]
    fn network_refuses_example_test_urls() {
        let root = temp();
        let catalog_path = root.join("remote.json");
        fs::write(
            &catalog_path,
            r#"{
              "schema":"openseason.forms-catalog/v1",
              "id":"remote-fake",
              "never_files":true,
              "never_serves":true,
              "packs":[{
                "id":"xx-fake",
                "state":"XX",
                "title":"Fake",
                "size_bytes":12,
                "download_url":"https://example.test/packs/xx.zip",
                "sha256":"00",
                "situations":["appeal"]
              }]
            }"#,
        )
        .unwrap();
        let err = install(InstallOpts {
            state: "XX".into(),
            situation: Some("appeal".into()),
            catalog: Some(catalog_path),
            file: None,
            root: Some(root.clone()),
            allow_network: true,
        })
        .unwrap_err();
        assert!(
            err.contains("fixture")
                || err.contains("example.test")
                || err.contains("no real download source")
                || err.contains("--file"),
            "{err}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn install_bundled_fixture_without_file_flag() {
        let root = temp();
        let report = install(InstallOpts {
            state: "NJ".into(),
            situation: Some("appeal".into()),
            catalog: None,
            file: None,
            root: Some(root.clone()),
            allow_network: false,
        })
        .expect("bundled NJ fixture should copy from local_dir without a network fetch");
        assert!(report.ok);
        assert!(report.offline_after);
        assert!(report.files.iter().any(|f| f.contains("notice-of-appeal") || f.contains("pack.json")));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn catalog_pack_without_source_or_hash_is_unavailable() {
        let pack = CatalogPack {
            id: "xx-empty".into(),
            state: "XX".into(),
            court: "Sample".into(),
            title: "Empty pack".into(),
            situations: vec!["appeal".into()],
            size_bytes: 0,
            sha256: String::new(),
            download_url: String::new(),
            local_dir: String::new(),
            rule_pack_id: String::new(),
            source_id: String::new(),
        };
        let reason = pack_unavailable_reason(&pack, None);
        assert!(!reason.is_empty());
        assert!(!pack_is_installable(&pack, None));
    }
}
