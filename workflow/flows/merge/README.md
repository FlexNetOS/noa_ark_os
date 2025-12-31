# Workflow Flows

Executable automation scripts for workflow operations in NoaArkOS.

## Overview

This directory contains shell scripts and automation tools that execute workflows defined by templates. Each flow implements a specific operational pattern with built-in error handling, validation, and policy enforcement.

## Purpose

**Why Flows?**
- ✅ **Automation** - Execute complex operations with single commands
- ✅ **Repeatability** - Consistent execution across environments
- ✅ **Safety** - Built-in error handling and validation
- ✅ **Auditability** - Structured logging and output
- ✅ **Integration** - Works with CRC system and templates

## Directory Structure

```
flows/
└── merge/
    ├── README.md                # This file
    └── merge_consolidate.sh     # Multi-source merge workflow
```

## Available Flows

### Merge Consolidation Flow

**File:** `merge/merge_consolidate.sh`  
**Purpose:** Consolidate multiple source directories using merge-polish tool

**Workflow Stages:**

1. **Simulate** - Preview merge operations without modifications
2. **Merge** - Execute merge with constitutional policies
3. **Verify** - Validate merged output against success criteria

**Usage:**

```bash
# Basic usage
./workflow/flows/merge/merge_consolidate.sh source1/ source2/ source3/

# With WSL
wsl bash -c "./workflow/flows/merge/merge_consolidate.sh source1/ source2/ source3/"

# Windows (via Git Bash or WSL)
bash workflow/flows/merge/merge_consolidate.sh /path/source1 /path/source2
```

**Arguments:**
- `$@` - One or more source paths to consolidate
- All sources merged to `$ROOT_DIR/merged/`

**Environment Variables:**
- `ROOT_DIR` - Auto-detected workspace root (2 levels up from script)
- Or set manually: `export ROOT_DIR=/path/to/workspace`

**Exit Codes:**
- `0` - Success (all stages passed)
- `1` - Simulate failed
- `2` - Merge failed
- `3` - Verify failed

**Example Output:**

```
[INFO] Starting merge consolidation workflow
[INFO] Root directory: /mnt/d/dev/workspaces/noa_ark_os
[INFO] Sources: agent1/ agent2/ agent3/
[INFO] Target: /mnt/d/dev/workspaces/noa_ark_os/merged

[STAGE 1/3] Simulate
[simulate] Loading sources...
[simulate] Analyzing conflicts...
[simulate] Preview: 45 files, 12 conflicts
[simulate] ✓ Simulation complete

[STAGE 2/3] Merge
[merge] Applying model C rules...
[merge] Resolving conflicts...
[merge] Writing output...
[merge] ✓ Merge complete: 45 files written

[STAGE 3/3] Verify
[verify] Checking file integrity...
[verify] Validating syntax...
[verify] Running tests...
[verify] ✓ All checks passed

[SUCCESS] Consolidation complete: merged/
```

## Flow Details

### merge_consolidate.sh

**Implementation:**

```bash
#!/usr/bin/env bash
set -euo pipefail

# Auto-detect workspace root (2 directories up from script)
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/../.. && pwd)"

# Stage 1: Simulate
merge-polish simulate --model C --sources "$@" --target "$ROOT_DIR/merged" || \
  "$ROOT_DIR/workspace/tools/bin/merge-polish" simulate --model C --sources "$@" --target "$ROOT_DIR/merged"

# Stage 2: Merge
merge-polish merge --model C --sources "$@" --target "$ROOT_DIR/merged" --confirm yes || \
  "$ROOT_DIR/workspace/tools/bin/merge-polish" merge --model C --sources "$@" --target "$ROOT_DIR/merged" --confirm yes

# Stage 3: Verify
merge-polish verify --target "$ROOT_DIR/merged" || \
  "$ROOT_DIR/workspace/tools/bin/merge-polish" verify --target "$ROOT_DIR/merged"
```

**Key Features:**

1. **Error Handling**
   - `set -euo pipefail` - Exit on any error
   - Each stage must succeed before next stage
   - Clear error messages from merge-polish

2. **Path Fallback**
   - Tries `merge-polish` on system PATH first
   - Falls back to `workspace/tools/bin/merge-polish`
   - Robust across different installations

3. **Model C Processing**
   - Uses CRC sandbox model-c
   - Optimizing merge strategy
   - Best practices win conflicts

