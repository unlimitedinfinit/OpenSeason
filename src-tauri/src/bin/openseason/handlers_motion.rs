use open_season_lib::case_profile;
use super::args::MotionCmd;
use super::emit;

pub fn handle_motion(action: MotionCmd, json: bool) -> Result<(), String> {
    match action {
        MotionCmd::List { path } => {
            let profile = case_profile::load_case(&path)?;
            if json {
                emit(true, &profile.filings, vec![]);
            } else {
                println!("Multi-Filing Motion Docket ({} filings):", profile.filings.len());
                for f in &profile.filings {
                    println!("  [{}] {} ({}) - Status: {}", f.id, f.title, f.kind, f.status);
                }
            }
            Ok(())
        }
        MotionCmd::Create { path, title, kind, rule, text } => {
            let mut profile = case_profile::load_case(&path)?;
            let now = chrono::Utc::now().to_rfc3339();
            let filing = case_profile::CaseFilingItem {
                id: uuid::Uuid::new_v4().to_string(),
                title,
                kind,
                rule,
                status: "draft".to_string(),
                user_text: text,
                relief_sought: String::new(),
                created_at: now.clone(),
                updated_at: now,
            };
            profile.filings.push(filing.clone());
            case_profile::save_case(&path, &profile)?;
            if json {
                emit(true, &filing, vec![]);
            } else {
                println!("Created filing '{}' (ID: {})", filing.title, filing.id);
            }
            Ok(())
        }
        MotionCmd::Export { path, id } => {
            let profile = case_profile::load_case(&path)?;
            let filing = profile.filings.iter().find(|f| f.id == id)
                .ok_or_else(|| format!("Filing ID '{id}' not found in case."))?;
            let paths = open_season_lib::documents::export_custom_filing(&path, &profile, &filing.title, &filing.user_text, None)?;
            if json {
                emit(true, &paths, vec![]);
            } else {
                println!("Exported filing '{}':", filing.title);
                println!("  Word: {}", paths.docx.display());
                println!("  PDF:  {}", paths.pdf.display());
            }
            Ok(())
        }
        MotionCmd::Extension { path, target, days, consent, suggest } => {
            let profile = case_profile::load_case(&path)?;
            if suggest || target.is_empty() {
                let assessment = open_season_lib::consultation::assess_baseline_extension(&profile, Some(&path));
                if json {
                    emit(true, &assessment, vec![]);
                } else {
                    println!("=== 30-Day Extension & Baseline Assessment ===");
                    println!("Extension Recommended: {}", assessment.is_extension_recommended);
                    println!("Urgency Level:         {}", assessment.urgency_level);
                    println!("Recommended Target:    {}", assessment.recommended_target);
                    println!("Governing Rule:        {}", assessment.governing_rule);
                    if let Some(form) = &assessment.recommended_form_number {
                        println!("Recommended Form:      {}", form);
                    }
                    if !assessment.missing_elements.is_empty() {
                        println!("\nMissing Case Elements:");
                        for m in &assessment.missing_elements {
                            println!("  - {m}");
                        }
                    }
                    println!("\n--- Good Cause Statement ---\n{}", assessment.good_cause_statement);
                    println!("\n--- Opposing Counsel Conferral Letter ---\n{}", assessment.conferral_letter_draft);
                    println!("\n--- Formal Motion for Enlargement ---\n{}", assessment.formal_motion_draft);
                }
            } else {
                let prop = open_season_lib::consultation::evaluate_extension_request(&profile, &target, days, consent);
                if json {
                    emit(true, &prop, vec![]);
                } else {
                    println!("=== Extension Route: {:?} ===", prop.route);
                    println!("Governing Rule: {}", prop.governing_rule);
                    println!("Requested Days: {} (Max: {})", prop.requested_days, prop.maximum_days_allowed);
                    if let Some(warn) = &prop.non_alterable_warning {
                        println!("WARNING: {}", warn);
                    }
                    if !prop.rendered_document_text.is_empty() {
                        println!("\n--- Generated Text ---\n{}\n---------------------", prop.rendered_document_text);
                    }
                }
            }
            Ok(())
        }
        MotionCmd::ExtensionSuggest { path } => {
            let profile = case_profile::load_case(&path)?;
            let assessment = open_season_lib::consultation::assess_baseline_extension(&profile, Some(&path));
            if json {
                emit(true, &assessment, vec![]);
            } else {
                println!("=== 30-Day Extension & Baseline Assessment ===");
                println!("Extension Recommended: {}", assessment.is_extension_recommended);
                println!("Urgency Level:         {}", assessment.urgency_level);
                println!("Recommended Target:    {}", assessment.recommended_target);
                println!("Governing Rule:        {}", assessment.governing_rule);
                if let Some(form) = &assessment.recommended_form_number {
                    println!("Recommended Form:      {}", form);
                }
                if !assessment.missing_elements.is_empty() {
                    println!("\nMissing Case Elements:");
                    for m in &assessment.missing_elements {
                        println!("  - {m}");
                    }
                }
                println!("\n--- Opposing Counsel Conferral Letter ---\n{}", assessment.conferral_letter_draft);
                println!("\n--- Formal Motion for Enlargement ---\n{}", assessment.formal_motion_draft);
            }
            Ok(())
        }
    }
}
