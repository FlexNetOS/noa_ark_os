# Fix Green Environment Startup
# Simple, reliable configuration that actually works

Write-Host ""
Write-Host "🔧 FIXING GREEN ENVIRONMENT STARTUP" -ForegroundColor Cyan
Write-Host ("=" * 70) -ForegroundColor Cyan
Write-Host ""

# Step 1: Stop any existing servers
Write-Host "🛑 Step 1: Cleaning up existing processes..." -ForegroundColor Yellow
Get-Process | Where-Object { $_.ProcessName -like "*llama*" } | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
Write-Host "   ✅ Cleanup complete" -ForegroundColor Green
Write-Host ""

# Step 2: Test Blue server first
Write-Host "🔵 Step 2: Testing Blue Environment (current)..." -ForegroundColor Blue
Write-Host ""

$bluePort = 8080
$blueRunning = $false

try {
    $health = Invoke-WebRequest -Uri "http://127.0.0.1:$bluePort/health" -TimeoutSec 2 -UseBasicParsing -ErrorAction Stop
    if ($health.StatusCode -eq 200) {
        $blueRunning = $true
        Write-Host "   ✅ Blue is already running on port $bluePort" -ForegroundColor Green
    }
} catch {
    Write-Host "   ⚠️  Blue not running, will start it first" -ForegroundColor Yellow
}

Write-Host ""

# Step 3: Start Blue if not running
if (-not $blueRunning) {
    Write-Host "🚀 Step 3: Starting Blue Environment..." -ForegroundColor Blue
    Write-Host ""
    
    $blueCmd = @"
cd 'D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp'
Write-Host '🔵 BLUE Environment Starting...' -ForegroundColor Blue
Write-Host 'Port: 8080' -ForegroundColor Gray
Write-Host 'Config: Standard (batch 4096, parallel 16)' -ForegroundColor Gray
Write-Host ''
.\bin\llama-server.exe ``
    --model 'D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp\models\deepseek-coder-v2-q4.gguf' ``
    --host 127.0.0.1 ``
    --port 8080 ``
    --n-gpu-layers 99 ``
    --threads 32 ``
    --ctx-size 16384 ``
    --batch-size 4096 ``
    --n-parallel 16 ``
    --flash-attn
"@
    
    Start-Process powershell -ArgumentList "-NoExit", "-Command", $blueCmd -WindowStyle Normal
    
    Write-Host "   ⏳ Waiting for Blue to start (20 seconds)..." -ForegroundColor Yellow
    Start-Sleep -Seconds 20
    
    # Verify Blue started
    $attempts = 0
    $blueHealthy = $false
    while (-not $blueHealthy -and $attempts -lt 10) {
        try {
            $health = Invoke-WebRequest -Uri "http://127.0.0.1:$bluePort/health" -TimeoutSec 2 -UseBasicParsing -ErrorAction Stop
            if ($health.StatusCode -eq 200) {
                $blueHealthy = $true
                Write-Host "   ✅ Blue is now running!" -ForegroundColor Green
            }
        } catch {
            $attempts++
            Write-Host "   ⏳ Attempt $attempts/10..." -ForegroundColor Gray
            Start-Sleep -Seconds 3
        }
    }
    
    if (-not $blueHealthy) {
        Write-Host "   ❌ Blue failed to start!" -ForegroundColor Red
        Write-Host ""
        Write-Host "   Troubleshooting:" -ForegroundColor Yellow
        Write-Host "   1. Check if model file exists:" -ForegroundColor White
        Write-Host "      Test-Path 'D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp\models\deepseek-coder-v2-q4.gguf'" -ForegroundColor Gray
        Write-Host "   2. Check if llama-server.exe exists:" -ForegroundColor White
        Write-Host "      Test-Path 'D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp\bin\llama-server.exe'" -ForegroundColor Gray
        Write-Host "   3. Check GPU availability: nvidia-smi" -ForegroundColor Gray
        Write-Host ""
        exit 1
    }
    
    Write-Host ""
}

# Step 4: Test Blue performance
Write-Host "📊 Step 4: Testing Blue Performance..." -ForegroundColor Yellow
Write-Host ""

