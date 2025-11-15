# NOA ARK OS - Setup Portable Cargo (One-Time)
# This script downloads and installs Cargo as a portable tool

param(
    [string]$WorkspaceRoot = "D:\dev\workspaces\noa_ark_os"
)

Write-Host "🚀 NOA ARK OS - Portable Cargo Setup" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray

# Define paths
$CARGO_HOME = "$WorkspaceRoot\server\tools\cargo-portable"
$RUSTUP_HOME = "$WorkspaceRoot\server\tools\rustup-portable"

Write-Host "`n📂 Creating directories..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "$CARGO_HOME\bin" | Out-Null
New-Item -ItemType Directory -Force -Path "$CARGO_HOME\lib" | Out-Null
New-Item -ItemType Directory -Force -Path $RUSTUP_HOME | Out-Null

Write-Host "✅ Directories created" -ForegroundColor Green

# Check if already installed
if (Test-Path "$CARGO_HOME\bin\cargo.exe") {
    Write-Host "`n⚠️  Cargo already installed at:" -ForegroundColor Yellow
    Write-Host "   $CARGO_HOME\bin\cargo.exe" -ForegroundColor Gray
    $response = Read-Host "`nReinstall? (y/N)"
    if ($response -ne 'y') {
        Write-Host "Skipping installation." -ForegroundColor Gray
        exit 0
    }
}

# Set environment variables
$env:CARGO_HOME = $CARGO_HOME
$env:RUSTUP_HOME = $RUSTUP_HOME

Write-Host "`n📥 Downloading rustup-init..." -ForegroundColor Yellow
try {
    Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
    Write-Host "✅ Downloaded rustup-init.exe" -ForegroundColor Green
} catch {
    Write-Host "❌ Failed to download rustup-init: $_" -ForegroundColor Red
    exit 1
}

Write-Host "`n🔧 Installing Rust (portable)..." -ForegroundColor Yellow
Write-Host "   This may take a few minutes..." -ForegroundColor Gray

# Install Rust to portable location
.\rustup-init.exe --default-toolchain stable --profile minimal --no-modify-path -y

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Rust installed successfully!" -ForegroundColor Green
} else {
    Write-Host "❌ Installation failed!" -ForegroundColor Red
    Remove-Item rustup-init.exe -ErrorAction SilentlyContinue
    exit 1
}

# Cleanup
Remove-Item rustup-init.exe -ErrorAction SilentlyContinue

Write-Host "`n🔍 Verifying installation..." -ForegroundColor Yellow

# Add to PATH temporarily for verification
$env:Path = "$CARGO_HOME\bin;$env:Path"

$cargoVersion = & "$CARGO_HOME\bin\cargo.exe" --version
$rustcVersion = & "$CARGO_HOME\bin\rustc.exe" --version

Write-Host "✅ Verification successful!" -ForegroundColor Green
Write-Host "   Cargo: $cargoVersion" -ForegroundColor Gray
Write-Host "   Rustc: $rustcVersion" -ForegroundColor Gray

Write-Host "`n📝 Creating activation script..." -ForegroundColor Yellow

# Create activation script
$activateScript = @"
# NOA ARK OS - Activate Portable Cargo
# Usage: .\server\tools\activate-cargo.ps1

`$WORKSPACE_ROOT = "$WorkspaceRoot"
`$CARGO_HOME = "`$WORKSPACE_ROOT\server\tools\cargo-portable"
`$RUSTUP_HOME = "`$WORKSPACE_ROOT\server\tools\rustup-portable"

# Set environment variables
`$env:CARGO_HOME = `$CARGO_HOME
`$env:RUSTUP_HOME = `$RUSTUP_HOME
`$env:Path = "`$CARGO_HOME\bin;`$env:Path"

Write-Host "🚀 Portable Cargo Activated!" -ForegroundColor Green
Write-Host "CARGO_HOME: `$CARGO_HOME" -ForegroundColor Cyan
Write-Host "RUSTUP_HOME: `$RUSTUP_HOME" -ForegroundColor Cyan
Write-Host ""

# Verify
cargo --version
rustc --version

Write-Host "`n💡 Tip: You can now run 'cargo build' in the crc/ directory" -ForegroundColor Gray
"@

$activateScript | Out-File -FilePath "$WorkspaceRoot\server\tools\activate-cargo.ps1" -Encoding UTF8

Write-Host "✅ Activation script created" -ForegroundColor Green

Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host "🎉 Setup Complete!" -ForegroundColor Green
Write-Host "`nTo use portable Cargo, run:" -ForegroundColor Yellow
Write-Host "   .\server\tools\activate-cargo.ps1" -ForegroundColor Cyan
Write-Host "`nThen navigate to crc/ and run:" -ForegroundColor Yellow
Write-Host "   cargo build" -ForegroundColor Cyan
Write-Host "`n📁 Installation location:" -ForegroundColor Gray
Write-Host "   $CARGO_HOME" -ForegroundColor Gray
Write-Host ""
