use crate::proto::inference_service_server::{InferenceService, InferenceServiceServer};
use crate::proto::orchestration_service_server::{
    OrchestrationService, OrchestrationServiceServer,
};
use crate::proto::retrieval_service_server::{RetrievalService, RetrievalServiceServer};
use crate::proto::{
    InferenceRequest, InferenceResponse, OrchestrationRequest, OrchestrationResponse,
    RetrievalRequest, RetrievalResponse, RoutedPlan,
};
use crate::ApiState;
use axum::body::Body;
use axum::BoxError;
use http_body_util::BodyExt;
use hyper::http::{header, StatusCode};
use hyper::{Request, Response};
use metrics::counter;
use noa_gateway::{Protocol, RoutePlan};
use serde_json::{json, Value};
use std::convert::Infallible;
use tonic::body::BoxBody as TonicBody;
use tonic::{async_trait, Request as GrpcRequest, Response as GrpcResponse, Status};
use tower::util::BoxCloneService;
use tower::{Service, ServiceExt};
use uuid::Uuid;

pub fn build_grpc_service(
    state: ApiState,
) -> BoxCloneService<Request<Body>, Response<Body>, Infallible> {
    let handler = GrpcHandler::new(state);
    let inner = tonic::transport::Server::builder()
        .add_service(InferenceServiceServer::new(handler.clone()))
        .add_service(RetrievalServiceServer::new(handler.clone()))
        .add_service(OrchestrationServiceServer::new(handler))
        .into_service()
        .map_request(|req: Request<Body>| req.map(axum_body_into_tonic))
        .map_response(|res: Response<TonicBody>| res.map(tonic_body_into_axum));

    let svc = tower::service_fn(move |req: Request<Body>| {
        let mut service = inner.clone();
        async move {
            match service.call(req).await {
                Ok(response) => Ok(response),
                Err(err) => {
                    tracing::error!(?err, "grpc service failed");
                    Ok(grpc_error_response())
                }
            }
        }
    });

    BoxCloneService::new(svc)
}

#[derive(Clone)]
struct GrpcHandler {
    state: ApiState,
}

fn axum_body_into_tonic(body: Body) -> TonicBody {
    body.map_err(|err| Status::internal(err.to_string()))
        .boxed_unsync()
}

fn tonic_body_into_axum(body: TonicBody) -> Body {
    Body::new(body.map_err(|status| -> BoxError { Box::new(status) }))
}

fn grpc_error_response() -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "application/grpc")
        .header("grpc-status", "13")
        .body(Body::empty())
        .expect("grpc error response")
}

impl GrpcHandler {
    fn new(state: ApiState) -> Self {
        Self { state }
    }

    fn parse_protocol(&self, value: &str, fallback: Protocol) -> Protocol {
        match value.to_ascii_lowercase().as_str() {
            "graphql" => Protocol::GraphQl,
            "grpc" => Protocol::Grpc,
            "websocket" => Protocol::WebSocket,
            _ => fallback,
        }
    }

    fn parse_metadata(&self, raw: &str) -> Value {
        if raw.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(raw).unwrap_or(Value::Null)
        }
    }

    fn encode_plan(&self, plan: RoutePlan) -> RoutedPlan {
        RoutedPlan {
            protocol: format!("{:?}", plan.protocol),
            targets: plan.targets,
            metadata_json: serde_json::to_string(&plan.metadata).unwrap_or_default(),
        }
    }

    fn route(&self, protocol: Protocol, payload: Value) -> Result<RoutePlan, Status> {
        self.state
            .route(protocol, payload)
            .map_err(|err| Status::invalid_argument(format!("routing error: {err}")))
    }
}