try {
    $testPrompt = "Test: Hello world"
    $testBody = @{
        prompt = $testPrompt
        temperature = 0.7
        max_tokens = 50
    } | ConvertTo-Json
    
    $startTime = Get-Date
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:$bluePort/completion" `
        -Method Post `
        -Body $testBody `
        -ContentType "application/json" `
        -TimeoutSec 30
    $endTime = Get-Date
    $elapsed = ($endTime - $startTime).TotalSeconds
    
    $blueThroughput = 50 / $elapsed
    
    Write-Host "   ✅ Blue Performance:" -ForegroundColor Green
    Write-Host "      • Response time: $([math]::Round($elapsed, 2))s" -ForegroundColor White
    Write-Host "      • Throughput: $([math]::Round($blueThroughput, 1)) tok/s" -ForegroundColor White
    Write-Host ""
    
} catch {
    Write-Host "   ⚠️  Blue test failed: $_" -ForegroundColor Yellow
    Write-Host ""
}

# Step 5: Start Green with SAME parameters (just different port)
Write-Host "🟢 Step 5: Starting Green Environment (optimized)..." -ForegroundColor Green
Write-Host ""

$greenPort = 8081

Write-Host "   Green Configuration:" -ForegroundColor Yellow
Write-Host "   • Port: $greenPort" -ForegroundColor White
Write-Host "   • Batch: 8192 (2x Blue)" -ForegroundColor Cyan
Write-Host "   • Parallel: 32 (2x Blue)" -ForegroundColor Cyan
Write-Host "   • Context: 32768 (2x Blue)" -ForegroundColor Cyan
Write-Host ""

$greenCmd = @"
cd 'D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp'
Write-Host '🟢 GREEN Environment Starting...' -ForegroundColor Green
Write-Host 'Port: 8081' -ForegroundColor Gray
Write-Host 'Config: Optimized (batch 8192, parallel 32)' -ForegroundColor Gray
Write-Host ''
.\bin\llama-server.exe ``
    --model 'D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp\models\deepseek-coder-v2-q4.gguf' ``
    --host 127.0.0.1 ``
    --port 8081 ``
    --n-gpu-layers 99 ``
    --threads 64 ``
    --ctx-size 32768 ``
    --batch-size 8192 ``
    --n-parallel 32 ``
    --flash-attn
"@

Start-Process powershell -ArgumentList "-NoExit", "-Command", $greenCmd -WindowStyle Normal

Write-Host "   ⏳ Waiting for Green to start (25 seconds)..." -ForegroundColor Yellow
Start-Sleep -Seconds 25

# Step 6: Verify Green started
Write-Host "🏥 Step 6: Health Check - Green Environment..." -ForegroundColor Cyan
Write-Host ""

$greenHealthy = $false
$attempts = 0
$maxAttempts = 15

while (-not $greenHealthy -and $attempts -lt $maxAttempts) {
    try {
        $health = Invoke-WebRequest -Uri "http://127.0.0.1:$greenPort/health" -TimeoutSec 3 -UseBasicParsing -ErrorAction Stop
        if ($health.StatusCode -eq 200) {
            $greenHealthy = $true
            Write-Host "   ✅ Green is HEALTHY!" -ForegroundColor Green
            Write-Host "      • Port: $greenPort" -ForegroundColor White
            Write-Host "      • Status: Ready for traffic" -ForegroundColor White
        }
    } catch {
        $attempts++
        Write-Host "   ⏳ Health check attempt $attempts/$maxAttempts..." -ForegroundColor Yellow
        Start-Sleep -Seconds 4
    }
}

if (-not $greenHealthy) {
    Write-Host "   ❌ Green failed to start after $maxAttempts attempts" -ForegroundColor Red
    Write-Host ""
    Write-Host "   Debugging steps:" -ForegroundColor Yellow
    Write-Host "   1. Check the Green server window for error messages" -ForegroundColor White
    Write-Host "   2. Verify GPU has enough free VRAM:" -ForegroundColor White
    Write-Host "      nvidia-smi" -ForegroundColor Gray
    Write-Host "   3. Try reducing Green batch size to 4096" -ForegroundColor White
    Write-Host "   4. Check if port 8081 is already in use:" -ForegroundColor White
    Write-Host "      netstat -ano | findstr :8081" -ForegroundColor Gray
    Write-Host ""
    
    # Try to get more info
    Write-Host "   Current GPU status:" -ForegroundColor Yellow
    try {
        $gpu = nvidia-smi --query-gpu=name,memory.used,memory.total,utilization.gpu --format=csv,noheader
        Write-Host "   $gpu" -ForegroundColor White
    } catch {
        Write-Host "   Could not get GPU info" -ForegroundColor Gray
    }
    
    Write-Host ""
    Write-Host "   Keeping Blue active (no changes made)" -ForegroundColor Blue
    exit 1
}

Write-Host ""

# Step 7: Test Green performance
Write-Host "📊 Step 7: Testing Green Performance..." -ForegroundColor Yellow
Write-Host ""

try {
    $testPrompt = "Test: Hello world"
    $testBody = @{
        prompt = $testPrompt
        temperature = 0.7
        max_tokens = 50
    } | ConvertTo-Json
    
    $startTime = Get-Date
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:$greenPort/completion" `
        -Method Post `
        -Body $testBody `
        -ContentType "application/json" `
        -TimeoutSec 30
    $endTime = Get-Date
    $elapsed = ($endTime - $startTime).TotalSeconds
    
    $greenThroughput = 50 / $elapsed
    
    Write-Host "   ✅ Green Performance:" -ForegroundColor Green
    Write-Host "      • Response time: $([math]::Round($elapsed, 2))s" -ForegroundColor White
    Write-Host "      • Throughput: $([math]::Round($greenThroughput, 1)) tok/s" -ForegroundColor White
    Write-Host ""
    
    # Compare
    Write-Host "   📈 Comparison:" -ForegroundColor Cyan
    if ($blueThroughput -gt 0) {
        $improvement = ($greenThroughput - $blueThroughput) / $blueThroughput * 100
        Write-Host "      • Blue: $([math]::Round($blueThroughput, 1)) tok/s" -ForegroundColor Blue
        Write-Host "      • Green: $([math]::Round($greenThroughput, 1)) tok/s" -ForegroundColor Green
        Write-Host "      • Improvement: $([math]::Round($improvement, 1))%" -ForegroundColor $(if ($improvement -gt 0) { 'Green' } else { 'Red' })
    }
    Write-Host ""
    
} catch {
    Write-Host "   ⚠️  Green test failed: $_" -ForegroundColor Yellow
    Write-Host ""
}

# Step 8: GPU Status
Write-Host "🎮 Step 8: GPU Utilization Check..." -ForegroundColor Cyan
Write-Host ""

try {
    $gpu = nvidia-smi --query-gpu=name,utilization.gpu,memory.used,memory.total --format=csv,noheader
    Write-Host "   Current GPU Status:" -ForegroundColor Yellow
    $gpuLines = $gpu -split "`n"
    foreach ($line in $gpuLines) {
        if ($line.Trim()) {
            Write-Host "   • $line" -ForegroundColor White
        }
    }
} catch {
    Write-Host "   ℹ️  GPU monitoring unavailable" -ForegroundColor Gray
}

Write-Host ""

# Final Summary
Write-Host "🎊 BOTH ENVIRONMENTS RUNNING!" -ForegroundColor Magenta
Write-Host ("=" * 70) -ForegroundColor Magenta
Write-Host ""

Write-Host "✅ Blue-Green Deployment Ready:" -ForegroundColor Green
Write-Host "   • Blue (8080): Standard config - $([math]::Round($blueThroughput, 1)) tok/s" -ForegroundColor Blue
Write-Host "   • Green (8081): Optimized config - $([math]::Round($greenThroughput, 1)) tok/s" -ForegroundColor Green
Write-Host ""

Write-Host "🔄 Next Steps:" -ForegroundColor Yellow
Write-Host "   1. Test both environments thoroughly" -ForegroundColor White
Write-Host "   2. Compare performance under load" -ForegroundColor White
Write-Host "   3. Switch traffic to Green if better" -ForegroundColor White
Write-Host "   4. Run infinite optimization loop" -ForegroundColor White
Write-Host ""

Write-Host "💡 To run optimization loop:" -ForegroundColor Cyan
Write-Host "   .\scripts\autonomous\infinite-optimization-loop.ps1" -ForegroundColor White
Write-Host ""
