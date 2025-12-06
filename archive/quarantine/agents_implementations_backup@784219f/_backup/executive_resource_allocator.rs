use anyhow::Result;
use async_trait::async_trait;
use chrono;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use crate::agents::{
    Agent, AgentContext, AgentId, AgentMessage, AgentMetadata, AgentRole, AgentState,
    HealthStatus, Priority, ResourceRequirements, ResourceUsage, Task, TaskResult, TaskStatus,
};

/// Resource Allocator Agent - Dynamic resource management and optimization
/// 
/// The Resource Allocator is responsible for:
/// - Monitoring system resource usage across all agents and clusters
/// - Dynamic resource allocation and reallocation based on demand
/// - Resource optimization and cost management
/// - Capacity planning and scaling decisions
/// - Resource contention resolution
/// - Performance-based resource adjustment
pub struct ResourceAllocator {
    metadata: AgentMetadata,
    state: RwLock<AgentState>,
    context: Option<AgentContext>,
    
    /// Resource management engine
    resource_manager: Arc<RwLock<ResourceManager>>,
    
    /// Resource optimization engine
    optimizer: Arc<RwLock<ResourceOptimizer>>,
    
    /// Resource monitoring system
    monitor: Arc<RwLock<ResourceMonitor>>,
    
    /// Capacity planner
    capacity_planner: Arc<RwLock<CapacityPlanner>>,
    
    /// Configuration
    config: ResourceAllocatorConfig,
}

/// Configuration for Resource Allocator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocatorConfig {
    /// Resource monitoring interval
    pub monitoring_interval: Duration,
    
    /// Optimization cycle interval
    pub optimization_interval: Duration,
    
    /// Resource allocation timeout
    pub allocation_timeout: Duration,
    
    /// Minimum resource headroom percentage
    pub min_headroom_percentage: f64,
    
    /// Maximum resource utilization before scaling
    pub max_utilization_threshold: f64,
    
    /// Resource rebalancing threshold
    pub rebalancing_threshold: f64,
    
    /// Cost optimization enabled
    pub cost_optimization_enabled: bool,
    
    /// Auto-scaling enabled
    pub auto_scaling_enabled: bool,
    
    /// Resource prediction window
    pub prediction_window: Duration,
    
    /// Historical data retention
    pub history_retention: Duration,
}

impl Default for ResourceAllocatorConfig {
    fn default() -> Self {
        Self {
            monitoring_interval: Duration::from_secs(30),
            optimization_interval: Duration::from_secs(300), // 5 minutes
            allocation_timeout: Duration::from_secs(60),
            min_headroom_percentage: 20.0,
            max_utilization_threshold: 80.0,
            rebalancing_threshold: 25.0,
            cost_optimization_enabled: true,
            auto_scaling_enabled: true,
            prediction_window: Duration::from_secs(1800), // 30 minutes
            history_retention: Duration::from_secs(259200), // 3 days
        }
    }
}

/// Resource management system
#[derive(Debug, Default)]
struct ResourceManager {
    /// Available resources in the system
    available_resources: HashMap<String, ResourcePool>,
    
    /// Resource allocations to agents
    allocations: HashMap<AgentId, ResourceAllocation>,
    
    /// Resource reservations (temporary holds)
    reservations: HashMap<Uuid, ResourceReservation>,
    
    /// Resource usage tracking
    usage_tracker: ResourceUsageTracker,
    
    /// Allocation history
    allocation_history: VecDeque<AllocationEvent>,
}

/// Resource pool representing available resources of a specific type
#[derive(Debug)]
struct ResourcePool {
    pub resource_type: ResourceType,
    pub total_capacity: ResourceCapacity,
    pub available_capacity: ResourceCapacity,
    pub allocated_capacity: ResourceCapacity,
    pub reserved_capacity: ResourceCapacity,
    pub cost_per_unit: f64,
    pub provider: ResourceProvider,
    pub location: String,
    pub constraints: Vec<ResourceConstraint>,
    pub last_updated: Instant,
}

/// Types of resources
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    CPU,
    Memory,
    Storage,
    Network,
    GPU,
    Custom(String),
}

/// Resource capacity definition
#[derive(Debug, Clone)]
pub struct ResourceCapacity {
    pub cpu_cores: f64,
    pub memory_bytes: u64,
    pub storage_bytes: u64,
    pub network_mbps: f64,
    pub gpu_units: u32,
    pub custom_units: HashMap<String, f64>,
}

impl Default for ResourceCapacity {
    fn default() -> Self {
        Self {
            cpu_cores: 0.0,
            memory_bytes: 0,
            storage_bytes: 0,
            network_mbps: 0.0,
            gpu_units: 0,
            custom_units: HashMap::new(),
        }
    }
}

/// Resource providers
#[derive(Debug, Clone)]
pub enum ResourceProvider {
    Local,
    Cloud(String),
    Container(String),
    Kubernetes(String),
    External(String),
}

