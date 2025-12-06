use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use noa_crc::cas::Cas;
use noa_crc::digestors::api::ApiDigestor;
use noa_crc::digestors::bin::BinaryDigestor;
use noa_crc::digestors::config::ConfigDigestor;
use noa_crc::digestors::git::GitDigestor;
use noa_crc::digestors::sbom::SbomDigestor;
use noa_crc::digestors::{AssetRecord, Digestor};
use noa_crc::engine::Engine;
use noa_crc::graph::{CRCGraph, GraphNode, NodeKind};
use noa_crc::ir::Lane;
use noa_crc::parallel::ParallelDropProcessor;
use noa_crc::telemetry;
use noa_crc::transform::{execute_plan, DummyVerifier, FileReplacePlan, TransformPlan};
use noa_crc::watcher::spawn_watcher;
use noa_crc::{CRCConfig, CRCSystem};
use serde::Deserialize;
use serde_json::json;
use tokio::fs as async_fs;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "crc", version, about = "Continuous ReCode System CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the long-lived CRC service
    Serve,
    /// Execute a CRC plan once
    Run(RunArgs),
    /// Digest a repository or artifact root
    Ingest(IngestArgs),
    /// Interact with the content addressed store
    Cas(CasArgs),
    /// Execute structural migration flows
    Migrate(MigrateArgs),
    /// Inspect CRC graph plans
    Graph(GraphArgs),
}

#[derive(Args)]
struct RunArgs {
    /// Path to the plan definition (metadata only)
    #[arg(long)]
    plan: Option<PathBuf>,
    /// Checkpoint output directory
    #[arg(long, default_value = "out/ckpt")]
    checkpoint: PathBuf,
}

#[derive(Args)]
struct IngestArgs {
    #[arg(long)]
    root: PathBuf,
    #[arg(long)]
    report: PathBuf,
}

#[derive(Args)]
struct CasArgs {
    #[command(subcommand)]
    command: CasCommand,
}

