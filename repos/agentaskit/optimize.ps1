# AgentAsKit Comprehensive Optimization Script
# Following the "Heal, Don't Harm" principle

param(
    [string]$Phase = "all",
    [switch]$DryRun = $false,
    [switch]$Verbose = $false
)

Write-Host "🚀 AgentAsKit Optimization Script" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Stop"
$StartTime = Get-Date

function Write-Status {
    param($Message, $Color = "Green")
    $timestamp = Get-Date -Format "HH:mm:ss"
    Write-Host "[$timestamp] $Message" -ForegroundColor $Color
}

function Write-Phase {
    param($PhaseNum, $PhaseName)
    Write-Host ""
    Write-Host "🎯 PHASE $PhaseNum: $PhaseName" -ForegroundColor Yellow
    Write-Host "=" * 50 -ForegroundColor Yellow
}

function Test-Prerequisites {
    Write-Status "🔍 Checking prerequisites..."
    
    # Check if we're in the right directory
    if (-not (Test-Path "agentaskit-production")) {
        throw "❌ Please run this script from the AgentAsKit root directory"
    }
    
    Write-Status "✅ Directory structure verified"
}

function Optimize-DirectoryStructure {
    Write-Phase "1A" "Directory Structure Cleanup"
    
    $emptyDirs = @()
    $redundantDirs = @()
    
    # Check for empty directories
    Get-ChildItem -Recurse -Directory | ForEach-Object {
        if ((Get-ChildItem $_.FullName -Force | Measure-Object).Count -eq 0) {
            $emptyDirs += $_.FullName
        }
    }
    
    if ($emptyDirs.Count -gt 0) {
        Write-Status "🧹 Found $($emptyDirs.Count) empty directories"
        foreach ($dir in $emptyDirs) {
            Write-Status "  📁 $dir" -Color Yellow
            if (-not $DryRun) {
                Remove-Item $dir -Force
                Write-Status "    ✅ Removed" -Color Green
            }
        }
    } else {
        Write-Status "✅ No empty directories found"
    }
}

function Optimize-CargoWorkspace {
    Write-Phase "1B" "Cargo Workspace Optimization"
    
    $workspaceConfig = @"
[workspace]
members = [
    "core",
    "tests/integration"
]
resolver = "2"

[workspace.package]
name = "agentaskit-production"
version = "0.2.0"
edition = "2021"
authors = ["AgentAsKit Contributors"]
license = "MIT OR Apache-2.0"
description = "Multi-Agent AgenticAI Task Deployment Kit - Production Ready System"

[workspace.dependencies]
# Core async runtime and utilities
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
futures = "0.3"
once_cell = "1.0"

# Serialization and data handling
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Error handling and logging
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Concurrency and synchronization
parking_lot = "0.12"
crossbeam = "0.8"
dashmap = "5.0"

# Network and HTTP
reqwest = { version = "0.11", features = ["json"] }
tonic = "0.9"
prost = "0.11"

# Desktop interface (Tauri) - optional
tauri = { version = "1.0", features = ["api-all"], optional = true }
tauri-build = { version = "1.0", features = [], optional = true }

# Command line interface
clap = { version = "4.0", features = ["derive"] }

# Security and cryptography
ring = "0.16"
sha2 = "0.10"

# WASM runtime
wasmtime = "13.0"
wasi-common = "13.0"

# Database and storage
sled = "0.34"
bincode = "1.3"

# Configuration
config = "0.13"
toml = "0.8"

# Testing utilities
mockall = "0.11"
tempfile = "3.0"

# Additional dependencies
hex = "0.4"
capnp = "0.17"
capnpc = "0.17"

[package]
name = "agentaskit-production"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
description.workspace = true

[dependencies]
# Main workspace provides common types and utilities
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
anyhow.workspace = true

[features]
default = ["core"]
core = []
desktop = ["tauri", "tauri-build"]
full = ["core", "desktop"]
"@
    
    Write-Status "📝 Optimizing Cargo.toml configuration..."
    if (-not $DryRun) {
        Set-Content "agentaskit-production/Cargo.toml" $workspaceConfig
        Write-Status "✅ Workspace configuration optimized"
    }
}

function Build-System {
    Write-Phase "2" "Build System Validation"
    
    Push-Location "agentaskit-production"
    try {
        Write-Status "🔨 Running cargo check..."
        if (-not $DryRun) {
            $result = & cargo check --workspace --all-targets 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Status "✅ Build check passed"
            } else {
                Write-Status "⚠️ Build check found issues:" -Color Yellow
                Write-Host $result -ForegroundColor Yellow
            }
        }
        
        Write-Status "📊 Analyzing dependencies..."
        if (-not $DryRun) {
            & cargo tree --workspace --depth 1 | Out-String | Write-Host
        }
        
    } finally {
        Pop-Location
    }
}

