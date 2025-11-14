# Event-Driven Triage and Remediation Service

The triage service (`scripts/triage_analyzer.py`) now runs as an event-driven
process that continuously inspects `out/triage/events/` (or a configured
location) for incident envelopes. Each event produces a structured incident
workspace under `tools/offline_pr_queue/triage/<timestamp>/<event-id>/` containing:

- `manifest.json` – raw payload, classification signals, and the policy decision
  that authorised remediation.
- Copied artefacts, such as the offending log file or directory tree.
- `remediation.log` – stdout/stderr from the CLI invocation that dispatched an
  auto-fix agent.

## Offline-First Operation

The service performs all work locally:

1. Events are read from the filesystem; no network listeners are required.
2. Artefacts are copied into the repository-managed incident tree for audit.
3. Remediation is triggered via the new Rust CLI (`workflow-cli`) using the
   default command `cargo run -p noa_workflow --bin workflow-cli --`. Supply a
   pre-built binary or alternate launcher with `--remediation-cli` when running
   in air-gapped mode.

## Failure Classification

Heuristics combine log content and event hints to categorise failures into
`lint`, `type`, `flaky_test`, `test`, or `infrastructure`. Classification
signals are recorded in the manifest and propagated to the auto-fix plan to
keep evidence and policy alignment intact.

## Auto-Fix Agents

Auto-fix automation lives in `workflow/src/auto_fixers.rs` and exposes four
specialists:

| Kind        | Plan Highlights                                                               |
|-------------|-------------------------------------------------------------------------------|
| `lint`      | `cargo fmt`, `cargo clippy`, and `pnpm lint`                                   |
| `type`      | `mypy`, `pnpm ts:check`, and `cargo check`                                     |
| `flaky-test`| `pytest` reruns, `cargo test --include-ignored`, and targeted JS test retries  |
| `refactor`  | `cargo clippy --fix`, guarded Python refactors, and workspace formatting       |

Each execution:

1. Materialises a plan and policy decision (`PolicyDecisionRecord`).
2. Records the snapshot via `PipelineInstrumentation::record_auto_fix_action`,
   producing a signed ledger entry with plan and policy digests.
3. Returns an `AutoFixActionReceipt` containing the policy statement and
   snapshot path for downstream reporting.

Run the CLI manually with:

```bash
cargo run -p noa_workflow --bin workflow-cli -- \
  auto-fix lint --incident tools/offline_pr_queue/triage/<...> --dry-run
```

## Budget Guardians

Resource guards in `workflow/src/budget_guardian.rs` monitor telemetry exported
by the gateway (`storage/telemetry/gateway_events.log`). They compute token
consumption and average latency across recent events and compare the results to
configurable limits (defaults: 2 000 tokens, 1 200 ms). When thresholds are
breached the guardian:

1. Removes tasks tagged with `"budget_sensitive": true` from the stage plan.
2. Records the decision through `PipelineInstrumentation::record_budget_decision`
   with a serialised copy of the rewritten stage.
3. Emits console output describing the action and ledger snapshot path.

If no safe rewrite exists the guardian escalates (action `escalate`) so the
planner can route the incident to human review while continuing execution.

You can inspect a stage in isolation with:

```bash
cargo run -p noa_workflow --bin workflow-cli -- \
  budget-check --workflow demo --stage ./stage.json
```

The command serialises the stage outcome, reports the guardian action, and
prints the ledger snapshot that documents the policy decision.

## Escalation and Evidence Discipline

Every triage run yields signed artefacts:

- Auto-fix actions append to `storage/db/auto_fix/` with policy digests in the
  immutable log.
- Budget decisions append to `storage/db/budget_guardian/`, ensuring that
  escalations and rewrites are auditable.
- Incident workspaces persist the raw evidence required for manual follow-up.

Escalations should feed into the normal playbooks under `docs/runbook/`; the
manifest identifies the triggering signals and CLI policy statement to speed up
manual handling.

## Restoring the Legacy Script Snapshot

Binary archives are not committed to the repository. To inspect the previous
`triage_analyzer.py`, restore the text-armoured snapshot and write it to a
scratch location:

```bash
python tools/archive_utils/autoload.py \
  archive/2025/11/scripts/triage_analyzer.py.archive.json \
  --output /tmp/triage_analyzer_legacy.py
```

The loader decodes the stored base64 payload and produces the original script so
you can diff or execute it without reintroducing binary blobs to git history.
