use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use noa_plugin_sdk::{ToolDescriptor, ToolRegistry};
use futures::StreamExt;
use noa_inference::{
    CompletionRequest, ProviderRouter, TelemetryEvent, TelemetryHandle, TelemetrySink,
    TelemetryStatus,
};
use noa_cicd::CICDSystem;
use noa_workflow::EvidenceLedgerEntry;
use noa_workflow::{InferenceMetric, PipelineInstrumentation};
use relocation_daemon::{ExecutionMode, RelocationDaemon};
use serde::Serialize;
use serde_json::json;
use tokio::runtime::Runtime;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutputMode {
    Json,
    Yaml,
}

#[derive(Parser)]
#[command(
    name = "noa",
    about = "NOA Ark OS relocation daemon tooling",
    about = "NOA Ark OS unified CLI (kernel, world, registry, trust, snapshot, agent, policy, sbom, pipeline, profile)",
    version
)]
struct Cli {
    /// Emit JSON output (default)
    #[arg(long, global = true, default_value_t = true)]
    json: bool,
    /// Emit YAML output
    #[arg(long, global = true)]
    yaml: bool,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    fn output_mode(&self) -> OutputMode {
        if self.yaml { OutputMode::Yaml } else { OutputMode::Json }
    }
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
    /// Kernel operations
    Kernel { #[command(subcommand)] cmd: KernelCmd },
    /// World model operations
    World { #[command(subcommand)] cmd: WorldCmd },
    /// Registry operations
    Registry { #[command(subcommand)] cmd: RegistryCmd },
    /// Trust operations
    Trust { #[command(subcommand)] cmd: TrustCmd },
    /// Snapshot operations
    Snapshot { #[command(subcommand)] cmd: SnapshotCmd },
    /// Policy operations
    Policy { #[command(subcommand)] cmd: PolicyCmd },
    /// SBOM operations
    Sbom { #[command(subcommand)] cmd: SbomCmd },
    /// Profile operations
    Profile { #[command(subcommand)] cmd: ProfileCmd },

    /// Relocation daemon tooling (legacy)
    Relocation { #[command(subcommand)] cmd: RelocationCmd },

    /// Inspect the evidence ledger
    Evidence {
        #[arg(long)]
        workflow: Option<String>,
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Surface observability tooling from the shared registry
    Observability { #[command(flatten)] query: RegistryArgs },
    /// Surface automation tooling from the shared registry
    Automation { #[command(flatten)] query: RegistryArgs },
    /// Surface analysis tooling from the shared registry
    Analysis { #[command(flatten)] query: RegistryArgs },
    /// Surface collaboration tooling from the shared registry
    Collaboration { #[command(flatten)] query: RegistryArgs },
    /// Surface plugin tooling from the shared registry
    Plugin { #[command(flatten)] query: RegistryArgs },
    /// Interact with agents using configured inference providers
    Agent { #[command(subcommand)] command: AgentCommands },
    /// Run a natural language query through the inference router
    Query {
        #[arg(value_name = "PROMPT")]
        prompt: String,
        #[arg(long)]
        stream: bool,
    },
    /// Manage CI/CD pipelines and agent approvals
    Pipeline { #[command(subcommand)] command: PipelineCommands },
}

#[derive(Subcommand)]
enum KernelCmd { Start, Stop, Status, Logs }

#[derive(Subcommand)]
enum WorldCmd { Verify, Fix, Graph, Diff { snapshot: String } }

#[derive(Subcommand)]
enum RegistryCmd { List, Describe { tool: String }, Search { query: String }, Validate }

#[derive(Subcommand)]
enum TrustCmd { Score, Audit { #[arg(long)] history: bool }, Thresholds { #[arg(value_name = "RULES")] rules: Option<String> } }

#[derive(Subcommand)]
enum SnapshotCmd { Create { name: String }, List, Rollback { id: String }, Verify { id: String } }

#[derive(Subcommand)]
enum PolicyCmd { Validate { file: PathBuf }, Apply { file: PathBuf }, Test { file: PathBuf }, DryRun { file: PathBuf } }

#[derive(Subcommand)]
enum SbomCmd { Generate, Verify, Sign, Audit }

#[derive(Subcommand)]
enum ProfileCmd { Switch { name: String }, List, Validate { name: String }, Diff { a: String, b: String } }

#[derive(Subcommand)]
enum RelocationCmd {
    /// Execute the relocation daemon once with the selected mode
    Run {
        #[command(flatten)]
        daemon: DaemonArgs,
        #[arg(long, default_value = "approval", value_parser = parse_mode)]
        mode: ExecutionMode,
    },
    /// Display the current relocation state snapshot
    Status { #[command(flatten)] daemon: DaemonArgs },
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
}

#[derive(Serialize)]
struct ToolCategoryResponse {
    schema_version: String,
    generated_at: String,
    category: String,
    total: usize,
    tools: Vec<ToolDescriptor>,
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

#[derive(Subcommand)]
enum PipelineCommands {
    /// Approve a pipeline using agent trust policies
    Approve {
        #[arg(long)]
        pipeline: String,
        #[arg(long)]
        agent: String,
        #[arg(long)]
        agent_id: Option<String>,
        #[arg(long, default_value_t = 0.8)]
        trust: f32,
        #[arg(long, value_delimiter = ',')]
        evidence: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
}

fn parse_mode(value: &str) -> std::result::Result<ExecutionMode, String> {
    ExecutionMode::from_str(value).map_err(|err| err.to_string())
}

fn parse_uuid(value: &str) -> std::result::Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|err| err.to_string())
}

fn print_obj(mode: OutputMode, value: &serde_json::Value) -> Result<()> {
    match mode {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(value)?),
        OutputMode::Yaml => println!("{}", serde_yaml::to_string(value)?),
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let out_mode = cli.output_mode();
    let runtime = Runtime::new().context("failed to start Tokio runtime")?;

    runtime.block_on(async move {
        match cli.command {
            Commands::Kernel { cmd } => {
                let v = match cmd {
                    KernelCmd::Start => json!({"component":"kernel","action":"start","status":"not_implemented"}),
                    KernelCmd::Stop => json!({"component":"kernel","action":"stop","status":"not_implemented"}),
                    KernelCmd::Status => json!({"component":"kernel","action":"status","status":"not_implemented"}),
                    KernelCmd::Logs => json!({"component":"kernel","action":"logs","status":"not_implemented"}),
                };
                print_obj(out_mode, &v)?;
            }
            Commands::World { cmd } => {
                let v = match cmd {
                    WorldCmd::Verify => json!({"component":"world","action":"verify","status":"not_implemented"}),
                    WorldCmd::Fix => json!({"component":"world","action":"fix","status":"not_implemented"}),
                    WorldCmd::Graph => json!({"component":"world","action":"graph","status":"not_implemented"}),
                    WorldCmd::Diff { snapshot } => json!({"component":"world","action":"diff","snapshot": snapshot, "status":"not_implemented"}),
                };
                print_obj(out_mode, &v)?;
            }
            Commands::Registry { cmd } => {
                let v = match cmd {
                    RegistryCmd::List => json!({"component":"registry","action":"list","status":"not_implemented"}),
                    RegistryCmd::Describe { tool } => json!({"component":"registry","action":"describe","tool": tool, "status":"not_implemented"}),
                    RegistryCmd::Search { query } => json!({"component":"registry","action":"search","query": query, "status":"not_implemented"}),
                    RegistryCmd::Validate => json!({"component":"registry","action":"validate","status":"not_implemented"}),
                };
                print_obj(out_mode, &v)?;
            }
            Commands::Trust { cmd } => {
                let v = match cmd {
                    TrustCmd::Score => json!({"component":"trust","action":"score","status":"not_implemented"}),
                    TrustCmd::Audit { history } => json!({"component":"trust","action":"audit","history": history, "status":"not_implemented"}),
                    TrustCmd::Thresholds { rules } => json!({"component":"trust","action":"thresholds","rules": rules, "status":"not_implemented"}),
                };
                print_obj(out_mode, &v)?;
            }
            Commands::Snapshot { cmd } => {
                let v = match cmd {
                    SnapshotCmd::Create { name } => json!({"component":"snapshot","action":"create","name": name, "status":"not_implemented"}),
                    SnapshotCmd::List => json!({"component":"snapshot","action":"list","status":"not_implemented"}),
                    SnapshotCmd::Rollback { id } => json!({"component":"snapshot","action":"rollback","id": id, "status":"not_implemented"}),
                    SnapshotCmd::Verify { id } => json!({"component":"snapshot","action":"verify","id": id, "status":"not_implemented"}),
                };
                print_obj(out_mode, &v)?;
            }
            Commands::Policy { cmd } => {
                let v = match cmd {
                    PolicyCmd::Validate { file } => json!({"component":"policy","action":"validate","file": file, "status":"not_implemented"}),
                    PolicyCmd::Apply { file } => json!({"component":"policy","action":"apply","file": file, "status":"not_implemented"}),
                    PolicyCmd::Test { file } => json!({"component":"policy","action":"test","file": file, "status":"not_implemented"}),
                    PolicyCmd::DryRun { file } => json!({"component":"policy","action":"dry_run","file": file, "status":"not_implemented"}),
                };
                print_obj(out_mode, &v)?;
            }
            Commands::Sbom { cmd } => {
                let v = match cmd {
                    SbomCmd::Generate => json!({"component":"sbom","action":"generate","status":"not_implemented"}),
                    SbomCmd::Verify => json!({"component":"sbom","action":"verify","status":"not_implemented"}),
                    SbomCmd::Sign => json!({"component":"sbom","action":"sign","status":"not_implemented"}),
                    SbomCmd::Audit => json!({"component":"sbom","action":"audit","status":"not_implemented"}),
                };
                print_obj(out_mode, &v)?;
            }
            Commands::Profile { cmd } => {
                let v = match cmd {
                    ProfileCmd::Switch { name } => json!({"component":"profile","action":"switch","name": name, "status":"not_implemented"}),
                    ProfileCmd::List => json!({"component":"profile","action":"list","status":"not_implemented"}),
                    ProfileCmd::Validate { name } => json!({"component":"profile","action":"validate","name": name, "status":"not_implemented"}),
                    ProfileCmd::Diff { a, b } => json!({"component":"profile","action":"diff","a": a, "b": b, "status":"not_implemented"}),
                };
                print_obj(out_mode, &v)?;
            }
            Commands::Relocation { cmd } => {
                match cmd {
                    RelocationCmd::Run { daemon, mode } => {
                        let daemon = build_daemon(daemon).await?;
                        let report = daemon.run(mode).await?;
                        print_obj(out_mode, &serde_json::to_value(report)?)?;
                    }
                    RelocationCmd::Status { daemon } => {
                        let daemon = build_daemon(daemon).await?;
                        let state = daemon.status().await?;
                        print_obj(out_mode, &serde_json::to_value(state)?)?;
                    }
                    RelocationCmd::Approve { daemon, action_id } => {
                        let daemon = build_daemon(daemon).await?;
                        match daemon.approve_action(action_id).await? {
                            Some(result) => {
                                eprintln!("Approved action {}", result.action.id);
                                print_obj(out_mode, &serde_json::to_value(result)?)?;
                            }
                            None => {
                                let v = json!({"component":"relocation","action":"approve","id": action_id, "status":"not_found"});
                                print_obj(out_mode, &v)?;
                            }
                        }
                    }
                    RelocationCmd::Override { daemon, source, destination, force } => {
                        let daemon = build_daemon(daemon).await?;
                        let result = daemon
                            .manual_override(
                                source.to_string_lossy().to_string(),
                                destination.to_string_lossy().to_string(),
                                force,
                            )
                            .await?;
                        print_obj(out_mode, &serde_json::to_value(result)?)?;
                    }
                }
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
            Commands::Pipeline { command } => match command {
                PipelineCommands::Approve {
                    pipeline,
                    agent,
                    agent_id,
                    trust,
                    evidence,
                    tags,
                    workspace,
                } => {
                    let workspace_root = workspace.unwrap_or_else(|| {
                        std::env::current_dir().expect("unable to determine workspace")
                    });
                    std::env::set_var("NOA_WORKFLOW_ROOT", &workspace_root);
                    let cicd = CICDSystem::new();
                    cicd.configure_workspace_root(workspace_root.clone());
                    let agent_identifier = agent_id.unwrap_or_else(|| agent.clone());
                    let status = cicd
                        .register_agent_approval(
                            &pipeline,
                            &agent,
                            &agent_identifier,
                            trust,
                            tags.clone(),
                            evidence.clone(),
                        )
                        .map_err(anyhow::Error::msg)?;
                    let payload = json!({
                        "pipeline_id": pipeline,
                        "status": format!("{:?}", status),
                        "agent": agent_identifier,
                        "trust_score": trust,
                        "evidence": evidence,
                        "tags": tags,
                    });
                    println!("{}", serde_json::to_string_pretty(&payload)?);
                }
            },
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
