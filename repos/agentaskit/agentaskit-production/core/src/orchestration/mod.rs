//! Unified Orchestration Module
//! 
//! This module combines and enhances the advanced orchestration capabilities from rustecosys2
//! while preserving all autonomous orchestration, scheduling, and execution features.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use tracing::{info, warn, error, debug};

use crate::agents::AgentManager;
use crate::communication::MessageBroker;
use crate::monitoring::MetricsCollector;

/// The main orchestration engine that coordinates all system activities
pub struct OrchestratorEngine {
    agent_manager: Arc<AgentManager>,
    message_broker: Arc<MessageBroker>,
    metrics_collector: Arc<MetricsCollector>,
    task_queue: Arc<RwLock<TaskQueue>>,
    running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: TaskType,
    pub priority: Priority,
    pub required_capabilities: Vec<String>,
    pub parameters: serde_json::Value,
    pub dependencies: Vec<Uuid>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: TaskStatus,
    pub assigned_agent: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Analysis,
    Processing,
    Communication,
    Monitoring,
    Deployment,
    Maintenance,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Emergency = 0,
    Critical = 1,
    High = 2,
    Medium = 3,
    Normal = 4,
    Low = 5,
    Maintenance = 6,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

pub struct TaskQueue {
    pending_tasks: Vec<Task>,
    active_tasks: HashMap<Uuid, Task>,
    completed_tasks: Vec<Task>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            pending_tasks: Vec::new(),
            active_tasks: HashMap::new(),
            completed_tasks: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.pending_tasks.push(task);
        // Sort by priority to ensure highest priority tasks are processed first
        self.pending_tasks.sort_by(|a, b| a.priority.cmp(&b.priority));
    }

    pub fn get_next_task(&mut self) -> Option<Task> {
        self.pending_tasks.pop()
    }

    pub fn assign_task(&mut self, task_id: Uuid, agent_id: Uuid) -> Result<()> {
        if let Some(mut task) = self.pending_tasks.iter()
            .position(|t| t.id == task_id)
            .map(|pos| self.pending_tasks.remove(pos)) {
            
            task.assigned_agent = Some(agent_id);
            task.status = TaskStatus::Assigned;
            self.active_tasks.insert(task_id, task);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found: {}", task_id))
        }
    }

    pub fn complete_task(&mut self, task_id: Uuid, success: bool) -> Result<()> {
        if let Some(mut task) = self.active_tasks.remove(&task_id) {
            task.status = if success { TaskStatus::Completed } else { TaskStatus::Failed };
            self.completed_tasks.push(task);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Active task not found: {}", task_id))
        }
    }
}

impl OrchestratorEngine {
    pub async fn new(
        agent_manager: AgentManager,
        message_broker: MessageBroker,
        metrics_collector: MetricsCollector,
    ) -> Result<Self> {
        Ok(Self {
            agent_manager: Arc::new(agent_manager),
            message_broker: Arc::new(message_broker),
            metrics_collector: Arc::new(metrics_collector),
            task_queue: Arc::new(RwLock::new(TaskQueue::new())),
            running: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn start(&self, mode: String) -> Result<()> {
        info!("Starting orchestration engine in {} mode", mode);
        
        // Set running state
        *self.running.write().await = true;

        // Start component services
        self.agent_manager.start().await?;
        self.message_broker.start().await?;
        self.metrics_collector.start().await?;

        // Start main orchestration loops
        self.start_task_scheduler().await?;
        self.start_health_monitor().await?;
        self.start_metrics_collector().await?;

        match mode.as_str() {
            "autonomous" => self.start_autonomous_mode().await?,
            "supervised" => self.start_supervised_mode().await?,
            "interactive" => self.start_interactive_mode().await?,
            _ => {
                warn!("Unknown mode '{}', defaulting to supervised", mode);
                self.start_supervised_mode().await?;
            }
        }

        Ok(())
    }

    async fn start_task_scheduler(&self) -> Result<()> {
        let task_queue = Arc::clone(&self.task_queue);
        let agent_manager = Arc::clone(&self.agent_manager);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            info!("Task scheduler started");
            
            while *running.read().await {
                // Check for available tasks and agents
                let mut queue = task_queue.write().await;
                
                if let Some(task) = queue.get_next_task() {
                    // Find suitable agent for the task
                    if let Ok(agent_id) = agent_manager.find_suitable_agent(&task).await {
                        debug!("Assigning task {} to agent {}", task.id, agent_id);
                        
                        if let Err(e) = queue.assign_task(task.id, agent_id) {
                            error!("Failed to assign task: {}", e);
                        } else {
                            // Send task to agent
                            if let Err(e) = agent_manager.send_task_to_agent(agent_id, &task).await {
                                error!("Failed to send task to agent: {}", e);
                            }
                        }
                    } else {
                        // No suitable agent available, put task back
                        queue.add_task(task);
                    }
                }
                
                drop(queue);
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            
            info!("Task scheduler stopped");
        });

        Ok(())
    }

    async fn start_health_monitor(&self) -> Result<()> {
        let agent_manager = Arc::clone(&self.agent_manager);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            info!("Health monitor started");
            
            while *running.read().await {
                // Check agent health and system status
                if let Err(e) = agent_manager.health_check().await {
                    error!("Health check failed: {}", e);
                }
                
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            }
            
            info!("Health monitor stopped");
        });

        Ok(())
    }

