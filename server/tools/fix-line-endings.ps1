# Fix line endings for Unix/Linux shell scripts
param(
    [Parameter(Mandatory=$false)]
    [string]$FilePath = ".\server\tools\activate-cargo.sh"
)

Write-Host "Fixing line endings for: $FilePath" -ForegroundColor Yellow

# Read the file and convert CRLF to LF
$content = Get-Content $FilePath -Raw
$content = $content -replace "`r`n", "`n"

# Write back with UTF-8 encoding and no BOM
$utf8NoBOM = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText((Resolve-Path $FilePath), $content, $utf8NoBOM)

Write-Host "✅ Line endings fixed (converted CRLF to LF)" -ForegroundColor Green
Write-Host "File is now Unix-compatible" -ForegroundColor Green
