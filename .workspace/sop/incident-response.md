# Incident Response SOP

## Purpose
Coordinate technical, documentation, and communication workflows during incidents.

## Severity Levels
- **SEV0**: Production outage affecting all tenants.
- **SEV1**: Major functionality loss with workaround available.
- **SEV2**: Degraded experience, limited blast radius.

## Activation
- Automated alert from monitoring stack.
- Manual escalation from operations team.

## Response Flow
1. Initiate incident bridge and assign roles (Incident Commander, Documentation Liaison, Comms).
2. Documentation Liaison launches the documentation agent in recovery mode.
3. Collect workflow states from `workflow/incident-*` definitions.
4. Update `docs/wiki/incidents/<incident-id>.md` with timeline and actions.
5. Verify runbook steps in `docs/runbook/incident.md` and log results.
6. Coordinate recovery tasks with runtime services and track status in `.workspace/sop/incident-response.md`.
7. Close incident after verification checklist passes and retrospective is scheduled.

## Post-Incident Requirements
- Publish final summary to `docs/reports/incident-digest.md`.
- Attach remediation tasks to backlog.
- Ensure documentation agent sync completes successfully.
