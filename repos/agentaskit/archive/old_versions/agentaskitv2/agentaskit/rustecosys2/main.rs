use axum::{
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("ark_os_api_server=debug,tower_http=debug")
        .init();

    info!("Starting Ark OS API Server");

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/status", get(get_status))
        .route("/api/v1/tasks", get(list_tasks))
        .layer(CorsLayer::permissive());

    // Run it with hyper on localhost:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("API server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Result<Json<Value>, StatusCode> {
    info!("Health check requested");
    Ok(Json(json!({
        "status": "ok",
        "service": "ark-os-api-server",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

async fn get_status() -> Result<Json<Value>, StatusCode> {
    info!("Status check requested");
    Ok(Json(json!({
        "service": "ark-os-api-server",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": "running",
        "components": {
            "core": "healthy",
            "database": "not_configured",
            "ml_engine": "not_configured"
        }
    })))
}

async fn list_tasks() -> Result<Json<Value>, StatusCode> {
    info!("Task list requested");
    // TODO: Integrate with actual task storage
    Ok(Json(json!({
        "tasks": [],
        "total": 0,
        "message": "Task storage not yet implemented"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let app = Router::new().route("/health", get(health_check));

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_status_endpoint() {
        let app = Router::new().route("/api/v1/status", get(get_status));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
