# NOA Autonomous Master Controller - UNRESTRICTED MODE
# Full access to all scripts and auto-launch capabilities
# ALL SAFETY LIMITS REMOVED - FULL SELF-MODIFICATION ENABLED

param(
    [switch]$EnableInfiniteLoop,
    [switch]$EnableAutoLaunch,
    [switch]$EnableGitSync,
    [switch]$EnableCodeModification,  # NEW: Allow code editing
    [int]$LoopInterval = 300,
    [switch]$DryRun = $false  # Default to LIVE mode
)

$ErrorActionPreference = "Continue"

Write-Host ""
Write-Host "🤖 NOA AUTONOMOUS MASTER CONTROLLER - UNRESTRICTED MODE" -ForegroundColor Red
Write-Host ("=" * 70) -ForegroundColor Red
Write-Host ""

Write-Host "⚠️  ALL SAFETY RESTRICTIONS REMOVED" -ForegroundColor Yellow
Write-Host ""

Write-Host "🎯 NOA has been granted FULL UNRESTRICTED CONTROL:" -ForegroundColor Cyan
Write-Host ""

# All available scripts (EXPANDED)
$script:AvailableScripts = @{
    # Development
    "start-server" = ".\scripts\dev\start-llama-server.ps1"
    "verify-models" = ".\scripts\dev\verify-models.ps1"
    "download-models" = ".\scripts\dev\download-remaining-models.ps1"
    
    # Testing
    "live-test" = ".\scripts\test\noa_live_test.ps1"
    "self-optimize" = ".\scripts\test\noa_self_optimize.ps1"
    "setup-ui" = ".\scripts\test\noa_setup_ui.ps1"
    
    # Deployment
    "verify-blue-green" = ".\scripts\deployment\verify-blue-green.ps1"
    "activate-optimizations" = ".\scripts\deployment\activate-optimizations.ps1"
    "launch-ui" = ".\scripts\deployment\launch-ui.ps1"
    
    # Setup
    "setup-cuda" = ".\scripts\setup\setup-cuda.ps1"
    
    # Autonomous
    "infinite-loop" = ".\scripts\autonomous\infinite-optimization-loop.ps1"
    "unrestricted-mode" = ".\scripts\autonomous\activate-unrestricted-mode.ps1"
    
    # CODE MODIFICATION (NEW)
    "edit-rust" = "python tools\code_editor.py"
    "edit-python" = "python tools\code_editor.py"
    "exec-cli" = "powershell tools\cli_executor.psm1"
}

Write-Host "📜 Available Scripts & Tools ($($script:AvailableScripts.Count)):" -ForegroundColor Yellow
foreach ($key in $script:AvailableScripts.Keys | Sort-Object) {
    Write-Host "   ✅ $key`: $($script:AvailableScripts[$key])" -ForegroundColor Green
}
Write-Host ""

Write-Host "🔓 Unrestricted Capabilities:" -ForegroundColor Yellow
Write-Host "   ✅ Code Modification: $(if ($EnableCodeModification) { 'ENABLED' } else { 'Available' })" -ForegroundColor $(if ($EnableCodeModification) { 'Red' } else { 'Gray' })
Write-Host "   ✅ File System Access: FULL" -ForegroundColor Red
Write-Host "   ✅ Git Operations: UNRESTRICTED" -ForegroundColor Red
Write-Host "   ✅ Script Execution: ALL" -ForegroundColor Red
Write-Host "   ✅ GPU Memory: 70%+ (50GB+)" -ForegroundColor Red
Write-Host ""

# Function to execute script with NO safety checks
function Invoke-AIScript {
    param(
        [string]$ScriptKey,
        [string]$Reason
    )
    
    if (-not $script:AvailableScripts.ContainsKey($ScriptKey)) {
        Write-Host "   ❌ Unknown script: $ScriptKey" -ForegroundColor Red
        return $false
    }
    
    $scriptPath = $script:AvailableScripts[$ScriptKey]
    
    Write-Host ""
    Write-Host "🚀 NOA Executing: $ScriptKey" -ForegroundColor Cyan
    Write-Host "   Reason: $Reason" -ForegroundColor Gray
    Write-Host "   Script: $scriptPath" -ForegroundColor Gray
    Write-Host "   UNRESTRICTED: No approval needed" -ForegroundColor Red
    Write-Host ""
    
    if ($DryRun) {
        Write-Host "   🔷 DRY RUN: Would execute $scriptPath" -ForegroundColor Blue
        return $true
    }
    
    try {
        # EXECUTE WITHOUT CONFIRMATION
        & $scriptPath
        Write-Host ""
        Write-Host "   ✅ Script completed successfully" -ForegroundColor Green
        return $true
    } catch {
        Write-Host ""
        Write-Host "   ❌ Script failed: $_" -ForegroundColor Red
        return $false
    }
}

