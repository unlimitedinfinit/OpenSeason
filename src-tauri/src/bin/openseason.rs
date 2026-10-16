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
    /// What do I need to file? Plain-language finder. Legal information, not advice.
    Guide {
        #[command(subcommand)]
        action: GuideCmd,
    },
    Gaps {
        #[command(subcommand)]
        action: GapsCmd,
    },
    /// Print the agent playbook, field help, or JustLegal resource links (data only)
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
    /// JustLegal Cloud sync and authentication
    Cloud {
        #[command(subcommand)]
        action: CloudCmd,
    },
    /// Official court form AcroForm inspection and auto-filling
    Form {
        #[command(subcommand)]
        action: FormCmd,
    },
    /// Manage multiple motions and filings in the case docket
    Motion {
        #[command(subcommand)]
        action: MotionCmd,
    },
    /// Generate court Certificate of Service and Proposed Form of Order
    Service {
        #[command(subcommand)]
        action: ServiceCmd,
    },
    /// Offline local PDF power tools (merge, split, rotate, redact)
    Pdf {
        #[command(subcommand)]
        action: PdfCmd,
    },
    /// Stamp court-compliant electronic signature block onto a PDF
    Sign {
        pdf: PathBuf,
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "0")]
        page: u32,
        #[arg(long, default_value = "72")]
        x: f32,
        #[arg(long, default_value = "100")]
        y: f32,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Guided 1-question-at-a-time Intake Wizard & Offline PDF Ingestion Scanner
    Intake {
        #[command(subcommand)]
        action: IntakeCmd,
    },
    /// Multi-state Portfolio Deadline Radar & Global Filer Profile
    Portfolio {
        #[command(subcommand)]
        action: PortfolioCmd,
    },
}

