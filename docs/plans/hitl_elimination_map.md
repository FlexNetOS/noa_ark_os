# HITL Elimination Map

This map tracks each manual control, the automation task that retires it, and any cascading dependencies across subsystems.

| HITL ID | Manual Step | Automation Task | Notes |
| --- | --- | --- | --- |
| HITL-QS-01 | Manual execution of Quickstart bootstrapper | [AMP-BOOT-01](automation_master_plan.md#amp-boot-quickstart) | Depends on provisioning service reliability improvements landed in the bootstrap orchestrator. |
| HITL-MR-02 | Manual curation of model metadata for registry promotion | [AMP-MREG-02](automation_master_plan.md#amp-mreg-updater) | Consumes telemetry emitted by the automated bootstrap to validate environment readiness before publishing. |
| HITL-DEV-03 | Manual offline GitHub CLI setup | [AMP-GHCLI-03](automation_master_plan.md#amp-ghcli-offline) | Requires access to credential seeding pipeline output packaged with the bootstrap artifacts. |

All entries now reference automation anchors directly, maintaining the one-to-one traceability needed for HITL elimination reporting.
