# Gap Remediation Task List

This document tracks actionable tasks to resolve outstanding gaps and quality issues identified during verification of the NOA ARK OS roadmap implementation.

## Task Overview

- **AGENTOS-1 — Populate Workflow Blueprint Catalog** ([View task](#task-agentos-1))
- **AGENTOS-2 — Deliver Marketplace Assets & Tooling** ([View task](#task-agentos-2))
- **AGENTOS-3 — Implement Analytics Pipelines** ([View task](#task-agentos-3))
- **AGENTOS-4 — Seed Telemetry Storage Samples** ([View task](#task-agentos-4))
- **AGENTOS-5 — Fix CICD Pipeline Struct Duplication** ([View task](#task-agentos-5))
- **AGENTOS-6 — Extend GPU Detection Beyond NVIDIA** ([View task](#task-agentos-6))

---

## 1. Populate Workflow Blueprint Catalog

**Objective:** Stand up a curated library of reusable workflow definitions that contributors can extend without duplicating schemas or discovery logic.

**Success criteria:**
- Blueprint manifests pass automated schema validation and dry-run execution tests.
- New blueprints appear in the UI search and filtering experience without manual intervention.

**Dependencies:** Workflow manifest schema decision (tracked in `workflow/adr/0001-workflow-schema.md`), availability of workflow engine dry-run mode.

**Milestones & tasks:**
- [ ] **Catalog foundation**
  - [ ] Create the `workflow/blueprints` directory with subfolders per blueprint category (e.g., `ci_cd/`, `data_ops/`, `agent_swarms/`).
  - [ ] Publish a top-level `README.md` describing folder layout, naming conventions, and contribution workflow.
- [ ] **Schema enablement**
  - [ ] Provide JSON Schema definitions under `workflow/blueprints/schemas` covering required fields (inputs, outputs, triggers) and optional metadata blocks.
  - [ ] Add a schema versioning policy and CHANGELOG to communicate breaking versus additive updates.
- [ ] **Reference assets**
  - [ ] Land at least three sample blueprints (CI/CD pipeline, data pipeline, agent swarm) annotated with prerequisites, resource requirements, and rollback paths.
  - [ ] Include validation fixtures for each blueprint to support dry-run execution.
- [ ] **Runtime integration**
  - [ ] Wire blueprint discovery into UI/server workflow loaders with hot-reload for local development.
  - [ ] Extend workflow search indexing to ingest blueprint metadata (title, tags, capability tier) for UI filtering.
- [ ] **Quality gates**
  - [ ] Add tests that validate manifests against JSON Schema and execute dry-run workflows end-to-end.
  - [ ] Configure CI to block merges on schema validation or discovery regressions.

**Risks & mitigation:** Schema churn blocked on ADR closure → escalate during architecture sync; discovery hooks rely on runtime feature flag → add temporary fallback to file polling.

---

## 2. Deliver Marketplace Assets & Tooling

**Objective:** Provide a discoverable marketplace catalog and CLI tooling so partners can package, validate, and publish assets safely.

**Success criteria:**
- `marketctl` packages and validates assets end-to-end with CI automation.
- At least five curated listings are available with documentation and smoke tests.

**Dependencies:** Artifact storage bucket configuration, signing keys for publication, CLI distribution channel (Homebrew/Chocolatey).

**Milestones & tasks:**
- [ ] **Catalog foundation**
  - [ ] Establish `apps/marketplace/catalog` structure and seed it with representative listings (agents, workflows, datasets).
  - [ ] Provide skeleton `README.md` templates outlining submission metadata, review expectations, and quality bars.
- [ ] **Tooling implementation**
  - [ ] Implement `tools/marketctl/` CLI with `package`, `validate`, `publish`, and `inspect` commands.
  - [ ] Integrate manifest linting (schema validation, checksum verification), dependency scanning, and sandbox execution checks.
- [ ] **Sample inventory**
  - [ ] Publish ≥5 example listings including licensing metadata, kernel compatibility notes, dependency requirements, and screenshots/placeholders.
  - [ ] Author automated smoke tests that install sample listings into disposable workspaces using `marketctl`.
- [ ] **Operational playbooks**
  - [ ] Document submission, review, and approval workflows in `docs/community/marketplace_submission.md` with SLAs and reviewer roles.
  - [ ] Add contributor lifecycle guidelines (maintenance expectations, deprecation/removal policies, security escalation path).

**Risks & mitigation:** Publishing pipeline requires credentials → gate `publish` command behind feature flag until secrets management story lands; sample screenshots may require design assets → partner with design team for placeholders.

---

## 3. Implement Analytics Pipelines

**Objective:** Capture core CRC, deployment, and agent efficiency metrics with automated ingestion and dashboard surfacing.

**Success criteria:**
- Nightly analytics CI run passes using fixture data.
- Dashboards render aggregated metrics with documented refresh cadences.

**Dependencies:** Data access policies for CRC logs, orchestration environment (Temporal/Dagster) availability, Grafana/Metabase instances.

**Milestones & tasks:**
- [ ] **Repository scaffolding**
  - [ ] Create `storage/analytics/pipelines`, `models`, and `views` directories with interface documentation and README.
  - [ ] Define versioned configuration format (`analytics.yaml`) and loader utilities under `storage/analytics`.
- [ ] **Ingestion jobs**
  - [ ] Implement jobs aggregating CRC throughput, deployment frequency, and agent efficiency metrics by environment.
  - [ ] Schedule jobs via orchestrator (Temporal/Dagster) with retry policies, alerting hooks, and observability metrics.
- [ ] **Dashboard consumers**
  - [ ] Provide Grafana/Metabase dashboards under `docs/analytics/dashboards`, referencing required data sources.
  - [ ] Document data refresh cadence, retention policies, and access requirements.
- [ ] **Validation**
  - [ ] Add integration tests executing pipelines against fixture data to verify metric correctness and schema compatibility.
  - [ ] Wire analytics pipelines into nightly CI to catch regressions and data availability issues.

**Risks & mitigation:** Access to production telemetry may lag → bootstrap with anonymized fixture dumps; orchestrator capacity constraints → coordinate with platform team for worker allocation.

---

## 4. Seed Telemetry Storage Samples

**Objective:** Provide realistic telemetry datasets and automation so teams can validate observability integrations quickly.

**Success criteria:**
- Local bootstrap scripts can load sample telemetry via opt-in flag.
- Documentation points to datasets and import workflows for major tooling.

**Dependencies:** Finalization of telemetry retention policies, sign-off from privacy/legal on anonymized artifacts.

**Milestones & tasks:**
- [ ] **Sample datasets**
  - [ ] Add representative OpenTelemetry traces, metrics, and logs under `storage/telemetry` with README coverage of capture context, anonymization, and intended use.
  - [ ] Provide both success and failure scenarios to support troubleshooting drills.
- [ ] **Documentation**
  - [ ] Update gateway observability documentation referencing sample datasets, validation steps, and expected visualizations.
  - [ ] Publish quick-start guides for importing samples into Jaeger, Prometheus, and Grafana.
- [ ] **Automation**
  - [ ] Integrate sample loading into `scripts/bootstrap_dev_env.sh` (opt-in flag, idempotent operations).
  - [ ] Add CI checks ensuring telemetry samples track schema changes and anonymization guarantees.

**Risks & mitigation:** Sample data may drift from production schema → schedule quarterly refresh; privacy review backlog → prep anonymization summary for legal early.

---

## 5. Fix CICD Pipeline Struct Duplication

**Objective:** Remove redundant struct definitions in the CICD crate and formalize serialization contracts.

**Success criteria:**
- `cargo test`, `cargo fmt`, and `cargo clippy -- -D warnings` pass for the CICD crate.
- Consumers can deserialize both legacy and new pipeline manifests without data loss.

**Dependencies:** Agreement on canonical pipeline schema, regression fixtures from downstream users.

**Milestones & tasks:**
- [ ] **Refactor implementation**
  - [ ] Consolidate duplicate fields in `cicd/src/lib.rs` and document canonical `Pipeline`, `Stage`, and `Job` structs.
  - [ ] Align serde attributes across structs and update module documentation explaining migration steps.
- [ ] **Compatibility guardrails**
  - [ ] Introduce backward compatibility fixtures exercising legacy manifests.
  - [ ] Add unit tests covering serialization/deserialization, round-trip equality, and optional fields.
- [ ] **Quality gates**
  - [ ] Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test`; enforce via CI workflow updates.

**Risks & mitigation:** Downstream consumers might rely on duplicate fields → communicate migration plan and provide deprecation window; serde attribute drift → add lint enforcing attribute parity.

---

## 6. Extend GPU Detection Beyond NVIDIA

**Objective:** Expand hardware detection to produce normalized capability descriptors for AMD, Intel, and Apple GPUs.

**Success criteria:**
- Capability probe returns consistent descriptors across vendors, including containerized environments.
- Scheduler uses enriched descriptors to inform workload placement.

**Dependencies:** Access to AMD/Intel/Apple hardware (or cloud equivalents), legal clearance for vendor SDK distribution.

**Milestones & tasks:**
- [ ] **Architecture**
  - [ ] Introduce a hardware detection trait in `core/hardware` to encapsulate vendor-specific probes.
  - [ ] Define normalized descriptor structs (memory, compute units, supported frameworks, driver version).
- [ ] **Vendor implementations**
  - [ ] Implement AMD detection via ROCm SMI, Intel via Level Zero/oneAPI, and Apple via Metal System Profiler APIs.
  - [ ] Add fallbacks for environments lacking vendor tooling (e.g., parse `/proc` or driver outputs) returning degraded descriptors.
- [ ] **Validation & rollout**
  - [ ] Create mocks or recorded outputs to simulate each vendor and cover virtualization/container edge cases.
  - [ ] Update runtime scheduling policies to consume expanded descriptors and document configuration flags.

**Risks & mitigation:** Vendor tooling may require elevated privileges → document requirements and provide degraded mode; hardware coverage gaps → partner with infra team for remote lab access.

---

## 7. Deliver Value-Add Ecosystem Content

**Objective:** Launch a cohesive ecosystem release bundling blueprints, marketplace assets, and analytics resources with clear community pathways.

**Success criteria:**
- Ecosystem bundle artifact published with release notes, install instructions, and verification checklist.
- Community announcement drives adoption metrics (downloads, sign-ups) tracked via analytics dashboards.

**Dependencies:** Completion of tasks 1–6, coordination with marketing/comms, publication infrastructure for release bundles.

**Milestones & tasks:**
- [ ] **Release packaging**
  - [ ] Curate blueprint bundles, marketplace items, and analytics datasets into a distributable artifact (zip/tarball) with checksum and manifest.
  - [ ] Draft release notes summarizing highlights, upgrade steps, and backward compatibility notes.
- [ ] **Documentation alignment**
  - [ ] Update `docs/workflows`, `docs/community`, and `docs/analytics` with feature spotlights, cross-links, and onboarding walkthroughs.
  - [ ] Produce `docs/releases/ecosystem_launch.md` playbook covering deliverables, upgrade paths, FAQs, and rollback guidance.
- [ ] **Community enablement**
  - [ ] Coordinate announcements across UI dashboard, release notes, Discord/Forum, and partner newsletters with clear CTAs.
  - [ ] Establish ecosystem contribution guidelines, reviewer rotation, triage board, and SLAs for incoming submissions.

**Risks & mitigation:** Launch alignment may slip due to upstream blockers → maintain dependency tracker and contingency date; community guidelines adoption risk → run office hours/webinars post-launch.

## Task Details

<a id="task-agentos-1"></a>
### AGENTOS-1 — Populate Workflow Blueprint Catalog

See [Populate Workflow Blueprint Catalog](#1-populate-workflow-blueprint-catalog) for objectives, success criteria, dependencies, milestones, and risks.

<a id="task-agentos-2"></a>
### AGENTOS-2 — Deliver Marketplace Assets & Tooling

See [Deliver Marketplace Assets & Tooling](#2-deliver-marketplace-assets--tooling) for objectives, success criteria, dependencies, milestones, and risks.

<a id="task-agentos-3"></a>
### AGENTOS-3 — Implement Analytics Pipelines

See [Implement Analytics Pipelines](#3-implement-analytics-pipelines) for objectives, success criteria, dependencies, milestones, and risks.

<a id="task-agentos-4"></a>
### AGENTOS-4 — Seed Telemetry Storage Samples

See [Seed Telemetry Storage Samples](#4-seed-telemetry-storage-samples) for objectives, success criteria, dependencies, milestones, and risks.

<a id="task-agentos-5"></a>
### AGENTOS-5 — Fix CICD Pipeline Struct Duplication

See [Fix CICD Pipeline Struct Duplication](#5-fix-cicd-pipeline-struct-duplication) for objectives, success criteria, dependencies, milestones, and risks.

<a id="task-agentos-6"></a>
### AGENTOS-6 — Extend GPU Detection Beyond NVIDIA

See [Extend GPU Detection Beyond NVIDIA](#6-extend-gpu-detection-beyond-nvidia) for objectives, success criteria, dependencies, milestones, and risks.
