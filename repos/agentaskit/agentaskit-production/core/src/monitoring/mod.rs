use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use tracing::{info, warn, error, debug};

/// System metrics collected for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub memory_total_mb: u64,
    pub disk_usage_mb: u64,
    pub disk_total_mb: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub active_connections: u32,
    pub uptime_seconds: u64,
}

/// Agent-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub agent_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub status: String,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub tasks_in_progress: u32,
    pub average_response_time_ms: f64,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u64,
    pub message_queue_size: u32,
    pub last_activity: DateTime<Utc>,
}

/// Task execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub task_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
    pub success: bool,
    pub agent_id: Uuid,
    pub task_type: String,
    pub priority: String,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_time_ms: u64,
    pub memory_peak_mb: u64,
    pub disk_io_mb: u64,
    pub network_io_mb: u64,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Degraded,
    Offline,
}

/// Alert levels for monitoring
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertLevel {
    Critical = 0,
    Warning = 1,
    Info = 2,
    Debug = 3,
}

/// System alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub level: AlertLevel,
    pub title: String,
    pub description: String,
    pub source: String,
    pub metadata: serde_json::Value,
    pub acknowledged: bool,
    pub resolved: bool,
}

/// Performance threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub cpu_usage_warning: f64,
    pub cpu_usage_critical: f64,
    pub memory_usage_warning: f64,
    pub memory_usage_critical: f64,
    pub response_time_warning_ms: f64,
    pub response_time_critical_ms: f64,
    pub task_failure_rate_warning: f64,
    pub task_failure_rate_critical: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            cpu_usage_warning: 70.0,
            cpu_usage_critical: 90.0,
            memory_usage_warning: 80.0,
            memory_usage_critical: 95.0,
            response_time_warning_ms: 5000.0,
            response_time_critical_ms: 10000.0,
            task_failure_rate_warning: 5.0,
            task_failure_rate_critical: 15.0,
        }
    }
}

/// Metrics collector and monitoring system
pub struct MetricsCollector {
    system_metrics: Arc<RwLock<Vec<SystemMetrics>>>,
    agent_metrics: Arc<RwLock<HashMap<Uuid, Vec<AgentMetrics>>>>,
    task_metrics: Arc<RwLock<Vec<TaskMetrics>>>,
    alerts: Arc<RwLock<Vec<Alert>>>,
    thresholds: Arc<RwLock<PerformanceThresholds>>,
    collection_enabled: Arc<RwLock<bool>>,
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        Self {
            system_metrics: Arc::clone(&self.system_metrics),
            agent_metrics: Arc::clone(&self.agent_metrics),
            task_metrics: Arc::clone(&self.task_metrics),
            alerts: Arc::clone(&self.alerts),
            thresholds: Arc::clone(&self.thresholds),
            collection_enabled: Arc::clone(&self.collection_enabled),
        }
    }
}

