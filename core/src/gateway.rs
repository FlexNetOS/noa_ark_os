use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, SystemTime};

/// Represents a type of symbol supported by the gateway.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Api,
    Hook,
    Plugin,
    Extension,
    Stub,
    FeatureFlag,
    Tag,
    Channel,
    Dataset,
    Service,
    Custom(String),
}

/// Normalized metadata describing a symbol connector.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: String,
    pub kind: SymbolKind,
    pub version: String,
    pub capabilities: HashSet<String>,
    pub schema_hash: String,
}

impl Symbol {
    pub fn matches_capabilities(&self, required: &HashSet<String>) -> bool {
        required.is_subset(&self.capabilities)
    }
}

/// Policy guardrails applied to a connector.
#[derive(Debug, Clone)]
pub struct ConnectionPolicy {
    pub max_latency_ms: u32,
    pub min_trust_score: f32,
    pub allowed_zones: HashSet<String>,
    pub encryption_required: bool,
}

impl ConnectionPolicy {
    pub fn allows(&self, constraints: &IntentConstraints) -> bool {
        if self.max_latency_ms > constraints.max_latency_ms {
            return false;
        }
        if self.min_trust_score < constraints.min_trust_score {
            return false;
        }
        if self.encryption_required && !constraints.encryption_supported {
            return false;
        }
        if !constraints.allowed_zones.is_subset(&self.allowed_zones) {
            return false;
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct IntentConstraints {
    pub max_latency_ms: u32,
    pub min_trust_score: f32,
    pub encryption_supported: bool,
    pub allowed_zones: HashSet<String>,
}

/// High level business goal compiled into routing requirements.
#[derive(Debug, Clone)]
pub struct Intent {
    pub description: String,
    pub target_kind: SymbolKind,
    pub required_capabilities: HashSet<String>,
    pub constraints: IntentConstraints,
}

/// Observed state of a connector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Connected,
    Disconnected,
    Pending,
    Faulted,
}

#[derive(Debug, Clone)]
struct ConnectorRecord {
    symbol: Arc<Symbol>,
    policy: ConnectionPolicy,
    state: ConnectionState,
    last_seen: SystemTime,
    health_score: f32,
}

impl ConnectorRecord {
    fn new(symbol: Arc<Symbol>, policy: ConnectionPolicy) -> Self {
        Self {
            symbol,
            policy,
            state: ConnectionState::Disconnected,
            last_seen: SystemTime::now(),
            health_score: 0.7,
        }
    }

    fn refresh(&mut self, now: SystemTime) -> ScanEvent {
        let since_last = now.duration_since(self.last_seen).unwrap_or(Duration::ZERO);

        if since_last > Duration::from_secs(5) {
            self.health_score *= 0.95;
            self.state = if self.health_score < 0.4 {
                ConnectionState::Faulted
            } else {
                ConnectionState::Pending
            };
        } else {
            self.health_score = (self.health_score + 1.0).min(1.0);
            self.state = ConnectionState::Connected;
        }

        self.last_seen = now;

        ScanEvent {
            connector_id: self.symbol.id.clone(),
            state: self.state,
            health_score: self.health_score,
        }
    }
}

/// Result of a scan pass.
#[derive(Debug, Clone)]
pub struct ScanEvent {
    pub connector_id: String,
    pub state: ConnectionState,
    pub health_score: f32,
}

/// Plan produced after intent compilation and verification.
#[derive(Debug, Clone)]
pub struct RoutePlan {
    pub connectors: Vec<String>,
    pub predicted_latency_ms: u32,
    pub verified: bool,
}

#[derive(Debug, Clone)]
pub struct SelfHealAction {
    pub connector_id: String,
    pub action: String,
}

#[derive(Debug, Clone)]
pub struct GatewaySnapshot {
    pub connected: usize,
    pub pending: usize,
    pub faulted: usize,
    pub average_health: f32,
}

type ConnectorId = String;

#[derive(Debug)]
pub enum GatewayError {
    AlreadyRegistered(String),
    NotFound(String),
    PolicyViolation(String),
    NoRouteFound,
    VerificationFailed(String),
    Poisoned(&'static str),
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GatewayError::AlreadyRegistered(id) => {
                write!(f, "connector {id} is already registered")
            }
            GatewayError::NotFound(id) => write!(f, "connector {id} not found"),
            GatewayError::PolicyViolation(msg) => write!(f, "policy violation: {msg}"),
            GatewayError::NoRouteFound => write!(f, "no viable route found for intent"),
            GatewayError::VerificationFailed(msg) => write!(f, "verification failed: {msg}"),
            GatewayError::Poisoned(resource) => write!(f, "gateway state poisoned: {resource}"),
        }
    }
}

impl std::error::Error for GatewayError {}

static GLOBAL_GATEWAY: OnceLock<Gateway> = OnceLock::new();

/// Primary entry point for the self-aware gateway.
#[derive(Debug, Default)]
pub struct Gateway {
    connectors: RwLock<HashMap<ConnectorId, ConnectorRecord>>,
    topology: RwLock<HashMap<SymbolKind, HashSet<ConnectorId>>>,
}

impl Gateway {
    pub fn new() -> Self {
        Self::default()
    }

