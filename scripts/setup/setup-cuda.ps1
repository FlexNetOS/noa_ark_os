# CUDA Toolkit Setup for NOA ARK OS
# Installs CUDA 13.0.1 for GPU acceleration

param(
    [switch]$Download,
    [switch]$Install,
    [switch]$Verify
)

$CudaVersion = "13.0.1"
$CudaUrl = "https://developer.download.nvidia.com/compute/cuda/13.0.1/local_installers/cuda_13.0.1_531.14_windows.exe"
$ToolsDir = "D:\dev\workspaces\noa_ark_os\server\tools\cuda"
$InstallerPath = Join-Path $ToolsDir "cuda_13.0.1_windows.exe"

Write-Host "🚀 CUDA Toolkit Setup" -ForegroundColor Cyan
Write-Host "=" * 70
Write-Host ""

# Detect GPUs
Write-Host "📊 Detecting GPUs..." -ForegroundColor Cyan
$gpus = nvidia-smi --query-gpu=name,memory.total --format=csv,noheader 2>&1
if ($gpus -notmatch "not recognized") {
    $gpus | ForEach-Object { Write-Host "  ✅ $_" -ForegroundColor Green }
    Write-Host ""
} else {
    Write-Host "  ❌ No NVIDIA GPUs detected" -ForegroundColor Red
    exit 1
}

if ($Download) {
    Write-Host "📥 Downloading CUDA $CudaVersion..." -ForegroundColor Cyan
    Write-Host "  URL: $CudaUrl"
    Write-Host "  Destination: $InstallerPath"
    Write-Host ""
    
    # Create directory
    if (-not (Test-Path $ToolsDir)) {
        New-Item -ItemType Directory -Path $ToolsDir -Force | Out-Null
    }
    
    Write-Host "⏳ This is ~3 GB and may take a while..." -ForegroundColor Yellow
    try {
        Invoke-WebRequest -Uri $CudaUrl -OutFile $InstallerPath -UseBasicParsing
        Write-Host "✅ Download complete!" -ForegroundColor Green
    } catch {
        Write-Host "❌ Download failed: $_" -ForegroundColor Red
        exit 1
    }
}

if ($Install) {
    if (-not (Test-Path $InstallerPath)) {
        Write-Host "❌ Installer not found: $InstallerPath" -ForegroundColor Red
        Write-Host "   Run with -Download first" -ForegroundColor Yellow
        exit 1
    }
    
    Write-Host "📦 Installing CUDA Toolkit..." -ForegroundColor Cyan
    Write-Host "⏳ This may take 10-15 minutes..." -ForegroundColor Yellow
    Write-Host ""
    
    # Silent install with minimal components
    $installArgs = @(
        "-s",  # Silent
        "nvcc_13.0",  # Compiler
        "cudart_13.0",  # Runtime
        "cublas_13.0",  # Math library
        "visual_studio_integration_13.0"  # VS integration
    )
    
    try {
        Start-Process -FilePath $InstallerPath -ArgumentList $installArgs -Wait -NoNewWindow
        Write-Host "✅ Installation complete!" -ForegroundColor Green
    } catch {
        Write-Host "❌ Installation failed: $_" -ForegroundColor Red
        exit 1
    }
}

if ($Verify -or (-not $Download -and -not $Install)) {
    Write-Host "🔍 Verifying CUDA installation..." -ForegroundColor Cyan
    Write-Host ""
    
    # Check for CUDA in standard location
    $cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.0"
    
    if (Test-Path $cudaPath) {
        Write-Host "✅ CUDA found at: $cudaPath" -ForegroundColor Green
        
        # Check nvcc
        $nvccPath = Join-Path $cudaPath "bin\nvcc.exe"
        if (Test-Path $nvccPath) {
            Write-Host "✅ nvcc found" -ForegroundColor Green
            & $nvccPath --version | Select-Object -First 5
        }
        
        Write-Host ""
        Write-Host "📝 Add to PATH:" -ForegroundColor Cyan
        Write-Host "  $cudaPath\bin"
        Write-Host "  $cudaPath\libnvvp"
        Write-Host ""
        Write-Host "Or add to PowerShell profile:" -ForegroundColor Cyan
        Write-Host "  `$env:PATH += `";$cudaPath\bin;$cudaPath\libnvvp`""
        
    } else {
        Write-Host "⚠️  CUDA not found at standard location" -ForegroundColor Yellow
        Write-Host "   Expected: $cudaPath" -ForegroundColor Gray
        Write-Host ""
        Write-Host "💡 To install:" -ForegroundColor Cyan
        Write-Host "   .\scripts\setup\setup-cuda.ps1 -Download -Install"
    }
}

Write-Host ""
Write-Host "🎯 Next Steps:" -ForegroundColor Cyan
Write-Host "  1. Ensure CUDA is in PATH"
Write-Host "  2. Rebuild llama.cpp with GPU support"
Write-Host "  3. Set GPU layers in server config (35+ layers)"
Write-Host "  4. Restart llama.cpp server"
Write-Host ""
