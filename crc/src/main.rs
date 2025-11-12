use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
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
use noa_crc::transform::{execute_plan, DummyVerifier, FileReplacePlan, TransformPlan};
use noa_crc::watcher::spawn_watcher;
use noa_crc::{CRCConfig, CRCSystem};
use serde::Deserialize;
use serde_json::json;
use tokio::fs as async_fs;
use tracing::{error, info};
use tracing_subscriber;
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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("noa_crc=debug".parse()?)
                .add_directive("info".parse()?),
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Serve => run_service().await,
        Command::Run(args) => run_once(args).await,
        Command::Ingest(args) => ingest(args).await,
        Command::Migrate(args) => migrate(args).await,
        Command::Graph(args) => graph(args).await,
    }
}

async fn run_service() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting CRC service");
    verify_directory_structure()?;
    let config = load_config();
    let crc_system = CRCSystem::new(config.clone());
    let watcher_handle = spawn_watcher(crc_system.clone()).await?;
    let processor = ParallelDropProcessor::new_with_config(config);

    tokio::select! {
        result = watcher_handle => {
            error!("File watcher stopped unexpectedly: {:?}", result);
        }
        result = processor.start_processing() => {
            error!("Processor stopped unexpectedly: {:?}", result);
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received");
        }
    }

    info!("CRC service stopped cleanly");
    Ok(())
}

async fn run_once(args: RunArgs) -> Result<(), Box<dyn std::error::Error>> {
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
        info!("Using plan hint: {}", plan.display());
    }

    let engine = Engine::new(graph);
    let summary = engine.run(&args.checkpoint).await?;
    info!("Executed {} nodes", summary.executed.len());
    Ok(())
}

async fn ingest(args: IngestArgs) -> Result<(), Box<dyn std::error::Error>> {
    let digestors: Vec<Box<dyn Digestor>> = vec![
        Box::new(GitDigestor::default()),
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
    info!("Ingest report written to {}", args.report.display());
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
    info!("migration outcomes: {}", outcomes.len());
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
            for node in graph.nodes() {
                println!("{}\t{:?}\t{:?}", node.name, node.kind, node.lane);
            }
        }
        GraphCommand::Show { node } => {
            for graph_node in graph.nodes() {
                if graph_node.name == node {
                    println!(
                        "Node {} => kind={:?} lane={:?}",
                        graph_node.name, graph_node.kind, graph_node.lane
                    );
                }
            }
        }
        GraphCommand::Trace { node } => {
            let topo = graph.topo_order()?;
            println!("trace for {}", node);
            for id in topo {
                if let Some(graph_node) = graph.node(&id) {
                    println!(" - {}", graph_node.name);
                }
            }
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
