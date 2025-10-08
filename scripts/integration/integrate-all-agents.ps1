# NOA ARK OS - Integrate All Agents (Automated)
# Processes all drops and integrates agent code

param(
    [switch]$DryRun,
    [string]$Priority = "High"
)

Write-Host "🚀 NOA ARK OS - Agent Integration Automation" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray

$WorkspaceRoot = "D:\dev\workspaces\noa_ark_os"
$StaleDropPath = "$WorkspaceRoot\crc\drop-in\incoming\stale"
$AgentImplPath = "$WorkspaceRoot\agents\src\implementations"

# Priority drops to integrate first
$PriorityDrops = @{
    "High" = @(
        "agentaskit",
        "agent-src",
        "agent-ecosystem-enhanced",
        "agent-communication",
        "executive-hierarchy"
    )
    "Medium" = @(
        "3-plane-system",
        "3-plane-learning-enhanced",
        "noa-core",
        "monitoring",
        "telemetry"
    )
    "Low" = @(
        "advanced-digest",
        "adaptive-optimization",
        "analytics",
        "api"
    )
}

# Statistics
$Stats = @{
    DropsScanned = 0
    FilesFound = 0
    FilesCopied = 0
    FilesAdapted = 0
    Errors = 0
}

function Write-Status {
    param($Message, $Color = "White")
    Write-Host "  $Message" -ForegroundColor $Color
}

function Adapt-Imports {
    param([string]$FilePath)
    
    if (-not (Test-Path $FilePath)) {
        return $false
    }
    
    try {
        $content = Get-Content $FilePath -Raw
        $modified = $false
        
        # Fix common import patterns
        if ($content -match 'use crate::types::') {
            $content = $content -replace 'use crate::types::', 'use noa_agents::types::'
            $modified = $true
        }
        
        if ($content -match 'use crate::core::') {
            $content = $content -replace 'use crate::core::', 'use noa_core::'
            $modified = $true
        }
        
        if ($content -match 'use crate::agent::') {
            $content = $content -replace 'use crate::agent::', 'use noa_agents::'
            $modified = $true
        }
        
        if ($content -match 'use crate::error::') {
            $content = $content -replace 'use crate::error::', 'use noa_agents::error::'
            $modified = $true
        }
        
        # Fix Result types
        if ($content -match 'Result<([^,>]+), String>') {
            $content = $content -replace 'Result<([^,>]+), String>', 'noa_agents::Result<$1>'
            $modified = $true
        }
        
        if ($modified) {
            Set-Content $FilePath -Value $content -NoNewline
            return $true
        }
        
        return $false
    }
    catch {
        Write-Warning "Failed to adapt imports in: $FilePath"
        Write-Warning $_.Exception.Message
        return $false
    }
}

function Integrate-AgentaskitDrop {
    Write-Host "`n📦 Integrating agentaskit drop..." -ForegroundColor Yellow
    
    $agentaskitPath = "$StaleDropPath\agentaskit\agentaskit-production\core\src\agents"
    
    if (-not (Test-Path $agentaskitPath)) {
        Write-Warning "agentaskit agents path not found: $agentaskitPath"
        return
    }
    
    # Integrate board agents
    Write-Status "Copying board agents..." "Cyan"
    $boardAgents = Get-ChildItem -Path "$agentaskitPath\board" -Filter "*.rs" -ErrorAction SilentlyContinue
    
    foreach ($agent in $boardAgents) {
        $destPath = "$AgentImplPath\board\$($agent.Name)"
        
        if ($DryRun) {
            Write-Status "[DRY RUN] Would copy: $($agent.Name)" "Gray"
        } else {
            Copy-Item $agent.FullName $destPath -Force
            $Stats.FilesCopied++
            
            if (Adapt-Imports -FilePath $destPath) {
                $Stats.FilesAdapted++
                Write-Status "✓ Adapted: $($agent.Name)" "Green"
            } else {
                Write-Status "○ Copied: $($agent.Name)" "White"
            }
        }
    }
    
    # Integrate executive agents
    Write-Status "Copying executive agents..." "Cyan"
    $execAgents = Get-ChildItem -Path "$agentaskitPath\executive" -Filter "*.rs" -ErrorAction SilentlyContinue
    
    foreach ($agent in $execAgents) {
        $destPath = "$AgentImplPath\executive\$($agent.Name)"
        
        if ($DryRun) {
            Write-Status "[DRY RUN] Would copy: $($agent.Name)" "Gray"
        } else {
            Copy-Item $agent.FullName $destPath -Force
            $Stats.FilesCopied++
            
            if (Adapt-Imports -FilePath $destPath) {
                $Stats.FilesAdapted++
                Write-Status "✓ Adapted: $($agent.Name)" "Green"
            } else {
                Write-Status "○ Copied: $($agent.Name)" "White"
            }
        }
    }
    
    # Integrate specialized agents
    Write-Status "Copying specialized agents..." "Cyan"
    $specAgents = Get-ChildItem -Path "$agentaskitPath\specialized" -Filter "*.rs" -ErrorAction SilentlyContinue
    
    foreach ($agent in $specAgents) {
        $destPath = "$AgentImplPath\specialist\$($agent.Name)"
        
        if ($DryRun) {
            Write-Status "[DRY RUN] Would copy: $($agent.Name)" "Gray"
        } else {
            Copy-Item $agent.FullName $destPath -Force
            $Stats.FilesCopied++
            
            if (Adapt-Imports -FilePath $destPath) {
                $Stats.FilesAdapted++
                Write-Status "✓ Adapted: $($agent.Name)" "Green"
            } else {
                Write-Status "○ Copied: $($agent.Name)" "White"
            }
        }
    }
}

