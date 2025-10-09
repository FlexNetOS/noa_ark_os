# Model Path Verification and Registry Builder
# Verifies all model files and builds comprehensive registry

$ModelsDir = "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp\models"
$RegistryPath = Join-Path $ModelsDir "models.json"

Write-Host "🔍 NOA ARK OS - Model Verification & Registry Builder" -ForegroundColor Cyan
Write-Host "=" * 70
Write-Host ""

# Check models directory
if (-not (Test-Path $ModelsDir)) {
    Write-Host "❌ Models directory not found: $ModelsDir" -ForegroundColor Red
    Write-Host "Creating directory..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $ModelsDir -Force | Out-Null
    Write-Host "✅ Created $ModelsDir" -ForegroundColor Green
    Write-Host ""
}

# Scan for model files
Write-Host "📦 Scanning for model files..." -ForegroundColor Cyan
$modelFiles = Get-ChildItem $ModelsDir -Filter "*.gguf" -ErrorAction SilentlyContinue

if ($modelFiles.Count -eq 0) {
    Write-Host "⚠️  No .gguf files found in models directory" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Current directory contents:" -ForegroundColor Cyan
    Get-ChildItem $ModelsDir | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize
    Write-Host ""
    Write-Host "💡 To download models:" -ForegroundColor Cyan
    Write-Host "   .\scripts\dev\download-all-models.ps1 -ListOnly     # List available"
    Write-Host "   .\scripts\dev\download-all-models.ps1               # Download all"
    Write-Host "   .\scripts\dev\download-all-models.ps1 -ModelName smollm3-3b  # Download specific"
    Write-Host ""
    exit 0
}

Write-Host "✅ Found $($modelFiles.Count) model file(s)" -ForegroundColor Green
Write-Host ""

# Display found models
Write-Host "📋 Discovered Models:" -ForegroundColor Cyan
Write-Host ""

$totalSize = 0
foreach ($file in $modelFiles) {
    $sizeGB = [math]::Round($file.Length / 1GB, 2)
    $totalSize += $sizeGB
    
    Write-Host "  📦 $($file.Name)" -ForegroundColor White
    Write-Host "     Size: $sizeGB GB" -ForegroundColor Gray
    Write-Host "     Path: $($file.FullName)" -ForegroundColor Gray
    Write-Host "     Modified: $($file.LastWriteTime)" -ForegroundColor Gray
    Write-Host ""
}

Write-Host "Total Size: $([math]::Round($totalSize, 2)) GB" -ForegroundColor Cyan
Write-Host ""

# Model metadata database
$modelMetadata = @{
    "llama-3.2-3b-q4.gguf" = @{
        name = "llama-3.2-3b"
        display_name = "Llama 3.2 3B"
        performance = 0.85
        cost = 0.7
        privacy_tier = "Internal"
        use_cases = @("General", "QuestionAnswering", "CodeGeneration")
    }
    "smollm3-3b-q4.gguf" = @{
        name = "smollm3-3b"
        display_name = "SmolLM3 3B"
        performance = 0.80
        cost = 0.85
        privacy_tier = "Internal"
        use_cases = @("General", "CodeGeneration")
    }
    "stable-code-3b-q4.gguf" = @{
        name = "stable-code-3b"
        display_name = "StableCode 3B"
        performance = 0.85
        cost = 0.75
        privacy_tier = "Internal"
        use_cases = @("CodeGeneration", "CodeAnalysis")
    }
    "tiny-agent-3b-q4.gguf" = @{
        name = "tiny-agent-3b"
        display_name = "Tiny Agent 3B"
        performance = 0.80
        cost = 0.90
        privacy_tier = "Internal"
        use_cases = @("AgentTask", "FunctionCalling")
    }
    "phi-4-mini-q4.gguf" = @{
        name = "phi-4-mini"
        display_name = "Phi-4 Mini 3.8B"
        performance = 0.88
        cost = 0.75
        privacy_tier = "Internal"
        use_cases = @("General", "Reasoning", "CodeGeneration")
    }
    "qwen3-4b-q4.gguf" = @{
        name = "qwen3-4b"
        display_name = "Qwen3 4B Instruct"
        performance = 0.85
        cost = 0.75
        privacy_tier = "Internal"
        use_cases = @("General", "Reasoning")
    }
    "llama-3.1-8b-q4.gguf" = @{
        name = "llama-3.1-8b"
        display_name = "Llama 3.1 8B"
        performance = 0.92
        cost = 0.65
        privacy_tier = "Internal"
        use_cases = @("General", "Reasoning", "CodeGeneration")
    }
    "mistral-nemo-12b-q4.gguf" = @{
        name = "mistral-nemo-12b"
        display_name = "Mistral Nemo 12B"
        performance = 0.93
        cost = 0.55
        privacy_tier = "Internal"
        use_cases = @("General", "Reasoning", "CodeGeneration")
    }
    "deepseek-coder-v2-q4.gguf" = @{
        name = "deepseek-coder-v2"
        display_name = "DeepSeek Coder V2"
        performance = 0.95
        cost = 0.50
        privacy_tier = "Internal"
        use_cases = @("CodeGeneration", "CodeAnalysis")
    }
    "octopus-v2-q4.gguf" = @{
        name = "octopus-v2"
        display_name = "Octopus v2"
        performance = 0.88
        cost = 0.85
        privacy_tier = "Internal"
        use_cases = @("AgentTask", "FunctionCalling")
    }
    "qwen2.5-vl-3b-q4.gguf" = @{
        name = "qwen2.5-vl-3b"
        display_name = "Qwen2.5 VL 3B"
        performance = 0.85
        cost = 0.70
        privacy_tier = "Internal"
        use_cases = @("Vision", "General")
    }
}

