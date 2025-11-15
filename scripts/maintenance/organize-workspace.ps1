# Workspace Organization Script
# Cleans up root directory and organizes files into proper structure

param(
    [Parameter(Mandatory=$false)]
    [switch]$WhatIf = $false,  # Dry run mode
    
    [Parameter(Mandatory=$false)]
    [switch]$Backup = $true    # Create backup before moving
)

$WorkspaceRoot = "D:\dev\workspaces\noa_ark_os"
$BackupDir = Join-Path $WorkspaceRoot ".workspace-backup-$(Get-Date -Format 'yyyyMMdd-HHmmss')"

function Write-Info { param($Message) Write-Host "ℹ️  $Message" -ForegroundColor Cyan }
function Write-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "⚠️  $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor Red }

Write-Info "Workspace Organization Script"
Write-Info "Workspace: $WorkspaceRoot"
if ($WhatIf) {
    Write-Warning "DRY RUN MODE - No files will be moved"
}

# Change to workspace
Set-Location $WorkspaceRoot

# Phase 1: Create Directory Structure
Write-Info "Phase 1: Creating directory structure..."

$directories = @(
    "docs/notes",
    "docs/audits/security",
    "docs/audits/performance",
    "docs/audits/dependency",
    "docs/reports/sessions",
    "docs/reports/builds",
    "docs/reports/integrations",
    "docs/reports/analysis",
    "docs/links",
    "docs/projects/milestones",
    "docs/projects/sprints",
    "docs/references/rust",
    "docs/references/python",
    "docs/references/go",
    "docs/references/architecture",
    "docs/tests/test-plans",
    "docs/tests/test-results",
    "docs/tests/coverage-reports",
    "docs/guides",
    "docs/archive",
    "scripts/build",
    "scripts/dev",
    "scripts/integration",
    "scripts/maintenance",
    "scripts/tools",
    "scripts/testing",
    "logs/app/server",
    "logs/app/agents",
    "logs/app/crc",
    "logs/build",
    "logs/test",
    "logs/system"
)

foreach ($dir in $directories) {
    if (!$WhatIf) {
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
    }
    Write-Success "Created: $dir"
}

# Phase 2: Create README files
Write-Info "Phase 2: Creating README files..."

$readmes = @{
    "docs/README.md" = "# Documentation`n`nComplete documentation for NOA ARK OS.`n`n## Structure`n- notes/ - Development notes`n- audits/ - Security and code audits`n- reports/ - Session and integration reports`n- links/ - External references`n- projects/ - Project planning`n- references/ - Technical references`n- tests/ - Test documentation`n- guides/ - User and developer guides`n- archive/ - Old documentation"
    
    "docs/reports/README.md" = "# Reports`n`n## Structure`n- sessions/ - Session completion reports`n- builds/ - Build status reports`n- integrations/ - Integration reports`n- analysis/ - Analysis and review reports"
    
    "docs/guides/README.md" = "# Guides`n`nUser and developer documentation.`n`n- GETTING_STARTED.md - Getting started guide`n- CONTRIBUTING.md - How to contribute`n- Troubleshooting guides`n- Integration guides"
    
    "scripts/README.md" = "# Scripts`n`nAutomation scripts for NOA ARK OS.`n`n## Structure`n- build/ - Build automation`n- dev/ - Development helpers`n- integration/ - Integration scripts`n- maintenance/ - Cleanup and backup`n- tools/ - Tool configuration`n- testing/ - Test runners`n`n## Usage`nAlways run from workspace root.`nActivate portable Cargo first for Rust scripts."
    
    "logs/README.md" = "# Logs`n`nLog files directory.`n`n**Note**: All log files are gitignored. Only directory structure is tracked.`n`n## Structure`n- app/ - Application logs`n  - server/ - Server logs`n  - agents/ - Agent execution logs`n  - crc/ - CRC processing logs`n- build/ - Build logs`n- test/ - Test logs`n- system/ - System logs`n`n## Retention`n- Development: 7 days`n- Production: 30 days`n- Rotation: Daily or by size (100MB)"
}

foreach ($readme in $readmes.Keys) {
    if (!$WhatIf) {
        $readmes[$readme] | Set-Content -Path $readme -Encoding UTF8
    }
    Write-Success "Created: $readme"
}

# Phase 3: Move Documentation
Write-Info "Phase 3: Moving documentation files..."

