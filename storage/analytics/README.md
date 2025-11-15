# Business Value Analytics

The analytics storage tier aggregates operational signals from CRC pipelines, deployment workflows, and agent swarms to surface actionable business value insights in the unified UI. Metrics are persisted as normalized fact tables that power dashboards and alerting.

## Metric Families

| Metric | Source | Definition | UI Surface |
|--------|--------|------------|------------|
| CRC Throughput | CRC telemetry bus | Mean and percentile task completions per CRC cycle, segmented by blueprint and workspace. | Value stream burn-up chart and CRC health indicators. |
| Deployment Frequency | CI/CD blueprint events | Count of successful production promotions per service over trailing 24h, 7d, 30d windows. | Release velocity widget and executive KPIs. |
| Agent Efficiency | Swarm telemetry (`telemetry:swarm-hivemind`) | Ratio of successful task resolutions to total assignments with mean time-to-resolution. | Swarm mission control efficiency gauge. |
| Notebook Update Cadence | Notebook automation run log | Mean minutes between automated and manual notebook executions, stratified by scope. | Notebook operations insights dashboard cadence panel. |
| Notebook Adoption Coverage | Workspace registry + notebook usage fact | Unique author counts and automation flags per workspace and blueprint. | Notebook operations insights dashboard adoption heatmap. |

## Storage Layout

```
storage/analytics/
  pipelines/
    crc_throughput.parquet
    deployment_frequency.parquet
    agent_efficiency.parquet
    goal_metrics.jsonl
    notebook_update_cadence.parquet
    notebook_adoption.parquet
  models/
    crc_throughput.sql
    deployment_frequency.sql
    agent_efficiency.sql
    notebook_update_cadence.sql
    notebook_adoption.sql
  views/
    ui_value_stream.json
  raw/
    workflow_events.jsonl
    notebook_operations_insights.json
```

- **`pipelines/`**: materialized metric outputs produced by nightly and real-time aggregators (see refresh cadence below).
- **`models/`**: SQL or DSL templates defining aggregations (dbt or kernel-native query syntax).
- **`views/`**: UI composition metadata describing dashboard layouts, thresholds, and drill-down routes.
- **`raw/`**: source workflow and telemetry events consumed by materializers.

## Refresh Cadence

The analytics surfaces are regenerated nightly by the `workflow/cron/analytics_materializer.py` job, fulfilling
[`AGENTOS-3`](../../docs/plans/gap_remediation_tasks.md#task-agentos-3). The materializer reads
`storage/analytics/raw/workflow_events.jsonl`, writes the canonical snapshot to
`storage/db/analytics/goal_kpis.json`, and mirrors the same payload to
`storage/analytics/pipelines/goal_metrics.jsonl` for downstream tools. Operators can re-run the job on-demand via:

```bash
python -m workflow.cron.analytics_materializer
```

When running manually, ensure fresh workflow events are available. Dashboards reference the resulting outputs with a
24-hour SLA; if a refresh fails, update the gap remediation tracker with the root cause and next scheduled run.

## Aggregation Logic

### CRC Throughput
- Join CRC task events with blueprint metadata to compute cycle time distributions.
- Emit percentile bands (P50, P90, P99) and annotate anomalies when throughput deviates from seven-day baseline.

### Deployment Frequency
- Ingest deployment audit logs from CI/CD blueprints (`ci_cd/continuous-assurance`).
- Calculate daily deployment counts per service, smoothing via seven-day moving average.
- Publish delta indicators to highlight acceleration or slowdown trends.

### Agent Efficiency
- Consume swarm telemetry frames capturing assignments, completions, and escalation outcomes.
- Compute efficiency ratio and mean time-to-resolution (MTTR) per swarm, tagging outliers for UI badges.
- Record governance score derived from policy compliance events.

### Notebook Update Cadence
- Track automation-triggered and manual notebook runs from the gateway evidence lake.
- Calculate minutes between successive executions to surface stale notebooks and automation drift.
- Flag scopes where cadence drops below policy thresholds for triage.

### Notebook Adoption Coverage
- Join workspace registry metadata with notebook usage facts to capture author adoption and automation depth.
- Classify workspaces into `inactive`, `manual`, or `automated` adoption modes.
- Provide blueprint-level insights for enablement teams and governance reviews.

## UI Integration

1. The unified UI queries `views/ui_value_stream.json` to build dashboard modules and KPI callouts.
2. `views/notebook_operations_insights.json` composes notebook cadence/adoption visualizations alongside CRC throughput and agent efficiency benchmarks.
3. Drill-down interactions request raw metric slices via the analytics API, enabling cross-filtering by blueprint, workspace, and time window.
4. Alerts trigger when thresholds defined in the view metadata are breached (e.g., CRC throughput P90 drops >15% or cadence_per_hour < 0.05).

## Extensibility

- New metrics should register a model under `models/`, materialized output in `pipelines/`, and documentation updates within this README.
- Analytics producers must emit CRC attestations to maintain provenance and trust.
- Coordinate with the marketplace team so installable apps can extend analytics via plug-in descriptors.
