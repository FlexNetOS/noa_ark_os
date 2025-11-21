param(
    [string]$PwshVersion = "7.4.5",
    [string[]]$Platforms,
    [switch]$AllPlatforms,
    [string]$DefaultPlatform
)

$supportedPlatforms = @("linux-x64", "linux-arm64", "osx-x64", "osx-arm64", "win-x64")

function Normalize-Platform {
    param([string]$Platform)
    if (-not $Platform) { return $null }
    switch ($Platform.ToLowerInvariant()) {
        "linux" { return "linux-x64" }
        "linux64" { return "linux-x64" }
        "linux-arm" { return "linux-arm64" }
        "linux-arm64" { return "linux-arm64" }
        "linux-aarch64" { return "linux-arm64" }
        "mac" { return "osx-x64" }
        "macos" { return "osx-x64" }
        "darwin" { return "osx-x64" }
        "osx" { return "osx-x64" }
        "mac-arm" { return "osx-arm64" }
        "macos-arm64" { return "osx-arm64" }
        "darwin-arm64" { return "osx-arm64" }
        "windows" { return "win-x64" }
        "win" { return "win-x64" }
        "win64" { return "win-x64" }
        default { return $Platform }
    }
}

function Get-HostPlatform {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
        if ($arch -eq [System.Runtime.InteropServices.Architecture]::X64) { return "win-x64" }
    } elseif ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Linux)) {
        switch ($arch) {
            ([System.Runtime.InteropServices.Architecture]::X64) { return "linux-x64" }
            ([System.Runtime.InteropServices.Architecture]::Arm64) { return "linux-arm64" }
        }
    } elseif ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::OSX)) {
        switch ($arch) {
            ([System.Runtime.InteropServices.Architecture]::X64) { return "osx-x64" }
            ([System.Runtime.InteropServices.Architecture]::Arm64) { return "osx-arm64" }
        }
    }
    throw "Unsupported host platform/architecture: $($PSVersionTable.OS) / $arch"
}

function Resolve-Platforms {
    param([string[]]$Requested,[switch]$IncludeAll)
    if ($IncludeAll) { return $supportedPlatforms }
    if (-not $Requested -or $Requested.Count -eq 0) { return @(Get-HostPlatform) }
    return $Requested | Where-Object { $_ } | ForEach-Object { Normalize-Platform $_ } | Select-Object -Unique
}

function Ensure-PlatformSupported {
    param([string]$Platform)
    if ($supportedPlatforms -notcontains $Platform) {
        throw "Unsupported platform target: $Platform"
    }
}

function Get-PlatformMetadata {
    param([string]$Platform)
    $isWindows = $Platform -like "win-*"
    $archiveExt = if ($isWindows) { "zip" } else { "tar.gz" }
    $archiveName = if ($isWindows) {
        "PowerShell-$PwshVersion-$Platform.$archiveExt"
    } else {
        "powershell-$PwshVersion-$Platform.$archiveExt"
    }
    $binaryName = if ($isWindows) { "pwsh.exe" } else { "pwsh" }
    $sourceUrl = "https://github.com/PowerShell/PowerShell/releases/download/v$PwshVersion/$archiveName"
    return [pscustomobject]@{
        Platform   = $Platform
        ArchiveExt = $archiveExt
        Archive    = $archiveName
        Binary     = $binaryName
        SourceUrl  = $sourceUrl
    }
}

function Expand-ArchivePortable {
    param(
        [string]$ArchivePath,
        [string]$Destination,
        [string]$ArchiveExt
    )
    if ($ArchiveExt -eq "zip") {
        Add-Type -AssemblyName System.IO.Compression.FileSystem -ErrorAction SilentlyContinue | Out-Null
        [System.IO.Compression.ZipFile]::ExtractToDirectory($ArchivePath, $Destination, $true)
    } else {
        & tar -xzf $ArchivePath -C $Destination
    }
}

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

$scriptDir = Split-Path -Parent $PSCommandPath
$repoRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)
$pwshRoot = Join-Path $scriptDir "pwsh-portable"
$platformRoot = Join-Path $pwshRoot "platforms"
$downloads = Join-Path $pwshRoot "downloads"
$manifests = Join-Path $pwshRoot "manifests"
$binDir = Join-Path $pwshRoot "bin"
$null = New-Item -ItemType Directory -Force -Path $downloads, $manifests, $binDir, $platformRoot

$resolvedPlatforms = Resolve-Platforms -Requested $Platforms -IncludeAll:$AllPlatforms
foreach ($platform in $resolvedPlatforms) { Ensure-PlatformSupported -Platform $platform }

if (-not $DefaultPlatform) { $DefaultPlatform = Get-HostPlatform }
if ($resolvedPlatforms -notcontains $DefaultPlatform) {
    throw "Default platform '$DefaultPlatform' was not requested. Requested: $($resolvedPlatforms -join ', ')"
}

$entries = @()
$currentLink = Join-Path $pwshRoot "current"

