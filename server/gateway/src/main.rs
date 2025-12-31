use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use noa_core::security::Permission;
use noa_gateway::{
    bootstrap_gateway, AuthCredentials, Gateway, GatewayRequest, GatewayResponse, Protocol,
};
use noa_observability::{self as observability, LogFormat, MetricsExporter, TracingConfig};
use noa_server_core::config::{self, ConfigOverrides, ServerConfig};
use redis::Client as RedisClient;
use serde::Deserialize;
use serde_json::Value;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use tokio::{net::TcpListener, signal};
use tracing::{error, info};
use url::Url;
use uuid::Uuid;

#[derive(Parser, Debug, Clone)]
#[command(name = "noa-gateway", about = "Unified NOA gateway binary")]
struct GatewayCli {
    #[arg(long)]
    config: Option<std::path::PathBuf>,
    #[arg(long)]
    profile: Option<String>,
    #[arg(long)]
    host: Option<String>,
    #[arg(long)]
    port: Option<u16>,
    #[arg(long)]
    workers: Option<usize>,
    #[arg(long = "metrics-bind")]
    metrics_bind: Option<String>,
    #[arg(long = "metrics-port")]
    metrics_port: Option<u16>,
    #[arg(long = "log-level")]
    log_level: Option<String>,
    #[arg(long = "log-format")]
    log_format: Option<String>,
    #[arg(long = "otlp-endpoint")]
    otlp_endpoint: Option<String>,
}

