#!/usr/bin/env pwsh
param(
    [string]$Version = $env:CADDY_VERSION
)
if (-not $Version) { $Version = '2.8.4' }

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$caddyRoot = Join-Path $scriptDir 'caddy-portable'
$artifactDir = Join-Path $caddyRoot 'artifacts'
New-Item -ItemType Directory -Path $artifactDir -Force | Out-Null

$platform = if ($IsWindows) { 'windows' } elseif ($IsMacOS) { 'mac' } else { 'linux' }
$archiveExt = if ($platform -eq 'windows') { 'zip' } else { 'tar.gz' }
$archiveName = "caddy_${Version}_${platform}_amd64.$archiveExt"
$archivePath = Join-Path $artifactDir $archiveName
$downloadUrl = "https://github.com/caddyserver/caddy/releases/download/v$Version/$archiveName"
$extractDir = Join-Path $caddyRoot "caddy_${Version}_${platform}_amd64"

if (-not (Test-Path $archivePath)) {
    Write-Host "⬇️  Downloading Caddy $Version for $platform..."
    Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath
} else {
    Write-Host "ℹ️  Reusing $archivePath"
}

if (Test-Path $extractDir) { Remove-Item $extractDir -Recurse -Force }
New-Item -ItemType Directory -Path $extractDir | Out-Null
if ($archiveExt -eq 'zip') {
    Expand-Archive -Path $archivePath -DestinationPath $extractDir -Force
} else {
    tar -xzf $archivePath -C $extractDir
}

$nested = Join-Path $extractDir "caddy_${Version}_${platform}_amd64"
if (Test-Path $nested) {
    Get-ChildItem $nested | Move-Item -Destination $extractDir
    Remove-Item $nested -Recurse -Force
}

$binary = Join-Path $extractDir (if ($platform -eq 'windows') { 'caddy.exe' } else { 'caddy' })
if (-not (Test-Path $binary)) { throw "Caddy binary missing" }

$manifest = [ordered]@{
    version = $Version
    target_os = $platform
    archive = $archiveName
    archive_path = (Resolve-Path $archivePath).Path
    caddy_binary = (Resolve-Path $binary).Path
    generated_at = (Get-Date).ToUniversalTime().ToString('o')
}
$manifestPath = Join-Path $caddyRoot 'manifest.json'
$manifest | ConvertTo-Json | Set-Content -Path $manifestPath
Copy-Item $manifestPath (Join-Path $scriptDir 'caddy-portable.manifest.json') -Force

Remove-Item (Join-Path $caddyRoot 'current') -Force -ErrorAction SilentlyContinue
try {
    New-Item -ItemType SymbolicLink -Path (Join-Path $caddyRoot 'current') -Target $extractDir -ErrorAction Stop | Out-Null
} catch {
    if ($IsWindows) {
        # Fall back to junction if symlink fails (non-admin)
        cmd /c mklink /J (Join-Path $caddyRoot 'current') $extractDir | Out-Null
    } else {
        throw "Failed to create symbolic link: $($_.Exception.Message)"
    }
}

Write-Host "✅ Portable Caddy ready at $extractDir"
