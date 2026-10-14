use serde::{Deserialize, Serialize};
use std::path::Path;

pub const NJ_APP_DIV_JSON: &str = include_str!("../../templates/rule-packs/nj-app-div.json");
pub const CO_MANDAMUS_JSON: &str = include_str!("../../templates/rule-packs/co-mandamus.json");
pub const GENERIC_PACK_JSON: &str = include_str!("../../templates/rule-packs/generic.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeadlineRule {
    pub id: String,
    pub label: String,
    pub rule: String,
    pub trigger: String,
    pub days: i64,
    #[serde(default = "default_count")]
    pub count: String,
    #[serde(default = "default_roll")]
    pub weekend_holiday_roll: String,
    #[serde(default)]
    pub enlargeable_under: Option<String>,
    #[serde(default)]
    pub tolls_appeal_under: Option<String>,
    #[serde(default)]
    pub notes: String,
}

fn default_count() -> String {
    "calendar".into()
}
fn default_roll() -> String {
    "next_business".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct KnowledgeEntry {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RulePack {
    pub id: String,
    pub name: String,
    pub jurisdiction: String,
    pub court: String,
    #[serde(default = "default_holiday_set")]
    pub holiday_set: String,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub source_retrieved_on: Option<String>,
    #[serde(default)]
    pub source_note: String,
    #[serde(default)]
    pub never_files: bool,
    #[serde(default)]
    pub never_serves: bool,
    #[serde(default)]
    pub deadlines: Vec<DeadlineRule>,
    #[serde(default)]
    pub knowledge_base: Vec<KnowledgeEntry>,
}

fn default_holiday_set() -> String {
    "us-federal".into()
}

impl RulePack {
    pub fn generic() -> Self {
        serde_json::from_str(GENERIC_PACK_JSON).expect("generic rule pack")
    }

    pub fn nj_app_div() -> Self {
        serde_json::from_str(NJ_APP_DIV_JSON).expect("nj rule pack")
    }

    pub fn co_mandamus() -> Self {
        serde_json::from_str(CO_MANDAMUS_JSON).expect("co mandamus pack")
    }

    pub fn builtin(id: &str) -> Result<Self, String> {
        match id.trim() {
            "" | "generic" | "generic-us-appellate" => Ok(Self::generic()),
            "nj-app-div" | "nj" => Ok(Self::nj_app_div()),
            "co-mandamus" | "mandamus" | "co" => Ok(Self::co_mandamus()),
            other => Err(format!(
                "Unknown rule pack '{other}'. Use generic-us-appellate, nj-app-div, or co-mandamus."
            )),
        }
    }

    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&text).map_err(|e| format!("Rule pack is not valid JSON: {e}"))
    }

    pub fn find_deadline(&self, id_or_rule: &str) -> Result<&DeadlineRule, String> {
        let key = id_or_rule.trim().to_ascii_lowercase();
        self.deadlines
            .iter()
            .find(|d| {
                d.id.to_ascii_lowercase() == key
                    || d.rule.to_ascii_lowercase() == key
                    || d.rule.to_ascii_lowercase().replace(' ', "") == key.replace(' ', "")
            })
            .ok_or_else(|| {
                format!(
                    "No deadline named '{id_or_rule}' in pack {}. Known ids: {}.",
                    self.id,
                    self.deadlines
                        .iter()
                        .map(|d| d.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
    }

    pub fn list_builtin() -> Vec<Self> {
        vec![Self::generic(), Self::nj_app_div(), Self::co_mandamus()]
    }

    /// Validate a pack file. New states are data, not code.
    pub fn validate_file(path: &Path) -> Result<crate::case_profile::ValidationReport, String> {
        let pack = Self::load_from_path(path)?;
        Ok(pack.validate())
    }

    pub fn validate(&self) -> crate::case_profile::ValidationReport {
        use crate::case_profile::{ValidationIssue, ValidationReport};
        let mut issues = Vec::new();
        if self.id.trim().is_empty() {
            issues.push(ValidationIssue {
                field: "id".into(),
                message: "Pack needs an id such as nj-app-div.".into(),
            });
        }
        if self.name.trim().is_empty() {
            issues.push(ValidationIssue {
                field: "name".into(),
                message: "Pack needs a human-readable name.".into(),
            });
        }
        if !self.never_files || !self.never_serves {
            issues.push(ValidationIssue {
                field: "never_files".into(),
                message: "Packs must set never_files and never_serves to true.".into(),
            });
        }
        if self.deadlines.is_empty() {
            issues.push(ValidationIssue {
                field: "deadlines".into(),
                message: "Add at least one deadline object with id, rule, trigger, and days.".into(),
            });
        }
        for (i, d) in self.deadlines.iter().enumerate() {
            if d.id.trim().is_empty() || d.rule.trim().is_empty() {
                issues.push(ValidationIssue {
                    field: format!("deadlines.{i}"),
                    message: "Each deadline needs id and rule. See docs/RULE-PACKS.md.".into(),
                });
            }
        }
        ValidationReport {
            ok: issues.is_empty(),
            issues,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nj_pack_has_priority_rules() {
        let pack = RulePack::nj_app_div();
        assert_eq!(pack.id, "nj-app-div");
        assert!(pack.never_files && pack.never_serves);
        let recon = pack.find_deadline("reconsideration-appellate").unwrap();
        assert_eq!(recon.rule, "R. 2:11-6");
        assert_eq!(recon.days, 10);
        assert_eq!(recon.trigger, "entry");
        assert_eq!(recon.enlargeable_under.as_deref(), Some("R. 1:3-4(a)"));
        let trial = pack.find_deadline("R. 4:49-2").unwrap();
        assert_eq!(trial.days, 20);
        assert_eq!(trial.trigger, "service");
        assert!(pack.find_deadline("R. 2:8-1").is_ok());
        assert!(pack.find_deadline("R. 2:4-3(e)").is_ok());
    }

    #[test]
    fn mandamus_pack_notes_missing_upstream_repo() {
        let pack = RulePack::co_mandamus();
        assert!(pack.source_note.contains("not readable"));
        assert!(!pack.knowledge_base.is_empty());
    }

    #[test]
    fn bundled_packs_validate() {
        for pack in RulePack::list_builtin() {
            let report = pack.validate();
            assert!(report.ok, "{}: {:?}", pack.id, report.issues);
        }
    }
}
