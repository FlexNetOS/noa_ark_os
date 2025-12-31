use std::collections::HashMap;
use std::path::Path;
use std::fs;
use anyhow::{Result, Context};
use serde_json;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::data_models::*;
use crate::types::*;

/// Utility functions for agent management
pub mod agent_utils {
    use super::*;
    
    /// Generate a new unique agent ID
    pub fn generate_agent_id() -> AgentId {
        Uuid::new_v4()
    }
    
    /// Create agent metadata with default values
    pub fn create_agent_metadata(name: String, agent_type: String) -> AgentMetadata {
        AgentMetadata {
            id: generate_agent_id(),
            name,
            agent_type,
            version: "1.0.0".to_string(),
            capabilities: Vec::new(),
            status: AgentStatus::Inactive,
            health_status: HealthStatus::Unknown,
            created_at: Utc::now(),
            last_updated: Utc::now(),
            resource_requirements: ResourceRequirements::default(),
            tags: HashMap::new(),
        }
    }
    
    /// Validate agent metadata
    pub fn validate_agent_metadata(metadata: &AgentMetadata) -> Result<()> {
        if metadata.name.is_empty() {
            return Err(AgentAsKitError::ValidationFailed("Agent name cannot be empty".to_string()).into());
        }
        
        if metadata.agent_type.is_empty() {
            return Err(AgentAsKitError::ValidationFailed("Agent type cannot be empty".to_string()).into());
        }
        
        Ok(())
    }
    
    /// Check if agent is healthy
    pub fn is_agent_healthy(metadata: &AgentMetadata) -> bool {
        matches!(metadata.health_status, HealthStatus::Healthy) &&
        matches!(metadata.status, AgentStatus::Active | AgentStatus::Busy)
    }
}

/// Utility functions for task management
pub mod task_utils {
    use super::*;
    
    /// Generate a new unique task ID
    pub fn generate_task_id() -> TaskId {
        Uuid::new_v4()
    }
    
    /// Create a new task with default values
    pub fn create_task(name: String, task_type: String) -> Task {
        Task {
            id: generate_task_id(),
            name,
            description: String::new(),
            task_type,
            priority: Priority::Normal,
            status: TaskStatus::Pending,
            assigned_agent: None,
            dependencies: Vec::new(),
            input_data: serde_json::Value::Null,
            output_data: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            timeout: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
            tags: HashMap::new(),
        }
    }
    
    /// Check if task is in terminal state
    pub fn is_task_terminal(status: &TaskStatus) -> bool {
        matches!(status, TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled)
    }
    
    /// Check if task can be retried
    pub fn can_retry_task(task: &Task) -> bool {
        task.retry_count < task.max_retries && 
        matches!(task.status, TaskStatus::Failed)
    }
    
