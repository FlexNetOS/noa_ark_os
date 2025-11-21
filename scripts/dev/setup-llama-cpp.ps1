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

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$hostIsWindows = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)
$hostIsLinux = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Linux)
$hostIsMac = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::OSX)

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

function Test-CudaToolkit {
    if ($hostIsWindows) {
        return Test-Path "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA"
    }
    if ($hostIsLinux) {
        if (Test-Path "/usr/local/cuda") { return $true }
        try {
            $null = Get-Command nvcc -ErrorAction Stop
            return $true
        } catch {
            return $false
        }
    }
    return $false
}

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
    if ($hostIsWindows) {
        Write-Warning "Building on Windows requires CMake + Visual Studio Build Tools"
    } elseif ($hostIsLinux -or $hostIsMac) {
        Write-Warning "Ensure build-essential/clang and cmake are installed before continuing"
    }
    
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
    $hasCuda = Test-CudaToolkit
    
    $cmakeArgs = @("-DCMAKE_BUILD_TYPE=Release")
    if ($hasCuda) {
        Write-Info "CUDA detected, building with GPU support..."
        $cmakeArgs += "-DGGML_CUDA=ON"
    } else {
        Write-Info "Building CPU-only version..."
    }
    & cmake .. @cmakeArgs

    $buildArgs = @("--build", ".", "--config", "Release")
    $jobs = [Math]::Max([Environment]::ProcessorCount, 1)
    if (-not $hostIsWindows) {
        $buildArgs += @("--", "-j$jobs")
    }
    & cmake @buildArgs
    
    # Copy binaries
    Write-Info "Copying binaries..."
    Copy-Item "bin/Release/*" (Join-Path $LlamaCppDir "bin") -Force
    
    Write-Success "Llama.cpp built from source"
    
} else {
    Write-Info "Provisioning llama.cpp binaries..."
    $arch = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "x86" }
    if ($arch -ne "x64") {
        Write-Warning "Only x64 hosts are validated for prebuilt binaries."
    }
    $llamaCppVersion = "b4315"
    $releaseBaseUrl = "https://github.com/ggml-org/llama.cpp/releases/download/$llamaCppVersion"
    $assetName = if ($hostIsWindows) {
        $hasCuda = Test-CudaToolkit
        if ($hasCuda) {
            "llama-$llamaCppVersion-bin-win-cuda-cu12.4-$arch.zip"
        } else {
            "llama-$llamaCppVersion-bin-win-avx2-$arch.zip"
        }
    } elseif ($hostIsLinux) {
        "llama-$llamaCppVersion-bin-ubuntu-x64.zip"
    } else {
        throw "Unsupported OS for prebuilt llama.cpp; build from source."
    }
    $downloadUrl = "$releaseBaseUrl/$assetName"
    $zipPath = Join-Path $ServerAIDir "llama-cpp.zip"
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath
        Write-Success "Downloaded llama.cpp"
        Write-Info "Expanding $assetName into bin/ ..."
        $binRoot = Join-Path $LlamaCppDir "bin"
        Expand-Archive -Path $zipPath -DestinationPath $binRoot -Force
        Remove-Item $zipPath
        # Flatten nested bin directories so callers can rely on bin/llama-server*
        $nestedBins = Get-ChildItem -Path $binRoot -Directory -Recurse | Where-Object { $_.Name -eq "bin" -and $_.FullName -ne $binRoot }
        foreach ($nested in $nestedBins) {
            Get-ChildItem -Path $nested.FullName -File -ErrorAction SilentlyContinue | ForEach-Object {
                Copy-Item $_.FullName (Join-Path $binRoot $_.Name) -Force
            }
        }
        if ($hostIsLinux) {
            Get-ChildItem -Recurse -Path $binRoot -Filter "llama*" | ForEach-Object {
                chmod +x $_.FullName 2>$null
            }
        }
        Write-Success "Llama.cpp binaries installed"
    } catch {
        Write-Error "Failed to download: $($_.Exception.Message)"
        Write-Warning "Try building from source with -BuildFromSource"
        exit 1
    }
}

# Step 2: Download model
$modelUrl = $null
$modelFileName = $null
switch ($ModelSize) {
    "3b" {
        $modelUrl = "https://huggingface.co/QuantFactory/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct.Q4_K_M.gguf"
        $modelFileName = "Llama-3.2-3B-Instruct.Q4_K_M.gguf"
    }
    "7b" {
        $modelUrl = "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf"
        $modelFileName = "mistral-7b-instruct-v0.2.Q4_K_M.gguf"
    }
    "8b" {
        $modelUrl = "https://huggingface.co/QuantFactory/Meta-Llama-3.1-8B-Instruct-GGUF/resolve/main/Meta-Llama-3.1-8B-Instruct.Q4_K_M.gguf"
        $modelFileName = "Meta-Llama-3.1-8B-Instruct.Q4_K_M.gguf"
    }
    default {
        Write-Error "Invalid model size: $ModelSize. Use 3b, 7b, or 8b"
        exit 1
    }
}

if (!$SkipModelDownload) {
    Write-Info "Downloading model..."
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

# Build start script template without expanding runtime variables
$startScriptTemplate = @'
# Start Llama.cpp Server
param(
    [Parameter(Mandatory=$false)]
    [string]$ModelFile = "__DEFAULT_MODEL_FILE__",
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
$workspaceRoot = [System.IO.Path]::GetFullPath((Join-Path $scriptRoot "../.."))
$hostIsWindows = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)
$serverBinary = if ($hostIsWindows) { "llama-server.exe" } else { "llama-server" }
$serverDir = Join-Path $workspaceRoot "server"
$serverAiDir = Join-Path $serverDir "ai"
$llamaCppDir = Join-Path $serverAiDir "llama-cpp"
$binDir = Join-Path $llamaCppDir "bin"
$binPath = Join-Path $binDir $serverBinary
$modelsDir = Join-Path $llamaCppDir "models"
$modelPath = Join-Path $modelsDir $ModelFile
$logDir = Join-Path $llamaCppDir "logs"
$logPath = Join-Path $logDir "server.log"

Write-Host "Starting Llama.cpp server..." -ForegroundColor Cyan
Write-Host "Model: $modelPath" -ForegroundColor Gray
Write-Host "Server: http://${Host}:$Port" -ForegroundColor Gray

if (!(Test-Path $modelPath)) {
    Write-Host "ERROR: Model not found at $modelPath" -ForegroundColor Red
    Write-Host "Available models:" -ForegroundColor Yellow
    Get-ChildItem $modelsDir -Filter "*.gguf" | ForEach-Object { Write-Host "  - $($_.Name)" }
    exit 1
}

if (!(Test-Path $binPath)) {
    Write-Host "ERROR: Llama server binary missing at $binPath"
    exit 1
}

if (-not $hostIsWindows) {
    try {
        & chmod +x -- $binPath 2>$null
    } catch {
        Write-Host "Warning: Unable to mark $binPath executable ($_)." -ForegroundColor Yellow
    }
}

& $binPath `
    --model $modelPath `
    --host $Host `
    --port $Port `
    --ctx-size 8192 `
    --batch-size 512 `
    --threads $Threads `
    --n-gpu-layers $GpuLayers `
    --log-file $logPath

Write-Host "Server stopped" -ForegroundColor Yellow
'@

$scriptsDir = Join-Path $WorkspaceRoot "scripts"
$devScriptsDir = Join-Path $scriptsDir "dev"
$startScriptPath = Join-Path $devScriptsDir "start-llama-server.ps1"
$startScriptContent = $startScriptTemplate.Replace("__DEFAULT_MODEL_FILE__", $modelFileName)
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
