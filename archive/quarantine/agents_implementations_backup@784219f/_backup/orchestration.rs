use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{AgentRegistry, ConversationService, PersonalAssistantService, DevelopmentAssistantService};

#[derive(Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: Uuid,
    pub task_type: AgentTaskType,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub assigned_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>,
    pub result: Option<TaskResult>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AgentTaskType {
    Conversation,
    TaskManagement,
    CodeGeneration,
    CodeAnalysis,
    Scheduling,
    Learning,
    Monitoring,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub error_message: Option<String>,
}

pub struct AgentOrchestrator {
    registry: Arc<RwLock<AgentRegistry>>,
    conversation: Option<Arc<RwLock<ConversationService>>>,
    personal_assistant: Option<Arc<RwLock<PersonalAssistantService>>>,
    development_assistant: Option<Arc<RwLock<DevelopmentAssistantService>>>,
    tasks: Arc<RwLock<Vec<AgentTask>>>,
}

impl AgentOrchestrator {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(AgentRegistry::new())),
            conversation: None,
            personal_assistant: None,
            development_assistant: None,
            tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_conversation_service(&mut self, service: Arc<RwLock<ConversationService>>) {
        self.conversation = Some(service);
    }

    pub fn with_personal_assistant(&mut self, service: Arc<RwLock<PersonalAssistantService>>) {
        self.personal_assistant = Some(service);
    }

    pub fn with_development_assistant(&mut self, service: Arc<RwLock<DevelopmentAssistantService>>) {
        self.development_assistant = Some(service);
    }

    pub async fn submit_task(&self, task_type: AgentTaskType, description: String, priority: TaskPriority) -> Result<Uuid, Box<dyn std::error::Error>> {
        let task = AgentTask {
            id: Uuid::new_v4(),
            task_type,
            description,
            priority,
            status: TaskStatus::Pending,
            assigned_agent: None,
            created_at: Utc::now(),
            completed_at: None,
            result: None,
        };

        let task_id = task.id;
        self.tasks.write().await.push(task);

        // Try to assign task to appropriate agent
        self.assign_task(task_id).await?;

        Ok(task_id)
    }

    async fn assign_task(&self, task_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            // Assign based on task type
            let agent_id = match task.task_type {
                AgentTaskType::Conversation => "conversation-agent",
                AgentTaskType::TaskManagement | AgentTaskType::Scheduling => "personal-assistant-agent",
                AgentTaskType::CodeGeneration | AgentTaskType::CodeAnalysis => "development-assistant-agent",
                AgentTaskType::Learning => "learning-agent",
                AgentTaskType::Monitoring => "monitoring-agent",
            };

            task.assigned_agent = Some(agent_id.to_string());
            task.status = TaskStatus::Assigned;

            // Start task execution
            self.execute_task(task.clone()).await?;
        }
        Ok(())
    }

    async fn execute_task(&self, mut task: AgentTask) -> Result<(), Box<dyn std::error::Error>> {
        task.status = TaskStatus::InProgress;

        let result = match task.task_type {
            AgentTaskType::Conversation => {
                if let Some(conv) = &self.conversation {
                    // Execute conversation task
                    self.execute_conversation_task(&task).await
                } else {
                    Err("Conversation service not available".into())
                }
            },
            AgentTaskType::TaskManagement | AgentTaskType::Scheduling => {
                if let Some(pa) = &self.personal_assistant {
                    // Execute personal assistant task
                    self.execute_personal_assistant_task(&task).await
                } else {
                    Err("Personal assistant service not available".into())
                }
            },
            AgentTaskType::CodeGeneration | AgentTaskType::CodeAnalysis => {
                if let Some(da) = &self.development_assistant {
                    // Execute development assistant task
                    self.execute_development_assistant_task(&task).await
                } else {
                    Err("Development assistant service not available".into())
                }
            },
            _ => {
                // Placeholder for other task types
                Ok(TaskResult {
                    success: true,
                    data: serde_json::json!({"message": "Task completed"}),
                    error_message: None,
                })
            }
        };

        // Update task with result
        let mut tasks = self.tasks.write().await;
        if let Some(t) = tasks.iter_mut().find(|t| t.id == task.id) {
            match result {
                Ok(task_result) => {
                    t.status = TaskStatus::Completed;
                    t.completed_at = Some(Utc::now());
                    t.result = Some(task_result);
                },
                Err(e) => {
                    t.status = TaskStatus::Failed;
                    t.result = Some(TaskResult {
                        success: false,
                        data: serde_json::json!({"error": e.to_string()}),
                        error_message: Some(e.to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    async fn execute_conversation_task(&self, task: &AgentTask) -> Result<TaskResult, Box<dyn std::error::Error>> {
        // Placeholder implementation - would integrate with conversation service
        Ok(TaskResult {
            success: true,
            data: serde_json::json!({"conversation_id": Uuid::new_v4(), "response": "Conversation task completed"}),
            error_message: None,
        })
    }

    async fn execute_personal_assistant_task(&self, task: &AgentTask) -> Result<TaskResult, Box<dyn std::error::Error>> {
        // Placeholder implementation - would integrate with personal assistant service
        Ok(TaskResult {
            success: true,
            data: serde_json::json!({"task_id": Uuid::new_v4(), "action": "Task created/updated"}),
            error_message: None,
        })
    }

    async fn execute_development_assistant_task(&self, task: &AgentTask) -> Result<TaskResult, Box<dyn std::error::Error>> {
        // Placeholder implementation - would integrate with development assistant service
        Ok(TaskResult {
            success: true,
            data: serde_json::json!({"code_generated": true, "language": "rust"}),
            error_message: None,
        })
    }

    pub async fn get_task_status(&self, task_id: Uuid) -> Option<AgentTask> {
        self.tasks.read().await.iter().find(|t| t.id == task_id).cloned()
    }

    pub async fn list_tasks(&self, status_filter: Option<TaskStatus>) -> Vec<AgentTask> {
        let tasks = self.tasks.read().await;
        if let Some(status) = status_filter {
            tasks.iter().filter(|t| t.status == status).cloned().collect()
        } else {
            tasks.clone()
        }
    }
}

#[derive(Clone)]
pub struct AgentRegistry {
    agents: HashMap<String, AgentInfo>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub capabilities: Vec<String>,
    pub status: AgentStatus,
    pub last_seen: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Online,
    Offline,
    Busy,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn register_agent(&mut self, info: AgentInfo) {
        self.agents.insert(info.id.clone(), info);
    }

    pub fn get_agent(&self, id: &str) -> Option<&AgentInfo> {
        self.agents.get(id)
    }

    pub fn list_agents(&self) -> Vec<&AgentInfo> {
        self.agents.values().collect()
    }
}
