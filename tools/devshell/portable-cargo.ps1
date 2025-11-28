param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CargoArgs
)

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$WORKSPACE_ROOT = Split-Path -Parent (Split-Path -Parent $ScriptDir)
$Activator = Join-Path $WORKSPACE_ROOT "server/tools/activate-cargo.ps1"

if (-not (Test-Path $Activator)) {
    Write-Error "Unable to locate Cargo activator at $Activator"
    exit 1
}

$env:NOA_CARGO_ACTIVATE_SILENT = '1'
. $Activator
Remove-Item env:NOA_CARGO_ACTIVATE_SILENT -ErrorAction SilentlyContinue

if ($null -eq $CargoArgs -or $CargoArgs.Count -eq 0) {
    & cargo
} else {
    & cargo @CargoArgs
}
exit $LASTEXITCODE
