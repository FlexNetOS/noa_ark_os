# AGENTOS-1 — Establish kernel-first dependency graph & portable packaging

## Description
Build a canonical dependency map anchored on the kernel/service graph and package baselines that run identically on local, cloud, and air-gapped hosts.

## Checklist
- [ ] Document the kernel-centered dependency graph with required and optional services.
- [ ] Produce reproducible build artifacts for each service with pinned interfaces.
- [ ] Provide environment-agnostic packaging (container, tarball, OCI layout) driven from one manifest.
- [ ] Implement compatibility validation covering Linux, macOS, and containerized targets.
- [ ] Automate dependency compliance checks inside CI to guard drift.

## Acceptance criteria
- Kernel-first dependency graph published with machine-readable manifest and human guide.
- Packaging workflow produces artifacts that pass smoke tests on Linux, macOS, and container runners.
- CI blocks merges when dependency or interface drift is detected.
- Downstream task owners acknowledge the manifest as their source of truth.

## Meta
- Owner: TBA
- Priority: P0
- Status: Proposed
- Depends on: None
- Watchers: @platform-architecture
- Linked workstreams:
  - [Kernel service manifests](../../core/kernel)
  - [Packaging automation](../../scripts/packaging)

## Coordination
- Align with roadmap maintainers to publish the canonical kernel-first manifest and link downstream tasks to this issue when external tracking is available.
