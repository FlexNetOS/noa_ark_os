use anyhow::Context;
use clap::Parser;
use noa_api::{ApiConfig, ApiServer};
use noa_gateway::bootstrap_gateway;
use noa_orchestrator::UnifiedOrchestrator;
use tokio::runtime::Builder;
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(author, version, about = "Unified NOA server", long_about = None)]
struct Cli {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(long, default_value_t = 8080)]
    port: u16,

    #[arg(long, default_value_t = Cli::default_workers())]
    workers: usize,
}

impl Cli {
    fn default_workers() -> usize {
        num_cpus::get().max(1)
    }
}

fn init_tracing() {
    if tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init()
        .is_err()
    {
        warn!("tracing subscriber already initialised");
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_tracing();

    let orchestrator = UnifiedOrchestrator::default();
    let decision = orchestrator.evaluate_scaling();
    info!(?decision, "orchestrator ready");

    let gateway = bootstrap_gateway().context("failed to bootstrap gateway")?;
    info!("gateway initialised; unified server wiring API and gateway layers");

    let runtime = Builder::new_multi_thread()
        .enable_all()
        .worker_threads(cli.workers)
        .build()
        .context("failed to build tokio runtime")?;

    runtime.block_on(async move {
        let server = ApiServer::new(ApiConfig {
            host: cli.host,
            port: cli.port,
        })
        .context("failed to initialise API server")?;

        info!("starting Axum + Tonic API server");

        let _gateway = gateway; // Hold gateway wiring for the server lifetime.
        server.run().await
    })
}
