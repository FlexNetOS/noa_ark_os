# Trust Pipeline Overview

The trust pipeline quantifies runtime confidence across three axes defined in
`core/kernel/north_star.deflex.json`:

- **Integrity** – Attestation, checksum, and audit log health.
- **Reversibility** – Rollback success ratios and recoverability.
- **Capability** – Breadth of capabilities that can remain available without
  manual review.

Each metric carries a weight and thresholds for warning and critical states.
Threshold breaches trigger escalation policies and scope reductions that the
workflow orchestrator enforces automatically.

## Scorekeeper workflow

1. `make scorekeeper` runs the `noa_scorekeeper` binary.
2. The binary parses `north_star.deflex.json`, computes the trust posture using
   supplied counters (overridable via the `TRUST_*` environment variables), and
   writes a timestamped snapshot to `metrics/trust_score.json`.
3. The kernel HTTP API serves the persisted snapshot via `GET /v1/trust`,
   enabling dashboards and policy engines to consume the current posture.
4. Workflow orchestration reads the same snapshot and shrinks optional tool
   scopes whenever the capability metric drops below its warning or critical
   thresholds.


The Makefile target accepts optional overrides:

```bash
TRUST_INTEGRITY_PASS=150 TRUST_INTEGRITY_FAIL=2 make scorekeeper
```

All counters default to conservative values suitable for CI smoke runs. For
production usage feed the binary with observed metrics collected from
telemetry, audit logs, and workflow receipts.

## Validation

Use the Python helper to confirm the policy schema is healthy:

```bash
python -m core.kernel.north_star
```

Pytests under `tests/python/test_north_star.py` exercise the validator, while
Rust unit tests inside `core::scorekeeper` cover snapshot generation and
persistence. Workflow regression tests confirm capability scopes shrink in
response to degraded trust.
