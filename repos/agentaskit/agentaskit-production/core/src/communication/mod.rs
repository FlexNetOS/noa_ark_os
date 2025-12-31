use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, broadcast};
use uuid::Uuid;
use tracing::{info, warn, error, debug};

/// Message types for inter-agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Broadcast,
    Alert,
    Heartbeat,
    Registration,
}

/// Message priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Emergency = 0,
    Critical = 1,
    High = 2,
    Medium = 3,
    Normal = 4,
    Low = 5,
    Background = 6,
}

/// Agent communication message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub from: Uuid,
    pub to: Option<Uuid>, // None for broadcast messages
    pub message_type: MessageType,
    pub priority: Priority,
    pub payload: serde_json::Value,
    pub correlation_id: Option<Uuid>, // For request-response pairing
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub timeout: Option<chrono::DateTime<chrono::Utc>>,
}

impl Message {
    pub fn new(
        from: Uuid,
        to: Option<Uuid>,
        message_type: MessageType,
        priority: Priority,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to,
            message_type,
            priority,
            payload,
            correlation_id: None,
            timestamp: chrono::Utc::now(),
            timeout: None,
        }
    }

    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout = Some(chrono::Utc::now() + chrono::Duration::seconds(timeout_seconds as i64));
        self
    }

    pub fn as_response(mut self, request_message: &Message) -> Self {
        self.message_type = MessageType::Response;
        self.correlation_id = Some(request_message.id);
        self.to = Some(request_message.from);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(timeout) = self.timeout {
            chrono::Utc::now() > timeout
        } else {
            false
        }
    }
}

/// Agent communication channel
pub struct AgentChannel {
    agent_id: Uuid,
    message_receiver: mpsc::UnboundedReceiver<Message>,
    broadcast_receiver: broadcast::Receiver<Message>,
}

impl AgentChannel {
    pub async fn receive_message(&mut self) -> Option<Message> {
        tokio::select! {
            msg = self.message_receiver.recv() => msg,
            msg = self.broadcast_receiver.recv() => msg.ok(),
        }
    }
}

/// Message broker for handling inter-agent communication
pub struct MessageBroker {
    agent_channels: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Message>>>>,
    broadcast_sender: broadcast::Sender<Message>,
    message_queue: Arc<RwLock<Vec<Message>>>,
    running: Arc<RwLock<bool>>,
}

impl Clone for MessageBroker {
    fn clone(&self) -> Self {
        Self {
            agent_channels: Arc::clone(&self.agent_channels),
            broadcast_sender: self.broadcast_sender.clone(),
            message_queue: Arc::clone(&self.message_queue),
            running: Arc::clone(&self.running),
        }
    }
}

