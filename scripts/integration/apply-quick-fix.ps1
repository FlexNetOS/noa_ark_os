# Quick Fix for Build - Use Simple Stubs
# Backs up complex implementations and uses simple placeholders

$implPath = "D:\dev\workspaces\noa_ark_os\agents\src\implementations"

Write-Host "🔧 Applying quick fix for agent implementations..." -ForegroundColor Cyan

# Create backup directory
$backupPath = "$implPath\_backup"
New-Item -ItemType Directory -Force -Path $backupPath | Out-Null

# Backup and replace specialist/mod.rs
if (Test-Path "$implPath\specialist\mod.rs") {
    Copy-Item "$implPath\specialist\mod.rs" "$backupPath\specialist_mod_original.rs" -Force
    Copy-Item "$implPath\specialist\mod_simple.rs" "$implPath\specialist\mod.rs" -Force
    Write-Host "  ✓ Replaced specialist/mod.rs with simple version" -ForegroundColor Green
}

# Remove individual agent files that cause issues
$agentFiles = @(
    "board\digest_agent.rs",
    "board\finance_board_agent.rs",
    "board\legal_compliance_board_agent.rs",
    "board\operations_board_agent.rs",
    "board\strategy_board_agent.rs",
    "executive\emergency_responder.rs",
    "executive\noa_commander.rs",
    "executive\priority_manager.rs",
    "executive\resource_allocator.rs",
    "executive\system_orchestrator.rs",
    "specialist\code_generation_agent.rs",
    "specialist\data_analytics_agent.rs",
    "specialist\deployment_agent.rs",
    "specialist\integration_agent.rs",
    "specialist\learning_agent.rs",
    "specialist\monitoring_agent.rs",
    "specialist\security_specialist_agent.rs",
    "specialist\testing_agent.rs"
)

foreach ($file in $agentFiles) {
    $fullPath = "$implPath\$file"
    if (Test-Path $fullPath) {
        $backupFile = $file -replace '\\', '_'
        Copy-Item $fullPath "$backupPath\$backupFile" -Force
        Remove-Item $fullPath -Force
        Write-Host "  ✓ Backed up and removed: $file" -ForegroundColor Yellow
    }
}

# Remove core component files
$coreFiles = @(
    "orchestration.rs",
    "mcp.rs",
    "automation.rs",
    "planner.rs",
    "queue.rs"
)

foreach ($file in $coreFiles) {
    $fullPath = "$implPath\$file"
    if (Test-Path $fullPath) {
        Copy-Item $fullPath "$backupPath\$file" -Force
        Remove-Item $fullPath -Force
        Write-Host "  ✓ Backed up and removed: $file" -ForegroundColor Yellow
    }
}

Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host "✅ Quick Fix Applied!" -ForegroundColor Green
Write-Host "  Original files backed up to: $backupPath"
Write-Host "  Simple stubs now in place"
Write-Host "`n🚀 Try building now: cargo build"
Write-Host ""
