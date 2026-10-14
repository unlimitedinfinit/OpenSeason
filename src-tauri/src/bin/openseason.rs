use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use open_season_lib::agent::{AgentEnvelope, CreatedCase};
use open_season_lib::case_profile::{self, CaseMode, REVIEW_NOTICE};
use open_season_lib::check;
use open_season_lib::court_md;
use open_season_lib::documents;
use open_season_lib::rule_pack::RulePack;
use open_season_lib::sync::{self, SyncAction};

#[derive(Parser)]
#[command(
    name = "openseason",
    about = "Offline Case Builder / Document Builder. Formatting and checking only. Never files or serves.",
    long_about = "Create a portable case folder, import a Markdown draft, run record/authority/deadline checks, and export Word and PDF.\n\nHard rule: this tool only formats and checks the user's own material. Read AGENTS.md and docs/AGENT-INTERFACE.md before using it from an AI agent."
)]
struct Cli {
    /// Print machine-readable JSON (agent interface)
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Case {
        #[command(subcommand)]
        action: CaseCmd,
    },
    Draft {
        #[command(subcommand)]
        action: DraftCmd,
    },
    Evidence {
        #[command(subcommand)]
        action: EvidenceCmd,
    },
    Check {
        #[command(subcommand)]
        action: CheckCmd,
    },
    Export {
        #[command(subcommand)]
        action: ExportCmd,
    },
    Packs {
        #[command(subcommand)]
        action: PacksCmd,
    },
    Gaps {
        #[command(subcommand)]
        action: GapsCmd,
    },
    /// Print the agent playbook or field help
    Docs {
        #[command(subcommand)]
        action: DocsCmd,
    },
    /// MCP stdio server (same engine as the CLI)
    Mcp,
    Sync {
        path: PathBuf,
        #[arg(long, default_value = "backup")]
        action: String,
    },
}

#[derive(Subcommand)]
enum CaseCmd {
    Create {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "standard")]
        mode: String,
        #[arg(long, default_value = "motion")]
        kind: String,
        #[arg(long, default_value = "generic-us-appellate")]
        pack: String,
        #[arg(long, default_value = "")]
        motion_kind: String,
    },
    Validate {
        path: PathBuf,
    },
    Seal {
        path: PathBuf,
    },
    /// Copy the fake Lucille Example reconsideration case
    Seed {
        #[arg(long)]
        path: PathBuf,
        #[arg(long, default_value = "lucille")]
        template: String,
    },
}

#[derive(Subcommand)]
enum DraftCmd {
    Import {
        path: PathBuf,
        #[arg(long)]
        md: PathBuf,
    },
    Sections {
        path: PathBuf,
    },
    Fill {
        path: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        text: String,
    },
}

#[derive(Subcommand)]
enum EvidenceCmd {
    Add {
        path: PathBuf,
        #[arg(long)]
        file: PathBuf,
        #[arg(long, default_value = "")]
        description: String,
    },
    Index {
        path: PathBuf,
    },
    Link {
        path: PathBuf,
        #[arg(long)]
        fact_id: u32,
        #[arg(long)]
        letter: String,
    },
}

#[derive(Subcommand)]
enum CheckCmd {
    Fact {
        path: PathBuf,
        #[arg(long)]
        id: Option<u32>,
        #[arg(long)]
        cite: Option<String>,
        #[arg(long)]
        exhibit: Option<String>,
        #[arg(long)]
        verified: bool,
    },
    Authority {
        path: PathBuf,
        #[arg(long)]
        id: Option<u32>,
        #[arg(long, default_value = "unverified")]
        status: String,
        #[arg(long, default_value = "")]
        source: String,
        #[arg(long, default_value = "")]
        note: String,
    },
    Deadline {
        path: PathBuf,
        #[arg(long)]
        pack: String,
        #[arg(long)]
        rule: String,
        #[arg(long)]
        trigger_date: String,
    },
    Signoff {
        path: PathBuf,
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        checked: bool,
    },
    All {
        path: PathBuf,
        #[arg(long)]
        pack: Option<String>,
        #[arg(long)]
        rule: Option<String>,
        #[arg(long)]
        trigger_date: Option<String>,
    },
}

