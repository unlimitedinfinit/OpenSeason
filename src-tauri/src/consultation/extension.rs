use crate::case_profile::CaseProfile;
use super::types::{ExtensionProposal, ExtensionRoute};

/// Evaluates whether an extension of time should proceed via consent letter
/// or formal motion for enlargement, and flags jurisdictional non-alterable deadlines.
pub fn evaluate_extension_request(
    profile: &CaseProfile,
    target_deadline_name: &str,
    requested_days: u32,
    consent_obtained: bool,
) -> ExtensionProposal {
    let lower_target = target_deadline_name.to_lowercase();
    let state = if !profile.governing_state.is_empty() { profile.governing_state.as_str() } else { "NJ" };

    // Check if this is an initial appeal deadline which has statutory hard limits
    if lower_target.contains("notice of appeal") || lower_target.contains("appeal clock") {
        if requested_days > 30 {
            return ExtensionProposal {
                case_id: profile.id.clone(),
                route: ExtensionRoute::ProhibitedByRule,
                governing_rule: if state == "CO" { "C.A.R. 4(a) & 26(b)".into() } else { "N.J. Court Rule 2:4-4(a)".into() },
                maximum_days_allowed: 30,
                requested_days,
                jurisdiction: state.into(),
                consent_obtained,
                good_cause_required: true,
                non_alterable_warning: Some("Jurisdictional Notice of Appeal deadlines cannot be enlarged beyond 30 days under any circumstances. Failure to file within the statutory window deprives the appellate court of jurisdiction.".into()),
                rendered_document_text: String::new(),
                recommended_form_number: None,
                step_instructions: vec![
                    "Immediately file Notice of Appeal before the expiration of the statutory deadline.".into(),
                    "Extensions exceeding 30 days are barred as a matter of subject matter jurisdiction.".into(),
                ],
            };
        }

        // Within 30 days requires formal motion with excusable neglect/good cause
        let motion_text = format!(
            "MOTION FOR EXTENSION OF TIME TO FILE NOTICE OF APPEAL\n\n\
            COMES NOW Filer {}, proceeding without counsel, and respectfully moves the Court pursuant to {} \
            for an order extending the time to file a Notice of Appeal by {} days.\n\n\
            1. The order sought to be reviewed was entered on or about {}.\n\
            2. Good cause and excusable neglect exist for this extension due to administrative and record verification needs.\n\
            3. No party will suffer unfair prejudice by the requested enlargement.\n\n\
            WHEREFORE, Filer requests that the Court enter an order extending the deadline to file the Notice of Appeal.",
            profile.filer.name,
            if state == "CO" { "C.A.R. 4(a)" } else { "N.J. Court Rule 2:4-4(a)" },
            requested_days,
            profile.timeline.first().map(|e| e.date.as_str()).unwrap_or("the date of entry")
        );

        return ExtensionProposal {
            case_id: profile.id.clone(),
            route: ExtensionRoute::FormalMotionForEnlargement,
            governing_rule: if state == "CO" { "C.A.R. 4(a)".into() } else { "N.J. Court Rule 2:4-4(a)".into() },
            maximum_days_allowed: 30,
            requested_days,
            jurisdiction: state.into(),
            consent_obtained,
            good_cause_required: true,
            non_alterable_warning: Some("Requires showing of good cause and lack of prejudice. Cannot be granted by consent of parties.".into()),
            rendered_document_text: motion_text,
            recommended_form_number: Some(if state == "CO" { "JDF 1314".into() } else { "Form CN 10503".into() }),
            step_instructions: vec![
                "File formal motion with court clerk prior to the expiration of the 30-day extension window.".into(),
                "Attach certification setting forth verified factual grounds for good cause.".into(),
            ],
        };
    }

    // Brief or Procedural Motion Deadlines
    if consent_obtained && requested_days <= 30 {
        let letter_text = format!(
            "CONSENT STIPULATION EXTENDING TIME TO FILE {}\n\n\
            To the Clerk of the Court:\n\n\
            Please take notice that all parties to the above-captioned matter hereby consent and stipulate \
            pursuant to {} to an automatic thirty (30) day extension of time for filing {}.\n\n\
            The new filing deadline shall be thirty (30) days from the existing deadline.\n\n\
            Dated: {}\n\
            Consented to by all parties of record.",
            target_deadline_name.to_uppercase(),
            if state == "CO" { "C.A.R. 31(a)" } else { "N.J. Court Rule 2:4-4(c)" },
            target_deadline_name,
            chrono::Utc::now().format("%Y-%m-%d")
        );

        ExtensionProposal {
            case_id: profile.id.clone(),
            route: ExtensionRoute::ConsentStipulationLetter,
            governing_rule: if state == "CO" { "C.A.R. 31(a)".into() } else { "N.J. Court Rule 2:4-4(c)".into() },
            maximum_days_allowed: 30,
            requested_days,
            jurisdiction: state.into(),
            consent_obtained: true,
            good_cause_required: false,
            non_alterable_warning: None,
            rendered_document_text: letter_text,
            recommended_form_number: Some("Letter / Consent Stipulation".into()),
            step_instructions: vec![
                "Obtain written or electronic consent confirmation from opposing counsel or pro se parties.".into(),
                "File letter with appellate clerk prior to existing deadline expiration.".into(),
                "No judicial order or motion fee required for first consent extension.".into(),
            ],
        }
    } else {
        let motion_text = format!(
            "NOTICE OF MOTION FOR ENLARGEMENT OF TIME TO FILE {}\n\n\
            PLEASE TAKE NOTICE that the undersigned filer moves before the Court pursuant to {} \
            for an order enlarging the time to file {} by {} days.\n\n\
            In support of this motion, filer relies upon the accompanying certification showing good cause.\n\n\
            Respectfully submitted,\n/s/ {}",
            target_deadline_name.to_uppercase(),
            if state == "CO" { "C.R.C.P. 6(b) / C.A.R. 26(b)" } else { "N.J. Court Rule 1:3-4 / 2:4-4(a)" },
            target_deadline_name,
            requested_days,
            profile.filer.signature_name
        );

        ExtensionProposal {
            case_id: profile.id.clone(),
            route: ExtensionRoute::FormalMotionForEnlargement,
            governing_rule: if state == "CO" { "C.R.C.P. 6(b)".into() } else { "N.J. Court Rule 1:3-4".into() },
            maximum_days_allowed: 60,
            requested_days,
            jurisdiction: state.into(),
            consent_obtained: false,
            good_cause_required: true,
            non_alterable_warning: None,
            rendered_document_text: motion_text,
            recommended_form_number: Some(if state == "CO" { "JDF 1314".into() } else { "Form CN 10503".into() }),
            step_instructions: vec![
                "File formal notice of motion and proposed order with the clerk.".into(),
                "Serve opposing parties and file certificate of service.".into(),
                "Detail diligent efforts and good cause in supporting certification.".into(),
            ],
        }
    }
}

