// Agent types from NOA ARK OS registry
// Based on the comprehensive agent directory CSV

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent layer in the NOA hierarchy
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentLayer {
    /// Board-level agents (Executive Team)
    Board,
    /// Executive-level agents (CEO)
    Executive,
    /// Stack Chief agents (VPs/Directors)
    StackChief,
    /// Specialist agents (Domain experts)
    Specialist,
    /// Micro agents (Task-specific)
    Micro,
}

impl AgentLayer {
    /// Get layer name as string
    pub fn name(&self) -> &str {
        match self {
            AgentLayer::Board => "Board",
            AgentLayer::Executive => "Executive",
            AgentLayer::StackChief => "StackChief",
            AgentLayer::Specialist => "Specialist",
            AgentLayer::Micro => "Micro",
        }
    }
}

/// Agent category/domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentCategory {
    Operations,
    Research,
    Governance,
    Build,
    UX,
    Plugins,
    ModelSelection,
    Orchestration,
    Security,
    BoardExecutive,
    StackManagement,
    Uncategorized,
}

/// Health status of an agent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    NeedsRepair,
    Unknown,
}

/// Complete agent metadata from registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub agent_name: String,
    pub agent_id: String,
    pub role: String,
    pub layer: AgentLayer,
    pub category: AgentCategory,
    pub purpose: String,
    pub health_status: HealthStatus,
    
    // Operational details
    pub autonomy_level: String,
    pub escalation_to: Option<String>,
    pub stack: Option<String>,
    
    // Technical specs
    pub tools: Vec<String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub dependencies: Vec<String>,
    
    // Resource requirements
    pub cpu_min: String,
    pub ram_min: String,
    pub disk_min: String,
    
    // Metadata
    pub created_at: Option<String>,
    pub last_updated: Option<String>,
    pub version: Option<String>,
    
    // Issues and recommendations
    pub issues_identified: Vec<String>,
    pub repair_recommendations: Vec<String>,
}

impl AgentMetadata {
    /// Create a new agent metadata instance
    pub fn new(agent_name: String, agent_id: String) -> Self {
        Self {
            agent_name,
            agent_id,
            role: String::new(),
            layer: AgentLayer::Micro,
            category: AgentCategory::Uncategorized,
            purpose: String::new(),
            health_status: HealthStatus::Unknown,
            autonomy_level: "tbd".to_string(),
            escalation_to: None,
            stack: None,
            tools: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            dependencies: Vec::new(),
            cpu_min: "1".to_string(),
            ram_min: "1GB".to_string(),
            disk_min: "500MB".to_string(),
            created_at: None,
            last_updated: None,
            version: None,
            issues_identified: Vec::new(),
            repair_recommendations: Vec::new(),
        }
    }
    
    /// Check if agent is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.health_status, HealthStatus::Healthy)
    }
    
    /// Check if agent needs repair
    pub fn needs_repair(&self) -> bool {
        matches!(self.health_status, HealthStatus::NeedsRepair)
    }
    
    /// Get full agent identifier
    pub fn full_id(&self) -> String {
        format!("{}::{}", self.layer_name(), self.agent_id)
    }
    
    /// Get layer name as string
    pub fn layer_name(&self) -> &str {
        match self.layer {
            AgentLayer::Board => "Board",
            AgentLayer::Executive => "Executive",
            AgentLayer::StackChief => "StackChief",
            AgentLayer::Specialist => "Specialist",
            AgentLayer::Micro => "Micro",
        }
    }
}

/// Agent registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_agents: usize,
    pub healthy_agents: usize,
    pub needs_repair: usize,
    pub unknown_status: usize,
    pub agents_by_layer: HashMap<String, usize>,
    pub agents_by_category: HashMap<String, usize>,
}

impl RegistryStats {
    pub fn new() -> Self {
        Self {
            total_agents: 0,
            healthy_agents: 0,
            needs_repair: 0,
            unknown_status: 0,
            agents_by_layer: HashMap::new(),
            agents_by_category: HashMap::new(),
        }
    }
}

impl Default for RegistryStats {
    fn default() -> Self {
        Self::new()
    }
}
