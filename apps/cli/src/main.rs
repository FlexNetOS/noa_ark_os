use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use futures::StreamExt;
use noa_inference::{
    CompletionRequest, ProviderRouter, TelemetryEvent, TelemetryHandle, TelemetrySink,
    TelemetryStatus,
};
use noa_workflow::EvidenceLedgerEntry;
use noa_workflow::{InferenceMetric, PipelineInstrumentation};
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
    /// Interact with agents using configured inference providers
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },
    /// Run a natural language query through the inference router
    Query {
        #[arg(value_name = "PROMPT")]
        prompt: String,
        #[arg(long)]
        stream: bool,
    },
}

#[derive(Subcommand)]
enum AgentCommands {
    /// Invoke the default agent pipeline with a prompt
    Invoke {
        #[arg(value_name = "PROMPT")]
        prompt: String,
        #[arg(long)]
        stream: bool,
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
            Commands::Agent { command } => {
                let instrumentation = Arc::new(
                    PipelineInstrumentation::new()
                        .context("failed to initialise instrumentation")?,
                );
                let telemetry = inference_telemetry_handle(instrumentation);
                match command {
                    AgentCommands::Invoke { prompt, stream } => {
                        handle_invoke(prompt, stream, telemetry).await?
                    }
                }
            }
            Commands::Query { prompt, stream } => {
                let instrumentation = Arc::new(
                    PipelineInstrumentation::new()
                        .context("failed to initialise instrumentation")?,
                );
                let telemetry = inference_telemetry_handle(instrumentation);
                handle_query(prompt, stream, telemetry).await?;
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

struct InferenceTelemetryBridge {
    instrumentation: Arc<PipelineInstrumentation>,
}

impl TelemetrySink for InferenceTelemetryBridge {
    fn record(&self, event: TelemetryEvent) {
        let status = match event.status {
            TelemetryStatus::Success => "success",
            TelemetryStatus::Failure => "failure",
        }
        .to_string();
        let metric = InferenceMetric {
            provider: event.provider,
            model: event.model,
            status,
            latency_ms: event.latency_ms,
            tokens_prompt: event.tokens_prompt,
            tokens_completion: event.tokens_completion,
            error: event.error,
        };
        if let Err(err) = self.instrumentation.log_inference_metric(metric) {
            eprintln!("failed to record inference metric: {err}");
        }
    }
}

fn inference_telemetry_handle(instrumentation: Arc<PipelineInstrumentation>) -> TelemetryHandle {
    Arc::new(InferenceTelemetryBridge { instrumentation }) as TelemetryHandle
}

async fn build_router(telemetry: TelemetryHandle) -> Result<ProviderRouter> {
    let router = ProviderRouter::from_env()?;
    Ok(router.with_telemetry(telemetry))
}

async fn handle_invoke(prompt: String, stream: bool, telemetry: TelemetryHandle) -> Result<()> {
    let router = build_router(telemetry).await?;
    let request = CompletionRequest {
        prompt,
        temperature: None,
        max_tokens: Some(512),
        stop: None,
    };

    if stream {
        let mut stream = router.stream_completion(request).await?;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            if !chunk.content.is_empty() {
                print!("{}", chunk.content);
                std::io::stdout().flush().ok();
            }
        }
        println!();
    } else {
        let response = router.completion(request).await?;
        println!("{}", response.content);
    }

    Ok(())
}

async fn handle_query(prompt: String, stream: bool, telemetry: TelemetryHandle) -> Result<()> {
    let router = build_router(telemetry).await?;
    let request = CompletionRequest {
        prompt,
        temperature: None,
        max_tokens: Some(512),
        stop: None,
    };

    if stream {
        let mut stream = router.stream_completion(request).await?;
        print!("Response: ");
        std::io::stdout().flush().ok();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            if !chunk.content.is_empty() {
                print!("{}", chunk.content);
                std::io::stdout().flush().ok();
            }
        }
        println!();
    } else {
        let response = router.completion(request).await?;
        println!("Response: {}", response.content.trim());
    }

    Ok(())
}
