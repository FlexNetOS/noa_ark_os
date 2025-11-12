//! Monitoring Specialist Agent - Simplified version
use crate::unified_types::*;
use crate::Result;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct MonitoringAgent {
    metadata: AgentMetadata,
    state: RwLock<AgentState>,
}

impl MonitoringAgent {
    pub fn new() -> Self {
        Self {
            metadata: AgentMetadata {
                id: Uuid::new_v4(),
                agent_id: "monitoring-agent".to_string(),
                name: "Monitoring Agent".to_string(),
                layer: AgentLayer::L4Operations,
                category: AgentCategory::Operations,
                agent_type: AgentType::Worker,
                language: AgentLanguage::Rust,
                description: "Monitoring Specialist".to_string(),
                role: "Specialist Monitoring".to_string(),
                purpose: "System and application monitoring".to_string(),
                state: AgentState::Created,
                health_status: HealthStatus::Unknown,
                parent_id: None,
                escalation_to: Some("system-orchestrator".to_string()),
                stack: None,
                capabilities: vec!["monitoring".to_string(), "alerting".to_string()],
                tools: vec![],
                tags: vec!["specialist".to_string()],
                inputs: vec!["metrics".to_string()],
                outputs: vec!["alerts".to_string()],
                dependencies: vec![],
                cpu_min: "0.5".to_string(),
                ram_min: "512MB".to_string(),
                disk_min: "1GB".to_string(),
                autonomy_level: "autonomous".to_string(),
                disposable: false,
                issues_identified: vec![],
                repair_recommendations: vec![],
                created_at: Some(chrono::Utc::now().to_rfc3339()),
                last_updated: Some(chrono::Utc::now().to_rfc3339()),
                version: Some("1.0.0".to_string()),
            },
            state: RwLock::new(AgentState::Created),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        *self.state.write().await = AgentState::Ready;
        Ok(())
    }

    pub fn metadata(&self) -> &AgentMetadata {
        &self.metadata
    }
    pub async fn state(&self) -> AgentState {
        self.state.read().await.clone()
    }
}

impl Default for MonitoringAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_agent() {
        let agent = MonitoringAgent::new();
        assert_eq!(agent.metadata().name, "Monitoring Agent");
    }
}
