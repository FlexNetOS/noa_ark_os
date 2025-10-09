# Agent System Priority Fixes
# Fixes critical issues identified in audit

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("All", "Types", "Registry", "Backup", "Integration")]
    [string]$Fix = "All",
    
    [Parameter(Mandatory=$false)]
    [switch]$DryRun = $false
)

$WorkspaceRoot = "D:\dev\workspaces\noa_ark_os"

function Write-Info { param($Message) Write-Host "ℹ️  $Message" -ForegroundColor Cyan }
function Write-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "⚠️  $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor Red }
function Write-Step { param($Message) Write-Host "`n🔷 $Message" -ForegroundColor Blue }

Write-Step "Agent System Priority Fixes"
Write-Info "Workspace: $WorkspaceRoot"
Write-Info "Fix Target: $Fix"
Write-Info ""

# Priority 1: Fix Type Conflicts
if ($Fix -eq "All" -or $Fix -eq "Types") {
    Write-Step "P0: Fixing Type Conflicts"
    
    Write-Info "Issue: AgentMetadata defined in both lib.rs and types.rs"
    Write-Info "Solution: Use types.rs as single source of truth"
    Write-Info ""
    
    Write-Info "Required changes:"
    Write-Info "  1. Remove AgentMetadata from lib.rs"
    Write-Info "  2. Import from types module"
    Write-Info "  3. Update all references"
    Write-Info "  4. Remove duplicate enums"
    Write-Info ""
    
    if ($DryRun) {
        Write-Warning "[DRY RUN] Would update lib.rs"
    } else {
        Write-Info "Manual fix required - types are incompatible"
        Write-Info "Backup AgentMetadata uses different fields than current"
        Write-Info ""
        Write-Info "Options:"
        Write-Info "  A) Use simple lib.rs version (current factory)"
        Write-Info "  B) Use comprehensive types.rs version (backup agents)"
        Write-Info "  C) Merge both into new unified version"
        Write-Info ""
        Write-Warning "Recommended: Option C - Unified version"
    }
}

# Priority 2: Create Unified Types
if ($Fix -eq "All" -or $Fix -eq "Types") {
    Write-Step "Creating Unified Type System"
    
    $unifiedTypes = @'
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Agent layer in NOA hierarchy (5-layer architecture)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentLayer {
    L1Autonomy,      // Root CECCA level
    L2Reasoning,     // Board & Executive
    L3Orchestration, // Chief Commanders
    L4Operations,    // Specialists & Workers
    L5Infrastructure,// Micro & Subject agents
}

/// Agent category/domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentCategory {
    Analysis,
    Code,
    DevOps,
    Testing,
    Documentation,
    Security,
    Governance,
    Operations,
    Research,
    Other,
}

/// Agent runtime state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    Created,
    Initializing,
    Ready,
    Running,
    Paused,
    Terminating,
    Terminated,
}

/// Agent health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    NeedsRepair,
    Error,
    Unknown,
}

/// Agent type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    Master,      // CECCA, Board
    Worker,      // Specialists
    SubAgent,    // Micro agents
    Swarm,       // Coordinated groups
}

/// Agent language/runtime
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentLanguage {
    Rust,
    Python,
    Go,
}

/// Unified agent metadata (combines factory + registry)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    // Runtime identity
    pub id: Uuid,
    pub agent_id: String,
    pub name: String,
    
    // Classification
    pub layer: AgentLayer,
    pub category: AgentCategory,
    pub agent_type: AgentType,
    
    // Description
    pub description: String,
    pub role: String,
    pub purpose: String,
    
    // Status
    pub state: AgentState,
    pub health_status: HealthStatus,
    
    // Relationships
    pub parent_id: Option<String>,
    pub escalation_to: Option<String>,
    pub stack: Option<String>,
    
    // Capabilities
    pub capabilities: Vec<String>,
    pub tools: Vec<String>,
    pub tags: Vec<String>,
    
    // I/O
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub dependencies: Vec<String>,
    
    // Resources
    pub cpu_min: String,
    pub ram_min: String,
    pub disk_min: String,
    
    // Behavior
    pub autonomy_level: String,
    pub disposable: bool,
    
    // Diagnostics
    pub issues_identified: Vec<String>,
    pub repair_recommendations: Vec<String>,
    
    // Metadata
    pub created_at: Option<String>,
    pub last_updated: Option<String>,
    pub version: Option<String>,
}

