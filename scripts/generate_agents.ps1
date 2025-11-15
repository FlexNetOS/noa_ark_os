# Agent Auto-Generator (PowerShell)
# Automatically generates Rust agent implementations from the 928-agent registry

param(
    [int]$BatchSize = 50,
    [switch]$DryRun
)

# Existing agents (already implemented)
$EXISTING_AGENTS = @(
    "noa-commander",
    "digest-agent", "finance-agent", "legal-agent", "operations-agent", "strategy-agent",
    "emergency-responder", "priority-manager", "resource-allocator", "system-orchestrator",
    "code-generation-agent", "data-analytics-agent", "deployment-agent", "integration-agent",
    "learning-agent", "monitoring-agent", "security-agent", "testing-agent",
    "model-selector-agent"
)

# Layer mapping
$LAYER_MAP = @{
    "L1Autonomy" = "autonomy"
    "L2Reasoning" = "reasoning"
    "L3Orchestration" = "orchestration"
    "L4Operations" = "operations"
    "L5Infrastructure" = "infrastructure"
}

function Sanitize-Name {
    param([string]$Name)
    
    $result = $Name.ToLower()
    $result = $result -replace '[^a-z0-9_]', '_'
    $result = $result -replace '_+', '_'
    $result = $result.Trim('_')
    return $result
}

function ConvertTo-CamelCase {
    param([string]$Name)
    
    $parts = $Name.Split('_') | Where-Object { $_ }
    $result = ($parts | ForEach-Object { 
        $_.Substring(0,1).ToUpper() + $_.Substring(1) 
    }) -join ''
    return $result
}

function Generate-AgentStruct {
    param(
        [hashtable]$Agent
    )
    
    $structName = ConvertTo-CamelCase (Sanitize-Name $Agent.name)
    $agentId = Sanitize-Name $Agent.name
    $purpose = $Agent.purpose -replace '"', '\"'
    $name = $Agent.name -replace '"', '\"'
    $role = $Agent.role -replace '"', '\"'
    
    return @"
//! $name - Auto-generated
//! 
//! $purpose

use crate::unified_types::*;
use crate::Result;
use tokio::sync::RwLock;
use uuid::Uuid;

/// $name
pub struct $structName {
    metadata: AgentMetadata,
    state: RwLock<AgentState>,
}

impl $structName {
    pub fn new() -> Self {
        let metadata = AgentMetadata {
            id: Uuid::new_v4(),
            agent_id: "$agentId".to_string(),
            name: "$name".to_string(),
            layer: AgentLayer::$($Agent.layer),
            category: AgentCategory::Other,
            agent_type: AgentType::Worker,
            language: AgentLanguage::Rust,
            description: "$purpose".to_string(),
            role: "$role".to_string(),
            purpose: "$purpose".to_string(),
            state: AgentState::Created,
            health_status: HealthStatus::Unknown,
            parent_id: None,
            escalation_to: None,
            stack: None,
            capabilities: vec![],
            tools: vec![],
            tags: vec![],
            inputs: vec![],
            outputs: vec![],
            dependencies: vec![],
            cpu_min: "0.5".to_string(),
            ram_min: "256MB".to_string(),
            disk_min: "100MB".to_string(),
            autonomy_level: "autonomous".to_string(),
            disposable: false,
            issues_identified: vec![],
            repair_recommendations: vec![],
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            last_updated: Some(chrono::Utc::now().to_rfc3339()),
            version: Some("1.0.0".to_string()),
        };
        
        Self {
            metadata,
            state: RwLock::new(AgentState::Created),
        }
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        *self.state.write().await = AgentState::Ready;
        Ok(())
    }
    
    pub fn metadata(&self) -> &AgentMetadata {
        &self.metadata
    }
    
    pub async fn state(&self) -> AgentState {
        self.state.read().await.clone()
    }
}

impl Default for $structName {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_agent_creation() {
        let agent = ${structName}::new();
        assert_eq!(agent.metadata().name, "$name");
    }
    
    #[tokio::test]
    async fn test_agent_initialization() {
        let mut agent = ${structName}::new();
        agent.initialize().await.unwrap();
        assert_eq!(agent.state().await, AgentState::Ready);
    }
}
"@
}

Write-Host "🚀 NOA ARK OS - Agent Auto-Generator" -ForegroundColor Cyan
Write-Host ("=" * 50) -ForegroundColor Cyan
Write-Host ""

# Load registry
$registryPath = "agents\data\agent_directory.csv"
if (-not (Test-Path $registryPath)) {
    Write-Host "❌ Registry not found: $registryPath" -ForegroundColor Red
    exit 1
}