/// Resource constraints
#[derive(Debug, Clone)]
pub enum ResourceConstraint {
    Location(String),
    CostLimit(f64),
    PerformanceClass(String),
    SecurityLevel(String),
    Availability(f64),
    Custom(String, serde_json::Value),
}

/// Resource allocation to an agent
#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub agent_id: AgentId,
    pub allocated_resources: ResourceCapacity,
    pub allocation_time: Instant,
    pub last_updated: Instant,
    pub allocation_priority: Priority,
    pub constraints: Vec<ResourceConstraint>,
    pub cost: f64,
    pub utilization: ResourceUtilization,
}

/// Resource utilization metrics
#[derive(Debug, Clone)]
pub struct ResourceUtilization {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub storage_usage_percent: f64,
    pub network_usage_percent: f64,
    pub gpu_usage_percent: f64,
    pub efficiency_score: f64,
    pub last_measured: chrono::DateTime<chrono::Utc>,
}

impl Default for ResourceUtilization {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_percent: 0.0,
            storage_usage_percent: 0.0,
            network_usage_percent: 0.0,
            gpu_usage_percent: 0.0,
            efficiency_score: 0.0,
            last_measured: chrono::Utc::now(),
        }
    }
}

/// Temporary resource reservation
#[derive(Debug)]
struct ResourceReservation {
    pub id: Uuid,
    pub agent_id: AgentId,
    pub requested_resources: ResourceCapacity,
    pub reservation_time: Instant,
    pub expiration_time: Instant,
    pub purpose: String,
}

/// Resource usage tracker
#[derive(Debug, Default)]
struct ResourceUsageTracker {
    /// Current usage snapshots
    current_usage: HashMap<AgentId, ResourceUtilization>,
    
    /// Historical usage data
    usage_history: VecDeque<ResourceUsageSnapshot>,
    
    /// Usage patterns and trends
    usage_patterns: HashMap<AgentId, UsagePattern>,
}

/// Resource usage snapshot
#[derive(Debug)]
struct ResourceUsageSnapshot {
    pub timestamp: Instant,
    pub agent_usages: HashMap<AgentId, ResourceUtilization>,
    pub total_utilization: ResourceUtilization,
}

/// Usage pattern analysis
#[derive(Debug)]
struct UsagePattern {
    pub agent_id: AgentId,
    pub average_utilization: ResourceUtilization,
    pub peak_utilization: ResourceUtilization,
    pub utilization_trend: UtilizationTrend,
    pub seasonal_patterns: Vec<SeasonalPattern>,
    pub prediction_accuracy: f64,
}

/// Utilization trend
#[derive(Debug)]
enum UtilizationTrend {
    Increasing,
    Stable,
    Decreasing,
    Cyclical,
    Unpredictable,
}

/// Seasonal pattern
#[derive(Debug)]
struct SeasonalPattern {
    pub period: Duration,
    pub amplitude: f64,
    pub phase_offset: Duration,
}

/// Allocation event for history tracking
#[derive(Debug)]
struct AllocationEvent {
    pub event_type: AllocationEventType,
    pub agent_id: AgentId,
    pub resource_change: ResourceCapacity,
    pub timestamp: Instant,
    pub reason: String,
    pub cost_impact: f64,
}

/// Types of allocation events
#[derive(Debug)]
enum AllocationEventType {
    Allocated,
    Deallocated,
    Increased,
    Decreased,
    Rebalanced,
    Optimized,
}

/// Resource optimization engine
#[derive(Debug, Default)]
struct ResourceOptimizer {
    /// Current optimization strategy
    optimization_strategy: OptimizationStrategy,
    
    /// Optimization objectives
    objectives: Vec<OptimizationObjective>,
    
    /// Optimization constraints
    constraints: Vec<OptimizationConstraint>,
    
    /// Optimization history
    optimization_history: VecDeque<OptimizationResult>,
}

/// Optimization strategies
#[derive(Debug)]
enum OptimizationStrategy {
    CostMinimization,
    PerformanceMaximization,
    UtilizationOptimization,
    BalancedOptimization,
    CustomStrategy(String),
}

impl Default for OptimizationStrategy {
    fn default() -> Self {
        OptimizationStrategy::BalancedOptimization
    }
}

/// Optimization objectives
#[derive(Debug)]
struct OptimizationObjective {
    pub objective_type: ObjectiveType,
    pub weight: f64,
    pub target_value: f64,
    pub tolerance: f64,
}

/// Types of optimization objectives
#[derive(Debug)]
enum ObjectiveType {
    MinimizeCost,
    MaximizePerformance,
    MaximizeUtilization,
    MinimizeLatency,
    MaximizeAvailability,
    Custom(String),
}

/// Optimization constraints
#[derive(Debug)]
struct OptimizationConstraint {
    pub constraint_type: ConstraintType,
    pub limit_value: f64,
    pub hard_constraint: bool,
}

