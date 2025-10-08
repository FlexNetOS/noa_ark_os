# Workflow Templates

Reusable templates for workflow automation in NoaArkOS.

## Overview

This directory contains YAML and Markdown templates that enforce structured, context-driven workflows. Each template includes constitutional policies and success criteria to ensure intentional, auditable operations.

## Purpose

**Why Templates?**
- ✅ **Consistency** - Standardized format across all workflows
- ✅ **Intentionality** - Forces explicit context and purpose
- ✅ **Auditability** - Creates permanent record of decisions
- ✅ **Quality** - Enforces success criteria upfront
- ✅ **Constitutional** - Policies embedded in every operation

## Directory Structure

```
templates/
└── merge/
    ├── README.md                      # This file
    └── agent_merge_request.yaml       # Merge operation template
```

## Available Templates

### Merge Request Template

**File:** `merge/agent_merge_request.yaml`  
**Purpose:** Specify multi-source merge operations with constitutional policies

**Use Cases:**
- Consolidating multiple agent implementations
- Merging configuration fragments
- Aggregating documentation from different sources
- Combining tool collections

**Template Structure:**

```yaml
merge_request:
  id: ""                          # Unique identifier for tracking
  
  context:
    purpose: ""                   # Why is this merge needed?
    success_criteria: ""          # How do we measure success?
    constraints: ""               # What are the limitations?
    scope: ""                     # What's included/excluded?
  
  sources: []                     # Array of source paths to merge
  
  target:
    location: ""                  # Where to output merged result
    format: "dir"                 # Output format (dir, file, archive)
  
  merge_model: "C"                # Which CRC model to use (A, B, C, D)
  
  policies:
    - "Heal, Don't Harm"          # Non-destructive operations
    - "Upgrades Only"             # Only improvements, no regressions
```

**Constitutional Policies:**

1. **"Heal, Don't Harm"**
   - Operations must be non-destructive
   - Preserve existing functionality
   - No data loss
   - Rollback capability required

2. **"Upgrades Only"**
   - Only accept improvements
   - No feature regressions
   - Maintain or improve quality
   - Backward compatibility preferred

**Connection to Trifecta Court:**
- These policies align with the Trifecta Court constitutional governance (agent-registry)
- All workflow operations validate against Scripture, Geometry, Law principles
- Constitutional validation at both service level (agent-registry) and workflow level (task exec kit)

## Usage Guide

### Creating a Merge Request

**Step 1: Copy Template**
```powershell
# Windows
Copy-Item workflow/templates/merge/agent_merge_request.yaml -Destination my-merge-request.yaml

# WSL
cp workflow/templates/merge/agent_merge_request.yaml my-merge-request.yaml
```

**Step 2: Fill in Context**
```yaml
merge_request:
  id: "agent-consolidation-2025-10-08"
  
  context:
    purpose: "Consolidate 3 experimental agent implementations into single production agent"
    success_criteria: |
      - All tests pass
      - Performance >= best individual agent
      - All features preserved
      - Documentation complete
    constraints: |
      - Must complete within 1 hour
      - No external dependencies
      - Preserve Trifecta Court validation
    scope: |
      INCLUDED: agent1/, agent2/, agent3/
      EXCLUDED: Test fixtures, build artifacts
  
  sources:
    - "D:/experiments/agent1"
    - "D:/experiments/agent2"
    - "D:/experiments/agent3"
  
  target:
    location: "D:/dev/workspaces/noa_ark_os/services/consolidated-agent"
    format: "dir"
  
  merge_model: "C"
  
  policies:
    - "Heal, Don't Harm"
    - "Upgrades Only"
```

**Step 3: Execute Workflow**
```bash
# Use merge consolidation flow
./workflow/flows/merge/merge_consolidate.sh \
  D:/experiments/agent1 \
  D:/experiments/agent2 \
  D:/experiments/agent3
```

**Step 4: Verify Output**
```powershell
# Check merged output
Get-ChildItem services/consolidated-agent -Recurse

# Run tests
cd services/consolidated-agent
cargo test
```

### Example: Configuration Merge

```yaml
merge_request:
  id: "config-unification-2025-10-08"
  
  context:
    purpose: "Merge development, staging, and production configs into unified config"
    success_criteria: |
      - All environments configurable via single file
      - No secrets in merged config
      - Validation passes for all environments
    constraints: "Must support environment variable overrides"
    scope: "Configuration files only, no secrets"
  
  sources:
    - "server/config/dev.toml"
    - "server/config/staging.toml"
    - "server/config/prod.toml"
  
  target:
    location: "server/config/unified.toml"
    format: "file"
  
  merge_model: "C"
  
  policies:
    - "Heal, Don't Harm"
    - "Upgrades Only"
```

### Example: Documentation Aggregation

