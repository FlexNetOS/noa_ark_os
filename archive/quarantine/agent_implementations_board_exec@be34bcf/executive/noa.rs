//! NOA Commander - Chief Executive Agent
//!
//! Simplified working version - Phase 3A
//! Original: 1,467 lines with 40+ structs
//! This version: ~300 lines, room to grow

use crate::unified_types::*;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// NOA Commander - Root CECCA Agent
///
/// The highest-level autonomous agent responsible for:
/// - Strategic decision-making
/// - Agent coordination
/// - Resource allocation
/// - Emergency response
/// - System-wide oversight
pub struct NoaCommander {
    metadata: AgentMetadata,
    state: RwLock<AgentState>,
    strategic_engine: Arc<RwLock<StrategicEngine>>,
    agent_coordinator: Arc<RwLock<AgentCoordinator>>,
    config: CommanderConfig,
}

/// Configuration for NOA Commander
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommanderConfig {
    /// Maximum concurrent decisions
    pub max_concurrent_decisions: usize,

    /// Emergency response threshold (0.0-1.0)
    pub emergency_threshold: f64,

    /// Strategic planning interval (seconds)
    pub planning_interval: u64,
}

impl Default for CommanderConfig {
    fn default() -> Self {
        Self {
            max_concurrent_decisions: 10,
            emergency_threshold: 0.95,
            planning_interval: 3600, // 1 hour
        }
    }
}

/// Strategic decision-making engine
#[derive(Debug)]
struct StrategicEngine {
    /// Current strategic goals
    goals: HashMap<String, StrategicGoal>,

    /// Decision history
    decisions: Vec<StrategicDecision>,

    /// System metrics
    metrics: SystemMetrics,
}

/// Strategic goal definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicGoal {
    pub id: String,
    pub name: String,
    pub description: String,
    pub priority: f64,
    pub progress: f64,
}

impl StrategicGoal {
    pub fn new(id: String, name: String, description: String, priority: f64) -> Self {
        Self {
            id,
            name,
            description,
            priority,
            progress: 0.0,
        }
    }
}

/// Strategic decision record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicDecision {
    pub id: Uuid,
    pub decision_type: DecisionType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub confidence: f64,
    pub outcome: String,
}

/// Decision types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionType {
    ResourceAllocation,
    AgentDeployment,
    SystemModification,
    EmergencyResponse,
    StrategicPlanning,
}

/// System-wide metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub health_score: f64,
    pub resource_utilization: f64,
    pub agent_count: usize,
    pub error_rate: f64,
}

/// Agent coordination system
#[derive(Debug, Default)]
struct AgentCoordinator {
    /// Registered agents
    agents: HashMap<AgentId, AgentInfo>,

    /// Active coordination tasks
    active_tasks: Vec<CoordinationTask>,
}

/// Agent information
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: AgentId,
    pub name: String,
    pub agent_type: AgentType,
    pub state: AgentState,
    pub capabilities: Vec<String>,
}

/// Coordination task
#[derive(Debug, Clone)]
pub struct CoordinationTask {
    pub id: Uuid,
    pub task_type: String,
    pub assigned_agents: Vec<AgentId>,
    pub status: String,
}

impl NoaCommander {
    /// Create new NOA Commander
    pub fn new() -> Self {
        Self::with_config(CommanderConfig::default())
    }

