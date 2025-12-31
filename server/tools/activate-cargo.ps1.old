param(
    [switch]$Silent
)

# NOA ARK OS - Activate Portable Cargo
# Usage: .\server\tools\activate-cargo.ps1 [-Silent]

# Get the script's directory to make it location-independent
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$WORKSPACE_ROOT = Split-Path -Parent (Split-Path -Parent $ScriptDir)

$CARGO_HOME = "$WORKSPACE_ROOT\server\tools\cargo-portable"
$RUSTUP_HOME = "$WORKSPACE_ROOT\server\tools\rustup-portable"

if (-not $Silent) {
    Write-Host "`n🔧 Activating Portable Cargo..." -ForegroundColor Cyan
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
}

# Verify installation exists
if (-not (Test-Path "$CARGO_HOME\bin\cargo.exe")) {
    Write-Host "`n❌ ERROR: Portable Cargo not found!" -ForegroundColor Red
    Write-Host "`nExpected location: $CARGO_HOME\bin\cargo.exe" -ForegroundColor Yellow
    Write-Host "`nPlease run setup first:" -ForegroundColor Yellow
    Write-Host "   .\server\tools\setup-portable-cargo.ps1" -ForegroundColor Cyan
    Write-Host ""
    exit 1
}

# Set environment variables for current session
$env:CARGO_HOME = $CARGO_HOME
$env:RUSTUP_HOME = $RUSTUP_HOME

# Prepend cargo bin to PATH (only if not already there)
if ($env:Path -notlike "*$CARGO_HOME\bin*") {
    $env:Path = "$CARGO_HOME\bin;$env:Path"
}

$env:NOA_CARGO_ENV = "1"

if (-not $Silent) {
    Write-Host "`n✅ Portable Cargo Activated Successfully!" -ForegroundColor Green
    Write-Host "`nEnvironment:" -ForegroundColor Yellow
    Write-Host "  CARGO_HOME   = $CARGO_HOME" -ForegroundColor Cyan
    Write-Host "  RUSTUP_HOME  = $RUSTUP_HOME" -ForegroundColor Cyan
    Write-Host "  PATH         = [cargo-portable\bin prepended]" -ForegroundColor Cyan

    Write-Host "`nVersions:" -ForegroundColor Yellow
    cargo --version
    rustc --version

    Write-Host "`n💡 Tips:" -ForegroundColor Gray
    Write-Host "  • Run 'cargo build' to build projects" -ForegroundColor Gray
    Write-Host "  • Run 'cargo run' to run projects" -ForegroundColor Gray
    Write-Host "  • Run 'cargo test' to run tests" -ForegroundColor Gray
    Write-Host "  • This activation is for the current PowerShell session only" -ForegroundColor Gray
    Write-Host ""
}
