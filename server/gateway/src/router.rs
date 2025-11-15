use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// Supported protocols by the programmable router.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    GraphQl,
    Grpc,
    WebSocket,
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::GraphQl
    }
}

/// Routing plan describing downstream targets and behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePlan {
    pub protocol: Protocol,
    pub targets: Vec<String>,
    pub metadata: HashMap<String, Value>,
}

impl RoutePlan {
    pub fn new(protocol: Protocol) -> Self {
        Self {
            protocol,
            targets: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum RoutingError {
    #[error("missing field: {0}")]
    MissingField(&'static str),
}

/// Programmable router that understands multiple transport protocols.
#[derive(Debug, Clone)]
pub struct ProgrammableRouter {
    graphql_services: Vec<String>,
    grpc_services: Vec<String>,
    websocket_channels: Vec<String>,
}

impl ProgrammableRouter {
    pub fn new(
        graphql_services: Vec<String>,
        grpc_services: Vec<String>,
        websocket_channels: Vec<String>,
    ) -> Self {
        Self {
            graphql_services,
            grpc_services,
            websocket_channels,
        }
    }

    pub fn route(&self, protocol: &Protocol, payload: &Value) -> Result<RoutePlan, RoutingError> {
        match protocol {
            Protocol::GraphQl => self.route_graphql(payload),
            Protocol::Grpc => self.route_grpc(payload),
            Protocol::WebSocket => self.route_websocket(payload),
        }
    }

    fn route_graphql(&self, payload: &Value) -> Result<RoutePlan, RoutingError> {
        let federation = payload
            .get("federation")
            .ok_or(RoutingError::MissingField("federation"))?;
        let services = federation
            .get("services")
            .and_then(|v| v.as_array())
            .ok_or(RoutingError::MissingField("federation.services"))?;

        let mut plan = RoutePlan::new(Protocol::GraphQl);
        for service in services {
            if let Some(name) = service.as_str() {
                if self.graphql_services.contains(&name.to_string()) {
                    plan.targets.push(name.to_string());
                }
            }
        }
        plan.metadata.insert(
            "query".into(),
            payload.get("query").cloned().unwrap_or(Value::Null),
        );
        plan.metadata
            .insert("mode".into(), Value::String("federated".into()));
        Ok(plan)
    }

    fn route_grpc(&self, payload: &Value) -> Result<RoutePlan, RoutingError> {
        let service = payload
            .get("service")
            .and_then(|v| v.as_str())
            .ok_or(RoutingError::MissingField("service"))?;
        let method = payload
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or(RoutingError::MissingField("method"))?;

        let mut plan = RoutePlan::new(Protocol::Grpc);
        if self.grpc_services.contains(&service.to_string()) {
            plan.targets.push(format!("{service}/{method}"));
        }
        plan.metadata
            .insert("mode".into(), Value::String("proxy".into()));
        Ok(plan)
    }

    fn route_websocket(&self, payload: &Value) -> Result<RoutePlan, RoutingError> {
        let channel = payload
            .get("channel")
            .and_then(|v| v.as_str())
            .ok_or(RoutingError::MissingField("channel"))?;

        let mut plan = RoutePlan::new(Protocol::WebSocket);
        if self.websocket_channels.contains(&channel.to_string()) {
            plan.targets.push(channel.to_string());
        }
        plan.metadata
            .insert("mode".into(), Value::String("multiplex".into()));
        Ok(plan)
    }
}

impl Default for ProgrammableRouter {
    fn default() -> Self {
        Self::new(
            vec!["serviceA".into(), "serviceB".into(), "analytics".into()],
            vec!["workflow".into(), "memory".into(), "security".into()],
            vec![
                "agent-activity".into(),
                "alerts".into(),
                "workflow-status".into(),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn graphql_routing_delegates_known_services() {
        let router =
            ProgrammableRouter::new(vec!["serviceA".into(), "serviceB".into()], vec![], vec![]);
        let payload = json!({
            "query": "{ serviceA { id } }",
            "federation": { "services": ["serviceA", "serviceZ"] },
        });

        let plan = router
            .route(&Protocol::GraphQl, &payload)
            .expect("graphql routing succeeds");

        assert_eq!(plan.protocol, Protocol::GraphQl);
        assert_eq!(plan.targets, vec!["serviceA".to_string()]);
        assert_eq!(
            plan.metadata.get("mode"),
            Some(&Value::String("federated".into())),
        );
    }

    #[test]
    fn grpc_routing_maps_service_and_method() {
        let router = ProgrammableRouter::new(vec![], vec!["workflow".into()], vec![]);
        let payload = json!({
            "service": "workflow",
            "method": "Run",
        });

        let plan = router
            .route(&Protocol::Grpc, &payload)
            .expect("grpc routing succeeds");

        assert_eq!(plan.protocol, Protocol::Grpc);
        assert_eq!(plan.targets, vec!["workflow/Run".to_string()]);
        assert_eq!(
            plan.metadata.get("mode"),
            Some(&Value::String("proxy".into())),
        );
    }
}
