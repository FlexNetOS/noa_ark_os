# Consolidation Merger • Complete Workflow System

**Source**: WSL noa_ark_os `consolidation_merger/consolidation_merger_workflow_guide.md`  
**Integration**: Cycle 5 - NOA Ark OS Selective Drop  
**Date**: 2025-01-XX  
**Constitutional Alignment**: "Heal, Don't Harm", "Upgrades Only"

---

## Purpose

Single, comprehensive workflow from user input to deployment with copy/paste templates, automation gates, and constitutional governance.

---

## All-in-One Flow (User Input → Deployment)

```mermaid
flowchart TD
    A[User Input] --> B[4D Analysis]
    B --> C[subject.todo: Inbox]
    C --> D[Under Analysis]
    D --> E[Ready for Promotion]
    E --> F[Create TASK STUB: agentask.todo]
    F --> G[Author]
    G --> H[Index/Sign]
    H --> I[Seal/Policy]
    I --> J[Tri-Run]
    J --> K[Merge(D)]
    K --> L[Verify/Contract]
    L --> M[WASM Build]
    M --> N[Run Core]
    N --> O[Anchor]
    O --> P[Promote]
    P --> Q[Completed: SOT.md]
    style A fill:#f9f,stroke:#333
    style Q fill:#bbf,stroke:#333
    subgraph "Automated Gates"
    H -->|hashes.pre| I
    L -->|verify.pass| M
    O -->|sot.appended| P
    end
```

---

## Quick Checklist

- **Capture**: Add raw request in `agentask.subject.todo` (Inbox)
- **Analyze**: Apply 4D in (Under Analysis) and write 1–3 sentence 4D summary
- **Promote**: Move to (Ready for Promotion) and complete Promotion Checklist
- **Create TASK STUB**: Add to `agentask.todo` (Current Tasks), link SUBJECT_ID → TASK_ID
- **Execute**: Follow phased table with evidence and hashes (PRE/POST)
- **Deploy**: Complete Promote step and record summary

---

## State Machine (Automatable)

### subject.todo States
- **Inbox** → **Under Analysis** (capture acknowledged)
- **Under Analysis** → **Ready for Promotion** (Promotion Checklist passes)
- **Ready for Promotion** → **Archived Subjects** (after TASK_ID mapping recorded)

### agentask.todo States
- **Current Tasks** → **In Progress** → **Verify/Contract** → **Anchor** → **Promote** → **Completed**

**Note**: Transitions enforced by "Automation Gates and Transitions". Supports parallel tasks via unique IDs.

---

## ID Conventions (Regex)

```yaml
id_conventions:
  subject_id: "^SUBJ-\\d{8}-\\d{3}$"
  task_id: "^TASK-\\d{8}-\\d{3}$"
```

**Examples**:
- `SUBJ-20250115-001`
- `TASK-20250115-042`

---

## Key Terms Glossary

- **4D**: Deconstruct (extract intent), Diagnose (audit gaps), Develop (craft solutions), Deliver (execute/output)
- **Tri-Run**: Triple validation run (dry-run, simulate, test plan)
- **Merge(D)**: Defensive merge (perform changes safely, heal without harm)
- **SOP**: Standard Operating Procedure (prescriptive playbook)
- **SOT**: Source of Truth (immutable ledger of completed work)
- **HASHES_PRE**: Baseline file hashes before changes
- **HASHES_POST**: Verification file hashes after changes
- **Truth Gate**: 6/6 verification pass required for production

---

## Automation Gates and Transitions (Single Source)

```yaml
automation_gates:
  require:
    - phases.index_sign.hashes_pre_file          # From manifest
    - phases.verify_contract.tests                # From manifest
    - phases.verify_contract.hashes_post_file     # From manifest
    - phases.wasm.module                          # From manifest
    - phases.wasm.target                          # From manifest
    - phases.wasm.build_cmd                       # From manifest
    - phases.anchor.receipt                       # From manifest
    - phases.anchor.sot_path                      # From manifest
    
  transitions:
    in_progress:
      require:
        - manifest.present                        # manifest file exists for TASK_ID
        - subject_id.linked                       # SUBJECT_ID linked in task metadata
        
    verify_contract:
      require:
        - tests.passed                            # exit code 0
        - hashes.pre.present                      # data/snapshots/${TASK_ID}/HASHES_PRE.txt
        - hashes.post.present                     # data/snapshots/${TASK_ID}/HASHES_POST.txt
        
    anchor:
      require:
        - sot.appended                            # SOT.md updated with sot_entry
        - evidence.paths.recorded                 # evidence links in SOT entry
        
    promote:
      require:
        - deploy.proof                            # deployment record written
        
    completed:
      require:
        - artifacts.linked                        # manifests, hashes, logs linked in SOT
```

