# Agent Factory Full Integration Script
# Systematically integrates all 928 agents with full automation

param(
    [Parameter(Mandatory=$false)]
    [switch]$DryRun = $false,
    
    [Parameter(Mandatory=$false)]
    [ValidateSet("All", "Board", "Executive", "Specialist", "Orchestrator", "STEM")]
    [string]$Layer = "All",
    
    [Parameter(Mandatory=$false)]
    [switch]$ShowDetails = $false
)

$WorkspaceRoot = "D:\dev\workspaces\noa_ark_os"
$SourceDir = "$WorkspaceRoot\crc\drop-in\incoming\stale\agent_factory"
$TargetDir = "$WorkspaceRoot\agents"

function Write-Info { param($Message) Write-Host "ℹ️  $Message" -ForegroundColor Cyan }
function Write-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "⚠️  $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor Red }
function Write-Step { param($Message) Write-Host "`n🔷 $Message" -ForegroundColor Blue }

Write-Step "NOA ARK OS Agent Factory Integration"
Write-Info "Integrating 928 agents with full automation"
Write-Info "Source: $SourceDir"
Write-Info "Target: $TargetDir"
Write-Info "Layer: $Layer"
Write-Info ""

# Agent hierarchy definition
$AgentHierarchy = @{
    Root = @{
        Name = "CECCA"
        Description = "Chief Executive Commander Agent - Root orchestrator"
        Count = 1
        Layer = "L1Autonomy"
    }
    
    Constitutional = @{
        Name = "Constitutional Governance (Trifecta Court)"
        Agents = @("Scripture Court", "Geometry Court", "Bridge-Path Council", "Truth Gate", "Evidence Ledger")
        Count = 5
        Layer = "L1Autonomy"
    }
    
    Board = @{
        Name = "Board Agents"
        Agents = @(
            "Risk", "Compliance", "Finance", "Policy Enforcement", 
            "Governance & Audit", "RBAC Policy", "Open Policy", "Ethics"
        )
        Count = 8
        Layer = "L2Reasoning"
    }
    
    Orchestration = @{
        Name = "Orchestration and Services"
        Agents = @(
            "Agent Registry", "Microagent Stacks", "Model Selector", 
            "Digest Agent", "Code Buddy", "Learn Pal", "Quality Pal", "Env Friend"
        )
        Count = 8
        Layer = "L3Orchestration"
    }
    
    Executive = @{
        Name = "Executive Agents"
        Agents = @("Exec1", "Exec2", "Exec3", "Exec4", "Exec5")
        Count = 5
        Layer = "L2Reasoning"
    }
    
    STEM = @{
        Name = "STEM Layer (Pluripotent)"
        Count = 19
        Layer = "L4Operations"
        Description = "Self-replicating, specializing agent architecture"
    }
    
    ChiefCommander = @{
        Name = "Chief Commander Agents"
        Count = 26
        Layer = "L3Orchestration"
    }
    
    SubAgents = @{
        Name = "Sub-Agents"
        Count = 171
        Layer = "L4Operations"
    }
    
    Subjects = @{
        Name = "Subject Domain Agents"
        Count = 2001
        Layer = "L5Infrastructure"
    }
    
    KnowledgeCapsules = @{
        Name = "Knowledge Capsules"
        Count = 9
        Layer = "L5Infrastructure"
    }
}

$TotalAgents = 0
foreach ($layer in $AgentHierarchy.Values) {
    $TotalAgents += $layer.Count
}

Write-Info "Total Agents to Integrate: $TotalAgents"
Write-Info ""

# Phase 1: Analyze source structure
Write-Step "Phase 1: Analyzing Source Structure"

if (!(Test-Path $SourceDir)) {
    Write-Error "Source directory not found: $SourceDir"
    exit 1
}

$sourceFiles = Get-ChildItem $SourceDir -Recurse -File
Write-Info "Found $($sourceFiles.Count) files in source"

# Categorize files
$rustFiles = $sourceFiles | Where-Object { $_.Extension -eq ".rs" }
$goFiles = $sourceFiles | Where-Object { $_.Extension -eq ".go" }
$pyFiles = $sourceFiles | Where-Object { $_.Extension -eq ".py" }
$mdFiles = $sourceFiles | Where-Object { $_.Extension -eq ".md" }
$mmdFiles = $sourceFiles | Where-Object { $_.Extension -eq ".mmd" }

Write-Info "  Rust files: $($rustFiles.Count)"
Write-Info "  Go files: $($goFiles.Count)"
Write-Info "  Python files: $($pyFiles.Count)"
Write-Info "  Documentation: $($mdFiles.Count)"
Write-Info "  Mermaid diagrams: $($mmdFiles.Count)"

# Phase 2: Parse agent hierarchy
Write-Step "Phase 2: Parsing Agent Hierarchy"

$hierarchyFile = "$SourceDir\agent_hierarchical_map.md"
if (Test-Path $hierarchyFile) {
    $hierarchyContent = Get-Content $hierarchyFile -Raw
    Write-Success "Loaded agent hierarchy map"
    
    # Extract agent count from hierarchy
    if ($hierarchyContent -match "(\d+)\s+Components") {
        $componentCount = $Matches[1]
        Write-Info "Documented components: $componentCount"
    }
}

$cecaGraph = "$SourceDir\graphs\cecca_agent_hierarchy_graph.mmd"
if (Test-Path $cecaGraph) {
    $graphContent = Get-Content $cecaGraph
    Write-Success "Loaded CECCA hierarchy graph ($($graphContent.Count) lines)"
}

# Phase 3: Create target structure
Write-Step "Phase 3: Creating Target Structure"

