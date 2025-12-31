use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, broadcast};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::{AgentId, AgentMessage, AgentRegistry, BroadcastScope, MessageId, Priority};

/// Inter-Agent Communication Protocol (IACP) - handles all communication between agents
pub struct CommunicationManager {
    /// Registry of all agents in the system
    registry: Arc<RwLock<AgentRegistry>>,
    
    /// Message routing table
    routing_table: Arc<RwLock<RoutingTable>>,
    
    /// Message queues for each agent
    agent_queues: Arc<RwLock<HashMap<AgentId, AgentMessageQueue>>>,
    
    /// Broadcast channels for topic-based communication
    broadcast_channels: Arc<RwLock<HashMap<String, broadcast::Sender<AgentMessage>>>>,
    
    /// Message delivery tracking
    delivery_tracker: Arc<RwLock<DeliveryTracker>>,
    
    /// Performance metrics
    metrics: Arc<RwLock<CommunicationMetrics>>,
    
    /// Configuration
    config: CommunicationConfig,
}

/// Communication configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationConfig {
    /// Maximum message queue size per agent
    pub max_queue_size: usize,
    
    /// Default message timeout
    pub default_timeout: Duration,
    
    /// Maximum retry attempts for failed deliveries
    pub max_retries: usize,
    
    /// Retry backoff duration
    pub retry_backoff: Duration,
    
    /// Heartbeat interval for agents
    pub heartbeat_interval: Duration,
    
    /// Dead letter queue size
    pub dead_letter_queue_size: usize,
    
    /// Enable message persistence
    pub enable_persistence: bool,
    
    /// Enable message encryption
    pub enable_encryption: bool,
}

impl Default for CommunicationConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10000,
            default_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_backoff: Duration::from_millis(100),
            heartbeat_interval: Duration::from_secs(10),
            dead_letter_queue_size: 1000,
            enable_persistence: false,
            enable_encryption: false,
        }
    }
}

/// Message routing table for efficient delivery
#[derive(Debug, Default)]
struct RoutingTable {
    /// Direct agent routes
    direct_routes: HashMap<AgentId, mpsc::UnboundedSender<AgentMessage>>,
    
    /// Topic subscriptions for broadcast messages
    topic_subscriptions: HashMap<String, Vec<AgentId>>,
    
    /// Capability-based routing
    capability_routes: HashMap<String, Vec<AgentId>>,
}

/// Message queue for each agent with priority handling
struct AgentMessageQueue {
    /// Priority queues (Emergency=0, Maintenance=5)
    priority_queues: [VecDeque<AgentMessage>; 6],
    
    /// Total message count
    total_messages: usize,
    
    /// Message sender for the agent
    sender: mpsc::UnboundedSender<AgentMessage>,
}

impl AgentMessageQueue {
    fn new(sender: mpsc::UnboundedSender<AgentMessage>) -> Self {
        Self {
            priority_queues: Default::default(),
            total_messages: 0,
            sender,
        }
    }
    
    fn enqueue(&mut self, message: AgentMessage) -> Result<()> {
        let priority = match &message {
            AgentMessage::Request { priority, .. } => *priority as usize,
            AgentMessage::Alert { severity, .. } => match severity {
                super::AlertSeverity::Emergency => 0,
                super::AlertSeverity::Critical => 1,
                super::AlertSeverity::Warning => 2,
                super::AlertSeverity::Info => 3,
                super::AlertSeverity::Debug => 4,
            },
            _ => Priority::Normal as usize,
        };
        
        self.priority_queues[priority].push_back(message);
        self.total_messages += 1;
        
        // Try to send next message
        self.try_send_next()?;
        
        Ok(())
    }
    
    fn try_send_next(&mut self) -> Result<bool> {
        for queue in &mut self.priority_queues {
            if let Some(message) = queue.pop_front() {
                if self.sender.send(message).is_ok() {
                    self.total_messages -= 1;
                    return Ok(true);
                } else {
                    // Channel closed, agent likely disconnected
                    return Err(anyhow::anyhow!("Agent channel closed"));
                }
            }
        }
        Ok(false)
    }
    
    fn size(&self) -> usize {
        self.total_messages
    }
}

/// Message delivery tracking for reliability
#[derive(Debug, Default)]
struct DeliveryTracker {
    /// Pending deliveries with retry information
    pending_deliveries: HashMap<MessageId, PendingDelivery>,
    
    /// Delivery confirmations
    confirmed_deliveries: HashMap<MessageId, chrono::DateTime<chrono::Utc>>,
    
    /// Failed deliveries for analysis
    failed_deliveries: HashMap<MessageId, DeliveryFailure>,
}

