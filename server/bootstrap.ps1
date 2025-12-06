param(
    [switch]$NoRun,
    [switch]$BuildOnly,
    [switch]$Help
)

if ($Help) {
    Write-Host "NOA ARK OS - Server Bootstrap (Windows)" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage:" -ForegroundColor Yellow
    Write-Host "  .\server\bootstrap.ps1             # Build and run unified server" 
    Write-Host "  .\server\bootstrap.ps1 -NoRun      # Build only"
    Write-Host "  .\server\bootstrap.ps1 -BuildOnly  # Build only"
    Write-Host ""
    Write-Host "Notes:"
    Write-Host "  - Uses tools/devshell/portable-cargo.ps1 if available, otherwise system 'cargo'."
    Write-Host "  - Intended as a convenience helper after cloning/moving the workspace."
    exit 0
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$WorkspaceRoot = Split-Path -Parent $ScriptDir
$ServerRoot = Join-Path $WorkspaceRoot "server"

Write-Host ""
Write-Host "[*] NOA ARK OS - Server bootstrap (Windows)" -ForegroundColor Cyan
Write-Host "[*] Workspace root: $WorkspaceRoot"
Write-Host ""

function Get-CargoRunner {
    $wrapper = Join-Path $WorkspaceRoot "tools/devshell/portable-cargo.ps1"
    if (Test-Path $wrapper) {
        return $wrapper
    }
    return "cargo"
}

function Build-Server {
    Ensure-CargoAvailable
    $cargo = Get-CargoRunner
    Write-Host "[*] Using Cargo runner: $cargo"
    Write-Host "[*] Building noa-unified-server (first run may take a while)..." -ForegroundColor Yellow

    Push-Location $WorkspaceRoot
    try {
        & $cargo build --manifest-path "$ServerRoot\Cargo.toml" --bin noa-unified-server
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Cargo build failed with exit code $LASTEXITCODE"
            exit $LASTEXITCODE
        }
    }
    finally {
        Pop-Location
    }

    Write-Host "[ok] Build complete" -ForegroundColor Green
}

function Run-Server {
    Ensure-CargoAvailable
    $cargo = Get-CargoRunner
    Write-Host "[*] Launching noa-unified-server..." -ForegroundColor Yellow
    Write-Host ""

    Push-Location $WorkspaceRoot
    try {
        & $cargo run --manifest-path "$ServerRoot\Cargo.toml" --bin noa-unified-server
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Cargo run failed with exit code $LASTEXITCODE"
            exit $LASTEXITCODE
        }
    }
    finally {
        Pop-Location
    }
}

Build-Server

if (-not $NoRun -and -not $BuildOnly) {
    Run-Server
} else {
    Write-Host "[*] Skipping server run (per flags)." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "[ok] Server bootstrap complete." -ForegroundColor Green
Write-Host "Workspace root: $WorkspaceRoot"
Write-Host "Server root:    $ServerRoot"
Write-Host ""
function Ensure-CargoAvailable {
    $wrapper = Join-Path $WorkspaceRoot "tools/devshell/portable-cargo.ps1"
    if (Test-Path $wrapper) {
        return
    }
    try {
        Get-Command cargo -ErrorAction Stop | Out-Null
        return
    } catch {
        Write-Error "No Cargo runner found. Install Rust (rustup) or configure the portable toolchain (see server/tools/README.md)."
        exit 1
    }
}

