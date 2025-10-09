# Model Switcher for Llama.cpp Server
# Quickly switch between installed models

param(
    [Parameter(Mandatory=$false)]
    [switch]$List = $false,
    
    [Parameter(Mandatory=$false)]
    [string]$ModelName = ""
)

$ModelsDir = "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp\models"
$StartScript = "D:\dev\workspaces\noa_ark_os\scripts\dev\start-llama-server.ps1"

function Write-Info { param($Message) Write-Host "ℹ️  $Message" -ForegroundColor Cyan }
function Write-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "⚠️  $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor Red }

if ($List) {
    Write-Info "Installed Models:"
    Write-Info ""
    
    $models = Get-ChildItem $ModelsDir -Filter "*.gguf" | Sort-Object Length
    
    if ($models.Count -eq 0) {
        Write-Warning "No models found in $ModelsDir"
        Write-Info "Download models with: .\scripts\dev\download-models.ps1 -ListAll"
        exit 0
    }
    
    foreach ($model in $models) {
        $sizeMB = [math]::Round($model.Length / 1MB, 2)
        $sizeGB = [math]::Round($model.Length / 1GB, 2)
        
        if ($sizeGB -ge 1) {
            $sizeStr = "$sizeGB GB"
        } else {
            $sizeStr = "$sizeMB MB"
        }
        
        Write-Host "  $($model.Name)" -ForegroundColor Yellow
        Write-Host "    Size: $sizeStr" -ForegroundColor Gray
    }
    
    Write-Info ""
    Write-Info "To use a model:"
    Write-Info "  .\scripts\dev\switch-model.ps1 -ModelName <filename>"
    exit 0
}

if ($ModelName -eq "") {
    Write-Error "Please specify -ModelName or use -List to see available models"
    Write-Info "Example: .\scripts\dev\switch-model.ps1 -ModelName llama-3.2-3b-q4.gguf"
    exit 1
}

# Check if model exists
$modelPath = Join-Path $ModelsDir $ModelName
if (!(Test-Path $modelPath)) {
    Write-Error "Model not found: $ModelName"
    Write-Info "Available models:"
    & $PSCommandPath -List
    exit 1
}

Write-Info "Switching to model: $ModelName"

# Update start script
$content = Get-Content $StartScript -Raw
$newContent = $content -replace '\$ModelPath = Join-Path \$LlamaCppDir "models\\[^"]*"', "`$ModelPath = Join-Path `$LlamaCppDir `"models\$ModelName`""

Set-Content $StartScript $newContent -Encoding UTF8

Write-Success "Model switched successfully!"
Write-Info ""
Write-Info "Current model: $ModelName"
Write-Info "Restart the server to use the new model:"
Write-Info "  .\scripts\dev\start-llama-server.ps1"
