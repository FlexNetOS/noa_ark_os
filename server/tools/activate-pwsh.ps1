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
$requestedPlatform = $env:NOA_PWSH_PLATFORM

function Write-Info {
    param([string]$Message)
    if ($Silent) { return }
    Write-Host $Message
}

function Resolve-ManifestPlatform {
    param([string]$Manifest, [string]$Binary)
    if (-not (Test-Path $Manifest)) { return $null }
    try {
        $data = Get-Content -Path $Manifest -Raw | ConvertFrom-Json -ErrorAction Stop
    } catch {
        return $null
    }
    if ($data.platforms) {
        foreach ($entry in $data.platforms) {
            if (-not $entry.binary) { continue }
            $abs = Resolve-Path -Path (Join-Path (Split-Path -Parent $Manifest) $entry.binary)
            if ($abs -and $abs.ToString().TrimEnd([System.IO.Path]::DirectorySeparatorChar) -eq $Binary.TrimEnd([System.IO.Path]::DirectorySeparatorChar)) {
                return $entry.platform
            }
        }
    }
    return $data.platform ?? $data.default_platform
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

$pwshCandidates = New-Object System.Collections.Generic.List[string]
if ($requestedPlatform) {
    $platformBin = Join-Path $binDir $requestedPlatform
    $pwshCandidates.Add((Join-Path $platformBin "pwsh.exe"))
    $pwshCandidates.Add((Join-Path $platformBin "pwsh"))
}
$pwshCandidates.Add((Join-Path $binDir "pwsh.exe"))
$pwshCandidates.Add((Join-Path $binDir "pwsh"))
$pwshCandidates.Add((Join-Path $currentLink "pwsh.exe"))
$pwshCandidates.Add((Join-Path $currentLink "pwsh"))

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
$resolvedPlatform = Resolve-ManifestPlatform -Manifest $manifestPath -Binary $pwshPath
if ($resolvedPlatform) { $env:NOA_PWSH_PLATFORM_RESOLVED = $resolvedPlatform }
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
    if ($resolvedPlatform) {
        Write-Info "   platform     = $resolvedPlatform"
    }
    try {
        & $pwshPath --version | Write-Info
    } catch {}
    Write-Info ""
}
