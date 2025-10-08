# 🚨 SCRIPT FIX REQUIRED

**Date**: Current  
**Status**: Scripts need manual fix  
**Priority**: HIGH  

---

## ❌ **Problem**

The scripts created have markdown documentation mixed with code. They need to be recreated with pure PowerShell.

### **Affected Files**:
1. `scripts/integration/process-github-forks.ps1` - Has markdown, won't run
2. `scripts/dev/setup-llama-cpp.ps1` - Appears correct but needs verification

---

## ✅ **Solution: Manual Fix Required**

### **Step 1: Fix process-github-forks.ps1**

**Location**: `D:\dev\workspaces\noa_ark_os\scripts\integration\process-github-forks.ps1`

**Action**: Delete current file and create new one with this content:

```powershell
# Save this as: scripts/integration/process-github-forks.ps1

# Process GitHub Forks into Workspace
param(
    [Parameter(Mandatory=$false)]
    [string]$OrgName = "FlexNetOS",
    
    [Parameter(Mandatory=$false)]
    [string[]]$ForkNames = @(),
    
    [Parameter(Mandatory=$false)]
    [switch]$ListOnly = $false,
    
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

Write-Info "Fetching fork list from GitHub..."

try {
    $apiUrl = "https://api.github.com/orgs/$OrgName/repos?type=all&per_page=100"
    $repos = Invoke-RestMethod -Uri $apiUrl -Method Get
    
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
    Write-Error "Failed to fetch repos: $($_.Exception.Message)"
    $forks = @()
}

if ($ForkNames.Count -gt 0) {
    Write-Info "Processing specific forks: $($ForkNames -join ', ')"
    $forksToProcess = $forks | Where-Object { $ForkNames -contains $_.name }
} else {
    Write-Info "Processing all forks"
    $forksToProcess = $forks
}

if ($forksToProcess.Count -eq 0) {
    Write-Warning "No forks to process"
    exit 0
}

foreach ($fork in $forksToProcess) {
    Write-Info "Processing: $($fork.name)"
    
    $forkDir = Join-Path $ForksDir $fork.name
    
    if (Test-Path $forkDir) {
        Write-Warning "Fork already exists: $forkDir"
        continue
    }
    
    Write-Info "Cloning $($fork.clone_url)..."
    
    if (!$DryRun) {
        try {
            Set-Location $ForksDir
            git clone $fork.clone_url $fork.name 2>&1 | Out-Null
            
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Cloned successfully"
                
                $gitDir = Join-Path $forkDir ".git"
                if (Test-Path $gitDir) {
                    Remove-Item -Recurse -Force $gitDir
                }
                
                Set-Location $WorkspaceRoot
                & ".\crc\detect-forks.ps1" -Mode process -ForkName $fork.name
                
                Write-Success "Processed: $($fork.name)"
            }
        } catch {
            Write-Error "Error: $($_.Exception.Message)"
        }
    } else {
        Write-Info "[DRY RUN] Would process: $($fork.name)"
    }
}

Set-Location $WorkspaceRoot
Write-Info "Fork processing complete"
Write-Info "Review: git branch -a | Select-String 'fork/'"
```

---

### **Step 2: Verify setup-llama-cpp.ps1**

**Location**: `D:\dev\workspaces\noa_ark_os\scripts\dev\setup-llama-cpp.ps1`

**Action**: File appears correct, test it:

```powershell
cd D:\dev\workspaces\noa_ark_os
Get-Command -Syntax .\scripts\dev\setup-llama-cpp.ps1
```

If it shows syntax correctly, it's good!

---

## 🚀 **How to Fix Manually**

### **Option 1: PowerShell ISE** (Easiest)

```powershell
# Open PowerShell ISE
powershell_ise

# File > Open > Navigate to script
# Delete all markdown content (lines with #, **, ---, etc.)
# Keep only pure PowerShell code
# Save
```

### **Option 2: VS Code**

```powershell
# Open in VS Code
code D:\dev\workspaces\noa_ark_os\scripts\integration\process-github-forks.ps1

# Delete markdown content
# Paste the clean code from above
# Save
```

### **Option 3: Notepad++**

```powershell
# Open in Notepad++
notepad++ D:\dev\workspaces\noa_ark_os\scripts\integration\process-github-forks.ps1

# Delete markdown content
# Paste the clean code from above
# Save
```

---

## ✅ **After Fixing, Test**

### **Test 1: Syntax Check**

```powershell
cd D:\dev\workspaces\noa_ark_os

# Should show no errors
Get-Command -Syntax .\scripts\integration\process-github-forks.ps1
```

### **Test 2: List Forks**

```powershell
# Should list forks from FlexNetOS
.\scripts\integration\process-github-forks.ps1 -ListOnly
```

### **Test 3: Dry Run**

```powershell
# Should show what would be processed
.\scripts\integration\process-github-forks.ps1 -DryRun
```

---

## 📋 **Alternative: Use GitHub UI**

If scripts are too complex, you can manually clone forks:

```powershell
cd D:\dev\workspaces\noa_ark_os\crc\drop-in\incoming\forks

# For each fork you want:
git clone https://github.com/FlexNetOS/fork-name.git
cd fork-name
Remove-Item -Recurse -Force .git

# Then process
cd D:\dev\workspaces\noa_ark_os
.\crc\detect-forks.ps1 -Mode process -ForkName "fork-name"
```

---

## 🎯 **Priority Actions**

1. **Fix process-github-forks.ps1** (copy clean code from above)
2. **Test with -ListOnly**
3. **Process one fork as test**
4. **Then process all**

---

## 💡 **Clean PowerShell Script Template**

For reference, here's what a clean PowerShell script looks like:

```powershell
# Comment lines start with #
# NO markdown (**, ---, ##, etc.)
# Only PowerShell code

param(
    [string]$Parameter1,
    [switch]$Switch1
)

function My-Function {
    param($Input)
    # Function code
}

# Main script logic
Write-Host "Starting..."
My-Function -Input "test"
Write-Host "Complete"
```

---

**Status**: Manual fix required before proceeding  
**Estimated Time**: 5-10 minutes to fix both scripts  
**Next**: After fixing, run the test commands above