    /// Create NOA Commander with custom config
    pub fn with_config(config: CommanderConfig) -> Self {
        let metadata = AgentMetadata {
            id: Uuid::new_v4(),
            agent_id: "noa-commander".to_string(),
            name: "NOA Commander".to_string(),
            layer: AgentLayer::L1Autonomy,
            category: AgentCategory::Governance,
            agent_type: AgentType::Master,
            language: AgentLanguage::Rust,
            description: "Chief Executive Commander Agent - Root CECCA".to_string(),
            role: "Executive Commander".to_string(),
            purpose: "Overall system coordination and strategic decision-making".to_string(),
            state: AgentState::Created,
            health_status: HealthStatus::Unknown,
            parent_id: None,
            escalation_to: None,
            stack: None,
            capabilities: vec![
                "strategic-planning".to_string(),
                "resource-allocation".to_string(),
                "emergency-response".to_string(),
                "agent-coordination".to_string(),
                "system-oversight".to_string(),
            ],
            tools: vec![],
            tags: vec![
                "root".to_string(),
                "cecca".to_string(),
                "executive".to_string(),
            ],
            inputs: vec![],
            outputs: vec![],
            dependencies: vec![],
            cpu_min: "2".to_string(),
            ram_min: "2GB".to_string(),
            disk_min: "1GB".to_string(),
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
            strategic_engine: Arc::new(RwLock::new(StrategicEngine {
                goals: HashMap::new(),
                decisions: Vec::new(),
                metrics: SystemMetrics::default(),
            })),
            agent_coordinator: Arc::new(RwLock::new(AgentCoordinator::default())),
            config,
        }
    }

    /// Initialize the commander
    pub async fn initialize(&mut self) -> Result<()> {
        *self.state.write().await = AgentState::Initializing;

        // Initialize strategic engine with default goals
        let mut engine = self.strategic_engine.write().await;

        engine.goals.insert(
            "system-stability".to_string(),
            StrategicGoal::new(
                "system-stability".to_string(),
                "Maintain System Stability".to_string(),
                "Ensure 99.9% uptime and stable performance".to_string(),
                1.0,
            ),
        );

        engine.goals.insert(
            "performance-optimization".to_string(),
            StrategicGoal::new(
                "performance-optimization".to_string(),
                "Optimize System Performance".to_string(),
                "Improve throughput and reduce latency".to_string(),
                0.8,
            ),
        );

        *self.state.write().await = AgentState::Ready;

        tracing::info!("NOA Commander initialized successfully");
        Ok(())
    }

    /// Start the commander
    pub async fn start(&mut self) -> Result<()> {
        *self.state.write().await = AgentState::Running;

        tracing::info!("NOA Commander started");
        Ok(())
    }

    /// Make strategic decision
    pub async fn make_decision(
        &self,
        decision_type: DecisionType,
        context: serde_json::Value,
    ) -> Result<StrategicDecision> {
        let mut engine = self.strategic_engine.write().await;

        // Simple decision-making logic
        let confidence = match decision_type {
            DecisionType::ResourceAllocation => 0.85,
            DecisionType::AgentDeployment => 0.80,
            DecisionType::SystemModification => 0.75,
            DecisionType::EmergencyResponse => 0.95,
            DecisionType::StrategicPlanning => 0.90,
        };

        let decision = StrategicDecision {
            id: Uuid::new_v4(),
            decision_type,
            timestamp: chrono::Utc::now(),
            confidence,
            outcome: "approved".to_string(),
        };

        engine.decisions.push(decision.clone());

        tracing::debug!("Strategic decision made: {:?}", decision.id);
        Ok(decision)
    }

    /// Register an agent
    pub async fn register_agent(&self, agent_info: AgentInfo) -> Result<()> {
        let mut coordinator = self.agent_coordinator.write().await;

        coordinator
            .agents
            .insert(agent_info.id.clone(), agent_info.clone());

        tracing::info!("Agent registered: {} ({})", agent_info.name, agent_info.id);
        Ok(())
    }

    /// Coordinate agents for a task
    pub async fn coordinate_task(
        &self,
        task_type: String,
        required_capabilities: Vec<String>,
    ) -> Result<CoordinationTask> {
        let mut coordinator = self.agent_coordinator.write().await;

        // Find agents with required capabilities
        let assigned_agents: Vec<AgentId> = coordinator
            .agents
            .iter()
            .filter(|(_, info)| {
                required_capabilities
                    .iter()
                    .any(|cap| info.capabilities.contains(cap))
            })
            .map(|(id, _)| id.clone())
            .collect();

        let task = CoordinationTask {
            id: Uuid::new_v4(),
            task_type,
            assigned_agents,
            status: "active".to_string(),
        };

        coordinator.active_tasks.push(task.clone());

        tracing::info!("Coordination task created: {:?}", task.id);
        Ok(task)
    }

