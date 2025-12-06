param(
  [string]$ReportPath = "agentaskit-production/docs/reports/cross_reference/artifacts/report.json",
  [string]$TodoPath = "agentaskit-production/core/src/orchestration/tasks.todo"
)

$ErrorActionPreference = 'Stop'
if (-not (Test-Path $ReportPath)) {
  Write-Warning "report.json not found at $ReportPath; nothing to update"
  exit 0
}

$report = Get-Content -Raw -Path $ReportPath | ConvertFrom-Json
$missingDirs = @()
$missingNames = @()
if ($report.production_missing_dirs) { $missingDirs = @($report.production_missing_dirs) }
if ($report.missing_in_production_basenames) { $missingNames = @($report.missing_in_production_basenames) }

$begin = "<!-- AUTO:WORKFLOW-009:SUBTASKS:BEGIN -->"
$end = "<!-- AUTO:WORKFLOW-009:SUBTASKS:END -->"
$content = Get-Content -Raw -Path $TodoPath

$lines = @()
if ($missingDirs.Count -gt 0) {
  $lines += "- [ ] Create missing production dirs:"
  foreach ($d in $missingDirs) { $lines += "  - [ ] $d" }
}
if ($missingNames.Count -gt 0) {
  $lines += "- [ ] Review basenames present outside production (top sample):"
  foreach ($n in ($missingNames | Select-Object -First 50)) { $lines += "  - [ ] $n" }
}
if ($lines.Count -eq 0) { $lines += "- [x] No gaps detected in current report.json" }
$payload = ($lines -join "`n")

$pattern = [regex]::Escape($begin) + "[\s\S]*?" + [regex]::Escape($end)
$updated = [regex]::Replace($content, $pattern, "$begin`n$payload`n$end")

if ($updated -ne $content) {
  Set-Content -Path $TodoPath -Value $updated -Encoding UTF8
  Write-Host "Updated tasks.todo subtasks from report.json"
} else {
  Write-Host "No changes to tasks.todo"
}
