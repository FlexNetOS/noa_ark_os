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
- **Storage doctor:** Run `noa storage doctor` (from `apps/cli`) before
  committing automation changes to confirm mirrored log pairs are present,
  begin with a genesis record, and remain in sync.

## Access

Use the CLI command `noa-cli evidence show` (see `apps/cli`) or subscribe to the
workflow event stream (`WorkflowEvent::StageReceiptGenerated`) for streaming
updates.

### CLI usage and filtering

Run `noa-cli evidence show` from a workspace root so the tool can locate
`storage/db/evidence/ledger.jsonl`. The command now supports several filters:

- `--workflow <ID>` narrows results to entries whose payload contains the
  matching `workflow_id`.
- `--kind <kind[,kind...]>` restricts output to specific ledger kinds
  (`genesis`, `stage_receipt`, `security_scan`, `task_dispatch`,
  `auto_fix_action`, `budget_decision`).
- `--since <millis>` / `--until <millis>` bound the timestamps (inclusive) so
  investigations can focus on a specific window.
- `--limit <N>` only prints the most recent `N` matching entries.
- `--verify-signatures` recomputes the operation hash, verifies the signature
  with the policy secret, and ensures the `previous_signature` chain is intact.
  Provide the same `NOA_POLICY_SECRET` that was used when emitting the ledger so
  verification succeeds.

When verification is enabled, each row prints either
`signature=verified` or `signature=INVALID(...)`. The CLI also emits a warning
summarising how many displayed rows failed validation.

### Diagnosing missing entries or corrupt chains

If ledger output shows `signature=INVALID(chain)` it means an entry's
`previous_signature` does not match the prior record—usually signalling a
missing line or tampering. Use `--since`/`--until` to narrow the range and
compare with mirrors in `.workspace/indexes/*.log`. Hash mismatches indicate the
payload was mutated after signing; cross-check the referenced artifact on disk
and regenerate the stage receipt if required. When both hash and chain validate
but an expected entry is absent, rebuild the ledger via
`PipelineInstrumentation` for the affected workflow and append the missing
record before rerunning verification.

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

## 2025-11-18 – Kernel Image Build (CORE-IMG-2025-11-18)

- **Artifacts**: `dist/kernel/noa_kernel`, `dist/kernel/noa_host_control`, `dist/kernel/manifest.yaml`, `dist/kernel/README.md`, `dist/kernel/test-results.log`
- **Inputs**: `core/config/default_manifest.yaml`, `server/tools/activate-cargo.sh`, `server/tools/activate-node.sh`, portable toolchains (cargo 1.91.1, rustc 1.91.1, node v20.19.5, pnpm 8.15.4)
- **Process**:
  1. `source ./server/tools/activate-cargo.sh && cargo build -p noa_core`
  2. `source ./server/tools/activate-cargo.sh && source ./server/tools/activate-node.sh && make image`
- **Verification**: `cargo test -p noa_core --tests -- --nocapture` emits 37 passing unit tests plus host control/world integration suites; results captured in `dist/kernel/test-results.log`.
- **Purpose**: Produces the hardened kernel binaries and accompanying manifest/test log for downstream packaging and Truth Gate review, confirming the offline toolchain remains hermetic.

## 2025-11-18 – Python Gateway & Manifest Verification (PY-VER-2025-11-18)

- **Artifacts**: `build_output/python-tests/test-results-2025-11-18.log`
- **Hashes**: `sha256:2b8f1df512f9a14c221f4153521bd0f268b19ddd254f8ca3ba7cbfc78bd309ac`
- **Inputs**: `tests/python/*.py`, `server/python/requirements.txt`, repo-managed virtual env `.venv` (python 3.12.3, pytest 7.4.3)
- **Process**:
  1. `python3 -m venv .venv && source .venv/bin/activate`
  2. `pip install -r server/python/requirements.txt`
  3. `python -m pytest tests/python | tee build_output/python-tests/test-results-2025-11-18.log`
