param(
    [string]$NodeVersion = "20.19.5",
    [string]$PnpmVersion = "8.15.4"
)

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$nodeRoot = Join-Path $scriptDir "node-portable"
$artifactDir = Join-Path $nodeRoot "artifacts"
$manifestPath = Join-Path $nodeRoot "manifest.json"

New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null

$archiveName = "node-v$NodeVersion-win-x64.zip"
$archivePath = Join-Path $artifactDir $archiveName
$nodeUrl = "https://nodejs.org/dist/v$NodeVersion/$archiveName"

if (-not (Test-Path $archivePath)) {
    Write-Host "⬇️  Downloading Node $NodeVersion ..."
    Invoke-WebRequest -Uri $nodeUrl -OutFile $archivePath
} else {
    Write-Host "ℹ️  Using existing archive at $archivePath"
}

$extractDir = Join-Path $nodeRoot "node-v$NodeVersion-win-x64"
if (Test-Path $extractDir) {
    Remove-Item -Recurse -Force $extractDir
}

Write-Host "📦 Extracting Node archive"
Expand-Archive -Path $archivePath -DestinationPath $nodeRoot -Force

$currentDir = Join-Path $nodeRoot "current"
if (Test-Path $currentDir) {
    Remove-Item -Recurse -Force $currentDir
}
Copy-Item -Recurse -Force $extractDir $currentDir

$pnpmAsset = "pnpm-win-x64.exe"
$pnpmUrl = "https://github.com/pnpm/pnpm/releases/download/v$PnpmVersion/$pnpmAsset"
$pnpmArchive = Join-Path $artifactDir $pnpmAsset
if (-not (Test-Path $pnpmArchive)) {
    Write-Host "⬇️  Downloading pnpm $PnpmVersion ..."
    Invoke-WebRequest -Uri $pnpmUrl -OutFile $pnpmArchive
}

$pnpmTarget = Join-Path $currentDir "pnpm.exe"
Copy-Item -Force $pnpmArchive $pnpmTarget

$pnpxPath = Join-Path $currentDir "pnpx.cmd"
"@echo off`r`n" + """%~dp0pnpm.exe"" dlx %*" | Set-Content -Encoding ASCII -Path $pnpxPath

$nodeExe = Join-Path $currentDir "node.exe"
if (-not (Test-Path $nodeExe)) {
    throw "node.exe not found at $nodeExe"
}

$nodeHash = (Get-FileHash -Path $nodeExe -Algorithm SHA256).Hash.ToLower()
$pnpmHash = (Get-FileHash -Path $pnpmTarget -Algorithm SHA256).Hash.ToLower()

$manifest = [ordered]@{
    node_version = $NodeVersion
    pnpm_version = $PnpmVersion
    platform = "win-x64"
    node_archive = $archiveName
    node_sha256 = $nodeHash
    pnpm_sha256 = $pnpmHash
    generated_at = [DateTime]::UtcNow.ToString("o")
}
$manifest | ConvertTo-Json | Set-Content -Path $manifestPath -Encoding UTF8

$publicManifest = Join-Path $scriptDir "node-portable.manifest.json"
Copy-Item -Force $manifestPath $publicManifest

Write-Host "✅ Portable Node ready at $currentDir"
