//! Unified Execution Module
//! 
//! This module combines and enhances the execution capabilities from rustecosys2
//! while providing a robust task execution framework.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

use crate::{ArkComponent, ArkOsConfig, HealthStatus};

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub max_concurrent_tasks: usize,
    pub default_timeout: Duration,
    pub retry_attempts: u32,
    pub worker_pool_size: usize,
    pub resource_limits: ResourceLimits,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            default_timeout: Duration::from_secs(300),
            retry_attempts: 3,
            worker_pool_size: 16,
            resource_limits: ResourceLimits::default(),
        }
    }
}

/// Resource limits for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_cores: f32,
    pub max_storage_mb: u64,
    pub max_network_mbps: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 4096, // 4GB
            max_cpu_cores: 4.0,
            max_storage_mb: 10240, // 10GB
            max_network_mbps: 1000, // 1Gbps
        }
    }
}

/// Execution metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub active_tasks: usize,
    pub average_execution_time: Duration,
    pub throughput_per_second: f64,
    pub resource_utilization: ExecutionResourceUsage,
}

/// Resource utilization during execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionResourceUsage {
    pub cpu_utilization: f64,
    pub memory_usage_mb: u64,
    pub storage_usage_mb: u64,
    pub network_usage_mbps: f64,
}

/// Task execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub task_id: Uuid,
    pub environment: HashMap<String, String>,
    pub working_directory: Option<String>,
    pub timeout: Duration,
    pub resource_allocation: ResourceAllocation,
}

/// Resource allocation for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub cpu_cores: f32,
    pub memory_mb: u64,
    pub storage_mb: u64,
    pub network_mbps: u64,
}

impl Default for ResourceAllocation {
    fn default() -> Self {
        Self {
            cpu_cores: 0.5,
            memory_mb: 256,
            storage_mb: 1024,
            network_mbps: 100,
        }
    }
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTaskResult {
    pub task_id: Uuid,
    pub status: TaskExecutionStatus,
    pub output: serde_json::Value,
    pub error_message: Option<String>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub execution_duration: Duration,
    pub resource_usage: ExecutionResourceUsage,
    pub exit_code: Option<i32>,
}

/// Task execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Timeout,
    Cancelled,
    Retrying,
}

/// Execution engine for running tasks
#[derive(Clone)]
pub struct ExecutionEngine {
    config: ExecutionConfig,
    worker_semaphore: Arc<Semaphore>,
    active_tasks: Arc<RwLock<HashMap<Uuid, ExecutionContext>>>,
    metrics: Arc<RwLock<ExecutionMetrics>>,
    task_results: Arc<RwLock<HashMap<Uuid, ExecutionTaskResult>>>,
}