/// Types of optimization constraints
#[derive(Debug)]
enum ConstraintType {
    MaxCost,
    MinPerformance,
    MaxLatency,
    MinAvailability,
    ResourceLimit(ResourceType),
    Custom(String),
}

/// Optimization result
#[derive(Debug)]
struct OptimizationResult {
    pub optimization_time: Instant,
    pub strategy_used: OptimizationStrategy,
    pub improvements: HashMap<String, f64>,
    pub cost_savings: f64,
    pub performance_gains: f64,
    pub actions_taken: Vec<OptimizationAction>,
}

/// Optimization actions
#[derive(Debug)]
enum OptimizationAction {
    ResourceReallocation { from: AgentId, to: AgentId, resource: ResourceType, amount: f64 },
    ResourceScaling { agent: AgentId, resource: ResourceType, scale_factor: f64 },
    ResourceMigration { agent: AgentId, from_pool: String, to_pool: String },
    ResourceConsolidation { agents: Vec<AgentId>, target_pool: String },
}

/// Resource monitoring system
#[derive(Debug, Default)]
struct ResourceMonitor {
    /// Active monitoring targets
    monitoring_targets: HashSet<AgentId>,
    
    /// Monitoring metrics
    metrics: HashMap<String, MonitoringMetric>,
    
    /// Alert thresholds
    alert_thresholds: HashMap<String, AlertThreshold>,
    
    /// Active alerts
    active_alerts: Vec<ResourceAlert>,
}

/// Monitoring metric
#[derive(Debug)]
struct MonitoringMetric {
    pub name: String,
    pub metric_type: MetricType,
    pub current_value: f64,
    pub target_value: Option<f64>,
    pub last_updated: Instant,
    pub history: VecDeque<MetricSample>,
}

/// Types of monitoring metrics
#[derive(Debug)]
enum MetricType {
    Utilization,
    Throughput,
    Latency,
    ErrorRate,
    Cost,
    Efficiency,
    Custom(String),
}

/// Metric sample for historical tracking
#[derive(Debug)]
struct MetricSample {
    pub timestamp: Instant,
    pub value: f64,
}

/// Alert threshold configuration
#[derive(Debug)]
struct AlertThreshold {
    pub metric_name: String,
    pub warning_threshold: f64,
    pub critical_threshold: f64,
    pub duration_threshold: Duration,
}

/// Resource alert
#[derive(Debug)]
struct ResourceAlert {
    pub alert_id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub affected_resources: Vec<String>,
    pub timestamp: Instant,
    pub acknowledged: bool,
}

/// Types of alerts
#[derive(Debug)]
enum AlertType {
    HighUtilization,
    LowUtilization,
    ResourceContention,
    AllocationFailure,
    CostOverrun,
    PerformanceDegradation,
    Custom(String),
}

/// Alert severity levels
#[derive(Debug)]
enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Capacity planning system
#[derive(Debug, Default)]
struct CapacityPlanner {
    /// Capacity forecasts
    forecasts: HashMap<String, CapacityForecast>,
    
    /// Scaling recommendations
    recommendations: Vec<ScalingRecommendation>,
    
    /// Capacity planning models
    models: HashMap<String, PredictionModel>,
}

/// Capacity forecast
#[derive(Debug)]
struct CapacityForecast {
    pub forecast_type: String,
    pub time_horizon: Duration,
    pub predicted_demand: ResourceCapacity,
    pub confidence_interval: (ResourceCapacity, ResourceCapacity),
    pub assumptions: Vec<String>,
    pub last_updated: Instant,
}

/// Scaling recommendation
#[derive(Debug)]
struct ScalingRecommendation {
    pub target: String,
    pub recommendation_type: RecommendationType,
    pub resource_changes: ResourceCapacity,
    pub expected_impact: ScalingImpact,
    pub confidence: f64,
    pub priority: Priority,
    pub estimated_cost: f64,
}

/// Types of scaling recommendations
#[derive(Debug)]
enum RecommendationType {
    ScaleUp,
    ScaleDown,
    ScaleOut,
    ScaleIn,
    Migrate,
    Consolidate,
}

/// Expected impact of scaling
#[derive(Debug)]
struct ScalingImpact {
    pub performance_change: f64,
    pub cost_change: f64,
    pub utilization_change: f64,
    pub availability_impact: f64,
}

/// Prediction model for capacity planning
#[derive(Debug)]
struct PredictionModel {
    pub model_type: ModelType,
    pub training_data_size: usize,
    pub accuracy: f64,
    pub last_trained: Instant,
    pub parameters: serde_json::Value,
}

/// Types of prediction models
#[derive(Debug)]
enum ModelType {
    LinearRegression,
    TimeSeriesARIMA,
    NeuralNetwork,
    EnsembleModel,
    Custom(String),
}

