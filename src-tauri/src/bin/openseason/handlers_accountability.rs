use std::error::Error;
use std::fs;
use serde_json::json;

use super::args::{AccountabilityCmd, AdaCmd, CourtCmd, OrderCmd, RecordCmd, ReviewCmd, TimelineCmd};
use open_season_lib::ada::types::{AccommodationCategory, AdaRequestPayload};
use open_season_lib::case_profile::load_case;

pub fn handle_record(cmd: RecordCmd, json: bool) -> Result<(), Box<dyn Error>> {
    match cmd {
        RecordCmd::Inventory { path } => {
            let profile = load_case(&path)?;
            let report = open_season_lib::record::inventory::scan_case_record(&path, &profile)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "never_serves": true,
                    "data": report
                }))?);
            } else {
                println!("=== Case Record Inventory: {} ===", profile.title);
                println!("{}", report.reconciliation_summary);
                for entry in &report.entries {
                    println!("  [{}] {} ({}) - SHA-256: {} - {}",
                        entry.id, entry.filename, entry.tier.label(), &entry.sha256[..8],
                        if entry.verified { "VERIFIED" } else { "UNVERIFIED" });
                }
            }
        }
        RecordCmd::Reconcile { path, docket } => {
            let profile = load_case(&path)?;
            let docket_text = match docket {
                Some(p) => fs::read_to_string(&p)?,
                None => String::new(),
            };
            let entries = open_season_lib::record::reconciliation::parse_text_docket(&docket_text);
            let report = open_season_lib::record::reconciliation::reconcile_docket(&path, &profile, &entries)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "never_serves": true,
                    "data": report
                }))?);
            } else {
                println!("=== Record Reconciliation: {} ===", profile.title);
                println!("{}", report.reconciliation_summary);
                if !report.missing_records.is_empty() {
                    println!("\nMissing Records Identified (Needed to perfect record):");
                    for m in &report.missing_records {
                        println!("  - [{}] {}: {}", m.record_type, m.title, m.reason);
                        println!("    Recommended Action: {}", m.recommended_action);
                    }
                }
            }
        }
        RecordCmd::Missing { path } => {
            let inv = open_season_lib::record::inventory::load_record_inventory(&path);
            let missing = inv.map(|r| r.missing_records).unwrap_or_default();
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "data": missing
                }))?);
            } else {
                println!("=== Missing Records for Case ===");
                if missing.is_empty() {
                    println!("No missing records flagged. Run 'record reconcile' with a docket sheet to verify completeness.");
                } else {
                    for m in &missing {
                        println!("- {} [{}]: {}", m.title, m.record_type, m.reason);
                        println!("  Form: {:?}", m.request_form);
                    }
                }
            }
        }
        RecordCmd::RequestForm { path, state } => {
            let profile = load_case(&path)?;
            let packet = open_season_lib::record::requests::generate_records_request(&path, &profile, state.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "never_serves": true,
                    "data": packet
                }))?);
            } else {
                println!("{}", packet.rendered_request_text);
                println!("--- Guidelines ---");
                println!("{}", packet.instructions);
                println!("Fee notice: {}", packet.statutory_fee_notice);
            }
        }
        RecordCmd::PlanRetrieval { path } => {
            let profile = load_case(&path)?;
            let plan = open_season_lib::court_access::plan_record_retrieval(&path, &profile);
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "never_serves": true,
                    "data": plan
                }))?);
            } else {
                println!("=== Court Record Retrieval Plan: {} ===", profile.title);
                println!("Court: {}", plan.court_name);
                println!("Recommended Form: {} ({})", plan.recommended_form, plan.recommended_form_number);
                println!("Estimated Cost: {}", plan.estimated_cost_explanation);
                if let Some(ref w) = plan.fee_waiver_rule {
                    println!("Fee Waiver Rule: {}", w);
                }
                println!("\nMissing Items Identified ({}):", plan.missing_records.len());
                for item in &plan.missing_records {
                    println!("  - [{}] {}: {}", item.record_tier, item.title, item.reason);
                }
                println!("\nSubmission Channels:");
                for ch in &plan.submission_channels {
                    println!("  - {}", ch);
                }
            }
        }
    }
    Ok(())
}

