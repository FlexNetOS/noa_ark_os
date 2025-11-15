use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{self, Formatter};
use std::str::FromStr;
use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, SystemTime};

const GENOME_DECAY: f32 = 0.92;
const RELIABILITY_MICRO_THRESHOLD: f32 = 0.18;

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
                SymbolKind::from_str(value).map_err(E::custom)
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
    pub fn with_stable_id(
        name: impl AsRef<str>,
        kind: SymbolKind,
        version: impl Into<String>,
        capabilities: HashSet<String>,
        schema_hash: impl Into<String>,
    ) -> Self {
        let version_string = version.into();
        let schema_hash_string = schema_hash.into();
        let mut capability_fingerprint: Vec<_> = capabilities.iter().cloned().collect();
        capability_fingerprint.sort();
        let signature = format!(
            "{}::{}::{}",
            version_string,
            capability_fingerprint.join("|"),
            schema_hash_string
        );
        let id =
            crate::symbols::stable_symbol_id("gateway", name.as_ref(), kind.as_str(), &signature);
        Self {
            id,
            kind,
            version: version_string,
            capabilities,
            schema_hash: schema_hash_string,
        }
    }

    pub fn stable_id(&self) -> &str {
        &self.id
    }

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
    pub min_attestation_score: f32,
    pub trusted_issuers: HashSet<String>,
    pub required_compliance: HashSet<String>,
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
pub struct IdentityProof {
    pub connector_id: String,
    pub issuer: String,
    pub expires_at: SystemTime,
    pub confidence: f32,
    pub compliance_evidence: HashSet<String>,
    pub hardware_rooted: bool,
}

impl IdentityProof {
    fn is_fresh(&self, now: SystemTime) -> bool {
        self.expires_at > now
    }
}

#[derive(Debug, Clone)]
struct AttestationRecord {
    proof: IdentityProof,
    validated_at: SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SandboxKind {
    Wasm,
    Container,
    Enclave,
    Native,
}

#[derive(Debug, Clone)]
pub struct ToolArtifact {
    pub artifact_id: String,
    pub version: String,
    pub checksum: String,
    pub supported_sandboxes: HashSet<SandboxKind>,
    pub max_parallel_sessions: usize,
}

impl ToolArtifact {
    fn supports(&self, sandbox: &SandboxKind) -> bool {
        self.supported_sandboxes.contains(sandbox)
    }
}

/// Compiled policy output translating an intent into an executable circuit layout.
#[derive(Debug, Clone)]
pub struct CircuitLayout {
    pub connectors: Vec<String>,
    pub optimized_latency_ms: u32,
    pub secure: bool,
    pub proofs: Vec<String>,
}

impl CircuitLayout {
    fn mark_secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }
}

/// Compiles business intents into circuit layouts and records the resulting audit log.
#[derive(Debug, Default)]
pub struct PolicyCompiler {
    verification_log: RwLock<Vec<String>>,
}

impl PolicyCompiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compile(
        &self,
        intent: &Intent,
        candidate_connectors: &[String],
        twin: &SemanticTwin,
    ) -> Result<CircuitLayout, GatewayError> {
        if candidate_connectors.is_empty() {
            return Err(GatewayError::NoRouteFound);
        }

        if !twin.evaluate_intent(intent, candidate_connectors) {
            return Err(GatewayError::VerificationFailed(
                "knowledge graph rejected connector combination".into(),
            ));
        }

        let optimized_latency_ms = candidate_connectors.len().min(3) as u32
            * intent.constraints.max_latency_ms
            / (candidate_connectors.len().max(1) as u32);

        let layout = CircuitLayout {
            connectors: candidate_connectors.to_vec(),
            optimized_latency_ms,
            secure: false,
            proofs: vec![format!("intent:{}", intent.description)],
        };

        {
            let mut log = self
                .verification_log
                .write()
                .map_err(|_| GatewayError::Poisoned("policy_compiler"))?;
            log.push(format!(
                "compiled:{} targets:{}",
                intent.description,
                candidate_connectors.join(",")
            ));
        }

        Ok(layout)
    }

    pub fn verification_log(&self) -> Result<Vec<String>, GatewayError> {
        self.verification_log
            .read()
            .map(|log| log.clone())
            .map_err(|_| GatewayError::Poisoned("policy_compiler"))
    }
}

/// Intent control plane orchestrating compilation and verification flows.
#[derive(Debug, Default)]
pub struct IntentControlPlane {
    policy_compiler: PolicyCompiler,
}

impl IntentControlPlane {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn policy_compiler(&self) -> &PolicyCompiler {
        &self.policy_compiler
    }

    pub fn translate(
        &self,
        intent: &Intent,
        candidate_connectors: &[String],
        twin: &SemanticTwin,
    ) -> Result<CircuitLayout, GatewayError> {
        self.policy_compiler
            .compile(intent, candidate_connectors, twin)
            .map(|layout| layout.mark_secure(true))
    }
}

/// Observed state of a connector.
#[derive(Debug, Clone)]
struct SymbolGenome {
    blueprint: HashMap<String, String>,
    performance_score: f32,
    mutation_history: Vec<String>,
}

impl SymbolGenome {
    fn for_symbol(symbol: &Symbol) -> Self {
        let mut blueprint = HashMap::new();
        blueprint.insert("id".into(), symbol.id.clone());
        blueprint.insert("kind".into(), symbol.kind.to_string());
        blueprint.insert("schema".into(), symbol.schema_hash.clone());
        Self {
            blueprint,
            performance_score: 0.75,
            mutation_history: Vec::new(),
        }
    }

    fn record_signal(&mut self, signal: &str, delta: f32) {
        self.performance_score = (self.performance_score * GENOME_DECAY + delta).clamp(0.0, 1.0);
        self.mutation_history
            .push(format!("{signal}:{delta:.2}:{}", self.performance_score));
        if self.mutation_history.len() > 16 {
            self.mutation_history.remove(0);
        }
    }

