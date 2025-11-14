use std::io::BufRead;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use noa_plugin_sdk::{ToolDescriptor, ToolRegistry};
use noa_workflow::EvidenceLedgerEntry;
use relocation_daemon::{ExecutionMode, RelocationDaemon};
use serde::Serialize;
use tokio::runtime::Runtime;
use uuid::Uuid;

#[derive(Parser)]
#[command(
    name = "noa",
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

#[derive(Args, Clone)]
struct RegistryArgs {
    #[arg(long, default_value = "registry/tools.registry.json")]
    registry: PathBuf,
    /// Optional tool identifier, alias, or CLI command string to filter on.
    #[arg(long)]
    tool: Option<String>,
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
    /// Surface observability tooling from the shared registry
    Observability {
        #[command(flatten)]
        query: RegistryArgs,
    },
    /// Surface automation tooling from the shared registry
    Automation {
        #[command(flatten)]
        query: RegistryArgs,
    },
    /// Surface analysis tooling from the shared registry
    Analysis {
        #[command(flatten)]
        query: RegistryArgs,
    },
    /// Surface collaboration tooling from the shared registry
    Collaboration {
        #[command(flatten)]
        query: RegistryArgs,
    },
    /// Surface plugin tooling from the shared registry
    Plugin {
        #[command(flatten)]
        query: RegistryArgs,
    },
}

#[derive(Serialize)]
struct ToolCategoryResponse {
    schema_version: String,
    generated_at: String,
    category: String,
    total: usize,
    tools: Vec<ToolDescriptor>,
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
            Commands::Observability { query } => {
                handle_registry_category("observability", query)?;
            }
            Commands::Automation { query } => {
                handle_registry_category("automation", query)?;
            }
            Commands::Analysis { query } => {
                handle_registry_category("analysis", query)?;
            }
            Commands::Collaboration { query } => {
                handle_registry_category("collaboration", query)?;
            }
            Commands::Plugin { query } => {
                handle_registry_category("plugin", query)?;
            }
        }

        Ok(())
    })
}

fn handle_registry_category(category: &str, query: RegistryArgs) -> Result<()> {
    let RegistryArgs { registry, tool } = query;
    let registry = ToolRegistry::from_path(&registry)
        .with_context(|| format!("failed to load tool registry from {}", registry.display()))?;

    let mut selected: Vec<ToolDescriptor> = registry
        .tools_for_category(category)
        .into_iter()
        .cloned()
        .collect();

    if let Some(reference) = tool {
        if let Some(tool) = registry.find_tool(&reference) {
            if !tool.is_category(category) {
                bail!(
                    "tool '{}' belongs to '{}' not '{}'",
                    reference,
                    tool.category,
                    category
                );
            }
            selected = vec![tool.clone()];
        } else {
            bail!("no tool matched reference '{}'", reference);
        }
    }

    if selected.is_empty() {
        bail!("no tools registered for category '{}'", category);
    }

    let response = ToolCategoryResponse {
        schema_version: registry.schema_version.clone(),
        generated_at: registry.generated_at.clone(),
        category: category.to_string(),
        total: selected.len(),
        tools: selected,
    };

    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
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
