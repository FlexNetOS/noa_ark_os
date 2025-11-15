//! Security Specialist Agent - Simplified version
use crate::unified_types::*;
use crate::Result;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct SecurityAgent {
    metadata: AgentMetadata,
    state: RwLock<AgentState>,
}

impl SecurityAgent {
    pub fn new() -> Self {
        Self {
            metadata: AgentMetadata {
                id: Uuid::new_v4(),
                agent_id: "security-agent".to_string(),
                name: "Security Agent".to_string(),
                layer: AgentLayer::L4Operations,
                category: AgentCategory::Security,
                agent_type: AgentType::Worker,
                language: AgentLanguage::Rust,
                description: "Security Specialist".to_string(),
                role: "Specialist Security".to_string(),
                purpose: "Security monitoring and enforcement".to_string(),
                state: AgentState::Created,
                health_status: HealthStatus::Unknown,
                parent_id: None,
                escalation_to: Some("emergency-responder".to_string()),
                stack: None,
                capabilities: vec!["security-scan".to_string(), "threat-detection".to_string()],
                tools: vec![],
                tags: vec!["specialist".to_string(), "security".to_string()],
                inputs: vec!["security-events".to_string()],
                outputs: vec!["security-alerts".to_string()],
                dependencies: vec![],
                cpu_min: "1".to_string(),
                ram_min: "1GB".to_string(),
                disk_min: "500MB".to_string(),
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

impl Default for SecurityAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_agent() {
        let agent = SecurityAgent::new();
        assert_eq!(agent.metadata().name, "Security Agent");
    }
}
