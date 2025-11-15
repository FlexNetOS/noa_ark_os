# Automated Model Registration System
# Scans models directory and auto-registers with ModelSelector

param(
    [Parameter(Mandatory=$false)]
    [switch]$DryRun = $false,
    
    [Parameter(Mandatory=$false)]
    [switch]$ShowDetails = $false
)

$ModelsDir = "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp\models"
$ConfigFile = "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp\models\models.json"

function Write-Info { param($Message) Write-Host "ℹ️  $Message" -ForegroundColor Cyan }
function Write-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "⚠️  $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor Red }

# Model metadata database
$ModelDatabase = @{
    # Code-specialized models
    "deepseek-coder" = @{
        use_cases = @("CodeGeneration", "CodeAnalysis")
        performance = 0.9
        cost = 0.6
        privacy = "Internal"
    }
    "stable-code" = @{
        use_cases = @("CodeGeneration")
        performance = 0.85
        cost = 0.7
        privacy = "Internal"
    }
    
    # Reasoning models
    "openthinker" = @{
        use_cases = @("Reasoning", "QuestionAnswering")
        performance = 0.9
        cost = 0.5
        privacy = "Internal"
    }
    
    # Agent-specialized
    "octopus" = @{
        use_cases = @("AgentTask", "FunctionCalling")
        performance = 0.85
        cost = 0.8
        privacy = "Internal"
    }
    "tiny-agent" = @{
        use_cases = @("AgentTask")
        performance = 0.8
        cost = 0.9
        privacy = "Internal"
    }
    
    # General purpose
    "llama-3.2" = @{
        use_cases = @("General", "QuestionAnswering")
        performance = 0.85
        cost = 0.7
        privacy = "Internal"
    }
    "llama-3.1" = @{
        use_cases = @("General", "QuestionAnswering", "Reasoning")
        performance = 0.9
        cost = 0.5
        privacy = "Internal"
    }
    "smollm" = @{
        use_cases = @("General")
        performance = 0.75
        cost = 0.95
        privacy = "Internal"
    }
    "phi-4" = @{
        use_cases = @("General", "Reasoning")
        performance = 0.85
        cost = 0.8
        privacy = "Internal"
    }
    
    # Multimodal
    "qwen2.5-vl" = @{
        use_cases = @("Vision", "General")
        performance = 0.8
        cost = 0.7
        privacy = "Internal"
    }
    
    # Specialized
    "mistral-nemo" = @{
        use_cases = @("General", "QuestionAnswering")
        performance = 0.9
        cost = 0.6
        privacy = "Internal"
    }
}

Write-Info "Scanning models directory: $ModelsDir"

$models = Get-ChildItem $ModelsDir -Filter "*.gguf" -ErrorAction SilentlyContinue

if ($models.Count -eq 0) {
    Write-Warning "No GGUF models found in $ModelsDir"
    Write-Info "Download models with: .\scripts\dev\download-models.ps1 -ListAll"
    exit 0
}

Write-Success "Found $($models.Count) model(s)"

$registeredModels = @()

foreach ($model in $models) {
    $sizeMB = [math]::Round($model.Length / 1MB, 0)
    $modelName = $model.BaseName
    
    if ($ShowDetails) {
        Write-Info "Processing: $($model.Name) ($sizeMB MB)"
    }
    
    # Try to match model name to database
    $metadata = $null
    foreach ($key in $ModelDatabase.Keys) {
        if ($modelName -like "*$key*") {
            $metadata = $ModelDatabase[$key]
            break
        }
    }
    
    if ($null -eq $metadata) {
        # Default metadata for unknown models
        $metadata = @{
            use_cases = @("General")
            performance = 0.7
            cost = 0.8
            privacy = "Internal"
        }
        if ($ShowDetails) {
            Write-Warning "  No metadata found, using defaults"
        }
    }
    
    $modelInfo = @{
        name = $modelName
        file_path = "models/$($model.Name)"
        size_mb = $sizeMB
        capabilities = @()
        performance_score = $metadata.performance
        cost_score = $metadata.cost
        privacy_tier = $metadata.privacy
        use_cases = $metadata.use_cases
    }
    
    $registeredModels += $modelInfo
    
    if ($ShowDetails) {
        Write-Host "  Use cases: $($metadata.use_cases -join ', ')" -ForegroundColor Gray
        Write-Host "  Performance: $($metadata.performance) | Cost: $($metadata.cost)" -ForegroundColor Gray
    }
}

Write-Info ""
Write-Info "Registration Summary:"
Write-Info "  Total models: $($registeredModels.Count)"
Write-Info "  Total size: $([math]::Round(($models | Measure-Object -Property Length -Sum).Sum / 1GB, 2)) GB"

# Group by use case
$byUseCase = @{}
foreach ($model in $registeredModels) {
    foreach ($useCase in $model.use_cases) {
        if (!$byUseCase.ContainsKey($useCase)) {
            $byUseCase[$useCase] = @()
        }
        $byUseCase[$useCase] += $model.name
    }
}

Write-Info ""
Write-Info "Models by use case:"
foreach ($useCase in $byUseCase.Keys | Sort-Object) {
    $count = $byUseCase[$useCase].Count
    Write-Host "  $useCase`: $count model(s)" -ForegroundColor Yellow
}

if (!$DryRun) {
    # Save configuration
    $config = @{
        registered_at = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
        models = $registeredModels
    }
    
    $json = $config | ConvertTo-Json -Depth 10
    Set-Content -Path $ConfigFile -Value $json -Encoding UTF8
    
    Write-Success ""
    Write-Success "Configuration saved to: $ConfigFile"
    Write-Info ""
    Write-Info "Models are now registered and ready for automatic selection!"
    Write-Info ""
    Write-Info "The ModelSelector agent will automatically choose the best model for each task based on:"
    Write-Info "  • Use case (code, reasoning, agent tasks, etc.)"
    Write-Info "  • Performance score"
    Write-Info "  • Cost efficiency"
    Write-Info "  • Privacy requirements"
    Write-Info "  • Historical success rates"
} else {
    Write-Info ""
    Write-Info "[DRY RUN] Would save configuration to: $ConfigFile"
}

Write-Info ""
Write-Info "Next steps:"
Write-Info "  1. Models are registered: ✅"
Write-Info "  2. Start llama server: .\scripts\dev\start-llama-server.ps1"
Write-Info "  3. Use ModelSelector in your code: let selector = ModelSelectorAgent::new();"
Write-Info "  4. Automatic model selection will optimize for each task!"
