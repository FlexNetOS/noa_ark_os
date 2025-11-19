use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use futures::FutureExt;
use noa_ui_api::{UiApiServer, UiSchemaGrpc};
use tokio::net::TcpListener;
use tokio::sync::Notify;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(
    name = "noa-ui-api-server",
    about = "Server-driven UI API for the NOA Ark OS workspace",
    version
)]
struct Args {
    /// Address the HTTP server should bind to. Accepts HOST:PORT.
    #[arg(
        long = "http-addr",
        env = "NOA_UI_API_ADDR",
        default_value = "127.0.0.1:8787"
    )]
    http_addr: String,

    /// Address the gRPC server should bind to. Accepts HOST:PORT.
    #[arg(
        long = "grpc-addr",
        env = "NOA_UI_API_GRPC_ADDR",
        default_value = "127.0.0.1:50051"
    )]
    grpc_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let args = Args::parse();

    let http_addr: SocketAddr = args
        .http_addr
        .parse()
        .with_context(|| format!("invalid HTTP address '{}'", args.http_addr))?;
    let grpc_addr: SocketAddr = args
        .grpc_addr
        .parse()
        .with_context(|| format!("invalid gRPC address '{}'", args.grpc_addr))?;

    let listener = TcpListener::bind(http_addr)
        .await
        .with_context(|| format!("failed to bind {http_addr}"))?;

    let drop_root =
        std::env::var("NOA_UI_DROP_ROOT").unwrap_or_else(|_| "crc/drop-in/incoming".into());
    info!(%http_addr, %grpc_addr, %drop_root, "starting NOA UI API server");

    let server = UiApiServer::new();
    let router = server.router();
    let shutdown = Shutdown::new();

    let http_future = axum::serve(listener, router)
        .with_graceful_shutdown(shutdown.clone().wait())
        .map_err(|err| anyhow!("HTTP server exited: {err}"));

    let grpc_state = server.state();
    let grpc_service = UiSchemaGrpc::new(grpc_state);
    let grpc_future = tonic::transport::Server::builder()
        .add_service(
            noa_ui_api::grpc::proto::ui_schema_service_server::UiSchemaServiceServer::new(
                grpc_service,
            ),
        )
        .serve_with_shutdown(grpc_addr, shutdown.clone().wait())
        .map_err(|err| anyhow!("gRPC server exited: {err}"));

    let signal_shutdown = shutdown.clone();
    let signal_task = tokio::spawn(async move {
        shutdown_signal().await;
        info!("shutdown signal received, notifying services");
        signal_shutdown.trigger();
    });

    let result = tokio::try_join!(http_future, grpc_future);
    shutdown.trigger();
    signal_task.abort();
    result?;

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

#[derive(Clone)]
struct Shutdown {
    notify: Arc<Notify>,
}

impl Shutdown {
    fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
        }
    }

    async fn wait(&self) {
        self.notify.notified().await;
    }

    fn trigger(&self) {
        self.notify.notify_waiters();
    }
}
