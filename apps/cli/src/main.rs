use std::io::BufRead;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use noa_workflow::EvidenceLedgerEntry;
use relocation_daemon::{ExecutionMode, RelocationDaemon};
use tokio::runtime::Runtime;
use uuid::Uuid;

#[derive(Parser)]
#[command(
    name = "noa-cli",
    about = "NOA Ark OS relocation daemon tooling",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Clone)]
struct DaemonArgs {
    #[arg(long, default_value = ".workspace/config/relocation_policies.yaml")]
    policy: PathBuf,
    #[arg(long, default_value = ".workspace/registry/files.json")]
    registry: PathBuf,
    #[arg(long, default_value = ".workspace/backups")]
    backups: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute the relocation daemon once with the selected mode
    Run {
        #[command(flatten)]
        daemon: DaemonArgs,
        #[arg(long, default_value = "approval", value_parser = parse_mode)]
        mode: ExecutionMode,
    },
    /// Display the current relocation state snapshot
    Status {
        #[command(flatten)]
        daemon: DaemonArgs,
    },
    /// Approve a pending relocation action by its UUID
    Approve {
        #[command(flatten)]
        daemon: DaemonArgs,
        #[arg(value_parser = parse_uuid)]
        action_id: Uuid,
    },
    /// Manually override a relocation for the provided source and destination
    Override {
        #[command(flatten)]
        daemon: DaemonArgs,
        source: PathBuf,
        destination: PathBuf,
        #[arg(long)]
        force: bool,
    },
    /// Inspect the evidence ledger for workflow receipts and scan results
    Evidence {
        #[arg(long)]
        workflow: Option<String>,
        #[arg(long)]
        limit: Option<usize>,
    },
}

fn parse_mode(value: &str) -> std::result::Result<ExecutionMode, String> {
    ExecutionMode::from_str(value).map_err(|err| err.to_string())
}

fn parse_uuid(value: &str) -> std::result::Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|err| err.to_string())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let runtime = Runtime::new().context("failed to start Tokio runtime")?;

    runtime.block_on(async move {
        match cli.command {
            Commands::Run { daemon, mode } => {
                let daemon = build_daemon(daemon).await?;
                let report = daemon.run(mode).await?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            Commands::Status { daemon } => {
                let daemon = build_daemon(daemon).await?;
                let state = daemon.status().await?;
                println!("{}", serde_json::to_string_pretty(&state)?);
            }
            Commands::Approve { daemon, action_id } => {
                let daemon = build_daemon(daemon).await?;
                match daemon.approve_action(action_id).await? {
                    Some(result) => {
                        println!("Approved action {}", result.action.id);
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    }
                    None => {
                        println!("No pending action found for {}", action_id);
                    }
                }
            }
            Commands::Override {
                daemon,
                source,
                destination,
                force,
            } => {
                let daemon = build_daemon(daemon).await?;
                let result = daemon
                    .manual_override(
                        source.to_string_lossy().to_string(),
                        destination.to_string_lossy().to_string(),
                        force,
                    )
                    .await?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            Commands::Evidence { workflow, limit } => {
                show_evidence(workflow, limit)?;
            }
        }

        Ok(())
    })
}

fn show_evidence(workflow_filter: Option<String>, limit: Option<usize>) -> Result<()> {
    let path = PathBuf::from("storage/db/evidence/ledger.jsonl");
    if !path.exists() {
        anyhow::bail!("evidence ledger not found at {}", path.display());
    }
    let file = std::fs::File::open(&path)?;
    let reader = std::io::BufReader::new(file);
    let mut entries = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: EvidenceLedgerEntry = serde_json::from_str(&line)?;
        if let Some(workflow) = &workflow_filter {
            let payload_workflow = entry
                .payload
                .get("workflow_id")
                .and_then(|value| value.as_str());
            if payload_workflow != Some(workflow.as_str()) {
                continue;
            }
        }
        entries.push(entry);
    }

    let to_print = limit.unwrap_or(entries.len());
    for entry in entries.into_iter().rev().take(to_print) {
        println!(
            "{} | kind={:?} | reference={}",
            entry.timestamp, entry.kind, entry.reference
        );
        println!("{}", serde_json::to_string_pretty(&entry.payload)?);
    }
    Ok(())
}

async fn build_daemon(config: DaemonArgs) -> Result<RelocationDaemon> {
    RelocationDaemon::new(config.policy, config.registry, config.backups).await
}
