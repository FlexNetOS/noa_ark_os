use noa_agents::registry::AgentRegistry;
use noa_agents::unified_types::AgentLayer;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    pub refill_interval: Duration,
    pub layer_limits: HashMap<AgentLayer, u32>,
    pub persistence: RatePersistence,
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
            persistence: RatePersistence::Sqlite {
                path: default_rate_db_path(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum RatePersistence {
    Memory,
    Sqlite { path: PathBuf },
}

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("agent identity required for rate limited operations")]
    MissingAgentIdentity,
    #[error("rate limit exceeded for agent {0}")]
    LimitExceeded(String),
    #[error("rate store failure: {0}")]
    StoreFailure(String),
}

#[derive(Debug, Clone)]
struct RateState {
    remaining: u32,
    last_refill: SystemTime,
}

#[derive(Debug, Error)]
pub enum RateStoreError {
    #[error("io error: {0}")]
    Io(String),
    #[error("sqlite error: {0}")]
    Sqlite(String),
    #[error("time error: {0}")]
    Time(String),
}

impl From<std::io::Error> for RateStoreError {
    fn from(value: std::io::Error) -> Self {
        RateStoreError::Io(value.to_string())
    }
}

impl From<rusqlite::Error> for RateStoreError {
    fn from(value: rusqlite::Error) -> Self {
        RateStoreError::Sqlite(value.to_string())
    }
}

trait RateStore: Send + Sync {
    fn load_states(&self) -> Result<HashMap<String, RateState>, RateStoreError>;
    fn persist_state(&self, agent_id: &str, state: &RateState) -> Result<(), RateStoreError>;
}

#[derive(Default)]
struct MemoryRateStore;

impl RateStore for MemoryRateStore {
    fn load_states(&self) -> Result<HashMap<String, RateState>, RateStoreError> {
        Ok(HashMap::new())
    }

    fn persist_state(&self, _agent_id: &str, _state: &RateState) -> Result<(), RateStoreError> {
        Ok(())
    }
}

struct SqliteRateStore {
    conn: Mutex<Connection>,
}

impl SqliteRateStore {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self, RateStoreError> {
        if let Some(parent) = path.as_ref().parent() {
            create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS rate_limits (
                agent_id TEXT PRIMARY KEY,
                remaining INTEGER NOT NULL,
                last_refill INTEGER NOT NULL
            );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

impl RateStore for SqliteRateStore {
    fn load_states(&self) -> Result<HashMap<String, RateState>, RateStoreError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT agent_id, remaining, last_refill FROM rate_limits",
        )?;
        let mut rows = stmt.query([])?;
        let mut states = HashMap::new();
        while let Some(row) = rows.next()? {
            let agent_id: String = row.get(0)?;
            let remaining: i64 = row.get(1)?;
            let last_refill_epoch: i64 = row.get(2)?;
            states.insert(
                agent_id,
                RateState {
                    remaining: remaining as u32,
                    last_refill: epoch_to_system_time(last_refill_epoch)?,
                },
            );
        }
        Ok(states)
    }

    fn persist_state(&self, agent_id: &str, state: &RateState) -> Result<(), RateStoreError> {
        let conn = self.conn.lock();
        let last_refill = system_time_to_epoch(state.last_refill)?;
        conn.execute(
            "INSERT INTO rate_limits(agent_id, remaining, last_refill)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(agent_id) DO UPDATE SET remaining=excluded.remaining, last_refill=excluded.last_refill",
            params![agent_id, state.remaining as i64, last_refill],
        )?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RateMetricsSnapshot {
    pub total_checks: u64,
    pub total_denied: u64,
    pub per_layer_remaining: HashMap<String, u32>,
}

#[derive(Default, Clone)]
struct RateMetricsInner {
    total_checks: u64,
    total_denied: u64,
}

/// Token bucket rate limiter informed by hive mind metadata.
pub struct RateLimiter {
    config: RateLimiterConfig,
    registry: Arc<AgentRegistry>,
    states: Mutex<HashMap<String, RateState>>,
    store: Arc<dyn RateStore>,
    metrics: Mutex<RateMetricsInner>,
}

impl RateLimiter {
    pub fn new(
        config: RateLimiterConfig,
        registry: Arc<AgentRegistry>,
    ) -> Result<Self, RateStoreError> {
        let store: Arc<dyn RateStore> = match &config.persistence {
            RatePersistence::Memory => Arc::new(MemoryRateStore::default()),
            RatePersistence::Sqlite { path } => Arc::new(SqliteRateStore::new(path)?),
        };

        let states = store.load_states()?;

        Ok(Self {
            config,
            registry,
            states: Mutex::new(states),
            store,
            metrics: Mutex::new(RateMetricsInner::default()),
        })
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
            last_refill: SystemTime::now(),
        });

        let elapsed = entry
            .last_refill
            .elapsed()
            .unwrap_or_else(|_| Duration::from_secs(0));
        if elapsed >= self.config.refill_interval {
            entry.remaining = limit;
            entry.last_refill = SystemTime::now();
        }

        if entry.remaining == 0 {
            drop(states);
            let mut metrics = self.metrics.lock();
            metrics.total_checks += 1;
            metrics.total_denied += 1;
            return Err(RateLimitError::LimitExceeded(agent_id));
        }

        entry.remaining -= 1;
        let state_clone = entry.clone();
        drop(states);

        {
            let mut metrics = self.metrics.lock();
            metrics.total_checks += 1;
        }

        self.store
            .persist_state(&agent_id, &state_clone)
            .map_err(|err| RateLimitError::StoreFailure(err.to_string()))?
        ;
        Ok(())
    }