Write-Host "🔍 Loading registry from: $registryPath" -ForegroundColor Yellow
$csv = Import-Csv $registryPath
Write-Host "✓ Loaded $($csv.Count) agents from registry" -ForegroundColor Green

# Filter and categorize agents
$newAgents = @()
$skipped = 0
$seenNames = @{}  # Track seen names to avoid duplicates

foreach ($row in $csv) {
    $agentName = $row.agent_name.Trim()
    if (-not $agentName) { continue }
    
    $sanitizedName = Sanitize-Name $agentName
    
    # Skip existing agents
    if ($EXISTING_AGENTS -contains $sanitizedName) {
        $skipped++
        continue
    }
    
    # Skip duplicates
    if ($seenNames.ContainsKey($sanitizedName)) {
        $skipped++
        continue
    }
    
    $seenNames[$sanitizedName] = $true
    
    # Determine layer
    $layerStr = $row.layer
    $layer = "L4Operations"
    
    if ($layerStr -match "board|l2") {
        $layer = "L2Reasoning"
    } elseif ($layerStr -match "executive|l1") {
        $layer = "L1Autonomy"
    } elseif ($layerStr -match "stack|l3") {
        $layer = "L3Orchestration"
    } elseif ($layerStr -match "micro|l5") {
        $layer = "L5Infrastructure"
    }
    
    $newAgents += @{
        name = $agentName
        role = $row.role
        layer = $layer
        purpose = $row.purpose
        sanitized = $sanitizedName
    }
}

Write-Host "✓ Found $($newAgents.Count) new agents to generate" -ForegroundColor Green
Write-Host "✓ Skipping $skipped existing agents" -ForegroundColor Green

# Group by layer
$byLayer = $newAgents | Group-Object layer

Write-Host "`n📊 Agent distribution:" -ForegroundColor Cyan
foreach ($group in $byLayer) {
    Write-Host "   $($group.Name): $($group.Count) agents" -ForegroundColor White
}

if ($DryRun) {
    Write-Host "`n⚠️  Dry run - no files will be generated" -ForegroundColor Yellow
    exit 0
}

# Generate agents
$outputDir = "agents\src\implementations\generated"
$totalGenerated = 0

foreach ($group in $byLayer) {
    $layer = $group.Name
    $agents = $group.Group
    
    $layerDir = $LAYER_MAP[$layer]
    if (-not $layerDir) { $layerDir = $layer.ToLower() }
    
    $layerPath = Join-Path $outputDir $layerDir
    New-Item -ItemType Directory -Path $layerPath -Force | Out-Null
    
    Write-Host "`n🔧 Generating $layer agents..." -ForegroundColor Yellow
    
    # Process in batches
    for ($i = 0; $i -lt $agents.Count; $i += $BatchSize) {
        $batch = $agents[$i..[Math]::Min($i + $BatchSize - 1, $agents.Count - 1)]
        $batchNum = [Math]::Floor($i / $BatchSize) + 1
        
        Write-Host "   Batch $batchNum`: $($batch.Count) agents" -NoNewline
        
        foreach ($agent in $batch) {
            $fileName = "$($agent.sanitized).rs"
            $filePath = Join-Path $layerPath $fileName
            
            # Generate code
            $code = Generate-AgentStruct $agent
            
            # Write file
            [System.IO.File]::WriteAllText($filePath, $code)
            
            $totalGenerated++
        }
        
        Write-Host " ✓" -ForegroundColor Green
    }
    
    # Generate mod.rs
    $modContent = "//! $layer Agents - Auto-generated`n`n"
    
    foreach ($agent in $agents) {
        $moduleName = $agent.sanitized
        $modContent += "pub mod $moduleName;`n"
    }
    
    $modContent += "`n// Re-exports`n"
    foreach ($agent in $agents) {
        $moduleName = $agent.sanitized
        $structName = ConvertTo-CamelCase $moduleName
        $modContent += "pub use ${moduleName}::${structName};`n"
    }
    
    $modPath = Join-Path $layerPath "mod.rs"
    [System.IO.File]::WriteAllText($modPath, $modContent)
}

Write-Host "`n✅ Generation complete!" -ForegroundColor Green
Write-Host "   Total agents generated: $totalGenerated" -ForegroundColor White
Write-Host "   Output directory: $outputDir" -ForegroundColor White

Write-Host "`n📋 Next steps:" -ForegroundColor Cyan
Write-Host "   1. Review generated code in: $outputDir" -ForegroundColor White
Write-Host "   2. Run: cargo build -p noa_agents" -ForegroundColor White
Write-Host "   3. Run: cargo test -p noa_agents" -ForegroundColor White
Write-Host "   4. Integrate into main module system" -ForegroundColor White

Write-Host "`n🎉 Successfully generated $totalGenerated agents!" -ForegroundColor Green
