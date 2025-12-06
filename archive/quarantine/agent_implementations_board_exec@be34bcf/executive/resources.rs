//! Resource Allocator Agent - Executive Layer
//!
//! Simplified working version
//! Manages system resource allocation and optimization

use crate::unified_types::*;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Resource Allocator Agent - Resource management and optimization
///
/// Responsible for:
/// - Resource allocation management
/// - Capacity planning
/// - Resource optimization
/// - Load balancing
pub struct ResourceAgent {
    metadata: AgentMetadata,
    state: RwLock<AgentState>,
    resource_data: Arc<RwLock<ResourceData>>,
}

/// Resource data
#[derive(Debug, Default)]
struct ResourceData {
    allocations: HashMap<String, ResourceAllocation>,
    pools: HashMap<String, ResourcePool>,
    metrics: ResourceMetrics,
}

/// Resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub allocation_id: Uuid,
    pub resource_type: String,
    pub allocated_to: String,
    pub amount: f64,
    pub utilization: f64,
}

/// Resource pool
#[derive(Debug, Clone)]
struct ResourcePool {
    pub pool_id: String,
    pub resource_type: String,
    pub total_capacity: f64,
    pub allocated: f64,
    pub available: f64,
}

/// Resource metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub total_allocations: u64,
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub storage_utilization: f64,
}

/// Resource report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReport {
    pub report_id: Uuid,
    pub metrics: ResourceMetrics,
    pub active_allocations: usize,
    pub recommendations: Vec<String>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl ResourceAgent {
    pub fn new() -> Self {
        let metadata = AgentMetadata {
            id: Uuid::new_v4(),
            agent_id: "resource-agent".to_string(),
            name: "Resource Allocator Agent".to_string(),
            layer: AgentLayer::L2Reasoning,
            category: AgentCategory::Operations,
            agent_type: AgentType::Master,
            language: AgentLanguage::Rust,
            description: "Resource Allocator - System resource management and optimization"
                .to_string(),
            role: "Executive Resources".to_string(),
            purpose: "Manage and optimize system resource allocation".to_string(),
            state: AgentState::Created,
            health_status: HealthStatus::Unknown,
            parent_id: None,
            escalation_to: Some("noa-commander".to_string()),
            stack: None,
            capabilities: vec![
                "resource-allocation".to_string(),
                "capacity-planning".to_string(),
                "resource-optimization".to_string(),
                "load-balancing".to_string(),
            ],
            tools: vec![],
            tags: vec!["executive".to_string(), "resources".to_string()],
            inputs: vec!["resource-requests".to_string()],
            outputs: vec!["resource-allocation".to_string()],
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
            resource_data: Arc::new(RwLock::new(ResourceData::default())),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        *self.state.write().await = AgentState::Initializing;

        // Initialize resource pools
        let mut data = self.resource_data.write().await;
        data.pools.insert(
            "cpu".to_string(),
            ResourcePool {
                pool_id: "cpu-pool".to_string(),
                resource_type: "CPU".to_string(),
                total_capacity: 100.0,
                allocated: 60.0,
                available: 40.0,
            },
        );

        data.pools.insert(
            "memory".to_string(),
            ResourcePool {
                pool_id: "memory-pool".to_string(),
                resource_type: "Memory".to_string(),
                total_capacity: 100.0,
                allocated: 70.0,
                available: 30.0,
            },
        );

        *self.state.write().await = AgentState::Ready;
        tracing::info!("Resource Allocator Agent initialized");
        Ok(())
    }

    pub async fn allocate_resource(&self, allocation: ResourceAllocation) -> Result<()> {
        let mut data = self.resource_data.write().await;

        data.allocations
            .insert(allocation.allocation_id.to_string(), allocation);
        data.metrics.total_allocations += 1;

        Ok(())
    }

    pub async fn generate_report(&self) -> Result<ResourceReport> {
        let data = self.resource_data.read().await;

        Ok(ResourceReport {
            report_id: Uuid::new_v4(),
            metrics: data.metrics.clone(),
            active_allocations: data.allocations.len(),
            recommendations: vec![
                "Monitor resource utilization for optimization opportunities".to_string(),
            ],
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

impl Default for ResourceAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_resource_agent() {
        let agent = ResourceAgent::new();
        assert_eq!(agent.metadata().name, "Resource Allocator Agent");
    }

    #[tokio::test]
    async fn test_initialize() {
        let mut agent = ResourceAgent::new();
        agent.initialize().await.unwrap();
        assert_eq!(agent.state().await, AgentState::Ready);
    }

    #[tokio::test]
    async fn test_allocate_resource() {
        let mut agent = ResourceAgent::new();
        agent.initialize().await.unwrap();

        let allocation = ResourceAllocation {
            allocation_id: Uuid::new_v4(),
            resource_type: "CPU".to_string(),
            allocated_to: "test-agent".to_string(),
            amount: 2.0,
            utilization: 0.75,
        };

        agent.allocate_resource(allocation).await.unwrap();

        let report = agent.generate_report().await.unwrap();
        assert_eq!(report.active_allocations, 1);
    }
}
