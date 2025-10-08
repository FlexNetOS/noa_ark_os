# Safe Merge of Old Work
# Processes old work through fork system for safe integration

param(
    [Parameter(Mandatory=$false)]
    [string]$OldWorkPath = "",  # Path to old work (local or URL)
    
    [Parameter(Mandatory=$false)]
    [string]$Strategy = "fork"  # fork, cherry-pick, or archive
)

$WorkspaceRoot = "D:\dev\workspaces\noa_ark_os"
$ForksDir = Join-Path $WorkspaceRoot "crc\drop-in\incoming\forks"

function Write-Info { param($Message) Write-Host "ℹ️  $Message" -ForegroundColor Cyan }
function Write-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "⚠️  $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor Red }

Write-Info "Safe Old Work Integration"
Write-Info "Strategy: $Strategy"

# Step 1: Create safety backup
Write-Info "Creating safety backup..."
git checkout main
git branch backup-before-merge -f
git tag -a "v0.1-pre-merge-$(Get-Date -Format 'yyyyMMdd-HHmmss')" -m "Pre-merge backup" -f

Write-Success "Safety backup created"

# Step 2: Process based on strategy
switch ($Strategy) {
    "fork" {
        Write-Info "Using Fork Processing Strategy (Recommended)"
        
        if ($OldWorkPath -eq "") {
            Write-Error "OldWorkPath required for fork strategy"
            Write-Info "Usage: .\merge-old-work.ps1 -OldWorkPath 'C:\path\to\old\work' -Strategy fork"
            exit 1
        }
        
        $forkName = "old-work-integration-$(Get-Date -Format 'yyyyMMdd-HHmmss')"
        $forkPath = Join-Path $ForksDir $forkName
        
        Write-Info "Creating fork directory: $forkName"
        New-Item -ItemType Directory -Force -Path $forkPath | Out-Null
        
        Write-Info "Copying old work files..."
        Copy-Item -Path "$OldWorkPath\*" -Destination $forkPath -Recurse -Force
        
        Write-Info "Processing through fork system..."
        & "$WorkspaceRoot\crc\detect-forks.ps1" -Mode process -ForkName $forkName
        
        Write-Success "Old work processed as fork: $forkName"
        Write-Info "Review branch: fork/$forkName"
        Write-Info "Next: git checkout fork/$forkName"
    }
    
    "cherry-pick" {
        Write-Info "Using Cherry-Pick Strategy"
        
        Write-Info "Creating review branch..."
        git checkout -b merge-review-$(Get-Date -Format 'yyyyMMdd-HHmmss')
        
        Write-Info "Fetching old work..."
        if ($OldWorkPath -match "^https://") {
            git remote add old-work $OldWorkPath -f
            git merge --no-commit --no-ff old-work/main
        } else {
            Write-Error "For cherry-pick, provide GitHub URL"
            exit 1
        }
        
        Write-Warning "Review conflicts and stage desired changes"
        Write-Info "Use: git status"
        Write-Info "Then: git cherry-pick <commit-hash>"
    }
    
    "archive" {
        Write-Info "Using Archive Strategy"
        
        Write-Info "Creating archive branch..."
        git checkout -b archive/old-work-$(Get-Date -Format 'yyyyMMdd')
        
        if ($OldWorkPath -match "^https://") {
            git pull $OldWorkPath main --allow-unrelated-histories
        } else {
            Write-Error "For archive, provide GitHub URL"
            exit 1
        }
        
        Write-Success "Old work archived in branch"
        Write-Info "Return to main: git checkout main"
        Write-Info "Compare: git diff main..archive/old-work-*"
    }
    
    default {
        Write-Error "Invalid strategy: $Strategy"
        Write-Info "Valid strategies: fork, cherry-pick, archive"
        exit 1
    }
}

Write-Success "Integration process started!"
Write-Info "Remember to:"
Write-Info "  1. Review changes carefully"
Write-Info "  2. Test: cargo build"
Write-Info "  3. Test: cargo test"
Write-Info "  4. Verify: agent registry intact"
