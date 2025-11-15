# Storage Mirror Database

`storage/db` is the immutable mirror of the append-only ledgers that live under
`.workspace/indexes/`. `PipelineInstrumentation` writes every entry to the live
workspace index first, then replicates it into this directory so auditors can
inspect evidence without relying on the mutable working tree.

## Mirrored Assets

- **Pipeline logs (`*.log`)** – Each line is a JSON-serialised `ImmutableLogEntry`
  containing the event payload, the signed policy decision, and the hash of the
  previous entry. See [`pipelines/README.md`](pipelines/README.md) for details on
  the log schema and the list of log files (`relocation.log`,
  `document_log.log`, `stage_receipt.log`, `task_dispatch.log`,
  `auto_fix.log`, `budget_decision.log`, `security_scan.log`,
  `inference_log.log`, `pipeline_events.log`).
- **Goal analytics** – `analytics/goal_kpis.json` tracks goal-level success,
  latency, and throughput metrics that power the self-status surface.
- **Evidence ledger** – `evidence/ledger.jsonl` is an append-only JSON Lines
  file containing Merkle receipts, task dispatch outcomes, auto-fix actions,
  budget decisions, and security scans.
- **Auto-fix manifests** – `auto_fix/` captures the repair plans and policy
  context written by automated fixers.
- **Budget guardian manifests** – `budget_guardian/` captures enforcement
  decisions for token and latency budgets.

All files are append-only and should only be generated through
`PipelineInstrumentation` APIs so that signatures, hash chains, and audit
metadata remain valid across both mirrors.
