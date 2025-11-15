# Consolidation Merger • Workflow Guide

## Purpose

- Single, drillable workflow from user input to deployment with copy/paste templates.

## All-in-One Flow (User Input → Deployment)

user input → 4D (Deconstruct → Diagnose → Develop → Deliver) → subject.todo (Inbox) → (Under Analysis) → (Ready for Promotion) → agentask.todo (Current Tasks) → Author → Index/Sign → Seal/Policy → Tri-Run → Merge(D) → Verify/Contract → Run Core → WASM → Anchor → Promote

## Quick Checklist

- Capture: add raw request in agentask.subject.todo (Inbox)
- Analyze: apply 4D in (Under Analysis) and write a 1–3 sentence 4D summary
- Promote: move to (Ready for Promotion) and complete Promotion Checklist
- Create Task: add to agentask.todo (Current Tasks), link SUBJECT_ID → TASK_ID
- Execute: follow 9-phase steps with evidence and hashes
- Deploy: complete Promote step and record summary

## State Machine (Automatable)

- subject.todo
  - Inbox → Under Analysis (capture acknowledged)
  - Under Analysis → Ready for Promotion (Promotion Checklist passes)
  - Ready for Promotion → Archived Subjects (after TASK_ID mapping recorded)
- agentask.todo
  - Current Tasks → In Progress → Verify/Contract → Anchor → Promote → Completed

Gate criteria (machine-checkable)

```yaml
promotion_checklist:
  subject_id: "SUBJ-YYYYMMDD-NNN" # required, unique
  intent: true # brief outcome present
  acceptance: # ≥1 criterion required
    - "..."
  priority: ["A", "B", "C"] # one of
  proposed_owner: "agent-or-person" # non-empty
  risks_or_deps: true # noted
  ready_to_create_task: true # final switch

task_entry_requirements:
  task_id: "TASK-YYYYMMDD-NNN" # required, unique
  subject_id: "SUBJ-YYYYMMDD-NNN" # linkage back to subject
  owner: "agent-or-person"
  priority: ["A", "B", "C"]
  effort: ["XS", "S", "M", "L", "XL"]
  due: "YYYY-MM-DD"
  acceptance:
    - "..."
```

## Task Transition Gates (Automatable)

Define machine-checkable requirements for each task status transition.

```yaml
task_transition_gates:
  in_progress:
    require:
      - manifest.present # manifest file exists for TASK_ID
      - subject_id.linked # SUBJECT_ID is linked in task metadata
  verify_contract:
    require:
      - tests.passed # test suite executed with exit code 0
      - hashes.pre.present # HASHES_PRE.txt exists
      - hashes.post.present # HASHES_POST.txt exists
  anchor:
    require:
      - sot.appended # SOT.md updated with sot_entry for TASK_ID
      - evidence.paths.recorded # evidence links recorded in SOT entry
  promote:
    require:
      - deploy.proof # deployment record or destination written
  completed:
    require:
      - artifacts.linked # manifests, hashes, logs linked in SOT
```

## ID Conventions (Regex)

```yaml
id_conventions:
  subject_id: "^SUBJ-\\d{8}-\\d{3}$"
  task_id: "^TASK-\\d{8}-\\d{3}$"
```

## Automation Gates (Minimal Contract)

```yaml
automation_gates:
  require:
    - phases.index_sign.hashes_file
    - phases.verify_contract.tests
    - phases.verify_contract.hashes_post_file
    - phases.wasm.module
    - phases.wasm.target
    - phases.wasm.build_cmd
    - phases.anchor.sot_path
```

## Rollback Quickref

- Verify/Contract fails → stop → restore baseline → fix → return to Tri-Run.
- WASM build fails → stop → fix build config/target → re-run WASM step.
- Deploy failure → stop → roll back to last known good → reopen task (IN_PROGRESS).

## Evidence Paths Convention

- Hashes/baselines: data/snapshots/${TASK_ID}/
- Manifests/outputs: data/exports/manifests/${TASK_ID}.yaml
- Logs: logs/development/${TASK_ID}/

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
EFFORT: (XS|S|M|L|XL)
DUE: YYYY-MM-DD
ACCEPTANCE:
  - Criterion 1
  - Criterion 2
STATUS: PLANNED | IN_PROGRESS | BLOCKED | VERIFYING | ANCHORING | PROMOTED
Links: <evidence, PRs, hashes, manifests>
```

## Execution Quick Reference (maps to agentask.sop)

### Sequence (9 phases)

- Author: define strategy, intent, acceptance
- Index/Sign: hash inputs, snapshot baseline
- Seal/Policy: lock constraints, policies
- Tri-Run: dry-run/simulate/test plan
- Merge(D): perform changes safely (heal, don’t harm)
- Verify/Contract: tests, hashes, acceptance proof
- Run Core: run main workload
- WASM: build/package WASM artifacts (required)
- Anchor: write audit record (SOT), link evidence
- Promote: deploy or publish result

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
    index_sign:
      hashes_file: "HASHES_PRE.txt"
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
    run_core:
      command: ""
    wasm:
      module: "path/to/module" # REQUIRED
      target: "wasm32-wasi" # REQUIRED
      build_cmd: "make wasm" # REQUIRED (e.g., cargo build --target wasm32-wasi --release)
    anchor:
      sot_path: "SOT.md" # Append a sot_entry for this TASK_ID
    promote:
      destination: ""
```

