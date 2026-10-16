//! Authoritative forms catalog and state registry adapters.

use std::fs;
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use crate::rule_pack::FormEntry;
use super::types::{CatalogPack, FormsCatalog, PackSource, FIXTURE_CATALOG_JSON};

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

pub fn default_library_root() -> PathBuf {
    dirs::document_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("JustLegal")
        .join("Library")
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
    let library_cached = default_library_root().join("catalog.json");
    if library_cached.is_file() {
        return load_catalog(Some(&library_cached));
    }
    bundled_catalog()
}

/// Keep the offline fixture pack visible even when a cached JustLegal catalog is in use.
pub fn merge_bundled_fixture_packs(mut cat: FormsCatalog) -> FormsCatalog {
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

pub fn justlegal_form_to_entry(f: &serde_json::Value, state: &str) -> FormEntry {
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

pub fn catalog_base(path: Option<&Path>) -> PathBuf {
    path.and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(repo_fixtures_catalog_dir)
}

pub fn repo_fixtures_catalog_dir() -> PathBuf {
    for root in fixture_search_roots() {
        if root.is_dir() {
            return root;
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/forms-catalog")
}

pub fn fixture_search_roots() -> Vec<PathBuf> {
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

pub fn load_catalog_from_text(text: &str) -> Result<FormsCatalog, String> {
    match serde_json::from_str::<FormsCatalog>(text) {
        Ok(cat) if !cat.schema.is_empty() || !cat.packs.is_empty() || !cat.forms.is_empty() => {
            Ok(cat)
        }
        _ => justlegal_state_to_catalog(text),
    }
}
