# Agentic Project Management & Unified UI Roadmap

## Context

NOA ARK OS currently ships two distinct experiences:

- A static **NOA Dashboard** (`ui/noa-dashboard/index.html`) that advertised the “Unified Control Plane” but never advanced beyond hard‑coded cards.
- The **Vibe Kanban workspace** (`ui/vibe-kanban/`) that now contains planner memory, CRC uploads, goal insights and assist logic, yet still runs largely outside the shared shell (`ui/core/src/module.rs`).

The upstream roadmap (`docs/plans/roadmap.md`), Kanban goal insights rollout plan, automation build kits, and the 5‑layer Agent Factory hierarchy call for a Level‑4 Agentic AI system: multi-agent planning with memory, critique, and autonomous retries. This document restates those directives as a consolidated execution plan so we can “rejoin” the dashboard and Kanban worlds into a single cross-platform Agentic workspace.

## Agentic North Star (Level 4)

Borrowing from the Agentic AI reference (Crew-based planning → Memory → Autonomy):

1. **Reason → Act → Observe loop** instrumented by planner & executor roles.
2. **Goal memory & environment layers** stored in SQLite/JSON and surfaced via Kanban assist/analytics.
3. **Hierarchical crews (L1–L5)** mapped to Agent Factory personas so escalations and automation respect governance lines.
4. **Cross-platform delivery** (desktop, web, mobile, XR) through the unified shell.

## Strategic Pillars

| Pillar | Description | Key Deliverables / Task Codes |
|--------|-------------|-------------------------------|
| **Kernel & Deployment Foundations** | Complete AGENTOS-1/2 so every UI build runs on reproducible kernels and the single-host profile is stable. | Dependency graph validation, single-host bootstrap, health dashboard. |
| **Unified Agentic Shell** | Finish AGENTOS-3 and extend it with AGENTOS-7 so the dashboard & Vibe Kanban share navigation, notifications, and persona-aware layout on every surface. | Shared module registry, shell architecture ADR, Kanban module wiring, interactive dashboard tiles. |
| **Agentic Project Management** | Promote Kanban from “AI agent board” to an Agentic AI Layer (planner, memory, executor) via AGENTOS-8/9/10 and the Kanban goal-insights rollout. | Persona-aware workflows, memory-first analytics, autonomous retry/escalation, integration with Agent Factory L1–L5. |
| **Automation & Ops** | Use the automation master plan (AMP-*) to keep the experience push-button: quickstart bootstrap, model registry automation, offline tooling, CRC drop orchestration. | `runa bootstrap --auto`, model registry reconciler, GitHub CLI bundle, archival drop tooling (AMP-BOOT-01..03, AMP-PM-04). |

> **Registry alignment:** All AGENTOS tasks referenced below are mirrored in `build_kits/pm_roadmap.json` and `build_kits/pm_roadmap.csv`. Automation guardrails continue to live in `build_kits/automation_master_plan.*`. This ensures planners, dashboards, and Git tooling consume the same codes regardless of interface.

## Phased Plan

1. **Phase 0 – Foundations**  
   - Execute `AGENTOS-1` and `AGENTOS-2`.  
   - Land AMP-BOOT-01 and AMP-MREG-02 to keep kernels/installations reproducible.
2. **Phase 1 – Shell Resynchronisation**  
   - Complete `AGENTOS-3`.  
   - Deliver `AGENTOS-7` to embed the new Agentic PM shell inside `ui/core` and retire the static dashboard.  
   - Update docs (`docs/ui_feature_review.md`, `docs/ARCHITECTURE.md`) with shared navigation patterns.
3. **Phase 2 – Cross-Platform Agentic UI**  
   - Execute `AGENTOS-8` to package the shell for desktop (Tauri), mobile (Expo), and XR, aligning with persona-aware overlays.  
   - Integrate automation tasks (AMP-GHCLI-03, AMP-PM-04) so every surface can bootstrap offline.
4. **Phase 3 – Agentic PM Autonomy**  
   - Roll out `AGENTOS-9` (Agent Factory alignment) + `AGENTOS-10` (goal insights/autonomy GA).  
   - Follow the Kanban rollout phases: Shadow, Insights Preview, Autonomy Pilot, GA.  
   - Ensure telemetry/memory endpoints (`/api/workspaces/.../assist`, `/api/goals/.../memory`) feed both the board and the shell.
5. **Phase 4 – Continuous Improvement**  
   - Close the loop using automation kits (drop integration docs) plus new backlog items discovered via analytics.  
   - Feed telemetry into `agents/` orchestrators so escalations hit the correct L1–L5 layers.

## Task Inventory

| Code | Theme | Description | Dependencies | Agentic Level |
|------|-------|-------------|--------------|---------------|
| AGENTOS-1..6 | **Existing roadmap tasks** ✳ | Kernel graph, single-host profile, unified shell, adaptive runtime, gateway observability, value-add ecosystem. | — | Level 1–2 foundations. |
| **AGENTOS-7** | Agentic PM shell join | Replace the static dashboard with the Agentic shell module; inject Kanban analytics, planner memory, and CRC uploads into `ui/core`. Includes interactive tiles for workflows, CRC drops, and planner memory. | AGENTOS-3 | Level 3 (multi-agent routing). |
| **AGENTOS-8** | Cross-platform interactive UX | Package unified shell for desktop/mobile/XR; add interactive widgets (workflow launchers, agent critique feed, drag-and-drop CRC drops) that call the FastAPI gateway. | AGENTOS-7 | Level 3/4 delivery. |
| **AGENTOS-9** | Agent Factory alignment | Map Agent Factory L1–L5 layers to Kanban roles (workspace owners, orchestrators, specialists, disposable agents) and surface escalation paths & health indicators in UI. | AGENTOS-8 | Level 4 (hierarchical crews). |
| **AGENTOS-10** | Goal insights & autonomy GA | Execute the rollout plan: gating, telemetry freshness, autonomous retry + escalation. Document kill switches and policy manifests. | AGENTOS-9 | Level 4 (memory + self-healing). |
| AMP-BOOT-01..03 | Automation foundations | Bootstrap automation, registry updates, offline GitHub tooling. | — | Supports all levels. |
| AMP-PM-04 *(new)* | PM automation pipeline | Automate Kanban snapshots, agentic retros, and CRC drop syncing so planners receive AI-generated reports each sprint. | AMP-BOOT-01, AMP-MREG-02 | Enables Level 4 telemetry. |