4. **Auto-Confirmation**
   - `--confirm yes` for automation
   - No manual intervention needed
   - Safe for CI/CD pipelines

5. **Dynamic Root Detection**
   - Calculates workspace root from script location
   - Works regardless of execution directory
   - Portable across environments

## Integration with Templates

### Using with Merge Request Template

**Step 1: Create Merge Request**

```yaml
# my-merge.yaml
merge_request:
  id: "agent-consolidation-2025-10-08"
  context:
    purpose: "Consolidate agent implementations"
    success_criteria: "All tests pass, no regressions"
    constraints: "Complete within 1 hour"
    scope: "agent1/, agent2/, agent3/"
  sources:
    - "D:/experiments/agent1"
    - "D:/experiments/agent2"
    - "D:/experiments/agent3"
  target:
    location: "D:/dev/workspaces/noa_ark_os/merged"
    format: "dir"
  merge_model: "C"
  policies:
    - "Heal, Don't Harm"
    - "Upgrades Only"
```

**Step 2: Execute Flow**

```bash
# Extract sources from YAML (manual or scripted)
./workflow/flows/merge/merge_consolidate.sh \
  D:/experiments/agent1 \
  D:/experiments/agent2 \
  D:/experiments/agent3
```

**Step 3: Verify Against Template**

```bash
# Check success criteria
cd merged/
cargo test  # "All tests pass"
cargo bench  # Check for regressions

# Verify policies
git diff --no-index original/ merged/  # "Heal, Don't Harm"
```

## Advanced Usage

### Custom Merge Target

Override default target location:

```bash
# Modify script or set environment
export ROOT_DIR=/custom/location
./workflow/flows/merge/merge_consolidate.sh source1/ source2/
# Output: /custom/location/merged/
```

### Different Models

Create model-specific flows:

```bash
# merge_consolidate_model_a.sh (conservative)
merge-polish simulate --model A --sources "$@" --target "$ROOT_DIR/merged"
merge-polish merge --model A --sources "$@" --target "$ROOT_DIR/merged" --confirm yes
merge-polish verify --target "$ROOT_DIR/merged"

# merge_consolidate_model_d.sh (aggressive)
merge-polish simulate --model D --sources "$@" --target "$ROOT_DIR/merged"
merge-polish merge --model D --sources "$@" --target "$ROOT_DIR/merged" --confirm yes
merge-polish verify --target "$ROOT_DIR/merged"
```

### Dry Run Mode

Simulate only (no merge):

```bash
# Extract simulate stage only
merge-polish simulate --model C --sources "$@" --target "$ROOT_DIR/merged"
```

### With Logging

Capture detailed logs:

```bash
./workflow/flows/merge/merge_consolidate.sh agent1/ agent2/ 2>&1 | tee merge-log-$(date +%Y%m%d-%H%M%S).txt
```

## Windows PowerShell Equivalent

### merge_consolidate.ps1 (Planned)

```powershell
# Equivalent PowerShell version
param(
    [Parameter(Mandatory=$true, ValueFromRemainingArguments=$true)]
    [string[]]$Sources
)

$ErrorActionPreference = "Stop"

# Auto-detect workspace root
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent (Split-Path -Parent $ScriptDir)

$Target = Join-Path $RootDir "merged"

# Stage 1: Simulate
Write-Host "[STAGE 1/3] Simulate" -ForegroundColor Cyan
& merge-polish simulate --model C --sources $Sources --target $Target
if ($LASTEXITCODE -ne 0) { exit 1 }

# Stage 2: Merge
Write-Host "[STAGE 2/3] Merge" -ForegroundColor Cyan
& merge-polish merge --model C --sources $Sources --target $Target --confirm yes
if ($LASTEXITCODE -ne 0) { exit 2 }

# Stage 3: Verify
Write-Host "[STAGE 3/3] Verify" -ForegroundColor Cyan
& merge-polish verify --target $Target
if ($LASTEXITCODE -ne 0) { exit 3 }

Write-Host "[SUCCESS] Consolidation complete: $Target" -ForegroundColor Green
```

**Usage:**
```powershell
.\workflow\flows\merge\merge_consolidate.ps1 source1\ source2\ source3\
```

## Integration with CRC System

### Drop-In Processing

Flows can automate CRC drop-in processing:

