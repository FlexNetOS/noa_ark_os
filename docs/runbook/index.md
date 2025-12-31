# NOA ARK OS Runbook
Generated: 2024-01-01T00:00:00+00:00

This runbook is compiled from the current SOP library and is intended to be executable during operations handoffs.

## Verification Matrix
| SOP | Orchestrator Role | Planner Role | Worker Role | Verifier Role |
| --- | --- | --- | --- | --- |
| Development | Routes intake, schedules phase gates | Expands task DAGs and dependencies | Executes build/test routines | Reviews evidence ledger before close |
| Release Management | Approves promotions and coordinates windows | Curates release manifests and rollback plans | Performs deployments and artifact pushes | Confirms rollout health and regression status |

## Operational Playbooks

### Development
- Source: `.workspace/sop/development.md`
- Last Reviewed: 2024-01-01T00:00:00+00:00
- Orchestrator: sequences backlog intake and sprint transitions
- Planner: maps implementation phases and dependency guards
- Worker: delivers code updates, tests, and documentation
- Verifier: validates evidence packages and sign-offs

### Release Management
- Source: `.workspace/sop/release.md`
- Last Reviewed: 2024-01-01T00:00:00+00:00
- Orchestrator: opens release windows and coordinates approvals
- Planner: maintains progressive delivery plans and rollback triggers
- Worker: executes promotions, packaging, and telemetry exports
- Verifier: certifies post-release metrics and rollback readiness