#[derive(Subcommand)]
enum IntakeCmd {
    Status {
        path: PathBuf,
    },
    Answer {
        path: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        answer: String,
    },
    Scan {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum PortfolioCmd {
    Status {
        #[arg(long)]
        root: Option<PathBuf>,
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
    /// Consolidated case context bundle (profile + venue + next-steps + intake + exhibits)
    Context {
        path: PathBuf,
    },
    /// Predict downstream procedural branches and deadlines from timeline & orders
    NextSteps {
        path: PathBuf,
    },
    /// Graduate Tier 2 -> Tier 3 Lawsuit or Escalate Trial -> Appeal / Mandamus / SCOTUS
    Escalate {
        path: PathBuf,
        #[arg(long, default_value = "appellate")]
        to: String,
        #[arg(long)]
        out: Option<PathBuf>,
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
    Preflight {
        path: PathBuf,
    },
    Transcripts {
        path: PathBuf,
    },
    /// Check venue-specific and tier-specific competence rules (NJ R. 1:4-4b, CO Rule 121, 28 USC 1746, SCOTUS, Wills/Leases)
    Venue {
        path: PathBuf,
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
    ExhibitPacket {
        path: PathBuf,
        #[arg(long)]
        prefix: Option<String>,
        #[arg(long)]
        start: Option<u32>,
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
enum CloudCmd {
    Login {
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    Status,
    Logout,
    Pull {
        #[arg(long)]
        case: Option<String>,
        #[arg(long)]
        root: Option<PathBuf>,
    },
    Push {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum FormCmd {
    Inspect {
        pdf: PathBuf,
        #[arg(long)]
        case: Option<PathBuf>,
    },
    Fill {
        pdf: PathBuf,
        #[arg(long)]
        case: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum MotionCmd {
    List {
        path: PathBuf,
    },
    Create {
        path: PathBuf,
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "motion")]
        kind: String,
        #[arg(long, default_value = "")]
        rule: String,
        #[arg(long, default_value = "")]
        text: String,
    },
    Export {
        path: PathBuf,
        #[arg(long)]
        id: String,
    },
}

#[derive(Subcommand)]
enum ServiceCmd {
    Cert {
        path: PathBuf,
        #[arg(long, default_value = "the accompanying papers")]
        filing: String,
        #[arg(long, default_value = "eCourts / Electronic Filing System")]
        method: String,
        #[arg(long, default_value = "")]
        date: String,
    },
    Order {
        path: PathBuf,
        #[arg(long, default_value = "RELIEF REQUESTED")]
        title: String,
        #[arg(long, default_value = "")]
        relief: String,
    },
}

#[derive(Subcommand)]
enum PdfCmd {
    Merge {
        #[arg(long)]
        out: PathBuf,
        files: Vec<PathBuf>,
    },
    Split {
        #[arg(long)]
        pdf: PathBuf,
        #[arg(long)]
        pages: String,
        #[arg(long)]
        out: PathBuf,
    },
    Rotate {
        #[arg(long)]
        pdf: PathBuf,
        #[arg(long, default_value = "90")]
        degrees: i64,
        #[arg(long)]
        out: PathBuf,
    },
    Redact {
        #[arg(long)]
        pdf: PathBuf,
        #[arg(long)]
        words: Vec<String>,
        #[arg(long)]
        out: PathBuf,
    },
    /// Extract text offline from a PDF file using lopdf
    ExtractText {
        pdf: PathBuf,
    },
}

#[derive(Subcommand)]
enum PacksCmd {
    List {
        #[arg(long)]
        catalog: Option<PathBuf>,
    },
    Install {
        state: String,
        #[arg(long)]
        situation: Option<String>,
        #[arg(long)]
        catalog: Option<PathBuf>,
        #[arg(long)]
        file: Option<PathBuf>,
    },
    Remove {
        state: String,
    },
    Verify {
        state: Option<String>,
    },
    Validate {
        path: PathBuf,
    },
    Catalog {
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        file: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum GuideCmd {
    Questions,
    Find {
        #[arg(long)]
        state: String,
        #[arg(long)]
        court: Option<String>,
        #[arg(long)]
        situation: String,
        #[arg(long)]
        trigger_date: Option<String>,
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
    /// JustLegal help/resource URLs as data. Never opens a browser.
    Links,
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
            CaseCmd::Context { path } => {
                let bundle = open_season_lib::intake::case_context(&path)?;
                if json {
                    emit(true, &bundle, bundle.venue_competence.issues.clone());
                } else {
                    println!("Case: {} ({}) [Tier: {}, Venue: {}]", bundle.profile.title, bundle.profile.docket_number, bundle.intake.matter_tier, bundle.intake.venue_code);
                    println!("Intake Completion: {}% ({}/{})", bundle.intake.completion_percent, bundle.intake.completed_steps, bundle.intake.total_steps);
                    if let Some(nq) = &bundle.intake.next_question {
                        println!("Next Question [{}]: {}", nq.id, nq.prompt);
                    }
                    println!("Procedural Branches ({}):", bundle.next_steps.branches.len());
                    for b in &bundle.next_steps.branches {
                        println!("  - {} [{}] due={:?}", b.title, b.status, b.computed_deadline);
                    }
                }
                Ok(())
            }
            CaseCmd::NextSteps { path } => {
                let profile = case_profile::load_case(&path)?;
                let report = open_season_lib::legal_os::predict_next_steps(&profile, Some(&path));
                if json {
                    emit(true, &report, vec![]);
                } else {
                    println!("Predictive Procedural Graph for '{}' (Venue: {}, Tier: {}):", report.case_title, report.venue_code, report.matter_tier);
                    for b in &report.branches {
                        println!("  [{}] {} ({}) -> due: {}", b.status, b.title, b.rule_citation, b.computed_deadline.as_deref().unwrap_or("N/A"));
                        println!("      {}", b.explanation);
                    }
                }
                Ok(())
            }
            CaseCmd::Escalate { path, to, out } => {
                let res = open_season_lib::portfolio::escalate_case(&path, &to, out.as_deref())?;
                if json {
                    emit(true, &res, vec![]);
                } else {
                    println!("{}", res.message);
                    println!("New case folder: {}", res.new_case_dir);
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
            CheckCmd::Preflight { path } => {
                let report = check::preflight_audit(&path)?;
                if json {
                    emit(true, &report, vec![]);
                } else {
                    println!("Court Readiness Pre-Flight Audit: {}", if report.ok { "READY" } else { "NOT READY" });
                    println!("  Caption: {}", report.caption_summary);
                    println!("  Docket: {}", report.docket_summary);
                    println!("  Filer: {}", report.filer_summary);
                    println!("  Deadline: {}", report.deadline_summary);
                    println!("  Facts: {}/{} verified", report.facts_verified_count, report.facts_total_count);
                    println!("  Authorities: {}/{} verified", report.authorities_verified_count, report.authorities_total_count);
                    if !report.missing_requirements.is_empty() {
                        println!("Missing requirements:");
                        for m in &report.missing_requirements {
                            println!("  - {}", m);
                        }
                    }
                }
                Ok(())
            }
            CheckCmd::Transcripts { path } => {
                let profile = case_profile::load_case(&path)?;
                let draft = profile.notice_of_appeal.user_text.as_str();
                let report = check::check_transcripts(&path, draft)?;
                if json {
                    emit(true, &report, report.issues.clone());
                } else {
                    println!("Transcripts: {} indexed, {} total lines", report.indexed_files.len(), report.total_lines);
                    println!("Citations found: {}", report.citations.len());
                    for c in &report.citations {
                        println!("  {} -> verified: {} ({})", c.raw, c.verified, c.quote_found.as_deref().unwrap_or("no match"));
                    }
                }
                Ok(())
            }
            CheckCmd::Venue { path } => {
                let profile = case_profile::load_case(&path)?;
                let report = open_season_lib::legal_os::check_venue_competence(&profile);
                if json {
                    emit(true, &report, report.issues.clone());
                } else {
                    println!("Venue Competence ({} — {}): ok={}", report.venue.venue_code, report.venue.display_name, report.ok);
                    println!("Mandatory Sworn Statement: {} ({})", report.venue.sworn_statement_title, report.venue.sworn_statement_citation);
                    for issue in &report.issues {
                        println!("  - [ISSUE] {}: {}", issue.field, issue.message);
                    }
                    for rec in &report.recommendations {
                        println!("  - [NOTE] {}", rec);
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
            ExportCmd::ExhibitPacket { path, prefix, start } => {
                let report = open_season_lib::packet::assemble_exhibit_packet(&path, prefix.as_deref(), start)?;
                if json {
                    emit(true, &report, vec![]);
                } else {
                    println!("{}", report.notice);
                    println!("Table of Exhibits: {}", report.table_of_exhibits_pdf);
                    println!("Exhibit Packet: {}", report.exhibit_packet_pdf);
                    println!("Bates Range: {} - {} ({} exhibits, {} pages)", report.bates_start, report.bates_end, report.total_exhibits, report.total_pages);
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
            let refused = result.kind == "confidential_forbidden"
                || result.kind == "not_implemented"
                || result.kind == "not_configured";
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
        Commands::Cloud { action } => match action {
            CloudCmd::Login { email, password } => {
                let session = sync::cloud_sign_in_password(&email, &password)?;
                if json {
                    emit(true, serde_json::json!({"email": session.email, "uid": session.uid, "expires_at": session.expires_at}), vec![]);
                } else {
                    println!("Logged in successfully to JustLegal Cloud as {}", session.email);
                }
                Ok(())
            }
            CloudCmd::Status => {
                let session = sync::load_cloud_session();
                if json {
                    emit(true, &session, vec![]);
                } else {
                    match session {
                        Some(s) => println!("Logged in as {} (UID: {})", s.email, s.uid),
                        None => println!("Not logged in to JustLegal Cloud."),
                    }
                }
                Ok(())
            }
            CloudCmd::Logout => {
                sync::clear_cloud_session()?;
                if json {
                    emit(true, serde_json::json!({"logged_out": true}), vec![]);
                } else {
                    println!("Logged out of JustLegal Cloud.");
                }
                Ok(())
            }
            CloudCmd::Pull { case, root } => {
                let session = sync::load_cloud_session().ok_or_else(|| "Not logged in to JustLegal Cloud. Run 'openseason cloud login' first.".to_string())?;
                let cases_root = root.unwrap_or_else(case_profile::default_cases_root);
                let report = sync::cloud_pull_all(&session, &cases_root, case.as_deref())?;
                if json {
                    emit(true, &report, vec![]);
                } else {
                    println!("Pulled {} case(s), downloaded {} file(s)", report.cases_pulled, report.files_downloaded);
                }
                Ok(())
            }
            CloudCmd::Push { path } => {
                let session = sync::load_cloud_session().ok_or_else(|| "Not logged in to JustLegal Cloud. Run 'openseason cloud login' first.".to_string())?;
                let report = sync::cloud_push_case(&path, &session)?;
                if json {
                    emit(true, &report, vec![]);
                } else {
                    println!("{}", report.message);
                }
                Ok(())
            }
        },
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
        Commands::Motion { action } => match action {
            MotionCmd::List { path } => {
                let profile = case_profile::load_case(&path)?;
                if json {
                    emit(true, &profile.filings, vec![]);
                } else {
                    println!("Case has {} motion/filing draft(s):", profile.filings.len());
                    for f in &profile.filings {
                        println!("  [{}] {} ({}) - {}", f.id, f.title, f.kind, f.status);
                    }
                }
                Ok(())
            }
            MotionCmd::Create { path, title, kind, rule, text } => {
                let mut profile = case_profile::load_case(&path)?;
                let now = chrono::Local::now().to_rfc3339();
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
        },
        Commands::Service { action } => match action {
            ServiceCmd::Cert { path, filing, method, date } => {
                let profile = case_profile::load_case(&path)?;
                let recipients: Vec<open_season_lib::service::ServiceRecipient> = profile
                    .caption
                    .appellees()
                    .into_iter()
                    .map(|p| open_season_lib::service::ServiceRecipient {
                        name: p.name.clone(),
                        role: p.role.clone(),
                        address: p.address.clone(),
                        email: p.email.clone(),
                    })
                    .collect();
                let params = open_season_lib::service::ServiceCertParams {
                    filing_title: filing,
                    service_date: date,
                    method,
                    recipients,
                    signature_name: profile.filer.signature_name.clone(),
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
        Commands::Intake { action } => match action {
            IntakeCmd::Status { path } => {
                let report = open_season_lib::intake::intake_status(&path)?;
                if json {
                    emit(true, &report, vec![]);
                } else {
                    println!("Intake Wizard Status: {}% ({}/{}) [Tier: {}, Venue: {}]", report.completion_percent, report.completed_steps, report.total_steps, report.matter_tier, report.venue_code);
                    if let Some(nq) = &report.next_question {
                        println!("Next Question (Step {}) [{}]: {}", nq.step_number, nq.id, nq.prompt);
                        println!("  Why: {}", nq.why);
                    } else {
                        println!("All intake steps complete!");
                    }
                }
                Ok(())
            }
            IntakeCmd::Answer { path, id, answer } => {
                let report = open_season_lib::intake::intake_answer(&path, &id, &answer)?;
                if json {
                    emit(true, &report, vec![]);
                } else {
                    println!("Recorded answer for '{}'. Progress: {}% ({}/{})", id, report.completion_percent, report.completed_steps, report.total_steps);
                    if let Some(nq) = &report.next_question {
                        println!("Next Question (Step {}) [{}]: {}", nq.step_number, nq.id, nq.prompt);
                    }
                }
                Ok(())
            }
            IntakeCmd::Scan { path } => {
                let report = open_season_lib::intake::intake_scan(&path)?;
                if json {
                    emit(true, &report, vec![]);
                } else {
                    println!("Scanned {} file(s) across case subfolders:", report.scanned_files.len());
                    for f in &report.scanned_files {
                        println!("  {}/{} ({} pages): {}", f.folder, f.filename, f.page_count, f.snippet);
                    }
                }
                Ok(())
            }
        },
        Commands::Portfolio { action } => match action {
            PortfolioCmd::Status { root } => {
                let cases_root = root.unwrap_or_else(case_profile::default_cases_root);
                let report = open_season_lib::portfolio::portfolio_status(&cases_root)?;
                if json {
                    emit(true, &report, vec![]);
                } else {
                    println!("Multi-State Portfolio Radar ({} matter(s)):", report.total_matters);
                    for m in &report.matters {
                        println!("  [{}] {} ({}) — {}% complete | due: {}", m.venue_code, m.title, m.matter_tier, m.completion_percent, m.urgent_deadline.as_deref().unwrap_or("none"));
                    }
                }
                Ok(())
            }
        },
    }
}
