use std::process::ExitCode;
use clap::Parser;
use open_season_lib::agent::AgentEnvelope;
use open_season_lib::case_profile;

mod args;
mod handlers_core;
mod handlers_tools;
mod handlers_accountability;
mod handlers_workflow;
mod handlers_motion;
mod handlers_quitam;

use args::{Cli, Commands};

pub(crate) fn emit<T: serde::Serialize>(
    json: bool,
    value: T,
    issues: Vec<case_profile::ValidationIssue>,
) {
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
        Commands::Motion { action } => handlers_motion::handle_motion(action, json),

        Commands::Case { .. }
        | Commands::Draft { .. }
        | Commands::Evidence { .. }
        | Commands::Check { .. }
        | Commands::Export { .. }
        | Commands::Sync { .. }
        | Commands::Cloud { .. }
        | Commands::Intake { .. }
        | Commands::Portfolio { .. } => handlers_core::handle_core_command(cli.command, json),

        Commands::Mcp
        | Commands::Agent { .. }
        | Commands::Capabilities
        | Commands::Packs { .. }
        | Commands::Guide { .. }
        | Commands::Gaps { .. }
        | Commands::Docs { .. }
        | Commands::Form { .. }
        | Commands::Service { .. }
        | Commands::Pdf { .. }
        | Commands::Sign { .. } => handlers_tools::handle_tools_command(cli.command, json),

        Commands::Record { action } => handlers_accountability::handle_record(action, json).map_err(|e| e.to_string()),
        Commands::Review { action } => handlers_accountability::handle_review(action, json).map_err(|e| e.to_string()),
        Commands::Accountability { action } => handlers_accountability::handle_accountability(action, json).map_err(|e| e.to_string()),
        Commands::Ada { action } => handlers_accountability::handle_ada(action, json).map_err(|e| e.to_string()),
        Commands::Order { action } => handlers_accountability::handle_order(action, json).map_err(|e| e.to_string()),
        Commands::Timeline { action } => handlers_accountability::handle_timeline(action, json).map_err(|e| e.to_string()),
        Commands::Court { action } => handlers_accountability::handle_court(action, json).map_err(|e| e.to_string()),
        Commands::Workflow { action } => handlers_workflow::handle_workflow(action, json).map_err(|e| e.to_string()),
        Commands::QuiTam { action } => handlers_quitam::handle_quitam(action, json),
        Commands::Doctor => handlers_core::handle_doctor(json),
    }
}


fn main() -> ExitCode {
    let json_flag = std::env::args().any(|a| a == "--json");
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            use clap::error::ErrorKind;
            if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
                let _ = e.print();
                return ExitCode::SUCCESS;
            }
            if json_flag {
                let env = AgentEnvelope::with_issues(
                    serde_json::Value::Null,
                    vec![case_profile::ValidationIssue {
                        field: "cli".into(),
                        message: e.to_string(),
                    }],
                );
                println!("{}", env.to_json().unwrap_or_else(|_| "{}".into()));
                return ExitCode::FAILURE;
            }
            let _ = e.print();
            return ExitCode::FAILURE;
        }
    };
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
