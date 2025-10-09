# NOA ARK OS - Comprehensive Model Downloader
# Downloads and registers all specified models for the agent system

param(
    [string]$ModelName = "all",
    [switch]$ListOnly,
    [switch]$DryRun
)

$ErrorActionPreference = "Continue"
$ModelsDir = "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp\models"

# Ensure models directory exists
if (-not (Test-Path $ModelsDir)) {
    New-Item -ItemType Directory -Path $ModelsDir -Force | Out-Null
}

# Comprehensive model catalog
$ModelCatalog = @(
    # Currently installed
    @{
        Name = "llama-3.2-3b"
        DisplayName = "Llama 3.2 3B"
        URL = "https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct-Q4_K_M.gguf"
        FileName = "llama-3.2-3b-q4.gguf"
        Size = "1.88 GB"
        UseCase = @("General", "QuestionAnswering")
        Performance = 0.85
        Cost = 0.7
        Installed = $true
    },
    
    # Tiny/Efficient Models (< 1GB)
    @{
        Name = "gemma-3-270m"
        DisplayName = "Gemma 3 270M IT"
        URL = "https://huggingface.co/bartowski/gemma-3-270m-it-GGUF/resolve/main/gemma-3-270m-it-Q4_K_M.gguf"
        FileName = "gemma-3-270m-q4.gguf"
        Size = "170 MB"
        UseCase = @("General", "AgentTask")
        Performance = 0.65
        Cost = 0.95
        Installed = $false
    },
    @{
        Name = "qwen3-0.6b"
        DisplayName = "Qwen3 0.6B"
        URL = "https://huggingface.co/Qwen/Qwen3-0.6B-Instruct-GGUF/resolve/main/qwen3-0_6b-instruct-q4_k_m.gguf"
        FileName = "qwen3-0.6b-q4.gguf"
        Size = "400 MB"
        UseCase = @("General", "AgentTask")
        Performance = 0.70
        Cost = 0.95
        Installed = $false
    },
    @{
        Name = "tinyllama-1.1b"
        DisplayName = "TinyLlama 1.1B"
        URL = "https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
        FileName = "tinyllama-1.1b-q4.gguf"
        Size = "670 MB"
        UseCase = @("General", "AgentTask")
        Performance = 0.68
        Cost = 0.95
        Installed = $false
    },
    
    # Small Models (1-3 GB)
    @{
        Name = "smollm3-3b"
        DisplayName = "SmolLM3 3B"
        URL = "https://huggingface.co/HuggingFaceTB/SmolLM3-3B-Instruct-GGUF/resolve/main/smollm3-3b-instruct-q4_k_m.gguf"
        FileName = "smollm3-3b-q4.gguf"
        Size = "1.8 GB"
        UseCase = @("General", "CodeGeneration")
        Performance = 0.80
        Cost = 0.85
        Installed = $false
    },
    @{
        Name = "stable-code-3b"
        DisplayName = "StableCode 3B"
        URL = "https://huggingface.co/stabilityai/stable-code-3b/resolve/main/stable-code-3b-Q4_K_M.gguf"
        FileName = "stable-code-3b-q4.gguf"
        Size = "1.9 GB"
        UseCase = @("CodeGeneration", "CodeAnalysis")
        Performance = 0.85
        Cost = 0.75
        Installed = $false
    },
    @{
        Name = "tiny-agent-3b"
        DisplayName = "Tiny Agent 3B"
        URL = "https://huggingface.co/driaforall/Tiny-Agent-a-3B-GGUF/resolve/main/Tiny-Agent-a-3B-Q4_K_M.gguf"
        FileName = "tiny-agent-3b-q4.gguf"
        Size = "1.8 GB"
        UseCase = @("AgentTask", "FunctionCalling")
        Performance = 0.80
        Cost = 0.90
        Installed = $false
    },
    @{
        Name = "phi-4-mini"
        DisplayName = "Phi-4 Mini 3.8B"
        URL = "https://huggingface.co/bartowski/Phi-4-Mini-Instruct-GGUF/resolve/main/Phi-4-Mini-Instruct-Q4_K_M.gguf"
        FileName = "phi-4-mini-q4.gguf"
        Size = "2.3 GB"
        UseCase = @("General", "Reasoning", "CodeGeneration")
        Performance = 0.88
        Cost = 0.75
        Installed = $false
    },
    @{
        Name = "qwen3-4b"
        DisplayName = "Qwen3 4B Instruct"
        URL = "https://huggingface.co/Qwen/Qwen3-4B-Instruct-GGUF/resolve/main/qwen3-4b-instruct-q4_k_m.gguf"
        FileName = "qwen3-4b-q4.gguf"
        Size = "2.5 GB"
        UseCase = @("General", "Reasoning")
        Performance = 0.85
        Cost = 0.75
        Installed = $false
    },
    @{
        Name = "gemma-3-4b"
        DisplayName = "Gemma 3 4B IT"
        URL = "https://huggingface.co/bartowski/gemma-3-4b-it-GGUF/resolve/main/gemma-3-4b-it-Q4_K_M.gguf"
        FileName = "gemma-3-4b-q4.gguf"
        Size = "2.5 GB"
        UseCase = @("General", "QuestionAnswering")
        Performance = 0.85
        Cost = 0.75
        Installed = $false
    },
    
    # Medium Models (4-8 GB)
    @{
        Name = "llama-3.1-8b"
        DisplayName = "Llama 3.1 8B"
        URL = "https://huggingface.co/bartowski/Meta-Llama-3.1-8B-Instruct-GGUF/resolve/main/Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf"
        FileName = "llama-3.1-8b-q4.gguf"
        Size = "4.9 GB"
        UseCase = @("General", "Reasoning", "CodeGeneration")
        Performance = 0.92
        Cost = 0.65
        Installed = $false
    },
    @{
        Name = "octopus-v2"
        DisplayName = "Octopus v2 2B"
        URL = "https://huggingface.co/NexaAIDev/Octopus-v2-GGUF/resolve/main/octopus-v2-q4_k_m.gguf"
        FileName = "octopus-v2-q4.gguf"
        Size = "1.2 GB"
        UseCase = @("AgentTask", "FunctionCalling")
        Performance = 0.88
        Cost = 0.85
        Installed = $false
    },
    
    # Large Models (8-15 GB)
    @{
        Name = "mistral-nemo-12b"
        DisplayName = "Mistral Nemo 12B"
        URL = "https://huggingface.co/bartowski/Mistral-Nemo-Instruct-2407-GGUF/resolve/main/Mistral-Nemo-Instruct-2407-Q4_K_M.gguf"
        FileName = "mistral-nemo-12b-q4.gguf"
        Size = "7.5 GB"
        UseCase = @("General", "Reasoning", "CodeGeneration")
        Performance = 0.93
        Cost = 0.55
        Installed = $false
    },
    @{
        Name = "deepseek-coder-v2"
        DisplayName = "DeepSeek Coder V2 16B"
        URL = "https://huggingface.co/bartowski/DeepSeek-Coder-V2-Lite-Instruct-GGUF/resolve/main/DeepSeek-Coder-V2-Lite-Instruct-Q4_K_M.gguf"
        FileName = "deepseek-coder-v2-q4.gguf"
        Size = "9.5 GB"
        UseCase = @("CodeGeneration", "CodeAnalysis")
        Performance = 0.95
        Cost = 0.50
        Installed = $false
    },
    
    # Multimodal Models
    @{
        Name = "qwen2.5-vl-3b"
        DisplayName = "Qwen2.5 VL 3B"
        URL = "https://huggingface.co/Qwen/Qwen2-VL-3B-Instruct-GGUF/resolve/main/qwen2-vl-3b-instruct-q4_k_m.gguf"
        FileName = "qwen2.5-vl-3b-q4.gguf"
        Size = "2.0 GB"
        UseCase = @("Vision", "General")
        Performance = 0.85
        Cost = 0.70
        Installed = $false
    }
)

