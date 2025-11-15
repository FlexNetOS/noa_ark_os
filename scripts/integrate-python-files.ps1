# Python File Integration Script
# Categorizes and moves Python files from stale directory

$source = "crc\drop-in\incoming\stale\agent_factory\python"
$pythonFiles = Get-ChildItem $source -File -Filter "*.py"

# Categorization rules
$categories = @{
    "agents" = @("*noa_commander*", "*agent*.py")
    "infrastructure" = @("*gateway*", "*service_mesh*", "*load_balanc*", "*service_discovery*")
    "constitutional" = @("*constitutional*", "*governance*", "*validator*", "*compliance*")
    "autonomy" = @("*self_*", "*autonomy*", "*proactive*")
    "devops" = @("*canary*", "*drift*", "*metrics*", "*rollback*", "*promotion*", "*testing*", "*monitor*")
    "optimization" = @("*optimi*", "*resource*", "*performance*", "*storage*", "*network*", "*hardware*")
}

Write-Host "🐍 Categorizing and moving Python files..."
Write-Host ""

$moved = @{}
foreach($cat in $categories.Keys) {
    $moved[$cat] = 0
}
$moved["other"] = 0

foreach($file in $pythonFiles) {
    $matched = $false
    foreach($cat in $categories.Keys) {
        foreach($pattern in $categories[$cat]) {
            if($file.Name -like $pattern) {
                $dest = "server\python\$cat\$($file.Name)"
                Copy-Item $file.FullName $dest -Force
                $moved[$cat]++
                $matched = $true
                break
            }
        }
        if($matched) { break }
    }
    
    if(-not $matched) {
        $dest = "server\python\infrastructure\$($file.Name)"
        Copy-Item $file.FullName $dest -Force
        $moved["other"]++
    }
}

Write-Host "✅ Python files moved:"
foreach($cat in $moved.Keys | Sort-Object) {
    if($moved[$cat] -gt 0) {
        Write-Host "   $cat`: $($moved[$cat]) files"
    }
}

Write-Host ""
Write-Host "Total: $(($moved.Values | Measure-Object -Sum).Sum) files"