    /// Calculate task duration
    pub fn calculate_task_duration(task: &Task) -> Option<chrono::Duration> {
        match (task.started_at, task.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

/// Utility functions for configuration management
pub mod config_utils {
    use super::*;
    
    /// Load configuration from file
    pub fn load_config_from_file<P: AsRef<Path>>(path: P) -> Result<AgentAsKitConfig> {
        let content = fs::read_to_string(path)
            .context("Failed to read configuration file")?;
        
        let config: AgentAsKitConfig = serde_json::from_str(&content)
            .context("Failed to parse configuration JSON")?;
        
        validate_config(&config)?;
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save_config_to_file<P: AsRef<Path>>(config: &AgentAsKitConfig, path: P) -> Result<()> {
        validate_config(config)?;
        
        let content = serde_json::to_string_pretty(config)
            .context("Failed to serialize configuration")?;
        
        fs::write(path, content)
            .context("Failed to write configuration file")?;
        
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate_config(config: &AgentAsKitConfig) -> Result<()> {
        if config.system_name.is_empty() {
            return Err(AgentAsKitError::ConfigurationError("System name cannot be empty".to_string()).into());
        }
        
        if config.max_agents == 0 {
            return Err(AgentAsKitError::ConfigurationError("Max agents must be greater than 0".to_string()).into());
        }
        
        if config.max_concurrent_tasks == 0 {
            return Err(AgentAsKitError::ConfigurationError("Max concurrent tasks must be greater than 0".to_string()).into());
        }
        
        Ok(())
    }
    
    /// Merge configurations with override taking precedence
    pub fn merge_configs(base: AgentAsKitConfig, override_config: AgentAsKitConfig) -> AgentAsKitConfig {
        AgentAsKitConfig {
            system_name: if override_config.system_name != "AgentAsKit" { 
                override_config.system_name 
            } else { 
                base.system_name 
            },
            version: override_config.version,
            environment: override_config.environment,
            max_agents: override_config.max_agents,
            agent_timeout_seconds: override_config.agent_timeout_seconds,
            agent_heartbeat_interval_seconds: override_config.agent_heartbeat_interval_seconds,
            max_concurrent_tasks: override_config.max_concurrent_tasks,
            task_timeout_seconds: override_config.task_timeout_seconds,
            task_retry_limit: override_config.task_retry_limit,
            message_queue_size: override_config.message_queue_size,
            message_timeout_seconds: override_config.message_timeout_seconds,
            broadcast_timeout_seconds: override_config.broadcast_timeout_seconds,
            health_check_interval_seconds: override_config.health_check_interval_seconds,
            health_check_timeout_seconds: override_config.health_check_timeout_seconds,
            metrics_collection_interval_seconds: override_config.metrics_collection_interval_seconds,
            memory_limit_mb: override_config.memory_limit_mb.or(base.memory_limit_mb),
            cpu_limit_percent: override_config.cpu_limit_percent.or(base.cpu_limit_percent),
            disk_limit_mb: override_config.disk_limit_mb.or(base.disk_limit_mb),
            network_limit_mbps: override_config.network_limit_mbps.or(base.network_limit_mbps),
            flexnetos: override_config.flexnetos,
            noa: override_config.noa,
            security: override_config.security,
            logging: override_config.logging,
            custom: {
                let mut merged = base.custom;
                merged.extend(override_config.custom);
                merged
            },
        }
    }
}

/// Utility functions for health monitoring
pub mod health_utils {
    use super::*;
    
    /// Calculate overall system health based on agent health statuses
    pub fn calculate_system_health(agent_healths: &HashMap<AgentId, HealthStatus>) -> HealthStatus {
        if agent_healths.is_empty() {
            return HealthStatus::Unknown;
        }
        
        let mut healthy = 0;
        let mut degraded = 0;
        let mut needs_repair = 0;
        let mut critical = 0;
        let mut unknown = 0;
        
        for status in agent_healths.values() {
            match status {
                HealthStatus::Healthy => healthy += 1,
                HealthStatus::Degraded => degraded += 1,
                HealthStatus::NeedsRepair => needs_repair += 1,
                HealthStatus::Critical => critical += 1,
                HealthStatus::Unknown => unknown += 1,
            }
        }
        
        let total = agent_healths.len();
        
        // If more than 10% are critical, system is critical
        if critical as f64 / total as f64 > 0.1 {
            return HealthStatus::Critical;
        }
        
        // If more than 25% need repair, system needs repair
        if needs_repair as f64 / total as f64 > 0.25 {
            return HealthStatus::NeedsRepair;
        }
        
        // If more than 50% are degraded, system is degraded
        if degraded as f64 / total as f64 > 0.5 {
            return HealthStatus::Degraded;
        }
        
        // If more than 80% are healthy, system is healthy
        if healthy as f64 / total as f64 > 0.8 {
            return HealthStatus::Healthy;
        }
        
        // Otherwise, system is degraded
        HealthStatus::Degraded
    }
    
    /// Check if health status indicates immediate attention is needed
    pub fn needs_immediate_attention(status: &HealthStatus) -> bool {
        matches!(status, HealthStatus::Critical | HealthStatus::NeedsRepair)
    }
    
    /// Get health status priority for sorting
    pub fn health_status_priority(status: &HealthStatus) -> u8 {
        match status {
            HealthStatus::Critical => 5,
            HealthStatus::NeedsRepair => 4,
            HealthStatus::Degraded => 3,
            HealthStatus::Unknown => 2,
            HealthStatus::Healthy => 1,
        }
    }
}

/// Utility functions for metrics and performance
pub mod metrics_utils {
    use super::*;
    
    /// Calculate average response time from metrics
    pub fn calculate_average_response_time(metrics: &[SystemMetrics]) -> f64 {
        if metrics.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = metrics.iter().map(|m| m.response_time_ms).sum();
        sum / metrics.len() as f64
    }
    
    /// Calculate resource utilization percentage
    pub fn calculate_resource_utilization(used: u64, total: u64) -> f64 {
        if total == 0 {
            return 0.0;
        }
        (used as f64 / total as f64) * 100.0
    }
    
    /// Check if resource usage is within acceptable limits
    pub fn is_resource_usage_acceptable(
        cpu_percent: f64, 
        memory_percent: f64, 
        disk_percent: f64
    ) -> bool {
        cpu_percent < 80.0 && memory_percent < 85.0 && disk_percent < 90.0
    }
    
    /// Generate performance alert if thresholds are exceeded
    pub fn check_performance_thresholds(metrics: &SystemMetrics) -> Vec<String> {
        let mut alerts = Vec::new();
        
        if metrics.cpu_usage_percent > 90.0 {
            alerts.push(format!("High CPU usage: {:.1}%", metrics.cpu_usage_percent));
        }
        
        if metrics.memory_usage_percent > 95.0 {
            alerts.push(format!("High memory usage: {:.1}%", metrics.memory_usage_percent));
        }
        
        if metrics.disk_usage_percent > 95.0 {
            alerts.push(format!("High disk usage: {:.1}%", metrics.disk_usage_percent));
        }
        
        if metrics.response_time_ms > 5000.0 {
            alerts.push(format!("High response time: {:.0}ms", metrics.response_time_ms));
        }
        
        if metrics.error_count_last_hour > 100 {
            alerts.push(format!("High error rate: {} errors in last hour", metrics.error_count_last_hour));
        }
        
        alerts
    }
}

/// Utility functions for FlexNetOS integration
pub mod flexnetos_utils {
    use super::*;
    
    /// Validate WASM module compatibility
    pub fn validate_wasm_module(module_path: &str) -> Result<()> {
        // Basic validation - in a real implementation, this would use wasmtime
        if !module_path.ends_with(".wasm") && !module_path.ends_with(".wat") {
            return Err(AgentAsKitError::ValidationFailed(
                "Invalid WASM module file extension".to_string()
            ).into());
        }
        Ok(())
    }
    
    /// Create capability token for FlexNetOS sandbox
    pub fn create_capability_token(
        capability: String,
        granted_to: AgentId,
        granted_by: AgentId,
        lifetime_seconds: u64,
    ) -> CapabilityToken {
        CapabilityToken {
            token_id: Uuid::new_v4(),
            capability,
            granted_to,
            granted_by,
            valid_until: Utc::now() + chrono::Duration::seconds(lifetime_seconds as i64),
            permissions: Vec::new(),
            restrictions: HashMap::new(),
            signature: "placeholder_signature".to_string(), // In real implementation, this would be properly signed
        }
    }
    
    /// Check if capability token is valid and not expired
    pub fn is_capability_token_valid(token: &CapabilityToken) -> bool {
        Utc::now() < token.valid_until
    }
}

/// Utility functions for NOA deployment management
pub mod noa_utils {
    use super::*;
    
    /// Parse NOA manifest file
    pub fn parse_noa_manifest<P: AsRef<Path>>(path: P) -> Result<Vec<DeploymentManifestEntry>> {
        let content = fs::read_to_string(path)
            .context("Failed to read NOA manifest file")?;
        
        let manifest: serde_json::Value = serde_json::from_str(&content)
            .context("Failed to parse NOA manifest JSON")?;
        
        // Parse the manifest structure - this would be more sophisticated in practice
        let entries = manifest.get("agents")
            .and_then(|agents| agents.as_array())
            .context("Invalid manifest structure: missing agents array")?;
        
        let mut deployment_entries = Vec::new();
        
        for entry in entries {
            let agent_name = entry.get("name")
                .and_then(|n| n.as_str())
                .context("Missing agent name in manifest")?;
            
            let agent_type = entry.get("type")
                .and_then(|t| t.as_str())
                .context("Missing agent type in manifest")?;
            
            let deployment_entry = DeploymentManifestEntry {
                agent_id: Uuid::new_v4(),
                agent_name: agent_name.to_string(),
                agent_type: agent_type.to_string(),
                deployment_config: entry.clone(),
                health_checks: Vec::new(), // Would be parsed from manifest
                scaling_policy: ScalingPolicy {
                    min_instances: 1,
                    max_instances: 10,
                    target_cpu_percent: Some(70.0),
                    target_memory_percent: Some(80.0),
                    scale_up_cooldown_seconds: 300,
                    scale_down_cooldown_seconds: 600,
                },
                dependencies: Vec::new(), // Would be parsed from manifest
            };
            
            deployment_entries.push(deployment_entry);
        }
        
        Ok(deployment_entries)
    }
    
    /// Validate deployment manifest entry
    pub fn validate_deployment_manifest_entry(entry: &DeploymentManifestEntry) -> Result<()> {
        if entry.agent_name.is_empty() {
            return Err(AgentAsKitError::ValidationFailed("Agent name cannot be empty".to_string()).into());
        }
        
        if entry.agent_type.is_empty() {
            return Err(AgentAsKitError::ValidationFailed("Agent type cannot be empty".to_string()).into());
        }
        
        if entry.scaling_policy.min_instances > entry.scaling_policy.max_instances {
            return Err(AgentAsKitError::ValidationFailed(
                "Min instances cannot be greater than max instances".to_string()
            ).into());
        }
        
        Ok(())
    }
}