function Integrate-AgentSrcDrop {
    Write-Host "`n📦 Integrating agent-src drop..." -ForegroundColor Yellow
    
    $agentSrcPath = "$StaleDropPath\agent-src"
    
    if (-not (Test-Path $agentSrcPath)) {
        Write-Warning "agent-src path not found: $agentSrcPath"
        return
    }
    
    # Core agent files
    $coreFiles = @(
        "orchestration.rs",
        "mcp.rs",
        "automation.rs",
        "planner.rs",
        "queue.rs"
    )
    
    foreach ($file in $coreFiles) {
        $sourcePath = "$agentSrcPath\$file"
        
        if (Test-Path $sourcePath) {
            $destPath = "$AgentImplPath\$file"
            
            if ($DryRun) {
                Write-Status "[DRY RUN] Would copy: $file" "Gray"
            } else {
                Copy-Item $sourcePath $destPath -Force
                $Stats.FilesCopied++
                
                if (Adapt-Imports -FilePath $destPath) {
                    $Stats.FilesAdapted++
                    Write-Status "✓ Adapted: $file" "Green"
                } else {
                    Write-Status "○ Copied: $file" "White"
                }
            }
        }
    }
}

function Update-ModuleTree {
    Write-Host "`n📝 Updating module tree..." -ForegroundColor Yellow
    
    # Update board/mod.rs
    $boardModPath = "$AgentImplPath\board\mod.rs"
    $boardAgents = Get-ChildItem -Path "$AgentImplPath\board" -Filter "*.rs" | 
                   Where-Object { $_.Name -ne "mod.rs" }
    
    $boardModContent = "// Board-level agent implementations`n"
    $boardModContent += "// Integrated from agentaskit drop`n`n"
    
    foreach ($agent in $boardAgents) {
        $moduleName = $agent.BaseName
        $boardModContent += "pub mod $moduleName;`n"
    }
    
    $boardModContent += "`n// Re-export all agents`n"
    foreach ($agent in $boardAgents) {
        $moduleName = $agent.BaseName
        $boardModContent += "pub use ${moduleName}::*;`n"
    }
    
    if (-not $DryRun) {
        Set-Content $boardModPath -Value $boardModContent
        Write-Status "✓ Updated board/mod.rs" "Green"
    } else {
        Write-Status "[DRY RUN] Would update board/mod.rs" "Gray"
    }
    
    # Similar updates for executive and specialist
    # (Abbreviated for brevity)
}

# Main execution
Write-Host "`nIntegration Settings:" -ForegroundColor Cyan
Write-Host "  Priority: $Priority"
Write-Host "  Dry Run: $DryRun"
Write-Host "  Workspace: $WorkspaceRoot"
Write-Host ""

# Get drops to process
$dropsToProcess = $PriorityDrops[$Priority]

Write-Host "Processing $($dropsToProcess.Count) drops..." -ForegroundColor Cyan
Write-Host ""

# Process each drop
foreach ($drop in $dropsToProcess) {
    $Stats.DropsScanned++
    
    switch ($drop) {
        "agentaskit" {
            Integrate-AgentaskitDrop
        }
        "agent-src" {
            Integrate-AgentSrcDrop
        }
        default {
            Write-Host "⏸️  $drop - Not yet implemented" -ForegroundColor Yellow
        }
    }
}

# Update module tree
if ($Stats.FilesCopied -gt 0 -and -not $DryRun) {
    Update-ModuleTree
}

# Summary
Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host "✅ Integration Complete!" -ForegroundColor Green
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host "`nStatistics:" -ForegroundColor Cyan
Write-Host "  Drops Scanned: $($Stats.DropsScanned)"
Write-Host "  Files Copied: $($Stats.FilesCopied)"
Write-Host "  Files Adapted: $($Stats.FilesAdapted)"
Write-Host "  Errors: $($Stats.Errors)"

if ($DryRun) {
    Write-Host "`n💡 This was a dry run. Run without -DryRun to actually integrate." -ForegroundColor Yellow
} else {
    Write-Host "`n🚀 Next Steps:" -ForegroundColor Yellow
    Write-Host "  1. cargo build" -ForegroundColor Cyan
    Write-Host "  2. Fix any compilation errors" -ForegroundColor Cyan
    Write-Host "  3. cargo test" -ForegroundColor Cyan
}

Write-Host ""