#[derive(Debug)]
struct PendingDelivery {
    message: AgentMessage,
    attempts: usize,
    last_attempt: chrono::DateTime<chrono::Utc>,
    timeout: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug)]
struct DeliveryFailure {
    message_id: MessageId,
    attempts: usize,
    error: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Communication performance metrics
#[derive(Debug, Default, Serialize)]
pub struct CommunicationMetrics {
    /// Total messages sent
    pub messages_sent: u64,
    
    /// Total messages delivered
    pub messages_delivered: u64,
    
    /// Total messages failed
    pub messages_failed: u64,
    
    /// Average delivery latency
    pub avg_delivery_latency: Duration,
    
    /// Message throughput (messages/second)
    pub throughput: f64,
    
    /// Queue utilization by priority
    pub queue_utilization: HashMap<String, f64>,
    
    /// Agent connection status
    pub connected_agents: usize,
}

impl CommunicationManager {
    pub fn new(config: CommunicationConfig) -> Self {
        Self {
            registry: Arc::new(RwLock::new(AgentRegistry::new())),
            routing_table: Arc::new(RwLock::new(RoutingTable::default())),
            agent_queues: Arc::new(RwLock::new(HashMap::new())),
            broadcast_channels: Arc::new(RwLock::new(HashMap::new())),
            delivery_tracker: Arc::new(RwLock::new(DeliveryTracker::default())),
            metrics: Arc::new(RwLock::new(CommunicationMetrics::default())),
            config,
        }
    }
    
    /// Register an agent with the communication system
    pub async fn register_agent(
        &self,
        agent_id: AgentId,
        sender: mpsc::UnboundedSender<AgentMessage>,
    ) -> Result<()> {
        let mut routing_table = self.routing_table.write().await;
        let mut agent_queues = self.agent_queues.write().await;
        
        // Add to routing table
        routing_table.direct_routes.insert(agent_id, sender.clone());
        
        // Create message queue
        let queue = AgentMessageQueue::new(sender);
        agent_queues.insert(agent_id, queue);
        
        tracing::info!("Registered agent {} for communication", agent_id.0);
        Ok(())
    }
    
    /// Deregister an agent from the communication system
    pub async fn deregister_agent(&self, agent_id: AgentId) -> Result<()> {
        let mut routing_table = self.routing_table.write().await;
        let mut agent_queues = self.agent_queues.write().await;
        
        // Remove from routing table
        routing_table.direct_routes.remove(&agent_id);
        
        // Remove capabilities routing
        for agents in routing_table.capability_routes.values_mut() {
            agents.retain(|&id| id != agent_id);
        }
        
        // Remove topic subscriptions
        for agents in routing_table.topic_subscriptions.values_mut() {
            agents.retain(|&id| id != agent_id);
        }
        
        // Remove message queue
        agent_queues.remove(&agent_id);
        
        tracing::info!("Deregistered agent {} from communication", agent_id.0);
        Ok(())
    }
    
    /// Subscribe an agent to a topic for broadcast messages
    pub async fn subscribe_to_topic(&self, agent_id: AgentId, topic: String) -> Result<()> {
        let mut routing_table = self.routing_table.write().await;
        
        routing_table
            .topic_subscriptions
            .entry(topic.clone())
            .or_insert_with(Vec::new)
            .push(agent_id);
        
        tracing::debug!("Agent {} subscribed to topic '{}'", agent_id.0, topic);
        Ok(())
    }
    
    /// Send a message to specific agent
    pub async fn send_message(&self, message: AgentMessage) -> Result<()> {
        let message_id = match &message {
            AgentMessage::Request { id, .. } => *id,
            AgentMessage::Response { id, .. } => *id,
            AgentMessage::Broadcast { id, .. } => *id,
            AgentMessage::Alert { id, .. } => *id,
            AgentMessage::Heartbeat { id, .. } => *id,
            AgentMessage::Registration { id, .. } => *id,
        };
        
        match &message {
            AgentMessage::Broadcast { scope, .. } => {
                self.handle_broadcast_message(message).await?;
            }
            _ => {
                let target_agent = self.get_target_agent(&message)?;
                self.deliver_message(target_agent, message).await?;
            }
        }
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.messages_sent += 1;
        
        Ok(())
    }
    
