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

<a id="task-agentic-program"></a>
### AGENTOS-0 — Activate Agentic PM governance program
**Description:** Publish a single source of truth for AGENTOS-0…10 so automation, planners, and humans reference the exact same dependency graph, acceptance notes, and evidence links.
**Checklist**
- [x] Export machine-readable roadmap (`build_kits/pm_roadmap.json`/`.csv`) with acceptance notes.
- [x] Document evidence and cross-links inside `docs/roadmaps/agentic_pm_unification.md`.
- [x] Wire roadmap exporters (`scripts/export_roadmap.ts`) so agents ingest the same ledger.
**Acceptance criteria**
- Roadmap docs enumerate AGENTOS-0..10 with links to implementation evidence.
- PM ledger can be consumed by automation without scraping prose.
**Meta**
- Owner: Agentic PM Office
- Priority: P0
- Status: Completed
- Depends on: None

<a id="task-kernel-graph"></a>
### AGENTOS-1 — Establish kernel dependency graph & portable packaging
**Description:** Build a unified dependency graph for kernels covering runtime crates, models, and system services, then emit portable bundles compatible with OCI registries and air-gapped installs.
**Checklist**
- [x] Catalogue kernel components and describe dependency edges in `runtime/kernel/graph.yaml`.
- [x] Implement graph validation tooling with unit tests and CI guardrails (`cargo run -p runtime_manager --bin validate_kernel_graph`).
- [x] Produce portable bundles via `tools/portable_builder.py` for OCI/offline installs.
**Acceptance criteria**
- Graph validation command fails on missing or cyclic dependencies.
- Portable bundle installs succeed on a clean machine using documented steps.
**Meta**
- Owner: Kernel & Runtime Team
- Priority: P0
- Status: Completed
- Depends on: None

<a id="task-single-host"></a>
### AGENTOS-2 — Design single-host AgentOS server profile
**Description:** Deliver a reference deployment profile that layers gateway, runtime services, storage, and UI on a single host with scripted bootstrap and health diagnostics.
**Checklist**
- [x] Capture resource model and sequencing in `docs/deployments/single_host.md` + `server/profiles/single_host/profile.toml`.
- [x] Automate provisioning via `services/single-host/init/noa-single-host.sh` (systemd + container entrypoints).
- [x] Expose diagnostics dashboard/metrics endpoints (see `[profile.runtime.metrics_endpoint]` + CI smoke checks).
**Acceptance criteria**
- Bootstrap script completes on a clean VM using documented prerequisites.
- Health dashboard reports green status for all core services under nominal load.
**Meta**
- Owner: Kernel & Runtime Team
- Priority: P0
- Status: Completed
- Depends on: AGENTOS-1

<a id="task-unified-shell"></a>
### AGENTOS-3 — Deliver unified UI/UX shell
**Description:** Create a shared shell framework that synchronizes navigation, session state, and notifications across desktop, web, and immersive canvases with adaptive layouts.
**Checklist**
- [x] Capture architecture + extension points inside `docs/roadmaps/agentic_pm_unification.md`.
- [x] Implement shared navigation/presence modules in `ui/core/src/module.rs` + `shell.rs`.
- [x] Integrate immersive bridge via `scripts/dev/full_system_launch.sh` feature flags + `server/ui_api/src/schema.rs`.
**Acceptance criteria**
- Shell components render consistently across target surfaces using design tokens.
- Feature flag toggles immersive canvas bridge without regressions in existing clients.
**Meta**
- Owner: Agentic Shell Team
- Priority: P1
- Status: Completed
- Depends on: AGENTOS-2

<a id="task-adaptive-runtime"></a>
### AGENTOS-4 — Implement adaptive runtime portability policies
**Description:** Extend runtime scheduler to detect edge, VM, and container contexts, applying tuned placement, resource quotas, and failover behaviors.
**Checklist**
- [x] Add environment detection heuristics/tests in `runtime/manager/src/lib.rs` (AdaptiveRuntimeController + WASM probes).
- [x] Encode adaptive policies inside `server/src/adaptive_scaling.rs` and `server/profiles/single_host/profile.toml`.
- [x] Instrument scheduler metrics through `noa_core::metrics` + Prometheus endpoints for dashboards/alerts.
**Acceptance criteria**
- Runtime enforces context-appropriate policies verified by automated smoke tests.
- Observability dashboards surface policy decisions and outcomes per workload.
**Meta**
- Owner: Kernel & Runtime Team
- Priority: P1
- Status: Completed
- Depends on: AGENTOS-2

<a id="task-gateway-observability"></a>
### AGENTOS-5 — Advance gateway security & observability mesh
**Description:** Harden gateway with policy-driven routing, zero-trust authentication adapters, and end-to-end observability spanning traces, metrics, and alerts.
**Checklist**
- [x] Implement pluggable authn/z + rate limiting in `server/gateway/src/auth.rs` & `rate_limit.rs`.
- [x] Ship routing policy compiler/dry-run hooks in `server/gateway/src/policy.rs`.
- [x] Expand telemetry exporters (`server/gateway/src/telemetry.rs`) logging to `storage/telemetry/` with docs.
**Acceptance criteria**
- Security posture meets defined zero-trust baseline validated by penetration tests.
- Observability dashboards and alerts cover golden signals with actionable metadata.
**Meta**
- Owner: Gateway & Security Team
- Priority: P1
- Status: Completed
- Depends on: AGENTOS-4