## Flow Drilldown (stage-by-stage)

0. User Input

- Inputs: raw request (chat, issue, note)
- Action: capture in subject.todo (Inbox)
- Evidence: timestamped entry (author)
- Gate to Under Analysis: captured=true

1. 4D (Deconstruct → Diagnose → Develop → Deliver)

- Inputs: request context, constraints
- Action: write 1–3 sentence 4D Summary under Under Analysis
- Evidence: 4D Summary text
- Gate to Ready for Promotion: promotion_checklist pre-filled (intent, acceptance ≥1, priority, proposed owner, risks)

2. subject.todo → Ready for Promotion

- Inputs: Subject metadata block
- Action: complete Promotion Checklist
- Evidence: checklist ticks + SUBJECT_ID format matches regex
- Gate to task creation: ready_to_create_task=true

3. Create Task (agentask.todo Current Tasks)

- Inputs: SUBJECT_ID, metadata, acceptance
- Action: assign TASK_ID, OWNER, DUE, EFFORT; create manifest skeleton
- Evidence: task block + manifest file present
- Gate to Author: manifest.present=true

4. Author

- Inputs: task metadata, manifest
- Action: set strategy/intent in manifest.phases.author.notes
- Evidence: notes non-empty
- Gate to Index/Sign: notes.present=true

5. Index/Sign

- Inputs: inputs listed
- Action: write HASHES_PRE.txt; set manifest.phases.index_sign.hashes_file
- Evidence: data/snapshots/${TASK_ID}/HASHES_PRE.txt
- Gate to Seal/Policy: hashes.pre.present=true

6. Seal/Policy

- Inputs: policies/constraints
- Action: record constraints in manifest.phases.seal_policy.constraints
- Evidence: constraints array committed
- Gate to Tri-Run: constraints.locked=true

7. Tri-Run

- Inputs: plan, env
- Action: simulate/dry-run; record plan in manifest.phases.tri_run.plan
- Evidence: logs/development/${TASK_ID}/tri_run.log (optional)
- Gate to Merge(D): tri_run.plan.present=true

8. Merge(D)

- Inputs: approved plan
- Action: perform safe changes; list actions in manifest.phases.merge_d.actions
- Evidence: commit/ops log
- Gate to Verify/Contract: merge.actions.recorded=true

9. Verify/Contract

- Inputs: tests, acceptance
- Action: run tests; write HASHES_POST.txt; set manifest.phases.verify_contract.{tests,evidence,hashes_post_file}
- Evidence: tests.passed, data/snapshots/${TASK_ID}/HASHES_POST.txt
- Gate to Run Core: verify.pass=true

10. Run Core

- Inputs: manifest.phases.run_core.command
- Action: execute workload; capture logs
- Evidence: logs/development/${TASK_ID}/run_core.log
- Gate to WASM: run_core.done=true

11. WASM (required)

- Inputs: module path, target, build_cmd
- Action: build/package WASM; artifacts produced
- Evidence: artifact at module path; build logs
- Gate to Anchor: wasm.built=true

12. Anchor

- Inputs: SOT.md path
- Action: append sot_entry for TASK_ID
- Evidence: SOT entry present with evidence links
- Gate to Promote: sot.appended=true

13. Promote

- Inputs: destination
- Action: deploy/publish; record destination in manifest
- Evidence: deploy.proof (URL, path, tag)
- Gate to Completed: deploy.proof=true; artifacts.linked=true

### Minimal per-phase contract (automation)

```yaml
per_phase_contract:
  author:
    require: [manifest.phases.author.notes]
  index_sign:
    require: [manifest.phases.index_sign.hashes_file]
  seal_policy:
    require: [manifest.phases.seal_policy.constraints]
  tri_run:
    require: [manifest.phases.tri_run.plan]
  merge_d:
    require: [manifest.phases.merge_d.actions]
  verify_contract:
    require:
      [
        manifest.phases.verify_contract.tests,
        manifest.phases.verify_contract.hashes_post_file,
      ]
  run_core:
    require: [manifest.phases.run_core.command]
  wasm:
    require:
      [
        manifest.phases.wasm.module,
        manifest.phases.wasm.target,
        manifest.phases.wasm.build_cmd,
      ]
  anchor:
    require: [manifest.phases.anchor.sot_path]
  promote:
    require: [manifest.phases.promote.destination]
```

## Pointers

- Subject TODO: /home/deflex/projects/work/consolidation_merger/agentask.subject.todo
- Task TODO: /home/deflex/projects/work/consolidation_merger/agentask.todo
- SOP (details): /home/deflex/workspace/repos/task_exec_kit/agentask.sop
- SOT (ledger): /home/deflex/projects/work/consolidation_merger/SOT.md
