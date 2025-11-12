# Automation Master Plan

## Overview
This plan drives the removal of remaining human-in-the-loop (HITL) touchpoints from the automation bootstrapper, model lifecycle management, and developer tooling flows. Each HITL dependency is mapped to an automation task with an explicit anchor so downstream documents and dashboards can link directly to the remediation work.

## HITL Inventory & Coverage
| HITL ID | Manual Touchpoint | Target Subsystem | Suggested Automation |
| --- | --- | --- | --- |
| HITL-QS-01 | Operators run the Quickstart bootstrap sequence manually and stitch logs across hosts. | Quickstart bootstrapper | [AMP-BOOT-01](#amp-boot-quickstart) |
| HITL-MR-02 | Release managers curate model metadata updates by hand before publishing. | Model registry | [AMP-MREG-02](#amp-mreg-updater) |
| HITL-DEV-03 | Field engineers install GitHub CLI by hand on air-gapped deployments. | Developer toolchain | [AMP-GHCLI-03](#amp-ghcli-offline) |

All HITL entries above resolve to the automation anchors referenced in the Suggested Tasks & Links section. With those mappings in place, this release plan maintains 100% HITL coverage: no manual control remains without an associated automation task and acceptance criteria trace.

## Suggested Tasks & Links
- [AMP-BOOT-01 · Quickstart bootstrap automation](#amp-boot-quickstart)
- [AMP-MREG-02 · Model registry updater](#amp-mreg-updater)
- [AMP-GHCLI-03 · Offline GitHub CLI installer](#amp-ghcli-offline)

### AMP-BOOT-01 · Quickstart bootstrap automation
<a id="amp-boot-quickstart"></a>
**Description**
Automate the Quickstart bootstrap pipeline so a single orchestrator script provisions infrastructure, seeds configuration, and emits machine-readable status artifacts without human intervention.

**Checklist**
- [ ] Capture current bootstrap flow and convert each manual gate into an orchestrated action with retries.
- [ ] Emit structured telemetry (JSONL) for bootstrap phases so CI can parse health signals.
- [ ] Package the orchestrator as a containerized task with versioned manifests.

**Acceptance criteria**
- A clean-room host reaches operational readiness using the orchestrator with no manual steps.
- Observability dashboards ingest the emitted telemetry and render pass/fail per bootstrap phase.

**Meta**
- Subsystem: Quickstart bootstrapper
- Dependencies: Core provisioning service, runtime configuration manager

### AMP-MREG-02 · Model registry updater
<a id="amp-mreg-updater"></a>
**Description**
Build a scheduled updater that reconciles model metadata from source repositories into the registry, validating schema drift and broadcasting change events automatically.

**Checklist**
- [ ] Implement diff-and-merge logic for model cards, configuration, and lineage manifests.
- [ ] Integrate schema validation and notify maintainers when mismatches occur.
- [ ] Publish change events to the automation bus for downstream consumption.

**Acceptance criteria**
- Registry updates run on schedule without human intervention and pass schema validation.
- Consumers receive change events within one minute of a new model landing in the registry.

**Meta**
- Subsystem: Model registry services
- Dependencies: [AMP-BOOT-01](#amp-boot-quickstart), metadata schema validator, event bus

### AMP-GHCLI-03 · Offline GitHub CLI installer
<a id="amp-ghcli-offline"></a>
**Description**
Provide an offline-capable GitHub CLI (gh) installer bundle with checksum verification and automated profile configuration for air-gapped deployments.

**Checklist**
- [ ] Build reproducible archives for supported platforms with signed checksums.
- [ ] Create an installation harness that configures authentication workflows without internet access.
- [ ] Document automated diagnostics that verify the CLI can issue commands against the mirrored gateway.

**Acceptance criteria**
- Air-gapped installation completes via the harness without hitting external networks.
- Diagnostics confirm gh can authenticate and execute required commands against the gateway mirror.

**Meta**
- Subsystem: Developer toolchain resilience
- Dependencies: [AMP-BOOT-01](#amp-boot-quickstart), credential seeding service, internal package mirror
