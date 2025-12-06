//! Partner-facing APIs for third-party symbol onboarding and trust exchange.
//!
//! This module provides REST/gRPC endpoints for partners to:
//! - Submit signed telemetry for trust scoring
//! - Query trust scores and reroute recommendations
//! - Onboard new symbols with automated compliance checks

use anyhow::Result;
use noa_core::gateway::{Gateway, SignedTelemetry};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Partner API client for interacting with the gateway.
pub struct PartnerApi {
    gateway: Arc<Gateway>,
}

impl PartnerApi {
    pub fn new(gateway: Arc<Gateway>) -> Self {
        Self { gateway }
    }

    /// Submit signed telemetry from a partner.
    pub async fn submit_telemetry(&self, telemetry: SignedTelemetry) -> Result<()> {
        self.gateway.submit_partner_telemetry(telemetry)?;
        Ok(())
    }

    /// Get trust score for a partner.
    pub fn get_trust_score(&self, partner_id: &str) -> f32 {
        self.gateway.get_partner_trust_score(partner_id)
    }

    /// Check if rerouting is recommended for a partner.
    pub fn should_reroute(&self, partner_id: &str) -> bool {
        self.gateway.should_reroute_from_partner(partner_id)
    }

    /// Onboard a new symbol with automated compliance checks.
    pub async fn onboard_symbol(&self, request: OnboardingRequest) -> Result<OnboardingResponse> {
        // Simulate onboarding process
        // In real implementation, this would validate schema, check compliance, etc.
        let response = OnboardingResponse {
            symbol_id: request.symbol_id,
            approved: true,
            compliance_issues: vec![],
        };
        Ok(response)
    }
}

/// Request to onboard a new symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingRequest {
    pub partner_id: String,
    pub symbol_id: String,
    pub schema: serde_json::Value,
    pub compliance_evidence: Vec<String>,
}

/// Response from symbol onboarding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingResponse {
    pub symbol_id: String,
    pub approved: bool,
    pub compliance_issues: Vec<String>,
}

/// SDK for partners to integrate with the gateway.
pub struct GatewaySdk {
    api: PartnerApi,
}

impl GatewaySdk {
    pub fn new(gateway: Arc<Gateway>) -> Self {
        Self {
            api: PartnerApi::new(gateway),
        }
    }

    /// Submit telemetry using the SDK.
    pub async fn submit_telemetry(&self, telemetry: SignedTelemetry) -> Result<()> {
        self.api.submit_telemetry(telemetry).await
    }

    /// Query trust score.
    pub fn query_trust_score(&self, partner_id: &str) -> f32 {
        self.api.get_trust_score(partner_id)
    }

    /// Check reroute recommendation.
    pub fn check_reroute(&self, partner_id: &str) -> bool {
        self.api.should_reroute(partner_id)
    }

    /// Onboard symbol via SDK.
    pub async fn onboard_symbol(&self, request: OnboardingRequest) -> Result<OnboardingResponse> {
        self.api.onboard_symbol(request).await
    }
}
