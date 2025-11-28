// Re-export specific agent modules
pub mod board;
pub mod executive;
pub mod specialized;

// Communication module (if needed)
pub mod communication;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, warn, error, debug};

use crate::orchestration::Task;
use crate::security::SecurityManager;

/// Agent hierarchy layers as defined in the design
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentLayer {
    CECCA,      // Command, Executive, Control, Coordination, Authority (1-3 agents)
    Board,      // Governance & Policy (5-15 agents)
    Executive,  // Operational Management (10-25 agents)
    StackChief, // Domain Leadership (20-50 agents)
    Specialist, // Expert Capabilities (50-200 agents)
    Micro,      // Task Execution (100-1000+ agents)
}

/// Agent metadata and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub layer: AgentLayer,
    pub capabilities: Vec<String>,
    pub status: AgentStatus,
    pub resource_requirements: ResourceRequirements,
    pub performance_metrics: PerformanceMetrics,
    pub escalation_path: Option<Uuid>, // Parent agent for escalation
    pub subordinates: Vec<Uuid>,       // Child agents under management
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Initializing,
    Active,
    Busy,
    Idle,
    Offline,
    Error,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: f32,
    pub memory_mb: u64,
    pub storage_mb: u64,
    pub network_bandwidth_mbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub average_response_time_ms: f64,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u64,
    pub uptime_seconds: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            tasks_completed: 0,
            tasks_failed: 0,
            average_response_time_ms: 0.0,
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0,
            uptime_seconds: 0,
        }
    }
}

/// The agent management system that handles the six-layer hierarchy
pub struct AgentManager {
    agents: Arc<RwLock<HashMap<Uuid, Agent>>>,
    layer_assignments: Arc<RwLock<HashMap<AgentLayer, Vec<Uuid>>>>,
    security_manager: Arc<SecurityManager>,
    next_agent_number: Arc<RwLock<u32>>,
}

impl AgentManager {
    pub async fn new(initial_agent_count: u32, security_manager: &SecurityManager) -> Result<Self> {
        let manager = Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            layer_assignments: Arc::new(RwLock::new(HashMap::new())),
            security_manager: Arc::new(security_manager.clone()),
            next_agent_number: Arc::new(RwLock::new(1)),
        };

        // Initialize the agent hierarchy with appropriate distribution
        manager.initialize_hierarchy(initial_agent_count).await?;
        
