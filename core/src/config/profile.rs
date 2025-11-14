//! Profile manifest loader for kernel capability constraints.

use std::fs;
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;
use thiserror::Error;

use crate::time::current_timestamp_millis;

/// Errors that can occur while loading or interpreting a profile manifest.
#[derive(Debug, Error)]
pub enum ProfileError {
    /// The manifest could not be read from disk.
    #[error("failed to read profile manifest: {0}")]
    Io(#[from] std::io::Error),
    /// The manifest could not be parsed as TOML.
    #[error("failed to parse profile manifest: {0}")]
    Parse(#[from] toml::de::Error),
}

/// Root document describing a runtime profile.
#[derive(Debug, Clone, Deserialize)]
pub struct ProfileDocument {
    pub profile: ProfileMetadata,
    #[serde(default)]
    pub tools: ToolSection,
    #[serde(default)]
    pub egress: EgressSection,
    #[serde(default)]
    pub budgets: BudgetSection,
    #[serde(default)]
    pub storage: StorageSection,
}

impl ProfileDocument {
    /// Load a profile manifest from disk.
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, ProfileError> {
        let raw = fs::read_to_string(path)?;
        let mut document: Self = toml::from_str(&raw)?;
        document.normalise();
        Ok(document)
    }

    fn normalise(&mut self) {
        self.tools.normalise();
        self.egress.normalise();
    }

    /// Convert the profile document into a capability token description.
    pub fn into_capability_token(self, ttl: Duration) -> CapabilityToken {
        let issued_at = current_timestamp_millis();
        let expires_at = issued_at + ttl.as_millis() as u128;

        CapabilityToken {
            profile_name: self.profile.name,
            description: self.profile.description,
            version: self.profile.version,
            allowed_tools: self.tools.allowed,
            denied_tools: self.tools.denied,
            egress_mode: self.egress.mode,
            allowed_egress_destinations: self.egress.allowed_domains,
            egress_notes: self.egress.notes,
            cpu_budget: self.budgets.cpu,
            memory_budget: self.budgets.memory,
            network_budget: self.budgets.network,
            storage_roots: self.storage.roots,
            issued_at_ms: issued_at,
            expires_at_ms: expires_at,
        }
    }
}

/// High-level metadata for the profile.
#[derive(Debug, Clone, Deserialize)]
pub struct ProfileMetadata {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub version: Option<String>,
}

/// Definition of allowed and denied tools.
#[derive(Debug, Clone, Deserialize)]
pub struct ToolSection {
    #[serde(default)]
    pub allowed: Vec<String>,
    #[serde(default)]
    pub denied: Vec<String>,
}

impl Default for ToolSection {
    fn default() -> Self {
        Self {
            allowed: Vec::new(),
            denied: Vec::new(),
        }
    }
}

impl ToolSection {
    fn normalise(&mut self) {
        self.allowed.sort_unstable();
        self.allowed.dedup();
        self.denied.sort_unstable();
        self.denied.dedup();
    }
}

/// Network egress policy.
#[derive(Debug, Clone, Deserialize)]
pub struct EgressSection {
    #[serde(default)]
    pub mode: EgressMode,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

impl Default for EgressSection {
    fn default() -> Self {
        Self {
            mode: EgressMode::AllowList,
            allowed_domains: Vec::new(),
            notes: None,
        }
    }
}

impl EgressSection {
    fn normalise(&mut self) {
        self.allowed_domains.sort_unstable();
        self.allowed_domains.dedup();
    }
}

/// CPU, memory, and network budget definitions.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BudgetSection {
    #[serde(default)]
    pub cpu: Option<CpuBudget>,
    #[serde(default)]
    pub memory: Option<MemoryBudget>,
    #[serde(default)]
    pub network: Option<NetworkBudget>,
}

/// CPU budget (core reservations and limits).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CpuBudget {
    pub reserved_cores: u32,
    pub max_cores: u32,
}

/// Memory budget (soft and hard caps in MiB).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MemoryBudget {
    pub soft_mb: u64,
    pub hard_mb: u64,
}

