# Fork Repository Integration Plan

**Date**: Current  
**Purpose**: Convert FlexNetOS fork repos to branches in main workspace  
**Status**: Ready to Execute  

---

## 🎯 **Objective**

Convert all fork repositories from FlexNetOS GitHub organization into branches in the main `noa_ark_os` repository using the fork processing system.

---

## 📋 **Step 1: Identify FlexNetOS Fork Repos**

### **Common FlexNetOS Fork Patterns**

Based on typical fork naming:
```
FlexNetOS/
├── noa_ark_os           # Main repo (already connected)
├── noa_ark_os-agent-*   # Agent forks
├── noa_ark_os-crc-*     # CRC forks
├── noa_ark_os-ui-*      # UI forks
└── other-forks          # External library forks
```

### **Discover Available Forks**

```powershell
# Option 1: Via GitHub CLI (if installed)
gh repo list FlexNetOS --fork

# Option 2: Via GitHub API
$org = "FlexNetOS"
$repos = Invoke-RestMethod -Uri "https://api.github.com/orgs/$org/repos?type=forks&per_page=100"
$repos | Select-Object name, html_url, fork

# Option 3: Manual list from GitHub web UI
# Visit: https://github.com/orgs/FlexNetOS/repositories?type=fork
```

---

## 🔧 **Step 2: Create Fork Processing Script**

<function_calls>
<invoke name="create_file">
<parameter name="content"># Process GitHub Forks into Workspace
# Clones fork repos and processes them through the fork system

param(
    [Parameter(Mandatory=$false)]
    [string]$OrgName = "FlexNetOS",
    
    [Parameter(Mandatory=$false)]
    [string[]]$ForkNames = @(),  # Specific forks to process, empty = all
    
    [Parameter(Mandatory=$false)]
    [switch]$ListOnly = $false,  # Just list forks, don't process
    
    [Parameter(Mandatory=$false)]
    [switch]$DryRun = $false
)

$WorkspaceRoot = "D:\dev\workspaces\noa_ark_os"
$ForksDir = Join-Path $WorkspaceRoot "crc\drop-in\incoming\forks"

function Write-Info { param($Message) Write-Host "ℹ️  $Message" -ForegroundColor Cyan }
function Write-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "⚠️  $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor Red }

Write-Info "FlexNetOS Fork Processor"
Write-Info "Organization: $OrgName"

# Step 1: Get list of forks from GitHub
Write-Info "Fetching fork list from GitHub..."

try {
    $apiUrl = "https://api.github.com/orgs/$OrgName/repos?type=all&per_page=100"
    $repos = Invoke-RestMethod -Uri $apiUrl -Method Get
    
    # Filter for forks or repos with fork-like names
    $forks = $repos | Where-Object { 
        $_.fork -eq $true -or 
        $_.name -like "*-fork" -or
        $_.name -like "noa_ark_os-*"
    }
    
    Write-Success "Found $($forks.Count) potential forks"
    
    if ($ListOnly) {
        Write-Info ""
        Write-Info "Available Forks:"
        $forks | ForEach-Object {
            Write-Host "  - $($_.name)" -ForegroundColor Yellow
            Write-Host "    URL: $($_.html_url)" -ForegroundColor Gray
            if ($_.fork) {
                Write-Host "    Source: $($_.parent.full_name)" -ForegroundColor Gray
            }
            Write-Host ""
        }
        exit 0
    }
    
} catch {
    Write-Error "Failed to fetch repos from GitHub: $($_.Exception.Message)"
    Write-Warning "Proceeding with manual fork list if provided..."
    $forks = @()
}

# Step 2: Filter forks to process
if ($ForkNames.Count -gt 0) {
    Write-Info "Processing specific forks: $($ForkNames -join ', ')"
    $forksToProcess = $forks | Where-Object { $ForkNames -contains $_.name }
} else {
    Write-Info "Processing all discovered forks"
    $forksToProcess = $forks
}

if ($forksToProcess.Count -eq 0) {
    Write-Warning "No forks to process"
    exit 0
}

Write-Info "Will process $($forksToProcess.Count) forks"

# Step 3: Process each fork
foreach ($fork in $forksToProcess) {
    Write-Info ""
    Write-Info "==========================================="
    Write-Info "Processing: $($fork.name)"
    Write-Info "==========================================="
    
    $forkDir = Join-Path $ForksDir $fork.name
    
    # Check if already exists
    if (Test-Path $forkDir) {
        Write-Warning "Fork directory already exists: $forkDir"
        Write-Info "Skipping. Delete directory first if you want to reprocess."
        continue
    }
    
    # Clone the fork
    Write-Info "Cloning $($fork.clone_url)..."
    
    if (!$DryRun) {
        try {
            Set-Location $ForksDir
            git clone $fork.clone_url $fork.name 2>&1 | Out-Null
            
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Cloned successfully"
                
                # Remove .git directory (we'll process as files)
                $gitDir = Join-Path $forkDir ".git"
                if (Test-Path $gitDir) {
                    Remove-Item -Recurse -Force $gitDir
                    Write-Info "Removed .git directory"
                }
                
                # Process through fork detection system
                Set-Location $WorkspaceRoot
                Write-Info "Processing through fork system..."
                & ".\crc\detect-forks.ps1" -Mode process -ForkName $fork.name
                
                Write-Success "Fork processed: $($fork.name)"
                
            } else {
                Write-Error "Clone failed for $($fork.name)"
            }
            
        } catch {
            Write-Error "Error processing $($fork.name): $($_.Exception.Message)"
        }
    } else {
        Write-Info "[DRY RUN] Would clone and process: $($fork.name)"
    }
}

Set-Location $WorkspaceRoot

Write-Info ""
Write-Info "==========================================="
Write-Info "Fork Processing Complete"
Write-Info "==========================================="
Write-Info ""
Write-Info "Summary:"
Write-Info "  Total forks found: $($forks.Count)"
Write-Info "  Processed: $($forksToProcess.Count)"
Write-Info ""
Write-Info "Next steps:"
Write-Info "  1. Review fork branches: git branch -a | Select-String 'fork/'"
Write-Info "  2. Check fork metadata: .\crc\detect-forks.ps1 -Mode list"
Write-Info "  3. Review changes in each fork branch"
Write-Info "  4. Cherry-pick or merge desired changes"