impl ResourceAllocator {
    pub fn new(config: ResourceAllocatorConfig) -> Self {
        let metadata = AgentMetadata {
            id: AgentId::from_name("resource-allocator"),
            name: "Resource Allocator".to_string(),
            role: AgentRole::Executive,
            capabilities: vec![
                "resource-management".to_string(),
                "resource-optimization".to_string(),
                "capacity-planning".to_string(),
                "resource-monitoring".to_string(),
                "cost-optimization".to_string(),
                "auto-scaling".to_string(),
            ],
            version: "1.0.0".to_string(),
            cluster_assignment: Some("orchestration".to_string()),
            resource_requirements: ResourceRequirements {
                min_cpu: 0.5,
                min_memory: 512 * 1024 * 1024, // 512MB
                min_storage: 10 * 1024 * 1024,  // 10MB
                max_cpu: 2.0,
                max_memory: 2 * 1024 * 1024 * 1024, // 2GB
                max_storage: 1024 * 1024 * 1024,     // 1GB
            },
            health_check_interval: Duration::from_secs(30),
        };

        Self {
            metadata,
            state: RwLock::new(AgentState::Initializing),
            context: None,
            resource_manager: Arc::new(RwLock::new(ResourceManager::default())),
            optimizer: Arc::new(RwLock::new(ResourceOptimizer::default())),
            monitor: Arc::new(RwLock::new(ResourceMonitor::default())),
            capacity_planner: Arc::new(RwLock::new(CapacityPlanner::default())),
            config,
        }
    }

    /// Allocate resources to an agent
    pub async fn allocate_resources(
        &self,
        agent_id: AgentId,
        requirements: ResourceRequirements,
        priority: Priority,
    ) -> Result<ResourceAllocation> {
        let mut resource_manager = self.resource_manager.write().await;
        
        // Check if agent already has allocation
        if let Some(existing) = resource_manager.allocations.get(&agent_id) {
            tracing::warn!("Agent {} already has resource allocation", agent_id);
        }
        
        // Find suitable resources
        let capacity = self.convert_requirements_to_capacity(&requirements);
        let suitable_pool_key = self.find_suitable_resource_pool(&resource_manager, &capacity).await?;
        
        // Create allocation
        let allocation = ResourceAllocation {
            agent_id,
            allocated_resources: capacity.clone(),
            allocation_time: Instant::now(),
            last_updated: Instant::now(),
            allocation_priority: priority,
            constraints: Vec::new(),
            cost: 0.0, // Placeholder; calculate based on pool if needed
            utilization: ResourceUtilization::default(),
        };
        
        // Update resource pools using the key
        if let Some(pool) = resource_manager.available_resources.get_mut(&suitable_pool_key) {
            self.subtract_capacity(&mut pool.available_capacity, &capacity);
            self.add_capacity(&mut pool.allocated_capacity, &capacity);
            allocation.cost = pool.cost_per_unit * self.calculate_resource_units(&capacity);
        }
        
        resource_manager.allocations.insert(agent_id, allocation.clone());
        
        // Record allocation event
        resource_manager.allocation_history.push_back(AllocationEvent {
            event_type: AllocationEventType::Allocated,
            agent_id,
            resource_change: capacity,
            timestamp: Instant::now(),
            reason: "Resource allocation request".to_string(),
            cost_impact: allocation.cost,
        });
        
        tracing::info!("Allocated resources to agent {}", agent_id);
        Ok(allocation)
    }
    
    /// Deallocate resources from an agent
    pub async fn deallocate_resources(&self, agent_id: AgentId) -> Result<()> {
        let mut resource_manager = self.resource_manager.write().await;
        
        if let Some(allocation) = resource_manager.allocations.remove(&agent_id) {
            // Return resources to pools
            for (resource_type, pool) in resource_manager.available_resources.iter_mut() {
                self.add_capacity(&mut pool.available_capacity, &allocation.allocated_resources);
                self.subtract_capacity(&mut pool.allocated_capacity, &allocation.allocated_resources);
            }
            
            // Record deallocation event
            resource_manager.allocation_history.push_back(AllocationEvent {
                event_type: AllocationEventType::Deallocated,
                agent_id,
                resource_change: allocation.allocated_resources,
                timestamp: Instant::now(),
                reason: "Resource deallocation request".to_string(),
                cost_impact: -allocation.cost,
            });
            
            tracing::info!("Deallocated resources from agent {}", agent_id);
        }
        
        Ok(())
    }
    
