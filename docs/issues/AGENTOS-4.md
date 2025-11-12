# AGENTOS-4 — Implement adaptive runtime orchestration

## Description
Extend runtime controllers to detect host capabilities, adjust workload placement, and surface portability guidance tied to the kernel graph.

## Suggested Tasks
- [Kernel-first dependency graph](../plans/gap_remediation_tasks.md#task-kernel-first)
- [Single-host profile boot plan](../plans/gap_remediation_tasks.md#task-single-host)
- [Adaptive runtime orchestration subtasks](../plans/gap_remediation_tasks.md#task-adaptive-runtime)

## Checklist
- [ ] Build capability detectors leveraging hardware, OS, and workload telemetry inputs.
- [ ] Map detection outputs to scheduling policies aligned with kernel-first dependencies.
- [ ] Implement fallback strategies for unsupported or degraded environments.
- [ ] Add simulation tests covering heterogeneous hosts and failure cases.
- [ ] Publish operator documentation for tuning adaptive policies.

## Acceptance Criteria
- Runtime automatically selects compatible workloads per host profile with auditable decisions.
- Portability guidance updates documentation and CLI feedback in sync.
- Simulation suite covers at least three heterogeneous scenarios and passes in CI.
- Adaptive controls remain configurable via the single-host profile without manual patching.

## Meta
- Issue: https://github.com/noa-ark/noa_ark_os/issues/104
- Owner: codex
- Priority: P1
- Status: Processing
- Depends on: #101, #102
- Blocked by: #101, #102
- Blocks: #106
- Owner: TBA
- Priority: P1
- Status: Proposed
- Depends on: [AGENTOS-1](../plans/gap_remediation_tasks.md#agentos-1), [AGENTOS-2](../plans/gap_remediation_tasks.md#agentos-2)

## Stakeholder Coordination
- 2025-11-12: Loop-in sent to Systems Memory Crew (runtime) via systems-memory@noa-ark.example requesting estimation and staffing availability.
- 2025-11-12: Loop-in sent to Virtual Filesystem Crew (infrastructure) via vfs@noa-ark.example requesting deployment portability staffing.

## Documentation Alignment
- 2025-11-12: Architecture & Docs Guild (docs@noa-ark.example) notified so roadmap materials can link to this tracking item.
