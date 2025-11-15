# Fix Integrated Agent Imports
# Automatically adapts imports from agentaskit to NOA workspace

$AgentImplPath = "D:\dev\workspaces\noa_ark_os\agents\src\implementations"

Write-Host "🔧 Fixing agent imports..." -ForegroundColor Cyan

$filesFixed = 0
$errors = 0

Get-ChildItem -Path $AgentImplPath -Recurse -Filter "*.rs" | Where-Object { $_.Name -ne "mod.rs" } | ForEach-Object {
    $file = $_.FullName
    $fileName = $_.Name
    
    try {
        $content = Get-Content $file -Raw
        $originalContent = $content
        $modified = $false
        
        # Fix: use crate::agents:: -> use noa_agents::
        if ($content -match 'use crate::agents::') {
            $content = $content -replace 'use crate::agents::', 'use crate::'
            $modified = $true
        }
        
        # Add missing imports at top
        if ($content -notmatch 'use serde_json') {
            $content = "use serde_json;\n" + $content
            $modified = $true
        }
        
        # Fix Agent trait imports - comment out for now since they need full implementation
        if ($content -match '@async_trait]') {
            $content = $content -replace '#\[@async_trait\]', '// #[async_trait] - TODO: Implement full Agent trait'
            $modified = $true
        }
        
        if ($content -match 'impl Agent for') {
            $content = $content -replace 'impl Agent for', '// impl Agent for // TODO: Implement full Agent trait'
            $modified = $true
        }
        
        if ($modified) {
            Set-Content $file -Value $content -NoNewline
            Write-Host "  ✓ Fixed: $fileName" -ForegroundColor Green
            $filesFixed++
        } else {
            Write-Host "  ○ Skipped: $fileName (no changes needed)" -ForegroundColor Gray
        }
    }
    catch {
        Write-Host "  ✗ Error: $fileName - $($_.Exception.Message)" -ForegroundColor Red
        $errors++
    }
}

Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host "Fixup Complete!" -ForegroundColor Green
Write-Host "  Files Fixed: $filesFixed"
Write-Host "  Errors: $errors"
Write-Host ""
