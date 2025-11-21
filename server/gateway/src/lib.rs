//! NOA Gateway - programmable, policy-enforced multi-protocol entrypoint
//!
//! This crate models the high-level orchestration of the application gateway used
//! across NOA ARK OS. The design emphasises:
//! - Programmable routing covering GraphQL federation, gRPC proxying, and WebSocket multiplexing.
//! - Unified authentication & authorisation that leverages the core security subsystem.
//! - Rate limiting tied to agent/service identities sourced from the hive mind registry.
//! - Distributed tracing and telemetry export compatible with OpenTelemetry pipelines.
//!
//! The implementation intentionally focuses on deterministic, testable behaviour
//! so it can run in CI without external infrastructure.

mod auth;
mod policy;
mod rate_limit;
mod router;
mod telemetry;

pub use auth::{AuthCredentials, UnifiedAuthenticator};
pub use policy::{GatewayPolicy, PolicyEnforcer};
pub use rate_limit::{RateLimiter, RateLimiterConfig};
pub use router::{ProgrammableRouter, Protocol, RoutePlan, RoutingError};
pub use telemetry::{GatewayMetrics, TelemetryEvent, TelemetrySink};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use noa_agents::registry::AgentRegistry;
use noa_core::security::{self, Permission};
use serde::Serialize;
use std::sync::Arc;
use tracing::instrument;

/// High-level request entering the gateway.
#[derive(Debug, Clone)]
pub struct GatewayRequest {
    pub request_id: String,
    pub user_id: security::UserId,
    pub agent_id: Option<String>,
    pub credentials: AuthCredentials,
    pub protocol: Protocol,
    pub payload: serde_json::Value,
    pub required_permission: Permission,
}

/// Simplified response emitted by the gateway after routing.
#[derive(Debug, Clone, Serialize)]
pub struct GatewayResponse {
    pub request_id: String,
    pub route_plan: RoutePlan,
    pub policy_enforced: bool,
    pub timestamp: DateTime<Utc>,
}

/// Core orchestrator wiring all gateway subsystems together.
pub struct Gateway {
    authenticator: UnifiedAuthenticator,
    policy: PolicyEnforcer,
    router: ProgrammableRouter,
    rate_limiter: RateLimiter,
    telemetry: TelemetrySink,
}

impl Gateway {
    /// Create a new gateway instance with sensible defaults.
    pub fn new(
        authenticator: UnifiedAuthenticator,
        policy: PolicyEnforcer,
        router: ProgrammableRouter,
        rate_limiter: RateLimiter,
        telemetry: TelemetrySink,
    ) -> Result<Self> {
        Ok(Self {
            authenticator,
            policy,
            router,
            rate_limiter,
            telemetry,
        })
    }

    /// Helper constructor that loads the shared agent registry and builds supporting components.
    pub fn with_defaults(registry: Arc<AgentRegistry>, telemetry: TelemetrySink) -> Result<Self> {
        let authenticator = UnifiedAuthenticator::default();
        let policy = PolicyEnforcer::new();
        let router = ProgrammableRouter::default();
        let rate_limiter = RateLimiter::new(RateLimiterConfig::default(), registry);
        Self::new(authenticator, policy, router, rate_limiter, telemetry)
    }

    /// Handle an incoming request by applying authN/Z, rate limiting, routing and telemetry.
    #[instrument(skip(self))]
    pub fn handle_request(&self, request: GatewayRequest) -> Result<GatewayResponse> {
        // Step 1 - authenticate
        self.authenticator
            .verify(&request.credentials, &request.agent_id)
            .context("authentication failed")?;

        // Step 2 - authorise via core security policies
        self.policy
            .enforce(request.user_id, request.required_permission)
            .context("policy enforcement failure")?;

        // Step 3 - enforce rate limits for the linked agent/service
        self.rate_limiter
            .check(&request.agent_id)
            .context("rate limit exceeded")?;

        // Step 4 - compute programmable route plan
        let route_plan = self.router.route(&request.protocol, &request.payload)?;

        // Step 5 - emit telemetry covering traces + metrics snapshot
        self.telemetry.record(TelemetryEvent::new(
            request.request_id.clone(),
            request.protocol.clone(),
            route_plan.clone(),
            request.agent_id.clone(),
        ))?;

        Ok(GatewayResponse {
            request_id: request.request_id,
            route_plan,
            policy_enforced: true,
            timestamp: Utc::now(),
        })
    }
}