---

## Rollback Quickref

| Phase           | Trigger                           | Action                                              | Target State             |
| --------------- | --------------------------------- | --------------------------------------------------- | ------------------------ |
| Verify/Contract | Tests fail or hashes mismatch     | Restore from HASHES_PRE; fix issue                  | Return to Tri-Run        |
| WASM            | Build fails (e.g., cmd error)     | Fix config/target; clear artifacts                  | Re-run WASM              |
| Promote         | Deploy fails (e.g., auth/network) | Roll back to last Anchor; log error                 | Reopen as IN_PROGRESS    |
| Any (general)   | Gate violation                    | Stop; audit logs; partial rollback via SOT baseline | BLOCKED in agentask.todo |

---

## Evidence Paths Convention

Use env var `${EVIDENCE_BASE}` for portability (default: current project root).

```
Hashes/baselines:  ${EVIDENCE_BASE}/data/snapshots/${TASK_ID}/
Manifests/outputs: ${EVIDENCE_BASE}/data/exports/manifests/${TASK_ID}.yaml
Logs:              ${EVIDENCE_BASE}/logs/development/${TASK_ID}/
Anchors:           ${EVIDENCE_BASE}/anchors/anchor-${TASK_ID}.json
SBOM:              ${EVIDENCE_BASE}/sbom/sbom.cdx.json
```

---

## Subject → Task Promotion

### Where to Work
`agentask.subject.todo` sections:
- **(Inbox)**
- **(Under Analysis)**
- **(Ready for Promotion)**
- **(Archived Subjects)**

### Subject Metadata (paste under Under Analysis item)

```
SUBJECT_ID: SUBJ-YYYYMMDD-NNN
Title: <short name>
Intent: <outcome desired>
4D Summary: <1–3 sentences>
Acceptance:
  - Criterion 1
  - Criterion 2
Priority: (A|B|C)
Proposed OWNER: <agent-or-person>
Risks/Dependencies: <notes>
```

### Promotion Checklist (use under Ready for Promotion)

```
- [ ] SUBJECT_ID present and unique
- [ ] Acceptance criteria listed (≥1)
- [ ] Priority set
- [ ] Proposed OWNER set
- [ ] Risks/dependencies noted
- [ ] Ready to create TASK_ID in agentask.todo
```

### Promotion Mapping (fill after promotion)

```
TASK_ID: TASK-YYYYMMDD-NNN
Status: PROMOTED
```

---

## Task Metadata Template

Copy into a new task block in `agentask.todo`:

```
TASK_ID: TASK-YYYYMMDD-NNN
SUBJECT_ID: SUBJ-YYYYMMDD-NNN
Title: <short name>
Intent: <outcome desired>
OWNER: <agent-or-person>
PRIORITY: (A|B|C)
EFFORT: (XS|S|M|L|XL)  # Total est. (e.g., XS=0.5d)
DUE: YYYY-MM-DD
ACCEPTANCE:
  - Criterion 1
  - Criterion 2
STATUS: PLANNED | IN_PROGRESS | VERIFY_CONTRACT | ANCHOR | PROMOTE | COMPLETED | BLOCKED
Links: <evidence, PRs, hashes, manifests>
```

---

## Phased Execution Table

