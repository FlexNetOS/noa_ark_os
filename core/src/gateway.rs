use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
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

impl SymbolKind {
    fn as_str(&self) -> &str {
        match self {
            SymbolKind::Api => "api",
            SymbolKind::Hook => "hook",
            SymbolKind::Plugin => "plugin",
            SymbolKind::Extension => "extension",
            SymbolKind::Stub => "stub",
            SymbolKind::FeatureFlag => "feature_flag",
            SymbolKind::Tag => "tag",
            SymbolKind::Channel => "channel",
            SymbolKind::Dataset => "dataset",
            SymbolKind::Service => "service",
            SymbolKind::Custom(value) => value.as_str(),
        }
    }
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for SymbolKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SymbolKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SymbolKindVisitor;

        impl<'de> Visitor<'de> for SymbolKindVisitor {
            type Value = SymbolKind;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a symbol kind string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                SymbolKind::from_str(value).map_err(|err| E::custom(err))
            }
        }

        deserializer.deserialize_str(SymbolKindVisitor)
    }
}

impl FromStr for SymbolKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "api" => Ok(SymbolKind::Api),
            "hook" => Ok(SymbolKind::Hook),
            "plugin" => Ok(SymbolKind::Plugin),
            "extension" => Ok(SymbolKind::Extension),
            "stub" => Ok(SymbolKind::Stub),
            "feature_flag" => Ok(SymbolKind::FeatureFlag),
            "tag" => Ok(SymbolKind::Tag),
            "channel" => Ok(SymbolKind::Channel),
            "dataset" => Ok(SymbolKind::Dataset),
            "service" => Ok(SymbolKind::Service),
            other if !other.trim().is_empty() => Ok(SymbolKind::Custom(other.to_string())),
            _ => Err("symbol kind cannot be empty".to_string()),
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStage {
    Prototype,
    Active,
    Deprecated,
    Retired,
}

