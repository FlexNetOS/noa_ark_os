//! Kernel manifest schema describing capabilities and runtime dependencies.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Capability identifier for the process subsystem.
pub const CAPABILITY_PROCESS: &str = "core.process";
/// Capability identifier for the memory subsystem.
pub const CAPABILITY_MEMORY: &str = "core.memory";
/// Capability identifier for the IPC subsystem.
pub const CAPABILITY_IPC: &str = "core.ipc";
/// Capability identifier for the file-system subsystem.
pub const CAPABILITY_FILESYSTEM: &str = "core.fs";
/// Capability identifier for the security subsystem.
pub const CAPABILITY_SECURITY: &str = "core.security";
/// Capability identifier for the gateway fabric.
pub const CAPABILITY_GATEWAY: &str = "core.gateway";
/// Capability identifier for the runtime manager.
pub const CAPABILITY_RUNTIME_MANAGER: &str = "core.runtime.manager";
/// Capability identifier for the agent factory subsystem.
pub const CAPABILITY_AGENT_FACTORY: &str = "agents.factory";
/// Scope granting host environment takeover privileges.
pub const SCOPE_HOST_ENVIRONMENT_TAKEOVER: &str = "host.environment.takeover";
/// Scope granting host resource arbitration privileges.
pub const SCOPE_HOST_RESOURCE_ARBITRATE: &str = "host.resource.arbitrate";

fn default_autostart() -> bool {
    true
}

fn default_policy_ttl() -> u64 {
    900
}

/// Root manifest describing kernel capabilities and runtimes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelManifest {
    /// Manifest schema version.
    pub version: String,
    /// Capability entries describing providers and dependencies.
    #[serde(default)]
    pub capabilities: Vec<CapabilityManifestEntry>,
    /// Runtime plugin definitions.
    #[serde(default)]
    pub runtimes: Vec<RuntimeManifestEntry>,
    /// Optional metadata for downstream tooling.
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
    /// Token policies describing scope configuration for capability tokens.
    #[serde(default)]
    pub token_policies: Vec<TokenPolicyManifestEntry>,
}

impl Default for KernelManifest {
    fn default() -> Self {
        // Delegate to the canonical constructor to avoid duplication.
        Self::manifest_default()
    }
}