| Phase               | Inputs                   | Actions                                      | Evidence                          | Gate                           | Effort Est. |
| ------------------- | ------------------------ | -------------------------------------------- | --------------------------------- | ------------------------------ | ----------- |
| 0. User Input       | Raw request              | Capture in subject.todo (Inbox)              | Timestamped entry                 | captured=true                  | -           |
| 1. 4D Analysis      | Request context          | Write 1–3 sentence summary                   | 4D Summary text                   | promotion_checklist pre-filled | XS          |
| 2. Promotion        | Subject metadata         | Complete checklist; assign TASK_ID           | Checklist ticks; ID regex match   | ready_to_create_task=true      | XS          |
| 3. Create TASK STUB | SUBJECT_ID, metadata     | Assign OWNER/DUE/EFFORT; create manifest     | Task block + manifest file        | manifest.present=true          | S           |
| 4. Author           | Task metadata            | Set strategy in manifest.phases.author.notes | Non-empty notes                   | notes.present=true             | S           |
| 5. Index/Sign       | Inputs listed            | Write HASHES_PRE.txt; update manifest        | HASHES_PRE.txt; SBOM              | hashes.pre.present=true        | M           |
| 6. Seal/Policy      | Policies                 | Record constraints in manifest               | Committed constraints array       | constraints.locked=true        | XS          |
| 7. Tri-Run          | Plan, env                | Simulate/dry-run; record plan                | tri_run.log (opt.)                | tri_run.plan.present=true      | M           |
| 8. Merge(D)         | Approved plan            | Perform safe changes; list actions           | Commit/ops log                    | merge.actions.recorded=true    | L           |
| 9. Verify/Contract  | Tests, acceptance        | Run tests; write HASHES_POST.txt             | tests.passed; HASHES_POST.txt     | verify.pass=true               | M           |
| 10. WASM (req.)     | Module path, target, cmd | Build/package                                | Artifact + build logs             | wasm.built=true                | L           |
| 11. Run Core        | Command                  | Execute workload                             | run_core.log                      | run_core.done=true             | XL          |
| 12. Anchor          | SOT path                 | Append sot_entry; write receipt              | SOT entry; anchor-${TASK_ID}.json | sot.appended=true              | S           |
| 13. Promote         | Destination              | Deploy/publish                               | deploy.proof (URL/path/tag)       | deploy.proof=true              | M           |

---

## SOP and SOT: Roles in the Flow

### SOP (Standard Operating Procedure)
- **Purpose**: Prescriptive "how we work" playbook
- **When Used**: Author (plan), Seal/Policy (lock), Verify/Contract (gates source)
- **Ownership**: Canonical, versioned; updated deliberately
- **Location**: `agentask.sop`

### SOT (Source of Truth Ledger)
- **Purpose**: Historical ledger of completed work and references
- **Contents**: Completed tasks, 4D summaries, manifests, hashes, logs, PRs, artifacts, references
- **When Updated**: Verify/Contract (proof snapshot), Anchor (audit entry), Promote (final record)
- **Reset Policy**: Keep TODO files lean; regularly transfer completed items to SOT
- **Location**: `SOT.md`

---

## SOT Entry Template

Append one per completed TASK_ID:

```yaml
sot_entry:
  task_id: "TASK-YYYYMMDD-NNN"
  subject_id: "SUBJ-YYYYMMDD-NNN"
  title: "Short title"
  intent: "Outcome"
  owner: "agent-or-person"
  completed_at: "YYYY-MM-DDThh:mm:ssZ"
  flow:
    phases:
      - "Author"
      - "Index/Sign"
      - "Seal/Policy"
      - "Tri-Run"
      - "Merge(D)"
      - "Verify/Contract"
      - "Run Core"
      - "WASM"
      - "Anchor"
      - "Promote"
  acceptance:
    - "Criterion 1"
  evidence:
    manifests:
      - "data/exports/manifests/TASK-...yaml"
    hashes:
      - "data/snapshots/TASK-.../HASHES_PRE.txt"
      - "data/snapshots/TASK-.../HASHES_POST.txt"
    logs:
      - "logs/development/TASK-.../verify.log"
    links:
      - "https://repo/pull/123"
  notes: "Key decisions and 4D summary."
  signoff:
    owner: "name"
    approver: "name"
    timestamp: "YYYY-MM-DDThh:mm:ssZ"
```

---

## Manifest Template (YAML)