pub fn handle_review(cmd: ReviewCmd, json: bool) -> Result<(), Box<dyn Error>> {
    match cmd {
        ReviewCmd::Record { path } => {
            let profile = load_case(&path)?;
            let report = open_season_lib::accountability::audit::audit_record_discrepancies(&path, &profile)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "never_serves": true,
                    "data": report
                }))?);
            } else {
                println!("=== Record Discrepancy Audit: {} ===", profile.title);
                println!("{}", report.summary);
                println!("\n> [!] {}", report.non_tolling_warning);
                for issue in &report.issues {
                    println!("\nIssue [{}]: {}", issue.id, issue.title);
                    println!("  Finding Source: {} (Page {:?})", issue.finding_source.document_name, issue.finding_source.page);
                    println!("  Quoted Finding: \"{}\"", issue.finding_source.quoted_text);
                    println!("  Status: {}", issue.record_support_status.label());
                    for route in &issue.available_routes {
                        println!("    -> Route: {} ({})", route.title, route.rule_citation);
                    }
                }
            }
        }
        ReviewCmd::StressTest { path, issue } => {
            let profile = load_case(&path)?;
            let results = if let Some(target_id) = issue {
                if let Some(target_issue) = profile.issues.iter().find(|i| i.id == target_id) {
                    vec![open_season_lib::consultation::stress_test_issue(&profile, target_issue)]
                } else {
                    return Err(format!("Issue with id '{}' not found in case.", target_id).into());
                }
            } else {
                open_season_lib::consultation::stress_test_all_issues(&profile)
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "never_serves": true,
                    "data": results
                }))?);
            } else {
                println!("=== Devil's Advocate Issue Stress Test: {} ===", profile.title);
                if results.is_empty() {
                    println!("No issues found to stress test. Add issues in case profile or intake wizard.");
                } else {
                    for res in &results {
                        println!("\nIssue [{}]: {} ({})", res.issue_id, res.issue_title, res.overall_risk_rating);
                        println!("  Asserted By: {}", res.asserted_by);
                        println!("  Summary: {}", res.procedural_posture_summary);
                        for v in &res.vulnerabilities {
                            println!("  - [{:?}] {} [{:?}]", v.vulnerability_type, v.title, v.severity);
                            println!("    Predicted Opposition: {}", v.opposing_argument_prediction);
                            println!("    Suggested Inquiry: {}", v.suggested_cure_or_inquiry);
                            if let Some(c) = &v.citation_anchor {
                                println!("    Anchor: {}", c);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn handle_accountability(cmd: AccountabilityCmd, json: bool) -> Result<(), Box<dyn Error>> {
    match cmd {
        AccountabilityCmd::Routes { path, issue } => {
            let profile = load_case(&path)?;
            let report = open_season_lib::accountability::audit::audit_record_discrepancies(&path, &profile)?;
            let routes = if let Some(target_id) = issue {
                report.issues.into_iter().find(|i| i.id == target_id).map(|i| i.available_routes).unwrap_or_default()
            } else {
                open_season_lib::accountability::routes::determine_routes_for_issue(
                    &profile,
                    open_season_lib::accountability::types::RecordSupportStatus::UnsupportedInRecord,
                    false,
                )
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "data": routes
                }))?);
            } else {
                println!("=== Available Procedural & Accountability Routes ===");
                for r in &routes {
                    println!("\n* {}", r.title);
                    println!("  Rule / Authority: {}", r.rule_citation);
                    println!("  Governing Standard: {}", r.governing_standard);
                    if let Some(dl) = &r.deadline_notice {
                        println!("  Deadline: {}", dl);
                    }
                    if r.is_disciplinary {
                        println!("  DISCIPLINARY ALERT: {}", r.warning_notice);
                    }
                }
            }
        }
        AccountabilityCmd::Resources { state, category } => {
            let list = open_season_lib::accountability::resources::list_accountability_resources(state.as_deref(), category.as_deref());
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "data": list
                }))?);
            } else {
                println!("=== Official Accountability & Oversight Resources ===");
                for item in &list {
                    println!("\n* {} [{}] ({})", item.title, item.state, item.agency);
                    println!("  Portal: {}", item.portal_url);
                    if let Some(pdf) = &item.direct_pdf_url {
                        println!("  Form PDF: {}", pdf);
                    }
                    println!("  Scope: {}", item.jurisdictional_scope);
                    println!("  Advisory: {}", item.non_appeal_warning);
                }
            }
        }
        AccountabilityCmd::Packet { path, issue, dry_run: _ } => {
            let profile = load_case(&path)?;
            let audit_rep = open_season_lib::accountability::audit::audit_record_discrepancies(&path, &profile)?;
            let target_issue = audit_rep
                .issues
                .into_iter()
                .find(|i| i.id == issue)
                .ok_or_else(|| format!("Issue '{issue}' not found in case."))?;
            let packet = open_season_lib::accountability::packets::assemble_factual_packet(&path, &profile, &target_issue)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "never_serves": true,
                    "data": packet
                }))?);
            } else {
                println!("{}", packet.rendered_markdown);
            }
        }
    }
    Ok(())
}

