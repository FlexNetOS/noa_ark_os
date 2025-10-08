# Setup Llama.cpp for NOA ARK OS
# Installs llama.cpp and prepares the inference server

param(
    [Parameter(Mandatory=$false)]
    [switch]$BuildFromSource = $false,
    
    [Parameter(Mandatory=$false)]
    [switch]$SkipModelDownload = $false,
    
    [Parameter(Mandatory=$false)]
    [string]$ModelSize = "3b"  # 3b, 7b, or 8b
)

$WorkspaceRoot = "D:\dev\workspaces\noa_ark_os"
$ServerAIDir = Join-Path $WorkspaceRoot "server\ai"
$LlamaCppDir = Join-Path $ServerAIDir "llama-cpp"

function Write-Info { param($Message) Write-Host "ℹ️  $Message" -ForegroundColor Cyan }
function Write-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "⚠️  $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor Red }

Write-Info "Llama.cpp Setup for NOA ARK OS"
Write-Info "Workspace: $WorkspaceRoot"

# Create directory structure
Write-Info "Creating directory structure..."
$directories = @(
    "$LlamaCppDir\bin",
    "$LlamaCppDir\models",
    "$LlamaCppDir\configs",
    "$LlamaCppDir\logs"
)

foreach ($dir in $directories) {
    if (!(Test-Path $dir)) {
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        Write-Success "Created: $dir"
    } else {
        Write-Info "Already exists: $dir"
    }
}

# Step 1: Install llama.cpp
if ($BuildFromSource) {
    Write-Info "Building llama.cpp from source..."
    Write-Warning "This requires CMake and Visual Studio Build Tools"
    
    $llamaCppSource = Join-Path $ServerAIDir "llama.cpp-source"
    
    if (!(Test-Path $llamaCppSource)) {
        Write-Info "Cloning llama.cpp repository..."
        Set-Location $ServerAIDir
        git clone https://github.com/ggerganov/llama.cpp.git llama.cpp-source
    }
    
    Set-Location $llamaCppSource
    
    Write-Info "Building with CMake..."
    mkdir build -Force | Out-Null
    Set-Location build
    
    # Check for CUDA
    $hasCuda = Test-Path "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA"
    
    if ($hasCuda) {
        Write-Info "CUDA detected, building with GPU support..."
        cmake .. -DGGML_CUDA=ON -DCMAKE_BUILD_TYPE=Release
    } else {
        Write-Info "Building CPU-only version..."
        cmake .. -DCMAKE_BUILD_TYPE=Release
    }
    
    cmake --build . --config Release
    
    # Copy binaries
    Write-Info "Copying binaries..."
    Copy-Item "bin\Release\*" "$LlamaCppDir\bin\" -Force
    
    Write-Success "Llama.cpp built from source"
    
} else {
    Write-Info "Downloading prebuilt llama.cpp binaries..."
    
    # Determine architecture
    $arch = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "x86" }
    $llamaCppVersion = "b4315"
    
    # Check for CUDA
    $hasCuda = Test-Path "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA"
    
    if ($hasCuda) {
        $downloadUrl = "https://github.com/ggerganov/llama.cpp/releases/download/$llamaCppVersion/llama-$llamaCppVersion-bin-win-cuda-cu12.2.0-$arch.zip"
        Write-Info "Downloading CUDA-enabled version..."
    } else {
        $downloadUrl = "https://github.com/ggerganov/llama.cpp/releases/download/$llamaCppVersion/llama-$llamaCppVersion-bin-win-avx2-$arch.zip"
        Write-Info "Downloading CPU-only version..."
    }
    
    $zipPath = Join-Path $ServerAIDir "llama-cpp.zip"
    
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath
        Write-Success "Downloaded llama.cpp"
        
        Expand-Archive -Path $zipPath -DestinationPath "$LlamaCppDir\bin" -Force
        Remove-Item $zipPath
        
        Write-Success "Llama.cpp binaries installed"
    } catch {
        Write-Error "Failed to download: $($_.Exception.Message)"
        Write-Warning "Try building from source with -BuildFromSource"
        exit 1
    }
}

