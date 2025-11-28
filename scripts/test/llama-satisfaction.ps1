# Llama.cpp Satisfaction Test
[CmdletBinding()]
param(
    [Parameter(Mandatory=$false)]
    [string]$WorkspaceRoot,

    [Parameter(Mandatory=$false)]
    [string]$ModelFile = "Llama-3.2-3B-Instruct.Q4_K_M.gguf",

    [Parameter(Mandatory=$false)]
    [int]$Port = 18115,

    [Parameter(Mandatory=$false)]
    [int]$ServerStartTimeoutSeconds = 120,

    [Parameter(Mandatory=$false)]
    [string]$ReportPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$hostIsWindows = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)
$hostIsLinux = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Linux)

if (-not $WorkspaceRoot) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $WorkspaceRoot = (Resolve-Path -Path (Join-Path $scriptRoot "..\..")).Path
} else {
    $WorkspaceRoot = (Resolve-Path -Path $WorkspaceRoot).Path
}
$llamaDir = Join-Path $WorkspaceRoot "server/ai/llama-cpp"
$binDir = Join-Path $llamaDir "bin"
$modelsDir = Join-Path $llamaDir "models"
$logsDir = Join-Path $llamaDir "logs"

if (-not (Test-Path $logsDir)) {
    New-Item -ItemType Directory -Force -Path $logsDir | Out-Null
}

$binaryName = if ($hostIsWindows) { "llama-server.exe" } else { "llama-server" }
$llamaBinary = Join-Path $binDir $binaryName

function New-OrderedHash {
    param([hashtable]$Source)
    $ordered = [ordered]@{}
    foreach ($key in $Source.Keys) {
        $ordered[$key] = $Source[$key]
    }
    return $ordered
}

$results = [ordered]@{}

function Set-Result {
    param(
        [string]$Key,
        [int]$Percent,
        [string]$Detail
    )
    $results[$Key] = [ordered]@{
        percent = $Percent
        detail = $Detail
    }
}

function Wait-LlamaHealth {
    param(
        [string]$Uri,
        [int]$TimeoutSeconds
    )
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        try {
            $response = Invoke-WebRequest -Uri $Uri -UseBasicParsing -TimeoutSec 5
            if ($response.StatusCode -eq 200) {
                return $true
            }
        } catch {
            Start-Sleep -Seconds 2
        }
    }
    return $false
}

function Invoke-LlamaCompletion {
    param(
        [string]$Uri,
        [int]$TimeoutSeconds
    )
    $prompt = "Respond with the single word READY."
    $body = @{
        prompt = $prompt
        temperature = 0.0
        max_tokens = 16
    } | ConvertTo-Json -Compress

    $response = Invoke-RestMethod -Uri $Uri -Method Post -Body $body -ContentType "application/json" -TimeoutSec $TimeoutSeconds
    if ($null -ne $response) {
        $text = $response.content
        if (-not $text -and $response.choices) {
            $text = $response.choices[0].text
        }
        return $text
    }
    return $null
}

# Healthy check
$binaryExists = Test-Path $llamaBinary
$models = @(Get-ChildItem -Path $modelsDir -Filter *.gguf -ErrorAction SilentlyContinue)
if (-not $ModelFile -and $models.Count -gt 0) {
    $ModelFile = $models[0].Name
}
$modelPath = if ($ModelFile) { Join-Path $modelsDir $ModelFile } else { $null }
if ($modelPath -and -not (Test-Path $modelPath) -and $models.Count -gt 0) {
    $modelPath = $models[0].FullName
    $ModelFile = Split-Path $modelPath -Leaf
}
$modelExists = $modelPath -and (Test-Path $modelPath)

if ($binaryExists -and $modelExists) {
    $binaryInfo = Get-Item $llamaBinary
    $detail = "Binary $(Split-Path $llamaBinary -Leaf) ($( [math]::Round($binaryInfo.Length / 1MB, 1) ) MB) and model $(Split-Path $modelPath -Leaf) located."
    Set-Result -Key "healthy" -Percent 100 -Detail $detail
} else {
    $detail = if (-not $binaryExists) {
        "Missing llama binary at $llamaBinary"
    } elseif (-not $modelsDir -or $models.Count -eq 0) {
        "No GGUF models found in $modelsDir"
    } else {
        "Requested model $ModelFile not found"
    }
    Set-Result -Key "healthy" -Percent 0 -Detail $detail
}

