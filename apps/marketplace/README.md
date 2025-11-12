# Kernel Marketplace

The marketplace curates installable applications and autonomous agents that have been reviewed by the Noa Ark kernel security council. Marketplace items expose machine-readable metadata for lifecycle management, licensing, and guided installation from the unified UI.

## Catalog Structure

Marketplace entries are stored under `apps/marketplace/catalog` with the following convention:

```
apps/marketplace/
  catalog/
    <slug>/
      manifest.yaml
      media/
      artifacts/
```

- **`manifest.yaml`**: canonical metadata document (see schema below).
- **`media/`**: product imagery or demo clips referenced by the UI.
- **`artifacts/`**: binaries, container recipes, or workflow snippets retrieved by the installer.

## Metadata Schema

```yaml
id: swarm-researcher
name: Swarm Researcher Toolkit
version: 2.3.1
publisher:
  name: Ark Labs
  contact: builds@ark.labs
licensing:
  model: enterprise-subscription
  price_tiers:
    - tier: starter
      monthly_usd: 499
    - tier: scale
      monthly_usd: 1499
compatibility:
  kernels: ["noa-ark>=1.4"]
  ui_targets: ["desktop-shell", "mobile-shell"]
capabilities:
  - research-orchestration
  - compliance-dossier
security:
  review_status: kernel-vetted
  attestation_ref: crc://artifacts/arklabs/srt/attestations/v2
install:
  prerequisites:
    - "Linked blueprint: agents/swarm-field-kit"
    - "Workspace entitlement: research"
  workflow: kernel-install-script
  steps:
    - id: fetch
      action: pull-artifact
      params:
        source: artifacts/srt/2.3.1.tgz
    - id: verify
      action: verify-attestation
      params:
        attestation: crc://artifacts/arklabs/srt/attestations/v2
    - id: hydrate
      action: deploy-agent
      params:
        kernel_module: swarm_researcher
marketing:
  summary: "Accelerate primary research with guided agent swarms and compliance-ready exports."
  highlights:
    - "Synchronized swarm planning"
    - "Auto-generated executive briefs"
    - "Regulatory-ready audit log"
```

## Licensing Workflows

1. The UI surfaces licensing metadata so operators can choose appropriate tiers before installation.
2. License acceptance events are recorded via the kernel's entitlement service and attached to the workspace ledger.
3. Renewals leverage the same metadata by polling `licensing.model` and `price_tiers` for upcoming expirations.

## Installation Flow

1. Users select an item card and review its `manifest.yaml` in the UI modal.
2. The UI invokes the kernel's marketplace API, which validates compatibility (`kernels`, `ui_targets`) and required blueprints.
3. Installation steps are executed in order, with progress bubbled back through the CRC telemetry bus for real-time visibility.
4. Successful installs register feature flags and extend the analytics pipeline defined in `storage/analytics` to monitor value realization.

## Security and Vetting

- Every submission requires CRC attestations and passes kernel runtime conformance tests.
- Publishers provide update channels; delistings propagate via catalog sync to disable installations automatically.
- The marketplace CLI (`tools/marketctl`, forthcoming) will lint manifests against this schema.

## Contribution Guidance

Refer to `docs/community/marketplace-contributions.md` for submission requirements, review SLAs, and publisher verification steps.
