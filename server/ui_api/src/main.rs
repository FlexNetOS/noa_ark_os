use std::net::SocketAddr;

use anyhow::{Context, Result};
use clap::Parser;
use noa_ui_api::UiApiServer;
use tokio::net::TcpListener;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(
    name = "noa-ui-api-server",
    about = "Server-driven UI API for the NOA Ark OS workspace",
    version
)]
struct Args {
    /// Address the server should bind to. Accepts HOST:PORT.
    #[arg(long, env = "NOA_UI_API_ADDR", default_value = "127.0.0.1:8787")]
    addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let args = Args::parse();

    let socket_addr: SocketAddr = args
        .addr
        .parse()
        .with_context(|| format!("invalid address '{}'", args.addr))?;

    let listener = TcpListener::bind(socket_addr)
        .await
        .with_context(|| format!("failed to bind {socket_addr}"))?;

    let drop_root = std::env::var("NOA_UI_DROP_ROOT").unwrap_or_else(|_| "crc/drop-in/incoming".into());
    info!(%socket_addr, %drop_root, "starting NOA UI API server");

    let server = UiApiServer::new();
    let router = server.router();

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("UI API server exited unexpectedly")?;

    Ok(())
}

fn init_tracing() {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .compact()
        .finish();

    if tracing::subscriber::set_global_default(subscriber).is_err() {
        // Tracing was already initialised by an embedding binary; ignore.
    }
}

async fn shutdown_signal() {
    if tokio::signal::ctrl_c().await.is_ok() {
        info!("shutdown signal received, stopping UI API server");
    } else {
        warn!("failed to listen for Ctrl+C shutdown signal");
    }
}
