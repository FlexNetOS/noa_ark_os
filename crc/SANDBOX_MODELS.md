# CRC Sandbox Models

## Overview

The CRC (Continuous ReCode) system uses isolated sandbox models for parallel development and safe code integration.

## Sandbox Model Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     INCOMING CODE                           │
│            (External Repos, Forks, Stale Code)             │
└────────────┬─────────────┬─────────────┬──────────────────┘
             │             │             │
             ▼             ▼             ▼
      ┌──────────┐  ┌──────────┐  ┌──────────┐
      │ Model A  │  │ Model B  │  │ Model C  │
      │ Feature  │  │ Bug Fix  │  │Experiment│
      │  Sandbox │  │  Sandbox │  │  Sandbox │
      └────┬─────┘  └────┬─────┘  └────┬─────┘
           │             │             │
           │      Validate Each        │
           │             │             │
           └─────────────┼─────────────┘
                         │
                         ▼
                  ┌──────────────┐
                  │   MERGE      │
                  │   A + B + C  │
                  └──────┬───────┘
                         │
                         ▼
                  ┌──────────────┐
                  │   Model D    │
                  │ Integration  │
                  │   Sandbox    │
                  └──────┬───────┘
                         │
                         ▼
                  ┌──────────────┐
                  │   CI/CD      │
                  │  Pipeline    │
                  └──────────────┘
```

## Sandbox Models

### Model A - Feature Development Sandbox

**Purpose**: Develop new features in isolation

**Characteristics**:
- For new functionality
- Breaking changes allowed
- Can be large refactors
- Independent of other sandboxes

**Use Cases**:
- New API endpoints
- New modules/components
- Architecture changes
- Performance improvements

**Example**:
```bash
# Drop new feature code
crc drop-in --sandbox model-a --type feature

# Code gets adapted and isolated in Model A
# Can develop without affecting B or C
```

---

### Model B - Bug Fix Sandbox

**Purpose**: Fix bugs without interference from new features

**Characteristics**:
- For bug fixes only
- Must maintain compatibility
- Smaller, focused changes
- Independent of features in A

**Use Cases**:
- Security patches
- Bug fixes
- Critical fixes
- Hotfixes

**Example**:
```bash
# Drop bug fix code
crc drop-in --sandbox model-b --type bugfix

# Bug fix isolated from features in A
# Can be merged to production quickly
```

---

### Model C - Experimental Sandbox

**Purpose**: Test experimental ideas without risk

**Characteristics**:
- For experiments and R&D
- May not reach production
- Can break things
- Completely isolated

**Use Cases**:
- New technologies
- Performance experiments
- Proof of concepts
- Research projects

**Example**:
```bash
# Drop experimental code
crc drop-in --sandbox model-c --type experimental

# Experiment safely isolated
# Won't affect A or B
```

---

### Model D - Integration Sandbox

**Purpose**: Integrate validated code from A, B, and C

**Characteristics**:
- Receives merged code from A, B, C
- All conflicts resolved
- Fully validated and tested
- Ready for CI/CD

**Process**:
1. Validate A, B, C independently
2. Check for conflicts
3. Merge A + B + C → D
4. Validate merged result
5. Trigger CI/CD pipeline

**Example**:
```bash
# Automatic merge when A, B, C are ready
crc merge-to-d

# Model D contains integrated code
# Proceeds to CI/CD automatically
```

## Isolation Guarantees

### File System Isolation

Each sandbox has its own directory:
```
crc/sandboxes/
├── model-a/
│   ├── src/
│   ├── tests/
│   └── adapted/
├── model-b/
│   ├── src/
│   ├── tests/
│   └── adapted/
├── model-c/
│   ├── src/
│   ├── tests/
│   └── adapted/
└── model-d/
    ├── merged/
    ├── tests/
    └── validated/
```

### Dependency Isolation

- Each sandbox has independent dependencies
- No shared state between A, B, C
- Model D receives merged dependencies
- Conflicts detected during merge

### Test Isolation

- Each sandbox runs tests independently
- No test interference
- Model D runs full test suite on merged code

## Validation Process

### Per-Sandbox Validation

```yaml
sandbox_validation:
  model_a:
    - lint_check
    - unit_tests
    - integration_tests
    - security_scan
    - performance_tests
  
  model_b:
    - lint_check
    - unit_tests
    - regression_tests  # Ensure bug is fixed
    - security_scan
  
  model_c:
    - lint_check
    - experimental_tests
    - feasibility_check
```

### Merge Validation

```yaml
merge_validation:
  pre_merge:
    - conflict_detection
    - dependency_analysis
    - compatibility_check
  
  post_merge:
    - full_test_suite
    - integration_tests
    - performance_regression
    - security_audit
```

## Merge Process

### Step-by-Step Merge

```rust
// 1. Check readiness
let a_ready = validate_sandbox(ModelA)?;
let b_ready = validate_sandbox(ModelB)?;
let c_ready = validate_sandbox(ModelC)?;

if !a_ready || !b_ready || !c_ready {
    return Err("Not all sandboxes ready");
}