impl MessageBroker {
    pub async fn new() -> Result<Self> {
        let (broadcast_sender, _) = broadcast::channel(1000);
        
        Ok(Self {
            agent_channels: Arc::new(RwLock::new(HashMap::new())),
            broadcast_sender,
            message_queue: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting message broker");
        
        *self.running.write().await = true;
        
        // Start message processing loop
        self.start_message_processor().await?;
        
        Ok(())
    }

    async fn start_message_processor(&self) -> Result<()> {
        let message_queue = Arc::clone(&self.message_queue);
        let agent_channels = Arc::clone(&self.agent_channels);
        let broadcast_sender = self.broadcast_sender.clone();
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            info!("Message processor started");
            
            while *running.read().await {
                let messages_to_process = {
                    let mut queue = message_queue.write().await;
                    let messages = queue.clone();
                    queue.clear();
                    messages
                };

                for message in messages_to_process {
                    if message.is_expired() {
                        warn!("Dropping expired message: {}", message.id);
                        continue;
                    }

                    match message.message_type {
                        MessageType::Broadcast => {
                            if let Err(e) = broadcast_sender.send(message.clone()) {
                                error!("Failed to broadcast message: {}", e);
                            }
                        }
                        _ => {
                            if let Some(to) = message.to {
                                let channels = agent_channels.read().await;
                                if let Some(sender) = channels.get(&to) {
                                    if let Err(e) = sender.send(message.clone()) {
                                        error!("Failed to send message to agent {}: {}", to, e);
                                    }
                                } else {
                                    warn!("Agent {} not found for message delivery", to);
                                }
                            }
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            
            info!("Message processor stopped");
        });

        Ok(())
    }

    pub async fn register_agent(&self, agent_id: Uuid) -> Result<AgentChannel> {
        info!("Registering agent {} with message broker", agent_id);
        
        let (sender, receiver) = mpsc::unbounded_channel();
        let broadcast_receiver = self.broadcast_sender.subscribe();
        
        self.agent_channels.write().await.insert(agent_id, sender);
        
        Ok(AgentChannel {
            agent_id,
            message_receiver: receiver,
            broadcast_receiver,
        })
    }

    pub async fn unregister_agent(&self, agent_id: Uuid) -> Result<()> {
        info!("Unregistering agent {} from message broker", agent_id);
        
        self.agent_channels.write().await.remove(&agent_id);
        
        Ok(())
    }

    pub async fn send_message(&self, message: Message) -> Result<()> {
        debug!("Queuing message: {} -> {:?}", message.from, message.to);
        
        // Validate message
        if message.to.is_none() && message.message_type != MessageType::Broadcast {
            return Err(anyhow::anyhow!("Non-broadcast message must have a recipient"));
        }

        // Add to processing queue
        self.message_queue.write().await.push(message);
        
        Ok(())
    }

    pub async fn send_request(&self, request: Message) -> Result<Message> {
        if request.message_type != MessageType::Request {
            return Err(anyhow::anyhow!("Message type must be Request"));
        }

        // Send the request
        self.send_message(request.clone()).await?;

        // TODO: Implement response waiting mechanism
        // For now, return a mock response
        let response = Message::new(
            request.to.unwrap_or_default(),
            Some(request.from),
            MessageType::Response,
            Priority::Normal,
            serde_json::json!({"status": "received"}),
        ).as_response(&request);

        Ok(response)
    }

    pub async fn broadcast(&self, message: Message) -> Result<()> {
        if message.message_type != MessageType::Broadcast {
            return Err(anyhow::anyhow!("Message type must be Broadcast"));
        }

        debug!("Broadcasting message from agent {}", message.from);
        
        self.send_message(message).await
    }

    pub async fn send_alert(&self, from: Uuid, alert_level: Priority, payload: serde_json::Value) -> Result<()> {
        let alert = Message::new(
            from,
            None, // Broadcast to all
            MessageType::Alert,
            alert_level,
            payload,
        );

        info!("Sending alert from agent {}: {:?}", from, alert.priority);
        self.broadcast(alert).await
    }

    pub async fn get_connection_count(&self) -> usize {
        self.agent_channels.read().await.len()
    }

    pub async fn get_queue_size(&self) -> usize {
        self.message_queue.read().await.len()
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down message broker");
        
        *self.running.write().await = false;
        
        // Clear all channels
        self.agent_channels.write().await.clear();
        self.message_queue.write().await.clear();
        
        info!("Message broker shutdown complete");
        Ok(())
    }
}

/// Utility functions for creating common message types
pub mod message_utils {
    use super::*;

    pub fn create_heartbeat(from: Uuid, metrics: serde_json::Value) -> Message {
        Message::new(
            from,
            None,
            MessageType::Heartbeat,
            Priority::Background,
            serde_json::json!({
                "type": "heartbeat",
                "metrics": metrics
            }),
        )
    }

    pub fn create_task_assignment(from: Uuid, to: Uuid, task_id: Uuid, task_data: serde_json::Value) -> Message {
        Message::new(
            from,
            Some(to),
            MessageType::Request,
            Priority::High,
            serde_json::json!({
                "type": "task_assignment",
                "task_id": task_id,
                "task_data": task_data
            }),
        ).with_timeout(300) // 5 minute timeout
    }

    pub fn create_task_completion(from: Uuid, to: Uuid, task_id: Uuid, result: serde_json::Value) -> Message {
        Message::new(
            from,
            Some(to),
            MessageType::Response,
            Priority::High,
            serde_json::json!({
                "type": "task_completion",
                "task_id": task_id,
                "result": result
            }),
        )
    }

    pub fn create_escalation(from: Uuid, to: Uuid, issue: String, context: serde_json::Value) -> Message {
        Message::new(
            from,
            Some(to),
            MessageType::Request,
            Priority::Critical,
            serde_json::json!({
                "type": "escalation",
                "issue": issue,
                "context": context
            }),
        ).with_timeout(60) // 1 minute timeout for escalations
    }

    pub fn create_system_alert(from: Uuid, level: Priority, message: String, details: serde_json::Value) -> Message {
        Message::new(
            from,
            None,
            MessageType::Alert,
            level,
            serde_json::json!({
                "type": "system_alert",
                "message": message,
                "details": details
            }),
        )
    }
}