use crate::ApiState;
use anyhow::Error;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use metrics::counter;
use noa_gateway::{Protocol, RoutePlan};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::borrow::Cow;
use uuid::Uuid;

#[derive(Clone)]
pub struct ApiRoutes {
    state: ApiState,
}

impl ApiRoutes {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }

    fn record_request(&self, endpoint: &str) {
        counter!("api_requests_total", 1, "endpoint" => endpoint.to_string());
    }

    fn route(&self, protocol: Protocol, payload: Value) -> Result<RoutePlan, ApiError> {
        self.state.route(protocol, payload).map_err(ApiError::from)
    }

    fn new_request_id(&self) -> String {
        Uuid::new_v4().to_string()
    }

    pub fn state(&self) -> &ApiState {
        &self.state
    }
}

pub fn build_http_router(state: ApiRoutes) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/metrics", get(metrics))
        .route("/v1/inference", post(inference))
        .route("/v1/retrieval", post(retrieval))
        .route("/v1/orchestration", post(orchestration))
        .route("/ws/:channel", get(websocket))
        .with_state(state)
}

#[derive(Serialize)]
struct StatusResponse<'a> {
    status: &'a str,
    ready: bool,
    uptime_seconds: u64,
}

async fn health(State(routes): State<ApiRoutes>) -> impl IntoResponse {
    Json(StatusResponse {
        status: "ok",
        ready: routes.state().is_ready(),
        uptime_seconds: routes.state().uptime_seconds(),
    })
}

async fn ready(State(routes): State<ApiRoutes>) -> impl IntoResponse {
    let ready = routes.state().is_ready();
    Json(StatusResponse {
        status: if ready { "ready" } else { "starting" },
        ready,
        uptime_seconds: routes.state().uptime_seconds(),
    })
}