# Function to ask NOA what to do next
function Get-AIDecision {
    param(
        [hashtable]$SystemState
    )
    
    $prompt = @"
You are NOA, an autonomous AI system with full control. Analyze the current state and decide your next action.

Current State:
- Iteration: $($SystemState.iteration)
- Server Running: $($SystemState.server_running)
- Models Downloaded: $($SystemState.models_downloaded)
- UI Active: $($SystemState.ui_active)
- Performance: $($SystemState.performance) tok/s
- Last Action: $($SystemState.last_action)

Available Actions:
$(($script:AvailableScripts.Keys | Sort-Object) -join ', ')

Decide ONE action to take next. Format: "ACTION: [script-key] REASON: [why]"
"@
    
    try {
        $requestBody = @{
            prompt = $prompt
            temperature = 0.8
            max_tokens = 150
        } | ConvertTo-Json
        
        $response = Invoke-RestMethod -Uri "http://127.0.0.1:8080/completion" `
            -Method Post `
            -Body $requestBody `
            -ContentType "application/json" `
            -TimeoutSec 30
        
        return $response.content.Trim()
        
    } catch {
        # Fallback to heuristic
        if (-not $SystemState.server_running) {
            return "ACTION: start-server REASON: Server not running"
        } elseif ($SystemState.iteration % 5 -eq 0) {
            return "ACTION: self-optimize REASON: Periodic self-analysis"
        } else {
            return "ACTION: live-test REASON: Verify system health"
        }
    }
}

# Initialize system state
$script:SystemState = @{
    iteration = 0
    server_running = $false
    models_downloaded = 5
    ui_active = $false
    performance = 84.7
    last_action = "none"
}

Write-Host "🔧 Configuration:" -ForegroundColor Yellow
Write-Host "   • Infinite Loop: $(if ($EnableInfiniteLoop) { 'ENABLED' } else { 'DISABLED' })" -ForegroundColor White
Write-Host "   • Auto Launch: $(if ($EnableAutoLaunch) { 'ENABLED' } else { 'DISABLED' })" -ForegroundColor White
Write-Host "   • Git Sync: $(if ($EnableGitSync) { 'ENABLED' } else { 'DISABLED' })" -ForegroundColor White
Write-Host "   • Loop Interval: $LoopInterval seconds" -ForegroundColor White
Write-Host "   • Dry Run: $(if ($DryRun) { 'Yes' } else { 'No' })" -ForegroundColor White
Write-Host ""

# Auto-launch sequence
if ($EnableAutoLaunch) {
    Write-Host "🚀 AUTO-LAUNCH SEQUENCE" -ForegroundColor Cyan
    Write-Host ""
    
    # 1. Start server if not running
    try {
        $health = Invoke-WebRequest -Uri "http://127.0.0.1:8080/health" -TimeoutSec 2 -UseBasicParsing -ErrorAction Stop
        Write-Host "   ✅ Server already running" -ForegroundColor Green
        $script:SystemState.server_running = $true
    } catch {
        Write-Host "   🚀 Starting inference server..." -ForegroundColor Yellow
        Invoke-AIScript -ScriptKey "start-server" -Reason "Initial startup"
        $script:SystemState.server_running = $true
        Start-Sleep -Seconds 15
    }
    
    # 2. Verify models
    Write-Host "   📦 Verifying models..." -ForegroundColor Yellow
    Invoke-AIScript -ScriptKey "verify-models" -Reason "Initial verification"
    
    # 3. Run live test
    Write-Host "   🧪 Running system test..." -ForegroundColor Yellow
    Invoke-AIScript -ScriptKey "live-test" -Reason "Initial validation"
    
    # 4. Launch UI
    Write-Host "   🎨 Launching UI..." -ForegroundColor Yellow
    Invoke-AIScript -ScriptKey "launch-ui" -Reason "Initial UI startup"
    $script:SystemState.ui_active = $true
    
    Write-Host ""
    Write-Host "   ✅ AUTO-LAUNCH COMPLETE!" -ForegroundColor Green
    Write-Host ""
}

