# Monitoring Service

This service ingests telemetry emitted by CRC automation sandboxes and renders dashboards plus alerting pipelines. The
configuration is declarative so the same manifests can be replayed in staging sandboxes before production promotion.

## Data Sources
- Storage relocation sandbox metrics emitted from `crc/sandboxes/model-relocation` scenarios.
- Documentation automation metrics emitted from `crc/sandboxes/model-documentation` scenarios.
- ROI metrics compiled in `.graphs/metrics/automation_effectiveness.yaml`.
- Executive report extracts published in `docs/reports/AUTOMATION_EFFECTIVENESS_ROI.md`.

## Pipelines
Pipelines are defined in `pipelines/` and describe how telemetry is transformed into alerts, routed to notification channels, and persisted for reporting. Each
pipeline declaration specifies ownership metadata, runbook sections, and service-level acknowledgement targets so on-call responders have
clear expectations when automation signals regressions.

## Quality Checks
- Alerts include descriptive annotations that link to remediation guidance in the ROI report runbook.
- Dashboard feeds enforce data quality gates (`quality_gate` transformation) before surfacing metrics to leadership views.
- All manifests include version headers and owners to simplify governance reviews.