/// Assesses whether a case requires a 30-day extension of time due to an unbuilt
/// case record baseline, missing hearing transcripts, or unfinalized intake.
pub fn assess_baseline_extension(
    profile: &CaseProfile,
    case_dir: Option<&std::path::Path>,
) -> super::types::BaselineExtensionAssessment {
    let state = if !profile.governing_state.is_empty() {
        profile.governing_state.as_str()
    } else {
        "NJ"
    };

    let mut missing = Vec::new();
    let mut evidence_count = 0;
    let mut has_transcripts = false;
    let mut order_count = 0;

    if let Some(dir) = case_dir {
        let ev_dir = dir.join("evidence");
        if let Ok(entries) = std::fs::read_dir(ev_dir) {
            for entry in entries.flatten() {
                evidence_count += 1;
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("transcript") || name.contains("record") || name.contains("hearing") {
                    has_transcripts = true;
                }
            }
        }
        let ord_dir = dir.join("orders");
        if let Ok(entries) = std::fs::read_dir(ord_dir) {
            for _ in entries.flatten() {
                order_count += 1;
            }
        }
    }

    if evidence_count == 0 {
        missing.push("No evidence files or exhibits indexed in case folder.".into());
    }
    if !has_transcripts {
        missing.push("Official hearing transcripts not yet requested or retrieved from clerk/transcriber.".into());
    }
    if order_count == 0 {
        missing.push("Lower court orders not yet imported into orders/ folder.".into());
    }
    if profile.timeline.is_empty() {
        missing.push("Procedural timeline events not yet logged or reconciled.".into());
    }
    if profile.issues.is_empty() {
        missing.push("Legal issues and claims not yet formulated from verified record.".into());
    }

    let is_recommended = !missing.is_empty();
    let recommended_days = 30;
    let recommended_target = match profile.document_kind.as_str() {
        "appellate_brief" => "Appellant's Brief",
        "notice_of_appeal" => "Notice of Appeal Record Verification",
        "reconsideration" => "Motion for Reconsideration Support",
        _ => "Initial Responsive Pleading / Brief",
    };

    let governing_rule = match state {
        "CO" => "C.A.R. 31(a) (Brief Extension by Stipulation) / C.R.C.P. 6(b)",
        "FED" => "Fed. R. App. P. 26(b) / Fed. R. Civ. P. 6(b)",
        _ => "N.J. Ct. R. 2:6-11(f) (30-day Consent Extension) & R. 1:3-4",
    };

    let form_number = match state {
        "CO" => Some("JDF 1314 (Motion for Extension of Time)".into()),
        _ => Some("Form CN 10503 (Appellate Motion for Extension)".into()),
    };

    let good_cause = format!(
        "Filer {} is proceeding without counsel (self-represented) and requires a 30-day extension \
        of time to establish a reliable baseline of the case record. Specifically, filer is actively \
        seeking official hearing transcripts from the reporting service, retrieving lower court records, \
        and assembling the evidentiary appendix to ensure all filings are accurate and grounded in the record.",
        profile.filer.name
    );

    let conferral_letter = format!(
        "RE: Request for Consent to 30-Day Extension of Time\n\
        Case: {} (Docket: {})\n\n\
        Dear Counsel / Opposing Party:\n\n\
        I am contacting you regarding the upcoming deadline for {} in the above-captioned matter. \
        As I am proceeding self-represented and actively working to assemble the lower court record \
        and obtain hearing transcripts, I write to respectfully inquire whether you will consent to a \
        standard 30-day extension of time pursuant to {}.\n\n\
        Under court rules, an initial 30-day enlargement for record assembly is customary and avoids unnecessary \
        motion practice. If you consent, please confirm by email or return signature so we may submit a joint \
        consent letter to the clerk.\n\n\
        If I do not receive a response within two business days, I will proceed with a formal motion for enlargement \
        certifying my diligent effort to confer.\n\n\
        Thank you for your professional courtesy,\n\n\
        /s/ {}\n\
        {}",
        profile.title,
        if !profile.docket_number.is_empty() { &profile.docket_number } else { "Pending Assignment" },
        recommended_target,
        governing_rule,
        profile.filer.name,
        profile.filer.email
    );

    let formal_motion = format!(
        "MOTION FOR ENLARGEMENT OF TIME TO FILE {}\n\n\
        COMES NOW Filer {}, self-represented, and moves the Court pursuant to {} \
        for an order extending the time to file {} by {} days.\n\n\
        CERTIFICATION OF GOOD CAUSE:\n\
        1. Filer is actively reconstructing the trial and hearing record and requires time to retrieve official transcripts.\n\
        2. {}\n\
        3. Prior to filing this motion, filer sought consent from opposing counsel; this motion is brought in good faith and not for purposes of undue delay.\n\
        4. No party will suffer prejudice by this first extension.\n\n\
        Respectfully submitted,\n/s/ {}",
        recommended_target.to_uppercase(),
        profile.filer.name,
        governing_rule,
        recommended_target,
        recommended_days,
        good_cause,
        profile.filer.name
    );

    let conferral_instructions = vec![
        "1. Send the Conferral Email/Letter to opposing counsel or pro se parties.".into(),
        "2. If opposing counsel consents: File the Consent Stipulation Letter with the clerk. No motion fee or judge order required.".into(),
        "3. If opposing counsel refuses or does not respond within 48 hours: File the Formal Motion for Enlargement noting diligent conferral efforts.".into(),
    ];

    let filing_instructions = vec![
        "1. Check the local electronic portal (JEDS for NJ, CCE for CO, CM/ECF for Federal).".into(),
        "2. Include the signed Certificate of Service confirming delivery to all parties.".into(),
        "3. File before the existing deadline expires to protect your appeal rights.".into(),
    ];

    super::types::BaselineExtensionAssessment {
        case_id: profile.id.clone(),
        is_extension_recommended: is_recommended,
        urgency_level: if is_recommended { "baseline_setup".into() } else { "normal".into() },
        missing_elements: missing,
        recommended_target: recommended_target.into(),
        recommended_days,
        governing_rule: governing_rule.into(),
        recommended_form_number: form_number,
        good_cause_statement: good_cause,
        conferral_letter_draft: conferral_letter,
        formal_motion_draft: formal_motion,
        conferral_instructions,
        filing_instructions,
    }
}
