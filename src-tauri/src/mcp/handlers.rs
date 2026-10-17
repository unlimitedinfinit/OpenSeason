use serde_json::{json, Value};

use crate::case_profile;
use crate::check;
use crate::court_md;
use crate::documents;
use crate::rule_pack::RulePack;

pub fn call_tool(name: &str, args: &Value) -> Result<Value, String> {
    let s = |k: &str| args.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
    match name {
        "case_create" => {
            let path = std::path::PathBuf::from(s("path"));
            let kind = if s("kind").is_empty() { "motion".into() } else { s("kind") };
            let (dir, mut profile) = case_profile::create_case_folder(
                &path,
                &s("title"),
                case_profile::CaseMode::Standard,
                &kind,
            )?;
            if !s("pack").is_empty() {
                profile.rule_pack_id = s("pack");
                profile.court.id = s("pack");
                case_profile::save_case(&dir, &profile)?;
            }
            Ok(json!({"id": profile.id, "folder": dir, "never_files": true}))
        }
        "draft_import" => {
            let doc = court_md::import_markdown(std::path::Path::new(&s("path")), &s("markdown"))?;
            Ok(json!({"lines": doc.lines.len(), "case_no": doc.meta.case_no, "never_files": true}))
        }
        "draft_preview" => {
            let profile = if s("path").is_empty() {
                case_profile::CaseProfile::new_standard("Preview", "motion")
            } else {
                case_profile::load_case(std::path::Path::new(&s("path")))?
            };
            let preview = court_md::preview_markdown(&profile, &s("markdown"));
            Ok(serde_json::to_value(preview).unwrap())
        }
        "evidence_add" => {
            let file = s("file");
            let bytes = std::fs::read(&file).map_err(|e| e.to_string())?;
            let dest = check::add_exhibit(
                std::path::Path::new(&s("path")),
                std::path::Path::new(&file)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("exhibit.bin"),
                &bytes,
                &s("description"),
            )?;
            Ok(json!({"path": dest, "never_files": true}))
        }
        "check_fact" => {
            let draft = check::draft_text_for_case(std::path::Path::new(&s("path")))?;
            let report = check::run_fact_check(std::path::Path::new(&s("path")), &draft)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "check_authority" => {
            let draft = check::draft_text_for_case(std::path::Path::new(&s("path")))?;
            let report = check::run_authority_check(std::path::Path::new(&s("path")), &draft)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "check_deadline" => {
            let report = check::run_deadline(
                std::path::Path::new(&s("path")),
                &s("pack"),
                &s("rule"),
                &s("trigger_date"),
            )?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "check_all" => {
            let pack = s("pack");
            let rule = s("rule");
            let trigger = s("trigger_date");
            let report = check::run_all(
                std::path::Path::new(&s("path")),
                if pack.is_empty() { None } else { Some(pack.as_str()) },
                if rule.is_empty() { None } else { Some(rule.as_str()) },
                if trigger.is_empty() { None } else { Some(trigger.as_str()) },
            )?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "export_court" => {
            let path = std::path::PathBuf::from(s("path"));
            let profile = case_profile::load_case(&path)?;
            let kind = if s("kind").is_empty() { "court_md".into() } else { s("kind") };
            let exported = documents::export_pleading(&path, &profile, &kind, None)?;
            Ok(json!({"docx": exported.docx, "pdf": exported.pdf, "never_files": true}))
        }
        "playbook" => Ok(json!({"playbook": crate::help::playbook()})),
        "case_seed" => {
            let (dir, profile) = crate::seed::seed_lucille(std::path::Path::new(&s("path")))?;
            Ok(json!({"folder": dir, "id": profile.id, "title": profile.title, "never_files": true}))
        }
        "draft_sections" => {
            let profile = case_profile::load_case(std::path::Path::new(&s("path")))?;
            let draft = crate::guided::load_or_init(std::path::Path::new(&s("path")), &profile.document_kind)?;
            Ok(serde_json::to_value(draft).unwrap())
        }
        "draft_fill" => {
            let draft = crate::guided::set_section(std::path::Path::new(&s("path")), &s("id"), &s("text"))?;
            Ok(serde_json::to_value(draft).unwrap())
        }
        "evidence_index" => {
            Ok(serde_json::to_value(crate::exhibits::rebuild(std::path::Path::new(&s("path")))?).unwrap())
        }
        "evidence_link" => {
            let fact_id = args.get("fact_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            Ok(serde_json::to_value(crate::exhibits::link_fact(
                std::path::Path::new(&s("path")),
                fact_id,
                &s("letter"),
            )?)
            .unwrap())
        }
        "export_packet" => {
            Ok(serde_json::to_value(crate::packet::assemble_for_case(std::path::Path::new(&s("path")))?).unwrap())
        }
        "export_ics" => {
            let dir = std::path::PathBuf::from(s("path"));
            let result = check::run_deadline(&dir, &s("pack"), &s("rule"), &s("trigger_date"))?;
            let ics = crate::calendar::write_ics(&dir, &result)?;
            Ok(json!({"ics": ics, "due": result.due, "never_files": true}))
        }
        "packs_list" => {
            let catalog = if s("catalog").is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(s("catalog")))
            };
            Ok(serde_json::to_value(crate::forms::list_packs(catalog.as_deref(), None)?).unwrap())
        }
        "packs_validate" => {
            Ok(serde_json::to_value(RulePack::validate_file(std::path::Path::new(&s("file")))?).unwrap())
        }
        "packs_install" => {
            let file = if s("file").is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(s("file")))
            };
            let catalog = if s("catalog").is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(s("catalog")))
            };
            let situation = if s("situation").is_empty() {
                None
            } else {
                Some(s("situation"))
            };
            Ok(serde_json::to_value(crate::forms::install(crate::forms::InstallOpts {
                state: s("state"),
                situation,
                catalog,
                file,
                root: None,
                allow_network: false,
            })?)
            .unwrap())
        }
        "packs_remove" => Ok(json!({"removed": crate::forms::remove_pack(&s("state"), None)?})),
        "packs_verify" => {
            let state = if s("state").is_empty() {
                None
            } else {
                Some(s("state"))
            };
            Ok(serde_json::to_value(crate::forms::verify_pack(state.as_deref(), None)?).unwrap())
        }
        "guide_questions" => Ok(serde_json::to_value(crate::guide::questions()?).unwrap()),
        "guide_find" => {
            let court = if s("court").is_empty() {
                None
            } else {
                Some(s("court"))
            };
            let trigger = if s("trigger_date").is_empty() {
                None
            } else {
                Some(s("trigger_date"))
            };
            Ok(serde_json::to_value(crate::guide::find(&s("state"), court.as_deref(), &s("situation"), trigger.as_deref())?).unwrap())
        }
        "help_links" => Ok(serde_json::to_value(crate::help_links::HelpLinks::load()?).unwrap()),
        "gaps_add" => Ok(serde_json::to_value(crate::gaps::add(
            std::path::Path::new(&s("path")),
            &s("title"),
            &s("court"),
            &s("rule"),
            &s("happened"),
            &s("hard"),
        )?)
        .unwrap()),
        "check_preflight" => {
            let p_str = s("path");
            let report = check::preflight_audit(std::path::Path::new(&p_str))?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "check_transcripts" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let draft = profile.notice_of_appeal.user_text.as_str();
            let report = check::check_transcripts(dir, draft)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "export_exhibit_packet" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let p_prefix = s("prefix");
            let prefix = if p_prefix.is_empty() { None } else { Some(p_prefix.as_str()) };
            let start = args.get("start").and_then(|v| v.as_u64()).map(|n| n as u32);
            let report = crate::packet::assemble_exhibit_packet(dir, prefix, start)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "form_inspect" => {
            let pdf = std::path::PathBuf::from(s("pdf"));
            let case = if s("path").is_empty() { None } else { Some(std::path::PathBuf::from(s("path"))) };
            let report = crate::forms::inspect_court_form(&pdf, case.as_deref())?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "form_fill" => {
            let pdf = std::path::PathBuf::from(s("pdf"));
            let case = std::path::PathBuf::from(s("path"));
            let report = crate::forms::fill_court_form(&pdf, &case, None, None)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "cloud_pull" => {
            let session = crate::sync::load_cloud_session().ok_or_else(|| "Not logged in to JustLegal cloud. Run cloud login first.".to_string())?;
            let root = if s("root").is_empty() { case_profile::default_cases_root() } else { std::path::PathBuf::from(s("root")) };
            let case_filter = if s("case").is_empty() { None } else { Some(s("case")) };
            let report = crate::sync::cloud_pull_all(&session, &root, case_filter.as_deref())?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "cloud_push" => {
            let session = crate::sync::load_cloud_session().ok_or_else(|| "Not logged in to JustLegal cloud. Run cloud login first.".to_string())?;
            let dir = std::path::PathBuf::from(s("path"));
            let report = crate::sync::cloud_push_case(&dir, &session)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "motion_list" => {
            let p_str = s("path");
            let profile = case_profile::load_case(std::path::Path::new(&p_str))?;
            Ok(serde_json::to_value(profile.filings).unwrap())
        }
        "motion_create" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let mut profile = case_profile::load_case(dir)?;
            let now = chrono::Local::now().to_rfc3339();
            let filing = case_profile::CaseFilingItem {
                id: uuid::Uuid::new_v4().to_string(),
                title: s("title"),
                kind: if s("kind").is_empty() { "motion".into() } else { s("kind") },
                rule: s("rule"),
                status: "draft".into(),
                user_text: s("text"),
                relief_sought: String::new(),
                created_at: now.clone(),
                updated_at: now,
            };
            profile.filings.push(filing.clone());
            case_profile::save_case(dir, &profile)?;
            Ok(serde_json::to_value(filing).unwrap())
        }
        "motion_export" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let id = s("id");
            let filing = profile.filings.iter().find(|f| f.id == id)
                .ok_or_else(|| format!("Filing ID '{id}' not found in case."))?;
            let paths = crate::documents::export_custom_filing(dir, &profile, &filing.title, &filing.user_text, None)?;
            Ok(serde_json::to_value(paths).unwrap())
        }
        "service_cert" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let recipients: Vec<crate::service::ServiceRecipient> = profile
                .caption
                .appellees()
                .into_iter()
                .map(|p| crate::service::ServiceRecipient {
                    name: p.name.clone(),
                    role: p.role.clone(),
                    address: p.address.clone(),
                    email: p.email.clone(),
                })
                .collect();
            let filing_t = if s("filing").is_empty() { "the accompanying papers".into() } else { s("filing") };
            let method = if s("method").is_empty() { "eCourts / Electronic Filing System".into() } else { s("method") };
            let params = crate::service::ServiceCertParams {
                filing_title: filing_t,
                service_date: s("date"),
                method,
                recipients,
                signature_name: profile.filer.signature_name.clone(),
            };
            let result = crate::service::generate_certificate_of_service(dir, &profile, &params)?;
            Ok(serde_json::to_value(result).unwrap())
        }
        "service_order" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let title = if s("title").is_empty() { "RELIEF REQUESTED".into() } else { s("title") };
            let result = crate::service::generate_proposed_order(dir, &profile, &title, &s("relief"))?;
            Ok(serde_json::to_value(result).unwrap())
        }
        "pdf_merge" => {
            let files_arr = args.get("files").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let paths: Vec<std::path::PathBuf> = files_arr.iter().filter_map(|v| v.as_str()).map(std::path::PathBuf::from).collect();
            let out = std::path::PathBuf::from(s("out"));
            let result = crate::pdf_tools::merge_pdfs(&paths, &out)?;
            Ok(serde_json::to_value(result).unwrap())
        }
        "pdf_split" => {
            let pdf = std::path::PathBuf::from(s("pdf"));
            let out = std::path::PathBuf::from(s("out"));
            let pages_str = s("pages");
            let mut page_list = Vec::new();
            for part in pages_str.split(&[',', ' '][..]) {
                let trimmed = part.trim();
                if trimmed.is_empty() { continue; }
                if let Some((s_str, e_str)) = trimmed.split_once('-') {
                    if let (Ok(start), Ok(end)) = (s_str.trim().parse::<u32>(), e_str.trim().parse::<u32>()) {
                        for p in start..=end { page_list.push(p); }
                    }
                } else if let Ok(num) = trimmed.parse::<u32>() {
                    page_list.push(num);
                }
            }
            let result = crate::pdf_tools::split_pdf(&pdf, &page_list, &out)?;
            Ok(serde_json::to_value(result).unwrap())
        }
        "pdf_rotate" => {
            let pdf = std::path::PathBuf::from(s("pdf"));
            let out = std::path::PathBuf::from(s("out"));
            let degrees = args.get("degrees").and_then(|v| v.as_i64()).unwrap_or(90);
            let result = crate::pdf_tools::rotate_pdf(&pdf, degrees, None, &out)?;
            Ok(serde_json::to_value(result).unwrap())
        }
        "pdf_redact" => {
            let pdf = std::path::PathBuf::from(s("pdf"));
            let out = std::path::PathBuf::from(s("out"));
            let words_arr = args.get("words").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let words: Vec<String> = words_arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
            let result = crate::pdf_tools::redact_pdf_text(&pdf, &words, &out)?;
            Ok(serde_json::to_value(result).unwrap())
        }
        "sign_pdf" => {
            let pdf = std::path::PathBuf::from(s("pdf"));
            let name = s("name");
            let page = args.get("page").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let x = args.get("x").and_then(|v| v.as_f64()).unwrap_or(72.0) as f32;
            let y = args.get("y").and_then(|v| v.as_f64()).unwrap_or(100.0) as f32;
            let out_str = s("out");
            let out_p = if out_str.is_empty() {
                pdf.with_file_name(format!("signed-{}", pdf.file_name().unwrap().to_string_lossy()))
            } else {
                std::path::PathBuf::from(out_str)
            };
            let result = crate::signature::stamp_pdf(&pdf, &name, page, x, y, &out_p)?;
            Ok(serde_json::to_value(result).unwrap())
        }
        "case_context" => {
            let dir = std::path::PathBuf::from(s("path"));
            Ok(serde_json::to_value(crate::intake::case_context(&dir)?).unwrap())
        }
        "case_next_steps" => {
            let dir = std::path::PathBuf::from(s("path"));
            let profile = case_profile::load_case(&dir)?;
            let report = crate::legal_os::predict_next_steps(&profile, Some(&dir));
            Ok(serde_json::to_value(report).unwrap())
        }
        "case_escalate" => {
            let dir = std::path::PathBuf::from(s("path"));
            let report = crate::portfolio::escalate_case(&dir, &s("target"), None)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "intake_status" => {
            let dir = std::path::PathBuf::from(s("path"));
            let report = crate::intake::intake_status(&dir)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "intake_answer" => {
            let dir = std::path::PathBuf::from(s("path"));
            let report = crate::intake::intake_answer(&dir, &s("question_id"), &s("answer"))?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "intake_scan" => {
            let dir = std::path::PathBuf::from(s("path"));
            Ok(serde_json::to_value(crate::intake::intake_scan(&dir)).unwrap())
        }
        "check_venue" => {
            let dir = std::path::PathBuf::from(s("path"));
            let profile = case_profile::load_case(&dir)?;
            Ok(serde_json::to_value(crate::legal_os::check_venue_competence(&profile)).unwrap())
        }
        "portfolio_status" => {
            let root = if s("root").is_empty() {
                case_profile::default_cases_root()
            } else {
                std::path::PathBuf::from(s("root"))
            };
            Ok(serde_json::to_value(crate::portfolio::portfolio_status(&root)?).unwrap())
        }
        "record_inventory" => {
            let dir = std::path::PathBuf::from(s("path"));
            let profile = case_profile::load_case(&dir)?;
            let report = crate::record::inventory::scan_case_record(&dir, &profile)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "record_reconcile" => {
            let dir = std::path::PathBuf::from(s("path"));
            let profile = case_profile::load_case(&dir)?;
            let entries = crate::record::reconciliation::parse_text_docket(&s("docket"));
            let report = crate::record::reconciliation::reconcile_docket(&dir, &profile, &entries)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "record_missing" => {
            let dir = std::path::PathBuf::from(s("path"));
            let inv = crate::record::inventory::load_record_inventory(&dir);
            Ok(serde_json::to_value(inv.map(|r| r.missing_records).unwrap_or_default()).unwrap())
        }
        "record_request_form" => {
            let dir = std::path::PathBuf::from(s("path"));
            let profile = case_profile::load_case(&dir)?;
            let state = if s("state").is_empty() { None } else { Some(s("state")) };
            let packet = crate::record::requests::generate_records_request(&dir, &profile, state.as_deref())?;
            Ok(serde_json::to_value(packet).unwrap())
        }
        "accountability_audit" => {
            let dir = std::path::PathBuf::from(s("path"));
            let profile = case_profile::load_case(&dir)?;
            let report = crate::accountability::audit::audit_record_discrepancies(&dir, &profile)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "accountability_routes" => {
            let dir = std::path::PathBuf::from(s("path"));
            let profile = case_profile::load_case(&dir)?;
            let report = crate::accountability::audit::audit_record_discrepancies(&dir, &profile)?;
            let target_id = s("issue_id");
            if !target_id.is_empty() {
                if let Some(issue) = report.issues.into_iter().find(|i| i.id == target_id) {
                    return Ok(serde_json::to_value(issue.available_routes).unwrap());
                }
            }
            Ok(serde_json::to_value(crate::accountability::routes::determine_routes_for_issue(
                &profile,
                crate::accountability::types::RecordSupportStatus::UnsupportedInRecord,
                false,
            )).unwrap())
        }
        "accountability_resources" => {
            let state = if s("state").is_empty() { None } else { Some(s("state")) };
            let cat = if s("category").is_empty() { None } else { Some(s("category")) };
            let res = crate::accountability::resources::list_accountability_resources(state.as_deref(), cat.as_deref());
            Ok(serde_json::to_value(res).unwrap())
        }
        "accountability_packet" => {
            let dir = std::path::PathBuf::from(s("path"));
            let profile = case_profile::load_case(&dir)?;
            let report = crate::accountability::audit::audit_record_discrepancies(&dir, &profile)?;
            let issue_id = s("issue_id");
            let issue = report.issues.into_iter().find(|i| i.id == issue_id)
                .ok_or_else(|| format!("Issue '{issue_id}' not found in case."))?;
            let packet = crate::accountability::packets::assemble_factual_packet(&dir, &profile, &issue)?;
            Ok(serde_json::to_value(packet).unwrap())
        }
        "ada_petition" => {
            let dir = std::path::PathBuf::from(s("path"));
            let profile = case_profile::load_case(&dir)?;
            let accomm_arr = args.get("accommodations").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let mut accommodations = Vec::new();
            for item in accomm_arr {
                if let Some(str_val) = item.as_str() {
                    let cat = match str_val {
                        "remote" | "remote_telephonic_appearance" => crate::ada::AccommodationCategory::RemoteTelephonicAppearance,
                        "mobility" | "physical_mobility_access" => crate::ada::AccommodationCategory::PhysicalMobilityAccess,
                        "listening" | "visual_hearing_assistance" => crate::ada::AccommodationCategory::VisualHearingAssistance,
                        "reading_time" | "extended_reading_time" => crate::ada::AccommodationCategory::ExtendedReadingTime,
                        "support_person" | "support_person_accompaniment" => crate::ada::AccommodationCategory::SupportPersonAccompaniment,
                        "magnification" | "document_magnification_accessible_pdf" => crate::ada::AccommodationCategory::DocumentMagnificationAccessiblePdf,
                        "interpreter" | "communication_interpreter" => crate::ada::AccommodationCategory::CommunicationInterpreter,
                        _ => continue,
                    };
                    accommodations.push(cat);
                }
            }
            if accommodations.is_empty() {
                accommodations.push(crate::ada::AccommodationCategory::RemoteTelephonicAppearance);
            }
            let payload = crate::ada::AdaRequestPayload {
                case_id: profile.id.clone(),
                requester_name: s("name"),
                requester_role: if s("role").is_empty() { "Self-Represented Litigant".into() } else { s("role") },
                contact_phone: s("phone"),
                contact_email: s("email"),
                accommodations,
                specific_needs_narrative: s("needs"),
                upcoming_date: if s("date").is_empty() { None } else { Some(s("date")) },
            };
            let report = crate::ada::generator::generate_ada_petition(&profile, &payload)?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "pdf_extract_text" => {
            let pdf = std::path::PathBuf::from(s("pdf"));
            Ok(serde_json::to_value(crate::pdf_tools::extract_pdf_text(&pdf)?).unwrap())
        }
        "agent_manifest" => Ok(crate::help::agent_manifest()),
        "import_folder" => {
            let dest = s("path");
            let from = s("from");
            let title = s("title");
            let report = crate::folder_import::import_folder(
                std::path::Path::new(&dest),
                std::path::Path::new(&from),
                if title.is_empty() { None } else { Some(title.as_str()) },
            )?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "order_decode" => {
            let pdf = std::path::PathBuf::from(s("pdf"));
            let p_str = s("path");
            if !p_str.is_empty() {
                let report = crate::order::decoder::ingest_order_into_case(std::path::Path::new(&p_str), &pdf)?;
                Ok(serde_json::to_value(report).unwrap())
            } else {
                let report = crate::order::decoder::decode_order_pdf(&pdf)?;
                Ok(serde_json::to_value(report).unwrap())
            }
        }
        "timeline_audit_mail" => {
            let p_str = s("path");
            let profile = case_profile::load_case(std::path::Path::new(&p_str))?;
            let state_str = s("state");
            let state = if state_str.is_empty() {
                if !profile.governing_state.is_empty() {
                    Some(profile.governing_state.as_str())
                } else {
                    Some("CO")
                }
            } else {
                Some(state_str.as_str())
            };
            let p_mark = s("postmark");
            let p_del = s("delivery");
            let p_clk = s("clerk_date");
            let report = crate::order::mail_audit::audit_mail_gap(
                &profile.id,
                &s("order_date"),
                if p_mark.is_empty() { None } else { Some(&p_mark) },
                if p_del.is_empty() { None } else { Some(&p_del) },
                if p_clk.is_empty() { None } else { Some(&p_clk) },
                state,
            );
            Ok(serde_json::to_value(report).unwrap())
        }
        "case_bootstrap" => {
            let from = std::path::PathBuf::from(s("from"));
            let to = std::path::PathBuf::from(s("to"));
            let title = s("title");
            let report = crate::order::bootstrap::bootstrap_case_from_folder(
                &from,
                &to,
                if title.is_empty() { None } else { Some(&title) },
            )?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "court_access_directory" => {
            let profiles = crate::court_access::get_all_court_profiles();
            Ok(serde_json::to_value(profiles).unwrap())
        }
        "court_access_profile" => {
            let profile = crate::court_access::find_court_access_profile(&s("query"));
            Ok(serde_json::to_value(profile).unwrap())
        }
        "record_retrieval_plan" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let plan = crate::court_access::plan_record_retrieval(dir, &profile);
            Ok(serde_json::to_value(plan).unwrap())
        }
        "record_retrieval_execute" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let mut profile = case_profile::load_case(dir)?;
            let state_str = s("state");
            let state = if state_str.is_empty() { None } else { Some(state_str.as_str()) };
            let packet = crate::court_access::execute_record_retrieval_request(dir, &mut profile, state)?;
            Ok(serde_json::to_value(packet).unwrap())
        }
        "workflow_plan" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let plan = crate::workflow::plan_workflow(dir, &profile, &s("objective"));
            crate::workflow::save_workflow_plan(dir, &plan)?;
            Ok(serde_json::to_value(plan).unwrap())
        }
        "workflow_status" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let plan = crate::workflow::load_workflow_plan(dir);
            Ok(serde_json::to_value(plan).unwrap())
        }
        "workflow_advance" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let mut profile = case_profile::load_case(dir)?;
            let mut plan = crate::workflow::load_workflow_plan(dir)
                .ok_or_else(|| "No active workflow plan found for this case. Formulate a plan first.".to_string())?;
            let approve = args.get("approve").and_then(|v| v.as_bool()).unwrap_or(false);
            let res = crate::workflow::advance_workflow_step(dir, &mut profile, &mut plan, approve)?;
            Ok(serde_json::to_value(res).unwrap())
        }
        "issues_stress_test" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let issue_id = s("issue_id");
            let results = if !issue_id.is_empty() {
                if let Some(target_issue) = profile.issues.iter().find(|i| i.id == issue_id) {
                    vec![crate::consultation::stress_test_issue(&profile, target_issue)]
                } else {
                    return Err(format!("Issue with id '{issue_id}' not found in case."));
                }
            } else {
                crate::consultation::stress_test_all_issues(&profile)
            };
            Ok(serde_json::to_value(results).unwrap())
        }
        "consultation_prepare" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let questions: Vec<String> = args.get("questions")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|str| str.to_string())).collect())
                .unwrap_or_default();
            let dossier = crate::consultation::generate_consultation_dossier(dir, &profile, &questions)?;
            Ok(serde_json::to_value(dossier).unwrap())
        }
        "extension_evaluate" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let target = s("target");
            let days = args.get("days").and_then(|v| v.as_u64()).unwrap_or(30) as u32;
            let consent = args.get("consent").and_then(|v| v.as_bool()).unwrap_or(false);
            let prop = crate::consultation::evaluate_extension_request(&profile, &target, days, consent);
            Ok(serde_json::to_value(prop).unwrap())
        }
        "extension_suggest" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let assessment = crate::consultation::assess_baseline_extension(&profile, Some(dir));
            Ok(serde_json::to_value(assessment).unwrap())
        }
        "qui_tam_verify_entity" => {
            let query = s("query");
            let state = if s("state").is_empty() { None } else { Some(s("state")) };
            let report = crate::qui_tam::verify_entity(&query, state.as_deref())?;
            Ok(serde_json::to_value(report).unwrap())
        }
        "qui_tam_audit_9b" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let audit = crate::qui_tam::audit_case_rule9b(dir, &profile)?;
            Ok(serde_json::to_value(audit).unwrap())
        }
        "qui_tam_ingest_observation" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let file_str = s("file");
            let file = std::path::Path::new(&file_str);
            let notes = s("notes");
            let observer = if s("observer").is_empty() { None } else { Some(s("observer")) };
            let entry = crate::qui_tam::ingest_observation(dir, file, &notes, observer.as_deref())?;
            Ok(serde_json::to_value(entry).unwrap())
        }
        "qui_tam_records_pack" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let state = if s("state").is_empty() { None } else { Some(s("state")) };
            let pack = crate::qui_tam::generate_records_pack(dir, &profile, state.as_deref())?;
            Ok(serde_json::to_value(pack).unwrap())
        }
        "qui_tam_export_disclosure" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let category = if s("category").is_empty() { None } else { Some(s("category")) };
            let attestation = if s("attestation").is_empty() { None } else { Some(s("attestation")) };
            let pkg = crate::qui_tam::export_doj_disclosure(dir, &profile, category.as_deref(), attestation.as_deref())?;
            Ok(serde_json::to_value(pkg).unwrap())
        }
        "qui_tam_format_portal" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let agency = if s("agency").is_empty() { "HHS".into() } else { s("agency") };
            let packet = crate::qui_tam::export_doj_disclosure(dir, &profile, None, None)?;
            let block = crate::qui_tam::generate_portal_submission_block(&packet, &agency);
            Ok(serde_json::to_value(block).unwrap())
        }
        "qui_tam_format_inquiry" => {
            let p_str = s("path");
            let dir = std::path::Path::new(&p_str);
            let profile = case_profile::load_case(dir)?;
            let packet = crate::qui_tam::export_doj_disclosure(dir, &profile, None, None)?;
            let pkg = crate::qui_tam::generate_attorney_inquiry_package(&packet, &profile);
            Ok(serde_json::to_value(pkg).unwrap())
        }
        other => Err(format!(
            "Unknown tool '{other}'. Call playbook or packs_list. Known tools are listed by tools/list."
        )),
    }
}

