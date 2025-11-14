# Agentic Kernel Launch Tasks

The following launch-ready tasks are sourced from [`docs/projects/ROADMAP_AGENTIC_KERNEL.md`](../projects/ROADMAP_AGENTIC_KERNEL.md) and cross-referenced with the active codebase so every link represents a 100%-healthy, AI-first objective. Dispatch the `task://` links to hand work directly to automation agents.

## Roadmap Source Verification

| Roadmap Section | Anchor | Primary Owners |
| --- | --- | --- |
| Phase 0 — Repo Foundation & Dead Code Quarantine | [`PHASE 0`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-0--repo-foundation--dead-code-quarantine) | `core/src/kernel.rs`, `tools/portable_builder.py`, `archive/2025/11/ledger.json` |
| Phase 0.5 — CLI-First Foundation | [`PHASE 0.5`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-05--cli-first-foundation--priority-0) | `apps/cli/Cargo.toml`, `apps/cli/src/main.rs` |
| Phase 1 — Kernel-First Baseline | [`PHASE 1`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-1--kernel-first-baseline) | `core/src/kernel.rs`, `core/src/runtime.rs`, `core/src/metrics.rs` |
| Phase 2 — North Star Implementation | [`PHASE 2`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-2--north-star-implementation) | `core/src/security.rs`, `core/src/gateway.rs`, `docs/architecture/kernel-first.md` |
| Phase 3 — World Model & Contract Tests | [`PHASE 3`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-3--world-model--contract-tests) | `core/src/config/manifest.rs`, `registry/capabilities.json`, `tests/integration_test.rs` |
| Phase 4 — CLI Expansion & Registry Completion | [`PHASE 4`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-4--cli-expansion--registry-completion) | `apps/cli/src/main.rs`, `registry/capabilities.json`, `docs/architecture/unified_shell.md` |
| Phase 5 — Gateway Slimming & Capability Tokens | [`PHASE 5`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-5--gateway-slimming--capability-tokens) | `core/src/gateway.rs`, `core/src/capabilities/builtin.rs`, `AGENT.md` |
| Phase 6 — Performance & Retrieval Discipline | [`PHASE 6`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-6--performance--retrieval-discipline) | `core/src/memory.rs`, `core/src/runtime.rs`, `storage/telemetry/README.md` |
| Phase 7 — Reward System | [`PHASE 7`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-7--reward-system-system-health-first) | `core/src/metrics.rs`, `storage/analytics/README.md` |
| Phase 8 — SBOM Split & Supply-Chain Integrity | [`PHASE 8`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-8--sbom-split--supply-chain-integrity) | `cicd/pipelines/crc-auto.yaml`, `docs/audits.md`, `Makefile` |
| Phase 9 — Deployment Profiles | [`PHASE 9`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-9--deployment-profiles-drop-in-everywhere) | `server/profiles/`, `runtime/manager`, `core/src/security.rs` |
| Phase 10 — Machine-First Pipelines | [`PHASE 10`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-10--tests-cicd-and-audit-trail-machine-first) | `cicd/pipelines/crc-auto.yaml`, `workflow/templates/merge/README.md`, `Makefile` |
| Phase 11 — Documentation & Agent Policy | [`PHASE 11`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-11--documentation--agent-policy) | `AGENT.md`, `docs/operations/firewall.md`, `policies/` |
| Phase 12 — Intelligent Growth & Self-Maintenance | [`PHASE 12`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-12--intelligent-growth--self-maintenance) | `agents/src/runtime.rs`, `tools/offline_pr_queue/`, `runtime/manager` |
| Phase 13 — Humanless Autonomy | [`PHASE 13`](../projects/ROADMAP_AGENTIC_KERNEL.md#phase-13--humanless-autonomy) | `workflow/flows/merge/`, `cicd/workflows/`, `tools/offline_pr_queue/` |
| End-Cap Automation Streams | [`End-Cap Automation Tasks`](../projects/ROADMAP_AGENTIC_KERNEL.md#end-cap-automation-tasks) | `tools/offline_pr_queue/`, `workflow/flows/`, `tools/maintenance/` |

The anchors above confirm that each task link maps to the authoritative roadmap sections in `docs/projects/ROADMAP_AGENTIC_KERNEL.md`.

## Phase 0 — Repo Foundation & Dead Code Quarantine
- **Roadmap Anchor:** PHASE 0 — Repo Foundation & Dead Code Quarantine
- **Code Touchpoints:** `core/src/kernel.rs`, `tools/portable_builder.py`, `archive/2025/11/ledger.json`
- **Outcome:** Establish quarantined component registry, reversible snapshots, and CI guardrails so the kernel never boots with zombie code.
- **Launch:** [Activate Task](task://AK-0)

## Phase 0.5 — CLI-First Foundation (Critical Path)
- **Roadmap Anchor:** PHASE 0.5 — CLI-First Foundation
- **Code Touchpoints:** `apps/cli/Cargo.toml`, `apps/cli/src/main.rs`
- **Outcome:** Elevate the `noa` CLI into the primary machine interface with JSON/YAML/TUI streaming outputs for autonomous agents.
- **Launch:** [Activate Task](task://AK-0.5)

## Phase 1 — Kernel-First Baseline
- **Roadmap Anchor:** PHASE 1 — Kernel-First Baseline
- **Code Touchpoints:** `core/src/kernel.rs`, `core/src/runtime.rs`, `core/src/metrics.rs`
- **Outcome:** Bundle a sovereign kernel package with host-control APIs, policy engine integration, and snapshot-ready SBOM hooks.
- **Launch:** [Activate Task](task://AK-1)

## Phase 2 — North Star Implementation
- **Roadmap Anchor:** PHASE 2 — North Star Implementation
- **Code Touchpoints:** `core/src/security.rs`, `core/src/gateway.rs`, `core/src/metrics.rs`, `docs/architecture/kernel-first.md`
- **Outcome:** Ship `north_star.deflex.json`, trust score computation, and adaptive scope gating so machine health dictates access.
- **Launch:** [Activate Task](task://AK-2)

## Phase 3 — World Model & Contract Tests
- **Roadmap Anchor:** PHASE 3 — World Model & Contract Tests
- **Code Touchpoints:** `core/src/config/manifest.rs`, `registry/capabilities.json`, `docs/architecture/registry-knowledge-graph.md`, `tests/integration_test.rs`
- **Outcome:** Author the world graph schema, drift reconciler, and contract-test harness that keep registry truth aligned with reality.
- **Launch:** [Activate Task](task://AK-3)

## Phase 4 — CLI Expansion & Registry Completion
- **Roadmap Anchor:** PHASE 4 — CLI Expansion & Registry Completion
- **Code Touchpoints:** `apps/cli/src/main.rs`, `apps/cli/Cargo.toml`, `registry/capabilities.json`, `docs/architecture/unified_shell.md`
- **Outcome:** Deliver the 40+ command CLI suite, machine-optimized query paths, and plugin system that enforces AI-first workflows.
- **Launch:** [Activate Task](task://AK-4)

## Phase 5 — Gateway Slimming & Capability Tokens
- **Roadmap Anchor:** PHASE 5 — Gateway Slimming & Capability Tokens
- **Code Touchpoints:** `core/src/gateway.rs`, `core/src/capabilities/builtin.rs`, `AGENT.md`
- **Outcome:** Enforce kernel-issued capability tokens for every adapter, keeping gateways lightweight while the kernel retains authority.
- **Launch:** [Activate Task](task://AK-5)

## Phase 6 — Performance & Retrieval Discipline
- **Roadmap Anchor:** PHASE 6 — Performance & Retrieval Discipline
- **Code Touchpoints:** `core/src/memory.rs`, `core/src/runtime.rs`, `storage/telemetry/README.md`, `tools/maintenance/workspace_optimization.sh`
- **Outcome:** Build hierarchical memory, retrieval budgets, and streaming diff tooling so agents stay fast and token-efficient.
- **Launch:** [Activate Task](task://AK-6)

## Phase 7 — Reward System (System-Health First)
- **Roadmap Anchor:** PHASE 7 — Reward System (System-Health First)
- **Code Touchpoints:** `core/src/metrics.rs`, `storage/analytics/README.md`, `storage/telemetry/README.md`
- **Outcome:** Generate reward and penalty loops that reinforce healthy agent behavior, persist history, and surface dashboards.
- **Launch:** [Activate Task](task://AK-7)

## Phase 8 — SBOM Split & Supply-Chain Integrity
- **Roadmap Anchor:** PHASE 8 — SBOM Split & Supply-Chain Integrity
- **Code Touchpoints:** `cicd/pipelines/crc-auto.yaml`, `docs/audits.md`, `Makefile`, `tools/security/`
- **Outcome:** Produce reproducible kernel and extension SBOMs with signatures and attestations that block unverified artifacts.
- **Launch:** [Activate Task](task://AK-8)

## Phase 9 — Deployment Profiles (Drop-In Everywhere)
- **Roadmap Anchor:** PHASE 9 — Deployment Profiles (Drop-In Everywhere)
- **Code Touchpoints:** `server/profiles/`, `runtime/manager`, `core/src/security.rs`
- **Outcome:** Define single-host, air-gapped, devcontainer, and edge profiles so the kernel issues scope-appropriate tokens by configuration.
- **Launch:** [Activate Task](task://AK-9)

## Phase 10 — Machine-First Pipelines
- **Roadmap Anchor:** PHASE 10 — Tests, CI/CD, and Audit Trail (Machine-First)
- **Code Touchpoints:** `cicd/pipelines/crc-auto.yaml`, `workflow/templates/merge/README.md`, `Makefile`, `tools/offline_pr_queue/`
- **Outcome:** Stand up the `make pipeline.local` execution chain with audit bundles, mandatory health gates, and AI-run DevOps rituals.
- **Launch:** [Activate Task](task://AK-DEVOPS)

## Phase 11 — Documentation & Agent Policy
- **Roadmap Anchor:** PHASE 11 — Documentation & Agent Policy
- **Code Touchpoints:** `AGENT.md`, `docs/operations/firewall.md`, `policies/`
- **Outcome:** Replace human job titles across docs and policies with planner/worker/verifier/orchestrator agents and codify machine governance.
- **Launch:** [Activate Task](task://AK-ROLES)

## Phase 12 — Intelligent Growth & Self-Maintenance
- **Roadmap Anchor:** PHASE 12 — Intelligent Growth & Self-Maintenance
- **Code Touchpoints:** `agents/src/runtime.rs`, `agents/src/implementations/`, `tools/offline_pr_queue/`, `runtime/manager`
- **Outcome:** Empower self-repair agents with trust gates, reflection APIs, and recurring improvement plans to keep the system evolving autonomously.
- **Launch:** [Activate Task](task://AK-12)

## Phase 13 — Humanless Autonomy
- **Roadmap Anchor:** PHASE 13 — Humanless Autonomy
- **Code Touchpoints:** `workflow/flows/merge/`, `cicd/workflows/`, `tools/offline_pr_queue/`, `docs/operations/`
- **Outcome:** Achieve 72+ hour autonomous operation with auto-PRs, auto-merges, fire-drill rollbacks, and signed releases free from human gating.
- **Launch:** [Activate Task](task://AK-13)

## AI MLOps — Autonomous Model Stewardship
- **Roadmap Anchors:** Phase 4 (CLI Expansion), Phase 6 (Retrieval Discipline), Phase 12 (Self-Maintenance)
- **Code Touchpoints:** `ai/README.md`, `inference/src/lib.rs`, `cicd/pipelines/`, `storage/telemetry/README.md`
- **Outcome:** Orchestrate continuous training, evaluation, drift detection, and rollout governed by agent roles with full evidence logging.
- **Launch:** [Activate Task](task://AK-MLOPS)

## End-Cap Automation Streams

### Error Auto-Triage
- **Roadmap Anchor:** End-Cap Automation Tasks — Error Auto-Triage
- **Code Touchpoints:** `workflow/flows/merge/README.md`, `tools/offline_pr_queue/triage/`, `tools/github/github_coding_agent.sh`
- **Outcome:** Auto-classify failures, open issues with repro metadata, and prime fix workflows for downstream agents.
- **Launch:** [Activate Task](task://AK-TRIAGE)

### Auto-Fixer Agents
- **Roadmap Anchor:** End-Cap Automation Tasks — Auto-Fixer Agents
- **Code Touchpoints:** `agents/src/implementations/`, `tools/maintenance/workspace_optimization.sh`, `docs/autonomous/INFINITE_OPTIMIZATION_GUIDE.md`
- **Outcome:** Deploy agent specialists that land low-risk patches (lint, types, flaky tests) under snapshot, test, and policy gates.
- **Launch:** [Activate Task](task://AK-AUTOFIX)

### Budget Guardians
- **Roadmap Anchor:** End-Cap Automation Tasks — Budget Guardians
- **Code Touchpoints:** `core/src/time.rs`, `core/src/metrics.rs`, `storage/telemetry/README.md`
- **Outcome:** Enforce token, latency, and resource budgets by automatically throttling or rewriting plans that exceed machine health thresholds.
- **Launch:** [Activate Task](task://AK-BUDGET)

### Local-First CI Harness
- **Roadmap Anchor:** End-Cap Automation Tasks — Local-First CI Harness
- **Code Touchpoints:** `Makefile`, `cicd/pipelines/crc-auto.yaml`, `workflow/templates/merge/README.md`, `tools/offline_pr_queue/`
- **Outcome:** Provide a fully offline-capable CI harness (`pipeline.local`) mirroring GitHub automation so agents certify health before remote pushes.
- **Launch:** [Activate Task](task://AK-LOCALCI)

---

### Usage Notes
1. Dispatch tasks via the provided `task://` links to ensure planners inherit the canonical scope and dependencies listed in `docs/tasks/index.json`.
2. Update `docs/tasks/index.json` after completing a task to keep roadmap automation synchronized.
3. Record health metrics and evidence in the ledger (`storage/db/evidence/`) so downstream agents can verify "100% healthy" outcomes.
