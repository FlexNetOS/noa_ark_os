param(
  [string]$Root = ".",
  [string]$OutDir = "agentaskit-production/docs/reports/cross_reference/artifacts"
)

$ErrorActionPreference = "Stop"
$rootPath = (Resolve-Path $Root).Path
# Use OutDir as-is if absolute; otherwise, make it relative to root
if ([System.IO.Path]::IsPathRooted($OutDir)) {
  $outPath = $OutDir
} else {
  $outPath = Join-Path $rootPath $OutDir
}
New-Item -ItemType Directory -Force -Path $outPath | Out-Null

# Ignore directories
$ignoreDirs = @('.git', '.github', '.venv', 'node_modules', 'target', '.idea', '.vs', '.vscode', '__pycache__')

function Is-IgnoredPath([string]$path) {
  foreach ($d in $ignoreDirs) {
    if ($path -match "(?i)(^|[\\/])$([regex]::Escape($d))([\\/]|$)") { return $true }
  }
  return $false
}

function Classify-Path([string]$rel) {
  if ($rel -match '(?i)archive[\\/]+old_versions[\\/]+') { return 'archive' }
  if ($rel -match '(?i)[\\/](V[2-7]|agentaskitv[2-7])[\\/]') { return 'archive' }
  if ($rel -match '(?i)^agentaskit-production[\\/]') { return 'production' }
  if ($rel -match '(?i)[\\/]agentaskit-production[\\/]') { return 'production' }
  return 'other'
}

$files = @()
$summary = @{ files_total = 0; archive_files = 0; production_files = 0; other_files = 0 }

Get-ChildItem -LiteralPath $rootPath -Recurse -File | ForEach-Object {
  $full = $_.FullName
  if (Is-IgnoredPath($full)) { return }
  $rel = ($full.Substring($rootPath.Length)).TrimStart('\','/')
  $kind = Classify-Path($rel)
  $sha = (Get-FileHash -LiteralPath $full -Algorithm SHA256).Hash.ToLower()
  $entry = [ordered]@{
    path = $rel -replace '\\','/'
    kind = $kind
    sha256 = $sha
    size = $_.Length
    text = $true
  }
  $files += $entry
  $summary.files_total++
  switch ($kind) {
    'archive' { $summary.archive_files++ }
    'production' { $summary.production_files++ }
    default { $summary.other_files++ }
  }
}

# lineage by basename
$byName = @{}
foreach ($f in $files) {
  $name = [System.IO.Path]::GetFileName($f.path).ToLower()
  if (-not $byName.ContainsKey($name)) { $byName[$name] = @() }
  $byName[$name] += $f
}
$lineage = @()
foreach ($k in $byName.Keys) {
  $entries = $byName[$k]
  if ($entries.Count -lt 2) { continue }
  $kinds = ($entries | ForEach-Object { $_.kind } | Sort-Object -Unique)
  if ($kinds -contains 'archive' -and $kinds -contains 'production') {
    $lineage += @{ name = $k; entries = $entries }
  }
}

# duplicates by hash
$byHash = @{}
foreach ($f in $files) {
  if (-not $byHash.ContainsKey($f.sha256)) { $byHash[$f.sha256] = @() }
  $byHash[$f.sha256] += $f
}
$duplicates = @()
foreach ($h in $byHash.Keys) {
  $arr = $byHash[$h]
  if ($arr.Count -gt 1) { $duplicates += ,$arr }
}

# production vs non-production basenames using hashtables as sets
$prodSet = @{}
$nonProdSet = @{}
foreach ($f in $files) {
  $nm = [System.IO.Path]::GetFileName($f.path).ToLower()
  if ($f.kind -eq 'production') { $prodSet[$nm] = $true } else { $nonProdSet[$nm] = $true }
}
$nonProdOnly = @()
foreach ($k in $nonProdSet.Keys) { if (-not $prodSet.ContainsKey($k)) { $nonProdOnly += $k } }
$prodOnly = @()
foreach ($k in $prodSet.Keys) { if (-not $nonProdSet.ContainsKey($k)) { $prodOnly += $k } }
$missingInProd = ($nonProdOnly | Sort-Object) | Select-Object -First 200
$extraInProd = ($prodOnly | Sort-Object) | Select-Object -First 200

$expectedDirs = @(
  'agentaskit-production/core/src/workflows/seven_phase',
  'agentaskit-production/tests',
  'agentaskit-production/dashboards',
  'agentaskit-production/alerts',
  'agentaskit-production/slo',
  'agentaskit-production/security/policies',
  'agentaskit-production/operational_hash',
  'agentaskit-production/TEST',
  '.github/workflows'
)
$missingDirs = @()
foreach ($d in $expectedDirs) {
  if (-not (Test-Path (Join-Path $rootPath $d))) { $missingDirs += $d }
}

$manifest = [ordered]@{
  root = $rootPath
  summary = $summary
  files = $files
}
$report = [ordered]@{
  summary = $summary
  lineage_pairs = $lineage.Count
  duplicates_groups = $duplicates.Count
  production_missing_dirs = $missingDirs
  missing_in_production_basenames = $missingInProd
  extra_in_production_basenames = $extraInProd
}

$manifest | ConvertTo-Json -Depth 8 | Set-Content -Path (Join-Path $outPath 'manifest.json') -Encoding UTF8
$report | ConvertTo-Json -Depth 8 | Set-Content -Path (Join-Path $outPath 'report.json') -Encoding UTF8

# Markdown
$md = @()
$md += '# Cross-reference Report (archives V2–V7 → production)'
$md += ("- Files total: {0} (archive: {1}, production: {2}, other: {3})" -f $summary.files_total, $summary.archive_files, $summary.production_files, $summary.other_files)
$md += ("- Lineage pairs (archive↔production filename collisions): {0}" -f $lineage.Count)
$md += ("- Duplicate groups (identical sha across different paths): {0}" -f $duplicates.Count)
if ($missingDirs.Count -gt 0) {
  $md += '## Missing expected production components'
  foreach ($m in $missingDirs) { $md += "- $m" }
}
if ($missingInProd.Count -gt 0) {
  $md += '## Basenames present outside production but not in final package (top 200)'
  foreach ($n in ($missingInProd | Select-Object -First 50)) { $md += "- $n" }
  if ($missingInProd.Count -gt 50) { $md += ("… and {0} more" -f ($missingInProd.Count-50)) }
}
if ($extraInProd.Count -gt 0) {
  $md += '## Basenames present in final package but not elsewhere (top 200)'
  foreach ($n in ($extraInProd | Select-Object -First 50)) { $md += "- $n" }
  if ($extraInProd.Count -gt 50) { $md += ("… and {0} more" -f ($extraInProd.Count-50)) }
}
Set-Content -Path (Join-Path $outPath 'report.md') -Value ($md -join "`n") -Encoding UTF8

Write-Host "Cross-reference completed. Artifacts at $outPath"
