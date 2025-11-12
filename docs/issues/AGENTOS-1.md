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
- Issue: https://github.com/noa-ark/noa_ark_os/issues/101
- Owner: codex
- Priority: P0
- Status: Processing
- Depends on: None
- Blocked by: None
- Blocks: #102, #104
