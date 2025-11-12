// Agent Registry - Loads and manages agent metadata from CRC drops
// Integrates the 928-agent directory from the stale/agents drop

use crate::unified_types::{AgentCategory, AgentLayer, AgentMetadata, HealthStatus, RegistryStats};
use crate::{Error, Result};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::Path;
use std::sync::{Arc, RwLock};
use tracing::info;

/// Main agent registry
pub struct AgentRegistry {
    /// All agents indexed by ID
    agents: Arc<RwLock<HashMap<String, AgentMetadata>>>,

    /// Index by layer
    by_layer: Arc<RwLock<HashMap<AgentLayer, Vec<String>>>>,

    /// Index by category
    by_category: Arc<RwLock<HashMap<AgentCategory, Vec<String>>>>,

    /// Registry statistics
    stats: Arc<RwLock<RegistryStats>>,
}

impl AgentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        info!("Initializing agent registry");
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            by_layer: Arc::new(RwLock::new(HashMap::new())),
            by_category: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(RegistryStats::new())),
        }
    }

    /// Convenience constructor that loads the embedded agent directory
    pub fn with_default_data() -> Result<Self> {
        let registry = Self::new();
        registry.load_default()?;
        Ok(registry)
    }

    /// Load agent directory from CSV file on disk
    /// Example path: `agents/data/agent_directory.csv`
    pub fn load_from_csv<P: AsRef<Path>>(&self, csv_path: P) -> Result<usize> {
        info!(
            "Loading agent directory from: {}",
            csv_path.as_ref().display()
        );

        let reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(csv_path)?;

        self.load_from_reader(reader)
    }

    /// Load the embedded default agent directory bundled with this crate
    pub fn load_default(&self) -> Result<usize> {
        info!("Loading embedded agent directory");

        let csv_bytes = include_bytes!("../data/agent_directory.csv");
        let cursor = Cursor::new(csv_bytes.as_ref());

        let reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(cursor);

        self.load_from_reader(reader)
    }

    /// Core CSV parsing logic shared by all loaders
    fn load_from_reader<R: Read>(&self, mut reader: csv::Reader<R>) -> Result<usize> {
        let mut count = 0;
        let mut agents_write = self.agents.write().unwrap();

        for result in reader.records() {
            let record = result?;

            if let Ok(agent) = Self::parse_csv_record(&record) {
                let agent_id = agent.agent_id.clone();
                let is_new = agents_write.insert(agent_id.clone(), agent).is_none();
                if is_new {
                    count += 1;
                } else {
                    info!(
                        "Duplicate agent entry detected, keeping latest: {}",
                        agent_id
                    );
                }
            }
        }

        drop(agents_write);

        self.rebuild_indexes()?;

        info!("✓ Loaded {} agents from registry", count);
        Ok(count)
    }

    /// Parse a CSV record into AgentMetadata
    fn parse_csv_record(record: &csv::StringRecord) -> Result<AgentMetadata> {
        let agent_name = record
            .get(0)
            .ok_or(Error::ParseError("Missing agent_name".into()))?
            .to_string();
        let agent_id = record.get(12).unwrap_or(&agent_name).to_string();

        let mut agent = AgentMetadata::from_registry(agent_name, agent_id);

        // Parse role
        if let Some(role) = record.get(1) {
            agent.role = role.to_string();
        }

        // Parse layer
        if let Some(layer_str) = record.get(2) {
            agent.layer = Self::parse_layer(layer_str);
        }

        // Parse purpose
        if let Some(purpose) = record.get(57) {
            agent.purpose = purpose.to_string();
        }

        // Parse health status
        if let Some(status_str) = record.get(72) {
            agent.health_status = Self::parse_health_status(status_str);
        }

        // Parse issues
        if let Some(issues) = record.get(74) {
            if !issues.is_empty() {
                agent.issues_identified = issues.split('.').map(|s| s.trim().to_string()).collect();
            }
        }

        // Parse repair recommendations
        if let Some(repairs) = record.get(73) {
            if !repairs.is_empty() {
                agent.repair_recommendations =
                    repairs.split('.').map(|s| s.trim().to_string()).collect();
            }
        }

        Ok(agent)
    }

    /// Parse agent layer from string
    ///
    /// Supports both legacy organizational naming and modern L1-L5 technical naming:
    /// - "Executive" (legacy) / "L1" / "L1Autonomy" → L1Autonomy (root governance)
    /// - "Board" (legacy) / "L2" / "L2Reasoning" → L2Reasoning (strategic planning)
    /// - "Stack-Chief" (legacy) / "L3" / "L3Orchestration" → L3Orchestration (coordination)
    /// - "Specialist" (legacy) / "L4" / "L4Operations" → L4Operations (domain experts)
    /// - "Micro" (legacy) / "L5" / "L5Infrastructure" → L5Infrastructure (task-specific)
    fn parse_layer(s: &str) -> AgentLayer {
        match s.to_lowercase().as_str() {
            // L1Autonomy: Root governance (was: Executive)
            "executive" | "l1" | "l1autonomy" => AgentLayer::L1Autonomy,

            // L2Reasoning: Strategic planning (was: Board)
            "board" | "l2" | "l2reasoning" => AgentLayer::L2Reasoning,

            // L3Orchestration: Coordination (was: Stack-Chief)
            "stack-chief" | "stack_chief" | "l3" | "l3orchestration" => AgentLayer::L3Orchestration,

            // L4Operations: Domain experts (was: Specialist)
            "specialist" | "l4" | "l4operations" => AgentLayer::L4Operations,

            // L5Infrastructure: Task-specific (was: Micro)
            "micro" | "l5" | "l5infrastructure" => AgentLayer::L5Infrastructure,

            // Default to L4Operations for unrecognized layers
            _ => AgentLayer::L4Operations,
        }
    }

    /// Parse health status from string
    fn parse_health_status(s: &str) -> HealthStatus {
        match s.to_lowercase().as_str() {
            "healthy" => HealthStatus::Healthy,
            "degraded" => HealthStatus::Degraded,
            "needs repair" | "needs_repair" => HealthStatus::NeedsRepair,
            "error" => HealthStatus::Error,
            _ => HealthStatus::Unknown,
        }
    }

    /// Rebuild all indexes
    fn rebuild_indexes(&self) -> Result<()> {
        let agents = self.agents.read().unwrap();

        let mut by_layer_write = self.by_layer.write().unwrap();
        let mut by_category_write = self.by_category.write().unwrap();
        let mut stats_write = self.stats.write().unwrap();

        by_layer_write.clear();
        by_category_write.clear();
        *stats_write = RegistryStats::new();

        stats_write.total_agents = agents.len();

        for agent in agents.values() {
            // Index by layer
            by_layer_write
                .entry(agent.layer.clone())
                .or_insert_with(Vec::new)
                .push(agent.agent_id.clone());

            // Index by category
            by_category_write
                .entry(agent.category.clone())
                .or_insert_with(Vec::new)
                .push(agent.agent_id.clone());

            // Update stats
            match agent.health_status {
                HealthStatus::Healthy => stats_write.healthy_agents += 1,
                HealthStatus::Degraded => stats_write.needs_repair += 1,
                HealthStatus::NeedsRepair => stats_write.needs_repair += 1,
                HealthStatus::Error => stats_write.needs_repair += 1,
                HealthStatus::Unknown => stats_write.unknown_status += 1,
            }

            *stats_write
                .agents_by_layer
                .entry(agent.layer_name().to_string())
                .or_insert(0) += 1;
        }

        Ok(())
    }

    /// Get agent by ID
    pub fn get(&self, agent_id: &str) -> Option<AgentMetadata> {
        let agents = self.agents.read().unwrap();
        agents.get(agent_id).cloned()
    }

    /// Get all agents
    pub fn all(&self) -> Vec<AgentMetadata> {
        let agents = self.agents.read().unwrap();
        agents.values().cloned().collect()
    }

    /// Get agents by layer
    pub fn by_layer(&self, layer: &AgentLayer) -> Vec<AgentMetadata> {
        let by_layer = self.by_layer.read().unwrap();
        let agents = self.agents.read().unwrap();

        by_layer
            .get(layer)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| agents.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get healthy agents
    pub fn healthy_agents(&self) -> Vec<AgentMetadata> {
        let agents = self.agents.read().unwrap();
        agents
            .values()
            .filter(|a| a.is_healthy())
            .cloned()
            .collect()
    }

    /// Get agents needing repair
    pub fn agents_needing_repair(&self) -> Vec<AgentMetadata> {
        let agents = self.agents.read().unwrap();
        agents
            .values()
            .filter(|a| a.needs_repair())
            .cloned()
            .collect()
    }

    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        let stats = self.stats.read().unwrap();
        stats.clone()
    }

    /// Total agent count
    pub fn count(&self) -> usize {
        let agents = self.agents.read().unwrap();
        agents.len()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_registry() {
        let registry = AgentRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_load_default_registry() {
        let registry = AgentRegistry::new();
        let count = registry.load_default().expect("load embedded directory");
        assert!(count >= 300);
        assert_eq!(count, registry.count());
        let stats = registry.stats();
        assert_eq!(stats.total_agents, count);
    }

    #[test]
    fn test_with_default_data_constructor() {
        let registry = AgentRegistry::with_default_data().expect("construct registry with data");
        assert!(registry.count() >= 300);
    }

    #[test]
    fn test_parse_layer() {
        assert_eq!(AgentRegistry::parse_layer("board"), AgentLayer::L2Reasoning);
        assert_eq!(
            AgentRegistry::parse_layer("Executive"),
            AgentLayer::L1Autonomy
        );
        assert_eq!(
            AgentRegistry::parse_layer("micro"),
            AgentLayer::L5Infrastructure
        );
        assert_eq!(
            AgentRegistry::parse_layer("specialist"),
            AgentLayer::L4Operations
        );
    }
}