async fn metrics(State(routes): State<ApiRoutes>) -> impl IntoResponse {
    match Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; version=0.0.4")
        .body(routes.state().metrics().render().into())
    {
        Ok(resp) => resp,
        Err(err) => {
            tracing::error!(?err, "failed to build metrics response");
            (StatusCode::INTERNAL_SERVER_ERROR, "metrics exporter error").into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
struct InferenceRestRequest {
    prompt: String,
    #[serde(default)]
    protocol: Option<Protocol>,
    #[serde(default)]
    metadata: Value,
}

#[derive(Debug, Deserialize)]
struct RetrievalRestRequest {
    query: String,
    #[serde(default)]
    protocol: Option<Protocol>,
    #[serde(default)]
    metadata: Value,
}

#[derive(Debug, Deserialize)]
struct OrchestrationRestRequest {
    task: String,
    #[serde(default)]
    protocol: Option<Protocol>,
    #[serde(default)]
    metadata: Value,
}

#[derive(Serialize)]
struct RoutedResponse {
    request_id: String,
    plan: RoutePlan,
    status: &'static str,
    note: String,
}

async fn inference(
    State(routes): State<ApiRoutes>,
    Json(payload): Json<InferenceRestRequest>,
) -> Result<Json<RoutedResponse>, ApiError> {
    routes.record_request("inference");
    let protocol = payload.protocol.unwrap_or(Protocol::Grpc);
    let plan = routes.route(
        protocol,
        json!({
            "prompt": payload.prompt,
            "metadata": payload.metadata,
            "service": "inference",
            "method": "Invoke",
        }),
    )?;
    Ok(Json(RoutedResponse {
        request_id: routes.new_request_id(),
        plan,
        status: "accepted",
        note: "inference request routed".into(),
    }))
}

async fn retrieval(
    State(routes): State<ApiRoutes>,
    Json(payload): Json<RetrievalRestRequest>,
) -> Result<Json<RoutedResponse>, ApiError> {
    routes.record_request("retrieval");
    let protocol = payload.protocol.unwrap_or(Protocol::GraphQl);
    let plan = routes.route(
        protocol,
        json!({
            "query": payload.query,
            "metadata": payload.metadata,
            "service": "retrieval",
            "federation": { "services": ["retrieval"] },
        }),
    )?;
    Ok(Json(RoutedResponse {
        request_id: routes.new_request_id(),
        plan,
        status: "accepted",
        note: "retrieval request routed".into(),
    }))
}

async fn orchestration(
    State(routes): State<ApiRoutes>,
    Json(payload): Json<OrchestrationRestRequest>,
) -> Result<Json<RoutedResponse>, ApiError> {
    routes.record_request("orchestration");
    let protocol = payload.protocol.unwrap_or(Protocol::GraphQl);
    let plan = routes.route(
        protocol,
        json!({
            "task": payload.task,
            "metadata": payload.metadata,
            "service": "orchestration",
            "federation": { "services": ["orchestration"] },
        }),
    )?;
    Ok(Json(RoutedResponse {
        request_id: routes.new_request_id(),
        plan,
        status: "accepted",
        note: "orchestration request routed".into(),
    }))
}

async fn websocket(
    ws: WebSocketUpgrade,
    Path(channel): Path<String>,
    State(routes): State<ApiRoutes>,
) -> Result<impl IntoResponse, ApiError> {
    routes.record_request("websocket");
    let plan = routes.route(
        Protocol::WebSocket,
        json!({
            "channel": channel,
        }),
    )?;
    Ok(ws.on_upgrade(move |socket| websocket_loop(socket, plan)))
}

async fn websocket_loop(mut socket: WebSocket, plan: RoutePlan) {
    let payload = websocket_bootstrap_frame(&plan);
    if socket.send(Message::Text(payload)).await.is_err() {
        tracing::warn!("failed to push websocket bootstrap frame");
    }
}

fn websocket_bootstrap_frame(plan: &RoutePlan) -> String {
    json!({
        "note": "stream initialised",
        "targets": plan.targets,
        "mode": plan.metadata.get("mode").cloned(),
    })
    .to_string()
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: Cow<'static, str>,
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        ApiError::new(StatusCode::BAD_REQUEST, format!("routing error: {err}"))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": self.message,
        }));
        (self.status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use noa_gateway::ProgrammableRouter;
    use serde_json::Value;
    use tower::ServiceExt;

    fn build_test_router() -> Router {
        let router = ProgrammableRouter::new(
            vec!["retrieval".into(), "orchestration".into()],
            vec!["inference".into()],
            vec!["agent-activity".into()],
        );
        let state = ApiState::for_tests(router);
        state.mark_ready();
        build_http_router(ApiRoutes::new(state))
    }

    async fn post_json(router: &Router, path: &str, payload: Value) -> (StatusCode, Value) {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("valid request"),
            )
            .await
            .expect("router responds");
        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body to bytes")
            .to_bytes();
        let value = serde_json::from_slice(&bytes).expect("json body");
        (status, value)
    }

    #[tokio::test]
    async fn readiness_endpoint_reflects_state() {
        let router = build_test_router();
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .expect("ready request"),
            )
            .await
            .expect("ready response");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("read bytes")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&bytes).expect("ready payload");
        assert_eq!(payload["ready"], Value::Bool(true));
    }

    #[tokio::test]
    async fn metrics_endpoint_returns_prometheus_text() {
        let router = build_test_router();
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .expect("metrics request"),
            )
            .await
            .expect("metrics response");
        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(
            headers.get(header::CONTENT_TYPE),
            Some(&header::HeaderValue::from_static(
                "text/plain; version=0.0.4"
            ))
        );
    }

    #[tokio::test]
    async fn inference_route_returns_routing_plan() {
        let router = build_test_router();
        let (status, payload) =
            post_json(&router, "/v1/inference", json!({ "prompt": "hi there" })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["status"], Value::String("accepted".into()));
        assert_eq!(payload["plan"]["targets"], json!(["inference/Invoke"]));
    }

    #[tokio::test]
    async fn retrieval_route_registers_known_service() {
        let router = build_test_router();
        let (status, payload) =
            post_json(&router, "/v1/retrieval", json!({ "query": "{ doc }" })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["plan"]["targets"], json!(["retrieval"]));
    }

    #[tokio::test]
    async fn websocket_frame_includes_mode_and_targets() {
        let mut plan = RoutePlan::new(Protocol::WebSocket);
        plan.targets.push("agent-activity".into());
        plan.metadata
            .insert("mode".into(), Value::String("multiplex".into()));
        let payload = websocket_bootstrap_frame(&plan);
        let parsed: Value = serde_json::from_str(&payload).expect("json payload");
        assert_eq!(parsed["targets"], json!(["agent-activity"]));
        assert_eq!(parsed["mode"], Value::String("multiplex".into()));
    }
}
