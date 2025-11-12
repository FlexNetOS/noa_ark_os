# Third-Party Blueprint Contributions

External partners can publish blueprint automations that extend Noa Ark workflows. Follow these standards to ensure interoperability, security, and a smooth review process.

## Submission Checklist

1. **Repository Layout**
   - Provide a public Git repository or artifact bundle containing blueprint assets.
   - Include `blueprint.yaml` detailing inputs, dependencies, kernel hooks, and UI affordances (see [Blueprint Schema](#blueprint-schema) below).
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

## Review Process

- **Intake:** Submit a merge request adding your blueprint entry under `workflow/blueprints/<slug>` with manifest and assets.
- **Automated Checks:** Kernel CI validates manifest schema, test evidence, security scans, and UI descriptors.
- **Human Review:** Workflow maintainers verify design, telemetry quality, and compliance posture within five business days.
- **Publication:** Approved blueprints receive a kernel-signed catalog entry and are surfaced in the UI with the `Third-Party` badge.

## Maintenance Expectations

- Keep manifests and documentation current with each release; version mismatches trigger catalog quarantine.
- Respond to security advisories within 48 hours; unresolved issues may result in temporary suspension.
- Coordinate with the analytics team to map any new metrics into `storage/analytics` to preserve dashboard continuity.

## Blueprint Schema

Each blueprint **must** include a `blueprint.yaml` file in its root directory that describes its metadata, configuration, and integration points. This manifest enables the Noa Ark kernel to validate, instantiate, and integrate your blueprint into the workflow ecosystem.

### Required Fields

```yaml
name: <string>                # Unique identifier (e.g., "monitoring/observability-stack")
display_name: <string>        # Human-readable name shown in UI
description: <string>         # Brief summary (1-2 sentences) of blueprint purpose
category: <string>            # Category: "CI/CD", "Data Processing", "Agent Swarms", 
                              # "Monitoring", "Security", or "Integration"
version: <string>             # Semantic version (e.g., "1.0.0")
```

### Optional Fields

```yaml
authors:                      # List of blueprint authors
  - name: <string>            # Author name
    contact: <string>         # (optional) Email or handle

inputs:                       # Configuration parameters
  - key: <string>             # Parameter name (e.g., "api_endpoint")
    type: <string>            # Data type: "string", "int", "bool", "enum", "list"
    required: <bool>          # Whether this input is mandatory
    default: <any>            # (optional) Default value
    description: <string>     # User-facing description
    enum:                     # (optional) Valid values for enum types
      - <string>

dependencies:                 # External system requirements
  - <string>                  # System name (e.g., "container-registry", "kafka-cluster")

kernel_hooks:                 # Kernel extension points
  - <string>                  # Hook identifier (e.g., "build:containers", "deploy:progressive")

ui_affordances:               # UI components provided
  - <string>                  # UI surface (e.g., "dashboard:deployment-overview", 
                              # "form:configuration", "tile:status")
```

### Example: CI/CD Blueprint

```yaml
# blueprint.yaml for ci_cd/continuous-assurance
name: "ci_cd/continuous-assurance"
display_name: "Continuous Assurance Pipeline"
description: "Enterprise CI/CD with risk-aware promotion, security scans, and rollback automation"
category: "CI/CD"
version: "1.2.0"

authors:
  - name: "Noa Ark Platform Team"
    contact: "platform@noaark.io"

inputs:
  - key: "source_repo_url"
    type: "string"
    required: true
    description: "Git repository URL for source code"
  
  - key: "container_registry"
    type: "string"
    required: true
    description: "Container registry endpoint (e.g., 'ghcr.io/org')"
  
  - key: "deployment_strategy"
    type: "enum"
    required: false
    default: "progressive"
    description: "Deployment rollout strategy"
    enum:
      - "progressive"
      - "blue-green"
      - "canary"
  
  - key: "enable_security_scan"
    type: "bool"
    required: false
    default: true
    description: "Run security vulnerability scans during build"

dependencies:
  - "source-control"
  - "container-registry"
  - "crc-telemetry-stream"
  - "notification-service"

kernel_hooks:
  - "build:containers"
  - "deploy:progressive"
  - "notify:executive-brief"
  - "rollback:automated"

ui_affordances:
  - "dashboard:deployment-overview"
  - "form:pipeline-configuration"
  - "tile:release-status"
  - "viewer:artifact-diff"
```

### Example: Agent Swarm Blueprint

```yaml
# blueprint.yaml for agents/swarm-field-kit
name: "agents/swarm-field-kit"
display_name: "Multi-Agent Swarm Kit"
description: "Bootstrap collaborative agent swarms with shared memory and coordinated execution"
category: "Agent Swarms"
version: "2.0.1"

authors:
  - name: "Agent Systems Research"
    contact: "agents@noaark.io"

inputs:
  - key: "swarm_size"
    type: "int"
    required: true
    description: "Number of agents in the swarm (2-50)"
  
  - key: "memory_strategy"
    type: "enum"
    required: false
    default: "episodic-sync"
    description: "Shared memory synchronization approach"
    enum:
      - "episodic-sync"
      - "distributed-cache"
      - "event-sourcing"
  
  - key: "agent_roles"
    type: "list"
    required: true
    description: "List of agent role identifiers"

dependencies:
  - "shared-memory-volume"
  - "swarm-governance-policy"
  - "secure-command-channel"

kernel_hooks:
  - "swarm:manifest"
  - "memory:episodic-sync"
  - "telemetry:swarm-hivemind"
  - "coordination:task-distribution"

ui_affordances:
  - "dashboard:swarm-mission-control"
  - "indicator:agent-health"
  - "timeline:collaboration-events"
  - "form:swarm-configuration"
```

### Validation

The kernel CI pipeline automatically validates your `blueprint.yaml` against this schema during the review process. Common validation errors include:

- **Missing required fields**: Ensure `name`, `display_name`, `description`, `category`, and `version` are present
- **Invalid category**: Must be one of the defined categories
- **Malformed version**: Must follow semantic versioning (MAJOR.MINOR.PATCH)
- **Unknown kernel hooks**: Reference only documented kernel extension points
- **Type mismatches**: Input types must match supported values

For the complete list of available kernel hooks and UI affordances, see the [Blueprint Catalog](../../workflow/blueprints/README.md).