    fn propose_mutation(&mut self, capabilities: &HashSet<String>) -> Option<String> {
        if self.performance_score > 0.95 {
            return None;
        }
        let mut fingerprint: Vec<_> = capabilities.iter().cloned().collect();
        fingerprint.sort();
        let mutation = format!(
            "adaptive-{}-{}",
            fingerprint.join("+"),
            self.mutation_history.len() + 1
        );
        self.performance_score = (self.performance_score + 0.12).min(1.0);
        self.mutation_history.push(mutation.clone());
        if self.mutation_history.len() > 16 {
            self.mutation_history.remove(0);
        }
        Some(mutation)
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionSchematic {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
    pub failover_paths: Vec<Vec<String>>,
    pub checksum: String,
}

impl ExecutionSchematic {
    fn for_symbol(symbol: &Symbol, policy: &ConnectionPolicy) -> Self {
        let base_node = format!("symbol:{}", symbol.id);
        let mut nodes = vec![base_node.clone()];
        let mut edges = Vec::new();
        for zone in &policy.allowed_zones {
            let zone_node = format!("zone:{}", zone);
            if !nodes.contains(&zone_node) {
                nodes.push(zone_node.clone());
            }
            edges.push((base_node.clone(), zone_node));
        }
        let failover_paths = if policy.allowed_zones.len() > 1 {
            vec![policy.allowed_zones.iter().cloned().collect()]
        } else {
            vec![]
        };
        let checksum = format!(
            "{}:{}:{}:{}",
            symbol.schema_hash, symbol.version, policy.max_latency_ms, policy.min_trust_score
        );
        Self {
            nodes,
            edges,
            failover_paths,
            checksum,
        }
    }

    fn incorporate_intent(&mut self, intent: &Intent) {
        let intent_node = format!("intent:{}", intent.description);
        if !self.nodes.contains(&intent_node) {
            self.nodes.push(intent_node.clone());
        }
        for capability in &intent.required_capabilities {
            let cap_node = format!("capability:{}", capability);
            if !self.nodes.contains(&cap_node) {
                self.nodes.push(cap_node.clone());
            }
            let edge = (intent_node.clone(), cap_node.clone());
            if !self.edges.contains(&edge) {
                self.edges.push(edge);
            }
        }
        if self.failover_paths.is_empty() {
            self.failover_paths
                .push(vec![intent.target_kind.to_string()]);
        }
        self.checksum = format!("{}#{}", self.checksum, self.nodes.len());
    }

    fn merge(mut self, other: &ExecutionSchematic) -> ExecutionSchematic {
        for node in &other.nodes {
            if !self.nodes.contains(node) {
                self.nodes.push(node.clone());
            }
        }
        for edge in &other.edges {
            if !self.edges.contains(edge) {
                self.edges.push(edge.clone());
            }
        }
        for path in &other.failover_paths {
            if !self.failover_paths.contains(path) {
                self.failover_paths.push(path.clone());
            }
        }
        self.checksum = format!("{}+{}", self.checksum, other.checksum);
        self
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionLease {
    pub artifact_id: String,
    pub sandbox: SandboxKind,
    pub started_at: SystemTime,
    pub shared: bool,
}

#[derive(Debug, Clone)]
struct DemandSignal {
    predicted_load: f32,
    last_updated: SystemTime,
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
pub struct VerificationReport {
    pub model_check_passed: bool,
    pub smt_passed: bool,
    pub failover_proved: bool,
    pub issues: Vec<String>,
}

impl VerificationReport {
    fn success() -> Self {
        Self {
            model_check_passed: true,
            smt_passed: true,
            failover_proved: true,
            issues: Vec::new(),
        }
    }

    fn evaluate(
        intent: &Intent,
        schematic: &ExecutionSchematic,
        connectors: &[ConnectorSnapshot],
        feeds: &HashMap<String, ReliabilityFeed>,
    ) -> Self {
        let mut issues = Vec::new();
        let model_check_passed = connectors.iter().all(|snapshot| {
            snapshot.state != ConnectionState::Faulted
                && snapshot.health_score >= intent.constraints.min_trust_score
        });
        if !model_check_passed {
            issues.push("connectors not healthy".to_string());
        }

        let smt_passed = connectors.iter().all(|snapshot| {
            snapshot.latency_budget <= intent.constraints.max_latency_ms
                && feeds
                    .get(&snapshot.provider_id)
                    .map(|feed| feed.probability_of_failure <= RELIABILITY_MICRO_THRESHOLD)
                    .unwrap_or(true)
        });
        if !smt_passed {
            issues.push("latency or reliability constraint violated".to_string());
        }

        let failover_proved =
            schematic.failover_paths.iter().any(|path| path.len() >= 1) || connectors.len() > 1;
        if !failover_proved {
            issues.push("no failover path available".to_string());
        }

        Self {
            model_check_passed,
            smt_passed,
            failover_proved,
            issues,
        }
    }

    fn is_success(&self) -> bool {
        self.model_check_passed && self.smt_passed && self.failover_proved
    }
}

#[derive(Debug, Clone)]
pub struct ReliabilityFeed {
    pub provider_id: String,
    pub probability_of_failure: f32,
    pub maintenance_windows: Vec<ReliabilityWindow>,
    pub last_update: SystemTime,
}

impl ReliabilityFeed {
    fn is_degraded(&self) -> bool {
        self.probability_of_failure > RELIABILITY_MICRO_THRESHOLD
    }
}

#[derive(Debug, Clone)]
pub struct ReliabilityWindow {
    pub start: SystemTime,
    pub end: SystemTime,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub struct FabricSnapshot {
    pub version_vector: HashMap<String, u64>,
    pub last_sync: SystemTime,
    pub conflicts_resolved: u64,
}

#[derive(Debug)]
struct FabricState {
    version_vector: HashMap<String, u64>,
    last_sync: SystemTime,
    conflicts_resolved: u64,
}

impl FabricState {
    fn new() -> Self {
        Self {
            version_vector: HashMap::new(),
            last_sync: SystemTime::now(),
            conflicts_resolved: 0,
        }
    }
}

#[derive(Debug)]
pub struct CoherenceFabric {
    state: RwLock<FabricState>,
}

impl CoherenceFabric {
    pub fn replicate(&self, scope: &str) -> FabricSnapshot {
        let mut state = self.state.write().expect("coherence fabric lock poisoned");
        let counter = state.version_vector.entry(scope.to_string()).or_insert(0);
        *counter += 1;
        state.last_sync = SystemTime::now();
        FabricSnapshot {
            version_vector: state.version_vector.clone(),
            last_sync: state.last_sync,
            conflicts_resolved: state.conflicts_resolved,
        }
    }

    pub fn observe_conflict(&self) {
        let mut state = self.state.write().expect("coherence fabric lock poisoned");
        state.conflicts_resolved += 1;
    }

    pub fn snapshot(&self) -> FabricSnapshot {
        let state = self.state.read().expect("coherence fabric lock poisoned");
        FabricSnapshot {
            version_vector: state.version_vector.clone(),
            last_sync: state.last_sync,
            conflicts_resolved: state.conflicts_resolved,
        }
    }
}

impl Default for CoherenceFabric {
    fn default() -> Self {
        Self {
            state: RwLock::new(FabricState::new()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GovernanceRecord {
    pub timestamp: SystemTime,
    pub actor: String,
    pub action: String,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Default)]
pub struct GovernanceLayer {
    ledger: RwLock<Vec<GovernanceRecord>>,
}

impl GovernanceLayer {
    fn record(&self, actor: &str, action: &str, details: HashMap<String, String>) {
        let mut ledger = self.ledger.write().expect("governance ledger poisoned");
        ledger.push(GovernanceRecord {
            timestamp: SystemTime::now(),
            actor: actor.to_string(),
            action: action.to_string(),
            details,
        });
    }

    pub fn snapshot(&self) -> Vec<GovernanceRecord> {
        self.ledger
            .read()
            .expect("governance ledger poisoned")
            .clone()
    }
}

#[derive(Debug, Default, Clone)]
struct EvolutionState {
    attempted: usize,
    accepted: usize,
}

#[derive(Debug, Clone)]
struct ConnectorSnapshot {
    id: String,
    provider_id: String,
    state: ConnectionState,
    health_score: f32,
    latency_budget: u32,
}

#[derive(Debug, Clone)]
struct ConnectorRecord {
    symbol: Arc<Symbol>,
    policy: ConnectionPolicy,
    state: ConnectionState,
    last_seen: SystemTime,
    health_score: f32,
    last_attestation: Option<AttestationRecord>,
    execution_lease: Option<ExecutionLease>,
    recent_feedback: VecDeque<bool>,
    genome: SymbolGenome,
    provider_id: String,
    schematic: ExecutionSchematic,
    verification_cache: VerificationReport,
}

impl ConnectorRecord {
    fn new(symbol: Arc<Symbol>, policy: ConnectionPolicy) -> Self {
        let provider_id = symbol.id.split('.').next().unwrap_or("global").to_string();
        let genome = SymbolGenome::for_symbol(&symbol);
        let schematic = ExecutionSchematic::for_symbol(&symbol, &policy);
        Self {
            symbol,
            policy,
            state: ConnectionState::Disconnected,
            last_seen: SystemTime::now(),
            health_score: 0.7,
            last_attestation: None,
            execution_lease: None,
            recent_feedback: VecDeque::with_capacity(20),
            genome,
            provider_id,
            schematic,
            verification_cache: VerificationReport::success(),
        }
    }

    fn snapshot(&self) -> ConnectorSnapshot {
        ConnectorSnapshot {
            id: self.symbol.id.clone(),
            provider_id: self.provider_id.clone(),
            state: self.state,
            health_score: self.health_score,
            latency_budget: self.policy.max_latency_ms,
        }
    }

    fn refresh(&mut self, now: SystemTime, feed: Option<&ReliabilityFeed>) -> ScanEvent {
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

        if !self.attestation_valid(now) {
            self.health_score *= 0.9;
            self.state = ConnectionState::Pending;
            self.genome.record_signal("stale", -0.05);
        } else {
            self.health_score = (self.health_score + 0.08).min(1.0);
            self.state = ConnectionState::Connected;
            self.genome.record_signal("fresh", 0.06);
        }

        if let Some(feed) = feed {
            if feed.is_degraded() {
                self.health_score *= 0.9;
                self.genome.record_signal("reliability", -0.08);
            } else {
                self.genome.record_signal("reliability", 0.04);
            }
        }

        self.last_seen = now;

        ScanEvent {
            connector_id: self.symbol.id.clone(),
            state: self.state,
            health_score: self.health_score,
        }
    }

    fn attestation_valid(&self, now: SystemTime) -> bool {
        self.last_attestation
            .as_ref()
            .map(|record| {
                let compliance_ok = if self.policy.required_compliance.is_empty() {
                    true
                } else {
                    self.policy
                        .required_compliance
                        .is_subset(&record.proof.compliance_evidence)
                };

                let attestation_age = now
                    .duration_since(record.validated_at)
                    .unwrap_or(Duration::ZERO);
                let attestation_fresh = attestation_age <= Duration::from_secs(120);

                record.proof.is_fresh(now)
                    && record.proof.confidence >= self.policy.min_attestation_score
                    && compliance_ok
                    && attestation_fresh
            })
            .unwrap_or(false)
    }

    fn adaptive_score(&self, signal: Option<&DemandSignal>, now: SystemTime) -> f32 {
        let base = self.health_score;
        let attestation_bonus = if self.attestation_valid(now) {
            0.1
        } else {
            -0.2
        };
        let load_penalty = signal
            .map(|s| {
                let staleness = now.duration_since(s.last_updated).unwrap_or(Duration::ZERO);
                let freshness = if staleness > Duration::from_secs(120) {
                    0.0
                } else {
                    1.0 - (staleness.as_secs_f32() / 120.0)
                };
                ((1.0 - s.predicted_load.clamp(0.0, 1.0)) - 0.5) * freshness
            })
            .unwrap_or(0.0);
        let feedback_bias: f32 = if self.recent_feedback.is_empty() {
            0.0
        } else {
            let successes = self
                .recent_feedback
                .iter()
                .filter(|result| **result)
                .count() as f32;
            (successes / self.recent_feedback.len() as f32) - 0.5
        };

        base + attestation_bonus + load_penalty + feedback_bias
    }

    fn record_feedback(&mut self, success: bool) {
        if self.recent_feedback.len() == self.recent_feedback.capacity() {
            self.recent_feedback.pop_front();
        }
        self.recent_feedback.push_back(success);
        if success {
            self.health_score = (self.health_score + 0.05).min(1.0);
        } else {
            self.health_score *= 0.92;
        }
    }

    fn evolve(&mut self, evolution: &mut EvolutionState) -> Option<String> {
        evolution.attempted += 1;
        if let Some(mutation) = self.genome.propose_mutation(&self.symbol.capabilities) {
            evolution.accepted += 1;
            self.schematic.nodes.push(format!("genome:{}", mutation));
            return Some(mutation);
        }
        None
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
    ZeroTrustValidated,
    ToolMounted,
    AdaptiveModelUpdated,
    PredictiveRewire,
    GenomeAdvanced,
    VerificationLoopCompleted,
    GovernanceSettlement,
    FabricReplicated,
}

/// Plan produced after intent compilation and verification.
#[derive(Debug, Clone)]
pub struct RoutePlan {
    pub connectors: Vec<String>,
    pub predicted_latency_ms: u32,
    pub verified: bool,
    pub schematic: ExecutionSchematic,
    pub verification: VerificationReport,
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

        let required_capabilities = self.capabilities.into_iter().collect::<HashSet<_>>();

        Ok(Intent {
            description: self.name,
            target_kind,
            required_capabilities,
            constraints: self.constraints.into_constraints(),
        })
    }
}

/// Individual micro-substation maintaining autonomy in a federated mesh.
#[derive(Debug, Clone)]
pub struct MicroSubstation {
    pub id: String,
    pub zone: String,
    pub local_connectors: HashSet<String>,
    pub autonomy_score: f32,
    last_heartbeat: SystemTime,
}

impl MicroSubstation {
    pub fn new(id: impl Into<String>, zone: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            zone: zone.into(),
            local_connectors: HashSet::new(),
            autonomy_score: 0.75,
            last_heartbeat: SystemTime::now(),
        }
    }

    pub fn record_connector(&mut self, connector: String) {
        self.local_connectors.insert(connector);
        self.autonomy_score = (self.autonomy_score + 0.02).min(1.0);
    }

    pub fn heartbeat(&mut self) {
        self.last_heartbeat = SystemTime::now();
    }
}

/// Consensus mesh synchronising micro-substations for global consistency.
#[derive(Debug, Default)]
pub struct ConsensusMesh {
    stations: RwLock<HashMap<String, MicroSubstation>>,
}

impl ConsensusMesh {
    pub fn new() -> Self {
        let mesh = Self {
            stations: RwLock::new(HashMap::new()),
        };
        mesh.bootstrap();
        mesh
    }

    fn bootstrap(&self) {
        let mut stations = self.stations.write().unwrap();
        if stations.is_empty() {
            stations.insert("alpha".into(), MicroSubstation::new("alpha", "global"));
            stations.insert("beta".into(), MicroSubstation::new("beta", "private"));
            stations.insert("gamma".into(), MicroSubstation::new("gamma", "partner"));
        }
    }

    pub fn register(&self, station: MicroSubstation) {
        self.stations
            .write()
            .unwrap()
            .insert(station.id.clone(), station);
    }

    pub fn assign_connector(&self, connector: &str) {
        let mut stations = self.stations.write().unwrap();
        if stations.is_empty() {
            stations.insert("alpha".into(), MicroSubstation::new("alpha", "global"));
        }

        if let Some((_, station)) = stations
            .iter_mut()
            .min_by_key(|(_, station)| station.local_connectors.len())
        {
            station.record_connector(connector.to_string());
        }
    }

    pub fn heartbeat(&self, station_id: &str) {
        if let Some(station) = self.stations.write().unwrap().get_mut(station_id) {
            station.heartbeat();
        }
    }

    pub fn global_snapshot(&self) -> Vec<MicroSubstation> {
        self.stations.read().unwrap().values().cloned().collect()
    }
}

/// Representation of post-quantum key material for connectors.
#[derive(Debug, Clone)]
pub struct QuantumKeyMaterial {
    pub algorithm: String,
    pub issued_at: SystemTime,
}

/// Security posture ensuring quantum resilient exchanges.
#[derive(Debug)]
pub struct QuantumSecurityPosture {
    keys: RwLock<HashMap<String, QuantumKeyMaterial>>,
    agility_log: RwLock<Vec<String>>,
}

impl QuantumSecurityPosture {
    pub fn new() -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
            agility_log: RwLock::new(vec!["kyber768".into(), "dilithium3".into()]),
        }
    }

    pub fn ensure_connector(&self, connector: &str) {
        let mut keys = self.keys.write().unwrap();
        keys.entry(connector.to_string())
            .or_insert_with(|| QuantumKeyMaterial {
                algorithm: "kyber768".into(),
                issued_at: SystemTime::now(),
            });
    }

    pub fn rekey(&self, connector: &str, algorithm: &str) {
        let mut keys = self.keys.write().unwrap();
        keys.insert(
            connector.to_string(),
            QuantumKeyMaterial {
                algorithm: algorithm.into(),
                issued_at: SystemTime::now(),
            },
        );
        self.agility_log
            .write()
            .unwrap()
            .push(format!("rekey:{}:{}", connector, algorithm));
    }

    pub fn algorithm_for(&self, connector: &str) -> Option<String> {
        self.keys
            .read()
            .unwrap()
            .get(connector)
            .map(|material| material.algorithm.clone())
    }
}

/// Hardware fabric abstraction offloading symbol operations to DPUs/SmartNICs.
#[derive(Debug, Default)]
pub struct HardwareAcceleratedFabric {
    lanes: RwLock<HashMap<String, Vec<String>>>,
}

impl HardwareAcceleratedFabric {
    pub fn new() -> Self {
        Self {
            lanes: RwLock::new(HashMap::new()),
        }
    }

    pub fn offload(&self, connector: &str, operation: &str) {
        let mut lanes = self.lanes.write().unwrap();
        let lane = lanes.entry(operation.into()).or_default();
        if lane.len() > 32 {
            lane.clear();
        }
        lane.push(connector.into());
    }

    pub fn active_lanes(&self) -> HashMap<String, Vec<String>> {
        self.lanes.read().unwrap().clone()
    }
}

#[derive(Debug, Clone)]
struct KnowledgeNode {
    kind: SymbolKind,
    capabilities: HashSet<String>,
    impact_radius: usize,
}

/// Semantic twin with knowledge graph reasoning.
#[derive(Debug, Default)]
pub struct SemanticTwin {
    graph: RwLock<HashMap<String, KnowledgeNode>>,
}

impl SemanticTwin {
    pub fn new() -> Self {
        Self {
            graph: RwLock::new(HashMap::new()),
        }
    }

    pub fn ingest_symbol(&self, symbol: &Symbol) {
        let node = KnowledgeNode {
            kind: symbol.kind.clone(),
            capabilities: symbol.capabilities.clone(),
            impact_radius: symbol.capabilities.len().max(1),
        };

        self.graph.write().unwrap().insert(symbol.id.clone(), node);
    }

    pub fn evaluate_intent(&self, intent: &Intent, connectors: &[String]) -> bool {
        let graph = self.graph.read().unwrap();
        connectors.iter().all(|connector| {
            graph
                .get(connector)
                .map(|node| {
                    node.kind == intent.target_kind
                        && intent
                            .required_capabilities
                            .iter()
                            .all(|cap| node.capabilities.contains(cap))
                })
                .unwrap_or(false)
        })
    }

    pub fn predict_risk(&self, connectors: &[String]) -> Vec<String> {
        let graph = self.graph.read().unwrap();
        let mut findings = Vec::new();
        for connector in connectors {
            if let Some(node) = graph.get(connector) {
                if node.capabilities.contains("restricted") && node.impact_radius > 1 {
                    findings.push(format!("connector:{} high impact", connector));
                }
            }
        }
        findings
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
    GovernanceViolation(String),
    NoRouteFound,
    VerificationFailed(String),
    Poisoned(&'static str),
    SchemaConflict(String),
    SchemaNotFound(String),
    IntentParse(String),
    AttestationFailed(String),
    ToolNotFound(String),
    SandboxUnsupported(String),
    ToolCapacity(String),
    ToolConflict(String),
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GatewayError::AlreadyRegistered(id) => {
                write!(f, "connector {id} is already registered")
            }
            GatewayError::NotFound(id) => write!(f, "connector {id} not found"),
            GatewayError::PolicyViolation(msg) => write!(f, "policy violation: {msg}"),
            GatewayError::GovernanceViolation(msg) => write!(f, "governance violation: {msg}"),
            GatewayError::NoRouteFound => write!(f, "no viable route found for intent"),
            GatewayError::VerificationFailed(msg) => write!(f, "verification failed: {msg}"),
            GatewayError::Poisoned(resource) => write!(f, "gateway state poisoned: {resource}"),
            GatewayError::SchemaConflict(id) => write!(f, "schema {id} already registered"),
            GatewayError::SchemaNotFound(id) => write!(f, "schema {id} not found"),
            GatewayError::IntentParse(msg) => write!(f, "intent parsing error: {msg}"),
            GatewayError::AttestationFailed(msg) => write!(f, "attestation failed: {msg}"),
            GatewayError::ToolNotFound(id) => write!(f, "tool artifact {id} not found"),
            GatewayError::SandboxUnsupported(msg) => {
                write!(f, "sandbox configuration unsupported: {msg}")
            }
            GatewayError::ToolCapacity(msg) => write!(f, "tool capacity exhausted: {msg}"),
            GatewayError::ToolConflict(id) => write!(f, "tool artifact {id} already registered"),
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
    tool_catalog: RwLock<HashMap<String, ToolArtifact>>,
    tool_sessions: RwLock<HashMap<String, usize>>,
    demand_signals: RwLock<HashMap<SymbolKind, DemandSignal>>,
    reliability_feeds: RwLock<HashMap<String, ReliabilityFeed>>,
    evolution: RwLock<EvolutionState>,
    coherence: CoherenceFabric,
    governance: GovernanceLayer,
    control_plane: IntentControlPlane,
    federation: ConsensusMesh,
    security_posture: QuantumSecurityPosture,
    symbol_fabric: HardwareAcceleratedFabric,
    semantic_twin: SemanticTwin,
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

        if !schema
            .compliance_tags
            .is_superset(&policy.required_compliance)
        {
            return Err(GatewayError::PolicyViolation(format!(
                "symbol {} lacks required compliance tags {:?}",
                symbol.id, policy.required_compliance
            )));
        }

        if !schema.capability_taxonomy.is_superset(&symbol.capabilities) {
            return Err(GatewayError::PolicyViolation(format!(
                "symbol {} declares capabilities not advertised by schema {}",
                symbol.id, schema.schema_id
            )));
        }

        if schema.compliance_tags.contains("pii_safe") && !policy.encryption_required {
            return Err(GatewayError::GovernanceViolation(
                "pii_safe schemas require encryption".into(),
            ));
        }

        let id = symbol.id.clone();
        let kind = symbol.kind.clone();
        let arc_symbol = Arc::new(symbol);
        let capabilities = arc_symbol.capabilities.clone();
        let schema_hash = arc_symbol.schema_hash.clone();

        let mut connectors = self.connectors_write()?;
        if connectors.contains_key(&id) {
            return Err(GatewayError::AlreadyRegistered(id));
        }

        let mut record = ConnectorRecord::new(arc_symbol.clone(), policy);
        {
            let mut evolution = self.evolution_write()?;
            if record.genome.propose_mutation(&capabilities).is_some() {
                evolution.accepted += 1;
            }
        }

        connectors.insert(id.clone(), record);
        drop(connectors);

        self.topology_write()?
            .entry(kind)
            .or_default()
            .insert(id.clone());

        self.governance.record(
            "gateway",
            "register_connector",
            HashMap::from([
                ("connector".into(), id.clone()),
                ("schema".into(), schema_hash.clone()),
            ]),
        );
        self.coherence.replicate("catalog");
        self.semantic_twin.ingest_symbol(arc_symbol.as_ref());
        self.federation.assign_connector(&id);
        self.security_posture.ensure_connector(&id);
        self.symbol_fabric.offload(&id, "registration");

        self.emit_event(
            TelemetryKind::ConnectorRegistered,
            format!("{}@{}", arc_symbol.id, arc_symbol.version),
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
        self.coherence.replicate("connections");
        self.governance.record(
            "gateway",
            "connect",
            HashMap::from([("connector".into(), connector_id.to_string())]),
        );
        drop(connectors);
        self.security_posture.ensure_connector(connector_id);
        self.symbol_fabric.offload(connector_id, "connect");
        self.federation.heartbeat("alpha");
        Ok(())
    }

    /// Disconnect a connector from the routing fabric.
    pub fn disconnect(&self, connector_id: &str) -> Result<(), GatewayError> {
        let mut connectors = self.connectors_write()?;
        let record = connectors
            .get_mut(connector_id)
            .ok_or_else(|| GatewayError::NotFound(connector_id.to_string()))?;
        record.state = ConnectionState::Disconnected;
        self.coherence.replicate("connections");
        self.governance.record(
            "gateway",
            "disconnect",
            HashMap::from([("connector".into(), connector_id.to_string())]),
        );
        Ok(())
    }

    /// Perform an auto-scan, updating health and discovering risky connectors.
    pub fn auto_scan(&self) -> Result<Vec<ScanEvent>, GatewayError> {
        let reliability = {
            let feeds = self.reliability_read()?;
            feeds.clone()
        };
        let now = SystemTime::now();
        let mut connectors = self.connectors_write()?;
        let mut evolution = self.evolution_write()?;
        let mut events = Vec::new();

        for record in connectors.values_mut() {
            let event = record.refresh(now, reliability.get(&record.provider_id));
            if let Some(mutation) = record.evolve(&mut evolution) {
                self.emit_event(
                    TelemetryKind::GenomeAdvanced,
                    format!("{}->{mutation}", record.symbol.id),
                )?;
            }
            events.push(event);
        }

        drop(connectors);
        drop(evolution);

        self.emit_event(
            TelemetryKind::ScanCompleted,
            format!("scan:{}", events.len()),
        )?;
        self.coherence.replicate("scan");
        Ok(events)
    }

    /// Calculate an optimized route for a given intent.
    pub fn route_intent(&self, intent: &Intent) -> Result<RoutePlan, GatewayError> {
        #[derive(Clone)]
        struct CandidateView {
            id: String,
            provider_id: String,
            max_latency: u32,
            snapshot: ConnectorSnapshot,
            schematic: ExecutionSchematic,
            score: f32,
        }

        let reliability = {
            let feeds = self.reliability_read()?;
            feeds.clone()
        };
        let demand_signals = {
            let signals = self.demand_signals_read()?;
            signals.clone()
        };
        let now = SystemTime::now();
        let demand_signal = demand_signals.get(&intent.target_kind);

        let connectors_guard = self.connectors_read()?;
        let mut candidates: Vec<CandidateView> = connectors_guard
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
            .map(|record| {
                let reliability_penalty = reliability
                    .get(&record.provider_id)
                    .map(|feed| feed.probability_of_failure)
                    .unwrap_or(0.0);
                let score = record.adaptive_score(demand_signal, now) - reliability_penalty;
                CandidateView {
                    id: record.symbol.id.clone(),
                    provider_id: record.provider_id.clone(),
                    max_latency: record.policy.max_latency_ms,
                    snapshot: record.snapshot(),
                    schematic: record.schematic.clone(),
                    score,
                }
            })
            .collect();
        drop(connectors_guard);

        if candidates.is_empty() {
            return Err(GatewayError::NoRouteFound);
        }

        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut schematic = candidates[0].schematic.clone();
        schematic.incorporate_intent(intent);
        for view in candidates.iter().skip(1) {
            schematic = schematic.merge(&view.schematic);
        }

        let selected: Vec<String> = candidates
            .iter()
            .take(3)
            .map(|view| view.id.clone())
            .collect();

        let predicted_latency_ms = candidates
            .iter()
            .map(|view| view.max_latency)
            .min()
            .unwrap_or(intent.constraints.max_latency_ms);

        let layout = self
            .control_plane
            .translate(intent, &selected, &self.semantic_twin)?;

        let snapshots: Vec<ConnectorSnapshot> = candidates
            .iter()
            .take(3)
            .map(|view| view.snapshot.clone())
            .collect();

        let verification =
            VerificationReport::evaluate(intent, &schematic, &snapshots, &reliability);

        self.governance.record(
            "gateway",
            "verification",
            HashMap::from([
                ("intent".into(), intent.description.clone()),
                ("success".into(), verification.is_success().to_string()),
                ("issues".into(), verification.issues.join("|")),
            ]),
        );
        self.emit_event(
            TelemetryKind::VerificationLoopCompleted,
            format!(
                "intent:{} success:{}",
                intent.description,
                verification.is_success()
            ),
        )?;

        if !verification.is_success() {
            return Err(GatewayError::VerificationFailed(
                verification.issues.join("; "),
            ));
        }

        let mut plan = RoutePlan {
            connectors: layout.connectors.clone(),
            predicted_latency_ms: layout.optimized_latency_ms.min(predicted_latency_ms),
            verified: false,
            schematic,
            verification,
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
            self.emit_event(
                TelemetryKind::AdaptiveModelUpdated,
                format!("intent:{}", intent.description),
            )?;

            let risks = self.semantic_twin.predict_risk(&plan.connectors);
            for connector in &plan.connectors {
                self.security_posture.rekey(connector, "falcon1024");
                self.symbol_fabric.offload(connector, "route");
            }
            if !risks.is_empty() {
                self.emit_event(TelemetryKind::SelfHealSuggested, risks.join("|"))?;
            }

            self.coherence.replicate("routing");
            Ok(plan)
        } else {
            Err(GatewayError::VerificationFailed(
                "intent constraints not satisfied in topology".into(),
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
        if let Some(allowed_set) = allowed {
            if !plan
                .connectors
                .iter()
                .all(|connector| allowed_set.contains(connector))
            {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }

        Ok(plan.predicted_latency_ms <= intent.constraints.max_latency_ms)
    }

    /// Run predictive self healing, returning any actions to be executed.
    pub fn predictive_self_heal(&self) -> Result<Vec<SelfHealAction>, GatewayError> {
        let reliability = {
            let feeds = self.reliability_read()?;
            feeds.clone()
        };
        let mut actions = Vec::new();
        let mut connectors = self.connectors_write()?;

        for record in connectors.values_mut() {
            let risk = reliability
                .get(&record.provider_id)
                .map(|feed| feed.probability_of_failure)
                .unwrap_or(0.0);
            if risk > RELIABILITY_MICRO_THRESHOLD && record.state == ConnectionState::Connected {
                record.state = ConnectionState::Pending;
                actions.push(SelfHealAction {
                    connector_id: record.symbol.id.clone(),
                    action: format!("predictive-rewire (risk {:.2})", risk),
                });
            }

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
            self.emit_event(
                TelemetryKind::PredictiveRewire,
                format!("hedged:{}", actions.len()),
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

    /// Perform a zero-trust handshake before connectors are energized.
    pub fn verify_attestation(&self, proof: IdentityProof) -> Result<(), GatewayError> {
        let mut connectors = self.connectors_write()?;
        let record = connectors
            .get_mut(&proof.connector_id)
            .ok_or_else(|| GatewayError::NotFound(proof.connector_id.clone()))?;

        if !record.policy.trusted_issuers.contains(&proof.issuer) {
            return Err(GatewayError::AttestationFailed(format!(
                "issuer {} not trusted for connector {}",
                proof.issuer, proof.connector_id
            )));
        }

        if proof.confidence < record.policy.min_attestation_score {
            return Err(GatewayError::AttestationFailed(format!(
                "attestation score {} below minimum {}",
                proof.confidence, record.policy.min_attestation_score
            )));
        }

        if !proof.is_fresh(SystemTime::now()) {
            return Err(GatewayError::AttestationFailed(
                "attestation expired".into(),
            ));
        }

        if !record
            .policy
            .required_compliance
            .is_subset(&proof.compliance_evidence)
        {
            return Err(GatewayError::AttestationFailed(
                "proof missing required compliance evidence".into(),
            ));
        }

        if record.policy.encryption_required && !proof.hardware_rooted {
            return Err(GatewayError::AttestationFailed(
                "connector lacks confidential-compute proof".into(),
            ));
        }

        record.last_attestation = Some(AttestationRecord {
            proof: proof.clone(),
            validated_at: SystemTime::now(),
        });
        record.state = ConnectionState::Pending;
        record.health_score = (record.health_score + 0.1).min(1.0);

        drop(connectors);

        self.emit_event(
            TelemetryKind::ZeroTrustValidated,
            format!("{}@{}", proof.connector_id, proof.issuer),
        )
    }

    /// Register an execution artifact that can be shared across connectors.
    pub fn register_tool_artifact(&self, artifact: ToolArtifact) -> Result<(), GatewayError> {
        let mut catalog = self
            .tool_catalog
            .write()
            .map_err(|_| GatewayError::Poisoned("tool_catalog"))?;
        if catalog.contains_key(&artifact.artifact_id) {
            return Err(GatewayError::ToolConflict(artifact.artifact_id));
        }
        catalog.insert(artifact.artifact_id.clone(), artifact);
        Ok(())
    }

    /// Mount a tool artifact inside a sandbox for a connector, enabling shared execution.
    pub fn assign_tool(
        &self,
        connector_id: &str,
        artifact_id: &str,
        sandbox: SandboxKind,
    ) -> Result<ExecutionLease, GatewayError> {
        let catalog = self
            .tool_catalog
            .read()
            .map_err(|_| GatewayError::Poisoned("tool_catalog"))?;
        let artifact = catalog
            .get(artifact_id)
            .cloned()
            .ok_or_else(|| GatewayError::ToolNotFound(artifact_id.into()))?;

        if !artifact.supports(&sandbox) {
            return Err(GatewayError::SandboxUnsupported(format!(
                "artifact {} does not support {:?}",
                artifact_id, sandbox
            )));
        }
        drop(catalog);

        let mut sessions = self
            .tool_sessions
            .write()
            .map_err(|_| GatewayError::Poisoned("tool_sessions"))?;
        let current = sessions.entry(artifact_id.into()).or_insert(0);
        if *current >= artifact.max_parallel_sessions {
            return Err(GatewayError::ToolCapacity(artifact_id.into()));
        }
        *current += 1;
        drop(sessions);

        let lease = ExecutionLease {
            artifact_id: artifact.artifact_id.clone(),
            sandbox,
            started_at: SystemTime::now(),
            shared: artifact.max_parallel_sessions > 1,
        };

        let mut connectors = self.connectors_write()?;
        let record = connectors
            .get_mut(connector_id)
            .ok_or_else(|| GatewayError::NotFound(connector_id.to_string()))?;
        record.execution_lease = Some(lease.clone());
        drop(connectors);

        self.emit_event(
            TelemetryKind::ToolMounted,
            format!("{}->{}", connector_id, artifact_id),
        )?;

        Ok(lease)
    }

    /// Release an execution lease once a connector is done with the shared tool.
    pub fn release_tool(&self, connector_id: &str) -> Result<(), GatewayError> {
        let mut connectors = self.connectors_write()?;
        let record = connectors
            .get_mut(connector_id)
            .ok_or_else(|| GatewayError::NotFound(connector_id.to_string()))?;
        if let Some(lease) = record.execution_lease.take() {
            drop(connectors);
            let mut sessions = self
                .tool_sessions
                .write()
                .map_err(|_| GatewayError::Poisoned("tool_sessions"))?;
            if let Some(count) = sessions.get_mut(&lease.artifact_id) {
                *count = count.saturating_sub(1);
            }
        } else {
            drop(connectors);
        }
        Ok(())
    }

    /// Update demand forecasts so adaptive routing can prepare capacity ahead of time.
    pub fn update_demand_signal(
        &self,
        kind: SymbolKind,
        predicted_load: f32,
    ) -> Result<(), GatewayError> {
        let mut demand = self
            .demand_signals
            .write()
            .map_err(|_| GatewayError::Poisoned("demand_signals"))?;
        demand.insert(
            kind,
            DemandSignal {
                predicted_load: predicted_load.clamp(0.0, 1.0),
                last_updated: SystemTime::now(),
            },
        );
        Ok(())
    }

    /// Feed reinforcement signals so the router can learn from outcomes.
    pub fn record_route_feedback(
        &self,
        connector_id: &str,
        success: bool,
    ) -> Result<(), GatewayError> {
        let mut connectors = self.connectors_write()?;
        let record = connectors
            .get_mut(connector_id)
            .ok_or_else(|| GatewayError::NotFound(connector_id.to_string()))?;
        record.record_feedback(success);
        Ok(())
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

    pub fn fabric_snapshot(&self) -> FabricSnapshot {
        self.coherence.snapshot()
    }

    pub fn governance_snapshot(&self) -> Vec<GovernanceRecord> {
        self.governance.snapshot()
    }

    pub fn update_reliability_feed(&self, feed: ReliabilityFeed) -> Result<(), GatewayError> {
        let mut feeds = self.reliability_write()?;
        feeds.insert(feed.provider_id.clone(), feed.clone());
        drop(feeds);
        self.emit_event(
            TelemetryKind::PredictiveRewire,
            format!(
                "provider:{} risk:{:.2}",
                feed.provider_id, feed.probability_of_failure
            ),
        )?;
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

    fn demand_signals_read(
        &self,
    ) -> Result<RwLockReadGuard<'_, HashMap<SymbolKind, DemandSignal>>, GatewayError> {
        self.demand_signals
            .read()
            .map_err(|_| GatewayError::Poisoned("demand_signals"))
    }

    fn reliability_read(
        &self,
    ) -> Result<RwLockReadGuard<'_, HashMap<String, ReliabilityFeed>>, GatewayError> {
        self.reliability_feeds
            .read()
            .map_err(|_| GatewayError::Poisoned("reliability"))
    }

    fn reliability_write(
        &self,
    ) -> Result<RwLockWriteGuard<'_, HashMap<String, ReliabilityFeed>>, GatewayError> {
        self.reliability_feeds
            .write()
            .map_err(|_| GatewayError::Poisoned("reliability"))
    }

    fn evolution_write(&self) -> Result<RwLockWriteGuard<'_, EvolutionState>, GatewayError> {
        self.evolution
            .write()
            .map_err(|_| GatewayError::Poisoned("evolution"))
    }
}

impl Default for Gateway {
    fn default() -> Self {
        Self {
            connectors: RwLock::new(HashMap::new()),
            topology: RwLock::new(HashMap::new()),
            schemas: RwLock::new(HashMap::new()),
            telemetry: RwLock::new(Vec::new()),
            tool_catalog: RwLock::new(HashMap::new()),
            tool_sessions: RwLock::new(HashMap::new()),
            demand_signals: RwLock::new(HashMap::new()),
            reliability_feeds: RwLock::new(HashMap::new()),
            evolution: RwLock::new(EvolutionState::default()),
            coherence: CoherenceFabric {
                state: RwLock::new(FabricState::new()),
            },
            governance: GovernanceLayer::default(),
            control_plane: IntentControlPlane::new(),
            federation: ConsensusMesh::new(),
            security_posture: QuantumSecurityPosture::new(),
            symbol_fabric: HardwareAcceleratedFabric::new(),
            semantic_twin: SemanticTwin::new(),
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
    use std::time::{Duration, SystemTime};

    fn sample_policy() -> ConnectionPolicy {
        ConnectionPolicy {
            max_latency_ms: 10,
            min_trust_score: 0.9,
            allowed_zones: HashSet::from(["global".into(), "edge".into()]),
            encryption_required: true,
            min_attestation_score: 0.85,
            trusted_issuers: HashSet::from(["noa-ca".into()]),
            required_compliance: HashSet::new(),
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

    fn sample_proof(connector_id: &str) -> IdentityProof {
        IdentityProof {
            connector_id: connector_id.into(),
            issuer: "noa-ca".into(),
            expires_at: SystemTime::now() + Duration::from_secs(60),
            confidence: 0.95,
            compliance_evidence: HashSet::new(),
            hardware_rooted: true,
        }
    }

    fn register_sample_symbol(gateway: &Gateway, id: &str) -> Symbol {
        let symbol = Symbol::with_stable_id(
            id,
            SymbolKind::Api,
            "1.0.0",
            HashSet::from(["stream".into(), "analytics".into()]),
            "abc123",
        );
        gateway
            .register_symbol(symbol.clone(), sample_policy())
            .expect("registration should succeed");
        gateway
            .verify_attestation(sample_proof(&symbol.id))
            .expect("attestation should succeed");
        gateway.connect(&symbol.id).expect("connect should succeed");
        symbol
    }

    #[test]
    fn register_and_route_symbol() {
        let gateway = Gateway::new();
        gateway.bootstrap_defaults().expect("defaults should load");
        let symbol = register_sample_symbol(&gateway, "analytics.api");

        let intent = Intent {
            description: "Replicate analytics stream".into(),
            target_kind: SymbolKind::Api,
            required_capabilities: HashSet::from(["stream".into()]),
            constraints: sample_constraints(),
        };

        let plan = gateway.route_intent(&intent).expect("route should succeed");
        assert!(plan.verified);
        assert!(plan.verification.is_success());
        assert!(plan.connectors.contains(&symbol.id));
        assert!(!plan.schematic.nodes.is_empty());
    }

    #[test]
    fn predictive_self_heal_flags_faults() {
        let gateway = Gateway::new();
        gateway.bootstrap_defaults().expect("defaults should load");
        let symbol = register_sample_symbol(&gateway, "analytics.api");

        {
            let mut connectors = gateway.connectors.write().unwrap();
            let record = connectors.get_mut(&symbol.id).unwrap();
            record.state = ConnectionState::Faulted;
            record.health_score = 0.2;
        }

        let actions = gateway
            .predictive_self_heal()
            .expect("self healing should succeed");
        assert!(actions
            .iter()
            .any(|action| action.connector_id == symbol.id));
    }

    #[test]
    fn control_plane_records_policy_trace() {
        let gateway = Gateway::new();
        gateway.bootstrap_defaults().expect("defaults should load");
        register_sample_symbol(&gateway, "analytics.api");

        let intent = Intent {
            description: "Replicate analytics stream".into(),
            target_kind: SymbolKind::Api,
            required_capabilities: HashSet::from(["stream".into()]),
            constraints: sample_constraints(),
        };

        gateway.route_intent(&intent).expect("route should succeed");
        let log = gateway
            .control_plane
            .policy_compiler()
            .verification_log()
            .expect("log available");
        assert!(!log.is_empty());
    }

    #[test]
    fn semantic_twin_blocks_restricted_route() {
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

        let symbol = Symbol::with_stable_id(
            "restricted.api",
            SymbolKind::Api,
            "1.2.0",
            HashSet::from(["restricted".into()]),
            "feedface",
        );

        let mut policy = sample_policy();
        policy.allowed_zones = HashSet::from(["private".into()]);
        gateway
            .register_symbol(symbol.clone(), policy)
            .expect("registration should succeed");
        gateway
            .verify_attestation(sample_proof(&symbol.id))
            .expect("attestation should succeed");
        gateway.connect(&symbol.id).expect("connect should succeed");

        let intent = Intent {
            description: "Access restricted api".into(),
            target_kind: SymbolKind::Api,
            required_capabilities: HashSet::from(["restricted".into()]),
            constraints: sample_constraints(),
        };

        let result = gateway.route_intent(&intent);
        assert!(matches!(
            result,
            Err(GatewayError::VerificationFailed(_)) | Err(GatewayError::NoRouteFound)
        ));
    }

    #[test]
    fn reliability_feed_guides_self_heal() {
        let gateway = Gateway::new();
        gateway.bootstrap_defaults().expect("defaults should load");
        register_sample_symbol(&gateway, "analytics.primary");
        register_sample_symbol(&gateway, "analytics.secondary");

        let provider_id = gateway
            .connectors_read()
            .expect("connectors accessible")
            .values()
            .next()
            .map(|record| record.provider_id.clone());
        assert!(
            provider_id.is_some(),
            "Test requires at least one connector in the registry"
        );
        let provider_id = provider_id.unwrap();

        gateway
            .update_reliability_feed(ReliabilityFeed {
                provider_id,
                probability_of_failure: 0.6,
                maintenance_windows: vec![],
                last_update: SystemTime::now(),
            })
            .expect("feed should update");

        let actions = gateway
            .predictive_self_heal()
            .expect("healing should succeed");
        assert!(!actions.is_empty());
    }
}
