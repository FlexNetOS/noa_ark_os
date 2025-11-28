mod grpc;
mod routes;

pub mod proto {
    tonic::include_proto!("noa.api.v1");
}

use crate::grpc::build_grpc_service;
use crate::routes::build_http_router;
use anyhow::{anyhow, Context, Result};
use axum::body::Body;
use hyper::{Request, Response};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use noa_gateway::{ProgrammableRouter, Protocol, RoutePlan};
use routes::ApiRoutes;
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::signal;
use tower::util::BoxCloneService;
use tracing::info;

/// Configuration controlling how the API server binds.
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8080,
        }
    }
}

#[derive(Clone)]
pub struct ApiState {
    inner: Arc<ApiStateInner>,
}

struct ApiStateInner {
    router: ProgrammableRouter,
    metrics: MetricsHandle,
    ready: AtomicBool,
    started_at: Instant,
}

#[derive(Clone)]
struct MetricsHandle {
    handle: Arc<PrometheusHandle>,
}

impl MetricsHandle {
    fn install() -> Result<Self> {
        static PROM_HANDLE: OnceLock<Arc<PrometheusHandle>> = OnceLock::new();
        static INSTALL_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

        if let Some(handle) = PROM_HANDLE.get() {
            return Ok(Self {
                handle: handle.clone(),
            });
        }

        let lock = INSTALL_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("metrics install lock poisoned");

        if let Some(handle) = PROM_HANDLE.get() {
            drop(lock);
            return Ok(Self {
                handle: handle.clone(),
            });
        }

        let handle = Arc::new(
            PrometheusBuilder::new()
                .install_recorder()
                .context("failed to install Prometheus recorder")?,
        );
        let _ = PROM_HANDLE.set(handle.clone());
        drop(lock);
        Ok(Self { handle })
    }

    fn render(&self) -> String {
        self.handle.render()
    }
}

impl ApiState {
    pub(crate) fn new(router: ProgrammableRouter, metrics: MetricsHandle) -> Self {
        Self {
            inner: Arc::new(ApiStateInner {
                router,
                metrics,
                ready: AtomicBool::new(false),
                started_at: Instant::now(),
            }),
        }
    }

    pub(crate) fn metrics(&self) -> &MetricsHandle {
        &self.inner.metrics
    }

    pub fn mark_ready(&self) {
        self.inner.ready.store(true, Ordering::SeqCst);
    }

    pub fn is_ready(&self) -> bool {
        self.inner.ready.load(Ordering::SeqCst)
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.inner.started_at.elapsed().as_secs()
    }

    pub fn route(&self, protocol: Protocol, payload: Value) -> Result<RoutePlan> {
        self.inner
            .router
            .route(&protocol, &payload)
            .map_err(|err| anyhow!("route computation failed: {err}"))
    }

    #[cfg(test)]
    pub(crate) fn for_tests(router: ProgrammableRouter) -> Self {
        let metrics = MetricsHandle::install().expect("metrics recorder installed for tests");
        Self::new(router, metrics)
    }
}

/// Axum + Tonic server wrapper.
pub struct ApiServer {
    config: ApiConfig,
    state: ApiState,
}

impl ApiServer {
    pub fn new(config: ApiConfig) -> Result<Self> {
        let router = ProgrammableRouter::default();
        let metrics = MetricsHandle::install().context("failed to install metrics exporter")?;
        Ok(Self {
            config,
            state: ApiState::new(router, metrics),
        })
    }

    pub async fn run(self) -> Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .context("invalid bind address")?;
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("failed to bind {addr}"))?;

        let http_router = build_http_router(ApiRoutes::new(self.state.clone()));
        let grpc_service: BoxCloneService<Request<Body>, Response<Body>, std::convert::Infallible> =
            build_grpc_service(self.state.clone());

        let app = http_router.fallback_service(grpc_service);
        self.state.mark_ready();
        info!(?addr, "API server listening");

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .context("api server terminated with error")
    }
}

async fn shutdown_signal() {
    if let Err(err) = signal::ctrl_c().await {
        tracing::warn!(?err, "ctrl-c listener failed");
    }
}