    /// Optimize resource allocation across all agents
    pub async fn optimize_allocation(&self) -> Result<OptimizationResult> {
        let mut optimizer = self.optimizer.write().await;
        let resource_manager = self.resource_manager.read().await;
        
        let optimization_start = Instant::now();
        let mut actions_taken = Vec::new();
        let mut cost_savings = 0.0;
        let mut performance_gains = 0.0;
        
        // Analyze current allocations
        for (agent_id, allocation) in &resource_manager.allocations {
            // Check for underutilized resources
            if allocation.utilization.efficiency_score < 0.5 {
                // Consider scaling down
                tracing::debug!("Agent {} has low resource efficiency: {}", agent_id, allocation.utilization.efficiency_score);
            }
            
            // Check for overutilized resources
            if allocation.utilization.cpu_usage_percent > 90.0 {
                // Consider scaling up
                tracing::debug!("Agent {} has high CPU utilization: {}%", agent_id, allocation.utilization.cpu_usage_percent);
            }
        }
        
        // Create optimization result
        let result = OptimizationResult {
            optimization_time: optimization_start,
            strategy_used: optimizer.optimization_strategy.clone(),
            improvements: HashMap::new(),
            cost_savings,
            performance_gains,
            actions_taken,
        };
        
        optimizer.optimization_history.push_back(result.clone());
        
        // Keep limited history
        while optimizer.optimization_history.len() > 100 {
            optimizer.optimization_history.pop_front();
        }
        
        tracing::info!("Completed resource optimization");
        Ok(result)
    }
    
    /// Update resource utilization for an agent
    pub async fn update_utilization(
        &self,
        agent_id: AgentId,
        utilization: ResourceUtilization,
    ) -> Result<()> {
        let mut resource_manager = self.resource_manager.write().await;
        
        if let Some(allocation) = resource_manager.allocations.get_mut(&agent_id) {
            allocation.utilization = utilization.clone();
            allocation.last_updated = Instant::now();
        }
        
        // Update usage tracker
        resource_manager.usage_tracker.current_usage.insert(agent_id, utilization);
        
        Ok(())
    }
    
    /// Get current resource allocation for an agent
    pub async fn get_allocation(&self, agent_id: AgentId) -> Result<Option<ResourceAllocation>> {
        let resource_manager = self.resource_manager.read().await;
        Ok(resource_manager.allocations.get(&agent_id).cloned())
    }
    
    /// Get system-wide resource metrics
    pub async fn get_system_metrics(&self) -> Result<SystemResourceMetrics> {
        let resource_manager = self.resource_manager.read().await;
        let monitor = self.monitor.read().await;
        
        let total_capacity = self.calculate_total_capacity(&resource_manager);
        let allocated_capacity = self.calculate_allocated_capacity(&resource_manager);
        let utilization = self.calculate_system_utilization(&resource_manager);
        
        Ok(SystemResourceMetrics {
            total_capacity,
            allocated_capacity,
            available_capacity: self.subtract_capacities(&total_capacity, &allocated_capacity),
            system_utilization: utilization,
            active_agents: resource_manager.allocations.len(),
            active_alerts: monitor.active_alerts.len(),
            total_cost: resource_manager.allocations.values().map(|a| a.cost).sum(),
        })
    }

    /// Helper methods for resource calculations
    fn convert_requirements_to_capacity(&self, requirements: &ResourceRequirements) -> ResourceCapacity {
        ResourceCapacity {
            cpu_cores: requirements.max_cpu as f64,
            memory_bytes: requirements.max_memory,
            storage_bytes: requirements.max_storage,
            network_mbps: 100.0, // Default
            gpu_units: 0,
            custom_units: HashMap::new(),
        }
    }
    
    fn find_suitable_resource_pool(
        &self,
        resource_manager: &ResourceManager,
        _capacity: &ResourceCapacity,
    ) -> Result<String> {
        // Find the first available pool and return its key (simplified)
        resource_manager.available_resources.keys()
            .next()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No suitable resource pool found"))
    }
    