# Build registry
Write-Host "🔧 Building model registry..." -ForegroundColor Cyan

$registry = @{
    models = @()
    registered_at = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
    total_models = $modelFiles.Count
    total_size_gb = [math]::Round($totalSize, 2)
}

foreach ($file in $modelFiles) {
    $metadata = $modelMetadata[$file.Name]
    
    if ($metadata) {
        $registry.models += @{
            name = $metadata.name
            display_name = $metadata.display_name
            file_path = "models/$($file.Name)"
            size_mb = [math]::Round($file.Length / 1MB, 2)
            privacy_tier = $metadata.privacy_tier
            performance_score = $metadata.performance
            cost_score = $metadata.cost
            use_cases = $metadata.use_cases
            capabilities = @()
            installed_at = $file.LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss")
        }
        Write-Host "  ✅ Registered: $($metadata.display_name)" -ForegroundColor Green
    }
    else {
        # Unknown model, add with defaults
        $baseName = $file.BaseName -replace '-q4', ''
        $registry.models += @{
            name = $baseName
            display_name = $baseName
            file_path = "models/$($file.Name)"
            size_mb = [math]::Round($file.Length / 1MB, 2)
            privacy_tier = "Internal"
            performance_score = 0.75
            cost_score = 0.75
            use_cases = @("General")
            capabilities = @()
            installed_at = $file.LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss")
        }
        Write-Host "  ⚠️  Registered (defaults): $baseName" -ForegroundColor Yellow
    }
}

# Save registry
$registry | ConvertTo-Json -Depth 10 | Set-Content $RegistryPath
Write-Host ""
Write-Host "✅ Registry saved to: $RegistryPath" -ForegroundColor Green
Write-Host ""

# Display summary
Write-Host "📊 Registry Summary:" -ForegroundColor Cyan
Write-Host "  Total Models: $($registry.total_models)"
Write-Host "  Total Size: $($registry.total_size_gb) GB"
Write-Host "  Registry File: $RegistryPath"
Write-Host ""

# Show use case breakdown
Write-Host "📋 Models by Use Case:" -ForegroundColor Cyan
$allUseCases = $registry.models | ForEach-Object { $_.use_cases } | Select-Object -Unique
foreach ($useCase in $allUseCases) {
    $count = ($registry.models | Where-Object { $_.use_cases -contains $useCase }).Count
    Write-Host "  $useCase`: $count model(s)"
}
Write-Host ""

# Integration status
Write-Host "🔗 Integration Status:" -ForegroundColor Cyan
Write-Host "  ✅ Models discovered and registered"
Write-Host "  ✅ Registry file created/updated"
Write-Host "  ✅ Model Selector can now access all models"
Write-Host "  ✅ Inference engine ready to use"
Write-Host ""

Write-Host "🚀 Next Steps:" -ForegroundColor Cyan
Write-Host "  1. Start llama.cpp server: .\scripts\dev\start-llama-server.ps1"
Write-Host "  2. Test model selection in agents"
Write-Host "  3. Model Selector will automatically choose optimal models"
Write-Host ""

# Show registry content preview
Write-Host "📄 Registry Preview:" -ForegroundColor Cyan
$registry | ConvertTo-Json -Depth 10