```yaml
merge_request:
  id: "docs-aggregation-2025-10-08"
  
  context:
    purpose: "Aggregate scattered READMEs into comprehensive documentation"
    success_criteria: |
      - All critical information preserved
      - Proper table of contents
      - No duplicate sections
      - Markdown lint passes
    constraints: "Maximum 100 KB total size"
    scope: "READMEs from all subdirectories"
  
  sources:
    - "agents/README.md"
    - "core/README.md"
    - "server/README.md"
    - "services/*/README.md"
  
  target:
    location: "docs/COMPLETE_REFERENCE.md"
    format: "file"
  
  merge_model: "B"
  
  policies:
    - "Heal, Don't Harm"
    - "Upgrades Only"
```

## Model Selection

Templates support different CRC models for merge operations:

| Model | Purpose | Conflict Resolution | Use Case |
|-------|---------|---------------------|----------|
| **A** | Conservative | Preserve all sources | Critical systems, no loss tolerated |
| **B** | Balanced | Merge intelligently | General purpose, most common |
| **C** | Optimizing | Best practices win | Code quality improvements |
| **D** | Aggressive | Latest wins | Fast-moving development |

**Default:** Model C (optimizing)

**Recommendation:**
- Use **Model A** for critical data merges
- Use **Model B** for general documentation/config
- Use **Model C** for code consolidation (default)
- Use **Model D** for rapid prototyping only

## Integration with CRC System

### Drop-In Processing

Templates can be used with the CRC drop-in system:

```
1. External Code → crc/drop-in/incoming/repos/
2. Create merge request → my-merge.yaml
3. CRC Analysis → Pattern matching, rules
4. Template Processing → Execute merge workflow
5. Validation → Verify against policies
6. Ready → crc/drop-in/ready/model-X-queue/
```

### Sandbox Integration

Merge operations use CRC sandbox models:

```
workflow/templates/merge/
    ↓ (specifies model C)
workflow/flows/merge/merge_consolidate.sh
    ↓ (executes with model C)
crc/sandboxes/model-c/
    ↓ (processes merge)
merged output
```

## Template Development

### Creating New Templates

**1. Identify Workflow Pattern**
- What operation is repeated?
- What context is always needed?
- What policies apply?

**2. Define Structure**
```yaml
workflow_name:
  id: ""
  context:
    purpose: ""
    success_criteria: ""
    constraints: ""
    scope: ""
  # workflow-specific fields
  policies:
    - "Constitutional Policy 1"
    - "Constitutional Policy 2"
```

**3. Document Template**
- Create README in template directory
- Provide usage examples
- Document all fields
- Explain policies

**4. Create Corresponding Flow**
- Implement automation in `workflow/flows/`
- Use template validation
- Enforce policies programmatically

### Template Best Practices

**Required Fields:**
- ✅ `id` - Unique identifier for tracking
- ✅ `context.purpose` - Why this operation?
- ✅ `context.success_criteria` - How to measure success?
- ✅ `policies` - Constitutional constraints

**Recommended Fields:**
- ✅ `context.constraints` - Limitations and requirements
- ✅ `context.scope` - What's included/excluded
- ✅ `metadata.created_at` - Timestamp
- ✅ `metadata.created_by` - Author

**Documentation:**
- ✅ Comment complex fields with examples
- ✅ Provide realistic examples
- ✅ Document validation rules
- ✅ Cross-reference related templates

## Cross-References

### Related Systems

- **CRC System** (`crc/`) - Sandbox models for merge processing
- **Workflow Flows** (`workflow/flows/`) - Executable automation scripts
- **Agent Registry** (`services/agent-registry/`) - Trifecta Court governance
- **Drop-In System** (`crc/drop-in/`) - Code intake and processing

### Related Documentation

- `workflow/flows/merge/README.md` - Merge workflow execution
- `workflow/README.md` - Overall workflow system
- `crc/README.md` - CRC system overview
- `TASK_EXEC_KIT_DROP_INTEGRATION_COMPLETE.md` - Integration guide

## Future Templates

**Planned Templates:**

1. **Task Decomposition Template**
   - Break large tasks into subtasks
   - Dependency tracking
   - Parallel execution support

2. **Validation Request Template**
   - Specify validation criteria
   - Define test suites
   - Success/failure thresholds

3. **Deployment Request Template**
   - Environment configuration
   - Rollback procedures
   - Health checks

4. **Rollback Request Template**
   - Specify rollback target
   - Validation after rollback
   - Incident documentation

## Resources

- **CRC Documentation:** `crc/README.md`
- **Workflow System:** `workflow/README.md`
- **Constitutional Governance:** `services/agent-registry/README.md` (Trifecta Court)
- **Integration Guide:** `TASK_EXEC_KIT_DROP_INTEGRATION_COMPLETE.md`

---

**Source:** WSL `/home/deflex/workspace/task_exec_kit/templates/`  
**Integrated:** Cycle 4 (Task Exec Kit Drop)  
**Status:** ✅ Production-Ready Templates  
**Version:** 1.0
