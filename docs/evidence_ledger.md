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