    /// Access the globally initialized gateway.
    pub fn global() -> &'static Self {
        GLOBAL_GATEWAY.get_or_init(Self::default)
    }

    /// Register a new connector with its policy envelope.
    pub fn register_symbol(
        &self,
        symbol: Symbol,
        policy: ConnectionPolicy,
    ) -> Result<(), GatewayError> {
        let mut connectors = self.connectors_write()?;
        if connectors.contains_key(&symbol.id) {
            return Err(GatewayError::AlreadyRegistered(symbol.id.clone()));
        }

        let id = symbol.id.clone();
        let kind = symbol.kind.clone();
        let symbol = Arc::new(symbol);

        connectors.insert(id.clone(), ConnectorRecord::new(symbol, policy));

        self.topology_write()?.entry(kind).or_default().insert(id);

        Ok(())
    }

    /// Establish an explicit connection if the policy allows it.
    pub fn connect(&self, connector_id: &str) -> Result<(), GatewayError> {
        let mut connectors = self.connectors_write()?;
        let record = connectors
            .get_mut(connector_id)
            .ok_or_else(|| GatewayError::NotFound(connector_id.to_string()))?;
        record.state = ConnectionState::Connected;
        record.last_seen = SystemTime::now();
        record.health_score = (record.health_score + 0.2).min(1.0);
        Ok(())
    }

    /// Disconnect a connector from the routing fabric.
    pub fn disconnect(&self, connector_id: &str) -> Result<(), GatewayError> {
        let mut connectors = self.connectors_write()?;
        let record = connectors
            .get_mut(connector_id)
            .ok_or_else(|| GatewayError::NotFound(connector_id.to_string()))?;
        record.state = ConnectionState::Disconnected;
        Ok(())
    }

    /// Perform an auto-scan, updating health and discovering risky connectors.
    pub fn auto_scan(&self) -> Result<Vec<ScanEvent>, GatewayError> {
        let mut connectors = self.connectors_write()?;
        let now = SystemTime::now();
        Ok(connectors
            .values_mut()
            .map(|record| record.refresh(now))
            .collect())
    }