pub fn handle_ada(cmd: AdaCmd, json: bool) -> Result<(), Box<dyn Error>> {
    match cmd {
        AdaCmd::Petition { path, need, remote, extended_time, mobility, hearing } => {
            let profile = load_case(&path)?;
            let mut accs = Vec::new();
            if remote {
                accs.push(AccommodationCategory::RemoteTelephonicAppearance);
            }
            if extended_time {
                accs.push(AccommodationCategory::ExtendedReadingTime);
            }
            if mobility {
                accs.push(AccommodationCategory::PhysicalMobilityAccess);
            }
            if hearing {
                accs.push(AccommodationCategory::VisualHearingAssistance);
            }
            if accs.is_empty() {
                accs.push(AccommodationCategory::RemoteTelephonicAppearance);
            }

            let filer_name = if !profile.filer.name.is_empty() {
                profile.filer.name.clone()
            } else {
                profile.caption.plaintiffs.first().map(|p| p.name.clone()).unwrap_or_else(|| "Self-Represented Litigant".into())
            };

            let payload = AdaRequestPayload {
                case_id: profile.id.clone(),
                requester_name: filer_name,
                requester_role: "Self-Represented Litigant".into(),
                contact_phone: profile.filer.phone.clone(),
                contact_email: profile.filer.email.clone(),
                accommodations: accs,
                specific_needs_narrative: need,
                upcoming_date: None,
            };

            let petition = open_season_lib::ada::generator::generate_ada_petition(&profile, &payload)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "never_serves": true,
                    "data": petition
                }))?);
            } else {
                println!("{}", petition.rendered_petition_text);
                println!("--- Submission Instructions ---");
                println!("{}", petition.submission_guidelines);
            }
        }
    }
    Ok(())
}

