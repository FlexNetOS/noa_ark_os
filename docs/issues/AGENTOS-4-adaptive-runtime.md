# AGENTOS-4 — Implement adaptive runtime orchestration

## Description
Extend runtime controllers to detect host capabilities, adjust workload placement, and surface portability guidance tied to the kernel graph.

## Checklist
- [ ] Build capability detectors leveraging hardware, OS, and workload telemetry inputs.
- [ ] Map detection outputs to scheduling policies aligned with kernel-first dependencies.
- [ ] Implement fallback strategies for unsupported or degraded environments.
- [ ] Add simulation tests covering heterogeneous hosts and failure cases.
- [ ] Publish operator documentation for tuning adaptive policies.

## Acceptance criteria
- Runtime automatically selects compatible workloads per host profile with auditable decisions.
- Portability guidance updates documentation and CLI feedback in sync.
- Simulation suite covers at least three heterogeneous scenarios and passes in CI.
- Adaptive controls remain configurable via the single-host profile without manual patching.

## Meta
- Owner: TBA
- Priority: P1
- Status: Proposed
- Depends on: AGENTOS-1, AGENTOS-2
- Watchers: @runtime-team
- Linked workstreams:
  - [Runtime schedulers](../../runtime)
  - [Capability detection probes](../../core/hardware)

## Coordination
- Coordinate with AGENTOS-1 and AGENTOS-2 owners to reuse manifests and single-host controls when publishing the external issue link.