impl Default for LifecycleStage {
    fn default() -> Self {
        LifecycleStage::Prototype
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityWindow {
    pub related_kind: SymbolKind,
    pub minimum_version: String,
    #[serde(default)]
    pub maximum_version: Option<String>,
}

impl CompatibilityWindow {
    pub fn accepts(&self, kind: &SymbolKind, version: &str) -> bool {
        if &self.related_kind != kind {
            return false;
        }
        if version < self.minimum_version.as_str() {
            return false;
        }
        if let Some(max) = &self.maximum_version {
            if version > max.as_str() {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolSchema {
    pub schema_id: String,
    pub kind: SymbolKind,
    pub version: String,
    #[serde(default)]
    pub capability_taxonomy: HashSet<String>,
    #[serde(default)]
    pub lifecycle: LifecycleStage,
    #[serde(default)]
    pub recommended_zones: HashSet<String>,
    #[serde(default)]
    pub compliance_tags: HashSet<String>,
    #[serde(default)]
    pub compatibility: Vec<CompatibilityWindow>,
    pub schema_hash: String,
}

impl SymbolSchema {
    pub fn matches_symbol(&self, symbol: &Symbol) -> bool {
        self.kind == symbol.kind
            && self.schema_hash == symbol.schema_hash
            && self.version == symbol.version
    }

    pub fn capability_coverage(&self, required: &HashSet<String>) -> f32 {
        if required.is_empty() {
            return 1.0;
        }
        let covered = required
            .iter()
            .filter(|cap| self.capability_taxonomy.contains(*cap))
            .count();
        covered as f32 / required.len() as f32
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

#[derive(Debug, Clone)]
pub struct TelemetryEvent {
    pub timestamp: SystemTime,
    pub kind: TelemetryKind,
    pub context: String,
}

#[derive(Debug, Clone)]
pub enum TelemetryKind {
    SchemaRegistered,
    ConnectorRegistered,
    ScanCompleted,
    RouteCompiled,
    SelfHealSuggested,
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

#[derive(Debug, Clone)]
pub struct SchemaCatalogSnapshot {
    pub total: usize,
    pub lifecycle_breakdown: HashMap<LifecycleStage, usize>,
    pub capability_index: HashMap<String, usize>,
}

#[derive(Debug, Deserialize)]
struct IntentDocument {
    intents: Vec<IntentSpec>,
}

#[derive(Debug, Deserialize)]
struct IntentSpec {
    name: String,
    target: String,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    constraints: IntentConstraintSpec,
}

#[derive(Debug, Deserialize)]
struct IntentConstraintSpec {
    #[serde(default = "default_latency")]
    max_latency_ms: u32,
    #[serde(default = "default_trust")]
    min_trust_score: f32,
    #[serde(default = "default_true")]
    encryption: bool,
    #[serde(default = "default_zones")]
    zones: Vec<String>,
}

impl Default for IntentConstraintSpec {
    fn default() -> Self {
        Self {
            max_latency_ms: default_latency(),
            min_trust_score: default_trust(),
            encryption: default_true(),
            zones: default_zones(),
        }
    }
}

fn default_latency() -> u32 {
    250
}

fn default_trust() -> f32 {
    0.6
}

fn default_true() -> bool {
    true
}

fn default_zones() -> Vec<String> {
    vec!["global".into()]
}

impl IntentConstraintSpec {
    fn into_constraints(self) -> IntentConstraints {
        IntentConstraints {
            max_latency_ms: self.max_latency_ms,
            min_trust_score: self.min_trust_score,
            encryption_supported: self.encryption,
            allowed_zones: self.zones.into_iter().collect::<HashSet<_>>(),
        }
    }
}

impl IntentSpec {
    fn into_intent(self) -> Result<Intent, GatewayError> {
        let target_kind = SymbolKind::from_str(&self.target).map_err(|err| {
            GatewayError::IntentParse(format!(
                "intent '{}' has invalid target '{}': {err}",
                self.name, self.target
            ))
        })?;

        let required_capabilities = self
            .capabilities
            .into_iter()
            .collect::<HashSet<_>>();

        Ok(Intent {
            description: self.name,
            target_kind,
            required_capabilities,
            constraints: self.constraints.into_constraints(),
        })
    }
}

pub struct IntentCompiler;

impl IntentCompiler {
    pub fn compile(script: &str) -> Result<Vec<Intent>, GatewayError> {
        let document: IntentDocument = serde_yaml::from_str(script)
            .map_err(|err| GatewayError::IntentParse(err.to_string()))?;
        document
            .intents
            .into_iter()
            .map(IntentSpec::into_intent)
            .collect()
    }
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
    SchemaConflict(String),
    SchemaNotFound(String),
    IntentParse(String),
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
            GatewayError::SchemaConflict(id) => write!(f, "schema {id} already registered"),
            GatewayError::SchemaNotFound(id) => write!(f, "schema {id} not found"),
            GatewayError::IntentParse(msg) => write!(f, "intent parsing error: {msg}"),
        }
    }
}

impl std::error::Error for GatewayError {}

static GLOBAL_GATEWAY: OnceLock<Gateway> = OnceLock::new();

/// Primary entry point for the self-aware gateway.
#[derive(Debug)]
pub struct Gateway {
    connectors: RwLock<HashMap<ConnectorId, ConnectorRecord>>,
    topology: RwLock<HashMap<SymbolKind, HashSet<ConnectorId>>>,
    schemas: RwLock<HashMap<String, SymbolSchema>>,
    telemetry: RwLock<Vec<TelemetryEvent>>,
}

impl Gateway {
    pub fn new() -> Self {
        Self::default()
    }

    /// Access the globally initialized gateway.
    pub fn global() -> &'static Self {
        GLOBAL_GATEWAY.get_or_init(Self::default)
    }

    /// Register a schema into the catalog.
    pub fn register_schema(&self, schema: SymbolSchema) -> Result<(), GatewayError> {
        let mut registry = self.schemas_write()?;
        if let Some(existing) = registry.get(&schema.schema_id) {
            if existing.schema_hash != schema.schema_hash {
                return Err(GatewayError::SchemaConflict(schema.schema_id));
            }
            return Ok(());
        }

        if registry
            .values()
            .any(|existing| existing.schema_hash == schema.schema_hash)
        {
            return Err(GatewayError::SchemaConflict(schema.schema_hash));
        }

        let descriptor = schema.clone();
        registry.insert(descriptor.schema_id.clone(), descriptor.clone());
        drop(registry);
        self.emit_event(
            TelemetryKind::SchemaRegistered,
            format!("{}@{}", descriptor.schema_id, descriptor.version),
        )?;
        Ok(())
    }

    /// Bulk ingest schemas, ensuring catalog consistency.
    pub fn ingest_schema_catalog(&self, schemas: &[SymbolSchema]) -> Result<(), GatewayError> {
        let mut registry = self.schemas_write()?;
        let mut new_entries = Vec::new();

        for schema in schemas {
            if let Some(existing) = registry.get(&schema.schema_id) {
                if existing.schema_hash != schema.schema_hash {
                    return Err(GatewayError::SchemaConflict(schema.schema_id.clone()));
                }
                continue;
            }

            if registry
                .values()
                .any(|existing| existing.schema_hash == schema.schema_hash)
            {
                return Err(GatewayError::SchemaConflict(schema.schema_hash.clone()));
            }

            registry.insert(schema.schema_id.clone(), schema.clone());
            new_entries.push(schema.clone());
        }

        drop(registry);

        for schema in new_entries {
            self.emit_event(
                TelemetryKind::SchemaRegistered,
                format!("{}@{}", schema.schema_id, schema.version),
            )?;
        }

        Ok(())
    }

    /// Provide a snapshot of the catalog for reporting.
    pub fn catalog_snapshot(&self) -> Result<SchemaCatalogSnapshot, GatewayError> {
        let registry = self.schemas_read()?;
        let mut lifecycle_breakdown: HashMap<LifecycleStage, usize> = HashMap::new();
        let mut capability_index: HashMap<String, usize> = HashMap::new();

        for schema in registry.values() {
            *lifecycle_breakdown.entry(schema.lifecycle).or_insert(0) += 1;
            for capability in &schema.capability_taxonomy {
                *capability_index.entry(capability.clone()).or_insert(0) += 1;
            }
        }

        Ok(SchemaCatalogSnapshot {
            total: registry.len(),
            lifecycle_breakdown,
            capability_index,
        })
    }

    /// Drain telemetry into the monitoring pipeline.
    pub fn drain_telemetry(&self) -> Result<Vec<TelemetryEvent>, GatewayError> {
        let mut events = self.telemetry_write()?;
        Ok(events.drain(..).collect())
    }

    /// Bootstraps the gateway with built-in schemas for critical services.
    pub fn bootstrap_defaults(&self) -> Result<(), GatewayError> {
        self.ingest_schema_catalog(&default_schemas())
    }

    /// Register a new connector with its policy envelope.
    pub fn register_symbol(
        &self,
        symbol: Symbol,
        policy: ConnectionPolicy,
    ) -> Result<(), GatewayError> {
        let schema = {
            let registry = self.schemas_read()?;
            registry
                .values()
                .find(|schema| schema.matches_symbol(&symbol))
                .cloned()
        };

        let schema =
            schema.ok_or_else(|| GatewayError::SchemaNotFound(symbol.schema_hash.clone()))?;

        if !schema.capability_taxonomy.is_superset(&symbol.capabilities) {
            return Err(GatewayError::PolicyViolation(format!(
                "symbol {} declares capabilities not advertised by schema {}",
                symbol.id, schema.schema_id
            )));
        }

        let mut connectors = self.connectors_write()?;
        if connectors.contains_key(&symbol.id) {
            return Err(GatewayError::AlreadyRegistered(symbol.id.clone()));
        }

        let id = symbol.id.clone();
        let kind = symbol.kind.clone();
        let symbol = Arc::new(symbol);

        connectors.insert(id.clone(), ConnectorRecord::new(symbol.clone(), policy));

        self.topology_write()?.entry(kind).or_default().insert(id);

        self.emit_event(
            TelemetryKind::ConnectorRegistered,
            format!("{}@{}", symbol.id, symbol.version),
        )?;

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
        let events: Vec<_> = connectors
            .values_mut()
            .map(|record| record.refresh(now))
            .collect();
        drop(connectors);
        self.emit_event(
            TelemetryKind::ScanCompleted,
            format!("scan:{}", events.len()),
        )?;
        Ok(events)
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
            self.emit_event(
                TelemetryKind::RouteCompiled,
                format!(
                    "intent:{} connectors:{}",
                    intent.description,
                    plan.connectors.len()
                ),
            )?;
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

        if !actions.is_empty() {
            self.emit_event(
                TelemetryKind::SelfHealSuggested,
                format!("actions:{}", actions.len()),
            )?;
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

    fn emit_event(&self, kind: TelemetryKind, context: String) -> Result<(), GatewayError> {
        let mut telemetry = self.telemetry_write()?;
        telemetry.push(TelemetryEvent {
            timestamp: SystemTime::now(),
            kind,
            context,
        });
        Ok(())
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

    fn schemas_read(
        &self,
    ) -> Result<RwLockReadGuard<'_, HashMap<String, SymbolSchema>>, GatewayError> {
        self.schemas
            .read()
            .map_err(|_| GatewayError::Poisoned("schemas"))
    }

    fn schemas_write(
        &self,
    ) -> Result<RwLockWriteGuard<'_, HashMap<String, SymbolSchema>>, GatewayError> {
        self.schemas
            .write()
            .map_err(|_| GatewayError::Poisoned("schemas"))
    }

    fn telemetry_write(&self) -> Result<RwLockWriteGuard<'_, Vec<TelemetryEvent>>, GatewayError> {
        self.telemetry
            .write()
            .map_err(|_| GatewayError::Poisoned("telemetry"))
    }
}

impl Default for Gateway {
    fn default() -> Self {
        Self {
            connectors: RwLock::new(HashMap::new()),
            topology: RwLock::new(HashMap::new()),
            schemas: RwLock::new(HashMap::new()),
            telemetry: RwLock::new(Vec::new()),
        }
    }
}

/// Initialize the global gateway and provide a stable foundation.
pub fn init() -> Result<(), GatewayError> {
    let gateway = Gateway::global();
    gateway.bootstrap_defaults()?;
    gateway.auto_scan()?;
    Ok(())
}

fn default_schemas() -> Vec<SymbolSchema> {
    vec![
        SymbolSchema {
            schema_id: "core.analytics.api".into(),
            kind: SymbolKind::Api,
            version: "1.0.0".into(),
            capability_taxonomy: HashSet::from([
                "stream".to_string(),
                "analytics".to_string(),
                "replication".to_string(),
            ]),
            lifecycle: LifecycleStage::Active,
            recommended_zones: HashSet::from(["global".to_string(), "edge".to_string()]),
            compliance_tags: HashSet::from(["pii_safe".to_string(), "audited".to_string()]),
            compatibility: vec![CompatibilityWindow {
                related_kind: SymbolKind::Dataset,
                minimum_version: "1.0.0".into(),
                maximum_version: None,
            }],
            schema_hash: "abc123".into(),
        },
        SymbolSchema {
            schema_id: "edge.render.plugin".into(),
            kind: SymbolKind::Plugin,
            version: "2.1.0".into(),
            capability_taxonomy: HashSet::from([
                "render".to_string(),
                "viz".to_string(),
                "gpu".to_string(),
            ]),
            lifecycle: LifecycleStage::Active,
            recommended_zones: HashSet::from(["edge".to_string(), "workstation".to_string()]),
            compliance_tags: HashSet::from(["license_required".to_string()]),
            compatibility: vec![CompatibilityWindow {
                related_kind: SymbolKind::Service,
                minimum_version: "2.0.0".into(),
                maximum_version: None,
            }],
            schema_hash: "deadbeef".into(),
        },
    ]
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
        gateway.bootstrap_defaults().expect("defaults should load");
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
        gateway.bootstrap_defaults().expect("defaults should load");
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
        gateway.bootstrap_defaults().expect("defaults should load");
        gateway
            .register_schema(SymbolSchema {
                schema_id: "restricted.api".into(),
                kind: SymbolKind::Api,
                version: "1.2.0".into(),
                capability_taxonomy: HashSet::from(["restricted".into()]),
                lifecycle: LifecycleStage::Active,
                recommended_zones: HashSet::from(["private".into()]),
                compliance_tags: HashSet::new(),
                compatibility: vec![],
                schema_hash: "feedface".into(),
            })
            .expect("schema should register");
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

    #[test]
    fn catalog_snapshot_tracks_schema_counts() {
        let gateway = Gateway::new();
        gateway.bootstrap_defaults().expect("defaults should load");

        let snapshot = gateway.catalog_snapshot().expect("snapshot should succeed");
        assert_eq!(snapshot.total, 2);
        assert_eq!(
            snapshot.lifecycle_breakdown.get(&LifecycleStage::Active),
            Some(&2)
        );
        assert!(snapshot.capability_index.contains_key("analytics"));
    }

    #[test]
    fn intent_compiler_parses_yaml_specs() {
        let yaml = r#"
intents:
  - name: replicate analytics stream
    target: api
    capabilities: ["stream", "analytics"]
    constraints:
      max_latency_ms: 20
      min_trust_score: 0.7
      encryption: true
      zones: ["global"]
"#;

        let intents = IntentCompiler::compile(yaml).expect("yaml should parse");
        assert_eq!(intents.len(), 1);
        assert_eq!(intents[0].constraints.max_latency_ms, 20);
        assert!(intents[0].required_capabilities.contains("analytics"));
    }

    #[test]
    fn telemetry_records_key_events() {
        let gateway = Gateway::new();
        gateway.bootstrap_defaults().expect("defaults should load");

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
        gateway.auto_scan().expect("scan should succeed");

        let events = gateway.drain_telemetry().expect("telemetry should drain");
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|event| matches!(event.kind, TelemetryKind::ConnectorRegistered)));
    }
}
