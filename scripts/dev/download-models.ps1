# Model Download and Management Script
# Downloads GGUF models for llama.cpp from HuggingFace

param(
    [Parameter(Mandatory=$false)]
    [string]$ModelName = "",
    
    [Parameter(Mandatory=$false)]
    [switch]$ListAll = $false,
    
    [Parameter(Mandatory=$false)]
    [switch]$DownloadAll = $false,
    
    [Parameter(Mandatory=$false)]
    [string]$Quantization = "Q4_K_M"
)

$ModelsDir = "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp\models"

function Write-Info { param($Message) Write-Host "ℹ️  $Message" -ForegroundColor Cyan }
function Write-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "⚠️  $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor Red }

# Model database with HuggingFace URLs
$Models = @{
    # Small models (< 1B)
    "gemma-270m" = @{
        name = "Gemma-3-270M-IT"
        url = "https://huggingface.co/google/gemma-3-270m-it-GGUF"
        file = "gemma-3-270m-it.Q4_K_M.gguf"
        size = "~200MB"
        description = "Tiny Google model, very fast"
    }
    "qwen-0.6b" = @{
        name = "Qwen3-0.6B"
        url = "https://huggingface.co/Qwen/Qwen3-0.6B-GGUF"
        file = "qwen3-0.6b.Q4_K_M.gguf"
        size = "~400MB"
        description = "Small but capable Qwen model"
    }
    "tinyllama" = @{
        name = "TinyLlama-1.1B"
        url = "https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF"
        file = "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
        size = "~700MB"
        description = "Very fast, good for testing"
    }
    
    # 3B models
    "smollm3-3b" = @{
        name = "SmolLM3-3B"
        url = "https://huggingface.co/HuggingFaceTB/SmolLM3-3B-Instruct-GGUF"
        file = "SmolLM3-3B-Instruct.Q4_K_M.gguf"
        size = "~2GB"
        description = "HuggingFace small model, well-optimized"
    }
    "stable-code-3b" = @{
        name = "StableCode-3B"
        url = "https://huggingface.co/stabilityai/stable-code-3b-GGUF"
        file = "stable-code-3b.Q4_K_M.gguf"
        size = "~2GB"
        description = "Code-specialized model"
    }
    "llama-3.2-3b" = @{
        name = "Llama-3.2-3B"
        url = "https://huggingface.co/QuantFactory/Llama-3.2-3B-Instruct-GGUF"
        file = "Llama-3.2-3B-Instruct.Q4_K_M.gguf"
        size = "~2GB"
        description = "Latest Llama, currently installed"
    }
    "tiny-agent-3b" = @{
        name = "Tiny-Agent-3B"
        url = "https://huggingface.co/driaforall/Tiny-Agent-a-3B-GGUF"
        file = "Tiny-Agent-a-3B.Q4_K_M.gguf"
        size = "~2GB"
        description = "Agent-specialized model"
    }
    "phi-4-mini" = @{
        name = "Phi-4-Mini-3.8B"
        url = "https://huggingface.co/bartowski/Phi-4-Mini-Instruct-GGUF"
        file = "Phi-4-Mini-Instruct-Q4_K_M.gguf"
        size = "~2.5GB"
        description = "Microsoft's efficient model"
    }
    "qwen2.5-vl-3b" = @{
        name = "Qwen2.5-VL-3B"
        url = "https://huggingface.co/unsloth/Qwen2.5-VL-3B-Instruct-GGUF"
        file = "Qwen2.5-VL-3B-Instruct.Q4_K_M.gguf"
        size = "~2GB"
        description = "Vision-language capable"
    }
    
    # 4-8B models
    "qwen3-4b" = @{
        name = "Qwen3-4B-Instruct"
        url = "https://huggingface.co/Qwen/Qwen3-4B-Instruct-GGUF"
        file = "qwen3-4b-instruct.Q4_K_M.gguf"
        size = "~2.5GB"
        description = "Balanced performance"
    }
    "gemma-4b" = @{
        name = "Gemma-3-4B-IT"
        url = "https://huggingface.co/google/gemma-3-4b-it-GGUF"
        file = "gemma-3-4b-it.Q4_K_M.gguf"
        size = "~2.5GB"
        description = "Google's mid-size model"
    }
    "llama-3.1-8b" = @{
        name = "Llama-3.1-8B"
        url = "https://huggingface.co/QuantFactory/Meta-Llama-3.1-8B-Instruct-GGUF"
        file = "Meta-Llama-3.1-8B-Instruct.Q4_K_M.gguf"
        size = "~5GB"
        description = "Powerful general model"
    }
    "octopus-v2" = @{
        name = "Octopus-v2"
        url = "https://huggingface.co/NexaAI/Octopus-v2-GGUF"
        file = "octopus-v2.Q4_K_M.gguf"
        size = "~2GB"
        description = "Function-calling specialist"
    }
    
    # 7B+ specialized models
    "openthinker-7b" = @{
        name = "OpenThinker3-7B"
        url = "https://huggingface.co/Mungert/OpenThinker3-7B-GGUF"
        file = "OpenThinker3-7B-q8_0.gguf"
        size = "~7GB"
        description = "Reasoning-focused model"
    }
    "mistral-nemo-12b" = @{
        name = "Mistral-Nemo-12B"
        url = "https://huggingface.co/TheBloke/Mistral-Nemo-Instruct-2407-GGUF"
        file = "mistral-nemo-instruct-2407.Q4_K_M.gguf"
        size = "~7GB"
        description = "Latest Mistral model"
    }
    "deepseek-coder-v2" = @{
        name = "DeepSeek-Coder-V2-12B"
        url = "https://huggingface.co/TheBloke/DeepSeek-Coder-V2-Instruct-GGUF"
        file = "deepseek-coder-v2-instruct.Q4_K_M.gguf"
        size = "~7GB"
        description = "Code generation expert"
    }
    
    # Large models (requires more RAM)
    "magistral-small" = @{
        name = "Magistral-Small-2509"
        url = "https://huggingface.co/unsloth/Magistral-Small-2509-GGUF"
        file = "Magistral-Small-2509-UD-Q4_K_XL.gguf"
        size = "~10GB"
        description = "High-quality general model"
    }
    "mistral-small-24b" = @{
        name = "Mistral-Small-24B"
        url = "https://huggingface.co/unsloth/Mistral-Small-3.2-24B-Instruct-2506"
        file = "Mistral-Small-3.2-24B-Instruct-2506-Q4_K_M.gguf"
        size = "~14GB"
        description = "Large capable model"
    }
}