#[derive(Subcommand)]
enum CasCommand {
    /// Store a file in the CAS
    Put {
        /// Path to the input file
        input: PathBuf,
    },
    /// Retrieve a file from the CAS
    Get {
        /// Content hash to retrieve
        hash: String,
        /// Optional output path; defaults to base64 on stdout
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Show metadata about a CAS object
    Stat {
        /// Content hash to inspect
        hash: String,
    },
}

#[derive(Args)]
struct MigrateArgs {
    #[arg(long = "plan")]
    plans: Vec<PathBuf>,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    apply: bool,
    #[arg(long)]
    rollback: bool,
    #[arg(long, default_value = ".")]
    root: PathBuf,
}

#[derive(Args)]
struct GraphArgs {
    #[command(subcommand)]
    command: GraphCommand,
}

#[derive(Subcommand)]
enum GraphCommand {
    Ls,
    Show { node: String },
    Trace { node: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    telemetry::init();

    let cli = Cli::parse();
    match cli.command {
        Command::Serve => run_service().await,
        Command::Run(args) => run_once(args).await,
        Command::Ingest(args) => ingest(args).await,
        Command::Cas(args) => cas_cli(args).await,
        Command::Migrate(args) => migrate(args).await,
        Command::Graph(args) => graph(args).await,
    }
}

async fn cas_cli(args: CasArgs) -> Result<(), Box<dyn std::error::Error>> {
    let cas = Cas::from_env_or_default()?;
    match args.command {
        CasCommand::Put { input } => {
            let bytes = async_fs::read(&input).await?;
            let hash = cas.put_bytes(&bytes)?;
            telemetry::info(
                "crc.cas",
                "put",
                "Stored bytes in CAS",
                "success",
                None,
                Some(json!({
                    "input": input,
                    "hash": hash,
                    "size": bytes.len()
                })),
            );
        }
        CasCommand::Get { hash, output } => {
            let bytes = cas.get(&hash)?;
            if let Some(path) = output {
                if let Some(parent) = path.parent() {
                    async_fs::create_dir_all(parent).await?;
                }
                async_fs::write(&path, &bytes).await?;
                telemetry::info(
                    "crc.cas",
                    "get_to_path",
                    "Materialized CAS object to disk",
                    "success",
                    None,
                    Some(json!({
                        "hash": hash,
                        "output": path,
                        "size": bytes.len()
                    })),
                );
            } else {
                use base64::engine::general_purpose::STANDARD_NO_PAD;
                use base64::Engine;
                let encoded = STANDARD_NO_PAD.encode(&bytes);
                telemetry::info(
                    "crc.cas",
                    "get_to_stdout",
                    "Returned CAS object as base64",
                    "success",
                    None,
                    Some(json!({ "hash": hash, "base64": encoded })),
                );
            }
        }
        CasCommand::Stat { hash } => {
            let entry = cas.stat(&hash)?;
            telemetry::info(
                "crc.cas",
                "stat",
                "Reported CAS object metadata",
                "success",
                None,
                Some(json!({ "hash": hash, "entry": entry })),
            );
        }
    }
    Ok(())
}

async fn run_service() -> Result<(), Box<dyn std::error::Error>> {
    let trace_id = telemetry::info(
        "crc.cli",
        "service_start",
        "Starting CRC service",
        "started",
        None,
        None,
    );
    verify_directory_structure()?;
    let config = load_config();
    let crc_system = CRCSystem::new(config.clone());
    let watcher_handle = spawn_watcher(crc_system.clone()).await?;
    let processor = ParallelDropProcessor::new_with_config(config);

    tokio::select! {
        result = watcher_handle => {
            telemetry::error(
                "crc.cli",
                "watcher_stopped",
                "File watcher stopped unexpectedly",
                "failed",
                Some(&trace_id),
                Some(json!({ "error": format!("{result:?}") })),
            );
        }
        result = processor.start_processing() => {
            telemetry::error(
                "crc.cli",
                "processor_stopped",
                "Processor stopped unexpectedly",
                "failed",
                Some(&trace_id),
                Some(json!({ "error": format!("{result:?}") })),
            );
        }
        _ = tokio::signal::ctrl_c() => {
            telemetry::info(
                "crc.cli",
                "shutdown_signal",
                "Shutdown signal received",
                "pending",
                Some(&trace_id),
                None,
            );
        }
    }

    telemetry::info(
        "crc.cli",
        "service_stop",
        "CRC service stopped cleanly",
        "success",
        Some(&trace_id),
        None,
    );
    Ok(())
}

async fn run_once(args: RunArgs) -> Result<(), Box<dyn std::error::Error>> {
    let trace_id = telemetry::new_trace_id();
    let mut graph = CRCGraph::new();
    let analyze = graph.add_node(GraphNode::new("analyze", NodeKind::Analyze, Lane::Fast));
    let decide = graph.add_node(GraphNode::new("decide", NodeKind::Decide, Lane::Fast));
    let transform = graph.add_node(GraphNode::new("transform", NodeKind::Transform, Lane::Fast));
    let verify = graph.add_node(GraphNode::new("verify", NodeKind::Verify, Lane::Deep));
    let persist = graph.add_node(GraphNode::new("persist", NodeKind::Persist, Lane::Deep));
    let _ = graph.add_edge(&analyze, &decide);
    let _ = graph.add_edge(&decide, &transform);
    let _ = graph.add_edge(&transform, &verify);
    let _ = graph.add_edge(&verify, &persist);

    if let Some(plan) = args.plan {
        telemetry::info(
            "crc.cli",
            "plan_hint",
            "Using plan hint",
            "observed",
            Some(&trace_id),
            Some(json!({ "plan": plan })),
        );
    }

    let engine = Engine::new(graph);
    let summary = engine.run(&args.checkpoint).await?;
    telemetry::info(
        "crc.cli",
        "plan_executed",
        "Executed plan nodes",
        "success",
        Some(&trace_id),
        Some(json!({ "executed": summary.executed.len(), "checkpoint": args.checkpoint })),
    );
    Ok(())
}

async fn ingest(args: IngestArgs) -> Result<(), Box<dyn std::error::Error>> {
    let trace_id = telemetry::new_trace_id();
    let digestors: Vec<Box<dyn Digestor>> = vec![
        Box::new(GitDigestor),
        Box::new(ConfigDigestor),
        Box::new(ApiDigestor),
        Box::new(SbomDigestor),
        Box::new(BinaryDigestor),
    ];
    let mut assets: Vec<AssetRecord> = Vec::new();
    for digestor in &digestors {
        let mut records = digestor.digest(&args.root)?;
        assets.append(&mut records);
    }
    let total_files = WalkDir::new(&args.root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count();
    let coverage = if total_files == 0 {
        1.0
    } else {
        (assets.len() as f32 / total_files as f32).min(1.0)
    };
    let report = json!({
        "root": args.root,
        "assets": assets,
        "coverage": coverage,
        "trust_average": assets.iter().map(|a| a.trust as f64).sum::<f64>() / assets.len().max(1) as f64,
    });
    if let Some(parent) = args.report.parent() {
        async_fs::create_dir_all(parent).await?;
    }
    async_fs::write(&args.report, serde_json::to_vec_pretty(&report)?).await?;
    telemetry::info(
        "crc.cli",
        "ingest_report",
        "Ingest report written",
        "success",
        Some(&trace_id),
        Some(
            json!({ "report_path": args.report, "asset_count": assets.len(), "coverage": coverage }),
        ),
    );
    Ok(())
}

#[derive(Debug, Deserialize)]
struct FilePlan {
    id: String,
    target: PathBuf,
    replacement: String,
}

async fn migrate(args: MigrateArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.plans.is_empty() {
        return Err(anyhow!("no plans provided").into());
    }
    let mut outcomes = HashMap::new();
    for plan_path in &args.plans {
        let bytes = async_fs::read(plan_path).await?;
        let plan: FilePlan = if plan_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("json"))
            .unwrap_or(false)
        {
            serde_json::from_slice(&bytes)?
        } else {
            serde_yaml::from_slice(&bytes)?
        };
        let transform = FileReplacePlan::new(&plan.id, &plan.target, &plan.replacement);
        let verifier: Box<dyn noa_crc::transform::Verifier> = Box::new(DummyVerifier);
        let result = execute_plan(
            &transform,
            &[verifier],
            &args.root,
            args.apply && !args.dry_run,
        )?;
        outcomes.insert(plan.id.clone(), result);
        if args.rollback {
            transform.rollback(&args.root)?;
        }
    }
    let plan_ids: Vec<String> = outcomes.keys().cloned().collect();
    telemetry::info(
        "crc.migrate",
        "plan_executed",
        "Executed migration plans",
        "success",
        None,
        Some(json!({
            "count": outcomes.len(),
            "plans": plan_ids
        })),
    );
    Ok(())
}

async fn graph(args: GraphArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut graph = CRCGraph::new();
    let analyze = graph.add_node(GraphNode::new("analyze", NodeKind::Analyze, Lane::Fast));
    let decide = graph.add_node(GraphNode::new("decide", NodeKind::Decide, Lane::Fast));
    let transform = graph.add_node(GraphNode::new("transform", NodeKind::Transform, Lane::Fast));
    let verify = graph.add_node(GraphNode::new("verify", NodeKind::Verify, Lane::Deep));
    let persist = graph.add_node(GraphNode::new("persist", NodeKind::Persist, Lane::Deep));
    let _ = graph.add_edge(&analyze, &decide);
    let _ = graph.add_edge(&decide, &transform);
    let _ = graph.add_edge(&transform, &verify);
    let _ = graph.add_edge(&verify, &persist);

    match args.command {
        GraphCommand::Ls => {
            let nodes: Vec<_> = graph
                .nodes()
                .map(|node| {
                    json!({
                        "name": node.name.clone(),
                        "kind": format!("{:?}", node.kind),
                        "lane": format!("{:?}", node.lane)
                    })
                })
                .collect();
            telemetry::info(
                "crc.graph",
                "list",
                "Listed CRC graph nodes",
                "success",
                None,
                Some(json!({ "nodes": nodes })),
            );
        }
        GraphCommand::Show { node } => {
            if let Some(graph_node) = graph.nodes().find(|n| n.name == node) {
                telemetry::info(
                    "crc.graph",
                    "show",
                    "Displayed CRC graph node",
                    "success",
                    None,
                    Some(json!({
                        "name": graph_node.name.clone(),
                        "kind": format!("{:?}", graph_node.kind),
                        "lane": format!("{:?}", graph_node.lane)
                    })),
                );
            } else {
                telemetry::warn(
                    "crc.graph",
                    "show_missing",
                    "Requested node not found",
                    "not_found",
                    None,
                    Some(json!({ "node": node })),
                );
            }
        }
        GraphCommand::Trace { node } => {
            let topo = graph.topo_order()?;
            let trace: Vec<_> = topo
                .into_iter()
                .filter_map(|id| graph.node(&id).map(|graph_node| graph_node.name.clone()))
                .collect();
            telemetry::info(
                "crc.graph",
                "trace",
                "Generated CRC graph trace",
                "success",
                None,
                Some(json!({ "target": node, "trace": trace })),
            );
        }
    }
    Ok(())
}

fn load_config() -> CRCConfig {
    let mut config = CRCConfig::default();

    if let Ok(max_concurrent) = std::env::var("CRC_MAX_CONCURRENT") {
        if let Ok(value) = max_concurrent.parse() {
            config.max_concurrent = value;
        }
    }

    if let Ok(auto_archive) = std::env::var("CRC_AUTO_ARCHIVE") {
        config.auto_archive = auto_archive.to_lowercase() == "true";
    }

    if let Ok(trigger_cicd) = std::env::var("CRC_TRIGGER_CICD") {
        config.trigger_cicd = trigger_cicd.to_lowercase() == "true";
    }

    if let Ok(compression) = std::env::var("CRC_COMPRESSION") {
        config.compression_algorithm = compression;
    }

    config
}

fn verify_directory_structure() -> Result<(), Box<dyn std::error::Error>> {
    let required_dirs = vec![
        "crc/drop-in/incoming/repos",
        "crc/drop-in/incoming/forks",
        "crc/drop-in/incoming/mirrors",
        "crc/drop-in/incoming/stale",
        "crc/drop-in/processing/adaptation",
        "crc/drop-in/processing/analysis",
        "crc/drop-in/processing/validation",
        "crc/drop-in/ready/model-a-queue",
        "crc/drop-in/ready/model-b-queue",
        "crc/drop-in/ready/model-c-queue",
        "crc/drop-in/ready/model-d-queue",
        "crc/archive/stale",
        "crc/archive/repos",
        "crc/archive/forks",
        "crc/archive/mirrors",
        "crc/temp/analysis-cache",
        "crc/temp/extracts",
        "crc/temp/logs",
        "storage/artifacts",
        "storage/artifacts/edge",
        "storage/artifacts/server",
    ];

    for dir in required_dirs {
        let path = PathBuf::from(dir);
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
    }

    Ok(())
}