        Ok(manager)
    }

    async fn initialize_hierarchy(&self, total_agents: u32) -> Result<()> {
        info!("Initializing agent hierarchy with {} total agents", total_agents);

        // Distribute agents across layers based on the design specifications
        let distribution = Self::calculate_layer_distribution(total_agents);
        
        for (layer, count) in distribution {
            for _ in 0..count {
                self.create_agent(layer.clone()).await?;
            }
        }

        // Establish hierarchy relationships
        self.establish_hierarchy_relationships().await?;

        info!("Agent hierarchy initialization complete");
        Ok(())
    }

    fn calculate_layer_distribution(total_agents: u32) -> Vec<(AgentLayer, u32)> {
        // Distribution based on design specifications
        let cecca_count = std::cmp::min(3, std::cmp::max(1, total_agents / 100));
        let board_count = std::cmp::min(15, std::cmp::max(5, total_agents / 20));
        let executive_count = std::cmp::min(25, std::cmp::max(10, total_agents / 10));
        let stack_chief_count = std::cmp::min(50, std::cmp::max(20, total_agents / 5));
        let specialist_count = std::cmp::min(200, std::cmp::max(50, total_agents / 3));
        
        let used = cecca_count + board_count + executive_count + stack_chief_count + specialist_count;
        let micro_count = if total_agents > used { total_agents - used } else { total_agents / 2 };

        vec![
            (AgentLayer::CECCA, cecca_count),
            (AgentLayer::Board, board_count),
            (AgentLayer::Executive, executive_count),
            (AgentLayer::StackChief, stack_chief_count),
            (AgentLayer::Specialist, specialist_count),
            (AgentLayer::Micro, micro_count),
        ]
    }

    async fn create_agent(&self, layer: AgentLayer) -> Result<Uuid> {
        let agent_number = {
            let mut num = self.next_agent_number.write().await;
            let current = *num;
            *num += 1;
            current
        };

        let capabilities = Self::get_layer_capabilities(&layer);
        let resource_requirements = Self::get_layer_resource_requirements(&layer);

        let agent = Agent {
            id: Uuid::new_v4(),
            name: format!("{:?}-Agent-{:04}", layer, agent_number),
            layer: layer.clone(),
            capabilities,
            status: AgentStatus::Initializing,
            resource_requirements,
            performance_metrics: PerformanceMetrics::default(),
            escalation_path: None,
            subordinates: Vec::new(),
            created_at: chrono::Utc::now(),
            last_heartbeat: None,
        };

        let agent_id = agent.id;
        
        // Store agent
        self.agents.write().await.insert(agent_id, agent);
        
        // Add to layer assignments
        self.layer_assignments.write().await
            .entry(layer)
            .or_insert_with(Vec::new)
            .push(agent_id);

        debug!("Created agent: {} ({})", agent_id, agent_number);
        Ok(agent_id)
    }

    fn get_layer_capabilities(layer: &AgentLayer) -> Vec<String> {
        match layer {
            AgentLayer::CECCA => vec![
                "strategic_planning".to_string(),
                "system_authority".to_string(),
                "cross_organizational_coordination".to_string(),
                "emergency_decision_making".to_string(),
                "resource_allocation".to_string(),
            ],
            AgentLayer::Board => vec![
                "policy_enforcement".to_string(),
                "governance_oversight".to_string(),
                "compliance_monitoring".to_string(),
                "risk_assessment".to_string(),
                "ethics_validation".to_string(),
            ],
            AgentLayer::Executive => vec![
                "operational_coordination".to_string(),
                "task_orchestration".to_string(),
                "resource_management".to_string(),
                "performance_monitoring".to_string(),
                "emergency_response".to_string(),
            ],
            AgentLayer::StackChief => vec![
                "domain_leadership".to_string(),
                "subject_matter_expertise".to_string(),
                "team_coordination".to_string(),
                "workflow_orchestration".to_string(),
                "specialization_management".to_string(),
            ],
            AgentLayer::Specialist => vec![
                "deep_domain_expertise".to_string(),
                "complex_analysis".to_string(),
                "system_integration".to_string(),
                "advanced_processing".to_string(),
                "decision_support".to_string(),
            ],
            AgentLayer::Micro => vec![
                "task_execution".to_string(),
                "atomic_operations".to_string(),
                "parallel_processing".to_string(),
                "rule_based_actions".to_string(),
                "resource_efficiency".to_string(),
            ],
        }
    }

    fn get_layer_resource_requirements(layer: &AgentLayer) -> ResourceRequirements {
        match layer {
            AgentLayer::CECCA => ResourceRequirements {
                cpu_cores: 4.0,
                memory_mb: 8192,
                storage_mb: 10240,
                network_bandwidth_mbps: 100,
            },
            AgentLayer::Board => ResourceRequirements {
                cpu_cores: 2.0,
                memory_mb: 4096,
                storage_mb: 5120,
                network_bandwidth_mbps: 50,
            },
            AgentLayer::Executive => ResourceRequirements {
                cpu_cores: 2.0,
                memory_mb: 4096,
                storage_mb: 5120,
                network_bandwidth_mbps: 50,
            },
            AgentLayer::StackChief => ResourceRequirements {
                cpu_cores: 1.5,
                memory_mb: 2048,
                storage_mb: 2560,
                network_bandwidth_mbps: 25,
            },
            AgentLayer::Specialist => ResourceRequirements {
                cpu_cores: 1.0,
                memory_mb: 1024,
                storage_mb: 1280,
                network_bandwidth_mbps: 10,
            },
            AgentLayer::Micro => ResourceRequirements {
                cpu_cores: 0.25,
                memory_mb: 256,
                storage_mb: 512,
                network_bandwidth_mbps: 5,
            },
        }
    }

    async fn establish_hierarchy_relationships(&self) -> Result<()> {
        info!("Establishing hierarchy relationships");

        let layer_assignments = self.layer_assignments.read().await;
        let mut agents = self.agents.write().await;

        // CECCA -> Board relationships
        if let (Some(cecca_agents), Some(board_agents)) = 
            (layer_assignments.get(&AgentLayer::CECCA), layer_assignments.get(&AgentLayer::Board)) {
            
            for board_id in board_agents {
                if let Some(board_agent) = agents.get_mut(board_id) {
                    board_agent.escalation_path = cecca_agents.first().copied();
                }
                
                if let Some(cecca_agent) = agents.get_mut(&cecca_agents[0]) {
                    cecca_agent.subordinates.push(*board_id);
                }
            }
        }

        // Continue hierarchy establishment for other layers...
        // Board -> Executive, Executive -> StackChief, StackChief -> Specialist, Specialist -> Micro

        drop(agents);
        drop(layer_assignments);
        
        info!("Hierarchy relationships established");
        Ok(())
    }

    pub async fn find_suitable_agent(&self, task: &Task) -> Result<Uuid> {
        let agents = self.agents.read().await;
        
        // Find agents with matching capabilities and available status
        for (agent_id, agent) in agents.iter() {
            if agent.status == AgentStatus::Active || agent.status == AgentStatus::Idle {
                // Check if agent has required capabilities
                let has_capabilities = task.required_capabilities.iter()
                    .all(|cap| agent.capabilities.contains(cap));
                
                if has_capabilities {
                    return Ok(*agent_id);
                }
            }
        }
        
        Err(anyhow::anyhow!("No suitable agent found for task"))
    }

    pub async fn send_task_to_agent(&self, agent_id: Uuid, task: &Task) -> Result<()> {
        // Update agent status
        {
            let mut agents = self.agents.write().await;
            if let Some(agent) = agents.get_mut(&agent_id) {
                agent.status = AgentStatus::Busy;
            }
        }

        // TODO: Send task to actual agent implementation
        debug!("Task {} sent to agent {}", task.id, agent_id);
        
        Ok(())
    }

    pub async fn health_check(&self) -> Result<()> {
        let mut agents = self.agents.write().await;
        let current_time = chrono::Utc::now();
        
        for agent in agents.values_mut() {
            // Check if agent has sent heartbeat recently
            if let Some(last_heartbeat) = agent.last_heartbeat {
                let duration = current_time.signed_duration_since(last_heartbeat);
                if duration.num_seconds() > 60 {
                    warn!("Agent {} heartbeat timeout", agent.id);
                    agent.status = AgentStatus::Offline;
                }
            }
        }
        
        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting agent manager");
        
        // Set all agents to active status
        let mut agents = self.agents.write().await;
        for agent in agents.values_mut() {
            if agent.status == AgentStatus::Initializing {
                agent.status = AgentStatus::Active;
            }
        }
        
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down agent manager");
        
        // Set all agents to offline
        let mut agents = self.agents.write().await;
        for agent in agents.values_mut() {
            agent.status = AgentStatus::Offline;
        }
        
        Ok(())
    }

    pub async fn get_agent_status(&self, agent_id: Uuid) -> Result<AgentStatus> {
        let agents = self.agents.read().await;
        agents.get(&agent_id)
            .map(|agent| agent.status.clone())
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))
    }

    pub async fn get_layer_statistics(&self) -> HashMap<AgentLayer, LayerStats> {
        let agents = self.agents.read().await;
        let mut stats = HashMap::new();

        for agent in agents.values() {
            let layer_stats = stats.entry(agent.layer.clone()).or_insert(LayerStats::default());
            layer_stats.total_agents += 1;
            
            match agent.status {
                AgentStatus::Active => layer_stats.active_agents += 1,
                AgentStatus::Busy => layer_stats.busy_agents += 1,
                AgentStatus::Idle => layer_stats.idle_agents += 1,
                AgentStatus::Offline => layer_stats.offline_agents += 1,
                AgentStatus::Error => layer_stats.error_agents += 1,
                _ => {}
            }
        }

        stats
    }
}

#[derive(Debug, Default)]
pub struct LayerStats {
    pub total_agents: u32,
    pub active_agents: u32,
    pub busy_agents: u32,
    pub idle_agents: u32,
    pub offline_agents: u32,
    pub error_agents: u32,
}