foreach ($platform in $resolvedPlatforms) {
    $meta = Get-PlatformMetadata -Platform $platform
    $archivePath = Join-Path $downloads $meta.Archive
    if (-not (Test-Path $archivePath)) {
        Write-Host "⬇️  Downloading $($meta.SourceUrl)"
        Invoke-WebRequest -Uri $meta.SourceUrl -OutFile $archivePath
    } else {
        Write-Host "ℹ️  Using cached archive $archivePath"
    }
    $archiveSha = (Get-FileHash -Path $archivePath -Algorithm SHA256).Hash.ToLower()

    $tmpDir = Join-Path $pwshRoot (".extract-" + [guid]::NewGuid().ToString())
    $null = New-Item -ItemType Directory -Force -Path $tmpDir
    Expand-ArchivePortable -ArchivePath $archivePath -Destination $tmpDir -ArchiveExt $meta.ArchiveExt

    $platformDir = Join-Path $platformRoot $platform
    if (Test-Path $platformDir) { Remove-Item -Recurse -Force $platformDir }
    $null = New-Item -ItemType Directory -Force -Path $platformDir

    $children = Get-ChildItem -LiteralPath $tmpDir
    if ($children.Count -eq 1 -and $children[0].PSIsContainer) {
        Move-Item -LiteralPath $children[0].FullName -Destination $platformDir
        $bundleDir = Join-Path $platformDir $children[0].Name
    } else {
        $bundleDir = Join-Path $platformDir ([System.IO.Path]::GetFileNameWithoutExtension($meta.Archive))
        if ($bundleDir.EndsWith('.tar')) { $bundleDir = $bundleDir.Substring(0, $bundleDir.Length - 4) }
        $null = New-Item -ItemType Directory -Force -Path $bundleDir
        foreach ($child in $children) {
            Move-Item -LiteralPath $child.FullName -Destination $bundleDir
        }
    }
    Remove-Item -Recurse -Force $tmpDir
    $bundleRootRel = [System.IO.Path]::GetRelativePath($scriptDir, $bundleDir)

    if ($meta.Platform -like "win-*") {
        $pwshBin = Join-Path $bundleDir "pwsh.exe"
        if (-not (Test-Path $pwshBin)) {
            $fallback = Join-Path $bundleDir "PowerShell.exe"
            if (-not (Test-Path $fallback)) {
                throw "PowerShell binary missing for $platform"
            }
            $pwshBin = $fallback
        }
    } else {
        $pwshBin = Join-Path $bundleDir $meta.Binary
        if (-not (Test-Path $pwshBin)) {
            throw "PowerShell binary missing for $platform"
        }
        & chmod +x -- $pwshBin | Out-Null
    }

    $platformBinDir = Join-Path $binDir $platform
    $null = New-Item -ItemType Directory -Force -Path $platformBinDir
    if ($meta.Platform -like "win-*") {
        Copy-Item -Force $pwshBin (Join-Path $platformBinDir "pwsh.exe")
    } else {
        $linkPath = Join-Path $platformBinDir "pwsh"
        if (Test-Path $linkPath) { Remove-Item -Force $linkPath }
        New-Item -ItemType SymbolicLink -Path $linkPath -Target $pwshBin | Out-Null
    }

    if ($platform -eq $DefaultPlatform) {
        if (Test-Path $currentLink) { Remove-Item -Recurse -Force $currentLink }
        try {
            New-Item -ItemType SymbolicLink -Path $currentLink -Target $bundleDir -Force | Out-Null
        } catch {
            Copy-Item -Recurse -Force $bundleDir $currentLink
        }
        if ($meta.Platform -like "win-*") {
            Copy-Item -Force $pwshBin (Join-Path $binDir "pwsh.exe")
        } else {
            $binLink = Join-Path $binDir "pwsh"
            if (Test-Path $binLink) { Remove-Item -Force $binLink }
            New-Item -ItemType SymbolicLink -Path $binLink -Target $pwshBin | Out-Null
        }
    }

    $pwshHash = (Get-FileHash -Path $pwshBin -Algorithm SHA256).Hash.ToLower()
    $binaryRel = [System.IO.Path]::GetRelativePath($scriptDir, $pwshBin)
    $binaryKind = if ($meta.Platform -like "win-*") { "pwsh.exe" } else { "pwsh" }
    $entry = [ordered]@{
        platform  = $platform
        archive   = $meta.Archive
        archive_sha256 = $archiveSha
        bundle_root = $bundleRootRel
        source_url = $meta.SourceUrl
        binary    = $binaryRel
        binary_kind = $binaryKind
        sha256    = $pwshHash
        generated_at = (Get-Date -AsUTC).ToString("o")
    }
    $entries += [pscustomobject]$entry

    $platformManifest = [ordered]@{
        pwsh_version = $PwshVersion
        platform     = $platform
        archive      = $meta.Archive
        archive_sha256 = $archiveSha
        source_url   = $meta.SourceUrl
        bundle_root  = $bundleRootRel
        binary_kind  = $binaryKind
        binary       = $binaryRel
        sha256       = $pwshHash
        generated_at = (Get-Date -AsUTC).ToString("o")
    }
    $platformManifest | ConvertTo-Json -Depth 4 | Set-Content -Path (Join-Path $manifests "pwsh-$PwshVersion-$platform.json") -Encoding UTF8
}

$manifestPath = Join-Path $pwshRoot "manifest.json"
$manifest = [ordered]@{
    pwsh_version    = $PwshVersion
    generated_at    = (Get-Date -AsUTC).ToString("o")
    default_platform = $DefaultPlatform
    platforms       = $entries
}
$manifest | ConvertTo-Json -Depth 5 | Set-Content -Path $manifestPath -Encoding UTF8

Archive-ExistingManifest
Copy-Item -Force $manifestPath (Join-Path $scriptDir "pwsh-portable.manifest.json")

Write-Host "✅ Portable PowerShell bundles ready. Default platform: $DefaultPlatform"