✳ Existing tasks sourced from `docs/plans/roadmap.md` & `build_kits/pm_roadmap.*`.

## Agent Factory Integration

Using `agents/README.md`:

- **L1/L2** → Governance: tie roadmap decisions and shell feature flags to CECCA/Board approvals.
- **L3** → Stack-Chief orchestrators: run Agentic sprint rituals, coordinate automation flows.
- **L4** → Specialists (Kanban columns/roles) now instrumented with goal insights & autonomy.
- **L5** → Disposable sub-agents: planner’s executor pods triggered by Kanban automation, reporting back via memory APIs.

The roadmap ensures each layer has visibility and control loops through the unified UI so escalations and automation match the Agentic AI definition (planner + memory + critique).

## AGENTOS Completion Evidence

| Code | Delivered work | Evidence / Notes |
|------|----------------|------------------|
| **AGENTOS-0** | Program governance ledger shared across human + agent surfaces. | `build_kits/pm_roadmap.json`/`.csv` and this document now list AGENTOS-0…10 with acceptance notes consumed by `scripts/export_roadmap.ts`. |
| **AGENTOS-1** | Kernel dependency graph + portable bundles. | `runtime/kernel/graph.yaml` validated via `cargo run -p runtime_manager --bin validate_kernel_graph runtime/kernel/graph.yaml`; portable bundles built with `tools/portable_builder.py` for OCI/offline installs. |
| **AGENTOS-2** | Single-host profile bootstrap + health diagnostics. | `docs/deployments/single_host.md`, `services/single-host/init/noa-single-host.sh`, and `server/profiles/single_host/profile.toml` script the end-to-end bring-up and health dashboard checks. |
| **AGENTOS-3** | Unified shell across desktop/web/immersive. | `ui/core/src/module.rs` (lazy modules + navigation) combined with `scripts/dev/full_system_launch.sh` and `server/ui_api/src/schema.rs` hydrate the shared UI shell with session state, notifications, and immersive toggles. |
| **AGENTOS-4** | Adaptive runtime portability. | `runtime/manager/src/lib.rs` (`AdaptiveRuntimeController`) classifies Minimal/Standard/Accelerated hosts while `server/src/adaptive_scaling.rs` enforces telemetry-driven policies surfaced in Prometheus and notes. |
| **AGENTOS-5** | Gateway zero-trust + observability mesh. | `server/gateway/src/auth.rs`, `policy.rs`, and `telemetry.rs` wire pluggable authz modules, routing policy compiler, and event/metrics writers under `storage/telemetry/`. |
| **AGENTOS-6** | Value-add automation/analytics packs. | `services/marketplace` bundle builder plus `apps/marketplace/catalog/*.json` publish curated packs with installer metadata and telemetry-driven onboarding. |
| **AGENTOS-7** | Vibe Kanban embedded into Agentic shell. | `ui/vibe-kanban` widgets (PlannerPanel, UploadPanel, AssistPanel, AutomationPanel) are registered via `server/ui_api/src/schema.rs` so Kanban analytics, CRC uploads, workflow launchers, and memory render inside `ui/core`. |
| **AGENTOS-8** | Cross-platform Agentic UI packages. | `ui/shared/src/schema.ts` + `ui/shared/src/samples/vibe-dashboard.ts` feed the same schema into `apps/desktop-shell`, `apps/mobile-shell`, and `apps/xr-shell` so interactive widgets stay in lockstep across Tauri/Expo/XR builds. |
| **AGENTOS-9** | Agent Factory hierarchy surfaced in PM flows. | `ui/vibe-kanban/app/components/AgentFactoryPanel.tsx` maps L1–L5 personas from `agents/README.md#5-layer-agent-hierarchy` into the workspace, exposing escalation paths and health indicators through the shell. |
| **AGENTOS-10** | Goal insights/autonomy GA. | `docs/plans/kanban_goal_insights_rollout.md`, `registry/capabilities.json`, and `ui/vibe-kanban/app/components/useBoardState.ts`+`featureFlags.ts` document telemetry freshness, autonomous retry, and kill-switch controls for GA. |

## Value Improvements

- **Cross-platform parity** ensures every collaborator (desktop, web, mobile, XR) sees identical Agentic workflows and telemetry.
- **Agentic PM** converts the Kanban board from “AI agent tasks” to a Level‑4 system with planner memory, escalation, and automation.
- **Automation kits** keep environments reproducible, enforce cadences, and backstop remote/offline deployments.
- **Documentation cadence** (`docs/roadmaps/`, rollout plans, drop manifests) guarantees both humans and agents execute from the same playbook.

## Verification Commands

- `cargo fmt --all && cargo test --workspace`
- `cargo run -p runtime_manager --bin validate_kernel_graph runtime/kernel/graph.yaml`
- `pnpm --filter vibe-kanban lint && pnpm --filter vibe-kanban test`
- `python tools/portable_builder.py --profile single_host` (portable bundle smoke)
- `scripts/dev/full_system_launch.sh --verify` (shell bootstrap)