    fn calculate_resource_units(&self, capacity: &ResourceCapacity) -> f64 {
        capacity.cpu_cores + (capacity.memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
    
    fn add_capacity(&self, target: &mut ResourceCapacity, addition: &ResourceCapacity) {
        target.cpu_cores += addition.cpu_cores;
        target.memory_bytes += addition.memory_bytes;
        target.storage_bytes += addition.storage_bytes;
        target.network_mbps += addition.network_mbps;
        target.gpu_units += addition.gpu_units;
    }
    
    fn subtract_capacity(&self, target: &mut ResourceCapacity, subtraction: &ResourceCapacity) {
        target.cpu_cores -= subtraction.cpu_cores;
        target.memory_bytes = target.memory_bytes.saturating_sub(subtraction.memory_bytes);
        target.storage_bytes = target.storage_bytes.saturating_sub(subtraction.storage_bytes);
        target.network_mbps -= subtraction.network_mbps;
        target.gpu_units = target.gpu_units.saturating_sub(subtraction.gpu_units);
    }
    
    fn subtract_capacities(&self, a: &ResourceCapacity, b: &ResourceCapacity) -> ResourceCapacity {
        ResourceCapacity {
            cpu_cores: a.cpu_cores - b.cpu_cores,
            memory_bytes: a.memory_bytes.saturating_sub(b.memory_bytes),
            storage_bytes: a.storage_bytes.saturating_sub(b.storage_bytes),
            network_mbps: a.network_mbps - b.network_mbps,
            gpu_units: a.gpu_units.saturating_sub(b.gpu_units),
            custom_units: HashMap::new(),
        }
    }
    
    fn calculate_total_capacity(&self, resource_manager: &ResourceManager) -> ResourceCapacity {
        let mut total = ResourceCapacity::default();
        for pool in resource_manager.available_resources.values() {
            self.add_capacity(&mut total, &pool.total_capacity);
        }
        total
    }
    
    fn calculate_allocated_capacity(&self, resource_manager: &ResourceManager) -> ResourceCapacity {
        let mut allocated = ResourceCapacity::default();
        for allocation in resource_manager.allocations.values() {
            self.add_capacity(&mut allocated, &allocation.allocated_resources);
        }
        allocated
    }
    
    fn calculate_system_utilization(&self, resource_manager: &ResourceManager) -> ResourceUtilization {
        let mut total_utilization = ResourceUtilization::default();
        let mut count = 0;
        
        for allocation in resource_manager.allocations.values() {
            total_utilization.cpu_usage_percent += allocation.utilization.cpu_usage_percent;
            total_utilization.memory_usage_percent += allocation.utilization.memory_usage_percent;
            total_utilization.storage_usage_percent += allocation.utilization.storage_usage_percent;
            total_utilization.network_usage_percent += allocation.utilization.network_usage_percent;
            total_utilization.gpu_usage_percent += allocation.utilization.gpu_usage_percent;
            total_utilization.efficiency_score += allocation.utilization.efficiency_score;
            count += 1;
        }
        
        if count > 0 {
            let count_f = count as f64;
            total_utilization.cpu_usage_percent /= count_f;
            total_utilization.memory_usage_percent /= count_f;
            total_utilization.storage_usage_percent /= count_f;
            total_utilization.network_usage_percent /= count_f;
            total_utilization.gpu_usage_percent /= count_f;
            total_utilization.efficiency_score /= count_f;
        }
        
        total_utilization.last_measured = chrono::Utc::now();
        total_utilization
    }
}

/// System-wide resource metrics
#[derive(Debug)]
pub struct SystemResourceMetrics {
    pub total_capacity: ResourceCapacity,
    pub allocated_capacity: ResourceCapacity,
    pub available_capacity: ResourceCapacity,
    pub system_utilization: ResourceUtilization,
    pub active_agents: usize,
    pub active_alerts: usize,
    pub total_cost: f64,
}

impl ToString for ResourceType {
    fn to_string(&self) -> String {
        match self {
            ResourceType::CPU => "cpu".to_string(),
            ResourceType::Memory => "memory".to_string(),
            ResourceType::Storage => "storage".to_string(),
            ResourceType::Network => "network".to_string(),
            ResourceType::GPU => "gpu".to_string(),
            ResourceType::Custom(name) => name.clone(),
        }
    }
}

#[async_trait]
impl Agent for ResourceAllocator {
    fn metadata(&self) -> &AgentMetadata {
        &self.metadata
    }

    async fn state(&self) -> AgentState {
        self.state.read().await.clone()
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing Resource Allocator");
        
        // Initialize resource pools
        let mut resource_manager = self.resource_manager.write().await;
        self.initialize_resource_pools(&mut resource_manager).await?;
        
        // Initialize monitoring targets
        let mut monitor = self.monitor.write().await;
        self.initialize_monitoring(&mut monitor).await?;
        
        // Initialize optimization objectives
        let mut optimizer = self.optimizer.write().await;
        self.initialize_optimization_objectives(&mut optimizer).await?;
        
        *self.state.write().await = AgentState::Active;
        
        tracing::info!("Resource Allocator initialized successfully");
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting Resource Allocator");
        
        // Start resource monitoring
        let monitor = self.monitor.clone();
        let monitoring_interval = self.config.monitoring_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(monitoring_interval);
            loop {
                interval.tick().await;
                if let Err(e) = Self::monitor_resources(monitor.clone()).await {
                    tracing::error!("Resource monitoring failed: {}", e);
                }
            }
        });
        
        // Start optimization cycle
        let optimizer = self.optimizer.clone();
        let resource_manager = self.resource_manager.clone();
        let optimization_interval = self.config.optimization_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(optimization_interval);
            loop {
                interval.tick().await;
                if let Err(e) = Self::run_optimization_cycle(optimizer.clone(), resource_manager.clone()).await {
                    tracing::error!("Resource optimization failed: {}", e);
                }
            }
        });
        
        tracing::info!("Resource Allocator started successfully");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping Resource Allocator");
        
        *self.state.write().await = AgentState::Terminating;
        
        // TODO: Implement graceful shutdown
        // - Save allocation state
        // - Complete pending operations
        // - Clean up resources
        
        tracing::info!("Resource Allocator stopped successfully");
        Ok(())
    }

    async fn handle_message(&mut self, message: AgentMessage) -> Result<Option<AgentMessage>> {
        match message {
            AgentMessage::Request { id, from, task, .. } => {
                let result = self.execute_task(task).await?;
                
                Ok(Some(AgentMessage::Response {
                    id: crate::agents::MessageId::new(),
                    request_id: id,
                    from: self.metadata.id,
                    to: from,
                    result,
                }))
            }
            _ => Ok(None),
        }
    }

    async fn execute_task(&mut self, task: Task) -> Result<TaskResult> {
        let start_time = Instant::now();
        
        match task.name.as_str() {
            "allocate-resources" => {
                let agent_id = task.parameters.get("agent_id")
                    .and_then(|v| v.as_str())
                    .map(AgentId::from_name)
                    .ok_or_else(|| anyhow::anyhow!("Missing agent_id parameter"))?;
                
                // Parse resource requirements from parameters
                let requirements = ResourceRequirements {
                    min_cpu: 0.1,
                    min_memory: 128 * 1024 * 1024,
                    min_storage: 1024 * 1024,
                    max_cpu: 1.0,
                    max_memory: 1024 * 1024 * 1024,
                    max_storage: 100 * 1024 * 1024,
                };
                
                let allocation = self.allocate_resources(agent_id, requirements, Priority::Normal).await?;
                
                Ok(TaskResult {
                    task_id: task.id,
                    status: TaskStatus::Completed,
                    result: serde_json::json!({
                        "allocated": true,
                        "cost": allocation.cost,
                        "cpu_cores": allocation.allocated_resources.cpu_cores,
                        "memory_bytes": allocation.allocated_resources.memory_bytes,
                    }),
                    error: None,
                    execution_time: start_time.elapsed(),
                    resource_usage: ResourceUsage::default(),
                })
            }
            "get-metrics" => {
                let metrics = self.get_system_metrics().await?;
                
                Ok(TaskResult {
                    task_id: task.id,
                    status: TaskStatus::Completed,
                    result: serde_json::json!({
                        "total_cpu": metrics.total_capacity.cpu_cores,
                        "allocated_cpu": metrics.allocated_capacity.cpu_cores,
                        "available_cpu": metrics.available_capacity.cpu_cores,
                        "utilization": metrics.system_utilization.cpu_usage_percent,
                        "active_agents": metrics.active_agents,
                        "total_cost": metrics.total_cost,
                    }),
                    error: None,
                    execution_time: start_time.elapsed(),
                    resource_usage: ResourceUsage::default(),
                })
            }
            "optimize" => {
                let result = self.optimize_allocation().await?;
                
                Ok(TaskResult {
                    task_id: task.id,
                    status: TaskStatus::Completed,
                    result: serde_json::json!({
                        "optimized": true,
                        "cost_savings": result.cost_savings,
                        "performance_gains": result.performance_gains,
                        "actions_count": result.actions_taken.len(),
                    }),
                    error: None,
                    execution_time: start_time.elapsed(),
                    resource_usage: ResourceUsage::default(),
                })
            }
            _ => {
                Ok(TaskResult {
                    task_id: task.id,
                    status: TaskStatus::Failed("Resource allocation failed".to_string()),
                    result: serde_json::Value::Null,
                    error: Some(format!("Unknown task type: {}", task.name)),
                    execution_time: start_time.elapsed(),
                    resource_usage: ResourceUsage::default(),
                })
            }
        }
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let state = self.state.read().await;
        let resource_manager = self.resource_manager.read().await;
        let monitor = self.monitor.read().await;
        
        Ok(HealthStatus {
            agent_id: self.metadata.id,
            state: state.clone(),
            last_heartbeat: chrono::Utc::now(),
            cpu_usage: 10.0, // Placeholder
            memory_usage: 128 * 1024 * 1024, // 128MB placeholder
            task_queue_size: 0,
            completed_tasks: resource_manager.allocation_history.len() as u64,
            failed_tasks: 0, // Track this in real implementation
            average_response_time: Duration::from_millis(50),
        })
    }

    async fn update_config(&mut self, config: serde_json::Value) -> Result<()> {
        tracing::info!("Updating Resource Allocator configuration");
        
        // TODO: Parse and update configuration
        Ok(())
    }

    fn capabilities(&self) -> &[String] {
        &self.metadata.capabilities
    }
}

