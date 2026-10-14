//! "What do I need to file?" finder. Teaching comes from packs. Never invents rules.

use serde::{Deserialize, Serialize};

use crate::case_profile::ValidationIssue;
use crate::check::NEVER_FILE_NOTICE;
use crate::deadlines::DeadlineResult;
use crate::forms::LEGAL_INFO_NOTICE;
use crate::help_links::{HelpLink, HelpLinks};
use crate::rule_pack::{FormEntry, RulePack, Situation, SituationStep, TeachBlock};

pub const GUIDE_DISCLAIMER: &str = "Legal information, not legal advice. This finder only repeats what a pack already contains. It does not choose a strategy for you. Confirm every rule and form on the court's own site. OpenSeason never files or serves.";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuideQuestion {
    pub id: String,
    pub prompt: String,
    pub why: String,
    pub choices: Vec<GuideChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuideChoice {
    pub id: String,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuideQuestions {
    pub disclaimer: String,
    pub never_files: bool,
    pub never_serves: bool,
    pub legal_information_not_advice: bool,
    pub questions: Vec<GuideQuestion>,
    pub help_links: Vec<HelpLink>,
    pub confirm_before_open: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuideResult {
    pub disclaimer: String,
    pub never_files: bool,
    pub never_serves: bool,
    pub legal_information_not_advice: bool,
    pub state: String,
    pub court: String,
    pub situation: String,
    pub situation_label: String,
    pub pack_id: String,
    pub why: String,
    pub forms: Vec<FormEntry>,
    pub steps: Vec<SituationStep>,
    pub deadline: Option<DeadlineResult>,
    pub citation: String,
    pub citation_status: String,
    pub unverified: Vec<String>,
    pub stuck: bool,
    pub teach: TeachBlock,
    pub help_links: Vec<HelpLink>,
    pub confirm_before_open: String,
    pub notice: String,
}

pub fn questions() -> Result<GuideQuestions, String> {
    let links = HelpLinks::load()?;
    let packs = RulePack::list_builtin();
    let mut states = Vec::new();
    let mut courts = Vec::new();
    let mut situations = Vec::new();
    for pack in &packs {
        states.push(GuideChoice {
            id: pack.jurisdiction.clone(),
            label: format!("{} — {}", pack.jurisdiction, pack.name),
            detail: pack.source_note.clone(),
        });
        courts.push(GuideChoice {
            id: pack.id.clone(),
            label: pack.court.clone(),
            detail: pack.name.clone(),
        });
        for sit in &pack.situations {
            situations.push(GuideChoice {
                id: sit.id.clone(),
                label: sit.label.clone(),
                detail: sit.question.clone(),
            });
        }
    }
    Ok(GuideQuestions {
        disclaimer: GUIDE_DISCLAIMER.into(),
        never_files: true,
        never_serves: true,
        legal_information_not_advice: true,
        questions: vec![
            GuideQuestion {
                id: "state".into(),
                prompt: "Which state is the case in?".into(),
                why: "Forms and deadlines are state-specific. The finder will not guess a state.".into(),
                choices: states,
            },
            GuideQuestion {
                id: "court".into(),
                prompt: "Which court?".into(),
                why: "Copy the court name from a paper you already have if you can. Leave it blank rather than guessing.".into(),
                choices: courts,
            },
            GuideQuestion {
                id: "situation".into(),
                prompt: "What happened?".into(),
                why: "Answer in plain language. The pack maps that answer to forms it already lists.".into(),
                choices: situations,
            },
            GuideQuestion {
                id: "trigger_date".into(),
                prompt: "What date started the clock? (YYYY-MM-DD)".into(),
                why: "Use the date of entry or service from your papers. The finder only runs pack math when the situation names a deadline already in the pack.".into(),
                choices: vec![],
            },
        ],
        help_links: links.links,
        confirm_before_open: links.confirm,
    })
}

pub fn find(
    state: &str,
    court: Option<&str>,
    situation: &str,
    trigger_date: Option<&str>,
) -> Result<GuideResult, String> {
    let links = HelpLinks::load()?;
    let pack = pick_pack(state, court)?;
    let mut issues: Vec<String> = Vec::new();
    let sit = pack.find_situation(situation);
    if sit.is_none() {
        return Ok(stuck_result(
            state,
            court.unwrap_or(""),
            situation,
            &pack,
            &links,
            format!(
                "No situation named '{situation}' in pack {}. Known: {}.",
                pack.id,
                pack.situations
                    .iter()
                    .map(|s| s.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    }
    let sit: &Situation = sit.unwrap();
    let mut forms = Vec::new();
    for id in &sit.form_ids {
        if let Some(form) = pack.form(id) {
            if form.citation_status != "pack_source" {
                issues.push(format!(
                    "Form '{}' is marked {}. Do not treat it as a verified rule.",
                    form.id, form.citation_status
                ));
            }
            forms.push(form.clone());
        } else {
            issues.push(format!(
                "Pack {} lists form '{id}' for this situation but does not include that form. Needs verification."
            , pack.id));
        }
    }
    let mut deadline = None;
    if let Some(deadline_id) = &sit.deadline_id {
        if let Some(date) = trigger_date.filter(|d| !d.trim().is_empty()) {
            match pack.find_deadline(deadline_id) {
                Ok(rule) => match crate::deadlines::parse_iso_date(date) {
                    Ok(parsed) => {
                        deadline = Some(crate::deadlines::compute_deadline(&pack, rule, parsed));
                    }
                    Err(err) => issues.push(err),
                },
                Err(err) => issues.push(err),
            }
        }
    } else if trigger_date.filter(|d| !d.trim().is_empty()).is_some() {
        issues.push(
            "This situation has no deadline in the pack. The date you entered was not used. Needs verification."
                .into(),
        );
    }
    if sit.citation_status != "pack_source" {
        issues.push(format!(
            "Situation '{}' is marked {}. The finder did not invent a rule to fill the gap.",
            sit.id, sit.citation_status
        ));
    }
    for step in &sit.steps {
        if step.citation_status != "pack_source" {
            issues.push(format!(
                "Step '{}' is marked {}.",
                step.id, step.citation_status
            ));
        }
    }
    Ok(GuideResult {
        disclaimer: GUIDE_DISCLAIMER.into(),
        never_files: true,
        never_serves: true,
        legal_information_not_advice: true,
        state: pack.jurisdiction.clone(),
        court: if sit.court.is_empty() {
            pack.court.clone()
        } else {
            sit.court.clone()
        },
        situation: sit.id.clone(),
        situation_label: sit.label.clone(),
        pack_id: pack.id.clone(),
        why: sit.why.clone(),
        forms,
        steps: sit.steps.clone(),
        deadline,
        citation: sit.citation.clone(),
        citation_status: sit.citation_status.clone(),
        unverified: issues,
        stuck: false,
        teach: TeachBlock {
            what_it_is: sit.why.clone(),
            when_used: sit.question.clone(),
            common_mistakes: sit
                .steps
                .iter()
                .flat_map(|s| s.teach.common_mistakes.clone())
                .collect(),
            learn_more: pack.source_note.clone(),
        },
        help_links: links.links,
        confirm_before_open: links.confirm,
        notice: format!("{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}\n{GUIDE_DISCLAIMER}"),
    })
}

fn pick_pack(state: &str, court: Option<&str>) -> Result<RulePack, String> {
    let state_key = state.trim().to_ascii_lowercase();
    if let Ok(pack) = RulePack::resolve(state) {
        return Ok(pack);
    }
    if let Some(court) = court {
        if let Ok(pack) = RulePack::resolve(court) {
            return Ok(pack);
        }
    }
    for pack in RulePack::list_builtin() {
        if pack.jurisdiction.eq_ignore_ascii_case(state)
            || pack.id.to_ascii_lowercase().contains(&state_key)
        {
            return RulePack::resolve(&pack.id);
        }
        if let Some(court) = court {
            let c = court.to_ascii_lowercase();
            if pack.court.to_ascii_lowercase().contains(&c) || pack.id.to_ascii_lowercase().contains(&c)
            {
                return RulePack::resolve(&pack.id);
            }
        }
    }
    Err(format!(
        "No rule pack for state '{state}'. Install a pack or use nj, co, or generic."
    ))
}

fn stuck_result(
    state: &str,
    court: &str,
    situation: &str,
    pack: &RulePack,
    links: &HelpLinks,
    why: String,
) -> GuideResult {
    GuideResult {
        disclaimer: GUIDE_DISCLAIMER.into(),
        never_files: true,
        never_serves: true,
        legal_information_not_advice: true,
        state: state.into(),
        court: court.into(),
        situation: situation.into(),
        situation_label: String::new(),
        pack_id: pack.id.clone(),
        why,
        forms: vec![],
        steps: vec![],
        deadline: None,
        citation: String::new(),
        citation_status: "needs_verification".into(),
        unverified: vec![
            "The finder did not invent a form or rule for this answer. Try another situation, install a state pack, or open JustLegal help after the confirmation prompt.".into(),
        ],
        stuck: true,
        teach: TeachBlock {
            what_it_is: "No matching situation in the pack.".into(),
            when_used: "When the pack does not cover what happened.".into(),
            common_mistakes: vec!["Guessing a form the pack does not list.".into()],
            learn_more: "Get legal help on JustLegal after you confirm opening a browser.".into(),
        },
        help_links: links.links.clone(),
        confirm_before_open: links.confirm.clone(),
        notice: format!("{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}\n{GUIDE_DISCLAIMER}"),
    }
}

pub fn issues_from(result: &GuideResult) -> Vec<ValidationIssue> {
    result
        .unverified
        .iter()
        .map(|message| ValidationIssue {
            field: if result.stuck {
                "guide".into()
            } else {
                "citation".into()
            },
            message: message.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn questions_cover_state_court_event_deadline() {
        let q = questions().unwrap();
        let ids: Vec<_> = q.questions.iter().map(|x| x.id.as_str()).collect();
        assert_eq!(ids, ["state", "court", "situation", "trigger_date"]);
        assert!(q.legal_information_not_advice);
        assert!(!q.help_links.is_empty());
        assert!(q.confirm_before_open.contains("internet"));
    }

    #[test]
    fn nj_reconsideration_uses_pack_rule_and_deadline() {
        let result = find("NJ", None, "look again", Some("2026-01-09")).unwrap();
        assert!(!result.stuck);
        assert_eq!(result.citation, "R. 2:11-6");
        assert_eq!(result.citation_status, "pack_source");
        assert_eq!(result.deadline.as_ref().unwrap().due, "2026-01-20");
        assert!(result.forms.iter().any(|f| f.id == "nj-reconsideration-motion"));
        assert!(result.help_links.iter().any(|l| l.id == "find-legal-help"));
        assert!(!result.forms.is_empty());
        assert!(!result.steps[0].teach.what_it_is.is_empty());
    }

    #[test]
    fn appeal_does_not_invent_a_day_count() {
        let result = find("nj", Some("Appellate Division"), "appeal", Some("2026-01-09")).unwrap();
        assert!(result.deadline.is_none());
        assert!(result
            .unverified
            .iter()
            .any(|m| m.contains("Needs verification") || m.contains("no deadline")));
        assert!(result.forms.iter().any(|f| f.id == "nj-cn-10837"));
    }

    #[test]
    fn unknown_situation_is_stuck_and_offers_help_links() {
        let result = find("NJ", None, "spaceship ticket", None).unwrap();
        assert!(result.stuck);
        assert!(result.forms.is_empty());
        assert!(!result.help_links.is_empty());
        assert_eq!(result.citation_status, "needs_verification");
    }
}
