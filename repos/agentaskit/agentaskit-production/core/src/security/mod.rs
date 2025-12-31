use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use ring::{digest, hmac};
use tracing::{info, warn, error, debug};

/// Security capabilities and permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Capability {
    // System-level capabilities
    SystemAdmin,
    AgentManagement,
    TaskOrchestration,
    SecurityManagement,
    MonitoringAccess,
    
    // Agent layer capabilities
    StrategicPlanning,
    PolicyEnforcement,
    OperationalControl,
    DomainExpertise,
    TaskExecution,
    
    // Functional capabilities
    DataAccess,
    NetworkAccess,
    FileSystemAccess,
    DatabaseAccess,
    ExternalAPIAccess,
    
    // Security operations
    AuditLog,
    ComplianceCheck,
    ThreatDetection,
    IncidentResponse,
}

/// Authentication token containing agent credentials and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub capabilities: Vec<Capability>,
    pub issuer: String,
    pub signature: String,
}

impl CapabilityToken {
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }

    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }

    pub fn time_until_expiry(&self) -> Option<Duration> {
        if self.is_valid() {
            Some(self.expires_at - Utc::now())
        } else {
            None
        }
    }
}

/// Access control entry for resource protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlEntry {
    pub resource_id: String,
    pub agent_id: Uuid,
    pub capabilities: Vec<Capability>,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Uuid,
}

/// Audit log entry for security monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub agent_id: Uuid,
    pub action: String,
    pub resource: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
}

/// Security manager for the agent system
pub struct SecurityManager {
    active_tokens: Arc<RwLock<HashMap<Uuid, CapabilityToken>>>,
    access_control_list: Arc<RwLock<HashMap<String, Vec<AccessControlEntry>>>>,
    audit_log: Arc<RwLock<Vec<AuditLogEntry>>>,
    signing_key: hmac::Key,
    token_validity_duration: Duration,
}

impl Clone for SecurityManager {
    fn clone(&self) -> Self {
        Self {
            active_tokens: Arc::clone(&self.active_tokens),
            access_control_list: Arc::clone(&self.access_control_list),
            audit_log: Arc::clone(&self.audit_log),
            signing_key: hmac::Key::new(hmac::HMAC_SHA256, b"agentaskit-security-key"),
            token_validity_duration: self.token_validity_duration,
        }
    }
}

impl SecurityManager {
    pub async fn new() -> Result<Self> {
        // In production, this should use a secure key management system
        let signing_key = hmac::Key::new(hmac::HMAC_SHA256, b"agentaskit-security-key");
        
        Ok(Self {
            active_tokens: Arc::new(RwLock::new(HashMap::new())),
            access_control_list: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            signing_key,
            token_validity_duration: Duration::hours(24),
        })
    }

    /// Issue a capability token for an agent
    pub async fn issue_token(&self, agent_id: Uuid, capabilities: Vec<Capability>) -> Result<CapabilityToken> {
        let token_id = Uuid::new_v4();
        let issued_at = Utc::now();
        let expires_at = issued_at + self.token_validity_duration;

        // Create token data for signing
        let token_data = format!("{}:{}:{}:{:?}", 
            token_id, agent_id, expires_at.timestamp(), capabilities);
        
        // Sign the token
        let signature = hmac::sign(&self.signing_key, token_data.as_bytes());
        let signature_hex = hex::encode(signature.as_ref());

        let token = CapabilityToken {
            id: token_id,
            agent_id,
            issued_at,
            expires_at,
            capabilities,
            issuer: "ARK-OS Security Manager".to_string(),
            signature: signature_hex,
        };

        // Store the token
        self.active_tokens.write().await.insert(token_id, token.clone());

        // Log the issuance
        self.log_security_event(
            agent_id,
            "token_issued".to_string(),
            None,
            true,
            None,
            serde_json::json!({
                "token_id": token_id,
                "capabilities": capabilities,
                "expires_at": expires_at
            }),
        ).await;

        info!("Issued capability token {} for agent {}", token_id, agent_id);
        Ok(token)
    }