// 2. Detect conflicts
let conflicts = detect_conflicts([ModelA, ModelB, ModelC])?;
if !conflicts.is_empty() {
    return Err("Conflicts detected");
}

// 3. Merge code
let merged = merge_code(ModelA, ModelB, ModelC)?;

// 4. Apply to Model D
apply_to_sandbox(ModelD, merged)?;

// 5. Validate merged result
validate_sandbox(ModelD)?;

// 6. Trigger CI/CD
trigger_cicd(ModelD)?;
```

### Conflict Resolution

**Automatic Resolution**:
- Non-overlapping files: Auto-merge
- Independent changes: Auto-merge
- Compatible dependencies: Auto-merge

**Manual Resolution** (if needed):
- Overlapping file changes
- Incompatible dependencies
- Breaking API changes

## Workflow Examples

### Example 1: Feature Development

```bash
# 1. Drop feature code to Model A
crc drop-in --sandbox model-a \
  --source github.com/external/new-feature

# 2. CRC processes in Model A
# [CRC] Analyzing code...
# [CRC] Adapting to workspace...
# [CRC] Validating in Model A...
# [CRC] Model A ready to merge

# 3. When B and C also ready, automatic merge
# [CRC] Merging A + B + C → D
# [CRC] Model D validated
# [CRC] Triggering CI/CD
```

### Example 2: Hotfix

```bash
# 1. Drop hotfix to Model B
crc drop-in --sandbox model-b \
  --source /hotfixes/security-patch \
  --priority critical

# 2. CRC processes quickly in Model B
# [CRC] Processing in Model B (Bug Fix)
# [CRC] Security patch validated
# [CRC] Model B ready

# 3. Can merge immediately (if A and C empty)
crc merge-to-d --force-b-only

# 4. Fast-track to production
```

### Example 3: Experiment

```bash
# 1. Drop experiment to Model C
crc drop-in --sandbox model-c \
  --source /experiments/new-algorithm \
  --type experimental

# 2. CRC processes in Model C
# [CRC] Processing in Model C (Experimental)
# [CRC] Running experimental tests...

# 3. Experiment fails, don't merge
crc abandon-sandbox model-c

# 4. Model C cleared, no impact on A or B
```

## Monitoring

### Sandbox Status

```bash
# Check all sandboxes
crc sandbox status

# Output:
# Model A (Feature):      2 drops, validated, ready
# Model B (Bug Fix):      1 drop, validated, ready
# Model C (Experimental): 0 drops, empty
# Model D (Integration):  0 drops, ready for merge
```

### Merge Readiness

```bash
# Check if ready to merge
crc merge-check

# Output:
# ✓ Model A: Ready (2 drops, all validated)
# ✓ Model B: Ready (1 drop, validated)
# ⚠ Model C: Empty (skip in merge)
# → Ready to merge A + B → D
```

## Best Practices

### DO:
✅ Use Model A for features
✅ Use Model B for bug fixes
✅ Use Model C for experiments
✅ Validate each sandbox independently
✅ Resolve conflicts before merging
✅ Test Model D thoroughly

### DON'T:
❌ Mix features and bug fixes in same sandbox
❌ Skip validation
❌ Force merge with conflicts
❌ Use Model D for development
❌ Ignore test failures
❌ Rush the merge process

## Configuration

**`crc/config/sandboxes.yaml`**:
```yaml
sandboxes:
  model_a:
    name: "Feature Development"
    type: "feature"
    auto_merge: true
    validation_required: true
    min_test_coverage: 80
  
  model_b:
    name: "Bug Fix"
    type: "bugfix"
    auto_merge: true
    validation_required: true
    fast_track: true  # Can skip Model C
  
  model_c:
    name: "Experimental"
    type: "experimental"
    auto_merge: false  # Manual decision
    validation_required: true
    can_abandon: true
  
  model_d:
    name: "Integration"
    type: "integration"
    sources: ["model_a", "model_b", "model_c"]
    full_validation: true
    trigger_cicd: true
```

## CLI Commands

```bash
# Sandbox management
crc sandbox list                     # List all sandboxes
crc sandbox status <model>           # Get sandbox status
crc sandbox clear <model>            # Clear sandbox
crc sandbox abandon <model>          # Abandon sandbox

# Validation
crc validate <model>                 # Validate sandbox
crc validate-all                     # Validate all sandboxes

# Merging
crc merge-check                      # Check merge readiness
crc merge-to-d                       # Merge A+B+C → D
crc merge-to-d --force               # Force merge (skip checks)
crc merge-to-d --models a,b          # Merge only A and B

# Inspection
crc sandbox diff <model1> <model2>   # Compare sandboxes
crc sandbox conflicts                # Show conflicts
crc sandbox dependencies <model>     # Show dependencies
```

## Future Enhancements

- [ ] More sandbox models (E, F, G)
- [ ] Parallel merges
- [ ] Incremental merges
- [ ] Conflict prediction
- [ ] Auto-conflict resolution
- [ ] Sandbox cloning
- [ ] Sandbox branching