function Write-Info { param($Msg) Write-Host "ℹ️  $Msg" -ForegroundColor Cyan }
function Write-Success { param($Msg) Write-Host "✅ $Msg" -ForegroundColor Green }
function Write-Warning { param($Msg) Write-Host "⚠️  $Msg" -ForegroundColor Yellow }
function Write-Error { param($Msg) Write-Host "❌ $Msg" -ForegroundColor Red }

if ($ListOnly) {
    Write-Info "Available Models:"
    Write-Host ""
    
    $ModelCatalog | ForEach-Object {
        $status = if ($_.Installed) { "✅ Installed" } else { "⬜ Not Installed" }
        Write-Host "$status - $($_.DisplayName) ($($_.Size))"
        Write-Host "   Name: $($_.Name)"
        Write-Host "   Use Cases: $($_.UseCase -join ', ')"
        Write-Host "   Performance: $($_.Performance), Cost: $($_.Cost)"
        Write-Host ""
    }
    
    $installed = ($ModelCatalog | Where-Object { $_.Installed }).Count
    $total = $ModelCatalog.Count
    $totalSize = ($ModelCatalog | Measure-Object -Property { [double]($_.Size -replace '[^0-9.]') } -Sum).Sum
    
    Write-Info "Summary:"
    Write-Host "  Installed: $installed/$total models"
    Write-Host "  Total catalog size: ~$([math]::Round($totalSize, 1)) GB"
    Write-Host ""
    
    exit 0
}

