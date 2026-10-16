//! Offline court order and notice decoding engine.
//!
//! Classifies court orders vs. boilerplate docketing notices, detects signers
//! ("BY THE COURT" screening staff vs named judge), extracts operative clauses,
//! and determines candidate response windows without inventing facts.

use std::path::Path;
use chrono::NaiveDate;
use regex::Regex;

use super::types::{DecodedOrder, OrderSigner};
use crate::pdf_tools::extract_pdf_text;

/// Decodes plain text from an order or notice.
pub fn decode_order_text(text: &str, file_name: Option<&str>) -> DecodedOrder {
    let lower = text.to_ascii_lowercase();

    // 1. Identify Court
    let court_name = if lower.contains("colorado court of appeals") {
        Some("Colorado Court of Appeals".to_string())
    } else if lower.contains("supreme court of colorado") {
        Some("Colorado Supreme Court".to_string())
    } else if lower.contains("appellate division") && (lower.contains("new jersey") || lower.contains("superior court of new jersey")) {
        Some("New Jersey Superior Court, Appellate Division".to_string())
    } else if lower.contains("united states court of appeals") {
        Some("United States Court of Appeals".to_string())
    } else if lower.contains("district court") && lower.contains("colorado") {
        Some("Colorado District Court".to_string())
    } else {
        None
    };

    // 2. Identify Docket / Case Number
    let docket_re = Regex::new(r"(?i)(?:case\s*no\.?|docket\s*no\.?|no\.?)\s*:?\s*([0-9]{2,4}\s*[A-Z]{1,3}\s*[0-9]{3,6}|A-[0-9]{4,6}-[0-9]{2}|[0-9]{2}-[0-9]{4,5})").unwrap();
    let docket_number = docket_re.captures(text).map(|c| c[1].trim().replace(" ", ""));

    // 3. Classify Signer
    let signer = if lower.contains("by the court") {
        OrderSigner::ByTheCourt {
            division: "Screening / Motions Division".into(),
        }
    } else if let Some(judge_cap) = Regex::new(r"(?i)(?:honorable|judge|justice)\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+){1,2})").unwrap().captures(text) {
        OrderSigner::NamedJudge {
            name: judge_cap[1].to_string(),
            title: Some("Judge".into()),
        }
    } else if lower.contains("clerk of the court") || lower.contains("clerk of court") || lower.contains("deputy clerk") {
        OrderSigner::Clerk { name: None }
    } else {
        OrderSigner::Unknown
    };

    // 4. Distinguish Boilerplate Notice vs Operative Order
    let has_show_cause = lower.contains("show cause");
    let has_ordered = lower.contains("it is ordered")
        || lower.contains("hereby ordered")
        || lower.contains("ordered that")
        || lower.contains("so ordered")
        || lower.contains("court orders")
        || lower.contains("ordered:");
    let has_directs = lower.contains("directs appellant") || lower.contains("directs plaintiff") || lower.contains("directs defendant");
    let has_dismissed = lower.contains("is dismissed") || lower.contains("appeal is dismissed");

    let is_boilerplate_notice = if lower.contains("notice of docketing")
        || (lower.contains("r. 2:11-1(b)") && lower.contains("oral argument"))
        || lower.contains("docketing notice")
        || lower.contains("notice to the bar")
    {
        !has_show_cause && !lower.contains("it is ordered") && !lower.contains("hereby ordered")
    } else if has_show_cause || has_ordered || has_directs || has_dismissed {
        false
    } else {
        false
    };

    // 5. Title
    let title = if has_show_cause {
        "Order to Show Cause".to_string()
    } else if is_boilerplate_notice {
        "Notice of Docketing / Administrative Notice".to_string()
    } else if has_ordered {
        "Court Order".to_string()
    } else if let Some(fname) = file_name {
        fname.replace(".pdf", "").replace(".docx", "").replace(['-', '_'], " ")
    } else {
        "Court Notice / Order".to_string()
    };

    // 6. Extract Operative Clauses
    let mut operative_clauses = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        let l_trim = trimmed.to_ascii_lowercase();
        if l_trim.contains("ordered that")
            || l_trim.contains("it is ordered")
            || l_trim.contains("show cause")
            || l_trim.contains("directs appellant")
            || l_trim.contains("shall file")
            || l_trim.contains("within ") && (l_trim.contains("days") || l_trim.contains("response"))
        {
            if !operative_clauses.contains(&trimmed.to_string()) && trimmed.len() > 10 {
                operative_clauses.push(trimmed.to_string());
            }
        }
    }

    // 7. Extract Candidate Response Window (Days)
    let window_re = Regex::new(r"(?i)within\s+(\d+)\s+days").unwrap();
    let candidate_response_window_days = window_re.captures(text).and_then(|c| c[1].parse::<u32>().ok());

    // 8. Extract Governing Rules Cited
    let mut governing_rules_cited = Vec::new();
    let rules_patterns = [
        r"C\.A\.R\.\s*[0-9]+(?:\([a-z0-9]+\))?",
        r"C\.R\.C\.P\.\s*[0-9]+(?:\([a-z0-9]+\))?",
        r"R\.\s*[0-9]+:[0-9]+(?:-[0-9]+)?",
        r"Fed\.\s*R\.\s*App\.\s*P\.\s*[0-9]+",
        r"28\s*U\.S\.C\.\s*§?\s*[0-9]+",
    ];
    for pat in rules_patterns {
        if let Ok(re) = Regex::new(pat) {
            for cap in re.captures_iter(text) {
                let rule = cap[0].trim().to_string();
                if !governing_rules_cited.contains(&rule) {
                    governing_rules_cited.push(rule);
                }
            }
        }
    }

    // 9. Extract Order Date
    let date_re = Regex::new(r"(?i)(?:dated|date|entered|filed):?\s*([A-Za-z]+\s+\d{1,2},\s*\d{4}|\d{4}-\d{2}-\d{2})").unwrap();
    let order_date = date_re.captures(text).map(|c| c[1].to_string());

    // 10. Calculate Candidate Deadline if Order Date & Window found
    let candidate_deadline = if let (Some(ref od), Some(window)) = (&order_date, candidate_response_window_days) {
        parse_loose_date(od).map(|d| {
            let deadline = d + chrono::Duration::days(window as i64);
            deadline.format("%Y-%m-%d").to_string()
        })
    } else {
        None
    };

    // 11. Uncertainty Notes & Status
    let mut uncertainty_notes = Vec::new();
    if signer.is_screening_division() {
        uncertainty_notes.push("Order signed 'BY THE COURT': Originated from staff attorneys/screening division, not a single named judge.".to_string());
    }
    if is_boilerplate_notice {
        uncertainty_notes.push("Standard boilerplate docketing notice: do not confuse oral argument reference with a scheduling order.".to_string());
    }
    if candidate_response_window_days.is_some() {
        uncertainty_notes.push("Candidate deadline unverified: Confirm whether response window counts calendar or court business days, and verify holiday roll rules.".to_string());
    }

    let status = if candidate_response_window_days.is_some() || is_boilerplate_notice {
        "needs_verification".to_string()
    } else {
        "verified".to_string()
    };

    // 12. Suggested Actions
    let mut suggested_actions = Vec::new();
    if has_show_cause {
        suggested_actions.push("Review governing rule and check tolling exceptions".to_string());
        suggested_actions.push("Check the existing record for postmark vs. clerk receipt dates".to_string());
        suggested_actions.push("Explore show-cause response options".to_string());
        suggested_actions.push("Prepare response to show cause".to_string());
    } else if is_boilerplate_notice {
        suggested_actions.push("Confirm active counsel of record on personnel roster".to_string());
        suggested_actions.push("Verify transcript delivery date to establish merits brief clock".to_string());
    } else {
        suggested_actions.push("Review operative clauses".to_string());
        suggested_actions.push("Add dates to procedural timeline".to_string());
    }

    DecodedOrder {
        title,
        court_name,
        docket_number,
        order_date,
        signer,
        is_boilerplate_notice,
        operative_clauses,
        candidate_response_window_days,
        candidate_deadline,
        governing_rules_cited,
        status,
        uncertainty_notes,
        source_page: 1,
        suggested_actions,
    }
}

/// Decodes a court order or notice from a PDF file.
pub fn decode_order_pdf(pdf_path: &Path) -> Result<DecodedOrder, String> {
    let extraction = extract_pdf_text(pdf_path)?;
    let fname = pdf_path.file_name().and_then(|s| s.to_str()).unwrap_or("order.pdf");
    let mut decoded = decode_order_text(&extraction.full_text, Some(fname));

    // Try to isolate the specific page where operative text was found
    for page in extraction.pages {
        let p_lower = page.text.to_ascii_lowercase();
        if p_lower.contains("ordered") || p_lower.contains("show cause") || p_lower.contains("within") {
            decoded.source_page = page.page_number as u32;
            break;
        }
    }

    Ok(decoded)
}

fn parse_loose_date(s: &str) -> Option<NaiveDate> {
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(d);
    }
    // Try Month DD, YYYY
    for fmt in &["%B %d, %Y", "%b %d, %Y", "%m/%d/%Y"] {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d);
        }
    }
    None
}
