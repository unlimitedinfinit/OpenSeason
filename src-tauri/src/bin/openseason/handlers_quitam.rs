use open_season_lib::case_profile::load_case;
use open_season_lib::qui_tam::*;
use crate::args::QuiTamCmd;
use crate::emit;

pub fn handle_quitam(cmd: QuiTamCmd, json: bool) -> Result<(), String> {
    match cmd {
        QuiTamCmd::VerifyEntity { name, npi, state } => {
            let query = npi.as_deref().unwrap_or(&name);
            let report = verify_entity(query, state.as_deref())?;

            if !json {
                println!("=== CMS NPPES & USASPENDING ENTITY VERIFICATION ===");
                println!("Target Query: {}", report.target_query);
                if let Some(ref ent) = report.nppes_entity {
                    println!("NPI: {} ({})", ent.npi, ent.entity_type);
                    if let Some(ref org) = ent.organization_name {
                        println!("Organization Name: {}", org);
                    }
                    if let Some(ref off) = ent.authorized_official {
                        println!("Authorized Official: {} {} ({})", off.first_name, off.last_name, off.title);
                    }
                    println!("Practice Location: {}, {}, {} {}", ent.practice_location.address_1, ent.practice_location.city, ent.practice_location.state, ent.practice_location.postal_code);
                    println!("Mailing Address: {}, {}, {} {}", ent.mailing_address.address_1, ent.mailing_address.city, ent.mailing_address.state, ent.mailing_address.postal_code);
                    for tax in &ent.taxonomies {
                        println!("Taxonomy: {} - {} (Primary: {})", tax.code, tax.desc, tax.primary);
                    }
                } else {
                    println!("No CMS NPI registration matched.");
                }
                println!("\nUSAspending Federal Awards: {} awards totaling ${:.2}", report.usaspending_count, report.total_federal_obligations);
                for a in &report.awards_summary {
                    println!(" - {} [${:.2}]: {}", a.awarding_agency, a.total_obligation, a.description.as_deref().unwrap_or(""));
                }
                if !report.red_flags.is_empty() {
                    println!("\nRegulatory Red Flags:");
                    for rf in &report.red_flags {
                        println!(" ⚠️ {}", rf);
                    }
                }
            } else {
                emit(json, report, Vec::new());
            }
            Ok(())
        }

        QuiTamCmd::Audit9b { path } => {
            let profile = load_case(&path).map_err(|e| format!("Failed to load case: {e}"))?;
            let audit = audit_case_rule9b(&path, &profile)?;

            if !json {
                println!("=== 31 U.S.C. § 3729 & FRCP RULE 9(b) VIABILITY AUDIT ===");
                println!("Case: {}", profile.title);
                println!("Viability Score: {} / 100", audit.score);
                println!("Viability Tier: {}", audit.viability_tier);
                println!("Public Disclosure Risk: {}", audit.public_disclosure_risk);
                println!("\nStatutory Elements Checklist:");
                for elem in &audit.elements {
                    let status = if elem.satisfied { "[PASS]" } else { "[DEFICIENT]" };
                    println!(" {} {} ({}): {}", status, elem.title, elem.statutory_reference, elem.details);
                }
                if !audit.missing_requirements.is_empty() {
                    println!("\nMissing Rule 9(b) Elements:");
                    for miss in &audit.missing_requirements {
                        println!(" - {}", miss);
                    }
                }
                if !audit.next_investigative_steps.is_empty() {
                    println!("\nRecommended Next Investigative Steps:");
                    for step in &audit.next_investigative_steps {
                        println!(" -> {}", step);
                    }
                }
            } else {
                emit(json, audit, Vec::new());
            }
            Ok(())
        }

        QuiTamCmd::IngestObservation { path, file, notes, observer } => {
            let entry = ingest_observation(&path, &file, &notes, observer.as_deref())?;

            if !json {
                println!("=== FIELD OBSERVATION CRYPTOGRAPHICALLY INGESTED ===");
                println!("Stored: {}", entry.file_path);
                println!("SHA-256: {}", entry.sha256);
                println!("SHA-512: {:.32}...", entry.sha512);
                println!("Observed Timestamp: {}", entry.observed_at);
                if let Some(ref dev) = entry.exif_device {
                    println!("EXIF Device: {}", dev);
                }
                println!("Merkle Chain Hash: {}", entry.chain_of_custody_hash);
                println!("Notes: {}", entry.notes);
                println!("File locked read-only on disk in local vault.");
            } else {
                emit(json, entry, Vec::new());
            }
            Ok(())
        }

        QuiTamCmd::RecordsPack { path, state } => {
            let profile = load_case(&path).map_err(|e| format!("Failed to load case: {e}"))?;
            let pack = generate_records_pack(&path, &profile, state.as_deref())?;

            if !json {
                println!("=== STATE LICENSING & PUBLIC RECORDS REQUEST GENERATED ===");
                println!("Governing Statute: {}", pack.statute_title);
                println!("Target Agency: {}", pack.target_agency);
                println!("Target Entity: {} ({})", pack.target_entity_name, pack.target_entity_address);
                println!("\nFormal Request Letter Text:\n{}", pack.formal_letter_text);
            } else {
                emit(json, pack, Vec::new());
            }
            Ok(())
        }

        QuiTamCmd::ExportDisclosure { path, category, attestation } => {
            let profile = load_case(&path).map_err(|e| format!("Failed to load case: {e}"))?;
            let packet = export_doj_disclosure(&path, &profile, category.as_deref(), attestation.as_deref())?;

            if !json {
                println!("=== DOJ 60-DAY RELATOR DISCLOSURE PACKET COMPILED ===");
                println!("Case: {} (NPI: {})", packet.case_title, packet.npi);
                println!("EO 14395 Category: {}", packet.eo_category);
                println!("Rule 9(b) Viability Score: {}/100", packet.rule9b_score);
                println!("Estimated Federal Loss: ${:.2}", packet.estimated_loss);
                if let Some(ref j_path) = packet.json_manifest_path {
                    println!("JSON Disclosure Manifest: {}", j_path);
                }
                if let Some(ref p_path) = packet.pdf_export_path {
                    println!("Court & DOJ Ready PDF: {}", p_path);
                }
                println!("\nNotice: File remains under mandatory seal pursuant to 31 U.S.C. § 3730(b)(2).");
            } else {
                emit(json, packet, Vec::new());
            }
            Ok(())
        }

        QuiTamCmd::FormatPortal { path, agency } => {
            let profile = load_case(&path).map_err(|e| format!("Failed to load case: {e}"))?;
            let packet = export_doj_disclosure(&path, &profile, None, None)?;
            let portal_block = generate_portal_submission_block(&packet, &agency);

            if !json {
                println!("=== DIRECT AGENCY PORTAL AUTO-FILL BLOCK [{}] ===", portal_block.agency);
                println!("Portal URL: {}", portal_block.portal_url);
                println!("Character Count: {} chars\n", portal_block.character_count);
                println!("--- FIELD 1: SUSPECT ENTITY ---");
                println!("{}", portal_block.field_suspect_entity);
                println!("\n--- FIELD 2: PROGRAM CLASSIFICATION ---");
                println!("{}", portal_block.field_program_classification);
                println!("\n--- FIELD 3: NARRATIVE DESCRIPTION ---");
                println!("{}", portal_block.field_narrative_summary);
            } else {
                emit(json, portal_block, Vec::new());
            }
            Ok(())
        }

        QuiTamCmd::FormatInquiry { path } => {
            let profile = load_case(&path).map_err(|e| format!("Failed to load case: {e}"))?;
            let packet = export_doj_disclosure(&path, &profile, None, None)?;
            let inquiry_pkg = generate_attorney_inquiry_package(&packet, &profile);

            if !json {
                println!("=== ABA RULE 1.18 PRIVILEGED QUI TAM ATTORNEY INQUIRY ===");
                println!("Subject: {}", inquiry_pkg.subject);
                println!("\nTransmission Checklist:");
                for step in &inquiry_pkg.transmission_checklist {
                    println!("  {}", step);
                }
                println!("\n--- COMPLETE COPY-PASTE EMAIL BODY ---\n");
                println!("{}", inquiry_pkg.complete_email_body);
            } else {
                emit(json, inquiry_pkg, Vec::new());
            }
            Ok(())
        }
    }
}
