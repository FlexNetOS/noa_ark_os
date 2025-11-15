# Update Visual Studio Copilot Settings Safely
# Enhances GitHub Copilot capabilities for NOA ARK OS workspace

$SettingsPath = "C:\Users\De-Flex.Net\AppData\Local\Microsoft\VisualStudio\18.0_0867e4d5\settings.json"
$BackupPath = "C:\Users\De-Flex.Net\AppData\Local\Microsoft\VisualStudio\18.0_0867e4d5\settings.backup-$(Get-Date -Format 'yyyyMMdd-HHmmss').json"

function Write-Info { param($Message) Write-Host "ℹ️  $Message" -ForegroundColor Cyan }
function Write-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "⚠️  $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor Red }

Write-Info "Visual Studio Copilot Settings Updater"
Write-Info "Settings: $SettingsPath"

# Backup current settings
Write-Info "Creating backup..."
Copy-Item -Path $SettingsPath -Destination $BackupPath -Force
Write-Success "Backup created: $BackupPath"

# Load current settings
Write-Info "Loading current settings..."
try {
    $settings = Get-Content $SettingsPath -Raw | ConvertFrom-Json
    Write-Success "Settings loaded successfully"
} catch {
    Write-Error "Failed to parse settings.json"
    Write-Error $_.Exception.Message
    exit 1
}

# Update settings
Write-Info "Applying enhancements..."

# 1. Increase max iterations
if ($settings.'copilot.general.chat.maxFunctionCallIterations' -lt 150) {
    Write-Info "Increasing max iterations to 150..."
    $settings | Add-Member -NotePropertyName 'copilot.general.chat.maxFunctionCallIterations' -NotePropertyValue 150 -Force
}

# 2. Enhance tool settings
Write-Info "Updating tool auto-execution settings..."
$toolSettings = @(
    @{
        toolName = "run_command_in_terminal"
        autoExecutionMode = "Always"
        resetPerSolution = $false
    },
    @{
        toolName = "edit_file"
        autoExecutionMode = "Always"
        resetPerSolution = $false
    },
    @{
        toolName = "create_file"
        autoExecutionMode = "Always"
        resetPerSolution = $false
    },
    @{
        toolName = "run_build"
        autoExecutionMode = "Always"
        resetPerSolution = $false
    },
    @{
        toolName = "get_file"
        autoExecutionMode = "Always"
        resetPerSolution = $false
    },
    @{
        toolName = "code_search"
        autoExecutionMode = "Always"
        resetPerSolution = $false
    }
)

$settings | Add-Member -NotePropertyName 'copilot.general.tools.toolSettings' -NotePropertyValue $toolSettings -Force

# 3. Ensure custom instructions are enabled
if (-not $settings.'copilot.general.chat.enableCustomInstructionsFiles') {
    Write-Info "Enabling custom instructions files..."
    $settings | Add-Member -NotePropertyName 'copilot.general.chat.enableCustomInstructionsFiles' -NotePropertyValue $true -Force
}

# 4. Verify preferred model
if ($settings.'copilot.general.chat.preferredModelFamily' -ne 'claude-sonnet-4.5') {
    Write-Warning "Preferred model is: $($settings.'copilot.general.chat.preferredModelFamily')"
    Write-Info "Consider using: claude-sonnet-4.5"
}

# Save settings
Write-Info "Saving enhanced settings..."
try {
    $json = $settings | ConvertTo-Json -Depth 10
    $json | Set-Content -Path $SettingsPath -Encoding UTF8 -Force
    Write-Success "Settings saved successfully"
} catch {
    Write-Error "Failed to save settings"
    Write-Error $_.Exception.Message
    Write-Warning "Restoring from backup..."
    Copy-Item -Path $BackupPath -Destination $SettingsPath -Force
    exit 1
}

# Validate
Write-Info "Validating settings..."
try {
    $null = Get-Content $SettingsPath -Raw | ConvertFrom-Json
    Write-Success "Settings validation passed"
} catch {
    Write-Error "Settings validation failed"
    Write-Warning "Restoring from backup..."
    Copy-Item -Path $BackupPath -Destination $SettingsPath -Force
    exit 1
}

Write-Success "Settings updated successfully!"
Write-Info ""
Write-Info "Enhancements Applied:"
Write-Info "  ✅ Max iterations: 150"
Write-Info "  ✅ Auto-execution: 6 tools enabled"
Write-Info "  ✅ Custom instructions: Enabled"
Write-Info ""
Write-Warning "⚠️  Restart Visual Studio to apply changes"
Write-Info ""
Write-Info "Backup location: $BackupPath"