# Activated check
$llamaSentinel = Join-Path $WorkspaceRoot "build_output/system-launch/llama-status.json"
if (Test-Path $llamaSentinel) {
    try {
        $sentinel = Get-Content $llamaSentinel -Raw | ConvertFrom-Json
        $recordedAtLocal = $sentinel | Select-Object -ExpandProperty recorded_at_local -ErrorAction SilentlyContinue
        $recordedAtRaw = $sentinel | Select-Object -ExpandProperty recorded_at -ErrorAction SilentlyContinue
        $recordedAtUtc = $sentinel | Select-Object -ExpandProperty recorded_at_utc -ErrorAction SilentlyContinue
        $modelSize = $sentinel | Select-Object -ExpandProperty model_size -ErrorAction SilentlyContinue
        $timestampCandidates = @($recordedAtLocal, $recordedAtRaw, $recordedAtUtc) | Where-Object { $_ }
        $recordedAt = $null
        foreach ($candidate in $timestampCandidates) {
            try {
                $recordedAt = [System.DateTimeOffset]::Parse($candidate, $null, [System.Globalization.DateTimeStyles]::RoundtripKind)
                break
            } catch {
                continue
            }
        }
        if (-not $recordedAt) {
            throw "no parseable timestamp in sentinel"
        }
        $ageSpan = [System.DateTimeOffset]::Now - $recordedAt
        $ageHours = [math]::Round($ageSpan.TotalHours, 2)
        $signedAge = "{0:+0.00;-0.00;+0.00}" -f $ageHours
        $detail = "Sentinel recorded ${signedAge}h ago (model_size=${modelSize})."
        Set-Result -Key "activated" -Percent 100 -Detail $detail
    } catch {
        Set-Result -Key "activated" -Percent 0 -Detail "Failed to parse sentinel $($llamaSentinel): $_"
    }
} else {
    Set-Result -Key "activated" -Percent 0 -Detail "Llama sentinel missing at $llamaSentinel"
}

# Connected check
$connectionPercent = 0
$connectionDetail = "Skipped because healthy check failed"
$serverProcess = $null
$healthUri = "http://127.0.0.1:$Port/health"
$completionUri = "http://127.0.0.1:$Port/completion"
$logFile = Join-Path $logsDir ("llama-test-" + (Get-Date -Format "yyyyMMddHHmmss") + ".log")

if ($results.healthy.percent -eq 100) {
    try {
        $args = @(
            "--model", $modelPath,
            "--host", "127.0.0.1",
            "--port", $Port.ToString(),
            "--ctx-size", "2048",
            "--threads", "4",
            "--n-gpu-layers", "0",
            "--log-file", $logFile
        )
        $serverProcess = Start-Process -FilePath $llamaBinary -ArgumentList $args -PassThru -WorkingDirectory $binDir
        $healthOk = Wait-LlamaHealth -Uri $healthUri -TimeoutSeconds $ServerStartTimeoutSeconds
        if ($healthOk) {
            $completion = Invoke-LlamaCompletion -Uri $completionUri -TimeoutSeconds 60
            if ($completion) {
                $snippet = $completion.Trim()
                if ($snippet.Length -gt 64) {
                    $snippet = $snippet.Substring(0, 64) + "..."
                }
                $connectionPercent = 100
                $connectionDetail = "Health probe + completion succeeded (response='$snippet')."
            } else {
                $connectionDetail = "Health endpoint succeeded but completion response was empty."
            }
        } else {
            $connectionDetail = "Health endpoint $healthUri did not respond within $ServerStartTimeoutSeconds seconds."
        }
    } catch {
        $connectionDetail = "Connection test failed: $_"
    } finally {
        if ($serverProcess -and -not $serverProcess.HasExited) {
            try {
                Stop-Process -Id $serverProcess.Id -Force -ErrorAction SilentlyContinue
            } catch {
                # ignore cleanup errors
            }
        }
    }
}

Set-Result -Key "connected" -Percent $connectionPercent -Detail $connectionDetail

$allGreen = ($results.healthy.percent -eq 100 -and $results.activated.percent -eq 100 -and $results.connected.percent -eq 100)
$satisfactionPercent = if ($allGreen) { 100 } else { 0 }
$results.satisfaction = [ordered]@{
    percent = $satisfactionPercent
    definition = "Satisfaction requires healthy=100%, activated=100%, connected=100%."
}
$results.environment = [ordered]@{
    workspace = $WorkspaceRoot
    binary = $llamaBinary
    model = $modelPath
    log = $logFile
    port = $Port
}
$results.timestamp = (Get-Date).ToString("o")

Write-Host ""
Write-Host "LLAMA.CPP SATISFACTION TEST" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
foreach ($key in @("healthy","activated","connected")) {
    $entry = $results[$key]
    $statusIcon = if ($entry.percent -eq 100) { "[OK]" } else { "[FAIL]" }
    Write-Host ("{0} {1,-11}: {2}% - {3}" -f $statusIcon, ($key.ToUpper()), $entry.percent, $entry.detail)
}
Write-Host "-----------------------------------------"
if ($results.satisfaction.percent -eq 100) {
    $statusColor = "Green"
} else {
    $statusColor = "Yellow"
}
Write-Host ("Satisfaction: {0}% (requires all checks at 100%)" -f $results.satisfaction.percent) -ForegroundColor $statusColor
Write-Host "Report log: $logFile"
Write-Host ""

if (-not $ReportPath) {
    $ReportPath = Join-Path $WorkspaceRoot "build_output/system-launch/llama-test-report.json"
}

$results | ConvertTo-Json -Depth 4 | Set-Content -Encoding UTF8 -Path $ReportPath
Write-Host "Report saved to $ReportPath"