    pub fn metrics_snapshot(&self) -> RateMetricsSnapshot {
        let metrics = self.metrics.lock().clone();
        let states = self.states.lock();
        let mut per_layer_remaining: HashMap<String, u32> = HashMap::new();
        for (agent_id, state) in states.iter() {
            let layer = self
                .registry
                .get(agent_id)
                .map(|m| m.layer)
                .unwrap_or(AgentLayer::L5Infrastructure);
            let key = format!("{:?}", layer);
            *per_layer_remaining.entry(key).or_insert(0) += state.remaining;
        }

        RateMetricsSnapshot {
            total_checks: metrics.total_checks,
            total_denied: metrics.total_denied,
            per_layer_remaining,
        }
    }
}

fn system_time_to_epoch(ts: SystemTime) -> Result<i64, RateStoreError> {
    ts.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .map_err(|err| RateStoreError::Time(err.to_string()))
}

fn epoch_to_system_time(value: i64) -> Result<SystemTime, RateStoreError> {
    if value < 0 {
        return Err(RateStoreError::Time("epoch value cannot be negative".into()));
    }
    UNIX_EPOCH
        .checked_add(Duration::from_secs(value as u64))
        .ok_or_else(|| RateStoreError::Time("epoch overflow".into()))
}

fn default_rate_db_path() -> PathBuf {
    if let Ok(path) = env::var("GATEWAY_RATE_DB") {
        return PathBuf::from(path);
    }
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("storage");
    path.push("rate_limits");
    path.push("gateway_rate.db");
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use noa_agents::registry::AgentRegistry;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn registry() -> Arc<AgentRegistry> {
        Arc::new(AgentRegistry::with_default_data().expect("registry"))
    }

    #[test]
    fn respects_rate_limits_in_memory() {
        let limiter = RateLimiter::new(
            RateLimiterConfig {
                layer_limits: HashMap::new(),
                persistence: RatePersistence::Memory,
                ..Default::default()
            },
            registry(),
        )
        .expect("limiter");

        let agent = Some("fixed_agent_gateway".to_string());
        for _ in 0..50 {
            limiter.check(&agent).expect("within bucket");
        }
        let err = limiter.check(&agent).expect_err("limit exceeded");
        assert!(matches!(err, RateLimitError::LimitExceeded(_)));
    }

    #[test]
    fn persists_quota_to_sqlite() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("rate.db");
        let config = RateLimiterConfig {
            persistence: RatePersistence::Sqlite { path: db_path.clone() },
            ..Default::default()
        };
        let limiter = RateLimiter::new(config, registry()).expect("limiter");
        let agent = Some("fixed_agent_gateway".to_string());
        limiter.check(&agent).expect("first pass");

        // Reload from disk and ensure bucket value persisted (remaining < limit).
        let limiter2 = RateLimiter::new(
            RateLimiterConfig {
                persistence: RatePersistence::Sqlite { path: db_path },
                ..Default::default()
            },
            registry(),
        )
        .expect("limiter2");
        let snapshot = limiter2.metrics_snapshot();
        assert_eq!(snapshot.total_checks, 0); // metrics reset but state restored
        let states = limiter2.states.lock();
        let state = states.get("fixed_agent_gateway").expect("state persists");
        assert!(state.remaining < 50);
    }
}
