# Consolidation Merger • Workflow Guide

## Table of Contents

- [Purpose](#purpose)
- [All-in-One Flow](#all-in-one-flow-user-input--deployment)
- [Quick Checklist](#quick-checklist)
- [State Machine](#state-machine-automatable)
- [ID Conventions](#id-conventions-regex)
- [Key Terms Glossary](#key-terms-glossary)
- [Automation Gates and Transitions](#automation-gates-and-transitions-single-source)
- [Rollback Quickref](#rollback-quickref)
- [Evidence Paths Convention](#evidence-paths-convention)
- [Subject → Task Promotion](#subject--task-promotion)
- [Task Metadata Template](#task-metadata-template-copy-into-a-new-task-block-in-agentasktodo)
- [Phased Execution Table](#phased-execution-table)
- [SOP and SOT: Roles in the Flow](#sop-and-sot-roles-in-the-flow)
- [SOT Entry Template](#sot-entry-template-append-one-per-completed-task_id)
- [Manifest Template](#manifest-template-yaml)
- [Minimal Per-Phase Contract](#minimal-per-phase-contract-automation)
- [Automation Implementation](#automation-implementation)
- [Pointers](#pointers)

## Purpose

- Single, drillable workflow from user input to deployment with copy/paste templates.

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

## Quick Checklist

- Capture: add raw request in agentask.subject.todo (Inbox)
- Analyze: apply 4D in (Under Analysis) and write a 1–3 sentence 4D summary
- Promote: move to (Ready for Promotion) and complete Promotion Checklist
- Create TASK STUB: add to agentask.todo (Current Tasks), link SUBJECT_ID → TASK_ID
- Execute: follow phased table with evidence and hashes (PRE/POST)
- Deploy: complete Promote step and record summary

## State Machine (Automatable)

- subject.todo
  - Inbox → Under Analysis (capture acknowledged)
  - Under Analysis → Ready for Promotion (Promotion Checklist passes)
  - Ready for Promotion → Archived Subjects (after TASK_ID mapping recorded)
- agentask.todo
  - Current Tasks → In Progress → Verify/Contract → Anchor → Promote → Completed

Note: Transitions are enforced by the “Automation Gates and Transitions” section. Supports parallel tasks via unique IDs; use separate manifests for multi-owner scenarios.

## ID Conventions (Regex)

```yaml
id_conventions:
  subject_id: "^SUBJ-\\d{8}-\\d{3}$"
  task_id: "^TASK-\\d{8}-\\d{3}$"
```

## Key Terms Glossary

- **4D**: Deconstruct (extract intent), Diagnose (audit gaps), Develop (craft solutions), Deliver (execute/output).
- **Tri-Run**: Triple validation run (dry-run, simulate, test plan).
- **Merge(D)**: Defensive merge (perform changes safely, heal without harm).
- **SOP**: Standard Operating Procedure (prescriptive playbook).
- **SOT**: Source of Truth (immutable ledger of completed work).

## Automation Gates and Transitions (Single Source)

```yaml
automation_gates:
  require:
    - phases.index_sign.hashes_pre_file # From manifest.phases.index_sign.hashes_pre_file
    - phases.verify_contract.tests # From manifest.phases.verify_contract.tests
    - phases.verify_contract.hashes_post_file # From manifest.phases.verify_contract.hashes_post_file
    - phases.wasm.module # From manifest.phases.wasm.module
    - phases.wasm.target # From manifest.phases.wasm.target
    - phases.wasm.build_cmd # From manifest.phases.wasm.build_cmd
    - phases.anchor.receipt # From manifest.phases.anchor.receipt
    - phases.anchor.sot_path # From manifest.phases.anchor.sot_path
  transitions:
    in_progress:
      require:
        - manifest.present # manifest file exists for TASK_ID at data/exports/manifests/${TASK_ID}.yaml
        - subject_id.linked # SUBJECT_ID is linked in task metadata
    verify_contract:
      require:
        - tests.passed # test suite executed with exit code 0 (from Verify/Contract evidence)
        - hashes.pre.present # HASHES_PRE.txt exists at data/snapshots/${TASK_ID}/
        - hashes.post.present # HASHES_POST.txt exists at data/snapshots/${TASK_ID}/
    anchor:
      require:
        - sot.appended # SOT.md updated with sot_entry for TASK_ID
        - evidence.paths.recorded # evidence links recorded in SOT entry
    promote:
      require:
        - deploy.proof # deployment record or destination written (e.g., URL/path in manifest)
    completed:
      require:
        - artifacts.linked # manifests, hashes, logs linked in SOT
```

## Rollback Quickref

| Phase           | Trigger                           | Action                                              | Target State             |
| --------------- | --------------------------------- | --------------------------------------------------- | ------------------------ |
| Verify/Contract | Tests fail or hashes mismatch     | Restore from HASHES_PRE; fix issue                  | Return to Tri-Run        |
| WASM            | Build fails (e.g., cmd error)     | Fix config/target; clear artifacts                  | Re-run WASM              |
| Promote         | Deploy fails (e.g., auth/network) | Roll back to last Anchor; log error                 | Reopen as IN_PROGRESS    |
| Any (general)   | Gate violation                    | Stop; audit logs; partial rollback via SOT baseline | BLOCKED in agentask.todo |

## Evidence Paths Convention

Use env var ${EVIDENCE_BASE} for portability (default: current project root).

- Hashes/baselines: ${EVIDENCE_BASE}/data/snapshots/${TASK_ID}/
- Manifests/outputs: ${EVIDENCE_BASE}/data/exports/manifests/${TASK_ID}.yaml
- Logs: ${EVIDENCE_BASE}/logs/development/${TASK_ID}/
- Anchors: ${EVIDENCE_BASE}/anchors/anchor-${TASK_ID}.json
- SBOM: ${EVIDENCE_BASE}/sbom/sbom.cdx.json

## Subject → Task Promotion

### Where to work

- agentask.subject.todo sections: (Inbox), (Under Analysis), (Ready for Promotion), (Archived Subjects)

### Subject metadata (paste under Under Analysis item)

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

### Promotion checklist (use under Ready for Promotion)

```
- [ ] SUBJECT_ID present and unique
- [ ] Acceptance criteria listed (≥1)
- [ ] Priority set
- [ ] Proposed OWNER set
- [ ] Risks/dependencies noted
- [ ] Ready to create TASK_ID in agentask.todo
```

### Promotion mapping (fill after promotion)

```
TASK_ID: TASK-YYYYMMDD-NNN
Status: PROMOTED
```

## Task Metadata Template (copy into a new task block in agentask.todo)

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

## Phased Execution Table

| Phase               | Inputs                   | Actions                                      | Evidence                          | Gate                           | Effort Est. (opt.) |
| ------------------- | ------------------------ | -------------------------------------------- | --------------------------------- | ------------------------------ | ------------------ |
| 0. User Input       | Raw request              | Capture in subject.todo (Inbox)              | Timestamped entry                 | captured=true                  | -                  |
| 1. 4D Analysis      | Request context          | Write 1–3 sentence summary                   | 4D Summary text                   | promotion_checklist pre-filled | XS                 |
| 2. Promotion        | Subject metadata         | Complete checklist; assign TASK_ID           | Checklist ticks; ID regex match   | ready_to_create_task=true      | XS                 |
| 3. Create TASK STUB | SUBJECT_ID, metadata     | Assign OWNER/DUE/EFFORT; create manifest     | Task block + manifest file        | manifest.present=true          | S                  |
| 4. Author           | Task metadata            | Set strategy in manifest.phases.author.notes | Non-empty notes                   | notes.present=true             | S                  |
| 5. Index/Sign       | Inputs listed            | Write HASHES_PRE.txt; update manifest        | ${TASK_ID}/HASHES_PRE.txt; SBOM   | hashes.pre.present=true        | M                  |
| 6. Seal/Policy      | Policies                 | Record constraints in manifest               | Committed constraints array       | constraints.locked=true        | XS                 |
| 7. Tri-Run          | Plan, env                | Simulate/dry-run; record plan                | tri_run.log (opt.)                | tri_run.plan.present=true      | M                  |
| 8. Merge(D)         | Approved plan            | Perform safe changes; list actions           | Commit/ops log                    | merge.actions.recorded=true    | L                  |
| 9. Verify/Contract  | Tests, acceptance        | Run tests; write HASHES_POST.txt             | tests.passed; HASHES_POST.txt     | verify.pass=true               | M                  |
| 10. WASM (req.)     | Module path, target, cmd | Build/package                                | Artifact + build logs             | wasm.built=true                | L                  |
| 11. Run Core        | Command                  | Execute workload                             | run_core.log                      | run_core.done=true             | XL                 |
| 12. Anchor          | SOT path                 | Append sot_entry; write receipt              | SOT entry; anchor-${TASK_ID}.json | sot.appended=true              | S                  |
| 13. Promote         | Destination              | Deploy/publish                               | deploy.proof (URL/path/tag)       | deploy.proof=true              | M                  |

## SOP and SOT: Roles in the Flow

- SOP (Standard Operating Procedure)

  - Purpose: Prescriptive “how we work” playbook; phases, gates, checklists, acceptance.
  - When used: Author (plan), Seal/Policy (lock), Verify/Contract (gates source).
  - Ownership: Canonical, versioned; updated deliberately and infrequently.
  - Location: /home/deflex/workspace/repos/task_exec_kit/agentask.sop

- SOT (Source of Truth ledger)
  - Purpose: Historical ledger of completed work and references; single lookup for what/when/who/evidence.
  - Contents: Completed tasks, 4D summaries, manifests, hashes, logs, PR links, artifacts, reference docs.
  - When updated: At Verify/Contract (proof snapshot), Anchor (audit entry), Promote (final record). Also when TODOs are reset—move DONE items + references into SOT.
  - Reset policy: Keep subject.todo and agentask.todo lean; regularly transfer completed items to SOT.
  - Location: /home/deflex/projects/work/consolidation_merger/SOT.md

### SOT entry template (append one per completed TASK_ID)

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
      effort_est: "S" # Optional: XS|S|M|L|XL per phase
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
      module: "path/to/module" # REQUIRED
      target: "wasm32-wasi" # REQUIRED
      build_cmd: "make wasm" # REQUIRED (e.g., cargo build --target wasm32-wasi --release)
    run_core:
      command: ""
    anchor:
      receipt: "anchors/anchor-${TASK_ID}.json"
      sot_path: "SOT.md" # Append a sot_entry for this TASK_ID
    promote:
      destination: ""
```

## Minimal Per-Phase Contract (automation)

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
  merge The browse_page tool returned: The content is empty, only dashes, no markdown.
```
