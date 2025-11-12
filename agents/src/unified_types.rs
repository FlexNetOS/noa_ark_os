//! Unified Type System for NOA Agent Architecture
//! Provides the complete type system for agent factory and registry

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// ENUMS - Single Source of Truth
// ============================================================================

/// Agent layer in NOA's five-layer architecture.
///
/// This enum represents the hierarchical organization of agents in the NOA ARK OS system,
/// from strategic governance down to infrastructure tasks. The L1-L5 naming convention
/// replaced the legacy Executive → Micro hierarchy to provide a clearer technical model,
/// though the registry parser still accepts both naming schemes for backward compatibility.
///
/// ## Layer Hierarchy (L1 → L5)
///
/// - **L1Autonomy** (Executive): Constitutional oversight and highest-level decision making.
/// - **L2Reasoning** (Board): Strategic planning, policy formation, and governance agents.
/// - **L3Orchestration** (Stack-Chief): Cross-domain coordination and workflow orchestration.
/// - **L4Operations** (Specialist): Operational execution and domain expertise.
/// - **L5Infrastructure** (Micro): Fine-grained task execution and infrastructure services.
///
/// Agents escalate upward through layers when decisions exceed their authority level. The
/// registry organizes its agents into this hierarchy while maintaining backward compatibility
/// with the legacy naming scheme.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentLayer {
    L1Autonomy,       // Root CECCA, Constitutional (was: Executive)
    L2Reasoning,      // Board & Executive agents (was: Board)
    L3Orchestration,  // Chief Commanders, Orchestrators (was: Stack-Chief)
    L4Operations,     // Specialists, Workers (was: Specialist)
    L5Infrastructure, // Micro agents, Subject domain (was: Micro)
}

impl Default for AgentLayer {
    fn default() -> Self {
        Self::L4Operations
    }
}

impl AgentLayer {
    pub fn name(&self) -> &str {
        match self {
            Self::L1Autonomy => "L1Autonomy",
            Self::L2Reasoning => "L2Reasoning",
            Self::L3Orchestration => "L3Orchestration",
            Self::L4Operations => "L4Operations",
            Self::L5Infrastructure => "L5Infrastructure",
        }
    }
}

/// Agent category/domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentCategory {
    Analysis,
    Code,
    DevOps,
    Testing,
    Documentation,
    Security,
    Governance,
    Operations,
    Research,
    BuildSystems,
    UX,
    Plugins,
    ModelSelection,
    Orchestration,
    Other,
}

impl Default for AgentCategory {
    fn default() -> Self {
        Self::Other
    }
}

/// Agent runtime state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    Created,
    Initializing,
    Ready,
    Running,
    Paused,
    Terminating,
    Terminated,
}

impl Default for AgentState {
    fn default() -> Self {
        Self::Created
    }
}

/// Agent health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    NeedsRepair,
    Error,
    Unknown,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Agent type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    Master,   // CECCA, Board, Executive
    Worker,   // Specialists, Operators
    SubAgent, // Micro agents
    Swarm,    // Coordinated groups
}

impl Default for AgentType {
    fn default() -> Self {
        Self::Worker
    }
}

/// Agent language/runtime
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentLanguage {
    Rust,
    Python,
    Go,
}

impl Default for AgentLanguage {
    fn default() -> Self {
        Self::Rust
    }
}

// ============================================================================
// AGENT METADATA - Unified Structure
// ============================================================================

/// Unified agent metadata (combines factory + registry needs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    // Runtime identity
    pub id: Uuid,
    pub agent_id: String,
    pub name: String,

    // Classification
    pub layer: AgentLayer,
    pub category: AgentCategory,
    pub agent_type: AgentType,
    pub language: AgentLanguage,

    // Description
    pub description: String,
    pub role: String,
    pub purpose: String,

    // Status
    pub state: AgentState,
    pub health_status: HealthStatus,

    // Relationships
    pub parent_id: Option<String>,
    pub escalation_to: Option<String>,
    pub stack: Option<String>,

    // Capabilities
    pub capabilities: Vec<String>,
    pub tools: Vec<String>,
    pub tags: Vec<String>,

    // I/O
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub dependencies: Vec<String>,

    // Resources
    pub cpu_min: String,
    pub ram_min: String,
    pub disk_min: String,

    // Behavior
    pub autonomy_level: String,
    pub disposable: bool,

    // Diagnostics
    pub issues_identified: Vec<String>,
    pub repair_recommendations: Vec<String>,

    // Metadata
    pub created_at: Option<String>,
    pub last_updated: Option<String>,
    pub version: Option<String>,
}

