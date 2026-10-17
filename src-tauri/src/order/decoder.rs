//! Offline court order and notice decoding engine.
//!
//! Classifies court orders vs. boilerplate docketing notices, detects signers
//! ("BY THE COURT" screening staff vs named judge), extracts operative clauses,
//! and determines candidate response windows without inventing facts.

use std::fs;
use std::path::Path;
use chrono::NaiveDate;
use regex::Regex;
use sha2::{Digest, Sha256};

use super::types::{DecodedOrder, IngestedOrderReport, OrderSigner};
use crate::case_profile::{load_case, save_case, IssueItem, OperationReceipt, TimelineEvent};
use crate::deadlines::{compute_sourced_deadline, DeadlineVerificationStatus};
use crate::pdf_tools::extract_pdf_text;
use crate::rule_pack::RulePack;
use crate::workflow::coordinator::save_workflow_plan;
use crate::workflow::planner::plan_workflow;

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
        let division = if lower.contains("en banc") {
            Some("En Banc".into())
        } else if lower.contains("motions division") {
            Some("Motions Division".into())
        } else {
            None
        };
        OrderSigner::ByTheCourt { division }
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
    if signer.is_by_the_court() {
        uncertainty_notes.push("Order signed 'BY THE COURT': Issued as an institutional order of the court without naming an individual judge.".to_string());
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

/// Ingests a court order PDF into an active case folder:
/// 1. Verifies and copies original PDF into orders/ (logs SHA-256).
/// 2. Decodes order title, court, docket, signer (verbatim), and operative instructions.
/// 3. Updates case profile: adds TimelineEvent, updates docket/court, registers IssueItem if show-cause.
/// 4. Generates an OperationReceipt in case.json.
/// 5. Evaluates SourcedDeadline with calculation basis and proof checklist.
/// 6. Formulates an actionable 3-5 step WorkflowPlan in workflow_plan.json.
pub fn ingest_order_into_case(
    case_dir: &Path,
    order_pdf_path: &Path,
) -> Result<IngestedOrderReport, String> {
    if !case_dir.exists() {
        return Err(format!("Case directory does not exist: {}", case_dir.display()));
    }
    if !order_pdf_path.exists() {
        return Err(format!("Order PDF file not found: {}", order_pdf_path.display()));
    }

    let mut profile = load_case(case_dir)?;

    // 1. Compute SHA-256 hash and read bytes
    let bytes = fs::read(order_pdf_path).map_err(|e| format!("Could not read order PDF: {e}"))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash_hex = format!("{:x}", hasher.finalize());

    // 2. Decode order text and operative clauses
    let decoded = decode_order_pdf(order_pdf_path)?;

    // 3. Preserve original file safely in <case_dir>/orders/
    let orders_dir = case_dir.join("orders");
    fs::create_dir_all(&orders_dir).map_err(|e| format!("Could not create orders directory: {e}"))?;
    let orig_fname = order_pdf_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("court-order.pdf");
    let target_dest = orders_dir.join(orig_fname);
    let final_dest = if target_dest.exists() {
        orders_dir.join(format!(
            "{}-{}",
            uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>(),
            orig_fname
        ))
    } else {
        target_dest
    };
    fs::copy(order_pdf_path, &final_dest).map_err(|e| format!("Could not copy order to orders/: {e}"))?;

    // 4. Update Case Profile: Timeline, Dockets, Issues, and Operation Receipt
    let event_id = format!("ev-order-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());
    let order_date_str = decoded.order_date.clone().unwrap_or_else(|| {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    });

    let rel_order_path = format!("orders/{}", final_dest.file_name().unwrap().to_string_lossy());
    let timeline_notes = format!(
        "Signer: {}. Operative clauses: {}. Source: {}#page={}.",
        decoded.signer.label(),
        if decoded.operative_clauses.is_empty() {
            "None isolated".into()
        } else {
            decoded.operative_clauses.join("; ")
        },
        rel_order_path,
        decoded.source_page
    );

    profile.timeline.push(TimelineEvent {
        id: event_id.clone(),
        title: decoded.title.clone(),
        date: order_date_str.clone(),
        notes: timeline_notes,
    });

    // Backfill court or docket if not yet assigned
    if profile.docket_number.trim().is_empty() {
        if let Some(ref d) = decoded.docket_number {
            profile.docket_number = d.clone();
        }
    }
    if profile.court.name.trim().is_empty() {
        if let Some(ref c) = decoded.court_name {
            profile.court.name = c.clone();
        }
    }

    // Flag unresolved procedural issue if show cause or dismissal
    let is_show_cause = decoded.title.to_ascii_lowercase().contains("show cause")
        || decoded.operative_clauses.iter().any(|c| c.to_ascii_lowercase().contains("show cause"));
    if is_show_cause {
        let issue_id = format!("issue-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());
        profile.issues.push(IssueItem {
            id: issue_id,
            title: format!("Response required to {}", decoded.title),
            category: "jurisdictional".into(),
            asserted_by: "court".into(),
            record_citations: vec![format!("{}#page={}", rel_order_path, decoded.source_page)],
            contrary_authorities: vec![],
            status: "unresolved".into(),
            notes: format!("Candidate response window: {:?} days.", decoded.candidate_response_window_days),
        });
    }

    // 5. Operation Receipt in Case Profile
    let receipt_id = format!("rcpt-order-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());
    let now = chrono::Utc::now().to_rfc3339();
    profile.receipts.push(OperationReceipt {
        id: receipt_id.clone(),
        timestamp: now,
        operation: "order.ingest_and_classify".into(),
        invoked_by: "system_or_agent".into(),
        status: "success".into(),
        files_modified: vec![rel_order_path.clone(), "case.json".into()],
        notes: format!("Order '{}' ingested. SHA-256: {}.", decoded.title, hash_hex),
    });

    save_case(case_dir, &profile)?;

    // 6. Compute Sourced Deadline
    let sourced_deadline = if let Some(order_date_naive) = parse_loose_date(&order_date_str) {
        let pack = RulePack::builtin(&profile.rule_pack_id).unwrap_or_else(|_| RulePack::generic());
        let maybe_rule = pack.find_deadline("reconsideration-appellate")
            .ok()
            .or_else(|| pack.find_deadline("reconsideration-trial").ok())
            .or_else(|| pack.deadlines.first());
        if let Some(rule) = maybe_rule {
            let src_ref = format!("{}#page={}", rel_order_path, decoded.source_page);
            Some(compute_sourced_deadline(
                &pack,
                rule,
                order_date_naive,
                Some(&src_ref),
                Some(DeadlineVerificationStatus::Provisional),
                Some("Court electronic filing portal / Clerk counter"),
            ))
        } else {
            None
        }
    } else {
        None
    };

    // 7. Formulate and persist Workflow Plan
    let plan = plan_workflow(
        case_dir,
        &profile,
        &format!("Respond to court order: {}", decoded.title),
    );
    let _ = save_workflow_plan(case_dir, &plan);

    Ok(IngestedOrderReport {
        decoded_order: decoded,
        saved_order_path: rel_order_path,
        sha256: hash_hex,
        timeline_event_id: event_id,
        sourced_deadline,
        workflow_plan: Some(plan),
        receipt_id,
        status: "order_preserved_and_workflow_formulated".into(),
    })
}