/// Network budget (egress throughput caps in Mbps).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NetworkBudget {
    pub egress_mbps: u64,
    pub burst_mbps: u64,
}

/// Storage root declarations.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct StorageSection {
    #[serde(default)]
    pub roots: Vec<StorageRoot>,
}

/// Storage mount definition.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct StorageRoot {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub mode: StorageMode,
    #[serde(default)]
    pub quota_mb: Option<u64>,
}

/// Read/write disposition for a storage root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum StorageMode {
    #[serde(rename = "read_only", alias = "ro")]
    ReadOnly,
    #[serde(rename = "read_write", alias = "rw")]
    ReadWrite,
}

impl Default for StorageMode {
    fn default() -> Self {
        StorageMode::ReadOnly
    }
}

/// Network egress mode enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EgressMode {
    Denied,
    AllowList,
    Unrestricted,
}

impl Default for EgressMode {
    fn default() -> Self {
        EgressMode::AllowList
    }
}

/// Capability token derived from a profile manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityToken {
    pub profile_name: String,
    pub description: String,
    pub version: Option<String>,
    pub allowed_tools: Vec<String>,
    pub denied_tools: Vec<String>,
    pub egress_mode: EgressMode,
    pub allowed_egress_destinations: Vec<String>,
    pub egress_notes: Option<String>,
    pub cpu_budget: Option<CpuBudget>,
    pub memory_budget: Option<MemoryBudget>,
    pub network_budget: Option<NetworkBudget>,
    pub storage_roots: Vec<StorageRoot>,
    pub issued_at_ms: u128,
    pub expires_at_ms: u128,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn parses_profile_and_generates_token() {
        let manifest = r#"
            [profile]
            name = "test_profile"
            description = "Test profile"
            version = "1.2.3"

            [tools]
            allowed = ["bash", "cargo", "cargo"]
            denied = ["curl"]

            [egress]
            mode = "allow_list"
            allowed_domains = ["example.com", "example.com"]
            notes = "Only allow example"

            [budgets.cpu]
            reserved_cores = 2
            max_cores = 4

            [budgets.memory]
            soft_mb = 1024
            hard_mb = 2048

            [budgets.network]
            egress_mbps = 50
            burst_mbps = 100

            [[storage.roots]]
            name = "workspace"
            path = "/tmp/workspace"
            mode = "read_write"
            quota_mb = 2048
        "#;

        let file = NamedTempFile::new().expect("create temp file");
        fs::write(file.path(), manifest).expect("write manifest");

        let document = ProfileDocument::load_from_path(file.path()).expect("load profile");
        let token = document.into_capability_token(Duration::from_secs(60));

        assert_eq!(token.profile_name, "test_profile");
        assert_eq!(token.description, "Test profile");
        assert_eq!(token.version.as_deref(), Some("1.2.3"));
        assert_eq!(token.allowed_tools, vec!["bash", "cargo"]);
        assert_eq!(token.denied_tools, vec!["curl"]);
        assert_eq!(token.egress_mode, EgressMode::AllowList);
        assert_eq!(token.allowed_egress_destinations, vec!["example.com"]);
        assert_eq!(
            token.cpu_budget,
            Some(CpuBudget {
                reserved_cores: 2,
                max_cores: 4,
            })
        );
        assert_eq!(
            token.memory_budget,
            Some(MemoryBudget {
                soft_mb: 1024,
                hard_mb: 2048,
            })
        );
        assert_eq!(
            token.network_budget,
            Some(NetworkBudget {
                egress_mbps: 50,
                burst_mbps: 100,
            })
        );
        assert_eq!(token.storage_roots.len(), 1);
        assert_eq!(token.storage_roots[0].name, "workspace");
        assert!(token.expires_at_ms > token.issued_at_ms);
    }
}