$docMoves = @{
    # Session reports
    "SESSION_COMPLETE.md" = "docs/reports/sessions/"
    "MISSION_ACCOMPLISHED.md" = "docs/reports/sessions/"
    "GITHUB_SYNC_COMPLETE.md" = "docs/reports/sessions/"
    "FINAL_SYNC_STATUS.md" = "docs/reports/sessions/"
    "ALL_ISSUES_RESOLVED.md" = "docs/reports/sessions/"
    "WORKSPACE_COMPLETE.md" = "docs/reports/sessions/"
    "WORKSPACE_FLOW.md" = "docs/reports/sessions/"
    "WORKSPACE_ORGANIZATION_COMPLETE.md" = "docs/reports/sessions/"
    "WORKSPACE_PHASE3_COMPLETE.md" = "docs/reports/sessions/"
    "WORKSPACE_PHASE4_COMPLETE.md" = "docs/reports/sessions/"
    
    # Build reports
    "BUILD_SUCCESS_STATUS.md" = "docs/reports/builds/"
    "BUILD_STATUS_FINAL.md" = "docs/reports/builds/"
    "BUILD_AFTER_INTEGRATION.md" = "docs/reports/builds/"
    "BUILD_ERROR_FIXED.md" = "docs/reports/builds/"
    "BUILD_FAILURE_ANALYSIS.md" = "docs/reports/builds/"
    "BUILD_FIX_APPLIED.md" = "docs/reports/builds/"
    "BUILD_FIX_STATUS.md" = "docs/reports/builds/"
    "BUILD_READY.md" = "docs/reports/builds/"
    "ALL_BUILD_ERRORS_FIXED.md" = "docs/reports/builds/"
    "REMAINING_BUILD_FIXES.md" = "docs/reports/builds/"
    
    # Integration reports
    "AGENT_INTEGRATION_PHASE1_COMPLETE.md" = "docs/reports/integrations/"
    "AGENT_INTEGRATION_SUMMARY.md" = "docs/reports/integrations/"
    "AGENT_REGISTRY_DROP_INTEGRATION_COMPLETE.md" = "docs/reports/integrations/"
    "COMPREHENSIVE_INTEGRATION_PLAN.md" = "docs/reports/integrations/"
    "INTEGRATION_READY.md" = "docs/reports/integrations/"
    "INTEGRATION_SUCCESS_26_AGENTS.md" = "docs/reports/integrations/"
    "PHASE2_INTEGRATION_PLAN.md" = "docs/reports/integrations/"
    "QUICK_START_INTEGRATION.md" = "docs/reports/integrations/"
    "CRC_AUTOMATION_COMPLETE.md" = "docs/reports/integrations/"
    "CRC_CI_CD_COMPLETE.md" = "docs/reports/integrations/"
    "CRC_DROP_IN_VERIFICATION_REPORT.md" = "docs/reports/integrations/"
    "CRC_IMPROVEMENTS_COMPLETE.md" = "docs/reports/integrations/"
    "CRC_TEST_STATUS.md" = "docs/reports/integrations/"
    
    # Analysis reports
    "OLD_WORK_ANALYSIS.md" = "docs/reports/analysis/"
    "MERGE_STRATEGY_GUIDE.md" = "docs/reports/analysis/"
    
    # Guides
    "WSL_CODE_DROP_GUIDE.md" = "docs/guides/"
    "MANUAL_CRC_FLOW.md" = "docs/guides/"
    
    # Archive old docs
    "AGENTASKIT_INTEGRATION_PLAN.md" = "docs/archive/"
    "COMPLETE_SYSTEM_DEMO_CREATED.md" = "docs/archive/"
    "SERVER_BUILD_SPEC_COMPLETE.md" = "docs/archive/"
    "SETUP_COMPLETE.md" = "docs/archive/"
    "OPTION_3_READY.md" = "docs/archive/"
    "OPTION_A_COMPLETE.md" = "docs/archive/"
    "OPTION3_5_CYCLES_COMPLETE.md" = "docs/archive/"
    "NOA_ARK_OS_SELECTIVE_DROP_INTEGRATION_COMPLETE.md" = "docs/archive/"
    "SERVER_WSL_DROP_INTEGRATION_COMPLETE.md" = "docs/archive/"
    "TASK_EXEC_KIT_DROP_INTEGRATION_COMPLETE.md" = "docs/archive/"
    "TOOLS_DROP_INTEGRATION_COMPLETE.md" = "docs/archive/"
    "SECRETS_MANAGEMENT.md" = "docs/archive/"
}

foreach ($file in $docMoves.Keys) {
    if (Test-Path $file) {
        $dest = $docMoves[$file]
        if (!$WhatIf) {
            Move-Item -Path $file -Destination $dest -Force
        }
        Write-Success "Moved: $file -> $dest"
    } else {
        Write-Warning "Not found: $file"
    }
}

