# NOA ARK OS - Quick Start Script (Windows)

Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Blue
Write-Host "║         NOA ARK OS - Quick Start                           ║" -ForegroundColor Blue
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Blue
Write-Host ""

Write-Host "This script will:" -ForegroundColor Cyan
Write-Host "  1. Verify prerequisites"
Write-Host "  2. Build all components"
Write-Host "  3. Run tests"
Write-Host "  4. Execute the full system demo"
Write-Host ""
Read-Host "Press Enter to continue"
Write-Host ""

# Check Rust
Write-Host "[1/4] Checking prerequisites..." -ForegroundColor Cyan
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Rust not found. Please install from: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}
$cargoVersion = cargo --version
Write-Host "✓ Rust toolchain found: $cargoVersion" -ForegroundColor Green
Write-Host ""

# Build
Write-Host "[2/4] Building NOA ARK OS..." -ForegroundColor Cyan
cargo build --release --workspace
Write-Host "✓ Build completed" -ForegroundColor Green
Write-Host ""

# Test
Write-Host "[3/4] Running tests..." -ForegroundColor Cyan
cargo test --workspace --release
Write-Host "✓ All tests passed" -ForegroundColor Green
Write-Host ""

# Run demo
Write-Host "[4/4] Running full system demo..." -ForegroundColor Cyan
Write-Host ""
cargo run --release --example full_system_demo
Write-Host ""

Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Blue
Write-Host "║                   Quick Start Complete!                    ║" -ForegroundColor Blue
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Blue
Write-Host ""
Write-Host "Next steps:"
Write-Host "  • Run the kernel: cargo run --release --bin noa_kernel"
Write-Host "  • Read the docs: Get-Content docs\GETTING_STARTED.md"
Write-Host "  • Explore examples: Get-ChildItem examples\"
Write-Host ""
