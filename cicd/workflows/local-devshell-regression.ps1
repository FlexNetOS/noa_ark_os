Param(
    [string]$WorkspaceRoot = (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))
)

Set-Location $WorkspaceRoot

Write-Host "`n🔁 Running cargo-check via make..."
make cargo-check

Write-Host "`n🧪 Running cargo-test (no args)..."
make cargo-test

Write-Host "`n🏗️  Building UI workspace..."
make ui-build

Write-Host "`n✅ devshell regression suite complete"
