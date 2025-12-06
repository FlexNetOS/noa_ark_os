# Generate Documentation for NOA ARK OS
# This script generates comprehensive documentation from Rust code comments

param(
    [switch]$OpenBrowser,
    [switch]$Clean,
    [string]$OutputDir = "docs/api"
)

Write-Host "🔧 NOA ARK OS Documentation Generator" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan

# Ensure we're in the right directory
$workspaceRoot = Split-Path -Parent $PSScriptRoot
Set-Location $workspaceRoot

# Clean previous docs if requested
if ($Clean) {
    Write-Host "🧹 Cleaning previous documentation..." -ForegroundColor Yellow
    if (Test-Path $OutputDir) {
        Remove-Item -Recurse -Force $OutputDir
    }
    if (Test-Path "target/doc") {
        Remove-Item -Recurse -Force "target/doc"
    }
}

# Activate portable Cargo if available
$cargoScript = ".\server\tools\activate-cargo.ps1"
if (Test-Path $cargoScript) {
    Write-Host "🔨 Activating portable Cargo..." -ForegroundColor Green
    & $cargoScript
}

# Generate documentation
Write-Host "📚 Generating Rust documentation..." -ForegroundColor Green
& cargo doc --workspace --no-deps --document-private-items

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Documentation generated successfully!" -ForegroundColor Green

    # Copy docs to output directory
    if (!(Test-Path $OutputDir)) {
        New-Item -ItemType Directory -Path $OutputDir | Out-Null
    }

    Write-Host "📋 Copying documentation to $OutputDir..." -ForegroundColor Blue
    Copy-Item -Recurse -Force "target/doc/*" $OutputDir

    # Generate additional docs
    Write-Host "📝 Generating additional documentation..." -ForegroundColor Blue

    # Generate crate documentation index
    $indexPath = Join-Path $OutputDir "index.html"
    $indexContent = @"
<!DOCTYPE html>
<html>
<head>
    <title>NOA ARK OS API Documentation</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #2c3e50; }
        .crate-list { margin: 20px 0; }
        .crate-item { margin: 10px 0; padding: 10px; border-left: 4px solid #3498db; background: #f8f9fa; }
        .crate-link { color: #3498db; text-decoration: none; font-weight: bold; }
        .crate-link:hover { text-decoration: underline; }
        .timestamp { color: #7f8c8d; font-size: 0.9em; }
    </style>
</head>
<body>
    <h1>🚀 NOA ARK OS API Documentation</h1>
    <p>Comprehensive API documentation for the NOA ARK OS platform components.</p>

    <div class="crate-list">
        <h2>Core Components</h2>
        <div class="crate-item">
            <a href="noa_core/index.html" class="crate-link">noa_core</a>
            <p>Core OS functionality including kernel, gateway, and system services</p>
        </div>
        <div class="crate-item">
            <a href="noa_agents/index.html" class="crate-link">noa_agents</a>
            <p>Agent factory with hive mind and swarm capabilities</p>
        </div>
        <div class="crate-item">
            <a href="noa_crc/index.html" class="crate-link">noa_crc</a>
            <p>Continuous ReCode system for AI-supervised adaptation</p>
        </div>
        <div class="crate-item">
            <a href="noa_cicd/index.html" class="crate-link">noa_cicd</a>
            <p>CI/CD pipeline with CRC integration</p>
        </div>
    </div>

    <div class="timestamp">
        Generated on: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
    </div>
</body>
</html>
"@

    $indexContent | Out-File -FilePath $indexPath -Encoding UTF8

    Write-Host "📖 Documentation index created at $indexPath" -ForegroundColor Green

    # Open browser if requested
    if ($OpenBrowser) {
        Write-Host "🌐 Opening documentation in browser..." -ForegroundColor Magenta
        Start-Process $indexPath
    }

    Write-Host ""
    Write-Host "📚 Documentation Summary:" -ForegroundColor Cyan
    Write-Host "  - Location: $OutputDir" -ForegroundColor White
    Write-Host "  - Index: $indexPath" -ForegroundColor White
    Write-Host "  - Open in browser: .\scripts\generate-docs.ps1 -OpenBrowser" -ForegroundColor White

} else {
    Write-Host "❌ Documentation generation failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "🎉 Documentation generation complete!" -ForegroundColor Green