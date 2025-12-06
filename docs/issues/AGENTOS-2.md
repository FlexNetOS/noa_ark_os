# AGENTOS-2 — Design single-host AgentOS server profile

## Description
Define and validate a single-host profile that boots the full AgentOS stack with deterministic ordering, self-healing, and constrained resource envelopes.

## Checklist
- [ ] Capture service inventory, boot order, and health semantics for the single-host mode.
- [ ] Provide orchestration scripts or manifests that start/stop all services with one command.
- [ ] Integrate automated health and readiness probes tied to the kernel-first manifest.
- [ ] Bundle local observability (logs, metrics) into an operator-facing snapshot.
- [ ] Document resource envelopes and scaling guidance for baseline and high-availability hardware.

## Acceptance criteria
- One command provisions and boots the single-host stack to a ready state within defined SLAs.
- Health probes surface in a consolidated status view and recover failed services without manual intervention.
- Local observability bundle exposes logs and metrics with documented access steps.
- Documentation references AGENTOS-1 manifest without divergence.

## Meta
- Issue: https://github.com/noa-ark/noa_ark_os/issues/102
- Owner: codex
- Priority: P0
- Status: Processing
- Depends on: #101
- Blocked by: #101
- Blocks: #103, #104, #105
