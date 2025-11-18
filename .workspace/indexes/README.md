# Immutable Pipeline Indexes

`PipelineInstrumentation` (`workflow/src/instrumentation.rs`) writes every
ledger entry to `.workspace/indexes/` first, then mirrors the same line into
`storage/db/`. The `.workspace` copy is treated as the canonical source while
pipelines are running; the `storage/db` mirror allows audits without mutating
developer working trees. All paths below are resolved relative to
`NOA_WORKFLOW_ROOT` (defaults to the repository root), so pointing that
environment variable at a snapshot automatically relocates these artefacts to
`<snapshot>/.workspace/indexes`.

## Quick Reference

| `.workspace/indexes/…` | Source API | Mirrored File | Purpose |
| --- | --- | --- | --- |
| `relocation.log` | `PipelineInstrumentation::log_relocation` | `storage/db/relocation.log` | Append-only record of every relocation/move that the workspace orchestrator executes. |
| `documentation.log` | `log_document_update` | `storage/db/documentation.log` | Tracks documentation edits that were routed through the workflow engine. |
| `stage_receipts.log` | `log_stage_receipt` | `storage/db/stage_receipts.log` | Carries Merkle receipts for each workflow stage along with leaf/task hashes. |
| `task_dispatches.log` | `log_task_dispatch` | `storage/db/task_dispatches.log` | Captures dispatch receipts (agent id, tool outputs, metadata). |
| `auto_fix_actions.log` | `record_auto_fix_action` | `storage/db/auto_fix_actions.log` | Records every approved auto-fix plan and links to the snapshot in `storage/db/auto_fix/`. |
| `budget_guardian.log` | `record_budget_decision` | `storage/db/budget_guardian.log` | Logs token/latency enforcement decisions together with action metadata. |
| `security_scans.log` | `log_security_scan` | `storage/db/security_scans.log` | Evidence for kernel/security scan executions and their signed operations. |
| `inference_metrics.log` | `log_inference_metric` | `storage/db/inference_metrics.log` | Telemetry about provider/model latency, token counts, and failures. |
| `pipeline_events.log` | `log_pipeline_event` | `storage/db/pipeline_events.log` | State transitions for pipelines (e.g. deployment state, trust score updates). |

> **Note:** These files are line-delimited JSON (`ImmutableLogEntry`) objects.
> Editing or reordering lines will break the hash chain enforced by the
> instrumentation policy checkers.

## Operational Notes

- `run_storage_doctor()` (available via `cargo run -p noa_cli -- storage doctor`)
	compares every `.workspace/indexes/*.log` file with its `storage/db/*.log`
	counterpart and reports drift, missing genesis entries, or absent mirrors.
- Auto-fix snapshots, budget manifests, and the evidence ledger themselves live
	under `storage/db/`; the `.workspace/indexes` copy only stores the signed
	event stream that points to those assets.
- When relocating or replaying a workspace snapshot, set `NOA_WORKFLOW_ROOT`
	before running workflow automation so that new entries land in the expected
	`.workspace/indexes` directory for that snapshot.
