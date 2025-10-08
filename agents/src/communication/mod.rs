use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Message classification used for inter-agent coordination.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MessageType {
    TaskAssignment,
    TaskUpdate,
    TaskCompletion,
    AgentStatus,
    ConstitutionalValidation,
    KnowledgeQuery,
    CoordinationRequest,
    SystemBroadcast,
    Heartbeat,
}

/// Identifies the logical agent role.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentType {
    NoaCommander,
    BoardAgent,
    ModelSelector,
    MicroAgent,
    KnowledgeGraph,
    TrifectaCourt,
}

/// Runtime status of a registered agent.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentStatus {
    Online,
    Busy,
    Idle,
    Offline,
    Error,
}

/// Message envelope shared between participants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub message_id: Uuid,
    pub message_type: MessageType,
    pub sender_id: String,
    pub sender_type: AgentType,
    pub recipient_id: Option<String>,
    pub recipient_type: Option<AgentType>,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub priority: u8,
    pub requires_response: bool,
    pub correlation_id: Option<Uuid>,
}

impl AgentMessage {
    pub fn new(
        message_type: MessageType,
        sender_id: impl Into<String>,
        sender_type: AgentType,
    ) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            message_type,
            sender_id: sender_id.into(),
            sender_type,
            recipient_id: None,
            recipient_type: None,
            payload: serde_json::Value::Object(Default::default()),
            timestamp: Utc::now(),
            priority: 1,
            requires_response: false,
            correlation_id: None,
        }
    }

    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }

    pub fn to_agent(mut self, agent_id: impl Into<String>, agent_type: AgentType) -> Self {
        self.recipient_id = Some(agent_id.into());
        self.recipient_type = Some(agent_type);
        self
    }
}

/// Metadata tracked for each agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub agent_id: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
    pub current_tasks: Vec<String>,
    pub last_heartbeat: DateTime<Utc>,
    pub tasks_completed: u64,
    pub average_response_time_ms: f64,
    pub success_rate: f64,
    pub load_factor: f64,
}

impl AgentInfo {
    fn new(agent_id: impl Into<String>, agent_type: AgentType, capabilities: Vec<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            agent_type,
            status: AgentStatus::Online,
            capabilities,
            current_tasks: Vec::new(),
            last_heartbeat: Utc::now(),
            tasks_completed: 0,
            average_response_time_ms: 0.0,
            success_rate: 1.0,
            load_factor: 0.0,
        }
    }
}

/// Handle returned to registered agents for publishing updates.
#[derive(Clone)]
pub struct AgentHandle {
    hub: AgentCommunicationHub,
    agent_id: String,
}

impl AgentHandle {
    pub async fn send(&self, message: AgentMessage) -> Result<(), CommunicationError> {
        self.hub.send_message(message).await
    }

    pub async fn heartbeat(&self) -> Result<(), CommunicationError> {
        self.hub.heartbeat(&self.agent_id).await
    }

    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }
}

/// Communication errors surfaced by the hub.
#[derive(Debug, thiserror::Error)]
pub enum CommunicationError {
    #[error("agent {0} not registered")]
    AgentNotRegistered(String),
    #[error("message channel closed")]
    ChannelClosed,
}

/// In-memory communication hub inspired by the CRC Python implementation.
///
/// Redis/WebSocket functionality is modelled via Tokio broadcast channels so the
/// component can be exercised inside the agent crate without additional services.
#[derive(Clone)]
pub struct AgentCommunicationHub {
    inner: Arc<HubState>,
}

struct HubState {
    agents: RwLock<HashMap<String, AgentInfo>>,
    subscriptions: RwLock<HashMap<String, broadcast::Sender<AgentMessage>>>,
    global_tx: broadcast::Sender<AgentMessage>,
    topics: RwLock<HashMap<String, broadcast::Sender<AgentMessage>>>,
}

impl AgentCommunicationHub {
    pub fn new() -> Self {
        let (global_tx, _) = broadcast::channel(128);
        Self {
            inner: Arc::new(HubState {
                agents: RwLock::new(HashMap::new()),
                subscriptions: RwLock::new(HashMap::new()),
                global_tx,
                topics: RwLock::new(HashMap::new()),
            }),
        }
    }