impl ResourceAllocator {
    /// Initialize resource pools
    async fn initialize_resource_pools(&self, resource_manager: &mut ResourceManager) -> Result<()> {
        // Create default resource pools
        let pools = vec![
            ResourcePool {
                resource_type: ResourceType::CPU,
                total_capacity: ResourceCapacity {
                    cpu_cores: 32.0,
                    memory_bytes: 0,
                    storage_bytes: 0,
                    network_mbps: 0.0,
                    gpu_units: 0,
                    custom_units: HashMap::new(),
                },
                available_capacity: ResourceCapacity {
                    cpu_cores: 32.0,
                    memory_bytes: 0,
                    storage_bytes: 0,
                    network_mbps: 0.0,
                    gpu_units: 0,
                    custom_units: HashMap::new(),
                },
                allocated_capacity: ResourceCapacity::default(),
                reserved_capacity: ResourceCapacity::default(),
                cost_per_unit: 0.10,
                provider: ResourceProvider::Local,
                location: "datacenter-1".to_string(),
                constraints: Vec::new(),
                last_updated: chrono::Utc::now(),
            },
            ResourcePool {
                resource_type: ResourceType::Memory,
                total_capacity: ResourceCapacity {
                    cpu_cores: 0.0,
                    memory_bytes: 128 * 1024 * 1024 * 1024, // 128GB
                    storage_bytes: 0,
                    network_mbps: 0.0,
                    gpu_units: 0,
                    custom_units: HashMap::new(),
                },
                available_capacity: ResourceCapacity {
                    cpu_cores: 0.0,
                    memory_bytes: 128 * 1024 * 1024 * 1024,
                    storage_bytes: 0,
                    network_mbps: 0.0,
                    gpu_units: 0,
                    custom_units: HashMap::new(),
                },
                allocated_capacity: ResourceCapacity::default(),
                reserved_capacity: ResourceCapacity::default(),
                cost_per_unit: 0.05,
                provider: ResourceProvider::Local,
                location: "datacenter-1".to_string(),
                constraints: Vec::new(),
                last_updated: chrono::Utc::now(),
            },
        ];
        
        for pool in pools {
            resource_manager.available_resources.insert(pool.resource_type.to_string(), pool);
        }
        
        tracing::info!("Initialized {} resource pools", resource_manager.available_resources.len());
        Ok(())
    }
    