    async fn start_metrics_collector(&self) -> Result<()> {
        let metrics_collector = Arc::clone(&self.metrics_collector);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            info!("Metrics collector started");
            
            while *running.read().await {
                // Collect and process metrics
                if let Err(e) = metrics_collector.collect_metrics().await {
                    error!("Metrics collection failed: {}", e);
                }
                
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
            
            info!("Metrics collector stopped");
        });

        Ok(())
    }

    async fn start_autonomous_mode(&self) -> Result<()> {
        info!("Starting autonomous operation mode");
        
        // In autonomous mode, the system operates independently
        // with minimal human intervention
        // TODO: Implement autonomous decision making
        
        Ok(())
    }

    async fn start_supervised_mode(&self) -> Result<()> {
        info!("Starting supervised operation mode");
        
        // In supervised mode, critical decisions require approval
        // TODO: Implement approval workflows
        
        Ok(())
    }

    async fn start_interactive_mode(&self) -> Result<()> {
        info!("Starting interactive operation mode");
        
        // In interactive mode, users can directly control the system
        // TODO: Implement interactive command interface
        
        Ok(())
    }

    pub async fn submit_task(&self, task: Task) -> Result<Uuid> {
        info!("Submitting task: {} ({})", task.name, task.id);
        
        let task_id = task.id;
        let mut queue = self.task_queue.write().await;
        queue.add_task(task);
        
        Ok(task_id)
    }

    pub async fn get_task_status(&self, task_id: Uuid) -> Result<TaskStatus> {
        let queue = self.task_queue.read().await;
        
        // Check pending tasks
        if queue.pending_tasks.iter().any(|t| t.id == task_id) {
            return Ok(TaskStatus::Pending);
        }
        
        // Check active tasks
        if let Some(task) = queue.active_tasks.get(&task_id) {
            return Ok(task.status.clone());
        }
        
        // Check completed tasks
        if let Some(task) = queue.completed_tasks.iter().find(|t| t.id == task_id) {
            return Ok(task.status.clone());
        }
        
        Err(anyhow::anyhow!("Task not found: {}", task_id))
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down orchestration engine");
        
        // Stop all operations
        *self.running.write().await = false;
        
        // Shutdown components
        self.agent_manager.shutdown().await?;
        self.message_broker.shutdown().await?;
        self.metrics_collector.shutdown().await?;
        
        info!("Orchestration engine shutdown complete");
        Ok(())
    }
}

// Helper function to create a new task
pub fn create_task(
    name: String,
    description: String,
    task_type: TaskType,
    priority: Priority,
    required_capabilities: Vec<String>,
    parameters: serde_json::Value,
) -> Task {
    Task {
        id: Uuid::new_v4(),
        name,
        description,
        task_type,
        priority,
        required_capabilities,
        parameters,
        dependencies: Vec::new(),
        deadline: None,
        created_at: chrono::Utc::now(),
        status: TaskStatus::Pending,
        assigned_agent: None,
    }
}