#[derive(Subcommand)]
enum ExportCmd {
    NoticeOfAppeal {
        path: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Pleading {
        path: PathBuf,
        #[arg(long, default_value = "motion")]
        kind: String,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    CourtMd {
        path: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Packet {
        path: PathBuf,
    },
    Ics {
        path: PathBuf,
        #[arg(long)]
        pack: String,
        #[arg(long)]
        rule: String,
        #[arg(long)]
        trigger_date: String,
    },
}

#[derive(Subcommand)]
enum PacksCmd {
    List,
    Validate {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum GapsCmd {
    Add {
        path: PathBuf,
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "")]
        court: String,
        #[arg(long, default_value = "")]
        rule: String,
        #[arg(long, default_value = "")]
        happened: String,
        #[arg(long, default_value = "")]
        hard: String,
    },
    Export {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum DocsCmd {
    Playbook,
    Fields,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json = cli.json;
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            if json {
                let env = AgentEnvelope::with_issues(
                    err.clone(),
                    vec![case_profile::ValidationIssue {
                        field: "cli".into(),
                        message: err,
                    }],
                );
                println!("{}", env.to_json().unwrap_or_else(|_| "{}".into()));
            } else {
                eprintln!("Error: {}", err);
            }
            ExitCode::FAILURE
        }
    }
}

fn emit<T: serde::Serialize>(json: bool, value: T, issues: Vec<case_profile::ValidationIssue>) {
    if json {
        println!(
            "{}",
            AgentEnvelope::with_issues(value, issues)
                .to_json()
                .unwrap()
        );
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let json = cli.json;
    match cli.command {
        Commands::Mcp => open_season_lib::mcp::serve_stdio(),
        Commands::Packs { action: PacksCmd::List } => {
            let packs: Vec<_> = RulePack::list_builtin()
                .into_iter()
                .map(|p| {
                    serde_json::json!({"id": p.id, "name": p.name, "jurisdiction": p.jurisdiction})
                })
                .collect();
            if json {
                emit(true, packs, vec![]);
            } else {
                for p in RulePack::list_builtin() {
                    println!("{}  {}  ({})", p.id, p.name, p.jurisdiction);
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
        Commands::Case { action } => match action {
            CaseCmd::Create {
                path,
                title,
                mode,
                kind,
                pack,
                motion_kind,
            } => {
                let mode = CaseMode::parse(&mode)?;
                let (dir, mut profile) = case_profile::create_case_folder(&path, &title, mode, &kind)?;
                profile.rule_pack_id = pack.clone();
                profile.court.id = pack.clone();
                profile.motion_kind = motion_kind;
                case_profile::save_case(&dir, &profile)?;
                let created = CreatedCase {
                    id: profile.id.clone(),
                    folder: dir.display().to_string(),
                    mode: profile.mode.as_str().into(),
                    document_kind: profile.document_kind.clone(),
                    rule_pack_id: profile.rule_pack_id.clone(),
                };
                if json {
                    emit(true, created, vec![]);
                } else {
                    println!("Created case {}", profile.id);
                    println!("Folder: {}", dir.display());
                    println!("Mode: {}", profile.mode.as_str());
                    println!("Pack: {}", profile.rule_pack_id);
                    println!("This tool never files or serves.");
                }
                Ok(())
            }
            CaseCmd::Validate { path } => {
                let profile = case_profile::load_case(&path)?;
                let report = case_profile::validate_case(&profile, Some(&path));
                if json {
                    emit(true, &report, report.issues.clone());
                    // Envelope.ok is the agent contract. Exit 0 means the command ran.
                    return Ok(());
                }
                if report.ok {
                    println!("Case is complete enough to export.");
                    return Ok(());
                }
                println!("Case is not ready ({} issue(s)):", report.issues.len());
                for issue in &report.issues {
                    println!("  - {}: {}", issue.field, issue.message);
                }
                Err("validation failed".into())
            }
            CaseCmd::Seal { path } => {
                let profile = open_season_lib::seal::seal_case_on_disk(&path)?;
                if json {
                    emit(true, profile.id, vec![]);
                } else {
                    println!("Case {} is now confidential.", profile.id);
                    println!("CLI seal does not encrypt files.");
                }
                Ok(())
            }
            CaseCmd::Seed { path, template } => {
                if template != "lucille" {
                    return Err("Unknown template. Use lucille (fake names only).".into());
                }
                let (dir, profile) = open_season_lib::seed::seed_lucille(&path)?;
                if json {
                    emit(
                        true,
                        serde_json::json!({"folder": dir, "id": profile.id, "title": profile.title}),
                        vec![],
                    );
                } else {
                    println!("Seeded fake case {} at {}", profile.title, dir.display());
                }
                Ok(())
            }
        },
        Commands::Draft { action: DraftCmd::Import { path, md } } => {
            let text = std::fs::read_to_string(&md).map_err(|e| e.to_string())?;
            let doc = court_md::import_markdown(&path, &text)?;
            if json {
                emit(true, &doc.meta, vec![]);
            } else {
                println!("Imported {} ({} lines).", md.display(), doc.lines.len());
            }
            Ok(())
        }
        Commands::Draft { action: DraftCmd::Sections { path } } => {
            let profile = case_profile::load_case(&path)?;
            let draft = open_season_lib::guided::load_or_init(&path, &profile.document_kind)?;
            if json {
                emit(true, &draft, vec![]);
            } else {
                for s in &draft.sections {
                    println!("{}. {}  empty={}", s.id, s.heading, s.user_text.trim().is_empty());
                    println!("    {}", s.hint);
                }
            }
            Ok(())
        }
        Commands::Draft {
            action: DraftCmd::Fill { path, id, text },
        } => {
            let draft = open_season_lib::guided::set_section(&path, &id, &text)?;
            if json {
                emit(true, &draft, vec![]);
            } else {
                println!("Saved section {id}.");
            }
            Ok(())
        }
        Commands::Evidence {
            action: EvidenceCmd::Add {
                path,
                file,
                description,
            },
        } => {
            let bytes = std::fs::read(&file).map_err(|e| e.to_string())?;
            let name = file
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("exhibit.bin");
            let dest = check::add_exhibit(&path, name, &bytes, &description)?;
            if json {
                emit(true, dest.display().to_string(), vec![]);
            } else {
                println!("Copied exhibit to {}", dest.display());
            }
            Ok(())
        }
        Commands::Evidence {
            action: EvidenceCmd::Index { path },
        } => {
            let index = open_season_lib::exhibits::rebuild(&path)?;
            if json {
                emit(true, &index, vec![]);
            } else {
                for row in &index.items {
                    println!("Ex. {}  {}  {}", row.letter, row.filename, row.description);
                }
            }
            Ok(())
        }
        Commands::Evidence {
            action: EvidenceCmd::Link {
                path,
                fact_id,
                letter,
            },
        } => {
            let report = open_season_lib::exhibits::link_fact(&path, fact_id, &letter)?;
            if json {
                emit(true, &report, report.issues.clone());
            } else {
                println!("Linked fact {fact_id} to Ex. {letter}.");
            }
            Ok(())
        }
        Commands::Check { action } => match action {
            CheckCmd::Fact {
                path,
                id,
                cite,
                exhibit,
                verified,
            } => {
                let draft = check::draft_text_for_case(&path)?;
                let report = if let Some(id) = id {
                    check::set_fact_cite(
                        &path,
                        id,
                        cite.as_deref().unwrap_or(""),
                        exhibit.as_deref().unwrap_or(""),
                        verified,
                    )?
                } else {
                    check::run_fact_check(&path, &draft)?
                };
                if json {
                    emit(true, &report, report.issues.clone());
                } else {
                    println!(
                        "Fact-check: {} row(s), {} issue(s). Never invents cites.",
                        report.rows.len(),
                        report.issues.len()
                    );
                    for row in &report.rows {
                        println!(
                            "  {}. [{}] {} | cite={}",
                            row.id,
                            if row.verified { "Y" } else { "N" },
                            row.assertion,
                            row.record_cite
                        );
                    }
                }
                Ok(())
            }
            CheckCmd::Authority {
                path,
                id,
                status,
                source,
                note,
            } => {
                let draft = check::draft_text_for_case(&path)?;
                let report = if let Some(id) = id {
                    let st = match status.as_str() {
                        "verified" => check::AuthorityStatus::Verified,
                        "not_found" => check::AuthorityStatus::NotFound,
                        _ => check::AuthorityStatus::Unverified,
                    };
                    check::set_authority_status(&path, id, st, &source, &note)?
                } else {
                    check::run_authority_check(&path, &draft)?
                };
                if json {
                    emit(true, &report, report.issues.clone());
                } else {
                    println!(
                        "Authority: {} cite(s). Invented: {}. Unverified cites stay tagged.",
                        report.rows.len(),
                        report.invented
                    );
                    for row in &report.rows {
                        println!("  {}. {:?}  {}", row.id, row.status, row.cite);
                    }
                }
                Ok(())
            }
            CheckCmd::Deadline {
                path,
                pack,
                rule,
                trigger_date,
            } => {
                let result = check::run_deadline(&path, &pack, &rule, &trigger_date)?;
                if json {
                    emit(true, &result, vec![]);
                } else {
                    println!("{}", result.math);
                    println!("Due: {}", result.due);
                    if let Some(t) = &result.tolls_appeal_under {
                        println!("Tolling note: {t}");
                    }
                }
                Ok(())
            }
            CheckCmd::Signoff { path, id, checked } => {
                let report = if let Some(id) = id {
                    check::set_signoff(&path, &id, checked)?
                } else {
                    check::load_or_init_signoff(&path)?
                };
                if json {
                    emit(true, &report, report.issues.clone());
                } else {
                    for item in &report.items {
                        println!(
                            "  [{}] {} ({})",
                            if item.checked { "x" } else { " " },
                            item.label,
                            item.id
                        );
                    }
                    println!("The app never files or serves.");
                }
                Ok(())
            }
            CheckCmd::All {
                path,
                pack,
                rule,
                trigger_date,
            } => {
                let report = check::run_all(
                    &path,
                    pack.as_deref(),
                    rule.as_deref(),
                    trigger_date.as_deref(),
                )?;
                if json {
                    emit(true, &report, report.issues.clone());
                } else {
                    println!("{}", report.notice);
                    println!(
                        "ok={} facts={} authority={} signoff={}",
                        report.ok, report.fact.ok, report.authority.ok, report.signoff.ok
                    );
                    for issue in &report.issues {
                        println!("  - {}: {}", issue.field, issue.message);
                    }
                }
                Ok(())
            }
        },
        Commands::Export { action } => match action {
            ExportCmd::NoticeOfAppeal { path, out } => {
                let profile = case_profile::load_case(&path)?;
                let exported = documents::export_notice_of_appeal(&path, &profile, out.as_deref())?;
                if json {
                    emit(
                        true,
                        serde_json::json!({"docx": exported.docx, "pdf": exported.pdf}),
                        vec![],
                    );
                } else {
                    println!("Wrote {}", exported.docx.display());
                    println!("Wrote {}", exported.pdf.display());
                    println!("{REVIEW_NOTICE}");
                }
                Ok(())
            }
            ExportCmd::Pleading { path, kind, out } => {
                let profile = case_profile::load_case(&path)?;
                let exported = documents::export_pleading(&path, &profile, &kind, out.as_deref())?;
                if json {
                    emit(
                        true,
                        serde_json::json!({"docx": exported.docx, "pdf": exported.pdf}),
                        vec![],
                    );
                } else {
                    println!("Wrote {}", exported.docx.display());
                    println!("Wrote {}", exported.pdf.display());
                    println!("This app did not file or serve the document.");
                }
                Ok(())
            }
            ExportCmd::CourtMd { path, out } => {
                let profile = case_profile::load_case(&path)?;
                let exported = documents::export_court_markdown(&path, &profile, out.as_deref())?;
                if json {
                    emit(
                        true,
                        serde_json::json!({"docx": exported.docx, "pdf": exported.pdf}),
                        vec![],
                    );
                } else {
                    println!("Wrote {}", exported.docx.display());
                    println!("Wrote {}", exported.pdf.display());
                }
                Ok(())
            }
            ExportCmd::Packet { path } => {
                let report = open_season_lib::packet::assemble_for_case(&path)?;
                if json {
                    emit(true, &report, vec![]);
                } else {
                    println!("{}", report.notice);
                    for item in &report.items {
                        println!("{}  {}", item.label, item.docx);
                    }
                }
                Ok(())
            }
            ExportCmd::Ics {
                path,
                pack,
                rule,
                trigger_date,
            } => {
                let result = check::run_deadline(&path, &pack, &rule, &trigger_date)?;
                let ics = open_season_lib::calendar::write_ics(&path, &result)?;
                if json {
                    emit(true, ics.display().to_string(), vec![]);
                } else {
                    println!("Wrote {}", ics.display());
                    println!("{}", result.math);
                }
                Ok(())
            }
        },
        Commands::Sync { path, action } => {
            let action = match action.as_str() {
                "backup" => SyncAction::Backup,
                "publish" => SyncAction::Publish,
                "pull" | "pull_account" => SyncAction::PullAccount,
                other => return Err(format!("Unknown action '{}'", other)),
            };
            let result = sync::describe_sync_for_dir(&path, action)?;
            let refused = result.kind == "confidential_forbidden" || result.kind == "not_implemented";
            if json {
                let issues = if refused {
                    vec![case_profile::ValidationIssue {
                        field: "sync".into(),
                        message: result.reason.clone(),
                    }]
                } else {
                    vec![]
                };
                emit(true, &result, issues);
                return Ok(());
            }
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
            if refused {
                return Err(result.reason);
            }
            Ok(())
        }
    }
}