    /// Validate a capability token
    pub async fn validate_token(&self, token_id: Uuid) -> Result<CapabilityToken> {
        let tokens = self.active_tokens.read().await;
        
        if let Some(token) = tokens.get(&token_id) {
            if token.is_valid() {
                Ok(token.clone())
            } else {
                // Token expired - remove it
                drop(tokens);
                self.revoke_token(token_id).await?;
                Err(anyhow::anyhow!("Token expired"))
            }
        } else {
            Err(anyhow::anyhow!("Token not found"))
        }
    }

    /// Revoke a capability token
    pub async fn revoke_token(&self, token_id: Uuid) -> Result<()> {
        let mut tokens = self.active_tokens.write().await;
        
        if let Some(token) = tokens.remove(&token_id) {
            self.log_security_event(
                token.agent_id,
                "token_revoked".to_string(),
                None,
                true,
                None,
                serde_json::json!({
                    "token_id": token_id,
                    "reason": "manual_revocation"
                }),
            ).await;
            
            info!("Revoked capability token {}", token_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Token not found"))
        }
    }

    /// Check if an agent has permission to access a resource
    pub async fn check_access(&self, agent_id: Uuid, resource: &str, required_capability: &Capability) -> Result<bool> {
        // Check if agent has an active token with the required capability
        let tokens = self.active_tokens.read().await;
        let agent_token = tokens.values()
            .find(|token| token.agent_id == agent_id && token.is_valid());

        if let Some(token) = agent_token {
            let has_capability = token.has_capability(required_capability);
            
            // Log the access attempt
            self.log_security_event(
                agent_id,
                "access_check".to_string(),
                Some(resource.to_string()),
                has_capability,
                if has_capability { None } else { Some("insufficient_capabilities".to_string()) },
                serde_json::json!({
                    "required_capability": required_capability,
                    "agent_capabilities": token.capabilities
                }),
            ).await;

            Ok(has_capability)
        } else {
            // No valid token found
            self.log_security_event(
                agent_id,
                "access_denied".to_string(),
                Some(resource.to_string()),
                false,
                Some("no_valid_token".to_string()),
                serde_json::json!({
                    "required_capability": required_capability
                }),
            ).await;

            Ok(false)
        }
    }

    /// Grant access to a resource for an agent
    pub async fn grant_access(&self, resource_id: String, agent_id: Uuid, capabilities: Vec<Capability>, granted_by: Uuid) -> Result<()> {
        let access_entry = AccessControlEntry {
            resource_id: resource_id.clone(),
            agent_id,
            capabilities,
            granted_at: Utc::now(),
            granted_by,
        };

        self.access_control_list.write().await
            .entry(resource_id.clone())
            .or_insert_with(Vec::new)
            .push(access_entry);

        self.log_security_event(
            granted_by,
            "access_granted".to_string(),
            Some(resource_id),
            true,
            None,
            serde_json::json!({
                "target_agent": agent_id,
                "capabilities": capabilities
            }),
        ).await;

        info!("Granted access to resource {} for agent {}", resource_id, agent_id);
        Ok(())
    }

    /// Revoke access to a resource for an agent
    pub async fn revoke_access(&self, resource_id: &str, agent_id: Uuid, revoked_by: Uuid) -> Result<()> {
        let mut acl = self.access_control_list.write().await;
        
        if let Some(entries) = acl.get_mut(resource_id) {
            entries.retain(|entry| entry.agent_id != agent_id);
            
            self.log_security_event(
                revoked_by,
                "access_revoked".to_string(),
                Some(resource_id.to_string()),
                true,
                None,
                serde_json::json!({
                    "target_agent": agent_id
                }),
            ).await;

            info!("Revoked access to resource {} for agent {}", resource_id, agent_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Resource not found in ACL"))
        }
    }

    /// Log a security event for audit purposes
    pub async fn log_security_event(
        &self,
        agent_id: Uuid,
        action: String,
        resource: Option<String>,
        success: bool,
        error_message: Option<String>,
        metadata: serde_json::Value,
    ) {
        let entry = AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            agent_id,
            action,
            resource,
            success,
            error_message,
            metadata,
        };

        self.audit_log.write().await.push(entry);
    }

    /// Get audit log entries for analysis
    pub async fn get_audit_log(&self, limit: Option<usize>) -> Vec<AuditLogEntry> {
        let log = self.audit_log.read().await;
        
        if let Some(limit) = limit {
            log.iter().rev().take(limit).cloned().collect()
        } else {
            log.clone()
        }
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired_tokens(&self) -> Result<()> {
        let mut tokens = self.active_tokens.write().await;
        let current_time = Utc::now();
        
        let expired_tokens: Vec<Uuid> = tokens.iter()
            .filter(|(_, token)| current_time >= token.expires_at)
            .map(|(id, _)| *id)
            .collect();

        for token_id in expired_tokens {
            if let Some(token) = tokens.remove(&token_id) {
                debug!("Removed expired token {} for agent {}", token_id, token.agent_id);
            }
        }

        Ok(())
    }

    /// Get security statistics
    pub async fn get_security_stats(&self) -> SecurityStats {
        let tokens = self.active_tokens.read().await;
        let audit_log = self.audit_log.read().await;
        let acl = self.access_control_list.read().await;

        let now = Utc::now();
        let one_hour_ago = now - Duration::hours(1);
        
        let recent_events = audit_log.iter()
            .filter(|entry| entry.timestamp >= one_hour_ago)
            .count();

        let failed_events = audit_log.iter()
            .filter(|entry| entry.timestamp >= one_hour_ago && !entry.success)
            .count();

        SecurityStats {
            active_tokens: tokens.len(),
            total_audit_events: audit_log.len(),
            recent_events,
            failed_events_last_hour: failed_events,
            protected_resources: acl.len(),
        }
    }
}

/// Security statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub active_tokens: usize,
    pub total_audit_events: usize,
    pub recent_events: usize,
    pub failed_events_last_hour: usize,
    pub protected_resources: usize,
}

/// Utility functions for common security operations
pub mod security_utils {
    use super::*;