    /// Calculate an optimized route for a given intent.
    pub fn route_intent(&self, intent: &Intent) -> Result<RoutePlan, GatewayError> {
        let connectors = self.connectors_read()?;
        let mut candidates: Vec<_> = connectors
            .values()
            .filter(|record| {
                record.symbol.kind == intent.target_kind
                    && record
                        .symbol
                        .matches_capabilities(&intent.required_capabilities)
                    && record.policy.allows(&intent.constraints)
                    && record.health_score >= intent.constraints.min_trust_score
                    && record.state != ConnectionState::Faulted
            })
            .collect();

        if candidates.is_empty() {
            return Err(GatewayError::NoRouteFound);
        }

        candidates.sort_by(|a, b| {
            b.health_score
                .partial_cmp(&a.health_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let predicted_latency_ms = candidates
            .iter()
            .map(|record| record.policy.max_latency_ms)
            .min()
            .unwrap_or(intent.constraints.max_latency_ms);

        let mut plan = RoutePlan {
            connectors: candidates
                .iter()
                .take(3)
                .map(|record| record.symbol.id.clone())
                .collect(),
            predicted_latency_ms,
            verified: false,
        };

        if self.formal_verification(intent, &plan)? {
            plan.verified = true;
            Ok(plan)
        } else {
            Err(GatewayError::VerificationFailed(
                "intent constraints not satisfied in twin".into(),
            ))
        }
    }

    /// Digital twin style verification of a plan.
    fn formal_verification(&self, intent: &Intent, plan: &RoutePlan) -> Result<bool, GatewayError> {
        if plan.connectors.is_empty() {
            return Ok(false);
        }

        let topology = self.topology_read()?;
        let allowed = topology.get(&intent.target_kind);

        for connector_id in &plan.connectors {
            if allowed.is_none_or(|set| !set.contains(connector_id)) {
                return Ok(false);
            }
        }

        Ok(plan.predicted_latency_ms <= intent.constraints.max_latency_ms)
    }

    /// Run predictive self healing, returning any actions to be executed.
    pub fn predictive_self_heal(&self) -> Result<Vec<SelfHealAction>, GatewayError> {
        let mut actions = Vec::new();
        let mut connectors = self.connectors_write()?;

        for record in connectors.values_mut() {
            if record.state == ConnectionState::Faulted || record.health_score < 0.45 {
                record.state = ConnectionState::Pending;
                record.health_score = (record.health_score + 0.1).min(0.8);
                actions.push(SelfHealAction {
                    connector_id: record.symbol.id.clone(),
                    action: "routed to redundant quick-connect".into(),
                });
            }
        }

        Ok(actions)
    }

    /// Produce an observability snapshot of the gateway.
    pub fn snapshot(&self) -> Result<GatewaySnapshot, GatewayError> {
        let connectors = self.connectors_read()?;
        let mut connected = 0;
        let mut pending = 0;
        let mut faulted = 0;
        let mut total_health = 0.0;

        for record in connectors.values() {
            total_health += record.health_score;
            match record.state {
                ConnectionState::Connected => connected += 1,
                ConnectionState::Pending => pending += 1,
                ConnectionState::Faulted => faulted += 1,
                ConnectionState::Disconnected => {}
            }
        }

        let average_health = if connectors.is_empty() {
            1.0
        } else {
            total_health / connectors.len() as f32
        };

        Ok(GatewaySnapshot {
            connected,
            pending,
            faulted,
            average_health,
        })
    }

    fn connectors_read(
        &self,
    ) -> Result<RwLockReadGuard<'_, HashMap<ConnectorId, ConnectorRecord>>, GatewayError> {
        self.connectors
            .read()
            .map_err(|_| GatewayError::Poisoned("connectors"))
    }

    fn connectors_write(
        &self,
    ) -> Result<RwLockWriteGuard<'_, HashMap<ConnectorId, ConnectorRecord>>, GatewayError> {
        self.connectors
            .write()
            .map_err(|_| GatewayError::Poisoned("connectors"))
    }

    fn topology_read(
        &self,
    ) -> Result<RwLockReadGuard<'_, HashMap<SymbolKind, HashSet<ConnectorId>>>, GatewayError> {
        self.topology
            .read()
            .map_err(|_| GatewayError::Poisoned("topology"))
    }

    fn topology_write(
        &self,
    ) -> Result<RwLockWriteGuard<'_, HashMap<SymbolKind, HashSet<ConnectorId>>>, GatewayError> {
        self.topology
            .write()
            .map_err(|_| GatewayError::Poisoned("topology"))
    }
}