# Move existing docs content
if (Test-Path "docs/ARCHITECTURE.md") {
    if (!$WhatIf) { Move-Item "docs/ARCHITECTURE.md" "docs/references/architecture/" -Force }
    Write-Success "Moved: docs/ARCHITECTURE.md -> docs/references/architecture/"
}
if (Test-Path "docs/GETTING_STARTED.md") {
    if (!$WhatIf) { Move-Item "docs/GETTING_STARTED.md" "docs/guides/" -Force }
    Write-Success "Moved: docs/GETTING_STARTED.md -> docs/guides/"
}
if (Test-Path "docs/INTEGRATION.md") {
    if (!$WhatIf) { Move-Item "docs/INTEGRATION.md" "docs/references/architecture/" -Force }
    Write-Success "Moved: docs/INTEGRATION.md -> docs/references/architecture/"
}
if (Test-Path "docs/ROADMAP.md") {
    if (!$WhatIf) { Move-Item "docs/ROADMAP.md" "docs/projects/" -Force }
    Write-Success "Moved: docs/ROADMAP.md -> docs/projects/"
}

# Phase 4: Move Scripts
Write-Info "Phase 4: Moving scripts..."

$scriptMoves = @{
    # Build scripts
    "build.ps1" = "scripts/build/"
    "build.sh" = "scripts/build/"
    "build-crc.ps1" = "scripts/build/"
    
    # Dev scripts
    "quickstart.ps1" = "scripts/dev/"
    "quickstart.sh" = "scripts/dev/"
    
    # Integration scripts
    "integrate-all-agents.ps1" = "scripts/integration/"
    "fix-agent-imports.ps1" = "scripts/integration/"
    "merge-old-work.ps1" = "scripts/integration/"
    "apply-quick-fix.ps1" = "scripts/integration/"
    
    # Maintenance scripts
    "simulate-crc-flow.ps1" = "scripts/maintenance/"
    
    # Tools scripts
    "update-copilot-settings.ps1" = "scripts/tools/"
}

foreach ($script in $scriptMoves.Keys) {
    if (Test-Path $script) {
        $dest = $scriptMoves[$script]
        if (!$WhatIf) {
            Move-Item -Path $script -Destination $dest -Force
        }
        Write-Success "Moved: $script -> $dest"
    } else {
        Write-Warning "Not found: $script"
    }
}

# Phase 5: Update .gitignore
Write-Info "Phase 5: Updating .gitignore..."

$gitignoreAdditions = @"

# Workspace organization
.workspace-backup-*/

# Logs (keep structure, ignore files)
logs/**/*.log
logs/**/*.txt
logs/**/!(.gitkeep|README.md)
"@

if (!$WhatIf) {
    Add-Content -Path ".gitignore" -Value $gitignoreAdditions -Encoding UTF8
}
Write-Success "Updated .gitignore"

# Phase 6: Create .gitkeep files
Write-Info "Phase 6: Creating .gitkeep files for logs..."

$gitkeeps = @(
    "logs/app/server/.gitkeep",
    "logs/app/agents/.gitkeep",
    "logs/app/crc/.gitkeep",
    "logs/build/.gitkeep",
    "logs/test/.gitkeep",
    "logs/system/.gitkeep"
)

foreach ($gitkeep in $gitkeeps) {
    if (!$WhatIf) {
        "" | Set-Content -Path $gitkeep -Encoding UTF8
    }
    Write-Success "Created: $gitkeep"
}

# Summary
Write-Info ""
Write-Info "========================================="
Write-Info "Workspace Organization Complete!"
Write-Info "========================================="
Write-Info ""
Write-Info "Summary:"
Write-Info "  ✅ Directory structure created"
Write-Info "  ✅ README files created"
Write-Info "  ✅ Documentation organized"
Write-Info "  ✅ Scripts organized"
Write-Info "  ✅ .gitignore updated"
Write-Info "  ✅ Log structure created"
Write-Info ""

if ($WhatIf) {
    Write-Warning "DRY RUN COMPLETE - No changes were made"
    Write-Info "Run without -WhatIf to apply changes"
} else {
    Write-Success "Workspace successfully organized!"
    Write-Info ""
    Write-Info "Next steps:"
    Write-Info "  1. Review changes: git status"
    Write-Info "  2. Commit changes: git add . && git commit -m 'refactor: Organize workspace structure'"
    Write-Info "  3. Push to GitHub: git push origin main"
}