/// Build a production-like gateway composed of workspace primitives.
pub fn bootstrap_gateway() -> Result<Gateway> {
    // Ensure the security subsystem is initialised so policy checks work.
    security::init().map_err(|err| anyhow!("failed to init security: {}", err))?;

    // Load agent metadata to power identity aware rate limiting.
    let registry =
        Arc::new(AgentRegistry::with_default_data().context("failed to load agent registry")?);

    let telemetry = TelemetrySink::default();
    Gateway::with_defaults(registry, telemetry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rate_limit::RateLimitError;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::tempdir;

    fn gateway_with_tempdir() -> (Gateway, tempfile::TempDir) {
        let _ = security::init();
        let registry =
            Arc::new(AgentRegistry::with_default_data().expect("agent registry should load"));
        let tempdir = tempdir().expect("tempdir");
        let telemetry = TelemetrySink::new(tempdir.path()).expect("telemetry sink");
        let gateway = Gateway::with_defaults(registry, telemetry).expect("gateway bootstrap");
        (gateway, tempdir)
    }

    #[test]
    fn handles_graphql_with_policy_and_rate_limiting() {
        let (gateway, _tmp) = gateway_with_tempdir();

        let request = GatewayRequest {
            request_id: "req-graphql".into(),
            user_id: 0,
            agent_id: Some("fixed_agent_gateway".into()),
            credentials: AuthCredentials {
                mtls: Some("agent-cert".into()),
                oidc: Some("id-token-verified".into()),
                api_key: Some("key-123".into()),
            },
            protocol: Protocol::GraphQl,
            payload: json!({
                "query": "{ serviceA { id name } }",
                "federation": {
                    "services": ["serviceA", "serviceB"],
                    "version": "1.0",
                }
            }),
            required_permission: Permission::Read,
        };

        let response = gateway.handle_request(request).expect("graphql request");
        assert_eq!(response.route_plan.protocol, Protocol::GraphQl);
        assert!(response.policy_enforced);
    }

    #[test]
    fn rejects_missing_authentication() {
        let (gateway, _tmp) = gateway_with_tempdir();

        let request = GatewayRequest {
            request_id: "req-fail".into(),
            user_id: 0,
            agent_id: Some("fixed_agent_gateway".into()),
            credentials: AuthCredentials::default(),
            protocol: Protocol::Grpc,
            payload: serde_json::json!({ "service": "workflow", "method": "Run" }),
            required_permission: Permission::Read,
        };

        let err = gateway.handle_request(request).expect_err("auth failure");
        assert!(err.to_string().contains("authentication failed"));
    }

    #[test]
    fn rate_limiting_blocks_after_token_bucket_exhaustion() {
        let _ = security::init();
        let registry =
            Arc::new(AgentRegistry::with_default_data().expect("agent registry should load"));
        let agent = registry
            .all()
            .into_iter()
            .next()
            .expect("at least one agent available");

        let mut layer_limits = HashMap::new();
        layer_limits.insert(agent.layer.clone(), 1);

        let limiter = RateLimiter::new(
            RateLimiterConfig {
                refill_interval: Duration::from_secs(60),
                layer_limits,
            },
            registry,
        );

        let agent_id = Some(agent.agent_id.clone());
        assert!(limiter.check(&agent_id).is_ok());
        let err = limiter
            .check(&agent_id)
            .expect_err("second call should exceed configured limit");
        assert!(matches!(err, RateLimitError::LimitExceeded(_)));
    }
}