impl AgentMetadata {
    /// Create minimal agent metadata for factory use
    pub fn minimal(name: String, description: String, category: AgentCategory) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id: name.clone(),
            name: name.clone(),
            layer: AgentLayer::L4Operations,
            category,
            agent_type: AgentType::Worker,
            language: AgentLanguage::Rust,
            description,
            role: String::new(),
            purpose: String::new(),
            state: AgentState::Created,
            health_status: HealthStatus::Unknown,
            parent_id: None,
            escalation_to: None,
            stack: None,
            capabilities: Vec::new(),
            tools: Vec::new(),
            tags: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            dependencies: Vec::new(),
            cpu_min: "1".to_string(),
            ram_min: "1GB".to_string(),
            disk_min: "500MB".to_string(),
            autonomy_level: "managed".to_string(),
            disposable: false,
            issues_identified: Vec::new(),
            repair_recommendations: Vec::new(),
            created_at: None,
            last_updated: None,
            version: Some("0.1.0".to_string()),
        }
    }

    /// Create from registry data
    pub fn from_registry(agent_name: String, agent_id: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id,
            name: agent_name,
            layer: AgentLayer::L4Operations,
            category: AgentCategory::Other,
            agent_type: AgentType::Worker,
            language: AgentLanguage::Rust,
            description: String::new(),
            role: String::new(),
            purpose: String::new(),
            state: AgentState::Created,
            health_status: HealthStatus::Unknown,
            parent_id: None,
            escalation_to: None,
            stack: None,
            capabilities: Vec::new(),
            tools: Vec::new(),
            tags: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            dependencies: Vec::new(),
            cpu_min: "1".to_string(),
            ram_min: "1GB".to_string(),
            disk_min: "500MB".to_string(),
            autonomy_level: "tbd".to_string(),
            disposable: false,
            issues_identified: Vec::new(),
            repair_recommendations: Vec::new(),
            created_at: None,
            last_updated: None,
            version: None,
        }
    }

    /// Backwards compatibility - old factory signature
    pub fn new(name: String, description: String, category: String) -> Self {
        let cat = match category.to_lowercase().as_str() {
            "code" => AgentCategory::Code,
            "devops" => AgentCategory::DevOps,
            "testing" => AgentCategory::Testing,
            "security" => AgentCategory::Security,
            "documentation" => AgentCategory::Documentation,
            "analysis" => AgentCategory::Analysis,
            _ => AgentCategory::Other,
        };

        Self::minimal(name, description, cat)
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.health_status, HealthStatus::Healthy)
    }

    pub fn needs_repair(&self) -> bool {
        matches!(
            self.health_status,
            HealthStatus::NeedsRepair | HealthStatus::Error
        )
    }

    pub fn layer_name(&self) -> &str {
        self.layer.name()
    }

    pub fn set_layer(&mut self, layer: String) {
        // Parse string to enum
        self.layer = match layer.to_lowercase().as_str() {
            "l1autonomy" | "l1" => AgentLayer::L1Autonomy,
            "l2reasoning" | "l2" => AgentLayer::L2Reasoning,
            "l3orchestration" | "l3" => AgentLayer::L3Orchestration,
            "l4operations" | "l4" => AgentLayer::L4Operations,
            "l5infrastructure" | "l5" => AgentLayer::L5Infrastructure,
            _ => AgentLayer::L4Operations,
        };
    }

    pub fn set_status(&mut self, status: String) {
        self.health_status = match status.to_lowercase().as_str() {
            "healthy" => HealthStatus::Healthy,
            "degraded" => HealthStatus::Degraded,
            "needs_repair" | "repair" => HealthStatus::NeedsRepair,
            "error" => HealthStatus::Error,
            _ => HealthStatus::Unknown,
        };
    }

    pub fn full_id(&self) -> String {
        format!("{}::{}", self.layer_name(), self.agent_id)
    }
}

// ============================================================================
// AGENT RUNTIME STRUCTURE
// ============================================================================

/// Runtime agent instance
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub agent_type: AgentType,
    pub language: AgentLanguage,
    pub state: AgentState,
    pub parent_id: Option<String>,
    pub capabilities: Vec<String>,
    pub disposable: bool,
}

impl Agent {
    pub fn new(id: String, name: String, agent_type: AgentType, language: AgentLanguage) -> Self {
        Self {
            id,
            name,
            agent_type,
            language,
            state: AgentState::Created,
            parent_id: None,
            capabilities: Vec::new(),
            disposable: false,
        }
    }

    pub fn make_disposable(mut self) -> Self {
        self.disposable = true;
        self
    }
}

// ============================================================================
// REGISTRY STATISTICS
// ============================================================================

/// Agent registry statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
        Self::default()
    }
}

// ============================================================================
// TYPE ALIASES
// ============================================================================

pub type AgentId = String;
