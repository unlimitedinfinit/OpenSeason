//! Case validation against required fields, placeholders, and court rules.

use regex::Regex;
use std::path::Path;

use super::profile::CaseProfile;
use super::storage::layout_missing;
use super::types::{ValidationIssue, ValidationReport};

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

pub fn placeholder_hits(text: &str) -> Vec<String> {
    let mut hits = Vec::new();
    let bracket = Regex::new(r"\[[A-Z0-9_][A-Z0-9_ \-/]{1,80}\]").unwrap();
    for cap in bracket.find_iter(text) {
        hits.push(cap.as_str().to_string());
    }
    let mustache = Regex::new(r"\{\{[^}]+\}\}").unwrap();
    for cap in mustache.find_iter(text) {
        hits.push(cap.as_str().to_string());
    }
    let chevron = Regex::new(r"<<[^>]+>>").unwrap();
    for cap in chevron.find_iter(text) {
        hits.push(cap.as_str().to_string());
    }
    for token in ["TODO", "TBD", "FIXME", "___"] {
        if text.to_ascii_uppercase().contains(token) {
            hits.push(token.to_string());
        }
    }
    hits.sort();
    hits.dedup();
    hits
}

pub fn validate_case(profile: &CaseProfile, case_dir: Option<&Path>) -> ValidationReport {
    let mut issues = Vec::new();
    let tier = profile.effective_tier();
    let kind = profile.document_kind.trim().to_ascii_lowercase();

    if is_blank(&profile.title) {
        issues.push(ValidationIssue {
            field: "title".into(),
            message: "Case or instrument title is required.".into(),
        });
    }
    if tier == "litigation" {
        if is_blank(&profile.court.name) {
            issues.push(ValidationIssue {
                field: "court.name".into(),
                message: "Court name is required so the caption can be filled.".into(),
            });
        }
        if is_blank(&profile.docket_number) && !profile.is_pre_filing() {
            issues.push(ValidationIssue {
                field: "docket_number".into(),
                message: "Docket number is required (or set procedural_stage to pre_filing for initial complaints).".into(),
            });
        }
    }
    if profile.caption.appellants().is_empty() {
        issues.push(ValidationIssue {
            field: "caption.appellants".into(),
            message: "Add at least one primary party (appellant, plaintiff, testator, landlord, or claimant).".into(),
        });
    }
    if kind != "will" && kind != "power_of_attorney" && profile.caption.appellees().is_empty() {
        issues.push(ValidationIssue {
            field: "caption.appellees".into(),
            message: "Add at least one counterparty (appellee, defendant, tenant, contractor, or respondent).".into(),
        });
    }
    if is_blank(&profile.filer.name) {
        issues.push(ValidationIssue {
            field: "filer.name".into(),
            message: "Filer / signer name is required for the signature block.".into(),
        });
    }
    if is_blank(&profile.filer.signature_name) {
        issues.push(ValidationIssue {
            field: "filer.signature_name".into(),
            message: "Signature name is required.".into(),
        });
    }

    if let Some(dir) = case_dir {
        for missing in layout_missing(dir) {
            issues.push(ValidationIssue {
                field: format!("folder.{}", missing),
                message: format!("Expected folder '{}' is missing.", missing),
            });
        }
    }

    if tier == "litigation" && (kind.is_empty() || kind == "notice_of_appeal" || kind == "appeal") {
        if is_blank(&profile.notice_of_appeal.judgment_date) {
            issues.push(ValidationIssue {
                field: "notice_of_appeal.judgment_date".into(),
                message: "Judgment or order date is required for a Notice of Appeal.".into(),
            });
        }
        if is_blank(&profile.notice_of_appeal.judgment_description) {
            issues.push(ValidationIssue {
                field: "notice_of_appeal.judgment_description".into(),
                message: "Describe the order or judgment being appealed, in the user's own words."
                    .into(),
            });
        }
        if is_blank(&profile.notice_of_appeal.user_text) {
            issues.push(ValidationIssue {
                field: "notice_of_appeal.user_text".into(),
                message: "Paste the user's own Notice of Appeal text. OpenSeason will not invent it.".into(),
            });
        }
        let combined = format!(
            "{} {} {} {}",
            profile.notice_of_appeal.user_text,
            profile.notice_of_appeal.judgment_description,
            profile.court.name,
            profile.title
        );
        for hit in placeholder_hits(&combined) {
            issues.push(ValidationIssue {
                field: "placeholders".into(),
                message: format!("Unfilled placeholder remains: {}", hit),
            });
        }
    } else if tier != "litigation" {
        if is_blank(&profile.motion.user_text) && is_blank(&profile.complaint.user_text) && is_blank(&profile.court_markdown) {
            issues.push(ValidationIssue {
                field: "motion.user_text".into(),
                message: "Instrument or notice body text is required. OpenSeason will not invent terms.".into(),
            });
        }
        let combined = format!("{} {} {}", profile.motion.user_text, profile.complaint.user_text, profile.title);
        push_placeholder_issues(&combined, &mut issues);
    }

    ValidationReport {
        ok: issues.is_empty(),
        issues,
    }
}

