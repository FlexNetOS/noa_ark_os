use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use log::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub status: AgentStatus,
    pub endpoint: String,
    pub metadata: HashMap<String, String>,
    pub registered_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Inactive,
    Maintenance,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterAgentRequest {
    pub name: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConstitutionalValidationRequest {
    pub action: String,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConstitutionalValidationResponse {
    pub valid: bool,
    pub reason: Option<String>,
    pub scripture: Option<serde_json::Value>,
    pub geometry: Option<serde_json::Value>,
    pub law: Option<serde_json::Value>,
}

type AgentRegistry = Arc<RwLock<HashMap<Uuid, Agent>>>;

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting Agent Registry Service with Constitutional Governance");

    let registry: AgentRegistry = Arc::new(RwLock::new(HashMap::new()));

    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({"status": "healthy"})));

    let register = warp::path("agents")
        .and(warp::path("register"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_registry(registry.clone()))
        .and_then(register_agent);

    let list_agents = warp::path("agents")
        .and(warp::get())
        .and(with_registry(registry.clone()))
        .and_then(list_agents_handler);

    let get_agent = warp::path("agents")
        .and(warp::path::param::<String>())
        .and(warp::get())
        .and(with_registry(registry.clone()))
        .and_then(get_agent_handler);

    let heartbeat = warp::path("agents")
        .and(warp::path::param::<String>())
        .and(warp::path("heartbeat"))
        .and(warp::post())
        .and(with_registry(registry.clone()))
        .and_then(heartbeat_handler);

    let routes = health
        .or(register)
        .or(list_agents)
        .or(get_agent)
        .or(heartbeat)
        .with(warp::cors().allow_any_origin().allow_headers(vec!["content-type"]).allow_methods(vec!["GET", "POST", "PUT", "DELETE"]));

    info!("Agent Registry Service listening on 0.0.0.0:3003");
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3003))
        .await;
}

fn with_registry(registry: AgentRegistry) -> impl Filter<Extract = (AgentRegistry,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || registry.clone())
}

async fn validate_with_trifecta_court(action: &str, context: HashMap<String, serde_json::Value>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let request = ConstitutionalValidationRequest {
        action: action.to_string(),
        context,
    };

    let response = client
        .post("http://trifecta-court:5000/court/trifecta")
        .json(&request)
        .send()
        .await?;

    let validation: ConstitutionalValidationResponse = response.json().await?;
    Ok(validation.valid)
}

async fn register_agent(
    req: RegisterAgentRequest,
    registry: AgentRegistry,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Registering new agent: {}", req.name);

    // Constitutional validation for agent registration
    let mut context = HashMap::new();
    context.insert("agent_name".to_string(), serde_json::Value::String(req.name.clone()));
    context.insert("agent_type".to_string(), serde_json::Value::String(req.agent_type.clone()));
    context.insert("capabilities".to_string(), serde_json::Value::Array(
        req.capabilities.iter().map(|c| serde_json::Value::String(c.clone())).collect()
    ));

    match validate_with_trifecta_court("register_agent", context).await {
        Ok(true) => {
            info!("Constitutional validation passed for agent registration: {}", req.name);
        }
        Ok(false) => {
            warn!("Constitutional validation failed for agent registration: {}", req.name);
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": "Constitutional validation failed"})),
                warp::http::StatusCode::FORBIDDEN,
            ));
        }
        Err(e) => {
            error!("Error during constitutional validation: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": "Validation service unavailable"})),
                warp::http::StatusCode::SERVICE_UNAVAILABLE,
            ));
        }
    }

    let agent_id = Uuid::new_v4();
    let now = Utc::now();
    
    let agent = Agent {
        id: agent_id,
        name: req.name,
        agent_type: req.agent_type,
        capabilities: req.capabilities,
        status: AgentStatus::Active,
        endpoint: req.endpoint,
        metadata: req.metadata.unwrap_or_default(),
        registered_at: now,
        last_heartbeat: now,
    };

    let mut registry_guard = registry.write().await;
    registry_guard.insert(agent_id, agent.clone());
    drop(registry_guard);

    info!("Agent registered successfully: {} ({})", agent.name, agent_id);
    Ok(warp::reply::json(&agent))
}

async fn list_agents_handler(registry: AgentRegistry) -> Result<impl warp::Reply, warp::Rejection> {
    let registry_guard = registry.read().await;
    let agents: Vec<&Agent> = registry_guard.values().collect();
    Ok(warp::reply::json(&agents))
}

async fn get_agent_handler(
    agent_id: String,
    registry: AgentRegistry,
) -> Result<impl warp::Reply, warp::Rejection> {
    let agent_uuid = match Uuid::parse_str(&agent_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": "Invalid agent ID"})),
                warp::http::StatusCode::BAD_REQUEST,
            ));
        }
    };

    let registry_guard = registry.read().await;
    match registry_guard.get(&agent_uuid) {
        Some(agent) => Ok(warp::reply::with_status(
            warp::reply::json(agent),
            warp::http::StatusCode::OK,
        )),
        None => Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"error": "Agent not found"})),
            warp::http::StatusCode::NOT_FOUND,
        )),
    }
}

async fn heartbeat_handler(
    agent_id: String,
    registry: AgentRegistry,
) -> Result<impl warp::Reply, warp::Rejection> {
    let agent_uuid = match Uuid::parse_str(&agent_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": "Invalid agent ID"})),
                warp::http::StatusCode::BAD_REQUEST,
            ));
        }
    };

    let mut registry_guard = registry.write().await;
    match registry_guard.get_mut(&agent_uuid) {
        Some(agent) => {
            agent.last_heartbeat = Utc::now();
            agent.status = AgentStatus::Active;
            info!("Heartbeat received from agent: {} ({})", agent.name, agent_id);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"status": "heartbeat_received"})),
                warp::http::StatusCode::OK,
            ))
        }
        None => Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"error": "Agent not found"})),
            warp::http::StatusCode::NOT_FOUND,
        )),
    }
}

