# Gateway Policy Audit Ledger

The gateway now records every policy enforcement decision as a signed operation and persists the
resulting JSON line in `docs/verification/gateway_policy_audit.jsonl`.

## Entry Format

Each line contains the serialized `SignedOperation` emitted by `noa_core::security::enforce_operation`.
Relevant metadata for downstream evidence bundles:

- `record.operation_id` – Monotonic identifier for the decision.
- `record.kind` – Always `gateway_policy` to distinguish these events from other logs.
- `record.actor` – `user-{id}` that initiated the request.
- `record.scope` – Gateway policy identifier (e.g., `policy.read.intent`).
- `record.metadata` – Structured JSON payload mirroring the symbol schema contract. It includes
  intent name, capability/zones, trust scores, compliance tags, schema IDs, outcome, and reason.
- `signature` / `previous_signature` – Hash chain ensuring tamper detection.

## Usage

1. Consume the ledger via any JSONL-aware tooling (e.g., `rg`, `jq`, `python -m json.tool`).
2. Attach matching entries to audit packages under `docs/verification/` when promoting gateway
   releases or responding to compliance requests.
3. Hash the file after each program increment and reference it inside the relevant verification
   report (see `COMPLETE_SYSTEM_VERIFICATION.md`).

The ledger is append-only and created automatically the first time the gateway authorises or denies
an intent. Remove stale entries only after exporting the data into a signed verification bundle.
