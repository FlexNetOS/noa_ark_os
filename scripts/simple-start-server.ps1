# Simple Server Startup - No fancy formatting, just works
# Use this if the other scripts are having issues

$ErrorActionPreference = "Continue"

Write-Host "Starting llama.cpp server..."
Write-Host ""

cd "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp"

# Simple command that definitely works
.\bin\llama-server.exe `
    --model "models\deepseek-coder-v2-q4.gguf" `
    --host 127.0.0.1 `
    --port 8080 `
    --n-gpu-layers 99 `
    --threads 32 `
    --ctx-size 16384 `
    --batch-size 4096 `
    --n-parallel 16

# Keep window open
Read-Host "Press Enter to close"
