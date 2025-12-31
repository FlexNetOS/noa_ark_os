param(
    [switch]$Silent
)

if ($env:NOA_ACTIVATE_SILENT -eq "1") {
    $Silent = $true
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$nodeRoot = Join-Path $scriptDir "node-portable"
$current = Join-Path $nodeRoot "current"
$corepackHome = Join-Path $nodeRoot "corepack"

if (-not (Test-Path $nodeRoot)) {
    Write-Error "Portable Node directory not found at $nodeRoot. Run ./server/tools/setup-portable-node.ps1 first."
    exit 1
}

$nodeBin = Join-Path $current "bin"
if (-not (Test-Path $nodeBin)) {
    # Windows zip places binaries at the root
    if (Test-Path (Join-Path $current "node.exe")) {
        $nodeBin = $current
    } else {
        Write-Error "Portable Node runtime missing at $current. Run ./server/tools/setup-portable-node.ps1 to download Node v20.19.5."
        exit 1
    }
}

$env:NOA_NODE_HOME = $current
$env:NOA_NODE_PORTABLE_ROOT = $nodeRoot
$env:COREPACK_HOME = $corepackHome

if (-not ($env:PATH -split ';' | Where-Object { $_ -eq $nodeBin })) {
    $env:PATH = ($nodeBin + ';' + $env:PATH)
}

$env:NOA_NODE_ENV = "1"
$env:NOA_DEV_ENV = "1"

if (-not $Silent) {
    $nodeVersion = & "$nodeBin/node" -v 2>$null
    $pnpmVersion = & "$nodeBin/pnpm" -v 2>$null
    Write-Host "`n🟢 Portable Node activated"
    Write-Host "   NODE_HOME = $($env:NOA_NODE_HOME)"
    Write-Host "   COREPACK  = $($env:COREPACK_HOME)"
    Write-Host "   node      = $nodeVersion"
    Write-Host "   pnpm      = $pnpmVersion`n"
}