impl ExecutionEngine {
    /// Create new execution engine
    pub fn new(config: ExecutionConfig) -> Self {
        let worker_semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));
        
        Self {
            config,
            worker_semaphore,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(ExecutionMetrics::default())),
            task_results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Execute a single task
    pub async fn execute_task<F, Fut>(&self, 
        task_id: Uuid, 
        context: ExecutionContext,
        task_fn: F
    ) -> Result<ExecutionTaskResult> 
    where
        F: FnOnce(ExecutionContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<serde_json::Value>> + Send + 'static,
    {
        let start_time = chrono::Utc::now();
        
        // Acquire semaphore permit
        let _permit = self.worker_semaphore.acquire().await?;
        
        // Register active task
        {
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.insert(task_id, context.clone());
        }

        let mut result = ExecutionTaskResult {
            task_id,
            status: TaskExecutionStatus::Running,
            output: serde_json::Value::Null,
            error_message: None,
            start_time,
            end_time: start_time, // Will be updated
            execution_duration: Duration::from_secs(0),
            resource_usage: ExecutionResourceUsage::default(),
            exit_code: None,
        };

        // Execute task with timeout
        let execution_result = tokio::time::timeout(
            context.timeout,
            task_fn(context.clone())
        ).await;

        let end_time = chrono::Utc::now();
        let execution_duration = end_time.signed_duration_since(start_time)
            .to_std().unwrap_or(Duration::from_secs(0));

        // Process result
        match execution_result {
            Ok(Ok(output)) => {
                result.status = TaskExecutionStatus::Completed;
                result.output = output;
                result.exit_code = Some(0);
            }
            Ok(Err(error)) => {
                result.status = TaskExecutionStatus::Failed;
                result.error_message = Some(error.to_string());
                result.exit_code = Some(1);
            }
            Err(_) => {
                result.status = TaskExecutionStatus::Timeout;
                result.error_message = Some("Task execution timeout".to_string());
                result.exit_code = Some(124); // Timeout exit code
            }
        }

        result.end_time = end_time;
        result.execution_duration = execution_duration;

        // Remove from active tasks
        {
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.remove(&task_id);
        }

        // Store result
        {
            let mut task_results = self.task_results.write().await;
            task_results.insert(task_id, result.clone());
        }

        // Update metrics
        self.update_metrics(&result).await;

        tracing::info!("Task {} completed with status: {:?}", task_id, result.status);
        Ok(result)
    }

    /// Execute multiple tasks in parallel
    pub async fn execute_parallel<F, Fut>(&self,
        tasks: Vec<(Uuid, ExecutionContext)>,
        task_fn: F
    ) -> Result<Vec<ExecutionTaskResult>>
    where
        F: Fn(Uuid, ExecutionContext) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<serde_json::Value>> + Send + 'static,
    {
        let mut handles = Vec::new();

        for (task_id, context) in tasks {
            let engine = self.clone();
            let task_fn_clone = task_fn.clone();
            
            let handle = tokio::spawn(async move {
                engine.execute_task(task_id, context.clone(), move |ctx| {
                    task_fn_clone(task_id, ctx)
                }).await
            });
            
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(error)) => {
                    tracing::error!("Task execution failed: {}", error);
                    // Create error result
                    results.push(ExecutionTaskResult {
                        task_id: Uuid::new_v4(),
                        status: TaskExecutionStatus::Failed,
                        output: serde_json::Value::Null,
                        error_message: Some(error.to_string()),
                        start_time: chrono::Utc::now(),
                        end_time: chrono::Utc::now(),
                        execution_duration: Duration::from_secs(0),
                        resource_usage: ExecutionResourceUsage::default(),
                        exit_code: Some(1),
                    });
                }
                Err(join_error) => {
                    tracing::error!("Task join failed: {}", join_error);
                    results.push(ExecutionTaskResult {
                        task_id: Uuid::new_v4(),
                        status: TaskExecutionStatus::Failed,
                        output: serde_json::Value::Null,
                        error_message: Some(format!("Task join error: {}", join_error)),
                        start_time: chrono::Utc::now(),
                        end_time: chrono::Utc::now(),
                        execution_duration: Duration::from_secs(0),
                        resource_usage: ExecutionResourceUsage::default(),
                        exit_code: Some(1),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Get task result by ID
    pub async fn get_task_result(&self, task_id: Uuid) -> Option<ExecutionTaskResult> {
        let task_results = self.task_results.read().await;
        task_results.get(&task_id).cloned()
    }

    /// Get all active tasks
    pub async fn get_active_tasks(&self) -> Vec<(Uuid, ExecutionContext)> {
        let active_tasks = self.active_tasks.read().await;
        active_tasks.iter().map(|(id, ctx)| (*id, ctx.clone())).collect()
    }

    /// Get execution metrics
    pub async fn get_metrics(&self) -> ExecutionMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Cancel a running task
    pub async fn cancel_task(&self, task_id: Uuid) -> Result<()> {
        // Remove from active tasks (simplified cancellation)
        let mut active_tasks = self.active_tasks.write().await;
        if let Some(_) = active_tasks.remove(&task_id) {
            tracing::info!("Task {} cancelled", task_id);
            
            // Create cancelled result
            let result = ExecutionTaskResult {
                task_id,
                status: TaskExecutionStatus::Cancelled,
                output: serde_json::Value::Null,
                error_message: Some("Task was cancelled".to_string()),
                start_time: chrono::Utc::now(),
                end_time: chrono::Utc::now(),
                execution_duration: Duration::from_secs(0),
                resource_usage: ExecutionResourceUsage::default(),
                exit_code: Some(130), // SIGINT exit code
            };

            let mut task_results = self.task_results.write().await;
            task_results.insert(task_id, result);
        }

        Ok(())
    }

    /// Update execution metrics
    async fn update_metrics(&self, result: &ExecutionTaskResult) {
        let mut metrics = self.metrics.write().await;
        
        metrics.total_tasks += 1;
        
        match result.status {
            TaskExecutionStatus::Completed => metrics.completed_tasks += 1,
            TaskExecutionStatus::Failed | TaskExecutionStatus::Timeout => metrics.failed_tasks += 1,
            _ => {}
        }

        // Update average execution time
        if metrics.total_tasks > 0 {
            let total_time = metrics.average_execution_time.as_secs() * (metrics.total_tasks - 1) 
                + result.execution_duration.as_secs();
            metrics.average_execution_time = Duration::from_secs(total_time / metrics.total_tasks);
        }

        // Calculate throughput (tasks per second)
        if result.execution_duration.as_secs() > 0 {
            metrics.throughput_per_second = 1.0 / result.execution_duration.as_secs_f64();
        }

        // Update resource utilization
        metrics.resource_utilization.cpu_utilization = 
            (metrics.resource_utilization.cpu_utilization + result.resource_usage.cpu_utilization) / 2.0;
        metrics.resource_utilization.memory_usage_mb = 
            metrics.resource_utilization.memory_usage_mb.max(result.resource_usage.memory_usage_mb);
    }
}

#[async_trait]
impl ArkComponent for ExecutionEngine {
    fn name(&self) -> &str {
        "execution_engine"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let metrics = self.get_metrics().await;
        let active_tasks = self.get_active_tasks().await;
        
        // Check if we're not overloaded
        if active_tasks.len() >= self.config.max_concurrent_tasks {
            return Ok(HealthStatus::Degraded);
        }

        // Check success rate
        let success_rate = if metrics.total_tasks > 0 {
            metrics.completed_tasks as f64 / metrics.total_tasks as f64
        } else {
            1.0
        };

        if success_rate > 0.9 {
            Ok(HealthStatus::Healthy)
        } else if success_rate > 0.7 {
            Ok(HealthStatus::Degraded)
        } else {
            Ok(HealthStatus::Unhealthy)
        }
    }

    async fn initialize(&mut self, _config: ArkOsConfig) -> Result<()> {
        tracing::info!("Initializing Execution Engine");
        tracing::info!("Max concurrent tasks: {}", self.config.max_concurrent_tasks);
        tracing::info!("Worker pool size: {}", self.config.worker_pool_size);
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting Execution Engine");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping Execution Engine");
        
        // Cancel all active tasks
        let active_task_ids: Vec<Uuid> = {
            let active_tasks = self.active_tasks.read().await;
            active_tasks.keys().cloned().collect()
        };

        for task_id in active_task_ids {
            self.cancel_task(task_id).await?;
        }

        Ok(())
    }

    async fn get_metrics(&self) -> Result<crate::ComponentMetrics> {
        let exec_metrics = self.get_metrics().await;
        let active_tasks = self.get_active_tasks().await;
        
        Ok(crate::ComponentMetrics {
            health: self.health_check().await?,
            uptime: Duration::from_secs(0), // Placeholder
            cpu_usage: exec_metrics.resource_utilization.cpu_utilization,
            memory_usage: exec_metrics.resource_utilization.memory_usage_mb * 1024 * 1024, // Convert to bytes
            active_tasks: active_tasks.len(),
            completed_tasks: exec_metrics.completed_tasks,
            error_count: exec_metrics.failed_tasks,
        })
    }

    async fn update_config(&mut self, _config: ArkOsConfig) -> Result<()> {
        Ok(())
    }
}

/// Utility functions for common task execution patterns
pub mod utils {
    use super::*;

    /// Execute a shell command
    pub async fn execute_shell_command(
        command: String,
        args: Vec<String>,
        working_dir: Option<String>,
    ) -> Result<serde_json::Value> {
        use tokio::process::Command;

        let mut cmd = Command::new(command);
        cmd.args(args);
        
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output().await?;
        
        Ok(serde_json::json!({
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
            "exit_code": output.status.code().unwrap_or(-1)
        }))
    }

    /// Execute a Python script
    pub async fn execute_python_script(
        script_path: String,
        args: Vec<String>,
    ) -> Result<serde_json::Value> {
        let mut cmd_args = vec![script_path];
        cmd_args.extend(args);
        
        execute_shell_command("python3".to_string(), cmd_args, None).await
    }

    /// Create a basic execution context
    pub fn create_execution_context(task_id: Uuid, timeout: Option<Duration>) -> ExecutionContext {
        ExecutionContext {
            task_id,
            environment: std::env::vars().collect(),
            working_directory: std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
            timeout: timeout.unwrap_or(Duration::from_secs(300)),
            resource_allocation: ResourceAllocation::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execution_engine_creation() {
        let config = ExecutionConfig::default();
        let engine = ExecutionEngine::new(config);
        
        let metrics = engine.get_metrics().await;
        assert_eq!(metrics.total_tasks, 0);
        assert_eq!(metrics.completed_tasks, 0);
    }

    #[tokio::test]
    async fn test_task_execution() {
        let config = ExecutionConfig::default();
        let engine = ExecutionEngine::new(config);
        let task_id = Uuid::new_v4();
        let context = utils::create_execution_context(task_id, None);

        let result = engine.execute_task(task_id, context, |_ctx| async {
            Ok(serde_json::json!({"result": "success"}))
        }).await.unwrap();

        assert_eq!(result.task_id, task_id);
        assert_eq!(result.status, TaskExecutionStatus::Completed);
        assert_eq!(result.exit_code, Some(0));
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let config = ExecutionConfig::default();
        let engine = ExecutionEngine::new(config);

        let tasks = vec![
            (Uuid::new_v4(), utils::create_execution_context(Uuid::new_v4(), None)),
            (Uuid::new_v4(), utils::create_execution_context(Uuid::new_v4(), None)),
        ];

        let results = engine.execute_parallel(tasks, |task_id, _ctx| async move {
            Ok(serde_json::json!({"task_id": task_id.to_string(), "result": "success"}))
        }).await.unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.status == TaskExecutionStatus::Completed));
    }
}