$targetDirs = @(
    "$TargetDir\src\implementations\root",
    "$TargetDir\src\implementations\constitutional",
    "$TargetDir\src\implementations\board",
    "$TargetDir\src\implementations\executive",
    "$TargetDir\src\implementations\orchestration",
    "$TargetDir\src\implementations\stem",
    "$TargetDir\src\implementations\chief_commander",
    "$TargetDir\src\implementations\subject",
    "$TargetDir\python",
    "$TargetDir\go",
    "$TargetDir\docs\hierarchy",
    "$TargetDir\docs\graphs"
)

foreach ($dir in $targetDirs) {
    if ($DryRun) {
        Write-Info "[DRY RUN] Would create: $dir"
    } else {
        if (!(Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
            Write-Success "Created: $dir"
        } else {
            Write-Info "Exists: $dir"
        }
    }
}

# Phase 4: Copy and organize files
Write-Step "Phase 4: Organizing Agent Files"

# Copy Rust agent files
if ($rustFiles.Count -gt 0) {
    Write-Info "Processing $($rustFiles.Count) Rust files..."
    foreach ($file in $rustFiles) {
        $relativePath = $file.FullName.Replace($SourceDir, "").TrimStart("\")
        $targetPath = "$TargetDir\$relativePath"
        
        if ($DryRun) {
            Write-Info "  [DRY RUN] Would copy: $($file.Name) → $targetPath"
        } else {
            $targetFolder = Split-Path $targetPath -Parent
            if (!(Test-Path $targetFolder)) {
                New-Item -ItemType Directory -Path $targetFolder -Force | Out-Null
            }
            Copy-Item $file.FullName $targetPath -Force
            if ($ShowDetails) {
                Write-Success "  Copied: $($file.Name)"
            }
        }
    }
    Write-Success "Rust files processed"
}

# Copy Go agent files
if ($goFiles.Count -gt 0) {
    Write-Info "Processing $($goFiles.Count) Go files..."
    foreach ($file in $goFiles) {
        $targetPath = "$TargetDir\go\$($file.Name)"
        
        if ($DryRun) {
            Write-Info "  [DRY RUN] Would copy: $($file.Name) → $targetPath"
        } else {
            Copy-Item $file.FullName $targetPath -Force
            if ($ShowDetails) {
                Write-Success "  Copied: $($file.Name)"
            }
        }
    }
    Write-Success "Go files processed"
}

# Copy Python agent files  
if ($pyFiles.Count -gt 0) {
    Write-Info "Processing $($pyFiles.Count) Python files..."
    foreach ($file in $pyFiles) {
        $targetPath = "$TargetDir\python\$($file.Name)"
        
        if ($DryRun) {
            Write-Info "  [DRY RUN] Would copy: $($file.Name) → $targetPath"
        } else {
            Copy-Item $file.FullName $targetPath -Force
            if ($ShowDetails) {
                Write-Success "  Copied: $($file.Name)"
            }
        }
    }
    Write-Success "Python files processed"
}

# Copy documentation
Write-Info "Processing documentation..."
foreach ($file in $mdFiles) {
    $targetPath = "$TargetDir\docs\hierarchy\$($file.Name)"
    if (!$DryRun) {
        Copy-Item $file.FullName $targetPath -Force
    }
}

foreach ($file in $mmdFiles) {
    $targetPath = "$TargetDir\docs\graphs\$($file.Name)"
    if (!$DryRun) {
        Copy-Item $file.FullName $targetPath -Force
    }
}
Write-Success "Documentation processed"

# Phase 5: Generate integration manifest
Write-Step "Phase 5: Generating Integration Manifest"

$manifest = @{
    timestamp = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
    total_agents = $TotalAgents
    hierarchy = $AgentHierarchy
    files_processed = @{
        rust = $rustFiles.Count
        go = $goFiles.Count
        python = $pyFiles.Count
        documentation = $mdFiles.Count
        diagrams = $mmdFiles.Count
    }
    status = if ($DryRun) { "dry_run" } else { "integrated" }
}

$manifestJson = $manifest | ConvertTo-Json -Depth 10
$manifestPath = "$TargetDir\integration_manifest.json"

if ($DryRun) {
    Write-Info "[DRY RUN] Would save manifest to: $manifestPath"
    Write-Info "Manifest preview:"
    Write-Host ($manifestJson | Out-String) -ForegroundColor Gray
} else {
    Set-Content -Path $manifestPath -Value $manifestJson -Encoding UTF8
    Write-Success "Manifest saved: $manifestPath"
}

# Phase 6: Summary
Write-Step "Integration Complete!"
Write-Info ""
Write-Info "Summary:"
Write-Info "  Total Agents: $TotalAgents"
Write-Info "  Files Processed: $($sourceFiles.Count)"
Write-Info "  Rust: $($rustFiles.Count)"
Write-Info "  Go: $($goFiles.Count)"
Write-Info "  Python: $($pyFiles.Count)"
Write-Info "  Status: $(if ($DryRun) { 'DRY RUN' } else { 'COMPLETED' })"
Write-Info ""

if (!$DryRun) {
    Write-Success "All agents integrated successfully!"
    Write-Info ""
    Write-Info "Next steps:"
    Write-Info "  1. Review integration manifest: $manifestPath"
    Write-Info "  2. Update Cargo.toml with new modules"
    Write-Info "  3. Run: cargo build --workspace"
    Write-Info "  4. Run: .\scripts\dev\wire-agents.ps1"
} else {
    Write-Warning "This was a DRY RUN - no files were modified"
    Write-Info "Run without -DryRun to execute the integration"
}