impl AgentMetadata {
    /// Create minimal agent metadata for factory
    pub fn minimal(name: String, description: String, category: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id: name.clone(),
            name: name.clone(),
            layer: AgentLayer::L4Operations,
            category: AgentCategory::Other,
            agent_type: AgentType::Worker,
            description,
            role: String::new(),
            purpose: String::new(),
            state: AgentState::Created,
            health_status: HealthStatus::Unknown,
            parent_id: None,
            escalation_to: None,
            stack: None,
            capabilities: Vec::new(),
            tools: Vec::new(),
            tags: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            dependencies: Vec::new(),
            cpu_min: "1".to_string(),
            ram_min: "1GB".to_string(),
            disk_min: "500MB".to_string(),
            autonomy_level: "managed".to_string(),
            disposable: false,
            issues_identified: Vec::new(),
            repair_recommendations: Vec::new(),
            created_at: None,
            last_updated: None,
            version: Some("0.1.0".to_string()),
        }
    }
    
    /// Create from registry data
    pub fn from_registry(agent_name: String, agent_id: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id,
            name: agent_name,
            layer: AgentLayer::L4Operations,
            category: AgentCategory::Other,
            agent_type: AgentType::Worker,
            description: String::new(),
            role: String::new(),
            purpose: String::new(),
            state: AgentState::Created,
            health_status: HealthStatus::Unknown,
            parent_id: None,
            escalation_to: None,
            stack: None,
            capabilities: Vec::new(),
            tools: Vec::new(),
            tags: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            dependencies: Vec::new(),
            cpu_min: "1".to_string(),
            ram_min: "1GB".to_string(),
            disk_min: "500MB".to_string(),
            autonomy_level: "tbd".to_string(),
            disposable: false,
            issues_identified: Vec::new(),
            repair_recommendations: Vec::new(),
            created_at: None,
            last_updated: None,
            version: None,
        }
    }
    
    pub fn is_healthy(&self) -> bool {
        matches!(self.health_status, HealthStatus::Healthy)
    }
    
    pub fn needs_repair(&self) -> bool {
        matches!(self.health_status, HealthStatus::NeedsRepair | HealthStatus::Error)
    }
    
    pub fn layer_name(&self) -> &str {
        match self.layer {
            AgentLayer::L1Autonomy => "L1Autonomy",
            AgentLayer::L2Reasoning => "L2Reasoning",
            AgentLayer::L3Orchestration => "L3Orchestration",
            AgentLayer::L4Operations => "L4Operations",
            AgentLayer::L5Infrastructure => "L5Infrastructure",
        }
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_agents: usize,
    pub healthy_agents: usize,
    pub needs_repair: usize,
    pub unknown_status: usize,
    pub agents_by_layer: HashMap<String, usize>,
    pub agents_by_category: HashMap<String, usize>,
}

impl RegistryStats {
    pub fn new() -> Self {
        Self {
            total_agents: 0,
            healthy_agents: 0,
            needs_repair: 0,
            unknown_status: 0,
            agents_by_layer: HashMap::new(),
            agents_by_category: HashMap::new(),
        }
    }
}

impl Default for RegistryStats {
    fn default() -> Self {
        Self::new()
    }
}
'@

    $targetFile = "$WorkspaceRoot\agents\src\unified_types.rs"
    
    if ($DryRun) {
        Write-Info "[DRY RUN] Would create: unified_types.rs"
    } else {
        Set-Content -Path $targetFile -Value $unifiedTypes -Encoding UTF8
        Write-Success "Created: unified_types.rs (unified type system)"
    }
}

# Summary
Write-Step "Fix Summary"

if ($DryRun) {
    Write-Warning "DRY RUN completed - no changes made"
} else {
    Write-Info "Changes made:"
    Write-Info "  • Created unified_types.rs"
    Write-Info ""
    Write-Info "Manual steps required:"
    Write-Info "  1. Update lib.rs to use unified_types"
    Write-Info "  2. Update registry.rs to use unified_types"
    Write-Info "  3. Remove duplicate type definitions"
    Write-Info "  4. Test compilation"
    Write-Info ""
    Write-Info "Run this after manual updates:"
    Write-Info "  cargo build -p noa_agents"
    Write-Info "  cargo test -p noa_agents"
}

Write-Success ""
Write-Success "Priority fixes analysis complete!"