- **Verification**: Suite result `14 passed, 1 skipped` covering `test_gateway.py`, `test_kernel_manifest.py`, `test_notebook_sync_token.py`, and auxiliary bundles ensures manifest descriptors align with service definitions and gateway routing stays healthy.
- **Purpose**: Documents that the Python verification stack ran inside the hermetic venv, producing audit-friendly logs and hashes for the Truth Gate.

## 2025-11-18 – Gateway Policy Self-Heal (GATEWAY-SH-2025-11-18)

- **Artifacts**: `build_output/gateway-self-heal.json`, `build_output/telemetry/self-heal-metrics.json`, `build_output/gateway-python-tests/gateway-python-tests.json`
- **Inputs**: `services/gateway/self_heal.py`, portable Python (repo `.venv`), kernel capability token helpers (`core/kernel/security/tokens.py`)
- **Process**:
  1. `python services/gateway/self_heal.py --output build_output/gateway-self-heal.json`
  2. Script issues kernel-signed capability tokens per service, exercises each policy route, and exports telemetry snapshots.
  3. Supplemental probe (`build_output/gateway-python-tests/gateway-python-tests.json`) captures manual gateway request permutations (missing token, client mismatch, scope enforcement) to preserve raw responses.
- **Verification**: All registered services (`gateway`, `runtime-manager`, `openai`, `anthropic`, `llama.cpp`, `notebook-sync`) returned HTTP 200 with policy and rate checks enforced; telemetry digest stored alongside the summary.
- **Purpose**: Establishes an auditable record that the refreshed self-heal ran offline and now guards `make pipeline.local`, preventing gateway policy regressions from progressing past the local pipeline.

## 2025-11-18 – Kernel Image Build (CORE-IMG-2025-11-18-R2)

- **Artifacts**: `dist/kernel/noa_kernel`, `dist/kernel/noa_host_control`, `dist/kernel/manifest.yaml`, `dist/kernel/README.md`, `dist/kernel/test-results.log`
- **Inputs**: `core/config/default_manifest.yaml`, portable toolchains activated via `./server/tools/activate-cargo.sh` and `./server/tools/activate-node.sh`, cargo 1.91.1/rustc 1.91.1/node v20.19.5/pnpm 8.15.4
- **Process**:
  1. `source ./server/tools/activate-cargo.sh && source ./server/tools/activate-node.sh && make kernel`
  2. `source ./server/tools/activate-cargo.sh && source ./server/tools/activate-node.sh && make image`
- **Verification**: `cargo test -p noa_core --tests -- --nocapture` plus the host-control/world suites executed automatically by `make image`; the resulting stdout is recorded in `dist/kernel/test-results.log`.
- **Purpose**: Confirms the hardened image can be regenerated deterministically and keeps the audit trail current for the Truth Gate.

## 2025-11-19 – Vault Runtime Contract (VAULT-HOME-2025-11-19)

- **Artifacts**: `.workspace/registry/environment.vault.json`, `server/vault/vault.hcl`,
  `server/vault/configure-gateway-auth-simple.sh`, `server/vault/README.md`,
  `docs/runtime/vault/README.md`
- **Purpose**: Replace hard-coded host paths with the gateway-managed
  `NOA_VAULT_HOME` setting so Vault data, runtime manifests, and gateway auth
  helpers resolve identically across machines.
- **Process**:
  1. Declared `NOA_VAULT_HOME` in the registry and documentation.
  2. Updated Vault Raft storage to use `{{env "NOA_VAULT_HOME"}}/data`.
  3. Taught `configure-gateway-auth-simple.sh` to export/seed the managed home
     and surface the override flag.
  4. Captured the new bootstrap flow in both `server/vault/README.md` and
     `docs/runtime/vault/README.md`.
- **Verification**: Manual review of generated paths plus script dry-run logic
  (file seeding) to ensure the new location is honored without requiring
  absolute paths.