    /// Initialize monitoring configuration
    async fn initialize_monitoring(&self, monitor: &mut ResourceMonitor) -> Result<()> {
        // Set up alert thresholds
        let thresholds = vec![
            AlertThreshold {
                metric_name: "cpu_utilization".to_string(),
                warning_threshold: 70.0,
                critical_threshold: 90.0,
                duration_threshold: Duration::from_secs(300),
            },
            AlertThreshold {
                metric_name: "memory_utilization".to_string(),
                warning_threshold: 75.0,
                critical_threshold: 85.0,
                duration_threshold: Duration::from_secs(300),
            },
        ];
        
        for threshold in thresholds {
            monitor.alert_thresholds.insert(threshold.metric_name.clone(), threshold);
        }
        
        tracing::info!("Initialized resource monitoring");
        Ok(())
    }
    
    /// Initialize optimization objectives
    async fn initialize_optimization_objectives(&self, optimizer: &mut ResourceOptimizer) -> Result<()> {
        let objectives = vec![
            OptimizationObjective {
                objective_type: ObjectiveType::MaximizeUtilization,
                weight: 0.4,
                target_value: 75.0,
                tolerance: 5.0,
            },
            OptimizationObjective {
                objective_type: ObjectiveType::MinimizeCost,
                weight: 0.3,
                target_value: 0.0,
                tolerance: 0.1,
            },
            OptimizationObjective {
                objective_type: ObjectiveType::MaximizePerformance,
                weight: 0.3,
                target_value: 90.0,
                tolerance: 5.0,
            },
        ];
        
        optimizer.objectives = objectives;
        
        tracing::info!("Initialized optimization objectives");
        Ok(())
    }
    
    /// Monitor resources (background task)
    async fn monitor_resources(monitor: Arc<RwLock<ResourceMonitor>>) -> Result<()> {
        let mut monitor = monitor.write().await;
        
        // TODO: Collect real resource metrics
        // This would involve:
        // 1. Querying system metrics
        // 2. Updating monitoring metrics
        // 3. Checking alert thresholds
        // 4. Generating alerts if needed
        
        tracing::debug!("Resource monitoring cycle completed");
        Ok(())
    }
    
    /// Run optimization cycle (background task)
    async fn run_optimization_cycle(
        optimizer: Arc<RwLock<ResourceOptimizer>>,
        resource_manager: Arc<RwLock<ResourceManager>>,
    ) -> Result<()> {
        let optimizer = optimizer.read().await;
        let resource_manager = resource_manager.read().await;
        
        // TODO: Implement optimization algorithm
        // This would involve:
        // 1. Analyzing current resource usage
        // 2. Identifying optimization opportunities
        // 3. Applying optimization strategies
        // 4. Recording optimization results
        
        tracing::debug!("Optimization cycle completed");
        Ok(())
    }
}
