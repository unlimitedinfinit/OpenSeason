use crate::case_profile::types::IssueItem;
use crate::case_profile::CaseProfile;
use super::types::{IssueStressTestResult, StressTestVulnerability, VulnerabilitySeverity, VulnerabilityType};

/// Evaluates a single identified issue for factual and procedural vulnerabilities
/// using strictly deterministic administrative rules. Never practices law or gives advice.
pub fn stress_test_issue(profile: &CaseProfile, issue: &IssueItem) -> IssueStressTestResult {
    let mut vulnerabilities = Vec::new();
    let lower_title = issue.title.to_lowercase();
    let lower_cat = issue.category.to_lowercase();

    // 1. Evidentiary Record Grounding Check
    let has_citations = !issue.record_citations.is_empty();
    if !has_citations {
        vulnerabilities.push(StressTestVulnerability {
            vulnerability_type: VulnerabilityType::MissingRecordGrounding,
            severity: VulnerabilitySeverity::Critical,
            title: "Absence of Grounded Transcript or Docket Citation".into(),
            description: "This issue does not cite an admitted exhibit, filed docket entry, or hearing transcript page:line.".into(),
            opposing_argument_prediction: "Opposing counsel will move to strike this assertion as unsupported by the official appellate record.".into(),
            suggested_cure_or_inquiry: "Audit hearing transcripts or request missing court records to isolate verbatim page and line references before asserting.".into(),
            citation_anchor: None,
        });
    }

    // 2. Standard of Review Vulnerability
    if lower_cat.contains("discretion") || lower_title.contains("denied") || lower_title.contains("reconsideration") {
        vulnerabilities.push(StressTestVulnerability {
            vulnerability_type: VulnerabilityType::StandardOfReview,
            severity: VulnerabilitySeverity::Significant,
            title: "Discretionary Deference Standard (Abuse of Discretion)".into(),
            description: "Trial court rulings on procedural motions, discovery, and reconsideration are governed by high judicial deference.".into(),
            opposing_argument_prediction: "Appellee will argue the trial judge acted within their broad discretion and that appellant simply disagrees with the outcome.".into(),
            suggested_cure_or_inquiry: "Frame challenge around an erroneous legal standard (reviewed de novo) or complete lack of evidential support rather than subjective disagreement.".into(),
            citation_anchor: Some("In re Estate of Hope, 390 N.J. Super. 533 (App. Div. 2007)".into()),
        });
    }

    // 3. Issue Preservation Check
    let notes = issue.notes.to_lowercase();
    if !notes.contains("object") && !notes.contains("raised below") && !notes.contains("preserve") {
        vulnerabilities.push(StressTestVulnerability {
            vulnerability_type: VulnerabilityType::PreservationIssue,
            severity: VulnerabilitySeverity::Significant,
            title: "Preservation on the Record Not Confirmed".into(),
            description: "Appellate courts generally decline to consider claims not timely raised and preserved before the trial judge.".into(),
            opposing_argument_prediction: "Opposing party will assert waiver and argue review is barred or subject to the stringent Plain Error hurdle (R. 2:10-2).".into(),
            suggested_cure_or_inquiry: "Verify the exact docket filing or hearing transcript location where this specific objection was stated to the trial judge.".into(),
            citation_anchor: Some("State v. Robinson, 200 N.J. 1 (2009)".into()),
        });
    }

    // 4. Interlocutory vs. Final Judgment Check
    let is_appeal = profile.document_kind.to_lowercase().contains("appeal");
    let is_interlocutory = profile.procedural_stage.to_lowercase().contains("interlocutory")
        || lower_title.contains("discovery")
        || lower_title.contains("temporary");
    if is_appeal && is_interlocutory {
        vulnerabilities.push(StressTestVulnerability {
            vulnerability_type: VulnerabilityType::InterlocutoryFinality,
            severity: VulnerabilitySeverity::Critical,
            title: "Interlocutory Order Appealed Without Leave of Court".into(),
            description: "The order challenged may not dispose of all issues as to all parties, making an appeal as of right premature.".into(),
            opposing_argument_prediction: "The appellate clerk or appellee will file a motion to dismiss the appeal for lack of jurisdiction under the finality doctrine.".into(),
            suggested_cure_or_inquiry: "Consider filing a Motion for Leave to Appeal (e.g. NJ R. 2:5-6 / 28 U.S.C. § 1292(b)) or seeking trial court certification under R. 4:42-2.".into(),
            citation_anchor: Some("Grow Co. v. Chokshi, 403 N.J. Super. 443 (App. Div. 2008)".into()),
        });
    }

    // 5. Harmless Error Posture
    if vulnerabilities.is_empty() {
        vulnerabilities.push(StressTestVulnerability {
            vulnerability_type: VulnerabilityType::HarmlessError,
            severity: VulnerabilitySeverity::Advisory,
            title: "Substantial Prejudice vs. Harmless Error".into(),
            description: "Even if an error of law or procedure occurred, appellate review requires demonstrating the error produced an unjust result.".into(),
            opposing_argument_prediction: "Opposing party will argue that any procedural irregularity was harmless and did not affect the substantial rights of the parties.".into(),
            suggested_cure_or_inquiry: "Identify and document tangible prejudice, such as exclusion of dispositive evidence or denial of opportunity to respond.".into(),
            citation_anchor: None,
        });
    }

    let overall_risk = if vulnerabilities.iter().any(|v| v.severity == VulnerabilitySeverity::Critical) {
        "High Procedural Risk".to_string()
    } else if vulnerabilities.iter().any(|v| v.severity == VulnerabilitySeverity::Significant) {
        "Moderate Vulnerability".to_string()
    } else {
        "Low Evidentiary Friction".to_string()
    };

    let summary = format!(
        "Audited issue '{}' across 5 procedural risk axes. Identified {} candidate vulnerabilities for attorney inquiry.",
        issue.title,
        vulnerabilities.len()
    );

    IssueStressTestResult {
        issue_id: issue.id.clone(),
        issue_title: issue.title.clone(),
        asserted_by: issue.asserted_by.clone(),
        vulnerabilities,
        overall_risk_rating: overall_risk,
        procedural_posture_summary: summary,
    }
}

/// Stress tests all issues registered in the case profile
pub fn stress_test_all_issues(profile: &CaseProfile) -> Vec<IssueStressTestResult> {
    profile
        .issues
        .iter()
        .map(|i| stress_test_issue(profile, i))
        .collect()
}