    /// Get default capabilities for each agent layer
    pub fn get_layer_capabilities(layer: &crate::agents::AgentLayer) -> Vec<Capability> {
        match layer {
            crate::agents::AgentLayer::CECCA => vec![
                Capability::SystemAdmin,
                Capability::AgentManagement,
                Capability::TaskOrchestration,
                Capability::SecurityManagement,
                Capability::MonitoringAccess,
                Capability::StrategicPlanning,
            ],
            crate::agents::AgentLayer::Board => vec![
                Capability::PolicyEnforcement,
                Capability::MonitoringAccess,
                Capability::ComplianceCheck,
                Capability::AuditLog,
            ],
            crate::agents::AgentLayer::Executive => vec![
                Capability::TaskOrchestration,
                Capability::OperationalControl,
                Capability::MonitoringAccess,
                Capability::AgentManagement,
            ],
            crate::agents::AgentLayer::StackChief => vec![
                Capability::DomainExpertise,
                Capability::OperationalControl,
                Capability::TaskOrchestration,
            ],
            crate::agents::AgentLayer::Specialist => vec![
                Capability::DomainExpertise,
                Capability::DataAccess,
                Capability::ExternalAPIAccess,
            ],
            crate::agents::AgentLayer::Micro => vec![
                Capability::TaskExecution,
                Capability::DataAccess,
            ],
        }
    }

    /// Validate that a requested capability is appropriate for an agent layer
    pub fn validate_capability_for_layer(layer: &crate::agents::AgentLayer, capability: &Capability) -> bool {
        let allowed_capabilities = get_layer_capabilities(layer);
        allowed_capabilities.contains(capability)
    }
}