impl MetricsCollector {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            system_metrics: Arc::new(RwLock::new(Vec::new())),
            agent_metrics: Arc::new(RwLock::new(HashMap::new())),
            task_metrics: Arc::new(RwLock::new(Vec::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            thresholds: Arc::new(RwLock::new(PerformanceThresholds::default())),
            collection_enabled: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting metrics collection");
        
        *self.collection_enabled.write().await = true;
        
        // Start periodic metrics collection
        self.start_system_metrics_collection().await?;
        self.start_health_monitoring().await?;
        self.start_alert_processing().await?;
        
        Ok(())
    }

    async fn start_system_metrics_collection(&self) -> Result<()> {
        let system_metrics = Arc::clone(&self.system_metrics);
        let collection_enabled = Arc::clone(&self.collection_enabled);

        tokio::spawn(async move {
            info!("System metrics collection started");
            
            while *collection_enabled.read().await {
                // Collect system metrics
                let metrics = Self::collect_system_metrics().await;
                
                // Store metrics (keep last 1000 entries)
                {
                    let mut metrics_store = system_metrics.write().await;
                    metrics_store.push(metrics);
                    
                    if metrics_store.len() > 1000 {
                        metrics_store.remove(0);
                    }
                }
                
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
            
            info!("System metrics collection stopped");
        });

        Ok(())
    }

    async fn start_health_monitoring(&self) -> Result<()> {
        let system_metrics = Arc::clone(&self.system_metrics);
        let agent_metrics = Arc::clone(&self.agent_metrics);
        let alerts = Arc::clone(&self.alerts);
        let thresholds = Arc::clone(&self.thresholds);
        let collection_enabled = Arc::clone(&self.collection_enabled);

        tokio::spawn(async move {
            info!("Health monitoring started");
            
            while *collection_enabled.read().await {
                // Check system health
                if let Err(e) = Self::check_system_health(
                    &system_metrics,
                    &agent_metrics,
                    &alerts,
                    &thresholds,
                ).await {
                    error!("Health check failed: {}", e);
                }
                
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            }
            
            info!("Health monitoring stopped");
        });

        Ok(())
    }

    async fn start_alert_processing(&self) -> Result<()> {
        let alerts = Arc::clone(&self.alerts);
        let collection_enabled = Arc::clone(&self.collection_enabled);

        tokio::spawn(async move {
            info!("Alert processing started");
            
            while *collection_enabled.read().await {
                // Process and potentially auto-resolve alerts
                {
                    let mut alerts_store = alerts.write().await;
                    let now = Utc::now();
                    
                    // Auto-resolve old info alerts (1 hour)
                    for alert in alerts_store.iter_mut() {
                        if alert.level == AlertLevel::Info 
                            && !alert.resolved 
                            && now.signed_duration_since(alert.timestamp).num_hours() >= 1 {
                            alert.resolved = true;
                        }
                    }
                    
                    // Remove very old alerts (7 days)
                    alerts_store.retain(|alert| {
                        now.signed_duration_since(alert.timestamp).num_days() < 7
                    });
                }
                
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
            
            info!("Alert processing stopped");
        });

        Ok(())
    }

    async fn collect_system_metrics() -> SystemMetrics {
        // In a real implementation, this would collect actual system metrics
        // For now, we'll return mock data
        SystemMetrics {
            timestamp: Utc::now(),
            cpu_usage_percent: 25.0,
            memory_usage_mb: 2048,
            memory_total_mb: 8192,
            disk_usage_mb: 10240,
            disk_total_mb: 102400,
            network_bytes_sent: 1024000,
            network_bytes_received: 2048000,
            active_connections: 150,
            uptime_seconds: 86400,
        }
    }

    pub async fn record_agent_metrics(&self, metrics: AgentMetrics) -> Result<()> {
        let mut agent_metrics = self.agent_metrics.write().await;
        
        let agent_history = agent_metrics.entry(metrics.agent_id).or_insert_with(Vec::new);
        agent_history.push(metrics);
        
        // Keep only the last 100 metrics per agent
        if agent_history.len() > 100 {
            agent_history.remove(0);
        }
        
        Ok(())
    }

    pub async fn record_task_metrics(&self, metrics: TaskMetrics) -> Result<()> {
        let mut task_metrics = self.task_metrics.write().await;
        task_metrics.push(metrics);
        
        // Keep only the last 10000 task metrics
        if task_metrics.len() > 10000 {
            task_metrics.remove(0);
        }
        
        Ok(())
    }

    async fn check_system_health(
        system_metrics: &Arc<RwLock<Vec<SystemMetrics>>>,
        agent_metrics: &Arc<RwLock<HashMap<Uuid, Vec<AgentMetrics>>>>,
        alerts: &Arc<RwLock<Vec<Alert>>>,
        thresholds: &Arc<RwLock<PerformanceThresholds>>,
    ) -> Result<()> {
        let thresholds = thresholds.read().await.clone();
        
        // Check latest system metrics
        {
            let metrics = system_metrics.read().await;
            if let Some(latest) = metrics.last() {
                let cpu_usage = latest.cpu_usage_percent;
                let memory_usage = (latest.memory_usage_mb as f64 / latest.memory_total_mb as f64) * 100.0;
                
                // Check CPU usage
                if cpu_usage >= thresholds.cpu_usage_critical {
                    Self::create_alert(
                        alerts,
                        AlertLevel::Critical,
                        "High CPU Usage".to_string(),
                        format!("CPU usage is at {:.1}%", cpu_usage),
                        "system_monitor".to_string(),
                        serde_json::json!({ "cpu_usage": cpu_usage }),
                    ).await;
                } else if cpu_usage >= thresholds.cpu_usage_warning {
                    Self::create_alert(
                        alerts,
                        AlertLevel::Warning,
                        "Elevated CPU Usage".to_string(),
                        format!("CPU usage is at {:.1}%", cpu_usage),
                        "system_monitor".to_string(),
                        serde_json::json!({ "cpu_usage": cpu_usage }),
                    ).await;
                }
                
                // Check memory usage
                if memory_usage >= thresholds.memory_usage_critical {
                    Self::create_alert(
                        alerts,
                        AlertLevel::Critical,
                        "High Memory Usage".to_string(),
                        format!("Memory usage is at {:.1}%", memory_usage),
                        "system_monitor".to_string(),
                        serde_json::json!({ "memory_usage": memory_usage }),
                    ).await;
                } else if memory_usage >= thresholds.memory_usage_warning {
                    Self::create_alert(
                        alerts,
                        AlertLevel::Warning,
                        "Elevated Memory Usage".to_string(),
                        format!("Memory usage is at {:.1}%", memory_usage),
                        "system_monitor".to_string(),
                        serde_json::json!({ "memory_usage": memory_usage }),
                    ).await;
                }
            }
        }

        // Check agent health
        {
            let agent_metrics = agent_metrics.read().await;
            let now = Utc::now();
            
            for (agent_id, metrics_history) in agent_metrics.iter() {
                if let Some(latest) = metrics_history.last() {
                    // Check if agent has been silent for too long
                    if now.signed_duration_since(latest.last_activity).num_minutes() > 5 {
                        Self::create_alert(
                            alerts,
                            AlertLevel::Warning,
                            "Agent Not Responding".to_string(),
                            format!("Agent {} has not been active for over 5 minutes", agent_id),
                            "agent_monitor".to_string(),
                            serde_json::json!({ "agent_id": agent_id }),
                        ).await;
                    }
                    
                    // Check task failure rate
                    let total_tasks = latest.tasks_completed + latest.tasks_failed;
                    if total_tasks > 0 {
                        let failure_rate = (latest.tasks_failed as f64 / total_tasks as f64) * 100.0;
                        
                        if failure_rate >= thresholds.task_failure_rate_critical {
                            Self::create_alert(
                                alerts,
                                AlertLevel::Critical,
                                "High Task Failure Rate".to_string(),
                                format!("Agent {} has {:.1}% task failure rate", agent_id, failure_rate),
                                "agent_monitor".to_string(),
                                serde_json::json!({ 
                                    "agent_id": agent_id,
                                    "failure_rate": failure_rate 
                                }),
                            ).await;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn create_alert(
        alerts: &Arc<RwLock<Vec<Alert>>>,
        level: AlertLevel,
        title: String,
        description: String,
        source: String,
        metadata: serde_json::Value,
    ) {
        let alert = Alert {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level,
            title: title.clone(),
            description,
            source,
            metadata,
            acknowledged: false,
            resolved: false,
        };

        // Check if similar alert already exists
        {
            let existing_alerts = alerts.read().await;
            let similar_exists = existing_alerts.iter().any(|a| {
                a.title == title && !a.resolved && 
                Utc::now().signed_duration_since(a.timestamp).num_minutes() < 30
            });
            
            if similar_exists {
                return; // Don't create duplicate alerts
            }
        }

        alerts.write().await.push(alert);
        info!("Created alert: {}", title);
    }

    pub async fn collect_metrics(&self) -> Result<()> {
        if !*self.collection_enabled.read().await {
            return Ok(());
        }

        // This is called by the orchestrator periodically
        // Additional metrics collection logic can be added here
        
        Ok(())
    }

    pub async fn get_system_health(&self) -> Result<HealthStatus> {
        let alerts = self.alerts.read().await;
        let recent_critical = alerts.iter()
            .filter(|a| a.level == AlertLevel::Critical && !a.resolved)
            .count();
        let recent_warnings = alerts.iter()
            .filter(|a| a.level == AlertLevel::Warning && !a.resolved)
            .count();

        Ok(if recent_critical > 0 {
            HealthStatus::Critical
        } else if recent_warnings > 5 {
            HealthStatus::Degraded
        } else if recent_warnings > 0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        })
    }

    pub async fn get_alerts(&self, level: Option<AlertLevel>) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        
        if let Some(filter_level) = level {
            alerts.iter()
                .filter(|alert| alert.level == filter_level)
                .cloned()
                .collect()
        } else {
            alerts.clone()
        }
    }

    pub async fn acknowledge_alert(&self, alert_id: Uuid) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            info!("Acknowledged alert: {}", alert.title);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Alert not found"))
        }
    }

    pub async fn resolve_alert(&self, alert_id: Uuid) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolved = true;
            info!("Resolved alert: {}", alert.title);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Alert not found"))
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down metrics collector");
        
        *self.collection_enabled.write().await = false;
        
        info!("Metrics collector shutdown complete");
        Ok(())
    }
}