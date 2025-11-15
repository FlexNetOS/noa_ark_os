use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl Task {
    pub fn new(name: String, description: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = chrono::Utc::now();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ArkOsError {
    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, ArkOsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("Test Task".to_string(), "A test task".to_string());
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.description, "A test task");
        assert!(matches!(task.status, TaskStatus::Pending));
    }

    #[test]
    fn test_task_status_update() {
        let mut task = Task::new("Test Task".to_string(), "A test task".to_string());
        let original_updated_at = task.updated_at;
        
        std::thread::sleep(std::time::Duration::from_millis(1));
        task.update_status(TaskStatus::Running);
        
        assert!(matches!(task.status, TaskStatus::Running));
        assert!(task.updated_at > original_updated_at);
    }
}
