# NOA ARK OS - Activate Cargo (portable or system)
# Usage: .\server\tools\activate-cargo.ps1

param()

$IsSilent = $false
if ($env:NOA_CARGO_ACTIVATE_SILENT -eq '1') {
    $IsSilent = $true
}

function Write-Log {
    param(
        [string]$Message,
        [ConsoleColor]$Color = [ConsoleColor]::Gray
    )

    if (-not $IsSilent) {
        Write-Host $Message -ForegroundColor $Color
    }
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$WORKSPACE_ROOT = Split-Path -Parent (Split-Path -Parent $ScriptDir)
$DevShellState = Join-Path $WORKSPACE_ROOT "tools/devshell/state"
New-Item -ItemType Directory -Force -Path $DevShellState | Out-Null

$PortableCargoHome = Join-Path $WORKSPACE_ROOT "server/tools/cargo-portable"
$PortableRustupHome = Join-Path $WORKSPACE_ROOT "server/tools/rustup-portable"
$PortableCargoExe = Join-Path $PortableCargoHome "bin/cargo.exe"
$PortableDetected = Test-Path $PortableCargoExe

$SystemRustup = Get-Command rustup -ErrorAction SilentlyContinue
$SystemRustupAvailable = $null -ne $SystemRustup

$Mode = $null
$PathModified = $false

Write-Log "`n🔧 Activating Cargo environment..." ([ConsoleColor]::Cyan)
Write-Log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" ([ConsoleColor]::DarkGray)

if ($PortableDetected) {
    $Mode = 'portable'
    $env:CARGO_HOME = $PortableCargoHome
    $env:RUSTUP_HOME = $PortableRustupHome
    if ($env:Path -notlike "*$PortableCargoHome\bin*") {
        $env:Path = "$PortableCargoHome\bin;$($env:Path)"
        $PathModified = $true
    }
} else {
    $Mode = 'system'
    if (-not $env:CARGO_HOME) {
        $env:CARGO_HOME = Join-Path $HOME '.cargo'
    }

    if (-not $env:RUSTUP_HOME) {
        if ($SystemRustupAvailable) {
            try {
                $env:RUSTUP_HOME = (& rustup show home).Trim()
            } catch {
                $env:RUSTUP_HOME = Join-Path $HOME '.rustup'
            }
        } else {
            $env:RUSTUP_HOME = Join-Path $HOME '.rustup'
        }
    }

    $CargoBin = Join-Path $env:CARGO_HOME 'bin'
    if ($env:Path -notlike "*$CargoBin*") {
        $env:Path = "$CargoBin;$($env:Path)"
        $PathModified = $true
    }
}

$CargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $CargoCommand) {
    Write-Log "`n❌ ERROR: Cargo could not be found in the configured environment." ([ConsoleColor]::Red)
    if ($Mode -eq 'portable') {
        Write-Log "`nExpected location: $PortableCargoExe" ([ConsoleColor]::Yellow)
        Write-Log "`nPlease run setup first:" ([ConsoleColor]::Yellow)
        Write-Log "   .\server\tools\setup-portable-cargo.ps1" ([ConsoleColor]::Cyan)
    } else {
        Write-Log "`nInstall Rustup with:" ([ConsoleColor]::Yellow)
        Write-Log "   powershell -ExecutionPolicy Bypass -Command \"& { iwr https://win.rustup.rs/x86_64 -OutFile rustup-init.exe; .\\rustup-init.exe -y }\"" ([ConsoleColor]::Cyan)
    }
    if ($IsSilent) {
        throw "Cargo not found"
    } else {
        exit 1
    }
}

$Timestamp = (Get-Date).ToUniversalTime().ToString('s') + 'Z'
$StatusJson = Join-Path $DevShellState 'cargo-env.json'
$StatusYaml = Join-Path $DevShellState 'cargo-env.yaml'

$status = [ordered]@{
    timestamp = $Timestamp
    mode = $Mode
    workspace_root = $WORKSPACE_ROOT
    cargo_home = $env:CARGO_HOME
    rustup_home = $env:RUSTUP_HOME
    path_modified = $PathModified
    portable_detected = $PortableDetected
    system_rustup_available = $SystemRustupAvailable
}

$status | ConvertTo-Json -Depth 2 | Set-Content -Path $StatusJson -Encoding UTF8

function Convert-ToYamlValue {
    param([object]$Value)

    if ($Value -is [bool]) {
        return $Value.ToString().ToLower()
    }

    $text = [string]$Value
    $escaped = $text -replace "'", "''"
    return "'$escaped'"
}

$yamlLines = @()
foreach ($key in $status.Keys) {
    $yamlLines += "$key: $(Convert-ToYamlValue $status[$key])"
}
$yamlLines | Set-Content -Path $StatusYaml -Encoding UTF8

if (-not $IsSilent) {
    Write-Log "`n✅ Cargo environment ready ($Mode)" ([ConsoleColor]::Green)
    Write-Log "`nEnvironment:" ([ConsoleColor]::Yellow)
    Write-Log "  CARGO_HOME   = $($env:CARGO_HOME)" ([ConsoleColor]::Cyan)
    Write-Log "  RUSTUP_HOME  = $($env:RUSTUP_HOME)" ([ConsoleColor]::Cyan)
    if ($PathModified) {
        Write-Log "  PATH         = [$($env:CARGO_HOME)\bin prepended]" ([ConsoleColor]::Cyan)
    } else {
        Write-Log "  PATH         = [unchanged]" ([ConsoleColor]::Cyan)
    }

    Write-Log "`nVersions:" ([ConsoleColor]::Yellow)
    & cargo --version
    $RustcCommand = Get-Command rustc -ErrorAction SilentlyContinue
    if ($RustcCommand) {
        & rustc --version
    } else {
        Write-Log "rustc not found in PATH" ([ConsoleColor]::Yellow)
    }

    Write-Log "`n💡 Tips:" ([ConsoleColor]::DarkGray)
    Write-Log "  • Run 'cargo build' to build projects" ([ConsoleColor]::DarkGray)
    Write-Log "  • Run 'cargo run' to run projects" ([ConsoleColor]::DarkGray)
    Write-Log "  • Run 'cargo test' to run tests" ([ConsoleColor]::DarkGray)
    Write-Log "  • This activation is for the current PowerShell session only" ([ConsoleColor]::DarkGray)
    Write-Log ""
}