impl GatewayCli {
    fn overrides(&self) -> ConfigOverrides {
        ConfigOverrides {
            config_path: self.config.clone(),
            profile: self.profile.clone(),
            server_host: self.host.clone(),
            server_port: self.port,
            server_workers: self.workers,
            metrics_bind: self.metrics_bind.clone(),
            metrics_port: self.metrics_port,
            log_level: self.log_level.clone(),
            log_format: self.log_format.clone(),
            otlp_endpoint: self.otlp_endpoint.clone(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = GatewayCli::parse();
    let server_config =
        config::load(cli.overrides()).context("failed to load server configuration")?;

    let log_format = if let Some(fmt_str) = cli.log_format.clone() {
        LogFormat::from_str(&fmt_str)?
    } else {
        LogFormat::from_str(&server_config.observability.log_format)?
    };
    let log_level = cli
        .log_level
        .clone()
        .unwrap_or_else(|| server_config.observability.log_level.clone());
    let otlp_endpoint = cli
        .otlp_endpoint
        .clone()
        .or_else(|| server_config.observability.otlp_endpoint.clone());

    let tracing_config = TracingConfig {
        service_name: "noa-gateway".into(),
        log_format,
        log_level,
        otlp_endpoint,
        resource_attributes: vec![("component".into(), "gateway".into())],
    };
    let (_tracing_guard, metrics_exporter) = observability::init(&tracing_config, None)?;

    let gateway = Arc::new(bootstrap_gateway().context("failed to bootstrap gateway")?);
    let dependencies = Arc::new(DependencyClients::initialise(&server_config)?);
    let readiness = Arc::new(ReadinessState::default());
    readiness.mark_ready();

    let state = AppState {
        gateway,
        metrics: metrics_exporter.clone(),
        readiness: readiness.clone(),
        dependencies,
    };

    let router = Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .route("/ready", get(readiness_probe))
        .route("/metrics", get(metrics_handler))
        .route("/v1/route", post(gateway_entrypoint))
        .with_state(state.clone());

    let addr = server_config
        .server
        .bind_address()
        .context("invalid server bind address")?;
    let metrics_addr = server_config
        .metrics_addr()
        .context("invalid metrics bind address")?;

    if metrics_addr != addr {
        let metrics_state = state.clone();
        let metrics_router = Router::new()
            .route("/metrics", get(metrics_handler))
            .with_state(metrics_state);
        let metrics_listener = TcpListener::bind(metrics_addr)
            .await
            .with_context(|| format!("failed to bind metrics endpoint on {metrics_addr}"))?;
        tokio::spawn(async move {
            info!(?metrics_addr, "starting dedicated metrics listener");
            if let Err(err) =
                axum::serve(metrics_listener, metrics_router.into_make_service()).await
            {
                error!(?err, "metrics server terminated");
            }
        });
    }
    if let Some(tls) = load_rustls(server_config.server.tls.as_ref()).await? {
        info!(?addr, "starting TLS gateway server");
        let handle = axum_server::Handle::new();
        let shutdown_handle = handle.clone();
        tokio::spawn(async move {
            shutdown_signal().await;
            shutdown_handle.graceful_shutdown(None);
        });
        axum_server::bind_rustls(addr, tls)
            .handle(handle)
            .serve(router.into_make_service())
            .await
            .context("gateway server exited")?;
    } else {
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("failed to bind gateway address {addr}"))?;
        info!(?addr, "starting HTTP gateway server");
        axum::serve(listener, router.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await
            .context("gateway server exited")?;
    }

    Ok(())
}

async fn readiness_probe(State(state): State<AppState>) -> impl IntoResponse {
    if state.readiness.is_ready() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.render();
    let headers = [(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; version=1.0.0; charset=utf-8"),
    )];
    (StatusCode::OK, headers, body)
}

async fn gateway_entrypoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<GatewayHttpRequest>,
) -> Result<Json<GatewayResponse>, GatewayHttpError> {
    let capability_scope = payload
        .capability_scope
        .clone()
        .or_else(|| header_value(&headers, "x-noa-capability-scope"));
    let capability_token = header_value(&headers, "x-noa-capability");
    enforce_capability_token(capability_token, capability_scope.as_deref())?;

    let credentials = credentials_from_headers(&headers);
    let user_id = payload
        .user_id
        .or_else(|| header_value(&headers, "x-noa-user-id").and_then(|v| v.parse().ok()))
        .unwrap_or_default();
    let permission = payload
        .required_permission
        .as_deref()
        .and_then(permission_from_str)
        .unwrap_or(Permission::Read);

    let request = GatewayRequest {
        request_id: payload
            .request_id
            .unwrap_or_else(|| Uuid::new_v4().to_string()),
        user_id,
        agent_id: payload.agent_id.clone(),
        credentials,
        protocol: payload.protocol.clone(),
        payload: payload.payload.clone(),
        required_permission: permission,
    };

    let response = state
        .gateway
        .handle_request(request)
        .map_err(|err| GatewayHttpError::internal(err.to_string()))?;

    Ok(Json(response))
}

fn enforce_capability_token(
    token: Option<String>,
    scope: Option<&str>,
) -> Result<(), GatewayHttpError> {
    let token = token.ok_or_else(|| GatewayHttpError::unauthorised("missing capability token"))?;
    let scope = scope.ok_or_else(|| GatewayHttpError::bad_request("missing capability scope"))?;
    noa_core::token::service()
        .validate(&token, scope)
        .map_err(|err| {
            GatewayHttpError::unauthorised(format!("invalid capability token: {err}"))
        })?;
    Ok(())
}

fn credentials_from_headers(headers: &HeaderMap) -> AuthCredentials {
    let oidc = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value
                .strip_prefix("Bearer ")
                .or_else(|| value.strip_prefix("bearer "))
        })
        .map(|token| token.to_string());
    let api_key = header_value(headers, "x-noa-api-key");
    AuthCredentials {
        mtls: None,
        oidc,
        api_key,
    }
}

#[derive(Debug, Deserialize)]
struct GatewayHttpRequest {
    #[serde(default)]
    request_id: Option<String>,
    #[serde(default)]
    user_id: Option<u64>,
    #[serde(default)]
    agent_id: Option<String>,
    protocol: Protocol,
    payload: Value,
    #[serde(default)]
    required_permission: Option<String>,
    #[serde(default)]
    capability_scope: Option<String>,
}