# Step 2: Download model
if (!$SkipModelDownload) {
    Write-Info "Downloading model..."
    
    $modelUrl = switch ($ModelSize) {
        "3b" {
            "https://huggingface.co/QuantFactory/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct.Q4_K_M.gguf"
        }
        "7b" {
            "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf"
        }
        "8b" {
            "https://huggingface.co/QuantFactory/Meta-Llama-3.1-8B-Instruct-GGUF/resolve/main/Meta-Llama-3.1-8B-Instruct.Q4_K_M.gguf"
        }
        default {
            Write-Error "Invalid model size: $ModelSize. Use 3b, 7b, or 8b"
            exit 1
        }
    }
    
    $modelFileName = Split-Path $modelUrl -Leaf
    $modelPath = Join-Path "$LlamaCppDir\models" $modelFileName
    
    if (Test-Path $modelPath) {
        Write-Info "Model already exists: $modelPath"
    } else {
        Write-Info "Downloading $ModelSize model (this may take a while)..."
        try {
            Invoke-WebRequest -Uri $modelUrl -OutFile $modelPath
            Write-Success "Model downloaded: $modelPath"
        } catch {
            Write-Error "Failed to download model: $($_.Exception.Message)"
            Write-Warning "You can download models manually from HuggingFace"
            exit 1
        }
    }
} else {
    Write-Info "Skipping model download"
}

# Step 3: Create configuration
Write-Info "Creating server configuration..."

$configContent = @"
# Llama.cpp Server Configuration
server:
  host: 127.0.0.1
  port: 8080
  threads: 8
  gpu_layers: 35  # Set to 0 for CPU-only

models:
  - name: default
    path: ./models/$modelFileName
    context_size: 8192
    batch_size: 512

inference:
  temperature: 0.7
  top_p: 0.9
  top_k: 40
  repeat_penalty: 1.1
  max_tokens: 2048

logging:
  level: info
  file: ./logs/server.log
  rotation: daily
"@

$configPath = Join-Path "$LlamaCppDir\configs" "server.yaml"
$configContent | Set-Content -Path $configPath -Encoding UTF8

Write-Success "Configuration created: $configPath"

# Step 4: Create start script
Write-Info "Creating start script..."

$startScriptContent = @"
# Start Llama.cpp Server
`$LlamaCppDir = "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp"
`$BinPath = Join-Path `$LlamaCppDir "bin\llama-server.exe"
`$ModelPath = Join-Path `$LlamaCppDir "models\$modelFileName"
`$LogPath = Join-Path `$LlamaCppDir "logs\server.log"

Write-Host "Starting Llama.cpp server..." -ForegroundColor Cyan
Write-Host "Model: `$ModelPath" -ForegroundColor Gray
Write-Host "Server: http://127.0.0.1:8080" -ForegroundColor Gray

& `$BinPath ``
    --model `$ModelPath ``
    --host 127.0.0.1 ``
    --port 8080 ``
    --ctx-size 8192 ``
    --batch-size 512 ``
    --threads 8 ``
    --n-gpu-layers 35 ``
    --log-format text ``
    --log-file `$LogPath

Write-Host "Server stopped" -ForegroundColor Yellow
"@

$startScriptPath = Join-Path $WorkspaceRoot "scripts\dev\start-llama-server.ps1"
$startScriptContent | Set-Content -Path $startScriptPath -Encoding UTF8

Write-Success "Start script created: $startScriptPath"

# Summary
Write-Info ""
Write-Info "=========================================="
Write-Info "Llama.cpp Setup Complete!"
Write-Info "=========================================="
Write-Info ""
Write-Info "Installation Summary:"
Write-Info "  ✅ Llama.cpp binaries installed"
if (!$SkipModelDownload) {
    Write-Info "  ✅ Model downloaded ($ModelSize)"
}
Write-Info "  ✅ Configuration created"
Write-Info "  ✅ Start script created"
Write-Info ""
Write-Info "Next steps:"
Write-Info "  1. Start server: .\scripts\dev\start-llama-server.ps1"
Write-Info "  2. Test server: .\scripts\testing\test-llama-server.ps1"
Write-Info "  3. Review config: $configPath"
Write-Info ""
Write-Info "Server will run on: http://127.0.0.1:8080"
Write-Info "API docs: http://127.0.0.1:8080/docs"