```yaml
manifest:
  task_id: "TASK-YYYYMMDD-NNN"
  subject_id: "SUBJ-YYYYMMDD-NNN"
  title: "Short title"
  intent: "Outcome statement"
  owner: "agent-or-person"
  priority: "A|B|C"
  acceptance:
    - "Criterion 1"
    - "Criterion 2"
  inputs:
    - path: "path/to/input"
      sha256: "..."
  outputs:
    - path: "path/to/output"
      expected: "desc"
  phases:
    author:
      notes: ""
      effort_est: "S"
    index_sign:
      hashes_pre_file: "data/snapshots/${TASK_ID}/HASHES_PRE.txt"
      sbom_file: "sbom/sbom.cdx.json"
    seal_policy:
      constraints: []
    tri_run:
      plan: ""
    merge_d:
      actions: []
    verify_contract:
      tests: []
      evidence: []
      hashes_post_file: "data/snapshots/${TASK_ID}/HASHES_POST.txt"
    wasm:
      module: "path/to/module"              # REQUIRED
      target: "wasm32-wasi"                 # REQUIRED
      build_cmd: "make wasm"                # REQUIRED
    run_core:
      command: ""
    anchor:
      receipt: "anchors/anchor-${TASK_ID}.json"
      sot_path: "SOT.md"
    promote:
      destination: ""
```

---

## Minimal Per-Phase Contract (Automation)

```yaml
per_phase_contract:
  author:
    require: [manifest.phases.author.notes]
  index_sign:
    require: [manifest.phases.index_sign.hashes_pre_file]
  seal_policy:
    require: [manifest.phases.seal_policy.constraints]
  tri_run:
    require: [manifest.phases.tri_run.plan]
  merge_d:
    require: [manifest.phases.merge_d.actions]
  verify_contract:
    require: [
      manifest.phases.verify_contract.tests,
      manifest.phases.verify_contract.hashes_post_file
    ]
  wasm:
    require: [
      manifest.phases.wasm.module,
      manifest.phases.wasm.target,
      manifest.phases.wasm.build_cmd
    ]
  run_core:
    require: [manifest.phases.run_core.command]
  anchor:
    require: [
      manifest.phases.anchor.receipt,
      manifest.phases.anchor.sot_path
    ]
  promote:
    require: [manifest.phases.promote.destination]
```

---

## Constitutional Principles

### Core Governance
1. **"Heal, Don't Harm"** - Non-destructive operations only
2. **"Upgrades Only"** - No downgrades, no version regressions
3. **Truth Gate** - 6/6 verification pass required
4. **Defensive Merging** - APPROVE=yes required for Merge(D)
5. **Cryptographic Verification** - Minisign + fs-verity
6. **Merkle Anchoring** - Immutable release receipts

### Workflow Enforcement
- **Pre-flight Checks**: Dry-run before file operations
- **HASHES_PRE/POST**: Before-after cryptographic verification
- **Policy Constraints**: Locked before execution begins
- **SOT Ledger**: Immutable record of completed work
- **SOP Canonical**: Versioned procedure governance

---

## Integration with NoaArkOS

### Alignment with Existing Systems

#### CRC Drop-in System
- **Subject Inbox**: `crc/drop-in/incoming/` → `agentask.subject.todo` (Inbox)
- **4D Analysis**: CRC analysis phase → (Under Analysis) with 4D summary
- **Task Creation**: Promoted subjects → `agentask.todo` with TASK_ID
- **Evidence Paths**: CRC `data/` → `${EVIDENCE_BASE}/data/`

#### Workflow Templates (Cycle 4)
- **Merge Workflow**: `merge_consolidate.sh` aligns with Merge(D) phase
- **Agent Templates**: `agent_merge_request.yaml` → Task metadata template
- **Constitutional Policies**: Already established in Cycle 4

#### CI/CD System
- **Automation Gates**: CI/CD → Verify/Contract phase gates
- **Artifact Storage**: `cicd/artifacts/` → Evidence paths
- **Metadata Generation**: CI metadata → Manifest system

---

## Automation Implementation

### Make Targets (from Makefile)

