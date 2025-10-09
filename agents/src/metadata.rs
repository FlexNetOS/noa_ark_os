use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
}

impl AgentMetadata {
    pub fn new(name: String, description: String, category: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            category,
            tags: vec![],
        }
    }
}
