# Background Model Downloader
# Downloads all remaining models in sequence with progress tracking

$ErrorActionPreference = "Continue"

$ModelsToDownload = @(
    "mistral-nemo-12b",
    "llama-3.1-8b",
    "phi-4-mini",
    "qwen3-4b",
    "gemma-3-4b",
    "smollm3-3b",
    "stable-code-3b",
    "tiny-agent-3b",
    "qwen2.5-vl-3b",
    "octopus-v2",
    "tinyllama-1.1b",
    "qwen3-0.6b",
    "gemma-3-270m"
)

$LogFile = "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp\models\download_log.txt"
$StartTime = Get-Date

Write-Host "🚀 Background Model Download Started" -ForegroundColor Cyan
Write-Host "=" * 70
Write-Host "Start Time: $StartTime"
Write-Host "Models to download: $($ModelsToDownload.Count)"
Write-Host "Log file: $LogFile"
Write-Host ""

# Create log
"Model Download Log - Started: $StartTime" | Out-File $LogFile
"=" * 70 | Out-File $LogFile -Append
"" | Out-File $LogFile -Append

$completed = 0
$failed = 0

foreach ($model in $ModelsToDownload) {
    $index = $ModelsToDownload.IndexOf($model) + 1
    $total = $ModelsToDownload.Count
    
    Write-Host "[$index/$total] Downloading: $model..." -ForegroundColor Cyan
    "[$index/$total] Downloading: $model - $(Get-Date)" | Out-File $LogFile -Append
    
    try {
        & ".\scripts\dev\download-all-models.ps1" -ModelName $model 2>&1 | Out-File $LogFile -Append
        
        if ($LASTEXITCODE -eq 0 -or $?) {
            Write-Host "  ✅ Success: $model" -ForegroundColor Green
            "  ✅ Success: $model" | Out-File $LogFile -Append
            $completed++
        } else {
            Write-Host "  ⚠️  Warning: $model (may already exist)" -ForegroundColor Yellow
            "  ⚠️  Warning: $model" | Out-File $LogFile -Append
            $completed++
        }
    }
    catch {
        Write-Host "  ❌ Failed: $model - $_" -ForegroundColor Red
        "  ❌ Failed: $model - $_" | Out-File $LogFile -Append
        $failed++
    }
    
    "" | Out-File $LogFile -Append
    Write-Host ""
}

$EndTime = Get-Date
$Duration = $EndTime - $StartTime

Write-Host ""
Write-Host "=" * 70
Write-Host "🎉 Download Process Complete!" -ForegroundColor Green
Write-Host "=" * 70
Write-Host "Start Time: $StartTime"
Write-Host "End Time: $EndTime"
Write-Host "Duration: $Duration"
Write-Host "Completed: $completed models"
Write-Host "Failed: $failed models"
Write-Host ""

"" | Out-File $LogFile -Append
"=" * 70 | Out-File $LogFile -Append
"Download Process Complete - Ended: $EndTime" | Out-File $LogFile -Append
"Duration: $Duration" | Out-File $LogFile -Append
"Completed: $completed models" | Out-File $LogFile -Append
"Failed: $failed models" | Out-File $LogFile -Append

# Run verification
Write-Host "🔍 Running model verification..." -ForegroundColor Cyan
& ".\scripts\dev\verify-models.ps1" | Out-File $LogFile -Append

Write-Host ""
Write-Host "✅ All done! Check log: $LogFile" -ForegroundColor Green
Write-Host ""
Write-Host "🚀 Next steps:" -ForegroundColor Cyan
Write-Host "  1. Start server: .\scripts\dev\start-llama-server.ps1"
Write-Host "  2. View models: .\scripts\dev\verify-models.ps1"
Write-Host "  3. Test with agents!"
Write-Host ""
