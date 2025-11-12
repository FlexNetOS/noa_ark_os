# Third-Party Blueprint Contributions

External partners can publish blueprint automations that extend Noa Ark workflows. Follow these standards to ensure interoperability, security, and a smooth review process.

## Submission Checklist

1. **Repository Layout**
   - Provide a public Git repository or artifact bundle containing blueprint assets.
   - Include `blueprint.yaml` detailing inputs, dependencies, kernel hooks, and UI affordances (see schema below).
   - Supply documentation (`README.md`) with deployment diagrams and configuration steps.
2. **Operational Readiness**
   - Demonstrate compatibility with the latest LTS kernel release (`noa-ark>=1.4`).
   - Provide automated tests or simulation recordings validating critical execution paths.
   - Attach CRC telemetry samples illustrating baseline throughput and anomaly handling.
3. **Security & Compliance**
   - Generate CRC attestations for every artifact and attach provenance references.
   - Document secret management expectations and least-privilege policies.
   - Declare any third-party services or data transfers involved.
4. **UI Integration**
   - Register required dashboards, forms, and status tiles under `ui.affordances` in `blueprint.yaml`.
   - Supply localization-ready copy for user-facing text.
   - Provide sample screenshots or motion clips for catalog previews.

## `blueprint.yaml` Schema

Each blueprint **must** include a `blueprint.yaml` file describing its metadata, configuration, and integration points. The expected schema is as follows:

```yaml
# blueprint.yaml
name: <string>                # Unique identifier for the blueprint (e.g., "ci_cd/continuous-assurance")
display_name: <string>        # Human-readable name for UI display
description: <string>         # Brief summary of the blueprint's purpose
category: <string>            # Category (e.g., "CI/CD", "Data Processing", "Agent Swarms")
version: <string>             # Semantic version (e.g., "1.0.0")
authors:
  - name: <string>
    contact: <string>         # (optional) Email or handle
inputs:
  - key: <string>             # Input parameter name
    type: <string>            # Data type (e.g., string, int, bool, enum)
    required: <bool>
    description: <string>
dependencies:
  - <string>                  # List of required external systems or services
kernel_hooks:
  - <string>                  # List of kernel hook identifiers (see examples in workflow/blueprints/README.md)
ui_affordances:
  - <string>                  # List of UI features or surfaces provided
```

For complete examples, refer to the [Blueprint Catalog](../../workflow/blueprints/README.md).

## Review Process

- **Intake:** Submit a merge request adding your blueprint entry under `workflow/blueprints/<slug>` with manifest and assets.
- **Automated Checks:** Kernel CI validates manifest schema, test evidence, security scans, and UI descriptors.
- **Human Review:** Workflow maintainers verify design, telemetry quality, and compliance posture within five business days.
- **Publication:** Approved blueprints receive a kernel-signed catalog entry and are surfaced in the UI with the `Third-Party` badge.

## Maintenance Expectations

- Keep manifests and documentation current with each release; version mismatches trigger catalog quarantine.
- Respond to security advisories within 48 hours; unresolved issues may result in temporary suspension.
- Coordinate with the analytics team to map any new metrics into `storage/analytics` to preserve dashboard continuity.
