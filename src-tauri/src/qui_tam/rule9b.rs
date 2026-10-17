//! 31 U.S.C. § 3729 & FRCP Rule 9(b) 6-Element Viability Audit Engine.

use chrono::Utc;
use std::path::Path;

use super::types::*;
use crate::case_profile::CaseProfile;

/// Evaluates a case workspace against the 6 statutory elements of the False Claims Act and Rule 9(b).
pub fn audit_case_rule9b(case_dir: &Path, profile: &CaseProfile) -> Result<Rule9bAudit, String> {
    let now = Utc::now().to_rfc3339();
    let evidence_dir = case_dir.join("evidence");
    let provenance_file = case_dir.join("evidence").join("provenance.json");

    let mut elements = Vec::new();
    let mut missing_requirements = Vec::new();
    let mut next_steps = Vec::new();
    let mut total_score = 0u32;

    // ---------------------------------------------------------
    // Element 1: Entity Identification (NPI, UEI, Corporate Officer, Address) [15 pts]
    // ---------------------------------------------------------
    let has_entity_id = !profile.title.is_empty() && (!profile.docket_number.is_empty() || profile.title.to_lowercase().contains("npi") || !profile.caption.defendants.is_empty());
    let mut elem1_missing = Vec::new();
    if profile.caption.defendants.is_empty() {
        elem1_missing.push("Defendant/Target entity official legal name and officers not listed in caption.".into());
    }
    let elem1_satisfied = has_entity_id && !profile.caption.defendants.is_empty();
    if elem1_satisfied {
        total_score += 15;
    } else {
        missing_requirements.push("Rule 9(b) Entity Identity: Provide target's legal name, 10-digit NPI or UEI, and registered address.".into());
        next_steps.push("Run 'openseason qui-tam verify-entity --name \"<target>\"' to pull official NPI and corporate officers.".into());
    }

    elements.push(Rule9bElement {
        id: "rule9b-1-entity-id".into(),
        title: "Target Entity Identification (Who)".into(),
        statutory_reference: "FRCP Rule 9(b); 31 U.S.C. § 3729".into(),
        satisfied: elem1_satisfied,
        details: if elem1_satisfied {
            format!("Target entity '{}' identified with defendant roster.", profile.title)
        } else {
            "Incomplete target entity identifiers.".into()
        },
        missing_evidence: elem1_missing,
    });

    // ---------------------------------------------------------
    // Element 2: Specific Claim / Drawdown Mechanism (What & How) [25 pts]
    // ---------------------------------------------------------
    // Rule 9(b) requires alleging specific claims or invoices, or a concrete billing scheme
    let mut has_claims_proof = false;
    let mut elem2_missing = Vec::new();
    if evidence_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&evidence_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("claim") || name.contains("invoice") || name.contains("billing") || name.contains("drawdown") || name.contains("remittance") {
                    has_claims_proof = true;
                    break;
                }
            }
        }
    }

    // Also check complaint sections or draft text for specific billing mentions
    let draft_text = profile.complaint.user_text.to_lowercase();
    if draft_text.contains("medicaid claim") || draft_text.contains("cms-1500") || draft_text.contains("asap.gov") || draft_text.contains("drawdown") {
        has_claims_proof = true;
    }

    if has_claims_proof {
        total_score += 25;
    } else {
        elem2_missing.push("No specific billing invoice, CMS-1500 claim form, or federal drawdown record identified in evidence/ or draft.".into());
        missing_requirements.push("Rule 9(b) Specific Claims: Must identify at least one representative false claim, invoice number, or drawdown date.".into());
        next_steps.push("Request monthly claims or payout records via state public records ('openseason qui-tam records-pack').".into());
    }

    elements.push(Rule9bElement {
        id: "rule9b-2-specific-claims".into(),
        title: "Specific False Claims or Drawdowns (What/How)".into(),
        statutory_reference: "31 U.S.C. § 3729(a)(1)(A); Clausen/Foglia pleading standard".into(),
        satisfied: has_claims_proof,
        details: if has_claims_proof {
            "Representative false claims or federal drawdown mechanism documented.".into()
        } else {
            "Lacks specific claims or representative billing dates required under Rule 9(b).".into()
        },
        missing_evidence: elem2_missing,
    });

    // ---------------------------------------------------------
    // Element 3: Falsity Type & Physical Impossibility Proof [20 pts]
    // ---------------------------------------------------------
    let mut has_falsity_proof = false;
    let mut elem3_missing = Vec::new();

    // Check if provenance.json or observation photos exist
    if provenance_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&provenance_file) {
            if content.contains("\"sha256\"") {
                has_falsity_proof = true;
            }
        }
    }

    if draft_text.contains("ghost") || draft_text.contains("closed") || draft_text.contains("unprovided") || draft_text.contains("not rendered") || draft_text.contains("upcoding") {
        has_falsity_proof = true;
    }

    if has_falsity_proof {
        total_score += 20;
    } else {
        elem3_missing.push("No field observation log, physical inspection photos, or concrete falsity evidence found.".into());
        missing_requirements.push("Rule 9(b) Falsity Proof: Detail why the claim was false (e.g. facility closed during stated operating hours).".into());
        next_steps.push("Ingest multi-day observation photos or logs using 'openseason qui-tam ingest-observation'.".into());
    }

    elements.push(Rule9bElement {
        id: "rule9b-3-falsity-proof".into(),
        title: "Falsity Type & Evidence (Why False)".into(),
        statutory_reference: "31 U.S.C. § 3729(a)(1)(B)".into(),
        satisfied: has_falsity_proof,
        details: if has_falsity_proof {
            "Falsity theory and supporting observation evidence documented.".into()
        } else {
            "Falsity remains unsupported by verified factual observations.".into()
        },
        missing_evidence: elem3_missing,
    });

    // ---------------------------------------------------------
    // Element 4: Scienter / Knowledge (Perjury Certification or Willful Disregard) [15 pts]
    // ---------------------------------------------------------
    let mut has_scienter = false;
    let mut elem4_missing = Vec::new();

    if draft_text.contains("knowingly") || draft_text.contains("perjury") || draft_text.contains("attestation") || draft_text.contains("certification") || draft_text.contains("reckless") {
        has_scienter = true;
    }

    if has_scienter {
        total_score += 15;
    } else {
        elem4_missing.push("Proof of knowledge, deliberate ignorance, or signed enrollment attestation not articulated.".into());
        missing_requirements.push("Rule 9(b) Scienter: Show defendant acted with actual knowledge, deliberate ignorance, or reckless disregard.".into());
        next_steps.push("Cite the target's signed enrollment agreement or annual certification of compliance.".into());
    }

    elements.push(Rule9bElement {
        id: "rule9b-4-scienter".into(),
        title: "Scienter & Knowledge Standard (Mental State)".into(),
        statutory_reference: "31 U.S.C. § 3729(b)(1)".into(),
        satisfied: has_scienter,
        details: if has_scienter {
            "Knowledge or reckless disregard element established via certifications or conduct.".into()
        } else {
            "Scienter element needs explicit statutory grounding.".into()
        },
        missing_evidence: elem4_missing,
    });

    // ---------------------------------------------------------
    // Element 5: Materiality & Quantitative Loss Calculation [15 pts]
    // ---------------------------------------------------------
    let mut has_loss = false;
    let mut elem5_missing = Vec::new();

    // Check timeline or draft for dollar signs or numbers
    if draft_text.contains('$') || draft_text.contains("dollar") || draft_text.contains("loss") || draft_text.contains("damages") {
        has_loss = true;
    }

    if has_loss {
        total_score += 15;
    } else {
        elem5_missing.push("No estimated federal financial loss or calculation formula provided.".into());
        missing_requirements.push("Rule 9(b) Materiality: Estimate total federal loss and explain why the falsehood was material to payment.".into());
        next_steps.push("Cross-reference USAspending obligations or monthly Medicaid reimbursements to compute estimated loss.".into());
    }

    elements.push(Rule9bElement {
        id: "rule9b-5-materiality-loss".into(),
        title: "Materiality & Federal Loss Estimate".into(),
        statutory_reference: "31 U.S.C. § 3729(a)(1)(G); Escobar materiality rule".into(),
        satisfied: has_loss,
        details: if has_loss {
            "Materiality and quantitative federal loss calculation articulated.".into()
        } else {
            "Estimated federal loss needs explicit mathematical estimation.".into()
        },
        missing_evidence: elem5_missing,
    });

    // ---------------------------------------------------------
    // Element 6: Original Source & Public Disclosure Check (§ 3730(e)(4)) [10 pts]
    // ---------------------------------------------------------
    let mut has_original_source = false;
    let mut elem6_missing = Vec::new();

    // If filer has entered observations, photos, or personal narrative, they qualify as an original source
    if provenance_file.exists() || !profile.timeline.is_empty() {
        has_original_source = true;
    }

    if has_original_source {
        total_score += 10;
    } else {
        elem6_missing.push("Lacks confirmation that relator has direct, independent firsthand knowledge of the fraud.".into());
        missing_requirements.push("Public Disclosure Bar: Confirm allegations are not derived solely from news, social media, or public reports.".into());
        next_steps.push("Complete the Relator Original Source Attestation in 'openseason qui-tam export-disclosure'.".into());
    }

    elements.push(Rule9bElement {
        id: "rule9b-6-original-source".into(),
        title: "Original Source & Public Disclosure Clearance".into(),
        statutory_reference: "31 U.S.C. § 3730(e)(4)(A)-(B)".into(),
        satisfied: has_original_source,
        details: if has_original_source {
            "Independent firsthand observations documented; public disclosure bar risk minimized.".into()
        } else {
            "Risk of Public Disclosure Bar if derived from third-party social media or news.".into()
        },
        missing_evidence: elem6_missing,
    });

    let viability_tier = match total_score {
        80..=100 => "High Viability (Attorney & DOJ Review Ready)".into(),
        50..=79 => "Developing (Claims Proof & Specific Drawdown Evidence Needed)".into(),
        _ => "Preliminary / Speculative (Missing Core Rule 9(b) Elements)".into(),
    };

    let public_disclosure_risk = if has_original_source {
        "LOW RISK: Grounded in independent firsthand observations and cryptographic evidence.".into()
    } else {
        "ELEVATED RISK: Ensure facts were not published on social media, news, or blogs prior to filing under seal.".into()
    };

    Ok(Rule9bAudit {
        case_id: profile.id.clone(),
        score: total_score,
        viability_tier,
        elements,
        missing_requirements,
        next_investigative_steps: next_steps,
        public_disclosure_risk,
        evaluated_at: now,
    })
}
