# Kanban Goal Insights & Autonomy Rollout

## Overview
The goal insights capability introduces telemetry-driven KPIs (lead time, success rate) and autonomous guardrails for the kanban workspace. This plan outlines the phased rollout, success measures, and fallback controls for the new capability tokens `kanban.goalInsights`, `kanban.autonomousRetry`, and `kanban.agentEscalation`.

## Capability Matrix
| Capability Token | Feature Flag | Description | Default |
| ---------------- | ------------ | ----------- | ------- |
| `kanban.goalInsights` | `goalInsights` | Exposes goal-level KPIs inside analytics widgets and board headers. | Disabled for starter tenants, enabled for growth+ in staged rollout. |
| `kanban.autonomousRetry` | `autonomousRetry` | Triggers autonomous re-planning when success-rate KPIs dip under threshold. | Pilot cohort only. |
| `kanban.agentEscalation` | `agentEscalation` | Escalates goals to senior agents when lead time thresholds breach. | Pilot cohort only. |

## Phased Rollout
1. **Phase 0 – Shadow Mode**
   * Keep capability tokens disabled; ingest telemetry only.
   * Validate goal metric persistence and analytics ingestion end-to-end.
2. **Phase 1 – Insights Preview**
   * Enable `goalInsights` for internal workspaces via registry updates.
   * Monitor lead-time variance and ensure UI gracefully hides KPIs when empty.
3. **Phase 2 – Autonomy Pilot**
   * Allow `autonomousRetry` and `agentEscalation` for selected tenants.
   * Configure thresholds through workspace policy manifests and review auto-trigger frequency daily.
4. **Phase 3 – General Availability**
   * Gradually enable tokens in the public registry based on adoption telemetry.
   * Update documentation and training materials, highlighting opt-out mechanisms via capability registry.

## Success Metrics
* Goal KPI freshness < 2 minutes from workflow completion.
* <5% autonomous retrigger false positives week-over-week.
* 100% of escalations include notifications in workspace event stream.

## Rollback & Safeguards
* Disable feature flags in `featureFlags.ts` for immediate kill-switch.
* Remove capability tokens from `registry/capabilities.json` to suppress UI surfaces.
* Persisted analytics remain read-only; no destructive migrations required.

## Verification Checklist
- [x] Unit tests covering capability gating and analytics visibility.
- [x] Feature flags default to safe values.
- [x] Documentation synced with capability registry definitions.
- [x] Workflow instrumentation exports KPI snapshots to `storage/db/analytics/goal_kpis.json`.
