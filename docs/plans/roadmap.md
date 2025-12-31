<!-- BEGIN: PM_ROADMAP -->
# NOA ARK OS Project Management Roadmap

## Themes & Suggested Tasks

### 1. Kernel-first dependency graph & portable packaging
- **Intent:** Model runtime and build artifacts so kernels ship with reproducible dependency graphs and OCI-friendly bundles.
- **Suggested task:** [View task](#task-kernel-graph)

### 2. Single-host server profile (full stack on one machine)
- **Intent:** Provide an opinionated deployment that runs gateway, runtime, and UI on a single host for labs and demos.
- **Suggested task:** [View task](#task-single-host)

### 3. Unified UI/UX shell (desktop + web + immersive)
- **Intent:** Deliver a consistent shell that coordinates desktop, web, and immersive canvases with shared navigation patterns.
- **Suggested task:** [View task](#task-unified-shell)

### 4. Adaptive runtime/portability (edge/VM/containers)
- **Intent:** Ensure runtime policies adapt automatically to edge nodes, virtual machines, and containerized clusters.
- **Suggested task:** [View task](#task-adaptive-runtime)

### 5. Advanced gateway & observability (routing, security, metrics)
- **Intent:** Harden gateway routing, security, and observability with opinionated defaults and pluggable policy hooks.
- **Suggested task:** [View task](#task-gateway-observability)

### 6. Value-add ecosystem (blueprints, automation, analytics)
- **Intent:** Ship curated automation, analytics packs, and blueprints that accelerate end-user adoption.
- **Suggested task:** [View task](#task-ecosystem-value)

## Task Details

<a id="task-kernel-graph"></a>
### AGENTOS-1 — Establish kernel dependency graph & portable packaging
**Description:** Build a unified dependency graph for kernels covering runtime crates, models, and system services, then emit portable bundles compatible with OCI registries and air-gapped installs.
**Checklist**
- [ ] Catalogue kernel components and describe dependency edges in `runtime/kernel/graph.yaml`.
- [ ] Implement graph validation tooling with unit tests and CI guardrails.
- [ ] Produce packaging manifest writers for OCI and offline tarball outputs.
**Acceptance criteria**
- Graph validation command fails on missing or cyclic dependencies.
- Portable bundle installs succeed on a clean machine using documented steps.
**Meta**
- Owner: TBA
- Priority: P0
- Status: Proposed
- Depends on: None

<a id="task-single-host"></a>
### AGENTOS-2 — Design single-host AgentOS server profile
**Description:** Deliver a reference deployment profile that layers gateway, runtime services, storage, and UI on a single host with scripted bootstrap and health diagnostics.
**Checklist**
- [ ] Capture resource model and sequencing in `deployments/single-host/profile.md`.
- [ ] Automate provisioning via `scripts/deploy_single_host.ts` with smoke verification.
- [ ] Expose diagnostics dashboard surfacing health checks and component logs.
**Acceptance criteria**
- Bootstrap script completes on a clean VM using documented prerequisites.
- Health dashboard reports green status for all core services under nominal load.
**Meta**
- Owner: TBA
- Priority: P0
- Status: Proposed
- Depends on: AGENTOS-1

<a id="task-unified-shell"></a>
### AGENTOS-3 — Deliver unified UI/UX shell
**Description:** Create a shared shell framework that synchronizes navigation, session state, and notifications across desktop, web, and immersive canvases with adaptive layouts.
**Checklist**
- [ ] Publish shell architecture ADR covering state orchestration and extension points.
- [ ] Implement shared navigation and presence components with Storybook docs.
- [ ] Integrate immersive canvas bridge with feature-flagged rollout plan.
**Acceptance criteria**
- Shell components render consistently across target surfaces using design tokens.
- Feature flag toggles immersive canvas bridge without regressions in existing clients.
**Meta**
- Owner: TBA
- Priority: P1
- Status: Proposed
- Depends on: AGENTOS-2

<a id="task-adaptive-runtime"></a>
### AGENTOS-4 — Implement adaptive runtime portability policies
**Description:** Extend runtime scheduler to detect edge, VM, and container contexts, applying tuned placement, resource quotas, and failover behaviors.
**Checklist**
- [ ] Add environment detection heuristics with integration tests spanning edge, VM, and container fixtures.
- [ ] Encode adaptive policies in configuration schema with documentation.
- [ ] Instrument scheduler metrics for policy decisions and publish Grafana dashboards.
**Acceptance criteria**
- Runtime enforces context-appropriate policies verified by automated smoke tests.
- Observability dashboards surface policy decisions and outcomes per workload.
**Meta**
- Owner: TBA
- Priority: P1
- Status: Proposed
- Depends on: AGENTOS-2

<a id="task-gateway-observability"></a>
### AGENTOS-5 — Advance gateway security & observability mesh
**Description:** Harden gateway with policy-driven routing, zero-trust authentication adapters, and end-to-end observability spanning traces, metrics, and alerts.
**Checklist**
- [ ] Implement pluggable authn/z modules with conformance tests.
- [ ] Ship routing policy compiler supporting dry-runs and diff previews.
- [ ] Expand observability exporters and alert rules with documentation.
**Acceptance criteria**
- Security posture meets defined zero-trust baseline validated by penetration tests.
- Observability dashboards and alerts cover golden signals with actionable metadata.
**Meta**
- Owner: TBA
- Priority: P1
- Status: Proposed
- Depends on: AGENTOS-4

<a id="task-ecosystem-value"></a>
### AGENTOS-6 — Launch value-add automation & analytics kits
**Description:** Publish curated blueprints, automation packs, and analytics accelerators with pipelines, sample data, and guided onboarding inside the marketplace.
**Checklist**
- [ ] Curate blueprint catalog with validation harnesses and tagging metadata.
- [ ] Automate analytics pack deployment with fixture datasets and dashboards.
- [ ] Embed guided onboarding flows surfaced in the marketplace UI.
**Acceptance criteria**
- Marketplace lists ≥3 automation/analytics packs with verified install flows.
- Guided onboarding results in successful trial run captured by telemetry events.
**Meta**
- Owner: TBA
- Priority: P2
- Status: Proposed
- Depends on: AGENTOS-5
<!-- END: PM_ROADMAP -->
