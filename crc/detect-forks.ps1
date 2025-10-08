# Fork Repository Detection and Processing
# Monitors incoming/forks/ directory for new repositories
# Triggers CRC processing pipeline

param(
    [Parameter(Mandatory=$false)]
    [string]$Mode = "watch",  # watch, process, list
    
    [Parameter(Mandatory=$false)]
    [string]$ForkName = "",   # For processing specific fork
    
    [Parameter(Mandatory=$false)]
    [int]$IntervalSeconds = 60  # Watch interval
)

# Workspace root
$WorkspaceRoot = "D:\dev\workspaces\noa_ark_os"
$ForksDir = Join-Path $WorkspaceRoot "crc\drop-in\incoming\forks"
$ArchiveDir = Join-Path $WorkspaceRoot "crc\archive\forks"
$BranchesDir = Join-Path $WorkspaceRoot "crc\branches"

# Ensure directories exist
New-Item -ItemType Directory -Force -Path $ForksDir | Out-Null
New-Item -ItemType Directory -Force -Path $ArchiveDir | Out-Null
New-Item -ItemType Directory -Force -Path $BranchesDir | Out-Null

# Colors for output
function Write-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Info { param($Message) Write-Host "ℹ️  $Message" -ForegroundColor Cyan }
function Write-Warning { param($Message) Write-Host "⚠️  $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor Red }
function Write-Progress { param($Message) Write-Host "⏳ $Message" -ForegroundColor Blue }

# Initialize fork metadata
function Initialize-ForkMetadata {
    param([string]$ForkPath)
    
    $forkName = Split-Path $ForkPath -Leaf
    $metadataPath = Join-Path $ForkPath "metadata.json"
    
    Write-Progress "Initializing metadata for fork: $forkName"
    
    # Count files and calculate metrics
    $files = Get-ChildItem -Path $ForkPath -Recurse -File | Where-Object { $_.Name -ne 'metadata.json' -and $_.Name -ne 'branch.txt' }
    $totalLines = 0
    $extensions = @{}
    
    foreach ($file in $files) {
        $ext = $file.Extension
        if ($extensions.ContainsKey($ext)) {
            $extensions[$ext]++
        } else {
            $extensions[$ext] = 1
        }
        
        # Count lines for text files
        if ($ext -match '\.(rs|toml|md|txt|json|yaml|yml|sh|ps1|py)$') {
            try {
                $content = Get-Content $file.FullName -ErrorAction SilentlyContinue
                $totalLines += $content.Count
            } catch {
                # Skip binary or unreadable files
            }
        }
    }
    
    # Detect primary language
    $language = "unknown"
    if ($extensions.ContainsKey(".rs")) { $language = "rust" }
    elseif ($extensions.ContainsKey(".py")) { $language = "python" }
    elseif ($extensions.ContainsKey(".go")) { $language = "go" }
    elseif ($extensions.ContainsKey(".cs")) { $language = "csharp" }
    elseif ($extensions.ContainsKey(".js") -or $extensions.ContainsKey(".ts")) { $language = "javascript" }
    
    # Create metadata
    $metadata = @{
        fork_id = [guid]::NewGuid().ToString()
        repo_name = $forkName
        original_url = "unknown"
        fork_source = "manual"
        received_date = (Get-Date).ToString("o")
        processed_date = $null
        status = "pending"
        language = $language
        dependencies = @()
        metrics = @{
            lines_of_code = $totalLines
            files = $files.Count
            file_types = $extensions
            test_coverage = 0
            security_score = 0
        }
        crc_analysis = @{
            confidence = 0
            useful_components = @()
            adaptation_required = $true
            estimated_effort = "unknown"
        }
        integration = @{
            branch = "fork/$forkName"
            merged = $false
            merge_commit = $null
            conflicts = 0
        }
        archive = @{
            path = $null
            size_bytes = 0
            checksum_sha256 = $null
            compressed_date = $null
        }
    }
    
    # Save metadata
    $metadata | ConvertTo-Json -Depth 10 | Set-Content -Path $metadataPath -Encoding UTF8
    
    Write-Success "Metadata initialized for $forkName"
    return $metadata
}

# Detect new forks
function Get-NewForks {
    Write-Info "Scanning for new forks in: $ForksDir"
    
    $forks = Get-ChildItem -Path $ForksDir -Directory | Where-Object {
        $_.Name -ne '.gitkeep' -and $_.Name -ne 'README.md'
    }
    
    $newForks = @()
    
    foreach ($fork in $forks) {
        $metadataPath = Join-Path $fork.FullName "metadata.json"
        
        if (!(Test-Path $metadataPath)) {
            Write-Info "New fork detected: $($fork.Name)"
            $newForks += $fork
        } else {
            # Check status
            $metadata = Get-Content $metadataPath | ConvertFrom-Json
            if ($metadata.status -eq "pending") {
                Write-Info "Pending fork found: $($fork.Name)"
                $newForks += $fork
            }
        }
    }
    
    return $newForks
}

# Create branch for fork
function New-ForkBranch {
    param([string]$ForkName)
    
    $branchName = "fork/$ForkName"
    Write-Progress "Creating branch: $branchName"
    
    Push-Location $WorkspaceRoot
    
    try {
        # Check if branch already exists
        $existingBranch = git branch --list $branchName 2>$null
        
        if ($existingBranch) {
            Write-Warning "Branch $branchName already exists, using existing"
        } else {
            # Create new branch from main
            git checkout main 2>$null
            git pull origin main 2>$null
            git checkout -b $branchName
            
            Write-Success "Branch created: $branchName"
        }
        
        # Write branch name to fork directory
        $forkPath = Join-Path $ForksDir $ForkName
        $branchFilePath = Join-Path $forkPath "branch.txt"
        $branchName | Set-Content -Path $branchFilePath -Encoding UTF8
        
        return $true
    }
    catch {
        Write-Error "Failed to create branch: $_"
        return $false
    }
    finally {
        Pop-Location
    }
}

# Process fork through CRC
function Invoke-ForkProcessing {
    param([string]$ForkName)
    
    Write-Info "========================================="
    Write-Info "Processing Fork: $ForkName"
    Write-Info "========================================="
    
    $forkPath = Join-Path $ForksDir $ForkName
    $metadataPath = Join-Path $forkPath "metadata.json"
    
    # Initialize metadata if needed
    if (!(Test-Path $metadataPath)) {
        Initialize-ForkMetadata -ForkPath $forkPath
    }
    
    # Load metadata
    $metadata = Get-Content $metadataPath | ConvertFrom-Json
    
    # Update status
    $metadata.status = "processing"
    $metadata | ConvertTo-Json -Depth 10 | Set-Content -Path $metadataPath -Encoding UTF8
    
    # Step 1: Create branch
    Write-Progress "Step 1: Creating branch"
    $branchCreated = New-ForkBranch -ForkName $ForkName
    
    if (!$branchCreated) {
        Write-Error "Failed to create branch for $ForkName"
        $metadata.status = "error"
        $metadata | ConvertTo-Json -Depth 10 | Set-Content -Path $metadataPath -Encoding UTF8
        return $false
    }
    
    # Step 2: CRC Analysis (placeholder - will be implemented later)
    Write-Progress "Step 2: CRC Analysis (placeholder)"
    Write-Info "  → Language: $($metadata.language)"
    Write-Info "  → Files: $($metadata.metrics.files)"
    Write-Info "  → Lines: $($metadata.metrics.lines_of_code)"
    
    # For now, mark as needs review
    $metadata.crc_analysis.confidence = 0
    $metadata.crc_analysis.estimated_effort = "manual_review_required"
    
    # Step 3: Mark for review
    Write-Progress "Step 3: Marking for manual review"
    $metadata.status = "needs_review"
    $metadata.processed_date = (Get-Date).ToString("o")
    $metadata | ConvertTo-Json -Depth 10 | Set-Content -Path $metadataPath -Encoding UTF8
    
    Write-Success "Fork $ForkName processed and marked for review"
    Write-Info "Branch: $($metadata.integration.branch)"
    Write-Info "Next: Manual review and integration"
    
    return $true
}

# List all forks
function Get-AllForks {
    Write-Info "========================================="
    Write-Info "Fork Repository Status"
    Write-Info "========================================="
    
    $forks = Get-ChildItem -Path $ForksDir -Directory | Where-Object {
        $_.Name -ne '.gitkeep' -and $_.Name -ne 'README.md'
    }
    
    if ($forks.Count -eq 0) {
        Write-Info "No forks found in: $ForksDir"
        return
    }
    
    foreach ($fork in $forks) {
        $metadataPath = Join-Path $fork.FullName "metadata.json"
        
        if (Test-Path $metadataPath) {
            $metadata = Get-Content $metadataPath | ConvertFrom-Json
            
            Write-Info ""
            Write-Info "Fork: $($fork.Name)"
            Write-Info "  Status: $($metadata.status)"
            Write-Info "  Language: $($metadata.language)"
            Write-Info "  Files: $($metadata.metrics.files)"
            Write-Info "  Lines: $($metadata.metrics.lines_of_code)"
            Write-Info "  Received: $($metadata.received_date)"
            
            if ($metadata.processed_date) {
                Write-Info "  Processed: $($metadata.processed_date)"
            }
            
            if ($metadata.integration.branch) {
                Write-Info "  Branch: $($metadata.integration.branch)"
            }
        } else {
            Write-Warning "Fork: $($fork.Name) (no metadata)"
        }
    }
    
    Write-Info ""
    Write-Info "========================================="
    Write-Info "Total Forks: $($forks.Count)"
    Write-Info "========================================="
}

# Watch mode - continuous monitoring
function Start-ForkWatch {
    Write-Info "========================================="
    Write-Info "Fork Repository Monitor"
    Write-Info "========================================="
    Write-Info "Watching: $ForksDir"
    Write-Info "Interval: $IntervalSeconds seconds"
    Write-Info "Press Ctrl+C to stop"
    Write-Info "========================================="
    
    while ($true) {
        $newForks = Get-NewForks
        
        if ($newForks.Count -gt 0) {
            Write-Success "Found $($newForks.Count) fork(s) to process"
            
            foreach ($fork in $newForks) {
                Invoke-ForkProcessing -ForkName $fork.Name
            }
        } else {
            Write-Host "." -NoNewline
        }
        
        Start-Sleep -Seconds $IntervalSeconds
    }
}

# Main execution
Write-Host ""
Write-Info "Fork Repository Manager"
Write-Info "Workspace: $WorkspaceRoot"
Write-Info "Mode: $Mode"
Write-Host ""

switch ($Mode.ToLower()) {
    "watch" {
        Start-ForkWatch
    }
    "process" {
        if ($ForkName -eq "") {
            Write-Error "ForkName parameter required for process mode"
            Write-Info "Usage: .\detect-forks.ps1 -Mode process -ForkName 'fork-name'"
            exit 1
        }
        
        Invoke-ForkProcessing -ForkName $ForkName
    }
    "list" {
        Get-AllForks
    }
    default {
        Write-Error "Invalid mode: $Mode"
        Write-Info "Valid modes: watch, process, list"
        Write-Info "Usage:"
        Write-Info "  Watch mode:   .\detect-forks.ps1 -Mode watch"
        Write-Info "  Process fork: .\detect-forks.ps1 -Mode process -ForkName 'fork-name'"
        Write-Info "  List forks:   .\detect-forks.ps1 -Mode list"
        exit 1
    }
}
