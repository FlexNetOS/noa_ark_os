use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use relocation_daemon::{ExecutionMode, RelocationDaemon};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::signal;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let policy = env_path(
        "RELOCATION_POLICY_PATH",
        ".workspace/config/relocation_policies.yaml",
    );
    let registry = env_path("RELOCATION_REGISTRY_PATH", ".workspace/registry/files.json");
    let backups = env_path("RELOCATION_BACKUPS_PATH", ".workspace/backups");
    let addr: SocketAddr = std::env::var("RELOCATION_SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .context("failed to parse RELOCATION_SERVER_ADDR")?;

    let daemon = RelocationDaemon::new(policy, registry, backups)
        .await
        .context("failed to initialise relocation daemon")?;

    let state = Arc::new(AppState { daemon });
    let service = make_service_fn(move |_| {
        let state = state.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let state = state.clone();
                async move { handle_request(req, state).await }
            }))
        }
    });

    info!(address = %addr, "starting relocation server");

    Server::bind(&addr)
        .serve(service)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    daemon: RelocationDaemon,
}

#[derive(Debug, Deserialize)]
struct RunRequest {
    mode: Option<ExecutionMode>,
}

#[derive(Debug, Deserialize)]
struct ApproveRequest {
    action_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct OverrideRequest {
    source: String,
    destination: String,
    #[serde(default)]
    force: bool,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn handle_request(
    req: Request<Body>,
    state: Arc<AppState>,
) -> Result<Response<Body>, Infallible> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    let response = match (method, path.as_str()) {
        (Method::GET, "/healthz") => {
            json_response(&HealthResponse { status: "ok" }, StatusCode::OK)
        }
        (Method::GET, "/relocation/status") => match state.daemon.status().await {
            Ok(snapshot) => json_response(&snapshot, StatusCode::OK),
            Err(err) => internal_error(err),
        },
        (Method::POST, "/relocation/run") => {
            match parse_json::<RunRequest>(req.into_body()).await {
                Ok(payload) => match state
                    .daemon
                    .run(payload.mode.unwrap_or(ExecutionMode::Approval))
                    .await
                {
                    Ok(report) => json_response(&report, StatusCode::OK),
                    Err(err) => internal_error(err),
                },
                Err(resp) => resp,
            }
        }
        (Method::POST, "/relocation/approve") => {
            match parse_json::<ApproveRequest>(req.into_body()).await {
                Ok(payload) => match state.daemon.approve_action(payload.action_id).await {
                    Ok(Some(result)) => json_response(&result, StatusCode::OK),
                    Ok(None) => json_error(
                        StatusCode::NOT_FOUND,
                        format!("pending action {} not found", payload.action_id),
                    ),
                    Err(err) => internal_error(err),
                },
                Err(resp) => resp,
            }
        }
        (Method::POST, "/relocation/override") => {
            match parse_json::<OverrideRequest>(req.into_body()).await {
                Ok(payload) => match state
                    .daemon
                    .manual_override(payload.source, payload.destination, payload.force)
                    .await
                {
                    Ok(result) => json_response(&result, StatusCode::OK),
                    Err(err) => internal_error(err),
                },
                Err(resp) => resp,
            }
        }
        _ => json_error(StatusCode::NOT_FOUND, "not found"),
    };

    Ok(response)
}

fn env_path(key: &str, default: &str) -> PathBuf {
    std::env::var(key)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(default))
}

async fn shutdown_signal() {
    if let Err(err) = signal::ctrl_c().await {
        warn!("failed to listen for shutdown signal: {}", err);
    }
}

async fn parse_json<T>(body: Body) -> Result<T, Response<Body>>
where
    T: DeserializeOwned,
{
    let bytes = to_bytes(body).await.map_err(|err| {
        json_error(
            StatusCode::BAD_REQUEST,
            format!("failed to read request body: {}", err),
        )
    })?;

    serde_json::from_slice(&bytes).map_err(|err| {
        json_error(
            StatusCode::BAD_REQUEST,
            format!("invalid JSON payload: {}", err),
        )
    })
}

fn json_response<T>(value: &T, status: StatusCode) -> Response<Body>
where
    T: Serialize,
{
    match serde_json::to_vec(value) {
        Ok(body) => Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap_or_else(|err| json_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
        Err(err) => json_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

fn json_error(status: StatusCode, message: impl Into<String>) -> Response<Body> {
    let body = serde_json::json!({ "error": message.into() });
    let encoded = serde_json::to_vec(&body).unwrap_or_else(|_| b"{}".to_vec());
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(encoded))
        .unwrap_or_else(|err| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(err.to_string()))
                .unwrap()
        })
}

fn internal_error(err: anyhow::Error) -> Response<Body> {
    json_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
