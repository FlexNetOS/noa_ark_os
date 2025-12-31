# CUDA Toolkit Setup for NOA ARK OS
# Installs or verifies CUDA 13.0.1 on Windows or Linux using portable PowerShell

param(
    [switch]$Download,
    [switch]$Install,
    [switch]$Verify,
    [string]$WorkspaceRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$cudaVersion = "13.0.1"
$isWindows = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)
$isLinux = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Linux)

if (-not $WorkspaceRoot) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $WorkspaceRoot = Resolve-Path -Path (Join-Path $scriptRoot "..\..")
}

$toolsDir = Join-Path $WorkspaceRoot "server/tools/cuda"
$null = New-Item -ItemType Directory -Force -Path $toolsDir

if (-not ($isWindows -or $isLinux)) {
    Write-Warning "Unsupported host platform for CUDA bootstrap. Only Windows and Linux are covered."
    return
}

$installerName = if ($isWindows) { "cuda_${cudaVersion}_windows.exe" } else { "cuda_${cudaVersion}_linux.run" }
$cudaUrl = if ($isWindows) {
    "https://developer.download.nvidia.com/compute/cuda/$cudaVersion/local_installers/cuda_${cudaVersion}_531.14_windows.exe"
} else {
    "https://developer.download.nvidia.com/compute/cuda/$cudaVersion/local_installers/cuda_${cudaVersion}_550.54.15_linux.run"
}
$installerPath = Join-Path $toolsDir $installerName

Write-Host "🚀 CUDA Toolkit Setup ($cudaVersion)" -ForegroundColor Cyan
Write-Host "Host: $([System.Environment]::OSVersion.VersionString)" -ForegroundColor DarkGray
Write-Host ""

function Test-GpuPresence {
    Write-Host "📊 Detecting NVIDIA GPUs..." -ForegroundColor Cyan
    try {
        $result = & nvidia-smi --query-gpu=name,memory.total --format=csv,noheader 2>&1
        if ($LASTEXITCODE -eq 0 -and $result) {
            $result | ForEach-Object { Write-Host "  ✅ $_" -ForegroundColor Green }
            return $true
        }
        Write-Warning "nvidia-smi reported no devices or is unavailable."
    } catch {
        Write-Warning "nvidia-smi not found. Install NVIDIA drivers before CUDA."
    }
    return $false
}

function Invoke-Download {
    param([string]$Url,[string]$Destination)
    Write-Host "📥 Downloading $Url" -ForegroundColor Cyan
    Write-Host "    -> $Destination"
    try {
        Invoke-WebRequest -Uri $Url -OutFile $Destination -UseBasicParsing
        Write-Host "✅ Download complete" -ForegroundColor Green
    } catch {
        Write-Warning "Download failed: $($_.Exception.Message)"
        throw
    }
}

function Install-OnWindows {
    param([string]$Installer)
    Write-Host "📦 Installing CUDA Toolkit (silent subset)" -ForegroundColor Cyan
    $args = @(
        "-s",
        "nvcc_13.0",
        "cudart_13.0",
        "cublas_13.0"
    )
    Start-Process -FilePath $Installer -ArgumentList $args -Wait -NoNewWindow
    Write-Host "✅ CUDA installer finished" -ForegroundColor Green
}

function Install-OnLinux {
    param([string]$Installer)
    Write-Host "📦 Preparing CUDA Toolkit installer" -ForegroundColor Cyan
    if (-not (Test-Path $Installer)) {
        throw "Installer missing: $Installer"
    }
    if (-not ([bool](Get-Command "bash" -ErrorAction SilentlyContinue))) {
        throw "bash not found; required to run the .run installer"
    }
    $uid = & id -u 2>$null
    if (-not $uid) {
        Write-Warning "Unable to determine user id; skipping installer. Run 'sudo pwsh ... -Install'."
        return
    }
    if ([int]$uid -ne 0) {
        Write-Warning "CUDA installer requires root privileges. Re-run with sudo pwsh ... -Install to proceed."
        return
    }
    chmod +x -- $Installer | Out-Null
    & bash $Installer --silent --toolkit
    Write-Host "✅ CUDA installer finished" -ForegroundColor Green
}

function Verify-Cuda {
    if ($isWindows) {
        $cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.0"
        if (Test-Path $cudaPath) {
            Write-Host "✅ CUDA found at $cudaPath" -ForegroundColor Green
            $nvccPath = Join-Path $cudaPath "bin\nvcc.exe"
            if (Test-Path $nvccPath) {
                Write-Host "nvcc version:" -ForegroundColor Gray
                & $nvccPath --version | Select-Object -First 5
            }
        } else {
            Write-Warning "CUDA directory not found at $cudaPath"
        }
    } else {
        try {
            $nvcc = Get-Command nvcc -ErrorAction Stop
            Write-Host "✅ nvcc detected at $($nvcc.Source)" -ForegroundColor Green
            & $nvcc.Source --version
        } catch {
            Write-Warning "nvcc not on PATH. Add /usr/local/cuda/bin after installing."
        }
        if (Test-Path "/usr/local/cuda/lib64") {
            Write-Host "✅ /usr/local/cuda present" -ForegroundColor Green
        } else {
            Write-Warning "/usr/local/cuda missing; run Install step or create symlink."
        }
    }
}

$gpuDetected = Test-GpuPresence

if ($Download) {
    Invoke-Download -Url $cudaUrl -Destination $installerPath
}

if ($Install) {
    if (-not (Test-Path $installerPath)) {
        throw "Installer not found at $installerPath. Run with -Download first."
    }
    if ($isWindows) {
        Install-OnWindows -Installer $installerPath
    } else {
        Install-OnLinux -Installer $installerPath
    }
}

if ($Verify -or (-not $Download -and -not $Install)) {
    Write-Host "🔍 Verifying CUDA toolchain" -ForegroundColor Cyan
    if (-not $gpuDetected) {
        Write-Warning "GPU detection failed earlier; continuing with filesystem checks."
    }
    Verify-Cuda
}

Write-Host ""
Write-Host "🎯 Next Steps" -ForegroundColor Cyan
Write-Host "  1. Ensure CUDA bin/lib directories are on PATH/LD_LIBRARY_PATH"
Write-Host "  2. Rebuild llama.cpp with GPU support"
Write-Host "  3. Configure llama server GPU layers (e.g. 35)"
Write-Host ""
