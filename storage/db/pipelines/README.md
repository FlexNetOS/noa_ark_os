# Pipeline Log Mirror Schema

Pipeline automation writes append-only ledgers to `.workspace/indexes/*.log` and
mirrors them into `storage/db/*.log`. Each file is line-delimited JSON where
every line deserialises into an `ImmutableLogEntry` defined in
`workflow/src/instrumentation.rs`.

## Log Entry Structure

```jsonc
{
  "event": {
    "event_type": "stage_receipt",             // Logical event name
    "actor": "workflow_engine",               // Component that emitted the event
    "scope": "workflow::stage",                // Hierarchical scope for the event
    "source": null,                             // Optional source identifier
    "target": null,                             // Optional target identifier
    "metadata": { ... },                        // Event-specific payload
    "timestamp": 1715704000789                  // Milliseconds since epoch
  },
  "policy": {
    "record": {
      "operation_id": "op_01H...",            // Stable identifier for the signed operation
      "kind": "StageReceipt",                  // OperationKind enum variant
      "actor": "workflow/run",                // Actor recorded for the policy decision
      "scope": "stage_name",                  // Scope recorded for the policy decision
      "source": null,
      "target": null,
      "metadata": { ... },                     // Structured metadata attached during signing
      "timestamp": 1715704000788
    },
    "hash": "3c2f...",                         // Hash of the policy record
    "signature": "8bc1...",                    // Signature chaining this entry to policy history
    "previous_signature": "GENESIS"            // Signature of the previous operation
  },
  "previous_hash": "GENESIS",                  // Hash of the previous ledger entry
  "entry_hash": "6fd9..."                      // Hash over event, policy, and previous_hash
}
```

The combination of `previous_hash` and the policy signature chain guarantees
that every log is tamper evident. Downstream services should verify both chains
before trusting automation outcomes.

## Log Files

The following ledgers are currently generated:

- `relocation.log`
- `document_log.log`
- `stage_receipt.log`
- `task_dispatch.log`
- `auto_fix.log`
- `budget_decision.log`
- `security_scan.log`
- `inference_log.log`
- `pipeline_events.log`

All files are mirrored 1:1 with `.workspace/indexes/`. Only
`PipelineInstrumentation` should append to them so signatures and hashes remain
valid.
