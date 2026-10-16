//! Types, models, and statutory oath formulas for the Legal OS.

use serde::{Deserialize, Serialize};
use crate::case_profile::ValidationIssue;

pub const NJ_R_1_4_4B_OATH: &str = "I certify that the foregoing statements made by me are true. I am aware that if any of the foregoing statements made by me are willfully false, I am subject to punishment.";

pub const CO_CRS_13_27_106_OATH: &str = "I declare under penalty of perjury under the law of Colorado that the foregoing is true and correct.";

pub const FED_28_USC_1746_OATH: &str = "I declare under penalty of perjury under the laws of the United States of America that the foregoing is true and correct.";

pub const CO_RULE_121_CONFERRAL_TEMPLATE: &str = "I conferred with the opposing party on [date]. The opposing party [agrees / takes no position / opposes]. Confirm whether C.R.C.P. 121 § 1-15(8) or the Colorado Appellate Rules govern this paper before filing.";

pub const ENTER_ORDER_DATE_PROMPT: &str = "Enter the order date to compute this deadline";
pub const DEADLINE_UNVERIFIED_LABEL: &str = "unverified, check the order and rule";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VenueProfile {
    pub venue_code: String,
    pub display_name: String,
    pub sworn_statement_title: String,
    pub sworn_statement_citation: String,
    pub mandatory_oath_text: String,
    pub requires_conferral_certificate: bool,
    pub conferral_rule: String,
    pub appeal_days: i64,
    pub reconsideration_days: i64,
    pub reconsideration_trigger: String,
    pub tolling_mode: String, // "pause_remaining" (NJ) vs "reset_full" (CO/Federal)
    pub mail_service_adder_days: i64,
    pub packet_manifest: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VenueCompetenceReport {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub matter_tier: String,
    pub document_kind: String,
    pub venue: VenueProfile,
    pub issues: Vec<ValidationIssue>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProceduralBranch {
    pub id: String,
    pub title: String,
    pub rule_citation: String,
    pub trigger_event_title: String,
    pub trigger_date: String,
    pub trigger_kind: String, // "entry" | "service" | "delivery"
    pub base_days: i64,
    pub mail_offset_days: i64,
    pub clock_interaction: String, // "independent" | "pause_remaining" | "reset_full"
    pub tolled_by_event: Option<String>,
    pub days_elapsed_before_toll: Option<i64>,
    pub days_remaining_after_toll: Option<i64>,
    pub computed_deadline: Option<String>,
    pub days_until_due: Option<i64>,
    pub status: String, // "active" | "paused_tolled" | "expired" | "action_ready" | "needs_trigger_date"
    pub template_kind: String,
    pub escalation_target: Option<String>,
    pub explanation: String,
    /// How the due date was calculated. Empty when no user trigger date exists.
    #[serde(default)]
    pub math: String,
    /// Provenance of the trigger date (timeline event, judgment_date field, etc.).
    #[serde(default)]
    pub trigger_source: String,
    /// Displayed deadlines are always unverified until the user checks the order and rule.
    #[serde(default)]
    pub unverified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NextStepsReport {
    pub ok: bool,
    pub never_files: bool,
    pub never_serves: bool,
    pub notice: String,
    pub case_id: String,
    pub case_title: String,
    pub matter_tier: String,
    pub venue_code: String,
    pub detected_triggers: Vec<String>,
    pub branches: Vec<ProceduralBranch>,
}
