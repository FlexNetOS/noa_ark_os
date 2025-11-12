# Blueprint Catalog

The blueprint catalog enumerates curated, UI-ready workflow pipelines that operators can deploy with a single click. Each blueprint includes the domain-specific automations, guardrails, and UI affordances required by the Noa Ark kernel to hydrate a runnable flow.

## Blueprint Index

| Blueprint | Category | Description | UI Entry Point |
|-----------|----------|-------------|----------------|
| `ci_cd/continuous-assurance` | CI/CD | Builds, tests, and promotes artifacts across dev, staging, and production environments with progressive delivery gates. | `workflow://blueprints/ci_cd/continuous-assurance` |
| `data/stream-refinery` | Data Processing | Orchestrates ingestion, transformation, quality checks, and publish/subscribe fan-out for mission-critical datasets. | `workflow://blueprints/data/stream-refinery` |
| `agents/swarm-field-kit` | Agent Swarms | Stages multi-agent collectives with shared memory, situational prompts, and guardrails tuned for cooperative tasks. | `workflow://blueprints/agents/swarm-field-kit` |

## Blueprint Details

### CI/CD – `ci_cd/continuous-assurance`
- **Purpose:** Provide enterprise CI/CD with risk-aware promotion, embedded security scans, and rollback automation.
- **Pre-requisites:** Linked source control, container registry credentials, CRC telemetry stream.
- **Kernel Hooks:**
  - `build:containers`: container build templates with supply chain attestation.
  - `deploy:progressive`: orchestrated deployment with automated soak monitoring.
  - `notify:executive-brief`: summarizes release impact for leadership dashboards.
- **UI Surfaces:** Deployment overview tile, release readiness checklist, and artifact diff viewer.

### Data Processing – `data/stream-refinery`
- **Purpose:** Normalize heterogeneous feeds, enforce data contracts, and push curated outputs to real-time consumers.
- **Pre-requisites:** Source connectors, schema registry, observability stream.
- **Kernel Hooks:**
  - `ingest:connectors`: connector pack enabling batch and streaming sources.
  - `quality:contract-enforcer`: configurable validations with auto-remediation recipes.
  - `publish:mesh-topics`: managed topic provisioning with lineage tracking.
- **UI Surfaces:** Data pipeline topology map, quality scorecards, and SLA compliance alerts.

### Agent Swarms – `agents/swarm-field-kit`
- **Purpose:** Bootstrap cross-functional agent swarms capable of collaborative planning, execution, and retrospection.
- **Pre-requisites:** Shared memory volume, swarm governance policy, secure command channel.
- **Kernel Hooks:**
  - `swarm:manifest`: declarative roster with roles, capabilities, and escalation chains.
  - `memory:episodic-sync`: manages synchronized episodic memory segments.
  - `telemetry:swarm-hivemind`: surfaces efficiency and cohesion metrics to analytics bus.
- **UI Surfaces:** Swarm mission control board, agent health indicators, and collaboration timeline.

## UI Deployment Flow
1. The UI lists blueprints via the catalog index and exposes metadata cards with capability tags.
2. Operators click **Deploy** to prefill configuration forms with blueprint defaults retrieved from kernel APIs.
3. Validation checks ensure required integrations (CRC stream, registries, connectors) are satisfied before instantiation.
4. Successful deployment registers the pipeline under the operator's workspace and emits observability hooks for analytics ingestion.

## Extensibility
- Add new blueprints as subdirectories under this folder and register them in the index table above.
- Include a `blueprint.yaml` file alongside implementation assets capturing inputs, dependencies, kernel hooks, and UI affordances.

### `blueprint.yaml` Schema

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
  - <string>                  # List of kernel hook identifiers (see examples above)
ui_affordances:
  - <string>                  # List of UI features or surfaces provided
- Use [`/docs/community/third-party-blueprints.md`](../../docs/community/third-party-blueprints.md) for contribution guidelines and review policies.