#[async_trait]
impl InferenceService for GrpcHandler {
    async fn invoke(
        &self,
        request: GrpcRequest<InferenceRequest>,
    ) -> Result<GrpcResponse<InferenceResponse>, Status> {
        let req = request.into_inner();
        counter!("api_requests_total", 1, "endpoint" => "grpc_inference");
        let protocol = self.parse_protocol(&req.protocol, Protocol::Grpc);
        let payload = json!({
            "prompt": req.prompt,
            "metadata": self.parse_metadata(&req.metadata_json),
            "service": "inference",
            "method": "Invoke",
        });
        let plan = self.route(protocol, payload)?;
        Ok(GrpcResponse::new(InferenceResponse {
            request_id: Uuid::new_v4().to_string(),
            plan: Some(self.encode_plan(plan)),
            status: "accepted".into(),
            note: "inference gRPC routed".into(),
        }))
    }
}

#[async_trait]
impl RetrievalService for GrpcHandler {
    async fn retrieve(
        &self,
        request: GrpcRequest<RetrievalRequest>,
    ) -> Result<GrpcResponse<RetrievalResponse>, Status> {
        let req = request.into_inner();
        counter!("api_requests_total", 1, "endpoint" => "grpc_retrieval");
        let protocol = self.parse_protocol(&req.protocol, Protocol::GraphQl);
        let payload = json!({
            "query": req.query,
            "metadata": self.parse_metadata(&req.metadata_json),
            "service": "retrieval",
            "federation": { "services": ["retrieval"] },
        });
        let plan = self.route(protocol, payload)?;
        Ok(GrpcResponse::new(RetrievalResponse {
            request_id: Uuid::new_v4().to_string(),
            plan: Some(self.encode_plan(plan)),
            status: "accepted".into(),
            note: "retrieval gRPC routed".into(),
        }))
    }
}

#[async_trait]
impl OrchestrationService for GrpcHandler {
    async fn execute(
        &self,
        request: GrpcRequest<OrchestrationRequest>,
    ) -> Result<GrpcResponse<OrchestrationResponse>, Status> {
        let req = request.into_inner();
        counter!("api_requests_total", 1, "endpoint" => "grpc_orchestration");
        let protocol = self.parse_protocol(&req.protocol, Protocol::GraphQl);
        let payload = json!({
            "task": req.task,
            "metadata": self.parse_metadata(&req.metadata_json),
            "service": "orchestration",
            "federation": { "services": ["orchestration"] },
        });
        let plan = self.route(protocol, payload)?;
        Ok(GrpcResponse::new(OrchestrationResponse {
            request_id: Uuid::new_v4().to_string(),
            plan: Some(self.encode_plan(plan)),
            status: "accepted".into(),
            note: "orchestration gRPC routed".into(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noa_gateway::ProgrammableRouter;

    fn handler() -> GrpcHandler {
        let router = ProgrammableRouter::new(
            vec!["retrieval".into(), "orchestration".into()],
            vec!["inference".into()],
            vec!["agent-activity".into()],
        );
        GrpcHandler::new(ApiState::for_tests(router))
    }

    #[tokio::test]
    async fn inference_grpc_routes_request() {
        let handler = handler();
        let response = handler
            .invoke(GrpcRequest::new(InferenceRequest {
                prompt: "explain".into(),
                protocol: String::new(),
                metadata_json: "{\"temperature\":0.2}".into(),
            }))
            .await
            .expect("gRPC inference succeeds")
            .into_inner();
        let plan = response.plan.expect("plan exists");
        assert_eq!(plan.targets, vec![String::from("inference/Invoke")]);
        assert!(plan.metadata_json.contains("proxy"));
    }

    #[tokio::test]
    async fn retrieval_grpc_sets_federated_mode() {
        let handler = handler();
        let response = handler
            .retrieve(GrpcRequest::new(RetrievalRequest {
                query: "{ doc }".into(),
                protocol: String::new(),
                metadata_json: String::new(),
            }))
            .await
            .expect("gRPC retrieval succeeds")
            .into_inner();
        let plan = response.plan.expect("plan exists");
        assert_eq!(plan.targets, vec![String::from("retrieval")]);
        assert!(plan.metadata_json.contains("federated"));
    }
}
