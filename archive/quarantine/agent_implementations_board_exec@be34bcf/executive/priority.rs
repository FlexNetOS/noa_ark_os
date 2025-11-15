//! Priority Manager Agent - Executive Layer
//!
//! Simplified working version
//! Manages task prioritization and execution ordering

use crate::unified_types::*;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Priority Manager Agent - Task prioritization and ordering
///
/// Responsible for:
/// - Task priority assignment
/// - Execution queue management
/// - Resource allocation prioritization
/// - Dynamic priority adjustment
pub struct PriorityAgent {
    metadata: AgentMetadata,
    state: RwLock<AgentState>,
    priority_data: Arc<RwLock<PriorityData>>,
}

/// Priority data
#[derive(Debug, Default)]
struct PriorityData {
    task_queue: VecDeque<PrioritizedTask>,
    priority_rules: Vec<PriorityRule>,
    metrics: PriorityMetrics,
}

/// Prioritized task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizedTask {
    pub task_id: Uuid,
    pub task_name: String,
    pub priority_level: PriorityLevel,
    pub urgency: f64,
    pub importance: f64,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}

/// Priority level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PriorityLevel {
    Critical,
    High,
    Medium,
    Low,
}

/// Priority rule
#[derive(Debug, Clone)]
struct PriorityRule {
    pub rule_id: String,
    pub condition: String,
    pub priority_boost: f64,
}

/// Priority metrics
#[derive(Debug, Default, Clone)]
struct PriorityMetrics {
    pub total_tasks_prioritized: u64,
    pub high_priority_tasks: u64,
    pub average_priority_time: f64,
}

/// Priority report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityReport {
    pub report_id: Uuid,
    pub queued_tasks: usize,
    pub high_priority_count: usize,
    pub average_wait_time: f64,
    pub recommendations: Vec<String>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl PriorityAgent {
    pub fn new() -> Self {
        let metadata = AgentMetadata {
            id: Uuid::new_v4(),
            agent_id: "priority-agent".to_string(),
            name: "Priority Manager Agent".to_string(),
            layer: AgentLayer::L2Reasoning,
            category: AgentCategory::Operations,
            agent_type: AgentType::Master,
            language: AgentLanguage::Rust,
            description: "Priority Manager - Task prioritization and execution ordering"
                .to_string(),
            role: "Executive Priority".to_string(),
            purpose: "Manage task priorities and optimize execution order".to_string(),
            state: AgentState::Created,
            health_status: HealthStatus::Unknown,
            parent_id: None,
            escalation_to: Some("noa-commander".to_string()),
            stack: None,
            capabilities: vec![
                "task-prioritization".to_string(),
                "queue-management".to_string(),
                "priority-adjustment".to_string(),
                "resource-prioritization".to_string(),
            ],
            tools: vec![],
            tags: vec!["executive".to_string(), "priority".to_string()],
            inputs: vec!["task-requests".to_string()],
            outputs: vec!["prioritized-queue".to_string()],
            dependencies: vec![],
            cpu_min: "0.5".to_string(),
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
            priority_data: Arc::new(RwLock::new(PriorityData::default())),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        *self.state.write().await = AgentState::Initializing;

        // Initialize priority rules
        let mut data = self.priority_data.write().await;
        data.priority_rules.push(PriorityRule {
            rule_id: "emergency-boost".to_string(),
            condition: "emergency".to_string(),
            priority_boost: 1.5,
        });

        *self.state.write().await = AgentState::Ready;
        tracing::info!("Priority Manager Agent initialized");
        Ok(())
    }

    pub async fn prioritize_task(&self, task: PrioritizedTask) -> Result<()> {
        let mut data = self.priority_data.write().await;

        // Insert task in priority order
        let insert_pos = data
            .task_queue
            .iter()
            .position(|t| t.priority_level < task.priority_level)
            .unwrap_or(data.task_queue.len());

        data.task_queue.insert(insert_pos, task);
        data.metrics.total_tasks_prioritized += 1;

        Ok(())
    }

    pub async fn generate_report(&self) -> Result<PriorityReport> {
        let data = self.priority_data.read().await;

        let high_priority = data
            .task_queue
            .iter()
            .filter(|t| {
                t.priority_level == PriorityLevel::High
                    || t.priority_level == PriorityLevel::Critical
            })
            .count();

        Ok(PriorityReport {
            report_id: Uuid::new_v4(),
            queued_tasks: data.task_queue.len(),
            high_priority_count: high_priority,
            average_wait_time: data.metrics.average_priority_time,
            recommendations: vec!["Continue monitoring task priorities".to_string()],
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

impl Default for PriorityAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_priority_agent() {
        let agent = PriorityAgent::new();
        assert_eq!(agent.metadata().name, "Priority Manager Agent");
    }

    #[tokio::test]
    async fn test_initialize() {
        let mut agent = PriorityAgent::new();
        agent.initialize().await.unwrap();
        assert_eq!(agent.state().await, AgentState::Ready);
    }

    #[tokio::test]
    async fn test_prioritize_task() {
        let mut agent = PriorityAgent::new();
        agent.initialize().await.unwrap();

        let task = PrioritizedTask {
            task_id: Uuid::new_v4(),
            task_name: "Test task".to_string(),
            priority_level: PriorityLevel::High,
            urgency: 0.8,
            importance: 0.9,
            assigned_at: chrono::Utc::now(),
        };

        agent.prioritize_task(task).await.unwrap();

        let report = agent.generate_report().await.unwrap();
        assert_eq!(report.queued_tasks, 1);
        assert_eq!(report.high_priority_count, 1);
    }
}
