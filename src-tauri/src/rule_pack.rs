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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TeachBlock {
    #[serde(default)]
    pub what_it_is: String,
    #[serde(default)]
    pub when_used: String,
    #[serde(default)]
    pub common_mistakes: Vec<String>,
    #[serde(default)]
    pub learn_more: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FormEntry {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub form_number: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub court: String,
    #[serde(default)]
    pub situation: String,
    #[serde(default)]
    pub size_bytes: u64,
    #[serde(default)]
    pub download_url: String,
    #[serde(default)]
    pub official_url: String,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub path_in_pack: String,
    #[serde(default)]
    pub citation: String,
    #[serde(default)]
    pub citation_status: String,
    #[serde(default)]
    pub teach: TeachBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SituationStep {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub why: String,
    #[serde(default)]
    pub form_ids: Vec<String>,
    #[serde(default)]
    pub citation: String,
    #[serde(default)]
    pub citation_status: String,
    #[serde(default)]
    pub teach: TeachBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Situation {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub court: String,
    #[serde(default)]
    pub form_ids: Vec<String>,
    #[serde(default)]
    pub packet_id: Option<String>,
    #[serde(default)]
    pub deadline_id: Option<String>,
    #[serde(default)]
    pub why: String,
    #[serde(default)]
    pub citation: String,
    #[serde(default)]
    pub citation_status: String,
    #[serde(default)]
    pub steps: Vec<SituationStep>,
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
    pub rule_pack_id: Option<String>,
    #[serde(default)]
    pub deadlines: Vec<DeadlineRule>,
    #[serde(default)]
    pub knowledge_base: Vec<KnowledgeEntry>,
    #[serde(default)]
    pub forms: Vec<FormEntry>,
    #[serde(default)]
    pub situations: Vec<Situation>,
}

fn default_holiday_set() -> String {
    "us-federal".into()
}

pub fn normalize_key(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace('_', " ")
        .replace('-', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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

    /// Bundled pack plus any locally installed form overlay with the same rule_pack_id.
    pub fn resolve(id: &str) -> Result<Self, String> {
        let mut pack = Self::builtin(id)?;
        if let Ok(overlays) = crate::forms::installed_overlays_for(&pack.id) {
            for overlay in overlays {
                pack.merge_overlay(overlay);
            }
        }
        Ok(pack)
    }

    pub fn merge_overlay(&mut self, other: RulePack) {
        for form in other.forms {
            if let Some(existing) = self.forms.iter_mut().find(|f| f.id == form.id) {
                if existing.sha256.is_empty() {
                    existing.sha256 = form.sha256.clone();
                }
                if existing.size_bytes == 0 {
                    existing.size_bytes = form.size_bytes;
                }
                if existing.path_in_pack.is_empty() {
                    existing.path_in_pack = form.path_in_pack.clone();
                }
                if existing.download_url.is_empty() {
                    existing.download_url = form.download_url.clone();
                }
            } else {
                self.forms.push(form);
            }
        }
        for sit in other.situations {
            if !self.situations.iter().any(|s| s.id == sit.id) {
                self.situations.push(sit);
            }
        }
    }

    pub fn find_situation(&self, query: &str) -> Option<&Situation> {
        let key = normalize_key(query);
        if key.is_empty() {
            return None;
        }
        self.situations.iter().find(|s| {
            normalize_key(&s.id) == key
                || s.aliases.iter().any(|a| normalize_key(a) == key)
                || normalize_key(&s.label) == key
        })
        .or_else(|| {
            self.situations.iter().find(|s| {
                normalize_key(&s.label).contains(&key)
                    || normalize_key(&s.question).contains(&key)
                    || s.aliases.iter().any(|a| normalize_key(a).contains(&key))
            })
        })
    }

    pub fn form(&self, id: &str) -> Option<&FormEntry> {
        let key = normalize_key(id);
        self.forms.iter().find(|f| normalize_key(&f.id) == key)
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

    #[test]
    fn nj_pack_teaches_reconsideration_without_inventing_appeal_days() {
        let pack = RulePack::nj_app_div();
        let recon = pack.find_situation("look again").expect("reconsideration");
        assert_eq!(recon.citation_status, "pack_source");
        assert_eq!(recon.deadline_id.as_deref(), Some("reconsideration-appellate"));
        let appeal = pack.find_situation("appeal").expect("appeal");
        assert!(appeal.deadline_id.is_none(), "must not invent a notice-of-appeal day count");
        let form = pack.form("nj-cn-10837").expect("catalogued kit");
        assert_eq!(form.form_number, "CN 10837");
        assert!(!form.teach.what_it_is.is_empty());
        assert!(!form.teach.common_mistakes.is_empty());
    }
}
