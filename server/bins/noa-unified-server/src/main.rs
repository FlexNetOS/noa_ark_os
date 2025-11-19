use anyhow::Context;
use noa_gateway::bootstrap_gateway;
use noa_orchestrator::UnifiedOrchestrator;
use tracing::{info, warn};

fn init_tracing() {
    if tracing_subscriber::fmt().with_env_filter("info").try_init().is_err() {
        warn!("tracing subscriber already initialised");
    }
}

fn main() -> anyhow::Result<()> {
    init_tracing();

    let orchestrator = UnifiedOrchestrator::default();
    let decision = orchestrator.evaluate_scaling();
    info!(?decision, "orchestrator ready");

    let gateway = bootstrap_gateway().context("failed to bootstrap gateway")?;
    info!("gateway initialised; unified server is ready for future wiring");

    // Placeholder until the HTTP server and orchestration loops are implemented.
    drop(gateway);
    Ok(())
}
