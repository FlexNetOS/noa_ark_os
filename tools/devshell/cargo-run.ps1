Param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Args
)

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Portable = Join-Path $ScriptDir "cargo-portable.ps1"

if (-not (Test-Path $Portable)) {
    Write-Error "Portable shim not found at $Portable"
    exit 1
}

& $Portable run @Args
