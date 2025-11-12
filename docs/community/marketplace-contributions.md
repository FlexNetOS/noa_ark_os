# Marketplace Contribution Guidelines

The marketplace showcases kernel-vetted applications and agents. Publishers must satisfy the following requirements before their listings become available to customers.

## Publisher Onboarding

1. **Identity Verification**: Submit corporate verification documents and establish a signed publishing key.
2. **Contact Channels**: Provide security and support email aliases, along with an escalation phone number.
3. **Entitlement Mapping**: Declare supported licensing models and align them with workspace entitlements for automated enforcement.

## Manifest Requirements

- Supply a `manifest.yaml` adhering to the schema demonstrated by the example in `apps/marketplace/README.md`.
- Include compatibility matrices for kernel versions, UI targets, and dependent blueprints.
- Reference CRC attestation bundles for every binary, container image, or workflow script.
- Provide localized summaries and highlight bullet points for the UI catalog card.

## Review Workflow

1. **Automated Linting**: `marketctl lint` validates manifest structure, attestation references, and semantic versioning.
2. **Security Scan**: Kernel security council reviews supply chain attestations, SBOMs, and vulnerability reports.
3. **Functional Evaluation**: Marketplace maintainers execute the installation workflow in a sandbox workspace.
4. **Contract Sign-off**: Licensing terms and revenue sharing agreements are finalized prior to publication.

## Post-Publication Responsibilities

- Publish update advisories using semantic versioning; breaking changes require a new major version.
- Maintain a public changelog and deprecation policy accessible from the marketplace listing.
- Respond to critical security findings within 24 hours; failure to do so may trigger forced delisting.
- Coordinate with the analytics team when new capabilities emit metrics, ensuring dashboards remain accurate.

## Submission Process

1. Fork the repository and add your listing under `apps/marketplace/catalog/<slug>` with manifests and assets.
2. Update `apps/marketplace/README.md` if your submission introduces new schema fields or workflow types.
3. Create a pull request referencing marketplace maintainers and attach evidence of compliance with these guidelines.
4. Upon approval, the kernel publishes the listing and triggers catalog sync for all operator workspaces.