```makefile
.PHONY: phase1 phase2 phase3 phase4 phase5-preflight phase5-run phase6 phase7 phase8 phase9

MERGE_ID ?= demo_merge
TARGET   ?= workspace/consolidated
SOURCES  ?=
OUT_DIR  := .merge/$(MERGE_ID)

phase1: ## Phase 1 - AUTHOR (Define strategy)
phase2: ## Phase 2 - INDEX/SIGN (Create HASHES_PRE_MERGE.txt)
phase3: ## Phase 3 - SEAL/POLICY (Create POLICY_CHECKLIST.yaml)
phase4: ## Phase 4 - TRI-RUN (Simulate A/B/C layouts)
phase5-preflight: ## Phase 5 - PRE-FLIGHT (Show proposed output)
phase5-run: ## Phase 5 - MERGE(D) (Execute if APPROVE=yes)
phase6: ## Phase 6 - VERIFY/CONTRACT (Hashes post-merge)
phase7: ## Phase 7 - ANCHOR (Merkle root)
phase8: ## Phase 8 - PROMOTE (Production move)
phase9: ## Phase 9 - ARCHIVE/CLEANUP (Archive sources)

all-phases: phase1 phase2 phase3 phase4 phase5-preflight phase5-run phase6 phase7 phase8 phase9
```

**Key Features**:
- Non-destructive by default
- Phase 5 requires `APPROVE=yes` to execute
- Outputs stored under `.merge/$(MERGE_ID)`
- Priorities follow SOURCES order (LATEST_WINS)

---

## Quick Start

### 1. Capture User Request
```bash
# Add to agentask.subject.todo under (Inbox)
SUBJECT_ID: SUBJ-20250115-001
Title: Integrate new auth system
Intent: Add OAuth2 authentication
Priority: A
```

### 2. Apply 4D Analysis
```bash
# Move to (Under Analysis), add:
4D Summary:
- Deconstruct: OAuth2 with JWT tokens
- Diagnose: Current system lacks SSO
- Develop: Use rust-oauth2 crate
- Deliver: Integration tests + documentation
```

### 3. Promote to Task
```bash
# Move to (Ready for Promotion), complete checklist
# Create in agentask.todo:
TASK_ID: TASK-20250115-042
SUBJECT_ID: SUBJ-20250115-001
Title: OAuth2 Integration
OWNER: auth-team
PRIORITY: A
EFFORT: L
DUE: 2025-01-30
STATUS: PLANNED
```

### 4. Execute Phases
```bash
# Phase 1-4: Planning
make phase1 MERGE_ID=oauth2 PURPOSE="Add OAuth2 auth"
make phase2  # Generate HASHES_PRE
make phase3  # Create policy checklist
make phase4  # Tri-run simulation

# Phase 5: Merge (requires approval)
make phase5-preflight  # Review proposed changes
make phase5-run APPROVE=yes  # Execute merge

# Phase 6-9: Verification & Deployment
make phase6  # Generate HASHES_POST
make phase7  # Create Merkle anchor
make phase8  # Promote to production
make phase9  # Archive sources
```

### 5. Record in SOT
```yaml
# Append to SOT.md:
sot_entry:
  task_id: "TASK-20250115-042"
  subject_id: "SUBJ-20250115-001"
  title: "OAuth2 Integration"
  completed_at: "2025-01-30T15:30:00Z"
  evidence:
    manifests: ["data/exports/manifests/TASK-20250115-042.yaml"]
    hashes: ["data/snapshots/TASK-20250115-042/HASHES_POST.txt"]
```

---

## References

### Related NoaArkOS Documentation
- Workflow Templates: `workflow/templates/`
- Merge Scripts: `workflow/flows/merge_consolidate.sh`
- CRC System: `crc/README.md`
- CI/CD: `cicd/README.md`

### Original Source
- WSL noa_ark_os: `consolidation_merger/consolidation_merger_workflow_guide.md`
- Cycle 5 Integration: `NOA_ARK_OS_SELECTIVE_DROP_MANIFEST.md`

### External References
- **4D Methodology**: Deconstruct, Diagnose, Develop, Deliver
- **Merkle Trees**: Cryptographic verification
- **WASM**: WebAssembly for capability isolation
- **fs-verity**: Linux kernel file verification

---

*Document v1.0 - Cycle 5 Integration - Complete Workflow System*