    /// Handle broadcast message distribution
    async fn handle_broadcast_message(&self, message: AgentMessage) -> Result<()> {
        if let AgentMessage::Broadcast { scope, topic, .. } = &message {
            let target_agents = self.resolve_broadcast_scope(scope).await?;
            
            // Special handling for topic-based broadcasts
            if !topic.is_empty() {
                let routing_table = self.routing_table.read().await;
                if let Some(subscribers) = routing_table.topic_subscriptions.get(topic) {
                    for &agent_id in subscribers {
                        let agent_message = message.clone();
                        self.deliver_message(agent_id, agent_message).await?;
                    }
                }
                return Ok(());
            }
            
            // Send to all matching agents
            for agent_id in target_agents {
                let agent_message = message.clone();
                self.deliver_message(agent_id, agent_message).await?;
            }
        }
        
        Ok(())
    }
    
    /// Resolve broadcast scope to list of agent IDs
    async fn resolve_broadcast_scope(&self, scope: &BroadcastScope) -> Result<Vec<AgentId>> {
        let registry = self.registry.read().await;
        
        let agents = match scope {
            BroadcastScope::All => {
                registry.all_agents().into_iter().map(|meta| meta.id).collect()
            }
            BroadcastScope::Role(role) => {
                registry.find_by_role(role)
            }
            BroadcastScope::Cluster(cluster) => {
                registry.find_by_cluster(cluster)
            }
            BroadcastScope::Capability(capability) => {
                registry.find_by_capability(capability)
            }
            BroadcastScope::Custom(agents) => {
                agents.clone()
            }
        };
        
        Ok(agents)
    }
    
    /// Deliver message to specific agent
    async fn deliver_message(&self, agent_id: AgentId, message: AgentMessage) -> Result<()> {
        let mut agent_queues = self.agent_queues.write().await;
        
        if let Some(queue) = agent_queues.get_mut(&agent_id) {
            if queue.size() < self.config.max_queue_size {
                queue.enqueue(message)?;
                
                // Update metrics
                let mut metrics = self.metrics.write().await;
                metrics.messages_delivered += 1;
            } else {
                // Queue full, handle overflow
                tracing::warn!("Agent {} message queue full, dropping message", agent_id.0);
                return Err(anyhow::anyhow!("Agent queue full"));
            }
        } else {
            tracing::warn!("Agent {} not found for message delivery", agent_id.0);
            return Err(anyhow::anyhow!("Agent not found"));
        }
        
        Ok(())
    }
    
    /// Get target agent from message
    fn get_target_agent(&self, message: &AgentMessage) -> Result<AgentId> {
        match message {
            AgentMessage::Request { to, .. } => Ok(*to),
            AgentMessage::Response { to, .. } => Ok(*to),
            AgentMessage::Alert { .. } => {
                // Alerts typically go to monitoring agents
                // For now, return error - should be handled differently
                Err(anyhow::anyhow!("Alert messages should use broadcast"))
            }
            AgentMessage::Heartbeat { .. } => {
                // Heartbeats go to health monitoring
                Err(anyhow::anyhow!("Heartbeat messages should use broadcast"))
            }
            AgentMessage::Registration { .. } => {
                // Registration goes to registry manager
                Err(anyhow::anyhow!("Registration messages should use broadcast"))
            }
            AgentMessage::Broadcast { .. } => {
                Err(anyhow::anyhow!("Broadcast messages handled separately"))
            }
        }
    }
    
    /// Get communication metrics
    pub async fn get_metrics(&self) -> CommunicationMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
    
    /// Health check for communication system
    pub async fn health_check(&self) -> Result<CommunicationHealth> {
        let routing_table = self.routing_table.read().await;
        let agent_queues = self.agent_queues.read().await;
        let metrics = self.metrics.read().await;
        
        let total_queue_size: usize = agent_queues.values().map(|q| q.size()).sum();
        let max_queue_utilization = agent_queues
            .values()
            .map(|q| q.size() as f64 / self.config.max_queue_size as f64)
            .fold(0.0, f64::max);
        
        Ok(CommunicationHealth {
            connected_agents: routing_table.direct_routes.len(),
            total_queue_size,
            max_queue_utilization,
            messages_sent: metrics.messages_sent,
            messages_delivered: metrics.messages_delivered,
            messages_failed: metrics.messages_failed,
            avg_delivery_latency: metrics.avg_delivery_latency,
        })
    }
    
