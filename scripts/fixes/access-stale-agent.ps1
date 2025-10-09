# Access Stale Agent Files
# Helper to view/copy agents from stale directory

param(
    [Parameter(Mandatory=$false)]
    [string]$AgentName,
    
    [Parameter(Mandatory=$false)]
    [ValidateSet("List", "View", "Copy", "Info")]
    [string]$Action = "List",
    
    [Parameter(Mandatory=$false)]
    [string]$TargetPath
)

$WorkspaceRoot = "D:\dev\workspaces\noa_ark_os"
$StalePath = "$WorkspaceRoot\crc\drop-in\incoming\stale\agent_factory"

function Write-Info { param($Message) Write-Host "ℹ️  $Message" -ForegroundColor Cyan }
function Write-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "⚠️  $Message" -ForegroundColor Yellow }

# Check if stale directory exists
if (!(Test-Path $StalePath)) {
    Write-Warning "Stale directory not found: $StalePath"
    exit 1
}

switch ($Action) {
    "List" {
        Write-Info "Listing agents in stale directory..."
        Write-Info ""
        
        $agents = Get-ChildItem $StalePath -Filter "*.rs" -File
        
        Write-Info "Board Agents:"
        $agents | Where-Object { $_.Name -like "*board*" } | ForEach-Object {
            $size = [math]::Round($_.Length / 1024, 1)
            Write-Host "  $($_.Name) ($size KB)" -ForegroundColor Yellow
        }
        
        Write-Info ""
        Write-Info "Executive Agents:"
        $agents | Where-Object { $_.Name -like "*executive*" } | ForEach-Object {
            $size = [math]::Round($_.Length / 1024, 1)
            Write-Host "  $($_.Name) ($size KB)" -ForegroundColor Green
        }
        
        Write-Info ""
        Write-Info "Specialist Agents:"
        $agents | Where-Object { $_.Name -like "*specialist*" } | ForEach-Object {
            $size = [math]::Round($_.Length / 1024, 1)
            Write-Host "  $($_.Name) ($size KB)" -ForegroundColor Cyan
        }
        
        Write-Info ""
        Write-Info "Other:"
        $agents | Where-Object { 
            $_.Name -notlike "*board*" -and 
            $_.Name -notlike "*executive*" -and 
            $_.Name -notlike "*specialist*" 
        } | ForEach-Object {
            $size = [math]::Round($_.Length / 1024, 1)
            Write-Host "  $($_.Name) ($size KB)" -ForegroundColor Gray
        }
        
        Write-Info ""
        Write-Info "Total: $($agents.Count) agents"
    }
    
    "View" {
        if (!$AgentName) {
            Write-Warning "Agent name required for View action"
            Write-Info "Usage: .\access-stale-agent.ps1 -AgentName 'board_digest' -Action View"
            exit 1
        }
        
        $agentFile = Get-ChildItem $StalePath -Filter "*$AgentName*.rs" -File | Select-Object -First 1
        
        if (!$agentFile) {
            Write-Warning "Agent not found: $AgentName"
            exit 1
        }
        
        Write-Info "Opening: $($agentFile.Name)"
        code $agentFile.FullName
    }
    
    "Copy" {
        if (!$AgentName) {
            Write-Warning "Agent name required for Copy action"
            exit 1
        }
        
        $agentFile = Get-ChildItem $StalePath -Filter "*$AgentName*.rs" -File | Select-Object -First 1
        
        if (!$agentFile) {
            Write-Warning "Agent not found: $AgentName"
            exit 1
        }
        
        if (!$TargetPath) {
            Write-Warning "Target path required for Copy action"
            Write-Info "Usage: .\access-stale-agent.ps1 -AgentName 'board_digest' -Action Copy -TargetPath 'agents\src\implementations\board\digest.rs'"
            exit 1
        }
        
        $targetDir = Split-Path $TargetPath -Parent
        if (!(Test-Path $targetDir)) {
            New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
        }
        
        Copy-Item $agentFile.FullName "$WorkspaceRoot\$TargetPath" -Force
        Write-Success "Copied: $($agentFile.Name) -> $TargetPath"
        
        # Open in editor
        code "$WorkspaceRoot\$TargetPath"
    }
    
    "Info" {
        if (!$AgentName) {
            Write-Warning "Agent name required for Info action"
            exit 1
        }
        
        $agentFile = Get-ChildItem $StalePath -Filter "*$AgentName*.rs" -File | Select-Object -First 1
        
        if (!$agentFile) {
            Write-Warning "Agent not found: $AgentName"
            exit 1
        }
        
        $content = Get-Content $agentFile.FullName
        $lines = $content.Count
        $size = [math]::Round($agentFile.Length / 1024, 1)
        
        Write-Info "Agent Information:"
        Write-Info "  Name: $($agentFile.Name)"
        Write-Info "  Path: $($agentFile.FullName)"
        Write-Info "  Size: $size KB"
        Write-Info "  Lines: $lines"
        Write-Info ""
        
        # Count structs, enums, functions
        $structs = ($content | Select-String "pub struct" | Measure-Object).Count
        $enums = ($content | Select-String "pub enum" | Measure-Object).Count
        $functions = ($content | Select-String "pub fn|pub async fn" | Measure-Object).Count
        
        Write-Info "  Structs: $structs"
        Write-Info "  Enums: $enums"
        Write-Info "  Functions: $functions"
        Write-Info ""
        
        # Show first few lines
        Write-Info "Preview (first 20 lines):"
        Write-Host ($content | Select-Object -First 20 | Out-String) -ForegroundColor Gray
    }
}

Write-Info ""
Write-Info "Available actions:"
Write-Info "  List  - List all agents in stale directory"
Write-Info "  View  - Open agent in VS Code"
Write-Info "  Copy  - Copy agent to target location"
Write-Info "  Info  - Show agent information"
Write-Info ""
Write-Info "Examples:"
Write-Info "  .\access-stale-agent.ps1"
Write-Info "  .\access-stale-agent.ps1 -AgentName 'board_digest' -Action View"
Write-Info "  .\access-stale-agent.ps1 -AgentName 'board_digest' -Action Info"
Write-Info "  .\access-stale-agent.ps1 -AgentName 'board_digest' -Action Copy -TargetPath 'agents\src\implementations\board\digest.rs'"
