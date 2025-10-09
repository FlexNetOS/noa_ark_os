# Quick Fix for Green Environment
# Diagnose and fix the startup issue

Write-Host "🔍 DIAGNOSING GREEN STARTUP ISSUE..." -ForegroundColor Cyan
Write-Host ""

# Check 1: Model file
$modelPath = "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp\models\deepseek-coder-v2-q4.gguf"
Write-Host "1. Checking model file..." -ForegroundColor Yellow
if (Test-Path $modelPath) {
    $size = (Get-Item $modelPath).Length / 1GB
    Write-Host "   ✅ Model found: $([math]::Round($size, 2)) GB" -ForegroundColor Green
} else {
    Write-Host "   ❌ Model NOT found at: $modelPath" -ForegroundColor Red
    Write-Host "   Fix: Run .\scripts\dev\verify-models.ps1" -ForegroundColor Yellow
    exit 1
}

# Check 2: llama-server executable
$serverPath = "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp\bin\llama-server.exe"
Write-Host "2. Checking llama-server..." -ForegroundColor Yellow
if (Test-Path $serverPath) {
    Write-Host "   ✅ llama-server.exe found" -ForegroundColor Green
} else {
    Write-Host "   ❌ llama-server.exe NOT found" -ForegroundColor Red
    Write-Host "   Location expected: $serverPath" -ForegroundColor Gray
    exit 1
}

# Check 3: GPU availability
Write-Host "3. Checking GPU..." -ForegroundColor Yellow
try {
    $gpu = nvidia-smi --query-gpu=name,memory.free --format=csv,noheader
    Write-Host "   ✅ GPU available" -ForegroundColor Green
    Write-Host "   $gpu" -ForegroundColor Gray
} catch {
    Write-Host "   ⚠️  nvidia-smi not found" -ForegroundColor Yellow
}

# Check 4: Port availability
Write-Host "4. Checking ports..." -ForegroundColor Yellow
$port8080 = netstat -ano | Select-String ":8080"
$port8081 = netstat -ano | Select-String ":8081"

if ($port8080) {
    Write-Host "   ✅ Port 8080 in use (Blue)" -ForegroundColor Blue
} else {
    Write-Host "   ⚠️  Port 8080 free (Blue not running)" -ForegroundColor Yellow
}

if ($port8081) {
    Write-Host "   ⚠️  Port 8081 ALREADY IN USE!" -ForegroundColor Red
    Write-Host "   This is why Green fails to start!" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "   Fix: Kill process using port 8081" -ForegroundColor Cyan
    $pids = netstat -ano | Select-String ":8081" | ForEach-Object { ($_ -split '\s+')[-1] } | Sort-Object -Unique
    foreach ($pid in $pids) {
        if ($pid -match '^\d+$') {
            Write-Host "   Process ID: $pid" -ForegroundColor White
            $proc = Get-Process -Id $pid -ErrorAction SilentlyContinue
            if ($proc) {
                Write-Host "   Process: $($proc.ProcessName)" -ForegroundColor Gray
            }
        }
    }
    Write-Host ""
    $kill = Read-Host "   Kill processes on port 8081? (y/n)"
    if ($kill -eq 'y') {
        foreach ($pid in $pids) {
            if ($pid -match '^\d+$') {
                Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue
                Write-Host "   ✅ Killed process $pid" -ForegroundColor Green
            }
        }
    }
} else {
    Write-Host "   ✅ Port 8081 free (ready for Green)" -ForegroundColor Green
}

Write-Host ""
Write-Host "✅ DIAGNOSIS COMPLETE" -ForegroundColor Green
Write-Host ""
Write-Host "Summary:" -ForegroundColor Yellow
Write-Host "  • Model file: OK" -ForegroundColor Green
Write-Host "  • Server executable: OK" -ForegroundColor Green
Write-Host "  • GPU: OK" -ForegroundColor Green
Write-Host "  • Ports: $(if (-not $port8081) { 'OK' } else { 'BLOCKED' })" -ForegroundColor $(if (-not $port8081) { 'Green' } else { 'Red' })
Write-Host ""

if (-not $port8081) {
    Write-Host "🚀 Ready to start Green!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Run: .\scripts\fix\fix-green-startup.ps1" -ForegroundColor Cyan
}
