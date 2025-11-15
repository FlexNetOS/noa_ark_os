# AI DevOps Operations Runbook

## Overview

This runbook documents the agent-managed DevOps flow that now powers CI/CD and deployment
operations for NOA ARK OS. Human approvals have been replaced by deterministic checkpoints
that are executed and signed by planner, operator, and verifier agents. The pipelines
coordinate self-healing routines from `tools/` with gateway telemetry exported from
`services/gateway/`.

## Agent Roles

| Role      | Agent ID             | Responsibilities |
|-----------|----------------------|------------------|
| planner   | `PlanAgent`          | Executes preflight and workspace self-healing activities, prepares artefacts for downstream stages. |
| operator  | `OperationsBoardAgent` | Applies deployment updates and records operational evidence into shared ledgers. |
| verifier  | `ReviewAgent`        | Audits deployment reports, ensures telemetry and documentation are synchronised, and signs off checkpoints. |

The role directory is codified in `agents/data/agent_roles.json` and consumed by the
workflow engine to resolve agent implementations dynamically.

## Pipeline Flow

1. **Prepare / Self-Heal** – Planner agents invoke
   `tools/maintenance/workspace_optimization.sh` and
   `services/gateway/self_heal.py` to remediate local issues and validate gateway
   policy health. Outputs are captured as Markdown/JSON artefacts for checkpoint review.
2. **Documentation Refresh** – Documentation jobs run `cargo` automation followed by the
   documentation sync agent (`DOC_AGENT_BIN`). Planner checkpoints must exist before the
   docs stage will execute.
3. **Deployment Verification** – Operator and verifier agents append structured evidence
   to `docs/reports/AGENT_DEPLOYMENT_OUTCOMES.md` via workflow instrumentation. The
   verifier checkpoint blocks promotion until the report is updated.

## Checkpoint Semantics

Agent checkpoints replace manual approvals. Each checkpoint entry specifies the agent
role, the artefact that must exist, and the summary requirement. Pipelines fail fast if a
checkpoint is missing or if the relevant artefact has not been generated.

## Telemetry and Evidence


### Instrumentation Artefact Checklist

Before the pipelines execute, automation must confirm that the mirrored schema
assets are present:

- [Auto-fix action snapshots](../../storage/db/auto_fix/README.md) for repair
  plans authorised by policy.
- [Budget guardian manifests](../../storage/db/budget_guardian/README.md) for
  token and latency enforcement context.
- [Pipeline log mirror schema](../../storage/db/pipelines/README.md) so agents
  can stream signed ledger updates into evidence reviews.

These mirrors originate from `.workspace/indexes/` as described in
[`storage/db/README.md`](../../storage/db/README.md) and are mandatory inputs for
policy verification and downstream DevOps analytics.

- Gateway telemetry is exported automatically to `build_output/telemetry/self-heal-metrics.json` alongside
  the self-heal summary so verification agents can review it without mutating tracked service folders.
- Deployment evidence is appended to `docs/reports/AGENT_DEPLOYMENT_OUTCOMES.md`.
- Instrumentation ensures Merkle receipts, task dispatch logs, and deployment outcomes
  are synchronised with the evidence ledger in `storage/db/evidence/`.

## Verification Scripts

End-to-end verification scripts live in `tests/ci/`:

- `test_agent_managed_doc_pipeline.py` parses the pipelines to assert agent checkpoints
  and self-healing steps are in place.
- `test_gateway_self_heal.py` executes `services/gateway/self_heal.py` and validates
  the generated report.

Run the suite with `pytest tests/ci` after changes to pipelines or workflow automation.

## Operational Notes

- Pipelines are designed for offline execution; network calls are restricted to
  gateway-managed endpoints only.
- Always commit updates to `agent_roles.json` when introducing new agent checkpoints.
- Generated artefacts are preserved for audit by the workflow instrumentation, enabling
  rapid rollback and forensic review.
