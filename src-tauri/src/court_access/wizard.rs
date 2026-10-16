use std::path::Path;
use chrono::Utc;

use super::directory::find_court_access_profile;
use super::types::{MissingRecordRetrievalItem, RecordRetrievalWizardPlan};
use crate::case_profile::types::{ActionTask, OperationReceipt};
use crate::case_profile::{load_case, save_case, CaseProfile};
use crate::record::inventory::load_record_inventory;
use crate::record::requests::{generate_records_request, CourtRecordsRequestPacket};

/// Plans court record retrieval based on current case gaps and court rules.
pub fn plan_record_retrieval(case_dir: &Path, profile: &CaseProfile) -> RecordRetrievalWizardPlan {
    let court_query = if !profile.court.name.is_empty() {
        &profile.court.name
    } else if !profile.governing_state.is_empty() {
        &profile.governing_state
    } else {
        "NJ"
    };

    let court_profile = find_court_access_profile(court_query)
        .or_else(|| find_court_access_profile("NJ"))
        .unwrap();

    let inventory = load_record_inventory(case_dir);
    let mut missing_records = Vec::new();

    if let Some(inv) = inventory {
        for m in inv.missing_records {
            missing_records.push(MissingRecordRetrievalItem {
                title: m.title.clone(),
                reason: m.reason.clone(),
                record_tier: m.record_type.clone(),
                target_agency_or_court: court_profile.court_name.clone(),
                required_form: if m.title.to_ascii_lowercase().contains("transcript") {
                    court_profile.transcript_request_form.clone()
                } else {
                    format!("{} Records Request", court_profile.jurisdiction_state)
                },
            });
        }
    }

    if missing_records.is_empty() {
        missing_records.push(MissingRecordRetrievalItem {
            title: "Official Register of Actions / Certified Docket Sheet".into(),
            reason: "Essential foundation: verify all entered court entries and dates before filing".into(),
            record_tier: "Tier1Official".into(),
            target_agency_or_court: court_profile.court_name.clone(),
            required_form: "Public Records Request Form".into(),
        });
        missing_records.push(MissingRecordRetrievalItem {
            title: "Underlying Oral Argument Hearing Transcripts / Audio Recordings".into(),
            reason: "Hearing transcripts are necessary to demonstrate issues were preserved on the record".into(),
            record_tier: "Tier2Hearing".into(),
            target_agency_or_court: court_profile.court_name.clone(),
            required_form: court_profile.transcript_request_form.clone(),
        });
    }

    let recommended_form = if court_profile.jurisdiction_state == "NJ" {
        "NJ Judiciary Form CN 10200".into()
    } else if court_profile.jurisdiction_state == "CO" {
        "Colorado Judicial Records Request (CJD 05-01 / JDF 4)".into()
    } else {
        "Federal PACER File Jacket Request / AO 435".into()
    };

    let recommended_form_number = if court_profile.jurisdiction_state == "NJ" {
        "CN 10200".into()
    } else if court_profile.jurisdiction_state == "CO" {
        "CJD 05-01".into()
    } else {
        "AO 435".into()
    };

    let mut submission_channels = Vec::new();
    if court_profile.pro_se_efiling_allowed {
        submission_channels.push(format!("Electronic Portal: {} ({})", court_profile.electronic_portal_name, court_profile.electronic_portal_url));
    }
    submission_channels.push(format!("Clerk Counter / Mail: {} ({})", court_profile.physical_address, court_profile.clerk_counter_hours));

    let fee_waiver_rule = if court_profile.indigent_fee_waiver_available {
        Some(if court_profile.jurisdiction_state == "NJ" {
            "R. 1:13-2 (Indigent Filer Fee Waiver Application)".into()
        } else if court_profile.jurisdiction_state == "CO" {
            "C.R.C.P. 121 § 1-15 & JDF 205 / 206 (Motion to File Without Payment)".into()
        } else {
            "28 U.S.C. § 1915 (In Forma Pauperis Proceedings)".into()
        })
    } else {
        None
    };

    let step_by_step_instructions = vec![
        format!("1. Verify the court venue and case number: {}", if profile.docket_number.is_empty() { "Docket number missing from profile" } else { &profile.docket_number }),
        format!("2. Select requested items from your record gap checklist (identified {} missing items).", missing_records.len()),
        format!("3. Choose submission route: {}.", submission_channels.first().cloned().unwrap_or_default()),
        "4. Attach the generated Request Form and keep a copy in your case's evidence folder with date and time of submission.".into(),
        "5. Mark calendar for clerk acknowledgment window (typically 7 to 14 business days).".into(),
    ];

    RecordRetrievalWizardPlan {
        case_id: profile.id.clone(),
        court_id: court_profile.court_id,
        court_name: court_profile.court_name,
        missing_records,
        recommended_form,
        recommended_form_number,
        submission_channels,
        estimated_cost_explanation: court_profile.statutory_fees_summary,
        fee_waiver_rule,
        followup_reminder_days: 14,
        step_by_step_instructions,
    }
}

/// Generates the request packet and registers follow-up tasks in the case profile.
pub fn execute_record_retrieval_request(
    case_dir: &Path,
    profile: &mut CaseProfile,
    state_override: Option<&str>,
) -> Result<CourtRecordsRequestPacket, String> {
    let packet = generate_records_request(case_dir, profile, state_override)?;

    let task_id = format!("task-records-req-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());
    let now_date = Utc::now().date_naive();
    let due_date = now_date + chrono::Duration::days(14);

    let task = ActionTask {
        id: task_id.clone(),
        title: format!("Follow up on Court Records Request ({})", packet.state),
        status: "pending".into(),
        due_date: Some(due_date.format("%Y-%m-%d").to_string()),
        assigned_to: Some("Filer".into()),
        category: "records_request".into(),
        target_ref: Some(packet.court_name.clone()),
        notes: format!("Requested {} items from {}. Submission method: {}", packet.records_requested.len(), packet.court_name, packet.submission_method),
    };

    profile.tasks.push(task);

    let receipt = OperationReceipt {
        id: format!("rcpt-records-req-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
        timestamp: Utc::now().to_rfc3339(),
        operation: "court_records_request_generated".into(),
        invoked_by: "user".into(),
        status: "success".into(),
        files_modified: vec!["records_request.md".into()],
        notes: format!("Generated official {} records request for {} items.", packet.state, packet.records_requested.len()),
    };
    profile.receipts.push(receipt);

    save_case(case_dir, profile)?;

    Ok(packet)
}
