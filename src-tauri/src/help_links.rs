//! JustLegal help / resource links. CLI and MCP return data only. Never open a browser.

use serde::{Deserialize, Serialize};

pub const LINKS_JSON: &str = include_str!("../../templates/justlegal-links.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HelpLink {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub description: String,
    pub url: String,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HelpLinks {
    pub schema: String,
    pub verified_on: String,
    pub source_note: String,
    pub offline_note: String,
    pub confirm: String,
    #[serde(default)]
    pub never_files: bool,
    #[serde(default)]
    pub never_serves: bool,
    pub links: Vec<HelpLink>,
    /// Always false: agents and the CLI must not open these URLs.
    #[serde(default)]
    pub opens_browser: bool,
}

impl HelpLinks {
    pub fn load() -> Result<Self, String> {
        let mut set: HelpLinks =
            serde_json::from_str(LINKS_JSON).map_err(|e| format!("justlegal-links.json: {e}"))?;
        set.opens_browser = false;
        set.never_files = true;
        set.never_serves = true;
        Ok(set)
    }

    pub fn for_stuck() -> Result<Self, String> {
        Self::load()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_data_only_and_use_verified_justlegal_urls() {
        let set = HelpLinks::load().unwrap();
        assert!(!set.opens_browser);
        assert!(set.never_files && set.never_serves);
        assert!(set.confirm.contains("web browser"));
        assert!(set.confirm.contains("internet"));
        let ids: Vec<_> = set.links.iter().map(|l| l.id.as_str()).collect();
        assert!(ids.contains(&"find-legal-help"));
        assert!(ids.contains(&"knowledge-hub"));
        assert!(ids.contains(&"registry"));
        for link in &set.links {
            assert!(link.verified, "{}", link.id);
            assert!(
                link.url.starts_with("https://justlegal.me/"),
                "{}",
                link.url
            );
            assert!(!link.url.contains("example.test"));
        }
    }
}
