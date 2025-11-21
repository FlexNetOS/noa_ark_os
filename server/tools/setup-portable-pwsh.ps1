param(
    [string]$PwshVersion = "7.4.5"
)

$scriptDir = Split-Path -Parent $PSCommandPath
$repoRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)
$pwshRoot = Join-Path $scriptDir "pwsh-portable"
$downloads = Join-Path $pwshRoot "downloads"
$manifests = Join-Path $pwshRoot "manifests"
$binDir = Join-Path $pwshRoot "bin"

$null = New-Item -ItemType Directory -Force -Path $downloads, $manifests, $binDir

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
function Resolve-PlatformSuffix {
    param([System.Runtime.InteropServices.Architecture]$Architecture)
    $archName = $Architecture.ToString()
    if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
        if ($archName -eq "X64") {
            return @{ Suffix = "win-x64"; ArchiveExt = "zip" }
        }
    } elseif ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Linux)) {
        switch ($archName) {
            "X64" { return @{ Suffix = "linux-x64"; ArchiveExt = "tar.gz" } }
            "Arm64" { return @{ Suffix = "linux-arm64"; ArchiveExt = "tar.gz" } }
        }
    } elseif ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::OSX)) {
        switch ($archName) {
            "X64" { return @{ Suffix = "osx-x64"; ArchiveExt = "tar.gz" } }
            "Arm64" { return @{ Suffix = "osx-arm64"; ArchiveExt = "tar.gz" } }
        }
    }
    throw "Unsupported platform/architecture combination: $($PSVersionTable.OS) / $Architecture"
}

$platform = Resolve-PlatformSuffix -Architecture $arch
$suffix = $platform.Suffix
$archiveExt = $platform.ArchiveExt

if ($suffix -like "win-*") {
    $archiveName = "PowerShell-$PwshVersion-$suffix.$archiveExt"
} else {
    $archiveName = "powershell-$PwshVersion-$suffix.$archiveExt"
}
$archiveBase = [System.IO.Path]::GetFileNameWithoutExtension($archiveName)
$downloadUrl = "https://github.com/PowerShell/PowerShell/releases/download/v$PwshVersion/$archiveName"
$archivePath = Join-Path $downloads $archiveName
$extractDir = Join-Path $pwshRoot $archiveBase

function Archive-ExistingManifest {
    $manifestRel = "server/tools/pwsh-portable.manifest.json"
    $manifestPath = Join-Path $scriptDir "pwsh-portable.manifest.json"
    if (-not (Test-Path $manifestPath)) { return }
    $now = Get-Date -AsUTC
    $year = $now.ToString("yyyy")
    $month = $now.ToString("MM")
    $timestamp = $now.ToString("yyyyMMddTHHmmssZ")
    $relativeDir = Split-Path $manifestRel -Parent
    $archiveDir = Join-Path $repoRoot "archive/$year/$month/$relativeDir"
    $null = New-Item -ItemType Directory -Force -Path $archiveDir
    $archiveTarget = Join-Path $repoRoot "archive/$year/$month/$manifestRel.$timestamp.tar.zst"
    $tarCmd = Get-Command tar -ErrorAction SilentlyContinue
    if (-not $tarCmd) {
        Write-Warning "tar with --zstd support not found; skipping manifest archival per policy"
        return
    }
    Push-Location $repoRoot
    try {
        & $tarCmd --zstd -cf $archiveTarget $manifestRel
    }
    finally {
        Pop-Location
    }
}

if (-not (Test-Path $archivePath)) {
    Write-Host "⬇️  Downloading $downloadUrl"
    Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath
} else {
    Write-Host "ℹ️  Using cached archive $archivePath"
}

if (Test-Path $extractDir) {
    Remove-Item -Recurse -Force $extractDir
}

if ($archiveExt -eq "zip") {
    Expand-Archive -Path $archivePath -DestinationPath $pwshRoot -Force
} else {
    & tar -xzf $archivePath -C $pwshRoot
}

$currentDir = Join-Path $pwshRoot "current"
if (Test-Path $currentDir) {
    Remove-Item -Recurse -Force $currentDir
}
Copy-Item -Recurse -Force $extractDir $currentDir

$pwshBin = if ($suffix -like "win-*") {
    Join-Path $currentDir "pwsh.exe"
} else {
    Join-Path $currentDir "pwsh"
}
if (-not (Test-Path $pwshBin)) {
    $fallback = Join-Path $currentDir "PowerShell.exe"
    if (Test-Path $fallback) {
        $pwshBin = $fallback
    } else {
        throw "PowerShell binary not found in $currentDir"
    }
}

if (-not $IsWindows) {
    & chmod +x -- $pwshBin | Out-Null
}

if ($suffix -like "win-*") {
    Copy-Item -Force $pwshBin (Join-Path $binDir "pwsh.exe")
} else {
    $linkPath = Join-Path $binDir "pwsh"
    if (Test-Path $linkPath) { Remove-Item -Force $linkPath }
    New-Item -ItemType SymbolicLink -Path $linkPath -Target $pwshBin | Out-Null
}

$pwshHash = (Get-FileHash -Path $pwshBin -Algorithm SHA256).Hash.ToLower()
$manifestPath = Join-Path $pwshRoot "manifest.json"
$generatedAt = (Get-Date -AsUTC).ToString("o")
$binaryRel = [System.IO.Path]::GetRelativePath($scriptDir, $pwshBin)
$manifest = [ordered]@{
    pwsh_version = $PwshVersion
    platform     = $suffix
    archive      = $archiveName
    source_url   = $downloadUrl
    binary       = $binaryRel
    sha256       = $pwshHash
    generated_at = $generatedAt
}
$manifest | ConvertTo-Json | Set-Content -Path $manifestPath -Encoding UTF8

Copy-Item -Force $manifestPath (Join-Path $manifests "pwsh-$PwshVersion-$suffix.json")
Archive-ExistingManifest
Copy-Item -Force $manifestPath (Join-Path $scriptDir "pwsh-portable.manifest.json")

Write-Host "✅ Portable PowerShell ready at $currentDir"
