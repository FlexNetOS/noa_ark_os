# Model Relocation Sandbox

This sandbox simulates storage relocation incidents and validates that automated playbooks safely move assets between facilities
while preserving uptime. It mirrors the production relocation playbook structure so telemetry emitted here maps directly to
dashboard and alert expectations.

## Simulation Goals
- Execute relocation playbooks defined in `crc/config/patterns.yaml` and `crc/config/rules.yaml`.
- Validate guardrails including dry-run enforcement, checkpointing, and checksum verification.
- Capture telemetry for recovery KPIs consumed by `.graphs/metrics/automation_effectiveness.yaml`.
- Exercise alert routes specified in `services/monitoring/pipelines/alerts.yaml` to confirm runbook accuracy.

## Recovery Flow Validation
1. Run the `relocation_incident.yaml` scenario to stage assets and trigger an automated relocation.
2. Confirm that rollback checkpoints are created before cut-over.
3. Verify post-move checksum reports and business owner sign-off are recorded.
4. Export incident metadata to `services/monitoring/pipelines/alerts.yaml` for proactive detection.
5. Review generated alerts to ensure acknowledgement SLAs defined in the pipeline manifest are realistic.

## Expected Outputs
- `logs/relocation/` contains relocation run outputs.
- `reports/postmortem.md` generated automatically for executive stakeholders.
- Telemetry events pushed to the monitoring service for dashboards and alerting.
- Annotated checkpoints stored for audit review in the automation ROI report.

## Telemetry Checklist
- [ ] Success rate and checksum metrics exported without gaps.
- [ ] Alert annotations include remediation steps from the ROI runbook.
- [ ] Dashboard feed normalization keeps runtime metrics within the configured jitter tolerance.
