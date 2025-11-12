# Business Value Analytics

The analytics storage tier aggregates operational signals from CRC pipelines, deployment workflows, and agent swarms to surface actionable business value insights in the unified UI. Metrics are persisted as normalized fact tables that power dashboards and alerting.

## Metric Families

| Metric | Source | Definition | UI Surface |
|--------|--------|------------|------------|
| CRC Throughput | CRC telemetry bus | Mean and percentile task completions per CRC cycle, segmented by blueprint and workspace. | Value stream burn-up chart and CRC health indicators. |
| Deployment Frequency | CI/CD blueprint events | Count of successful production promotions per service over trailing 24h, 7d, 30d windows. | Release velocity widget and executive KPIs. |
| Agent Efficiency | Swarm telemetry (`telemetry:swarm-hivemind`) | Ratio of successful task resolutions to total assignments with mean time-to-resolution. | Swarm mission control efficiency gauge. |

## Storage Layout

```
storage/analytics/
  pipelines/
    crc_throughput.parquet
    deployment_frequency.parquet
    agent_efficiency.parquet
  models/
    crc_throughput.sql
    deployment_frequency.sql
    agent_efficiency.sql
  views/
    ui_value_stream.json
```

- **`pipelines/`**: materialized metric outputs produced by nightly and real-time aggregators.
- **`models/`**: SQL or DSL templates defining aggregations (dbt or kernel-native query syntax).
- **`views/`**: UI composition metadata describing dashboard layouts, thresholds, and drill-down routes.

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

## UI Integration

1. The unified UI queries `views/ui_value_stream.json` to build dashboard modules and KPI callouts.
2. Drill-down interactions request raw metric slices via the analytics API, enabling cross-filtering by blueprint, workspace, and time window.
3. Alerts trigger when thresholds defined in the view metadata are breached (e.g., CRC throughput P90 drops >15%).

## Extensibility

- New metrics should register a model under `models/`, materialized output in `pipelines/`, and documentation updates within this README.
- Analytics producers must emit CRC attestations to maintain provenance and trust.
- Coordinate with the marketplace team so installable apps can extend analytics via plug-in descriptors.
