use open_season_lib::case_profile;
use open_season_lib::rule_pack::RulePack;

use super::args::{
    AgentCmd, Commands, DocsCmd, FormCmd, GapsCmd, GuideCmd, PacksCmd, PdfCmd, ServiceCmd,
};
use super::emit;

pub fn handle_tools_command(cmd: Commands, json: bool) -> Result<(), String> {
    match cmd {
        Commands::Mcp => open_season_lib::mcp::serve_stdio(),
        Commands::Capabilities => {
            let caps = open_season_lib::capabilities::list_capabilities();
            if json {
                emit(true, &caps, vec![]);
            } else {
                println!("{}", serde_json::to_string_pretty(&caps).unwrap());
            }
            Ok(())
        }
        Commands::Agent { action } => match action {
            AgentCmd::Manifest => {
                let manifest = open_season_lib::help::agent_manifest();
                if json {
                    emit(true, &manifest, vec![]);
                } else {
                    println!("{}", serde_json::to_string_pretty(&manifest).unwrap());
                }
                Ok(())
            }
            AgentCmd::Capabilities => {
                let caps = open_season_lib::capabilities::list_capabilities();
                if json {
                    emit(true, &caps, vec![]);
                } else {
                    println!("{}", serde_json::to_string_pretty(&caps).unwrap());
                }
                Ok(())
            }
        },
        Commands::Packs {
            action: PacksCmd::List { catalog },
        } => {
            let list = open_season_lib::forms::list_packs(catalog.as_deref(), None)?;
            if json {
                emit(true, &list, vec![]);
            } else {
                println!("Packs root: {}", list.packs_root);
                println!("Rule packs:");
                for p in RulePack::list_builtin() {
                    println!("  {}  {}  ({})", p.id, p.name, p.jurisdiction);
                }
                println!("Available form packs (sizes shown before download):");
                for p in &list.available {
                    println!(
                        "  {}  {}  {}  {}{}",
                        p.id,
                        p.state,
                        p.size_label,
                        p.title,
                        if p.installed { "  [installed]" } else { "" }
                    );
                }
                if list.available.is_empty() {
                    println!("  (none in catalog)");
                }
            }
            Ok(())
        }
        Commands::Packs {
            action: PacksCmd::Install {
                state,
                situation,
                catalog,
                file,
            },
        } => {
            let report = open_season_lib::forms::install(open_season_lib::forms::InstallOpts {
                state,
                situation,
                catalog,
                file,
                root: None,
                allow_network: false,
            })?;
            if json {
                emit(true, &report, vec![]);
            } else {
                println!(
                    "Installed {} ({}). Offline afterward. Never files.",
                    report.pack_id, report.size_label
                );
                println!("{}", report.dest);
            }
            Ok(())
        }
        Commands::Packs {
            action: PacksCmd::Remove { state },
        } => {
            let dest = open_season_lib::forms::remove_pack(&state, None)?;
            if json {
                emit(true, dest, vec![]);
            } else {
                println!("Removed {dest}");
            }
            Ok(())
        }
        Commands::Packs {
            action: PacksCmd::Verify { state },
        } => {
            let reports = open_season_lib::forms::verify_pack(state.as_deref(), None)?;
            let issues = reports
                .iter()
                .filter(|r| !r.ok)
                .map(|r| case_profile::ValidationIssue {
                    field: r.pack_id.clone(),
                    message: "SHA-256 check failed. Re-download the pack.".into(),
                })
                .collect();
            if json {
                emit(true, &reports, issues);
            } else {
                for r in &reports {
                    println!("{}  ok={}", r.pack_id, r.ok);
                    for row in &r.checked {
                        println!("  {}  {}", if row.ok { "ok" } else { "FAIL" }, row.path);
                    }
                }
            }
            Ok(())
        }
        Commands::Packs {
            action: PacksCmd::Catalog { url, file },
        } => {
            let src = url
                .or_else(|| file.map(|p| p.display().to_string()))
                .ok_or_else(|| "Pass --url or --file for the catalog JSON.".to_string())?;
            let dest = open_season_lib::forms::cache_remote_catalog(&src, None)?;
            if json {
                emit(true, dest.display().to_string(), vec![]);
            } else {
                println!("Cached catalog at {}", dest.display());
            }
            Ok(())
        }
        Commands::Guide {
            action: GuideCmd::Questions,
        } => {
            let q = open_season_lib::guide::questions()?;
            if json {
                emit(true, &q, vec![]);
            } else {
                println!("{}", q.disclaimer);
                for item in &q.questions {
                    println!("\n{}  {}", item.id, item.prompt);
                    for c in &item.choices {
                        println!("  - {}  {}", c.id, c.label);
                    }
                }
                println!("\nHelp links are returned as data. This command does not open a browser.");
            }
            Ok(())
        }
        Commands::Guide {
            action: GuideCmd::Find {
                state,
                court,
                situation,
                trigger_date,
            },
        } => {
            let result = open_season_lib::guide::find(
                &state,
                court.as_deref(),
                &situation,
                trigger_date.as_deref(),
            )?;
            let issues = open_season_lib::guide::issues_from(&result);
            if json {
                emit(true, &result, issues);
            } else {
                println!("{}", result.disclaimer);
                if result.stuck {
                    println!("No matching packet. The finder did not invent a form.");
                } else {
                    println!("{} — {}", result.situation_label, result.why);
                    for form in &result.forms {
                        println!("  form  {}  {}", form.id, form.title);
                        if !form.teach.what_it_is.is_empty() {
                            println!("        {}", form.teach.what_it_is);
                        }
                    }
                    if let Some(d) = &result.deadline {
                        println!("  due {}  ({})", d.due, d.rule);
                    }
                }
                for note in &result.unverified {
                    println!("  needs verification: {note}");
                }
                println!("Help links (not opened):");
                for link in &result.help_links {
                    println!("  {}  {}", link.label, link.url);
                }
            }
            Ok(())
        }
        Commands::Packs {
            action: PacksCmd::Validate { path },
        } => {
            let report = RulePack::validate_file(&path)?;
            if json {
                emit(true, &report, report.issues.clone());
            } else if report.ok {
                println!("Rule pack is valid.");
            } else {
                for issue in &report.issues {
                    println!("  - {}: {}", issue.field, issue.message);
                }
            }
            Ok(())
        }
        Commands::Gaps {
            action: GapsCmd::Add {
                path,
                title,
                court,
                rule,
                happened,
                hard,
            },
        } => {
            let log = open_season_lib::gaps::add(&path, &title, &court, &rule, &happened, &hard)?;
            if json {
                emit(true, &log, vec![]);
            } else {
                println!("Saved gap note. {} note(s).", log.notes.len());
            }
            Ok(())
        }
        Commands::Gaps {
            action: GapsCmd::Export { path },
        } => {
            let dest = open_season_lib::gaps::export_markdown(&path)?;
            if json {
                emit(true, dest.display().to_string(), vec![]);
            } else {
                println!("Wrote {}", dest.display());
            }
            Ok(())
        }
        Commands::Docs {
            action: DocsCmd::Playbook,
        } => {
            let text = open_season_lib::help::playbook();
            if json {
                emit(true, text, vec![]);
            } else {
                println!("{text}");
            }
            Ok(())
        }
        Commands::Docs {
            action: DocsCmd::Fields,
        } => {
            let fields = open_season_lib::help::field_help();
            if json {
                emit(true, fields, vec![]);
            } else {
                for f in fields {
                    println!("{}  {}", f.id, f.help);
                }
            }
            Ok(())
        }
        Commands::Docs {
            action: DocsCmd::Links,
        } => {
            let links = open_season_lib::help_links::HelpLinks::load()?;
            if json {
                emit(true, &links, vec![]);
            } else {
                println!("{}", links.confirm);
                println!("This command does not open a browser.");
                for link in &links.links {
                    println!("{}  {}  {}", link.id, link.label, link.url);
                }
            }
            Ok(())
        }
        Commands::Form { action } => match action {
            FormCmd::Inspect { pdf, case } => {
                let report = open_season_lib::forms::inspect_court_form(&pdf, case.as_deref())?;
                if json {
                    emit(true, &report, vec![]);
                } else {
                    println!("Form: {} ({})", report.form_name, report.detected_form_number.as_deref().unwrap_or("Custom"));
                    for f in &report.fields {
                        println!("  {} [{}]: {}", f.label, f.key, f.value_from_case.as_deref().unwrap_or("<empty>"));
                    }
                }
                Ok(())
            }
            FormCmd::Fill { pdf, case, out } => {
                let report = open_season_lib::forms::fill_court_form(&pdf, &case, out.as_deref(), None)?;
                if json {
                    emit(true, &report, vec![]);
                } else {
                    println!("{}", report.notice);
                    println!("Filled PDF written to: {}", report.output_pdf_path);
                    println!("JSON fields written to: {}", report.output_json_path);
                }
                Ok(())
            }
        },
        Commands::Service { action } => match action {
            ServiceCmd::Cert {
                path,
                filing,
                method,
                date,
                signer,
                recipients,
            } => {
                let profile = case_profile::load_case(&path)?;
                let recipients: Vec<open_season_lib::service::ServiceRecipient> =
                    if recipients.is_empty() {
                        profile
                            .caption
                            .appellees()
                            .into_iter()
                            .map(|p| open_season_lib::service::ServiceRecipient {
                                name: p.name.clone(),
                                role: p.role.clone(),
                                address: p.address.clone(),
                                email: p.email.clone(),
                            })
                            .collect()
                    } else {
                        recipients
                            .into_iter()
                            .map(|raw| parse_service_recipient(&raw))
                            .collect()
                    };
                let signature_name = if signer.trim().is_empty() {
                    profile.filer.signature_name.clone()
                } else {
                    signer
                };
                let params = open_season_lib::service::ServiceCertParams {
                    filing_title: filing,
                    service_date: date,
                    method,
                    recipients,
                    signature_name,
                };
                let result = open_season_lib::service::generate_certificate_of_service(&path, &profile, &params)?;
                if json {
                    emit(true, &result, vec![]);
                } else {
                    println!("{}", result.notice);
                    println!("Certificate of Service: {}", result.cert_pdf);
                }
                Ok(())
            }
            ServiceCmd::Order { path, title, relief } => {
                let profile = case_profile::load_case(&path)?;
                let result = open_season_lib::service::generate_proposed_order(&path, &profile, &title, &relief)?;
                if json {
                    emit(true, &result, vec![]);
                } else {
                    println!("{}", result.notice);
                    println!("Proposed Order: {}", result.order_pdf);
                }
                Ok(())
            }
        },
        Commands::Pdf { action } => match action {
            PdfCmd::Merge { out, files } => {
                let result = open_season_lib::pdf_tools::merge_pdfs(&files, &out)?;
                if json {
                    emit(true, &result, vec![]);
                } else {
                    println!("{}", result.message);
                }
                Ok(())
            }
            PdfCmd::Split { pdf, pages, out } => {
                let mut page_list = Vec::new();
                for part in pages.split(&[',', ' '][..]) {
                    let trimmed = part.trim();
                    if trimmed.is_empty() { continue; }
                    if let Some((s_str, e_str)) = trimmed.split_once('-') {
                        if let (Ok(s), Ok(e)) = (s_str.trim().parse::<u32>(), e_str.trim().parse::<u32>()) {
                            for p in s..=e { page_list.push(p); }
                        }
                    } else if let Ok(num) = trimmed.parse::<u32>() {
                        page_list.push(num);
                    }
                }
                let result = open_season_lib::pdf_tools::split_pdf(&pdf, &page_list, &out)?;
                if json {
                    emit(true, &result, vec![]);
                } else {
                    println!("{}", result.message);
                }
                Ok(())
            }
            PdfCmd::Rotate { pdf, degrees, out } => {
                let result = open_season_lib::pdf_tools::rotate_pdf(&pdf, degrees, None, &out)?;
                if json {
                    emit(true, &result, vec![]);
                } else {
                    println!("{}", result.message);
                }
                Ok(())
            }
            PdfCmd::Redact { pdf, words, out } => {
                let result = open_season_lib::pdf_tools::redact_pdf_text(&pdf, &words, &out)?;
                if json {
                    emit(true, &result, vec![]);
                } else {
                    println!("{}", result.message);
                }
                Ok(())
            }
            PdfCmd::ExtractText { pdf } => {
                let result = open_season_lib::pdf_tools::extract_pdf_text(&pdf)?;
                if json {
                    emit(true, &result, vec![]);
                } else {
                    println!("Extracted {} page(s) from {}:", result.page_count, result.path);
                    println!("{}", result.full_text);
                }
                Ok(())
            }
        },
        Commands::Sign { pdf, name, page, x, y, out } => {
            let in_p = &pdf;
            let out_p = out.unwrap_or_else(|| {
                in_p.with_file_name(format!("signed-{}", in_p.file_name().unwrap().to_string_lossy()))
            });
            let result = open_season_lib::signature::stamp_pdf(in_p, &name, page, x, y, &out_p)?;
            if json {
                emit(true, &result, vec![]);
            } else {
                println!("{}", result.message);
            }
            Ok(())
        }
        _ => Err("Not a tools command".into()),
    }
}

/// Parse `--recipient "Name|role|address|email"` (any suffix optional).
fn parse_service_recipient(raw: &str) -> open_season_lib::service::ServiceRecipient {
    let mut parts = raw.splitn(4, '|').map(|s| s.trim().to_string());
    open_season_lib::service::ServiceRecipient {
        name: parts.next().unwrap_or_default(),
        role: parts.next().unwrap_or_default(),
        address: parts.next().unwrap_or_default(),
        email: parts.next().unwrap_or_default(),
    }
}