    /// Handle emergency
    pub async fn handle_emergency(
        &self,
        emergency_type: String,
        severity: f64,
    ) -> Result<StrategicDecision> {
        tracing::error!(
            "Emergency detected: {} (severity: {})",
            emergency_type,
            severity
        );

        self.make_decision(
            DecisionType::EmergencyResponse,
            serde_json::json!({
                "emergency_type": emergency_type,
                "severity": severity,
            }),
        )
        .await
    }

    /// Get system status
    pub async fn system_status(&self) -> Result<SystemMetrics> {
        let engine = self.strategic_engine.read().await;
        let coordinator = self.agent_coordinator.read().await;

        let mut metrics = engine.metrics.clone();
        metrics.agent_count = coordinator.agents.len();
        metrics.health_score = 0.85; // TODO: Calculate real health
        metrics.resource_utilization = 0.70; // TODO: Calculate real utilization

        Ok(metrics)
    }

    /// Get metadata
    pub fn metadata(&self) -> &AgentMetadata {
        &self.metadata
    }

    /// Get current state
    pub async fn state(&self) -> AgentState {
        self.state.read().await.clone()
    }

    /// List strategic goals
    pub async fn list_goals(&self) -> Vec<StrategicGoal> {
        let engine = self.strategic_engine.read().await;
        engine.goals.values().cloned().collect()
    }

    /// List decisions
    pub async fn list_decisions(&self) -> Vec<StrategicDecision> {
        let engine = self.strategic_engine.read().await;
        engine.decisions.clone()
    }

    /// List registered agents
    pub async fn list_agents(&self) -> Vec<AgentInfo> {
        let coordinator = self.agent_coordinator.read().await;
        coordinator.agents.values().cloned().collect()
    }
}

impl Default for NoaCommander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_commander() {
        let commander = NoaCommander::new();
        assert_eq!(commander.metadata().name, "NOA Commander");
        assert_eq!(commander.metadata().layer, AgentLayer::L1Autonomy);
    }

    #[tokio::test]
    async fn test_initialize() {
        let mut commander = NoaCommander::new();
        commander.initialize().await.unwrap();

        let state = commander.state().await;
        assert_eq!(state, AgentState::Ready);

        let goals = commander.list_goals().await;
        assert!(goals.len() >= 2);
    }

    #[tokio::test]
    async fn test_make_decision() {
        let mut commander = NoaCommander::new();
        commander.initialize().await.unwrap();

        let decision = commander
            .make_decision(
                DecisionType::ResourceAllocation,
                serde_json::json!({"test": true}),
            )
            .await
            .unwrap();

        assert_eq!(decision.outcome, "approved");
        assert!(decision.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_register_agent() {
        let commander = NoaCommander::new();

        let agent_info = AgentInfo {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            agent_type: AgentType::Worker,
            state: AgentState::Ready,
            capabilities: vec!["testing".to_string()],
        };

        commander.register_agent(agent_info).await.unwrap();

        let agents = commander.list_agents().await;
        assert_eq!(agents.len(), 1);
    }

    #[tokio::test]
    async fn test_coordinate_task() {
        let commander = NoaCommander::new();

        // Register agent with capability
        let agent_info = AgentInfo {
            id: "worker-1".to_string(),
            name: "Worker 1".to_string(),
            agent_type: AgentType::Worker,
            state: AgentState::Ready,
            capabilities: vec!["data-processing".to_string()],
        };

        commander.register_agent(agent_info).await.unwrap();

        // Create coordination task
        let task = commander
            .coordinate_task(
                "process-data".to_string(),
                vec!["data-processing".to_string()],
            )
            .await
            .unwrap();

        assert_eq!(task.assigned_agents.len(), 1);
    }

    #[tokio::test]
    async fn test_system_status() {
        let mut commander = NoaCommander::new();
        commander.initialize().await.unwrap();

        let status = commander.system_status().await.unwrap();
        assert!(status.health_score > 0.0);
    }
}