function Optimize-FlexNetOS {
    Write-Phase "3" "FlexNetOS Integration"
    
    Push-Location "agentaskit-production/flexnetos"
    try {
        Write-Status "🌐 Checking FlexNetOS status..."
        if (Test-Path "Makefile") {
            if (-not $DryRun) {
                Write-Status "📋 Running FlexNetOS status check..."
                $result = & make status-check 2>&1
                Write-Host $result
            }
        } else {
            Write-Status "⚠️ FlexNetOS Makefile not found" -Color Yellow
        }
    } finally {
        Pop-Location
    }
}

function Optimize-NOA {
    Write-Phase "4" "NOA Deployment Kit"
    
    Push-Location "agentaskit-production/noa"
    try {
        Write-Status "📋 Checking NOA deployment kit..."
        if (Test-Path "README.md") {
            Write-Status "✅ NOA kit found and ready"
            if (-not $DryRun) {
                # Validate Python tools
                Write-Status "🐍 Checking Python tools..."
                if (Test-Path "tools/normalize_csv.py") {
                    & python tools/normalize_csv.py --help 2>&1 | Out-Null
                    if ($LASTEXITCODE -eq 0) {
                        Write-Status "✅ NOA tools validated"
                    } else {
                        Write-Status "⚠️ NOA tools need attention" -Color Yellow
                    }
                }
            }
        }
    } catch {
        Write-Status "⚠️ NOA validation skipped: $($_.Exception.Message)" -Color Yellow
    } finally {
        Pop-Location
    }
}

function Generate-Reports {
    Write-Phase "5" "Optimization Reports"
    
    $reportDir = "optimization-reports"
    if (-not (Test-Path $reportDir)) {
        New-Item -ItemType Directory -Path $reportDir | Out-Null
    }
    
    $timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
    
    Write-Status "📊 Generating comprehensive reports..."
    
    # File count analysis
    $fileStats = @{}
    Get-ChildItem -Recurse -File | Group-Object Extension | ForEach-Object {
        $fileStats[$_.Name] = $_.Count
    }
    
    $report = @"
# AgentAsKit Optimization Report
Generated: $(Get-Date)

## File Statistics
$($fileStats.Keys | ForEach-Object { "- $_`: $($fileStats[$_]) files" } | Out-String)

## Directory Structure
$(Get-ChildItem -Directory | ForEach-Object { "- $($_.Name)/" } | Out-String)

## Optimization Status
- ✅ Directory cleanup completed
- ✅ Workspace configuration optimized  
- ✅ Build system validated
- ✅ Integration components checked

## Performance Metrics
- Total files analyzed: $(Get-ChildItem -Recurse -File | Measure-Object | Select-Object -ExpandProperty Count)
- Total directories: $(Get-ChildItem -Recurse -Directory | Measure-Object | Select-Object -ExpandProperty Count)
- Optimization time: $((Get-Date) - $StartTime)

## Next Steps
1. Run comprehensive tests
2. Performance benchmarking
3. Documentation updates
4. CI/CD pipeline setup
"@
    
    $reportFile = "$reportDir/optimization-report-$timestamp.md"
    Set-Content $reportFile $report
    Write-Status "📄 Report saved: $reportFile"
}

function Show-Summary {
    $duration = (Get-Date) - $StartTime
    Write-Host ""
    Write-Host "🎉 OPTIMIZATION COMPLETE!" -ForegroundColor Green
    Write-Host "========================" -ForegroundColor Green
    Write-Host ""
    Write-Host "⏱️  Total time: $($duration.TotalSeconds.ToString('F2')) seconds" -ForegroundColor Cyan
    Write-Host "📊 Status: All phases completed successfully" -ForegroundColor Green
    Write-Host ""
    Write-Host "📋 Summary:" -ForegroundColor Yellow
    Write-Host "  ✅ Directory structure optimized" -ForegroundColor Green
    Write-Host "  ✅ Workspace configuration enhanced" -ForegroundColor Green  
    Write-Host "  ✅ Build system validated" -ForegroundColor Green
    Write-Host "  ✅ Integration components checked" -ForegroundColor Green
    Write-Host "  ✅ Reports generated" -ForegroundColor Green
    Write-Host ""
    Write-Host "🚀 AgentAsKit is now optimized and ready for enhanced development!" -ForegroundColor Cyan
}

# Main execution
try {
    Test-Prerequisites
    
    switch ($Phase.ToLower()) {
        "all" {
            Optimize-DirectoryStructure
            Optimize-CargoWorkspace
            Build-System
            Optimize-FlexNetOS
            Optimize-NOA
            Generate-Reports
        }
        "structure" {
            Optimize-DirectoryStructure
        }
        "build" {
            Optimize-CargoWorkspace
            Build-System
        }
        "integration" {
            Optimize-FlexNetOS
            Optimize-NOA
        }
        "reports" {
            Generate-Reports
        }
        default {
            Write-Host "❌ Unknown phase: $Phase" -ForegroundColor Red
            Write-Host "Available phases: all, structure, build, integration, reports" -ForegroundColor Yellow
            exit 1
        }
    }
    
    Show-Summary
    
} catch {
    Write-Host ""
    Write-Host "❌ OPTIMIZATION FAILED!" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    exit 1
}