pub fn handle_order(cmd: OrderCmd, json: bool) -> Result<(), Box<dyn Error>> {
    match cmd {
        OrderCmd::Decode { pdf, case } => {
            if let Some(case_path) = case {
                let ingested = open_season_lib::order::decoder::ingest_order_into_case(&case_path, &pdf)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&json!({
                        "ok": true,
                        "never_files": true,
                        "never_serves": true,
                        "data": ingested
                    }))?);
                } else {
                    println!("=== Court Order Ingested & Classified ===");
                    println!("Order Title: {}", ingested.decoded_order.title);
                    println!("Preserved Path: {}", ingested.saved_order_path);
                    println!("SHA-256 Hash: {}", ingested.sha256);
                    println!("Signer: {}", ingested.decoded_order.signer.label());
                    println!("Timeline Event: Registered ({})", ingested.timeline_event_id);
                    if let Some(dl) = &ingested.sourced_deadline {
                        println!("\n--- Sourced Response Deadline ---");
                        println!("Due Date: {} ({:?} - {} days remaining)", dl.due_date, dl.status, dl.days_remaining);
                        println!("Trigger: {} ({})", dl.trigger_event, dl.trigger_source);
                        println!("Governing Rule: {}", dl.governing_rule_citation);
                        println!("Calculation: {}", dl.calculation_basis);
                        println!("Filing Method: {}", dl.filing_method);
                    }
                    if let Some(plan) = &ingested.workflow_plan {
                        println!("\n--- Formulated Workflow Plan ({}) ---", plan.steps.len());
                        for (i, step) in plan.steps.iter().enumerate() {
                            println!("  {}. {} (Tool: {})", i + 1, step.title, step.tool_command);
                        }
                    }
                    println!("\nOperation Receipt: {} logged to case.json", ingested.receipt_id);
                }
            } else {
                let decoded = open_season_lib::order::decoder::decode_order_pdf(&pdf)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&json!({
                        "ok": true,
                        "never_files": true,
                        "never_serves": true,
                        "data": decoded
                    }))?);
                } else {
                    println!("=== Court Order Decoded: {} ===", decoded.title);
                    if let Some(c) = &decoded.court_name { println!("Court: {c}"); }
                    if let Some(d) = &decoded.docket_number { println!("Docket: {d}"); }
                    println!("Signer: {}", decoded.signer.label());
                    println!("Notice Type: {}", if decoded.is_boilerplate_notice { "Boilerplate Administrative Notice" } else { "Operative Court Order" });
                    if let Some(w) = decoded.candidate_response_window_days {
                        println!("Candidate Response Window: {w} days");
                    }
                    if let Some(cd) = &decoded.candidate_deadline {
                        println!("Candidate Deadline: {cd} ({})", decoded.status);
                    }
                    if !decoded.governing_rules_cited.is_empty() {
                        println!("Rules Cited: {}", decoded.governing_rules_cited.join(", "));
                    }
                    if !decoded.operative_clauses.is_empty() {
                        println!("\nOperative Clauses:");
                        for clause in &decoded.operative_clauses {
                            println!("  - {clause}");
                        }
                    }
                    if !decoded.uncertainty_notes.is_empty() {
                        println!("\nUncertainties & Warnings:");
                        for note in &decoded.uncertainty_notes {
                            println!("  [!] {note}");
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn handle_timeline(cmd: TimelineCmd, json: bool) -> Result<(), Box<dyn Error>> {
    match cmd {
        TimelineCmd::AuditMail { path, order_date, postmark, delivery, clerk_date, state } => {
            let profile = load_case(&path)?;
            let resolved_state = state.unwrap_or_else(|| {
                if !profile.governing_state.is_empty() {
                    profile.governing_state.clone()
                } else {
                    "CO".into()
                }
            });
            let report = open_season_lib::order::mail_audit::audit_mail_gap(
                &profile.id,
                &order_date,
                postmark.as_deref(),
                delivery.as_deref(),
                clerk_date.as_deref(),
                Some(&resolved_state),
            );
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "never_serves": true,
                    "data": report
                }))?);
            } else {
                println!("=== Mail-Gap & Jurisdictional Trap Audit: {} ===", profile.title);
                println!("Status: {}", report.status);
                println!("Receipt Rule: {}", report.jurisdiction_receipt_rule);
                println!("Mail Delay: {} days (Holiday/Weekend Impact: {})", report.mail_delay_days, report.holiday_or_weekend_impact);
                if !report.jurisdictional_traps.is_empty() {
                    println!("\nJurisdictional Traps:");
                    for trap in &report.jurisdictional_traps {
                        println!("  [!] {trap}");
                    }
                }
                if !report.recommended_actions.is_empty() {
                    println!("\nRecommended Inquiries & Actions:");
                    for act in &report.recommended_actions {
                        println!("  - {act}");
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn handle_court(cmd: CourtCmd, json: bool) -> Result<(), Box<dyn Error>> {
    match cmd {
        CourtCmd::List => {
            let profiles = open_season_lib::court_access::get_all_court_profiles();
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "never_serves": true,
                    "data": profiles
                }))?);
            } else {
                println!("=== Bundled Court Access Directory ===");
                for p in &profiles {
                    println!("\n[{}] {} ({})", p.court_id, p.court_name, p.jurisdiction_state);
                    println!("  Hours: {}", p.clerk_counter_hours);
                    println!("  Address: {}", p.physical_address);
                    println!("  Portal: {} ({})", p.electronic_portal_name, p.electronic_portal_url);
                    println!("  Pro Se E-Filing: {}", if p.pro_se_efiling_allowed { "ALLOWED" } else { "MOTION/PAPER REQUIRED" });
                    println!("  Access Rule: {}", p.records_access_rule);
                }
            }
        }
        CourtCmd::Info { query } => {
            let profile = open_season_lib::court_access::find_court_access_profile(&query);
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "never_files": true,
                    "never_serves": true,
                    "data": profile
                }))?);
            } else {
                match profile {
                    Some(p) => {
                        println!("=== Court Access Profile: {} ===", p.court_name);
                        println!("Jurisdiction: {}", p.jurisdiction_state);
                        println!("Clerk Counter Hours: {}", p.clerk_counter_hours);
                        println!("Physical Address: {}", p.physical_address);
                        println!("Phone: {}", p.phone);
                        println!("Electronic Portal: {} ({})", p.electronic_portal_name, p.electronic_portal_url);
                        println!("Pro Se E-Filing: {}", if p.pro_se_efiling_allowed { "ALLOWED" } else { "PAPER ONLY" });
                        println!("Docket Search: {}", p.docket_search_url);
                        println!("Records Access Rule: {}", p.records_access_rule);
                        println!("Transcript Form: {}", p.transcript_request_form);
                        println!("Transcript Deposit Rule: {}", p.transcript_deposit_rules);
                        println!("Statutory Fees: {}", p.statutory_fees_summary);
                        if !p.special_instructions.is_empty() {
                            println!("\nSpecial Instructions:");
                            for s in &p.special_instructions {
                                println!("  - {}", s);
                            }
                        }
                    }
                    None => {
                        println!("No court access profile found matching '{}'.", query);
                    }
                }
            }
        }
    }
    Ok(())
}