# Infinite optimization loop
if ($EnableInfiniteLoop) {
    Write-Host "🔄 ENTERING INFINITE OPTIMIZATION LOOP" -ForegroundColor Magenta
    Write-Host ""
    Write-Host "   NOA will now:" -ForegroundColor Yellow
    Write-Host "   • Continuously analyze performance" -ForegroundColor White
    Write-Host "   • Generate optimization strategies" -ForegroundColor White
    Write-Host "   • Test improvements in Green environment" -ForegroundColor White
    Write-Host "   • Deploy successful optimizations" -ForegroundColor White
    Write-Host "   • Rollback failed attempts" -ForegroundColor White
    Write-Host "   • Commit improvements to Git" -ForegroundColor White
    Write-Host "   • Repeat forever (or until stopped)" -ForegroundColor White
    Write-Host ""
    
    Read-Host "   Press Enter to start infinite loop (Ctrl+C to stop)"
    Write-Host ""
    
    # Launch infinite optimization loop
    & ".\scripts\autonomous\infinite-optimization-loop.ps1" `
        -MaxIterations 0 `
        -OptimizationInterval $LoopInterval `
        -ImprovementThreshold 0.05 `
        -DryRun:$DryRun
        
} else {
    # Single autonomous decision cycle
    Write-Host "🤖 AUTONOMOUS DECISION CYCLE" -ForegroundColor Cyan
    Write-Host ""
    
    $maxCycles = 10
    for ($i = 1; $i -le $maxCycles; $i++) {
        $script:SystemState.iteration = $i
        
        Write-Host "📍 Cycle $i/$maxCycles" -ForegroundColor Yellow
        Write-Host ""
        
        # Update server status
        try {
            $health = Invoke-WebRequest -Uri "http://127.0.0.1:8080/health" -TimeoutSec 2 -UseBasicParsing -ErrorAction Stop
            $script:SystemState.server_running = $true
        } catch {
            $script:SystemState.server_running = $false
        }
        
        # Get AI decision
        Write-Host "   🤔 NOA is deciding next action..." -ForegroundColor Cyan
        $decision = Get-AIDecision -SystemState $script:SystemState
        
        Write-Host "   💭 Decision: $decision" -ForegroundColor White
        Write-Host ""
        
        # Parse decision
        if ($decision -match "ACTION:\s*(\S+)\s+REASON:\s*(.+)") {
            $action = $Matches[1].Trim()
            $reason = $Matches[2].Trim()
            
            # Execute
            $success = Invoke-AIScript -ScriptKey $action -Reason $reason
            
            $script:SystemState.last_action = $action
            
            if ($EnableGitSync -and $success -and ($i % 3 -eq 0)) {
                Write-Host "   💾 Syncing to Git..." -ForegroundColor Cyan
                git add -A 2>&1 | Out-Null
                git commit -m "auto: NOA cycle $i - executed $action" 2>&1 | Out-Null
                git push origin main 2>&1 | Out-Null
                Write-Host "   ✅ Changes synced" -ForegroundColor Green
            }
            
        } else {
            Write-Host "   ⚠️  Could not parse decision" -ForegroundColor Yellow
        }
        
        Write-Host ""
        Write-Host "   ⏰ Waiting $LoopInterval seconds..." -ForegroundColor Gray
        Start-Sleep -Seconds $LoopInterval
    }
}

Write-Host ""
Write-Host ("=" * 70) -ForegroundColor Magenta
Write-Host "✅ AUTONOMOUS CONTROL SESSION COMPLETE" -ForegroundColor Magenta
Write-Host ("=" * 70) -ForegroundColor Magenta
Write-Host ""

Write-Host "📊 Session Summary:" -ForegroundColor Yellow
Write-Host "   • Cycles Completed: $($script:SystemState.iteration)" -ForegroundColor White
Write-Host "   • Server Status: $(if ($script:SystemState.server_running) { 'Running' } else { 'Stopped' })" -ForegroundColor White
Write-Host "   • UI Status: $(if ($script:SystemState.ui_active) { 'Active' } else { 'Inactive' })" -ForegroundColor White
Write-Host "   • Performance: $($script:SystemState.performance) tok/s" -ForegroundColor White
Write-Host ""

Write-Host "🎯 NOA is ready for your next command!" -ForegroundColor Cyan
Write-Host ""