/// Initialize the global gateway and provide a stable foundation.
pub fn init() -> Result<(), GatewayError> {
    let gateway = Gateway::global();
    gateway.auto_scan()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_policy() -> ConnectionPolicy {
        ConnectionPolicy {
            max_latency_ms: 10,
            min_trust_score: 0.9,
            allowed_zones: HashSet::from(["global".into(), "edge".into()]),
            encryption_required: true,
        }
    }

    fn sample_constraints() -> IntentConstraints {
        IntentConstraints {
            max_latency_ms: 15,
            min_trust_score: 0.6,
            encryption_supported: true,
            allowed_zones: HashSet::from(["global".into()]),
        }
    }

    #[test]
    fn register_and_route_symbol() {
        let gateway = Gateway::new();
        let symbol = Symbol {
            id: "analytics.api".into(),
            kind: SymbolKind::Api,
            version: "1.0.0".into(),
            capabilities: HashSet::from(["stream".into(), "analytics".into()]),
            schema_hash: "abc123".into(),
        };

        gateway
            .register_symbol(symbol.clone(), sample_policy())
            .expect("registration should succeed");
        gateway.connect(&symbol.id).expect("connect should succeed");

        let intent = Intent {
            description: "Replicate analytics stream".into(),
            target_kind: SymbolKind::Api,
            required_capabilities: HashSet::from(["stream".into()]),
            constraints: sample_constraints(),
        };

        let plan = gateway.route_intent(&intent).expect("route should succeed");
        assert!(plan.verified);
        assert_eq!(plan.connectors.len(), 1);
    }

    #[test]
    fn predictive_self_healing_flags_faults() {
        let gateway = Gateway::new();
        let symbol = Symbol {
            id: "legacy.plugin".into(),
            kind: SymbolKind::Plugin,
            version: "2.1.0".into(),
            capabilities: HashSet::from(["render".into()]),
            schema_hash: "deadbeef".into(),
        };

        gateway
            .register_symbol(symbol.clone(), sample_policy())
            .expect("registration should succeed");

        // Force the connector into a degraded state.
        {
            let mut connectors = gateway.connectors.write().unwrap();
            let record = connectors.get_mut(&symbol.id).unwrap();
            record.state = ConnectionState::Faulted;
            record.health_score = 0.3;
        }

        let actions = gateway
            .predictive_self_heal()
            .expect("self healing should succeed");
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].connector_id, symbol.id);
    }

    #[test]
    fn policy_violation_prevents_routing() {
        let gateway = Gateway::new();
        let symbol = Symbol {
            id: "restricted.api".into(),
            kind: SymbolKind::Api,
            version: "1.2.0".into(),
            capabilities: HashSet::from(["restricted".into()]),
            schema_hash: "feedface".into(),
        };

        let mut policy = sample_policy();
        policy.allowed_zones = HashSet::from(["private".into()]);

        gateway
            .register_symbol(symbol.clone(), policy)
            .expect("registration should succeed");
        gateway.connect(&symbol.id).expect("connect should succeed");

        let constraints = IntentConstraints {
            max_latency_ms: 10,
            min_trust_score: 0.5,
            encryption_supported: true,
            allowed_zones: HashSet::from(["global".into()]),
        };

        let intent = Intent {
            description: "Access restricted api".into(),
            target_kind: SymbolKind::Api,
            required_capabilities: HashSet::from(["restricted".into()]),
            constraints,
        };

        let result = gateway.route_intent(&intent);
        assert!(matches!(result, Err(GatewayError::NoRouteFound)));
    }

    #[test]
    fn policies_respect_trust_thresholds() {
        let mut policy = sample_policy();
        policy.min_trust_score = 0.9;

        let constraints = IntentConstraints {
            max_latency_ms: 15,
            min_trust_score: 0.6,
            encryption_supported: true,
            allowed_zones: HashSet::from(["global".into()]),
        };

        assert!(policy.allows(&constraints));

        let stricter = IntentConstraints {
            min_trust_score: 0.95,
            ..constraints
        };
        assert!(!policy.allows(&stricter));
    }
}
