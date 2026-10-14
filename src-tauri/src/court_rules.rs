use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const GENERIC_RULES_ID: &str = "generic-us-appellate";
pub const GENERIC_RULES_JSON: &str = include_str!("../../templates/court-rules/generic.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarginInches {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CourtRules {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_paper")]
    pub paper: String,
    #[serde(default = "default_page_width")]
    pub page_width_twips: u32,
    #[serde(default = "default_page_height")]
    pub page_height_twips: u32,
    pub margins_inches: MarginInches,
    pub font_family: String,
    #[serde(default)]
    pub font_fallbacks: Vec<String>,
    pub font_size_pt: u32,
    pub line_spacing: f64,
    #[serde(default = "default_caption_align")]
    pub caption_align: String,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub source_retrieved_on: Option<String>,
}

fn default_paper() -> String {
    "us-letter".to_string()
}
fn default_page_width() -> u32 {
    12240
}
fn default_page_height() -> u32 {
    15840
}
fn default_caption_align() -> String {
    "center".to_string()
}

impl CourtRules {
    pub fn generic() -> Self {
        serde_json::from_str(GENERIC_RULES_JSON)
            .expect("bundled generic court rules must be valid JSON")
    }

    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path)
            .map_err(|e| format!("Could not read court rules at {}: {}", path.display(), e))?;
        serde_json::from_str(&text)
            .map_err(|e| format!("Court rules file is not valid JSON: {}", e))
    }

    pub fn load_for_case(case_dir: &Path) -> Self {
        let preferred = case_dir.join("court-rules").join("generic.json");
        if preferred.exists() {
            if let Ok(rules) = Self::load_from_path(&preferred) {
                return rules;
            }
        }
        Self::generic()
    }

    pub fn typst_font_list(&self) -> String {
        let mut fonts = Vec::with_capacity(1 + self.font_fallbacks.len());
        fonts.push(format!("\"{}\"", self.font_family));
        for fallback in &self.font_fallbacks {
            fonts.push(format!("\"{}\"", fallback));
        }
        fonts.join(", ")
    }

    pub fn line_spacing_twips(&self) -> u32 {
        // Word single spacing is 240 twips. Double is 480.
        (240.0 * self.line_spacing).round() as u32
    }

    pub fn font_size_half_points(&self) -> u32 {
        self.font_size_pt * 2
    }

    pub fn margin_twips(&self) -> (u32, u32, u32, u32) {
        let inch = 1440.0;
        (
            (self.margins_inches.top * inch).round() as u32,
            (self.margins_inches.right * inch).round() as u32,
            (self.margins_inches.bottom * inch).round() as u32,
            (self.margins_inches.left * inch).round() as u32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_generic_rules_parse() {
        let rules = CourtRules::generic();
        assert_eq!(rules.id, GENERIC_RULES_ID);
        assert_eq!(rules.font_size_pt, 12);
        assert_eq!(rules.line_spacing, 2.0);
        assert!(rules.source_url.is_none());
    }
}