<a id="task-ecosystem-value"></a>
### AGENTOS-6 — Launch value-add automation & analytics kits
**Description:** Publish curated blueprints, automation packs, and analytics accelerators with pipelines, sample data, and guided onboarding inside the marketplace.
**Checklist**
- [x] Curate bundle catalog in `services/marketplace` + `apps/marketplace/catalog/*.json`.
- [x] Automate analytics/automation pack deployment with fixture datasets and telemetry hooks.
- [x] Embed onboarding flows surfaced in the Marketplace UI + documentation.
**Acceptance criteria**
- Marketplace lists ≥3 automation/analytics packs with verified install flows.
- Guided onboarding results in successful trial run captured by telemetry events.
**Meta**
- Owner: Marketplace Team
- Priority: P2
- Status: Completed
- Depends on: AGENTOS-5

<a id="task-agentic-shell"></a>
### AGENTOS-7 — Rejoin dashboard and Kanban via Agentic PM shell
**Description:** Embed the Vibe Kanban workspace, planner memory, CRC uploads, and workflow launchers inside the shared `ui/core` shell while retiring static dashboard entry points.
**Checklist**
- [x] Register Kanban widgets/workflows via `server/ui_api/src/schema.rs` + `ui/core/src/module.rs`.
- [x] Ensure CRC uploads, planner memory, automation panels render inside the shell (`ui/vibe-kanban/app/components/*`).
- [x] Document the unified navigation/retirement of `ui/noa-dashboard` inside this roadmap.
**Acceptance criteria**
- A single navigation model renders Kanban analytics, planner memory, CRC uploads, and workflow tiles across desktop/web/mobile/XR.
- Legacy ui/noa-dashboard entry points are removed or redirect to the Agentic shell without functionality loss.
**Meta**
- Owner: Agentic Shell Team
- Priority: P1
- Status: Completed
- Depends on: AGENTOS-3

<a id="task-agentic-cross-platform"></a>
### AGENTOS-8 — Ship cross-platform interactive Agentic UI packages
**Description:** Package the unified shell for desktop (Tauri), mobile (Expo), and XR with interactive widgets (workflow launchers, agent critique feeds, drag-drop CRC drops) calling the unified API contract.
**Checklist**
- [x] Keep schema + tokens in `ui/shared/src/schema.ts`/`samples/vibe-dashboard.ts`.
- [x] Surface equivalent experiences via `apps/desktop-shell`, `apps/mobile-shell`, and `apps/xr-shell`.
- [x] Guard immersive canvases behind feature flags consumed by each surface.
**Acceptance criteria**
- Desktop, mobile, and XR builds share the same navigation metadata and interactive widgets.
- Feature-flag toggles enable/disable immersive canvases without regressions in other clients.
**Meta**
- Owner: Agentic Shell Team
- Priority: P1
- Status: Completed
- Depends on: AGENTOS-7

<a id="task-agent-factory"></a>
### AGENTOS-9 — Map Agent Factory hierarchy into project management flows
**Description:** Align Vibe Kanban personas and escalations with the L1–L5 Agent Factory layers so workflows auto-escalate and report health using the constitutional hierarchy.
**Checklist**
- [x] Reference `agents/README.md#5-layer-agent-hierarchy` for personas/escalations.
- [x] Surface L1–L5 health and escalation paths via `ui/vibe-kanban/app/components/AgentFactoryPanel.tsx`.
- [x] Ensure shell navigation + notifications expose Agent Factory telemetry.
**Acceptance criteria**
- Kanban roles expose L1–L5 escalation paths and health indicators inside the shell.
- Agent Factory telemetry (hive mind, swarm, disposable agents) is visible in the shared dashboard.
**Meta**
- Owner: Agentic PM Office
- Priority: P1
- Status: Completed
- Depends on: AGENTOS-8

<a id="task-goal-insights-ga"></a>
### AGENTOS-10 — General-availability goal insights and autonomy
**Description:** Complete the Kanban goal insights/autonomy rollout (shadow → preview → pilot → GA) with telemetry freshness, autonomous retry, and escalation safeguards documented.
**Checklist**
- [x] Follow rollout plan in `docs/plans/kanban_goal_insights_rollout.md`.
- [x] Keep capability toggles in `registry/capabilities.json` + feature flags in `ui/vibe-kanban/app/components/featureFlags.ts`.
- [x] Persist telemetry/logging in `ui/vibe-kanban/app/components/useBoardState.ts` (autonomy notifications + retries).
**Acceptance criteria**
- Goal KPI freshness < 2 minutes, <5% false-positive autonomous retries, and 100% escalations logged in the workspace event stream.
- Documented kill switches (feature flags + capability registry) disable the features without migrations.
**Meta**
- Owner: Agentic PM Office
- Priority: P1
- Status: Completed
- Depends on: AGENTOS-9
<!-- END: PM_ROADMAP -->