    pub async fn register_agent(
        &self,
        agent_id: impl Into<String>,
        agent_type: AgentType,
        capabilities: Vec<String>,
    ) -> AgentHandle {
        let id = agent_id.into();

        {
            let mut agents = self.inner.agents.write().await;
            agents.insert(id.clone(), AgentInfo::new(&id, agent_type, capabilities));
        }

        // allocate subscription channel for direct messages
        let mut subscriptions = self.inner.subscriptions.write().await;
        subscriptions
            .entry(id.clone())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(64);
                tx
            });

        AgentHandle {
            hub: self.clone(),
            agent_id: id,
        }
    }

    pub async fn unregister_agent(&self, agent_id: &str) {
        self.inner.agents.write().await.remove(agent_id);
        self.inner.subscriptions.write().await.remove(agent_id);
    }

    /// Subscribe to messages destined for a specific agent.
    pub async fn subscribe_agent(
        &self,
        agent_id: &str,
    ) -> Result<broadcast::Receiver<AgentMessage>, CommunicationError> {
        let subscriptions = self.inner.subscriptions.read().await;
        let tx = subscriptions
            .get(agent_id)
            .ok_or_else(|| CommunicationError::AgentNotRegistered(agent_id.to_string()))?;
        Ok(tx.subscribe())
    }

    /// Subscribe to a global broadcast stream.
    pub fn subscribe_global(&self) -> broadcast::Receiver<AgentMessage> {
        self.inner.global_tx.subscribe()
    }

    /// Subscribe to a logical topic (standing in for Redis pub/sub).
    pub async fn subscribe_topic(
        &self,
        topic: &str,
    ) -> broadcast::Receiver<AgentMessage> {
        let mut topics = self.inner.topics.write().await;
        let tx = topics
            .entry(topic.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(64);
                tx
            })
            .clone();
        tx.subscribe()
    }

    pub async fn send_message(&self, message: AgentMessage) -> Result<(), CommunicationError> {
        if let Some(recipient_id) = &message.recipient_id {
            let subscriptions = self.inner.subscriptions.read().await;
            let tx = subscriptions
                .get(recipient_id)
                .ok_or_else(|| CommunicationError::AgentNotRegistered(recipient_id.clone()))?;
            tx.send(message.clone())
                .map_err(|_| CommunicationError::ChannelClosed)?;
        } else {
            self.inner
                .global_tx
                .send(message.clone())
                .map_err(|_| CommunicationError::ChannelClosed)?;
        }

        if let Some(topic) = message
            .recipient_type
            .as_ref()
            .map(|role| format!("topic::{role:?}"))
        {
            let mut topics = self.inner.topics.write().await;
            let tx = topics
                .entry(topic.clone())
                .or_insert_with(|| {
                    let (tx, _) = broadcast::channel(64);
                    tx
                })
                .clone();
            let _ = tx.send(message);
        }

        Ok(())
    }

    pub async fn heartbeat(&self, agent_id: &str) -> Result<(), CommunicationError> {
        let mut agents = self.inner.agents.write().await;
        let info = agents
            .get_mut(agent_id)
            .ok_or_else(|| CommunicationError::AgentNotRegistered(agent_id.to_string()))?;
        info.last_heartbeat = Utc::now();
        Ok(())
    }

    pub async fn update_status(
        &self,
        agent_id: &str,
        status: AgentStatus,
    ) -> Result<(), CommunicationError> {
        let mut agents = self.inner.agents.write().await;
        let info = agents
            .get_mut(agent_id)
            .ok_or_else(|| CommunicationError::AgentNotRegistered(agent_id.to_string()))?;
        info.status = status;
        Ok(())
    }

    pub async fn agent_info(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.inner.agents.read().await;
        agents.get(agent_id).cloned()
    }

    pub async fn list_agents(&self) -> Vec<AgentInfo> {
        let agents = self.inner.agents.read().await;
        agents.values().cloned().collect()
    }
}

impl Default for AgentCommunicationHub {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_and_exchange_messages() {
        let hub = AgentCommunicationHub::new();

        let commander = hub
            .register_agent("commander", AgentType::NoaCommander, vec!["routing".into()])
            .await;
        let worker = hub
            .register_agent("worker-1", AgentType::MicroAgent, vec!["execution".into()])
            .await;

        let mut worker_rx = hub.subscribe_agent("worker-1").await.unwrap();

        let message = AgentMessage::new(MessageType::TaskAssignment, commander.agent_id(), AgentType::NoaCommander)
            .to_agent("worker-1", AgentType::MicroAgent)
            .with_payload(serde_json::json!({"task": "process"}));

        commander.send(message.clone()).await.unwrap();

        let received = worker_rx.recv().await.unwrap();
        assert_eq!(received.message_type, MessageType::TaskAssignment);
        assert_eq!(received.payload["task"], "process");

        worker.heartbeat().await.unwrap();
        let info = hub.agent_info("worker-1").await.unwrap();
        assert_eq!(info.status, AgentStatus::Online);
    }

    #[tokio::test]
    async fn broadcast_reaches_all_listeners() {
        let hub = AgentCommunicationHub::new();
        hub.register_agent("micro-1", AgentType::MicroAgent, vec![]).await;

        let mut global_rx = hub.subscribe_global();
        let broadcast = AgentMessage::new(MessageType::SystemBroadcast, "system", AgentType::KnowledgeGraph)
            .with_payload(serde_json::json!({"info": "update"}));

        hub.send_message(broadcast.clone()).await.unwrap();
        let received = global_rx.recv().await.unwrap();
        assert_eq!(received.payload["info"], "update");
    }
}
