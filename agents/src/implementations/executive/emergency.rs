//! Emergency Responder Agent - Executive Layer
//! 
//! Simplified working version
//! Handles emergency response and crisis management

use crate::unified_types::*;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Emergency Responder Agent - Crisis management and response
/// 
/// Responsible for:
/// - Emergency detection and response
/// - Crisis coordination
/// - System recovery management
/// - Incident escalation
pub struct EmergencyAgent {
    metadata: AgentMetadata,
    state: RwLock<AgentState>,
    emergency_data: Arc<RwLock<EmergencyData>>,
}

/// Emergency data
#[derive(Debug, Default)]
struct EmergencyData {
    active_emergencies: Vec<Emergency>,
    response_protocols: Vec<ResponseProtocol>,
    metrics: EmergencyMetrics,
}

/// Emergency definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emergency {
    pub emergency_id: Uuid,
    pub emergency_type: EmergencyType,
    pub severity: EmergencySeverity,
    pub description: String,
    pub status: EmergencyStatus,
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

/// Emergency types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergencyType {
    SystemFailure,
    SecurityBreach,
    ResourceExhaustion,
    PerformanceDegradation,
    DataCorruption,
    Other(String),
}

/// Emergency severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergencySeverity {
    Critical,
    Major,
    Minor,
    Warning,
}

/// Emergency status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergencyStatus {
    Detected,
    Responding,
    Contained,
    Resolved,
}

/// Response protocol
#[derive(Debug, Clone)]
struct ResponseProtocol {
    pub protocol_id: String,
    pub emergency_type: EmergencyType,
    pub actions: Vec<String>,
}

/// Emergency metrics
#[derive(Debug, Default, Clone)]
struct EmergencyMetrics {
    pub total_emergencies: u64,
    pub resolved_emergencies: u64,
    pub average_response_time: f64,
}

/// Emergency report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyReport {
    pub report_id: Uuid,
    pub active_emergencies: usize,
    pub resolved_today: u64,
    pub response_time_avg: f64,
    pub recommendations: Vec<String>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl EmergencyAgent {
    pub fn new() -> Self {
        let metadata = AgentMetadata {
            id: Uuid::new_v4(),
            agent_id: "emergency-agent".to_string(),
            name: "Emergency Responder Agent".to_string(),
            layer: AgentLayer::L2Reasoning,
            category: AgentCategory::Operations,
            agent_type: AgentType::Master,
            language: AgentLanguage::Rust,
            description: "Emergency Response Agent - Crisis management and system recovery".to_string(),
            role: "Executive Emergency".to_string(),
            purpose: "Detect, respond to, and manage system emergencies".to_string(),
            state: AgentState::Created,
            health_status: HealthStatus::Unknown,
            parent_id: None,
            escalation_to: Some("noa-commander".to_string()),
            stack: None,
            capabilities: vec![
                "emergency-detection".to_string(),
                "crisis-response".to_string(),
                "system-recovery".to_string(),
                "incident-escalation".to_string(),
            ],
            tools: vec![],
            tags: vec!["executive".to_string(), "emergency".to_string()],
            inputs: vec!["system-alerts".to_string()],
            outputs: vec!["emergency-response".to_string()],
            dependencies: vec![],
            cpu_min: "1".to_string(),
            ram_min: "512MB".to_string(),
            disk_min: "100MB".to_string(),
            autonomy_level: "autonomous".to_string(),
            disposable: false,
            issues_identified: vec![],
            repair_recommendations: vec![],
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            last_updated: Some(chrono::Utc::now().to_rfc3339()),
            version: Some("1.0.0".to_string()),
        };
        
        Self {
            metadata,
            state: RwLock::new(AgentState::Created),
            emergency_data: Arc::new(RwLock::new(EmergencyData::default())),
        }
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        *self.state.write().await = AgentState::Initializing;
        
        // Initialize response protocols
        let mut data = self.emergency_data.write().await;
        data.response_protocols.push(ResponseProtocol {
            protocol_id: "system-failure".to_string(),
            emergency_type: EmergencyType::SystemFailure,
            actions: vec!["isolate".to_string(), "diagnose".to_string(), "recover".to_string()],
        });
        
        *self.state.write().await = AgentState::Ready;
        tracing::info!("Emergency Responder Agent initialized");
        Ok(())
    }
    
    pub async fn handle_emergency(&self, emergency: Emergency) -> Result<()> {
        let mut data = self.emergency_data.write().await;
        
        data.active_emergencies.push(emergency.clone());
        data.metrics.total_emergencies += 1;
        
        tracing::warn!("Emergency detected: {:?} - {}", emergency.emergency_type, emergency.description);
        
        Ok(())
    }
    
    pub async fn generate_report(&self) -> Result<EmergencyReport> {
        let data = self.emergency_data.read().await;
        
        Ok(EmergencyReport {
            report_id: Uuid::new_v4(),
            active_emergencies: data.active_emergencies.len(),
            resolved_today: data.metrics.resolved_emergencies,
            response_time_avg: data.metrics.average_response_time,
            recommendations: vec!["Continue monitoring for emergencies".to_string()],
            generated_at: chrono::Utc::now(),
        })
    }
    
    pub fn metadata(&self) -> &AgentMetadata {
        &self.metadata
    }
    
    pub async fn state(&self) -> AgentState {
        self.state.read().await.clone()
    }
}

impl Default for EmergencyAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_emergency_agent() {
        let agent = EmergencyAgent::new();
        assert_eq!(agent.metadata().name, "Emergency Responder Agent");
    }
    
    #[tokio::test]
    async fn test_initialize() {
        let mut agent = EmergencyAgent::new();
        agent.initialize().await.unwrap();
        assert_eq!(agent.state().await, AgentState::Ready);
    }
    
    #[tokio::test]
    async fn test_handle_emergency() {
        let mut agent = EmergencyAgent::new();
        agent.initialize().await.unwrap();
        
        let emergency = Emergency {
            emergency_id: Uuid::new_v4(),
            emergency_type: EmergencyType::SystemFailure,
            severity: EmergencySeverity::Critical,
            description: "Test emergency".to_string(),
            status: EmergencyStatus::Detected,
            detected_at: chrono::Utc::now(),
        };
        
        agent.handle_emergency(emergency).await.unwrap();
        
        let report = agent.generate_report().await.unwrap();
        assert_eq!(report.active_emergencies, 1);
    }
}
