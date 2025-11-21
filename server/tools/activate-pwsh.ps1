# NOA ARK OS - Activate Portable PowerShell runtime
param(
    [switch]$Silent
)

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path -Path (Join-Path $scriptDir "..\..")
$portableRoot = Join-Path $scriptDir "pwsh-portable"
$currentLink = Join-Path $portableRoot "current"
$binDir = Join-Path $portableRoot "bin"
$manifestPath = Join-Path $scriptDir "pwsh-portable.manifest.json"
$pwshVersion = $env:PWSH_VERSION
if (-not $pwshVersion) { $pwshVersion = "7.4.5" }

function Write-Info {
    param([string]$Message)
    if ($Silent) { return }
    Write-Host $Message
}

if ($env:NOA_PWSH_ENV) {
    Write-Info "ℹ️  Portable PowerShell already active ($($env:POWERSHELL_BIN))"
    return
}

if (-not (Test-Path $portableRoot)) {
    Write-Info "❌ Portable PowerShell directory missing: $portableRoot"
    Write-Info "   Run .\server\tools\setup-portable-pwsh.ps1 to provision v$pwshVersion."
    throw "Portable PowerShell missing"
}

$pwshCandidates = @(
    (Join-Path $binDir "pwsh.exe"),
    (Join-Path $binDir "pwsh"),
    (Join-Path $currentLink "pwsh.exe"),
    (Join-Path $currentLink "pwsh")
)

$pwshPath = $null
foreach ($cand in $pwshCandidates) {
    if (Test-Path $cand) {
        $pwshPath = Resolve-Path -Path $cand
        break
    }
}

if (-not $pwshPath) {
    Write-Info "❌ No pwsh executable found under $currentLink"
    Write-Info "   Run .\server\tools\setup-portable-pwsh.ps1 to provision v$pwshVersion."
    throw "PowerShell binary missing"
}

$env:POWERSHELL_BIN = $pwshPath
$env:NOA_PWSH_ENV = "1"
$env:NOA_PWSH_PORTABLE_ROOT = $portableRoot
$env:NOA_PWSH_MANIFEST = $manifestPath
$separator = [System.IO.Path]::PathSeparator
$prependPath = (Split-Path -Parent $pwshPath)
if ($env:PATH -notlike "$prependPath*") {
    $env:PATH = "$prependPath$separator$($env:PATH)"
}

if (-not $Silent) {
    Write-Info ""
    Write-Info "🟢 Portable PowerShell activated"
    Write-Info "   POWERSHELL_BIN = $pwshPath"
    if (Test-Path $manifestPath) {
        try {
            $sha256 = (Get-FileHash -Path $manifestPath -Algorithm SHA256).Hash
        } catch {
            $sha256 = "unavailable"
        }
        Write-Info "   manifest      = $manifestPath"
        Write-Info "   manifest sha  = $sha256"
    }
    try {
        & $pwshPath --version | Write-Info
    } catch {}
    Write-Info ""
}
