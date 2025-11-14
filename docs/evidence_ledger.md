# Evidence Ledger Specification

The evidence ledger supplements relocation/document logs by preserving
cryptographically chained artifacts for auditability.

## Layout

- **Location:** `storage/db/evidence/ledger.jsonl`
- **Format:** JSON Lines (UTF-8). Each entry is immutable.
- **Genesis:** Automatically appended when instrumentation initialises.

## Entry Structure

```json
{
  "kind": "stage_receipt",
  "timestamp": 1700000000000,
  "reference": "<merkle-root>",
  "payload": {
    "workflow_id": "example",
    "stage_id": "build",
    "stage_type": "sequential",
    "levels": [{"level":0,"nodes":["..."]}],
    "leaves": [{"index":0,"hash":"...","task_hash":"...","artifact_hash":"..."}]
  },
  "signed_operation": {
    "record": {"operation_id": "...", "kind": "stage_receipt", ...},
    "hash": "...",
    "signature": "...",
    "previous_signature": "..."
  }
}
```

Security scan entries follow the same structure with `kind="security_scan"`
and a payload describing the tool, issues, offline policy metadata, and artifact
path.

## Lifecycle & Retention

- **Append-only:** Never rewrite. To rotate, archive the entire file and emit a
  genesis entry for the new ledger.
- **Index mirroring:** Entries are mirrored into `.workspace/indexes/stage_receipts.log`
  and `.workspace/indexes/security_scans.log` for quick diffing.
- **Offline-first:** All scanners and workflows operate without network access;
  ledger entries must capture the offline cache provenance in their payload.
- **Verification:** Consumers recompute Merkle roots from `leaves` to confirm
  stage integrity. Signatures chain via `previous_signature`.

## Access

Use the CLI command `noa-cli evidence show` (see `apps/cli`) or subscribe to the
workflow event stream (`WorkflowEvent::StageReceiptGenerated`) for streaming
updates.

## Agent Verifier Metadata

- **Model Lifecycle Entries:** Training orchestrations emit verification records
  to `storage/db/evidence/ledger.jsonl` through the `FilesystemArtifactStore`
  implementation. Each entry mirrors the artifact checksum and the metrics
  published by `ml.lifecycle.controller` agent verifiers so reviewers can track
  promotion readiness.
- **Registry Cross-Reference:** The registry gateway appends a companion log at
  `registry/ml_artifacts.log` capturing lifecycle identifiers, artifact paths,
  and evaluation metrics for downstream drift detection workflows.
- **Verifier Responsibilities:** Agent verifiers must ensure metric thresholds
  provided in the evaluation plan are satisfied before writing ledger entries
  and must annotate any manual overrides in the `evaluation.notes` payload.

---
## Hermetic Toolchain Evidence
## 2025-11-14 – Portable Node/pnpm Bundle (HT-01)

- **Artifacts**: `server/tools/node-portable/current/bin/node`, `.../pnpm`
- **Versions**: Node v20.19.5, pnpm v8.15.4
- **Hashes**:
  - Node: `8d01d4c50e7a9047d70196f8fc6f1b165069065b44bd9402491544bd70586c7d`
  - pnpm: `7d26cc57186850a2d71ab77da7cf52ff0eeabf680ac446c8da2324aa63808aac`
- **Manifest**: `server/tools/node-portable.manifest.json`
- **Purpose**: Establishes the HT-01 hermetic Node toolchain mirrored into the workspace so Make targets and pnpm installs remain offline once cached.

## 2025-11-14 – Local Pipeline Evidence Gate (HT-03)

- **Artifacts**: `audit/local_pipeline_status.json`, `tools/git-hooks/pre-push.sh`, `.github/workflows/{ci,pipeline}.yml`
- **Inputs**: Hashes of `build_output.txt` and `test_output.txt`, commit SHA, tool versions from portable bundles
- **Process**: `scripts/pipeline/record_local_pipeline.sh` runs at the end of `make pipeline.local`, writing immutable metadata; git pre-push hook and GitHub Actions both execute `tools/ci/require_local_pipeline.py` to verify the evidence before any remote workflow proceeds.
- **Purpose**: Guarantees that every merge candidate has already passed the offline pipeline, keeping local execution authoritative and remote CI as a thin witness layer.
