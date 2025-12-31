$ErrorActionPreference = 'Stop'
$devDir = Split-Path -Parent $PSScriptRoot
$applier = Join-Path $devDir 'apply-config.mjs'

if (Get-Command node -ErrorAction SilentlyContinue) {
  if (Test-Path $applier) {
    $result = & node $applier pwsh
    if ($LASTEXITCODE -ne 0) {
      throw "Failed to apply devshell configuration via $applier"
    }
    if ($result) {
      try {
        $envVars = $result | ConvertFrom-Json
        foreach ($pair in $envVars.PSObject.Properties) {
          $envName = $pair.Name
          $envValue = $pair.Value
          if ($envName -and $envValue -is [string]) {
            $env:$envName = $envValue
          }
        }
      } catch {
        throw "Failed to parse environment variables from $applier output as JSON: $_"
      }
      Invoke-Expression $result
    }
  }
} else {
  Write-Warning 'Node.js is required to hydrate NOA Ark OS devshell environment'
}

if ($env:PNPM_HOME) {
  if (-not (Test-Path $env:PNPM_HOME)) {
    New-Item -ItemType Directory -Path $env:PNPM_HOME -Force | Out-Null
  }
  $env:Path = (Join-Path $env:PNPM_HOME '') + ';' + $env:Path
}

if ($env:NOA_PNPM_REQUIRED -and (Get-Command corepack -ErrorAction SilentlyContinue)) {
  try {
    corepack prepare "pnpm@$($env:NOA_PNPM_REQUIRED)" --activate | Out-Null
  } catch {
    Write-Warning "corepack failed to prepare pnpm@$($env:NOA_PNPM_REQUIRED): $_"
  }
}

if (-not $env:PNPM) {
  $env:PNPM = 'pnpm'
}
