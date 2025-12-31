param(
  [string]$Root = ".",
  [string]$Artifacts = "agentaskit-production/docs/reports/cross_reference/artifacts"
)
$ErrorActionPreference = 'Stop'
$rootPath = (Resolve-Path $Root).Path
$artifactsPath = Join-Path $rootPath $Artifacts

function Run-CrossRef {
  & "$rootPath/agentaskit-production/tools/analysis/cross_reference.ps1" -Root $rootPath -OutDir $artifactsPath | Out-Host
}

function Update-Todo {
  & "$rootPath/agentaskit-production/tools/analysis/update_todo_from_report.ps1" | Out-Host
}

Write-Host "[RUN-ALL] Cross-reference (pass 1)" -ForegroundColor Cyan
Run-CrossRef

$reportFile = Join-Path $artifactsPath 'report.json'
if (Test-Path $reportFile) {
  $report = Get-Content -Raw -Path $reportFile | ConvertFrom-Json
  $missingDirs = @()
  if ($report.production_missing_dirs) { $missingDirs = @($report.production_missing_dirs) }
  if ($missingDirs.Count -gt 0) {
    Write-Host "[RUN-ALL] Healing missing production directories" -ForegroundColor Cyan
    foreach ($d in $missingDirs) {
      $full = Join-Path $rootPath $d
      if (-not (Test-Path $full)) {
        New-Item -ItemType Directory -Force -Path $full | Out-Null
        $keep = Join-Path $full ".gitkeep"
        if (-not (Test-Path $keep)) { Set-Content -Path $keep -Value '' -Encoding UTF8 }
      }
    }
  }
}

Write-Host "[RUN-ALL] Cross-reference (pass 2)" -ForegroundColor Cyan
Run-CrossRef

Write-Host "[RUN-ALL] Update TODO subtasks from report" -ForegroundColor Cyan
Update-Todo

Write-Host "[RUN-ALL] Completed. Artifacts at $artifactsPath" -ForegroundColor Green
