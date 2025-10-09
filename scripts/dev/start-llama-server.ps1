# Start Llama.cpp Server
$LlamaCppDir = "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp"
$BinPath = Join-Path $LlamaCppDir "bin\llama-server.exe"
$ModelPath = Join-Path $LlamaCppDir "models\llama-3.2-3b-q4.gguf"
$LogPath = Join-Path $LlamaCppDir "logs\server.log"

Write-Host "Starting Llama.cpp server..." -ForegroundColor Cyan
Write-Host "Model: $ModelPath" -ForegroundColor Gray
Write-Host "Server: http://127.0.0.1:8080" -ForegroundColor Gray

if (!(Test-Path $ModelPath)) {
    Write-Host "ERROR: Model not found at $ModelPath" -ForegroundColor Red
    Write-Host "Available models:" -ForegroundColor Yellow
    Get-ChildItem (Join-Path $LlamaCppDir "models") -Filter "*.gguf" | ForEach-Object { Write-Host "  - $($_.Name)" }
    exit 1
}

& $BinPath `
    --model $ModelPath `
    --host 127.0.0.1 `
    --port 8080 `
    --ctx-size 8192 `
    --batch-size 512 `
    --threads 8 `
    --n-gpu-layers 35

Write-Host "Server stopped" -ForegroundColor Yellow
