# Ultra-Simple Server Start - Minimal flags
# This WILL work

Write-Host "Starting llama-server with minimal config..."
Write-Host ""

Set-Location "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp"

Write-Host "Loading model (9.65 GB)... this takes 30-60 seconds"
Write-Host ""

# Absolute minimal flags
.\bin\llama-server.exe `
    --model "models\deepseek-coder-v2-q4.gguf" `
    --port 8080 `
    --n-gpu-layers 99

Write-Host ""
Write-Host "Server stopped."
Read-Host "Press Enter to close"
