Param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Args
)

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$WorkspaceRoot = Split-Path -Parent (Split-Path -Parent $ScriptDir)
$ActivateScript = Join-Path $WorkspaceRoot "server/tools/activate-cargo.ps1"
$PortableCargo = Join-Path $WorkspaceRoot "server/tools/cargo-portable/bin/cargo.exe"

$usePortable = $false
if (Test-Path $ActivateScript -PathType Leaf -and (Test-Path $PortableCargo -PathType Leaf)) {
    . $ActivateScript
    $usePortable = $true
} elseif ($env:NOA_FORCE_PORTABLE) {
    Write-Error "NOA_FORCE_PORTABLE is set but portable toolchain was not found at $PortableCargo"
    exit 1
} elseif (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "Cargo executable not found. Install Rust or provision the portable toolchain."
    exit 1
} else {
    Write-Host "ℹ️  Portable Cargo bundle not detected; using system cargo on PATH." -ForegroundColor Yellow
}

if (-not $Args -or $Args.Length -eq 0) {
    Write-Error "Usage: cargo-portable.ps1 <cargo-args>"
    exit 2
}

& cargo @Args