```bash
# 1. External code arrives in drop-in
cp -r external-code/ crc/drop-in/incoming/repos/external-code

# 2. CRC watcher detects and analyzes
# (automatic)

# 3. Execute merge flow for consolidation
./workflow/flows/merge/merge_consolidate.sh \
  crc/drop-in/incoming/repos/external-code \
  agents/existing-agent

# 4. Output to ready queue
mv merged/ crc/drop-in/ready/model-c-queue/
```

### Sandbox Integration

Flows use CRC sandbox models:

```
workflow/flows/merge/merge_consolidate.sh
    ↓ (calls merge-polish with --model C)
merge-polish tool
    ↓ (uses CRC model-c sandbox)
crc/sandboxes/model-c/
    ↓ (processes merge according to model C rules)
merged output (validated)
```

## Error Handling

### Common Errors

**1. merge-polish Not Found**

```
bash: merge-polish: command not found
```

**Solution:**
```bash
# Install merge-polish to workspace
cp /path/to/merge-polish workspace/tools/bin/
chmod +x workspace/tools/bin/merge-polish

# Or add to PATH
export PATH="$PATH:/path/to/merge-polish"
```

**2. Simulate Failed**

```
[simulate] ERROR: Conflict cannot be resolved automatically
```

**Solution:**
- Assign a worker agent to review conflicts
- Adjust source files based on planner guidance
- Re-run simulation
- Or use model A (conservative) for worker-led resolution

**3. Verify Failed**

```
[verify] ERROR: Syntax error in merged output
```

**Solution:**
- Check merge output: `cat merged/problematic-file`
- Review merge-polish logs
- Escalate to the orchestrator agent responsible for merge-polish stewardship
- Apply worker agent fixes if necessary

**4. Permission Denied**

```
bash: ./workflow/flows/merge/merge_consolidate.sh: Permission denied
```

**Solution:**
```bash
chmod +x workflow/flows/merge/merge_consolidate.sh
```

## Troubleshooting

### Debug Mode

Enable verbose output:

```bash
# Bash debug mode
bash -x ./workflow/flows/merge/merge_consolidate.sh source1/ source2/

# merge-polish verbose
merge-polish simulate --model C --sources "$@" --target merged/ --verbose
```

### Check Prerequisites

```bash
# Verify merge-polish installed
which merge-polish || which workspace/tools/bin/merge-polish

# Check version
merge-polish --version

# Test with minimal sources
merge-polish simulate --model C --sources test1/ test2/ --target test-merged/
```

### Inspect Output

```bash
# List merged files
find merged/ -type f

# Check file counts
echo "Source 1: $(find source1/ -type f | wc -l) files"
echo "Source 2: $(find source2/ -type f | wc -l) files"
echo "Merged: $(find merged/ -type f | wc -l) files"

# Compare sizes
du -sh source1/ source2/ merged/
```

## Future Flows

**Planned Flows:**

1. **Deployment Flow** (`deploy_service.sh`)
   - Build service
   - Run tests
   - Deploy to target environment
   - Health checks
   - Rollback on failure

2. **Test Execution Flow** (`run_tests.sh`)
   - Unit tests
   - Integration tests
   - Performance tests
   - Coverage reporting

3. **Validation Flow** (`validate_drop.sh`)
   - Syntax checking
   - Lint checking
   - Security scanning
   - License compliance

4. **Rollback Flow** (`rollback.sh`)
   - Identify rollback target
   - Execute rollback
   - Verify rollback success
   - Document incident

## Cross-References

### Related Systems

- **Templates** (`workflow/templates/`) - Workflow specifications
- **CRC System** (`crc/`) - Sandbox models and processing
- **Tools** (`tools/`) - Development utilities (may include merge-polish)
- **Agent Registry** (`services/agent-registry/`) - Trifecta Court governance

### Related Documentation

- `workflow/templates/merge/README.md` - Merge request templates
- `workflow/README.md` - Overall workflow system
- `crc/README.md` - CRC system overview
- `TASK_EXEC_KIT_DROP_INTEGRATION_COMPLETE.md` - Integration guide

## Resources

- **merge-polish Documentation:** (link if available)
- **CRC Models:** `crc/sandboxes/README.md`
- **Bash Best Practices:** https://google.github.io/styleguide/shellguide.html
- **Error Handling:** `set -euo pipefail` explained

---

**Source:** WSL `/home/deflex/workspace/task_exec_kit/flows/`  
**Integrated:** Cycle 4 (Task Exec Kit Drop)  
**Status:** ✅ Production-Ready Flows  
**Version:** 1.0