#[derive(Clone)]
struct AppState {
    gateway: Arc<Gateway>,
    metrics: MetricsExporter,
    readiness: Arc<ReadinessState>,
    #[allow(dead_code)]
    dependencies: Arc<DependencyClients>,
}

#[derive(Default)]
struct ReadinessState {
    ready: AtomicBool,
}

impl Clone for ReadinessState {
    fn clone(&self) -> Self {
        Self {
            ready: AtomicBool::new(self.ready.load(Ordering::Relaxed)),
        }
    }
}

impl ReadinessState {
    fn mark_ready(&self) {
        self.ready.store(true, Ordering::Release);
    }

    fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }
}

#[derive(Clone)]
struct DependencyClients {
    #[allow(dead_code)]
    postgres: sqlx::Pool<sqlx::Postgres>,
    #[allow(dead_code)]
    redis: RedisClient,
    #[allow(dead_code)]
    qdrant: QdrantHandle,
}

impl DependencyClients {
    fn initialise(config: &ServerConfig) -> Result<Self> {
        let pg_options = PgConnectOptions::from_str(&config.database.url)
            .context("invalid postgres connection string")?;
        let postgres = PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .connect_lazy_with(pg_options);
        let redis = RedisClient::open(config.cache.url.as_str()).context("invalid redis URL")?;
        let qdrant = QdrantHandle::new(&config.qdrant.url, config.qdrant.api_key.clone())?;
        Ok(Self {
            postgres,
            redis,
            qdrant,
        })
    }
}

#[derive(Clone)]
struct QdrantHandle {
    #[allow(dead_code)]
    endpoint: Url,
    #[allow(dead_code)]
    api_key: Option<String>,
}

impl QdrantHandle {
    fn new(url: &str, api_key: Option<String>) -> Result<Self> {
        let endpoint = Url::parse(url).context("invalid Qdrant endpoint")?;
        Ok(Self { endpoint, api_key })
    }
}

struct GatewayHttpError {
    status: StatusCode,
    message: String,
}

impl GatewayHttpError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn unauthorised(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

impl IntoResponse for GatewayHttpError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": self.message,
        });
        (self.status, Json(body)).into_response()
    }
}

fn permission_from_str(value: &str) -> Option<Permission> {
    match value.to_ascii_lowercase().as_str() {
        "read" => Some(Permission::Read),
        "write" => Some(Permission::Write),
        "execute" => Some(Permission::Execute),
        "admin" => Some(Permission::Admin),
        _ => None,
    }
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|v| v.to_string())
}

async fn load_rustls(tls: Option<&config::ServerTlsConfig>) -> Result<Option<RustlsConfig>> {
    if let Some(tls) = tls {
        if let (Some(cert), Some(key)) = (&tls.cert_path, &tls.key_path) {
            let config = RustlsConfig::from_pem_file(cert.clone(), key.clone())
                .await
                .context("failed to load TLS certs")?;
            return Ok(Some(config));
        }
    }
    Ok(None)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        sigterm.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_permissions_case_insensitively() {
        assert_eq!(permission_from_str("read"), Some(Permission::Read));
        assert_eq!(permission_from_str("WRITE"), Some(Permission::Write));
        assert_eq!(permission_from_str("Execute"), Some(Permission::Execute));
        assert_eq!(permission_from_str("ADMIN"), Some(Permission::Admin));
        assert_eq!(permission_from_str("unknown"), None);
    }

    #[test]
    fn parses_log_format_variants() {
        assert_eq!(LogFormat::from_str("json").unwrap(), LogFormat::Json);
        assert_eq!(LogFormat::from_str("pretty").unwrap(), LogFormat::Pretty);
        assert!(LogFormat::from_str("??").is_err());
    }
}
