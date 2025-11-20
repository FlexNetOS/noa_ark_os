use noa_agents::registry::AgentRegistry;
use noa_agents::unified_types::AgentLayer;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    pub refill_interval: Duration,
    pub layer_limits: HashMap<AgentLayer, u32>,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        let mut layer_limits = HashMap::new();
        layer_limits.insert(AgentLayer::L1Autonomy, 200);
        layer_limits.insert(AgentLayer::L2Reasoning, 150);
        layer_limits.insert(AgentLayer::L3Orchestration, 120);
        layer_limits.insert(AgentLayer::L4Operations, 80);
        layer_limits.insert(AgentLayer::L5Infrastructure, 40);

        Self {
            refill_interval: Duration::from_secs(60),
            layer_limits,
        }
    }
}

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("agent identity required for rate limited operations")]
    MissingAgentIdentity,
    #[error("rate limit exceeded for agent {0}")]
    LimitExceeded(String),
}

#[derive(Debug)]
struct RateState {
    remaining: u32,
    last_refill: Instant,
}

/// Token bucket rate limiter informed by hive mind metadata.
pub struct RateLimiter {
    config: RateLimiterConfig,
    registry: Arc<AgentRegistry>,
    states: Mutex<HashMap<String, RateState>>,
}

impl RateLimiter {
    pub fn new(config: RateLimiterConfig, registry: Arc<AgentRegistry>) -> Self {
        Self {
            config,
            registry,
            states: Mutex::new(HashMap::new()),
        }
    }

    pub fn check(&self, agent_id: &Option<String>) -> Result<(), RateLimitError> {
        let agent_id = agent_id
            .as_ref()
            .ok_or(RateLimitError::MissingAgentIdentity)?
            .clone();

        let layer = self
            .registry
            .get(&agent_id)
            .map(|m| m.layer)
            .unwrap_or(AgentLayer::L5Infrastructure);

        let limit = self.config.layer_limits.get(&layer).copied().unwrap_or(50);

        let mut states = self.states.lock();
        let entry = states.entry(agent_id.clone()).or_insert_with(|| RateState {
            remaining: limit,
            last_refill: Instant::now(),
        });

        let elapsed = entry.last_refill.elapsed();
        if elapsed >= self.config.refill_interval {
            entry.remaining = limit;
            entry.last_refill = Instant::now();
        }

        if entry.remaining == 0 {
            return Err(RateLimitError::LimitExceeded(agent_id));
        }

        entry.remaining -= 1;
        Ok(())
    }
}