if ($ListAll) {
    Write-Info "Available Models:"
    Write-Info ""
    
    $categories = @{
        "Tiny (< 1GB)" = @("gemma-270m", "qwen-0.6b", "tinyllama")
        "Small (2-3GB)" = @("smollm3-3b", "stable-code-3b", "llama-3.2-3b", "tiny-agent-3b", "phi-4-mini", "qwen2.5-vl-3b")
        "Medium (3-8GB)" = @("qwen3-4b", "gemma-4b", "llama-3.1-8b", "octopus-v2", "openthinker-7b", "mistral-nemo-12b", "deepseek-coder-v2")
        "Large (10GB+)" = @("magistral-small", "mistral-small-24b")
    }
    
    foreach ($category in $categories.Keys | Sort-Object) {
        Write-Host "`n=== $category ===" -ForegroundColor Yellow
        foreach ($key in $categories[$category]) {
            $model = $Models[$key]
            Write-Host "  $key" -ForegroundColor Cyan -NoNewline
            Write-Host " - $($model.name)" -ForegroundColor White
            Write-Host "    Size: $($model.size) | $($model.description)" -ForegroundColor Gray
        }
    }
    
    Write-Info "`nUsage:"
    Write-Info "  .\scripts\dev\download-models.ps1 -ModelName <key>"
    Write-Info "  Example: .\scripts\dev\download-models.ps1 -ModelName smollm3-3b"
    exit 0
}

if ($ModelName -eq "" -and !$DownloadAll) {
    Write-Error "Please specify -ModelName or use -ListAll to see available models"
    Write-Info "Example: .\scripts\dev\download-models.ps1 -ModelName smollm3-3b"
    Write-Info "Or: .\scripts\dev\download-models.ps1 -ListAll"
    exit 1
}

if (!$DownloadAll) {
    if (!$Models.ContainsKey($ModelName)) {
        Write-Error "Model '$ModelName' not found"
        Write-Info "Use -ListAll to see available models"
        exit 1
    }
    
    $model = $Models[$ModelName]
    Write-Info "Downloading: $($model.name)"
    Write-Info "Size: $($model.size)"
    Write-Info "Description: $($model.description)"
    Write-Info ""
    
    $downloadUrl = "$($model.url)/resolve/main/$($model.file)"
    $outputPath = Join-Path $ModelsDir $model.file
    
    if (Test-Path $outputPath) {
        Write-Warning "Model already exists: $outputPath"
        $response = Read-Host "Overwrite? (y/n)"
        if ($response -ne "y") {
            Write-Info "Download cancelled"
            exit 0
        }
    }
    
    Write-Info "Downloading from: $downloadUrl"
    Write-Info "To: $outputPath"
    Write-Info "This may take a while..."
    
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $outputPath
        Write-Success "Model downloaded successfully!"
        Write-Info ""
        Write-Info "To use this model, update start-llama-server.ps1:"
        Write-Info "  `$ModelPath = Join-Path `$LlamaCppDir `"models\$($model.file)`""
    } catch {
        Write-Error "Download failed: $($_.Exception.Message)"
        Write-Info ""
        Write-Info "Manual download URL:"
        Write-Info "  $($model.url)"
        exit 1
    }
}

Write-Success "Model management complete!"
