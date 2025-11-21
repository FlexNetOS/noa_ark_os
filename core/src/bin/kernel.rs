use std::sync::Arc;

use noa_core::scorekeeper::{api, Scorekeeper};

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Kernel initialization failed: {err}");
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let kernel = noa_core::init()?;
    let scorekeeper = Arc::new(Scorekeeper::default()?);
    scorekeeper.bootstrap()?;

    let router = api::router(Arc::clone(&scorekeeper));

    let addr = std::env::var("NOA_KERNEL_HTTP_ADDR").unwrap_or_else(|_| "127.0.0.1:7878".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Kernel initialized. Kernel API listening on http://{addr}");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    drop(kernel);
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt())
            .expect("failed to install SIGINT handler");
        tokio::select! {
            _ = sigterm.recv() => {}
            _ = sigint.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
    println!("Shutting down kernel API");
}
