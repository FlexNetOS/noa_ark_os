# Start Llama.cpp Server
$LlamaCppDir = "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp"
$BinPath = Join-Path $LlamaCppDir "bin\llama-server.exe"
$ModelPath = Join-Path $LlamaCppDir "models\"
$LogPath = Join-Path $LlamaCppDir "logs\server.log"

Write-Host "Starting Llama.cpp server..." -ForegroundColor Cyan
Write-Host "Model: $ModelPath" -ForegroundColor Gray
Write-Host "Server: http://127.0.0.1:8080" -ForegroundColor Gray

& $BinPath `
    --model $ModelPath `
    --host 127.0.0.1 `
    --port 8080 `
    --ctx-size 8192 `
    --batch-size 512 `
    --threads 8 `
    --n-gpu-layers 35 `
    --log-format text `
    --log-file $LogPath

Write-Host "Server stopped" -ForegroundColor Yellow