pub fn push_core_profile_issues(profile: &CaseProfile, issues: &mut Vec<ValidationIssue>) {
    let tier = profile.effective_tier();
    let kind = profile.document_kind.trim().to_ascii_lowercase();
    if is_blank(&profile.title) {
        issues.push(ValidationIssue {
            field: "title".into(),
            message: "Case title is required.".into(),
        });
    }
    if tier == "litigation" {
        if is_blank(&profile.court.name) {
            issues.push(ValidationIssue {
                field: "court.name".into(),
                message: "Court name is required so the caption can be filled.".into(),
            });
        }
        if is_blank(&profile.docket_number) && !profile.is_pre_filing() {
            issues.push(ValidationIssue {
                field: "docket_number".into(),
                message: "Docket number is required.".into(),
            });
        }
    }
    if profile.caption.appellants().is_empty() {
        issues.push(ValidationIssue {
            field: "caption.parties".into(),
            message: "Add at least one plaintiff, appellant, or primary party.".into(),
        });
    }
    if kind != "will" && kind != "power_of_attorney" && profile.caption.appellees().is_empty() {
        issues.push(ValidationIssue {
            field: "caption.parties".into(),
            message: "Add at least one defendant, appellee, or counterparty.".into(),
        });
    }
    if is_blank(&profile.filer.name) {
        issues.push(ValidationIssue {
            field: "filer.name".into(),
            message: "Filer name is required for the signature block.".into(),
        });
    }
    if is_blank(&profile.filer.signature_name) {
        issues.push(ValidationIssue {
            field: "filer.signature_name".into(),
            message: "Signature name is required.".into(),
        });
    }
}

pub fn push_placeholder_issues(text: &str, issues: &mut Vec<ValidationIssue>) {
    for hit in placeholder_hits(text) {
        issues.push(ValidationIssue {
            field: "placeholders".into(),
            message: format!("Unfilled placeholder remains: {}", hit),
        });
    }
}

/// Validate the shared caption plus one pleading body (notice, complaint, or motion).
pub fn validate_pleading(
    profile: &CaseProfile,
    case_dir: Option<&Path>,
    kind: &str,
) -> ValidationReport {
    let mut issues = Vec::new();
    push_core_profile_issues(profile, &mut issues);
    if let Some(dir) = case_dir {
        for missing in layout_missing(dir) {
            issues.push(ValidationIssue {
                field: format!("folder.{}", missing),
                message: format!("Expected folder '{}' is missing.", missing),
            });
        }
    }

    let kind = kind.trim().to_ascii_lowercase();
    match kind.as_str() {
        "complaint" => {
            if is_blank(&profile.complaint.user_text) {
                issues.push(ValidationIssue {
                    field: "complaint.user_text".into(),
                    message: "Paste the user's own complaint text. This builder is basic and will not invent facts.".into(),
                });
            }
            push_placeholder_issues(
                &format!(
                    "{} {} {}",
                    profile.complaint.user_text, profile.court.name, profile.title
                ),
                &mut issues,
            );
        }
        "appellate_brief" | "brief" => {
            if is_blank(&profile.brief.user_text) {
                issues.push(ValidationIssue {
                    field: "brief.user_text".into(),
                    message: "Paste the user's own brief text. The scaffold will not invent argument.".into(),
                });
            }
            push_placeholder_issues(&profile.brief.user_text, &mut issues);
        }
        "appellate_motion" => {
            if is_blank(&profile.appellate_motion.user_text) {
                issues.push(ValidationIssue {
                    field: "appellate_motion.user_text".into(),
                    message: "Paste the user's own appellate-motion text.".into(),
                });
            }
            push_placeholder_issues(&profile.appellate_motion.user_text, &mut issues);
        }
        "mandamus" => {
            if is_blank(&profile.mandamus.user_text) {
                issues.push(ValidationIssue {
                    field: "mandamus.user_text".into(),
                    message: "Paste the user's own mandamus petition text. The Colorado pack will not invent the showing.".into(),
                });
            }
            push_placeholder_issues(&profile.mandamus.user_text, &mut issues);
        }
        "court_md" | "markdown" => {
            if is_blank(&profile.court_markdown) {
                issues.push(ValidationIssue {
                    field: "court_markdown".into(),
                    message: "Import a Markdown draft first.".into(),
                });
            }
        }
        "motion" | "reconsideration" | "enlargement" | "stay" | "amend_findings" | "will"
        | "lease" | "contractor" | "power_of_attorney" | "demand_letter" | "cure_notice"
        | "security_deposit_demand" | "records_request" | "agency_appeal" => {
            if is_blank(&profile.motion.user_text) {
                issues.push(ValidationIssue {
                    field: "motion.user_text".into(),
                    message: "Paste the user's own draft text. OpenSeason will not invent the terms or request.".into(),
                });
            }
            push_placeholder_issues(
                &format!(
                    "{} {} {}",
                    profile.motion.user_text, profile.court.name, profile.title
                ),
                &mut issues,
            );
        }
        _ => {
            if is_blank(&profile.notice_of_appeal.judgment_date) {
                issues.push(ValidationIssue {
                    field: "notice_of_appeal.judgment_date".into(),
                    message: "Judgment or order date is required for a Notice of Appeal.".into(),
                });
            }
            if is_blank(&profile.notice_of_appeal.judgment_description) {
                issues.push(ValidationIssue {
                    field: "notice_of_appeal.judgment_description".into(),
                    message: "Describe the order or judgment being appealed, in the user's own words."
                        .into(),
                });
            }
            if is_blank(&profile.notice_of_appeal.user_text) {
                issues.push(ValidationIssue {
                    field: "notice_of_appeal.user_text".into(),
                    message: "Paste the user's own Notice of Appeal text. OpenSeason will not invent it.".into(),
                });
            }
            push_placeholder_issues(
                &format!(
                    "{} {} {} {}",
                    profile.notice_of_appeal.user_text,
                    profile.notice_of_appeal.judgment_description,
                    profile.court.name,
                    profile.title
                ),
                &mut issues,
            );
        }
    }

    ValidationReport {
        ok: issues.is_empty(),
        issues,
    }
}
