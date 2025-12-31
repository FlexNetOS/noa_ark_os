#!/usr/bin/env pwsh
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$caddyHome = Join-Path $scriptDir 'caddy-portable'
$current = Join-Path $caddyHome 'current'

$binary = if (Test-Path (Join-Path $current 'caddy.exe')) {
    Join-Path $current 'caddy.exe'
} elseif (Test-Path (Join-Path $current 'caddy')) {
    Join-Path $current 'caddy'
} else {
    throw 'Portable Caddy not found. Run setup-portable-caddy.ps1 first.'
}

$env:NOA_CADDY_HOME = $current
$env:PATH = "$current;" + $env:PATH

$admin = if ($env:NOA_CADDY_ADMIN_ENDPOINT) { $env:NOA_CADDY_ADMIN_ENDPOINT } else { 'http://127.0.0.1:2019' }

Write-Host "✅ Caddy activated from $current"
Write-Host "   Binary : $binary"
Write-Host "   Admin  : $admin"