impl KernelManifest {
    fn manifest_default() -> Self {
        let mut capabilities = vec![
            CapabilityManifestEntry::new(CAPABILITY_PROCESS),
            CapabilityManifestEntry::new(CAPABILITY_MEMORY),
            CapabilityManifestEntry::new(CAPABILITY_IPC),
            CapabilityManifestEntry::new(CAPABILITY_FILESYSTEM),
            CapabilityManifestEntry::new(CAPABILITY_SECURITY),
            CapabilityManifestEntry {
                id: CAPABILITY_GATEWAY.to_string(),
                depends_on: vec![
                    CAPABILITY_PROCESS.to_string(),
                    CAPABILITY_MEMORY.to_string(),
                    CAPABILITY_SECURITY.to_string(),
                ],
                ..CapabilityManifestEntry::new(CAPABILITY_GATEWAY)
            },
            CapabilityManifestEntry {
                id: CAPABILITY_RUNTIME_MANAGER.to_string(),
                depends_on: vec![
                    CAPABILITY_PROCESS.to_string(),
                    CAPABILITY_MEMORY.to_string(),
                    CAPABILITY_SECURITY.to_string(),
                ],
                ..CapabilityManifestEntry::new(CAPABILITY_RUNTIME_MANAGER)
            },
        ];

        let mut agent_factory_capability = CapabilityManifestEntry::new(CAPABILITY_AGENT_FACTORY);
        agent_factory_capability.depends_on = vec![
            CAPABILITY_PROCESS.to_string(),
            CAPABILITY_MEMORY.to_string(),
            CAPABILITY_SECURITY.to_string(),
            CAPABILITY_GATEWAY.to_string(),
        ];
        agent_factory_capability.autostart = false;
        capabilities.push(agent_factory_capability);

        let runtimes = vec![
            RuntimeManifestEntry::new("rust", RuntimeKind::Rust, "1.75", "bin/noa_kernel"),
            RuntimeManifestEntry::new(
                "python",
                RuntimeKind::Python,
                "3.11",
                "python runtime/bootstrap.py",
            ),
            RuntimeManifestEntry::new("go", RuntimeKind::Go, "1.21", "go/bin/runtime"),
            RuntimeManifestEntry::new(
                "dotnet",
                RuntimeKind::DotNet,
                "8.0",
                "dotnet/Noa.Runtime.dll",
            ),
        ];

        let token_policies = vec![
            TokenPolicyManifestEntry {
                scope: SCOPE_HOST_ENVIRONMENT_TAKEOVER.to_string(),
                description: Some(
                    "Allows an actor to request exclusive control over a managed environment"
                        .to_string(),
                ),
                ttl_seconds: 600,
                capabilities: vec![
                    CAPABILITY_PROCESS.to_string(),
                    CAPABILITY_SECURITY.to_string(),
                ],
            },
            TokenPolicyManifestEntry {
                scope: SCOPE_HOST_RESOURCE_ARBITRATE.to_string(),
                description: Some(
                    "Allows an actor to arbitrate CPU/memory allocations for an environment"
                        .to_string(),
                ),
                ttl_seconds: 600,
                capabilities: vec![
                    CAPABILITY_PROCESS.to_string(),
                    CAPABILITY_MEMORY.to_string(),
                    CAPABILITY_SECURITY.to_string(),
                ],
            },
        ];

        Self {
            version: "1.0".to_string(),
            capabilities,
            runtimes,
            metadata: HashMap::new(),
            token_policies,
        }
    }
    /// Load a manifest from a YAML file.
    pub fn load_from_yaml(path: impl AsRef<Path>) -> Result<Self, ManifestError> {
        let content = fs::read_to_string(path).map_err(ManifestError::Io)?;
        let manifest: Self = serde_yaml::from_str(&content).map_err(ManifestError::Parse)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Provide a manifest populated with the built-in capabilities and runtimes.
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self { Self::manifest_default() }

    /// Validate manifest invariants.
    pub fn validate(&self) -> Result<(), ManifestError> {
        let mut capability_ids: HashSet<String> = HashSet::new();
        for capability in &self.capabilities {
            if !capability_ids.insert(capability.id.clone()) {
                return Err(ManifestError::Validation(format!(
                    "duplicate capability id {}",
                    capability.id
                )));
            }
        }

        let mut runtime_ids: HashSet<String> = HashSet::new();
        for runtime in &self.runtimes {
            if !runtime_ids.insert(runtime.name.clone()) {
                return Err(ManifestError::Validation(format!(
                    "duplicate runtime {}",
                    runtime.name
                )));
            }
        }

        for capability in &self.capabilities {
            for dependency in &capability.depends_on {
                if !capability_ids.contains(dependency) {
                    return Err(ManifestError::Validation(format!(
                        "capability {} depends on unknown capability {}",
                        capability.id, dependency
                    )));
                }
            }
        }

        for runtime in &self.runtimes {
            for dependency in &runtime.depends_on {
                if !runtime_ids.contains(dependency) {
                    return Err(ManifestError::Validation(format!(
                        "runtime {} depends on unknown runtime {}",
                        runtime.name, dependency
                    )));
                }
            }
        }

        let mut scope_ids: HashSet<String> = HashSet::new();
        for policy in &self.token_policies {
            if !scope_ids.insert(policy.scope.clone()) {
                return Err(ManifestError::Validation(format!(
                    "duplicate token scope {}",
                    policy.scope
                )));
            }
            if policy.ttl_seconds == 0 {
                return Err(ManifestError::Validation(format!(
                    "token scope {} must specify a non-zero ttl",
                    policy.scope
                )));
            }
            for capability in &policy.capabilities {
                if !capability_ids.contains(capability) {
                    return Err(ManifestError::Validation(format!(
                        "token scope {} references unknown capability {}",
                        policy.scope, capability
                    )));
                }
            }
        }

        Ok(())
    }

    /// Retrieve a capability manifest entry.
    pub fn capability(&self, id: &str) -> Option<&CapabilityManifestEntry> {
        self.capabilities.iter().find(|cap| cap.id == id)
    }

    /// Retrieve a runtime manifest entry.
    pub fn runtime(&self, name: &str) -> Option<&RuntimeManifestEntry> {
        self.runtimes.iter().find(|runtime| runtime.name == name)
    }

    /// Retrieve a token policy entry.
    pub fn token_policy(&self, scope: &str) -> Option<&TokenPolicyManifestEntry> {
        self.token_policies
            .iter()
            .find(|policy| policy.scope == scope)
    }

    /// List all token policies defined in the manifest.
    pub fn token_policies(&self) -> &[TokenPolicyManifestEntry] {
        &self.token_policies
    }
}

/// Manifest entry describing a capability provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityManifestEntry {
    pub id: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default = "default_autostart")]
    pub autostart: bool,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CapabilityManifestEntry {
    /// Create a new capability manifest entry with defaults.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            provider: None,
            version: None,
            depends_on: Vec::new(),
            autostart: default_autostart(),
            metadata: HashMap::new(),
        }
    }
}

/// Manifest entry describing a token scope policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPolicyManifestEntry {
    pub scope: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_policy_ttl")]
    pub ttl_seconds: u64,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

impl TokenPolicyManifestEntry {
    /// Create a new token policy manifest entry with defaults.
    pub fn new(scope: impl Into<String>) -> Self {
        Self {
            scope: scope.into(),
            description: None,
            ttl_seconds: default_policy_ttl(),
            capabilities: Vec::new(),
        }
    }
}

/// Runtime entry describing a language runtime plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeManifestEntry {
    pub name: String,
    pub kind: RuntimeKind,
    pub version: String,
    pub entrypoint: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub assets: Vec<String>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl RuntimeManifestEntry {
    /// Construct a runtime manifest entry with minimal metadata.
    pub fn new(
        name: impl Into<String>,
        kind: RuntimeKind,
        version: impl Into<String>,
        entrypoint: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            version: version.into(),
            entrypoint: entrypoint.into(),
            depends_on: Vec::new(),
            assets: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

/// Runtime types supported by the kernel bootstrapper.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    Rust,
    Python,
    Go,
    DotNet,
}

/// Errors raised when handling kernel manifests.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("failed to read manifest: {0}")]
    Io(std::io::Error),
    #[error("failed to parse manifest: {0}")]
    Parse(serde_yaml::Error),
    #[error("manifest validation error: {0}")]
    Validation(String),
}
