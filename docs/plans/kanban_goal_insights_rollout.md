# Kanban Goal Insights & Autonomy Rollout

## Overview
The goal insights capability introduces telemetry-driven KPIs (lead time, success rate) and autonomous guardrails for the kanban workspace. This plan outlines the phased rollout, success measures, and fallback controls for the new capability tokens `kanban.goalInsights`, `kanban.autonomousRetry`, and `kanban.agentEscalation`.

## Capability Matrix
| Capability Token | Feature Flag | Description | Default |
| ---------------- | ------------ | ----------- | ------- |
| `kanban.goalInsights` | `goalInsights` | Exposes goal-level KPIs inside analytics widgets and board headers. | Disabled for starter tenants, enabled for growth+ in staged rollout. |
| `kanban.autonomousRetry` | `autonomousRetry` | Triggers autonomous re-planning when success-rate KPIs dip under threshold. | Pilot cohort only. |
| `kanban.agentEscalation` | `agentEscalation` | Escalates goals to senior agents when lead time thresholds breach. | Pilot cohort only. |

## Phased Rollout *(completed – May 2024)*
1. **Phase 0 – Shadow Mode**
   * ✅ Capability tokens disabled while telemetry streamed into `storage/db/analytics/goal_kpis.json`.
   * ✅ Validated ingestion via `ui/vibe-kanban/app/components/useBoardState.ts` + regression tests.
2. **Phase 1 – Insights Preview**
   * ✅ `registry/capabilities.json` exposes `ui.kanban.goal_insights`; feature flag defaults tracked in `featureFlags.ts`.
   * ✅ UI hides KPIs gracefully when internal workspaces lack data (see analytics panel tests).
3. **Phase 2 – Autonomy Pilot**
   * ✅ `kanban.autonomousRetry` + `kanban.agentEscalation` granted to pilot tenants; thresholds configurable per workspace manifest.
   * ✅ Daily review + notification logging implemented in `useBoardState.ts` (autonomy summary + warnings).
4. **Phase 3 – General Availability**
   * ✅ Capability registry + `featureFlags.ts` enable GA for goal insights/autonomy.
   * ✅ Docs (`docs/roadmaps/agentic_pm_unification.md` & this plan) detail opt-out/kill-switch mechanics.

## Success Metrics
- Goal KPI freshness < 2 minutes from workflow completion (telemetry timestamp comparison in `goalEvaluationRef` logic).
- <5% autonomous retrigger false positives week-over-week (enforced by `autonomousRetry` gating + logging).
- 100% of escalations include notifications in workspace event stream (`notifications` array + CRC receipts).

## Rollback & Safeguards
* Disable feature flags in `featureFlags.ts` for immediate kill-switch.
* Remove capability tokens from `registry/capabilities.json` to suppress UI surfaces.
* Persisted analytics remain read-only; no destructive migrations required.

## Current Status
- **Freshness:** Board metrics + goal insights refreshed on every assist/board poll; average freshness ~60 seconds on nominal hardware.
- **Autonomy guardrails:** `useBoardState.ts` only triggers retries when `goalSuccessRate < 60` and escalations when `goalLeadTimeHours > 12`, emitting notifications + telemetry.
- **Event stream coverage:** CRC uploads + autonomy events append to `notifications` (persisted to workspace data) to satisfy 100% logging requirement.

## Kill Switches
- Feature flags: toggle `goalInsights`, `autonomousRetry`, `agentEscalation` in `ui/vibe-kanban/app/components/featureFlags.ts`.
- Capability registry: revoke `ui.kanban.goal_insights`/`ui.kanban.autonomy` entries inside `registry/capabilities.json`.
- Runtime policies: disable autonomy via workspace manifests or `useBoardState` environment variables without schema migrations.

## Verification Checklist
- [x] Unit tests covering capability gating and analytics visibility.
- [x] Feature flags default to safe values.
- [x] Documentation synced with capability registry definitions.
- [x] Workflow instrumentation exports KPI snapshots to `storage/db/analytics/goal_kpis.json`.
