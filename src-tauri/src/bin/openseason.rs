use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use open_season_lib::case_profile::{
    self, CaseMode, REVIEW_NOTICE,
};
use open_season_lib::documents;
use open_season_lib::sync::{self, SyncAction};

#[derive(Parser)]
#[command(
    name = "openseason",
    about = "OpenSeason local case folder and document builder. Formatting only. Never invents facts, arguments, or citations.",
    long_about = "Create a portable case folder, validate missing fields and placeholders, and export a Notice of Appeal to Word and PDF.\n\nHard rule: this tool only formats and assembles the user's own material. Read AGENTS.md before using it from an AI agent."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create or inspect a case folder
    Case {
        #[command(subcommand)]
        action: CaseCmd,
    },
    /// Export a court-formatted document from a case folder
    Export {
        #[command(subcommand)]
        action: ExportCmd,
    },
    /// Call the account sync stub (refuses confidential cases)
    Sync {
        /// Path to the case folder
        path: PathBuf,
        /// backup, publish, or pull
        #[arg(long, default_value = "backup")]
        action: String,
    },
}

#[derive(Subcommand)]
enum CaseCmd {
    /// Create a case folder with the standard layout
    Create {
        /// Folder to create (or a parent folder if it already has files)
        #[arg(long)]
        path: PathBuf,
        /// Caption-style title, for example "Example v. Sample County Clerk"
        #[arg(long)]
        title: String,
        /// standard or confidential
        #[arg(long, default_value = "standard")]
        mode: String,
        /// notice_of_appeal, complaint, motion, or other
        #[arg(long, default_value = "notice_of_appeal")]
        kind: String,
    },
    /// Report missing fields, missing folders, and unfilled placeholders
    Validate {
        /// Path to the case folder
        path: PathBuf,
    },
    /// Convert a standard case to Confidential. This cannot be undone.
    Seal {
        /// Path to the case folder
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum ExportCmd {
    /// Export a Notice of Appeal to Word (.docx) and PDF
    NoticeOfAppeal {
        /// Path to the case folder
        path: PathBuf,
        /// Optional output folder (defaults to <case>/exports)
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {}", err);
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Commands::Case { action } => match action {
            CaseCmd::Create {
                path,
                title,
                mode,
                kind,
            } => {
                let mode = CaseMode::parse(&mode)?;
                let (dir, profile) = case_profile::create_case_folder(&path, &title, mode, &kind)?;
                println!("Created case {}", profile.id);
                println!("Folder: {}", dir.display());
                println!("Mode: {}", profile.mode.as_str());
                println!("Next: fill case.json (court, docket, parties, filer, your own draft text), then run:");
                println!("  openseason case validate {}", dir.display());
                if mode == CaseMode::Confidential {
                    println!("This case is confidential. Sync, cloud backup, and publish are refused.");
                }
            }
            CaseCmd::Validate { path } => {
                let profile = case_profile::load_case(&path)?;
                let report = case_profile::validate_case(&profile, Some(&path));
                if report.ok {
                    println!("Case is complete enough to export.");
                    return Ok(());
                }
                println!("Case is not ready ({} issue(s)):", report.issues.len());
                for issue in &report.issues {
                    println!("  - {}: {}", issue.field, issue.message);
                }
                return Err("validation failed".to_string());
            }
            CaseCmd::Seal { path } => {
                let profile = open_season_lib::seal::seal_case_on_disk(&path)?;
                println!("Case {} is now confidential.", profile.id);
                println!("Wrote the {} marker. Sync, cloud backup, and publish will be refused.", open_season_lib::seal::SEAL_MARKER_NAME);
                println!("This cannot be reversed by editing case.json.");
                println!("CLI seal does not encrypt files. Use Confidential mode in the desktop app (unlocked vault) to encrypt a vault copy.");
            }
        },
        Commands::Export { action } => match action {
            ExportCmd::NoticeOfAppeal { path, out } => {
                let profile = case_profile::load_case(&path)?;
                let exported = documents::export_notice_of_appeal(
                    &path,
                    &profile,
                    out.as_deref(),
                )?;
                println!("Wrote {}", exported.docx.display());
                println!("Wrote {}", exported.pdf.display());
                println!("{}", REVIEW_NOTICE);
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
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
            if result.kind == "confidential_forbidden" {
                return Err(result.reason);
            }
            if result.kind == "not_implemented" {
                return Err(result.reason);
            }
        }
    }
    Ok(())
}
