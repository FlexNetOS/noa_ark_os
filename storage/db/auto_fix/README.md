# Auto-Fix Action Snapshots

`PipelineInstrumentation::record_auto_fix_action` emits one JSON document per
auto-fix attempt. Files are named `<timestamp>-<fixer>-auto-fix.json` where the
prefix is the millisecond timestamp and the fixer identifier is sanitised for
filesystem safety.

## JSON Structure

```jsonc
{
  "fixer": "agents/self_heal/repair_bot",        // Actor that executed the fix
  "target": "workflow/stage::identifier",        // Workflow scope receiving the fix
  "recorded_at": 1715704000123,                   // Milliseconds since epoch
  "plan": { ... },                                // Arbitrary JSON plan emitted by the fixer
  "policy": {
    "decision": "approve",                      // Policy verdict applied to the plan
    "reason": "policy rationale",               // Human-readable justification
    "signals": ["merkle::verified"],            // Optional signals that informed the decision
    "metadata": { ... }                           // Optional structured context
  }
}
```

The `plan` payload is stored exactly as provided by the fixer agent. The `policy`
block matches `PolicyDecisionRecord` from `workflow/src/instrumentation.rs` and
captures the policy engine's justification for allowing or rejecting the fix.

## Generation Rules

- Files are append-only and should only be created through
  `PipelineInstrumentation` so the corresponding auto-fix log entry and evidence
  ledger entry remain in sync.
- The mirror in `.workspace/indexes/auto_fix/` always contains the original
  emission; this directory is the immutable copy that downstream auditors read.