    /// Start background tasks for communication management
    pub async fn start_background_tasks(&self) -> Result<()> {
        // Start heartbeat processor
        let heartbeat_manager = HeartbeatManager::new(
            self.registry.clone(),
            self.config.heartbeat_interval,
        );
        tokio::spawn(async move {
            if let Err(e) = heartbeat_manager.run().await {
                tracing::error!("Heartbeat manager failed: {}", e);
            }
        });
        
        // Start delivery retry processor
        let retry_processor = DeliveryRetryProcessor::new(
            self.delivery_tracker.clone(),
            self.config.max_retries,
            self.config.retry_backoff,
        );
        tokio::spawn(async move {
            if let Err(e) = retry_processor.run().await {
                tracing::error!("Delivery retry processor failed: {}", e);
            }
        });
        
        // Start metrics collector
        let metrics_collector = MetricsCollector::new(self.metrics.clone());
        tokio::spawn(async move {
            if let Err(e) = metrics_collector.run().await {
                tracing::error!("Metrics collector failed: {}", e);
            }
        });
        
        tracing::info!("Communication background tasks started");
        Ok(())
    }
}

/// Communication system health status
#[derive(Debug, Serialize)]
pub struct CommunicationHealth {
    pub connected_agents: usize,
    pub total_queue_size: usize,
    pub max_queue_utilization: f64,
    pub messages_sent: u64,
    pub messages_delivered: u64,
    pub messages_failed: u64,
    pub avg_delivery_latency: Duration,
}

/// Heartbeat manager for agent health monitoring
struct HeartbeatManager {
    registry: Arc<RwLock<AgentRegistry>>,
    interval: Duration,
}

impl HeartbeatManager {
    fn new(registry: Arc<RwLock<AgentRegistry>>, interval: Duration) -> Self {
        Self { registry, interval }
    }
    
    async fn run(&self) -> Result<()> {
        let mut interval_timer = tokio::time::interval(self.interval);
        
        loop {
            interval_timer.tick().await;
            
            // Check for stale agents (no recent heartbeat)
            let mut registry = self.registry.write().await;
            let stale_threshold = chrono::Utc::now() - chrono::Duration::from_std(self.interval * 3).unwrap();
            
            let mut stale_agents = Vec::new();
            for (agent_id, health) in registry.health_status.iter() {
                if health.last_heartbeat < stale_threshold {
                    stale_agents.push(*agent_id);
                }
            }
            
            for agent_id in stale_agents {
                tracing::warn!("Agent {} appears to be stale, removing from registry", agent_id.0);
                // Note: In production, this should trigger more sophisticated recovery
                registry.deregister_agent(agent_id).await?;
            }
        }
    }
}

/// Delivery retry processor for failed messages
struct DeliveryRetryProcessor {
    delivery_tracker: Arc<RwLock<DeliveryTracker>>,
    max_retries: usize,
    retry_backoff: Duration,
}

impl DeliveryRetryProcessor {
    fn new(
        delivery_tracker: Arc<RwLock<DeliveryTracker>>,
        max_retries: usize,
        retry_backoff: Duration,
    ) -> Self {
        Self {
            delivery_tracker,
            max_retries,
            retry_backoff,
        }
    }
    
    async fn run(&self) -> Result<()> {
        let mut interval = tokio::time::interval(self.retry_backoff);
        
        loop {
            interval.tick().await;
            
            let mut tracker = self.delivery_tracker.write().await;
            let now = chrono::Utc::now();
            
            // Find messages ready for retry
            let mut retry_messages = Vec::new();
            for (message_id, pending) in tracker.pending_deliveries.iter_mut() {
                if pending.attempts < self.max_retries {
                    if (now - pending.last_attempt).num_seconds() >= self.retry_backoff.as_secs() as i64 {
                        retry_messages.push(*message_id);
                    }
                }
            }
            
            // Process retries (in a real implementation, this would re-queue the messages)
            for message_id in retry_messages {
                if let Some(pending) = tracker.pending_deliveries.get_mut(&message_id) {
                    pending.attempts += 1;
                    pending.last_attempt = now;
                    
                    tracing::debug!("Retrying message {} (attempt {})", message_id.0, pending.attempts);
                    
                    if pending.attempts >= self.max_retries {
                        // Move to failed deliveries
                        let failure = DeliveryFailure {
                            message_id,
                            attempts: pending.attempts,
                            error: "Max retries exceeded".to_string(),
                            timestamp: now,
                        };
                        tracker.failed_deliveries.insert(message_id, failure);
                        tracker.pending_deliveries.remove(&message_id);
                    }
                }
            }
        }
    }
}

/// Metrics collector for communication statistics
struct MetricsCollector {
    metrics: Arc<RwLock<CommunicationMetrics>>,
}

impl MetricsCollector {
    fn new(metrics: Arc<RwLock<CommunicationMetrics>>) -> Self {
        Self { metrics }
    }
    
    async fn run(&self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            // Update throughput calculation
            let mut metrics = self.metrics.write().await;
            let _current_time = chrono::Utc::now();
            
            // Calculate messages per second (simplified)
            metrics.throughput = metrics.messages_delivered as f64 / 60.0;
            
            tracing::debug!("Communication metrics updated: throughput = {:.2} msg/s", metrics.throughput);
        }
    }
}
