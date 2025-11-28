# Budget Guardian Telemetry

This directory stores rolling summaries produced by the workflow `BudgetGuardian`.
Summaries are written to `rolling_summary.json` and include token and latency
percentiles derived from recent gateway telemetry events so Scorekeeper and
self-status surfaces can detect budget regressions offline.
# Budget Guardian Manifests

`PipelineInstrumentation::record_budget_decision` serialises enforcement actions
for workflow token and latency budgets. Each manifest is written to
`<timestamp>-<workflow>-<stage>-budget.json`, using the millisecond timestamp and
sanitised identifiers so that ordering and provenance are obvious.

## JSON Structure

```jsonc
{
  "workflow_id": "workflow/run::id",              // Workflow that triggered the decision
  "stage_id": "stage::id",                        // Stage evaluated by the guardian
  "recorded_at": 1715704000456,                    // Milliseconds since epoch
  "tokens_used": 4200.5,                           // Tokens consumed by the stage
  "token_limit": 5000.0,                           // Configured token budget
  "latency_ms": 18234.0,                           // Observed latency in milliseconds
  "latency_limit": 20000.0,                        // Configured latency budget
  "action": "continue",                           // Enforcement action (e.g., continue, pause)
  "rewritten_plan": { ... }                        // Optional JSON plan rewritten by the guardian, or null
}
```

All numeric fields are stored as floats to preserve precision for agents that
run budget analytics. When `rewritten_plan` is omitted it will appear as `null`
in the JSON payload.

## Generation Rules

- Manifests must be emitted through `PipelineInstrumentation` so that the
  companion `budget_decision.log` entry and evidence ledger update remain in
  lockstep.
- `.workspace/indexes/budget_guardian/` contains the live workspace copy; this
  directory maintains the signed mirror for offline review.
