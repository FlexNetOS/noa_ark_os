# Setup Llama.cpp for NOA ARK OS
# Installs llama.cpp and prepares the inference server

param(
    [Parameter(Mandatory=$false)]
    [switch]$BuildFromSource = $false,
    
    [Parameter(Mandatory=$false)]
    [switch]$SkipModelDownload = $false,
    
    [Parameter(Mandatory=$false)]
    [string]$ModelSize = "3b",  # 3b, 7b, or 8b

    [Parameter(Mandatory=$false)]
    [string]$WorkspaceRoot
)

if (-not $WorkspaceRoot) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $WorkspaceRoot = Resolve-Path -Path (Join-Path $scriptRoot "..\..")
}

$ServerDir = Join-Path $WorkspaceRoot "server"
$ServerAIDir = Join-Path $ServerDir "ai"
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
    (Join-Path $LlamaCppDir "bin"),
    (Join-Path $LlamaCppDir "models"),
    (Join-Path $LlamaCppDir "configs"),
    (Join-Path $LlamaCppDir "logs")
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
    Copy-Item "bin/Release/*" (Join-Path $LlamaCppDir "bin") -Force
    
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
        
        Expand-Archive -Path $zipPath -DestinationPath (Join-Path $LlamaCppDir "bin") -Force
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
    $modelPath = Join-Path (Join-Path $LlamaCppDir "models") $modelFileName
    
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

$configPath = Join-Path (Join-Path $LlamaCppDir "configs") "server.yaml"
$configContent | Set-Content -Path $configPath -Encoding UTF8

Write-Success "Configuration created: $configPath"

# Step 4: Create start script
Write-Info "Creating start script..."

$startScriptContent = @"
# Start Llama.cpp Server
param(
    [Parameter(Mandatory=$false)]
    [string]$ModelFile = "$modelFileName",
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
$logDir = Join-Path $llamaCppDir "logs"
$logPath = Join-Path $logDir "server.log"

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
"@

$scriptsDir = Join-Path $WorkspaceRoot "scripts"
$devScriptsDir = Join-Path $scriptsDir "dev"
$startScriptPath = Join-Path $devScriptsDir "start-llama-server.ps1"
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
