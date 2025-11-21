# Start Llama.cpp Server
param(
    [Parameter(Mandatory=$false)]
    [string]$ModelFile = "llama-3.2-3b-q4.gguf",
    [Parameter(Mandatory=$false)]
    [string]$Host = "127.0.0.1",
    [Parameter(Mandatory=$false)]
    [int]$Port = 8080,
    [Parameter(Mandatory=$false)]
    [int]$Threads = 8,
    [Parameter(Mandatory=$false)]
    [int]$GpuLayers = 35
)

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$workspaceRoot = Resolve-Path -Path (Join-Path $scriptRoot "..\..")
$serverDir = Join-Path $workspaceRoot "server"
$serverAiDir = Join-Path $serverDir "ai"
$llamaCppDir = Join-Path $serverAiDir "llama-cpp"
$binDir = Join-Path $llamaCppDir "bin"
$binPath = Join-Path $binDir "llama-server.exe"
$modelsDir = Join-Path $llamaCppDir "models"
$modelPath = Join-Path $modelsDir $ModelFile
$logsDir = Join-Path $llamaCppDir "logs"
$logPath = Join-Path $logsDir "server.log"

Write-Host "Starting Llama.cpp server..." -ForegroundColor Cyan
Write-Host "Model: $modelPath" -ForegroundColor Gray
Write-Host "Server: http://$Host:$Port" -ForegroundColor Gray

if (!(Test-Path $modelPath)) {
    Write-Host "ERROR: Model not found at $modelPath" -ForegroundColor Red
    Write-Host "Available models:" -ForegroundColor Yellow
    Get-ChildItem $modelsDir -Filter "*.gguf" | ForEach-Object { Write-Host "  - $($_.Name)" }
    exit 1
}

& $binPath `
    --model $modelPath `
    --host $Host `
    --port $Port `
    --ctx-size 8192 `
    --batch-size 512 `
    --threads $Threads `
    --n-gpu-layers $GpuLayers `
    --log-format text `
    --log-file $logPath

Write-Host "Server stopped" -ForegroundColor Yellow
