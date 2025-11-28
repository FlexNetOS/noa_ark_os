use anyhow::Context;
use noa_tools_agent::server::{run_server, ServerOptions};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    let options = ServerOptions::default();
    run_server(options)
        .await
        .context("failed to start NOA tools agent server")
}