function Download-Model {
    param(
        [hashtable]$Model
    )
    
    $filePath = Join-Path $ModelsDir $Model.FileName
    
    if (Test-Path $filePath) {
        Write-Info "$($Model.DisplayName) already exists, skipping..."
        return $true
    }
    
    if ($DryRun) {
        Write-Info "[DRY RUN] Would download: $($Model.DisplayName) ($($Model.Size))"
        return $true
    }
    
    Write-Info "Downloading $($Model.DisplayName) ($($Model.Size))..."
    Write-Host "   URL: $($Model.URL)"
    Write-Host "   Destination: $filePath"
    
    try {
        # Use Invoke-WebRequest with progress
        $ProgressPreference = 'Continue'
        Invoke-WebRequest -Uri $Model.URL -OutFile $filePath -UseBasicParsing
        Write-Success "Downloaded $($Model.DisplayName)"
        return $true
    }
    catch {
        Write-Error "Failed to download $($Model.DisplayName): $_"
        return $false
    }
}

function Register-Models {
    Write-Info "Updating model registry..."
    
    $registryPath = Join-Path $ModelsDir "models.json"
    $registry = @{
        models = @()
        registered_at = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
    }
    
    # Scan for actual model files
    $modelFiles = Get-ChildItem $ModelsDir -Filter "*.gguf"
    
    foreach ($file in $modelFiles) {
        # Find matching catalog entry
        $catalogEntry = $ModelCatalog | Where-Object { $_.FileName -eq $file.Name } | Select-Object -First 1
        
        if ($catalogEntry) {
            $registry.models += @{
                name = $catalogEntry.Name
                display_name = $catalogEntry.DisplayName
                file_path = "models/$($file.Name)"
                size_mb = [math]::Round($file.Length / 1MB, 2)
                privacy_tier = "Internal"
                performance_score = $catalogEntry.Performance
                cost_score = $catalogEntry.Cost
                use_cases = $catalogEntry.UseCase
                capabilities = @()
            }
            
            Write-Success "Registered: $($catalogEntry.DisplayName)"
        }
    }
    
    $registry | ConvertTo-Json -Depth 10 | Set-Content $registryPath
    Write-Success "Registry updated: $($registry.models.Count) models registered"
}

# Main execution
Write-Host ""
Write-Host "🚀 NOA ARK OS - Model Downloader" -ForegroundColor Cyan
Write-Host "=" * 60
Write-Host ""

if ($ModelName -eq "all") {
    Write-Info "Downloading ALL models..."
    Write-Host ""
    
    $toDownload = $ModelCatalog | Where-Object { -not $_.Installed }
    $totalSize = ($toDownload | Measure-Object -Property { [double]($_.Size -replace '[^0-9.]') } -Sum).Sum
    
    Write-Warning "This will download $($toDownload.Count) models (~$([math]::Round($totalSize, 1)) GB)"
    Write-Warning "This may take several hours depending on your internet speed."
    Write-Host ""
    
    if (-not $DryRun) {
        $confirm = Read-Host "Continue? (yes/no)"
        if ($confirm -ne "yes") {
            Write-Info "Download cancelled"
            exit 0
        }
    }
    
    Write-Host ""
    $successful = 0
    $failed = 0
    
    foreach ($model in $toDownload) {
        if (Download-Model $model) {
            $successful++
        } else {
            $failed++
        }
        Write-Host ""
    }
    
    Write-Host ""
    Write-Info "Download Summary:"
    Write-Host "  Successful: $successful"
    Write-Host "  Failed: $failed"
    Write-Host ""
    
    if ($successful -gt 0 -and -not $DryRun) {
        Register-Models
    }
}
else {
    # Download specific model
    $model = $ModelCatalog | Where-Object { $_.Name -eq $ModelName } | Select-Object -First 1
    
    if (-not $model) {
        Write-Error "Model not found: $ModelName"
        Write-Info "Available models:"
        $ModelCatalog | ForEach-Object { Write-Host "  - $($_.Name)" }
        exit 1
    }
    
    if (Download-Model $model) {
        if (-not $DryRun) {
            Register-Models
        }
    }
}

Write-Host ""
Write-Success "Model download process complete!"
Write-Host ""
Write-Info "Next steps:"
Write-Host "  1. Start llama.cpp server: .\scripts\dev\start-llama-server.ps1"
Write-Host "  2. Test models with agents"
Write-Host "  3. Model Selector will automatically use registered models"
Write-Host ""
