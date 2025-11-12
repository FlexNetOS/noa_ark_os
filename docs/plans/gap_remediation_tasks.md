# Gap Remediation Task List

This document tracks actionable tasks to resolve outstanding gaps and quality issues identified during verification of the NOA ARK OS roadmap implementation.

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

<!-- BEGIN: GAP_REMEDIATION_TASKS -->

## Ensure everything depends on the kernel/service graph, simplify deployment, and make the platform portable across environments by default.
Ground platform evolution in a single kernel-first dependency graph that packages services consistently for any environment.
**Suggested task**
- Establish kernel-first dependency graph and portable packaging — [View task](#task-kernel-first)

## Single-Host AgentOS Server Orchestration
Deliver a constrained, single-host deployment profile that keeps the complete stack healthy without external orchestration.
**Suggested task**
- Design single-host AgentOS server profile — [View task](#task-single-host)

## Unified UI/UX Shell & Interaction Model
Present a cohesive operator experience across desktop, web, and terminal surfaces that reuses the same interaction vocabulary.
**Suggested task**
- Ship unified multi-surface UI shell — [View task](#task-ui-shell)

## Adaptive Runtime & Portability Enhancements
Add runtime intelligence to detect host capabilities, tune workloads, and keep deployments portable across infrastructure classes.
**Suggested task**
- Implement adaptive runtime orchestration — [View task](#task-adaptive-runtime)

## Advanced Gateway & Observability
Upgrade ingress, policy, and telemetry flows so operators gain real-time insight and can enforce controls without friction.
**Suggested task**
- Develop advanced gateway with observability — [View task](#task-gateway-observability)

## Value-Added Service Ecosystem
Layer differentiated services and packaged workflows on top of the hardened core to accelerate customer value delivery.
**Suggested task**
- Launch value-add ecosystem features — [View task](#task-value-ecosystem)

## Task Details

<a id="task-kernel-first"></a>
### AGENTOS-1 — Establish kernel-first dependency graph & portable packaging
**Description:** Build a canonical dependency map anchored on the kernel/service graph and package baselines that run identically on local, cloud, and air-gapped hosts.

**Checklist**
- [ ] Document the kernel-centered dependency graph with required and optional services.
- [ ] Produce reproducible build artifacts for each service with pinned interfaces.
- [ ] Provide environment-agnostic packaging (container, tarball, OCI layout) driven from one manifest.
- [ ] Implement compatibility validation covering Linux, macOS, and containerized targets.
- [ ] Automate dependency compliance checks inside CI to guard drift.

**Acceptance criteria**
- Kernel-first dependency graph published with machine-readable manifest and human guide.
- Packaging workflow produces artifacts that pass smoke tests on Linux, macOS, and container runners.
- CI blocks merges when dependency or interface drift is detected.
- Downstream task owners acknowledge the manifest as their source of truth.

**Meta**
- Owner: codex
- Priority: P0
- Status: Processing
- Depends on: None

<a id="task-single-host"></a>
### AGENTOS-2 — Design single-host AgentOS server profile
**Description:** Define and validate a single-host profile that boots the full AgentOS stack with deterministic ordering, self-healing, and constrained resource envelopes.

**Checklist**
- [ ] Capture service inventory, boot order, and health semantics for the single-host mode.
- [ ] Provide orchestration scripts or manifests that start/stop all services with one command.
- [ ] Integrate automated health and readiness probes tied to the kernel-first manifest.
- [ ] Bundle local observability (logs, metrics) into an operator-facing snapshot.
- [ ] Document resource envelopes and scaling guidance for baseline and high-availability hardware.

**Acceptance criteria**
- One command provisions and boots the single-host stack to a ready state within defined SLAs.
- Health probes surface in a consolidated status view and recover failed services without manual intervention.
- Local observability bundle exposes logs and metrics with documented access steps.
- Documentation references AGENTOS-1 manifest without divergence.

**Meta**
- Owner: codex
- Priority: P0
- Status: Processing
- Depends on: AGENTOS-1

<a id="task-ui-shell"></a>
### AGENTOS-3 — Ship unified multi-surface UI shell
**Description:** Create a shared UI shell, interaction model, and component library that can be deployed across desktop, browser, and CLI interfaces.

**Checklist**
- [ ] Inventory current UI entry points and map to shared navigation, layout, and identity patterns.
- [ ] Produce component library packages consumable by desktop, web, and terminal surfaces.
- [ ] Implement unified session/state management with pluggable auth providers.
- [ ] Add cross-surface telemetry to capture usage and error flows.
- [ ] Provide migration guides for teams adopting the shared shell.

**Acceptance criteria**
- UI shell renders consistently across supported surfaces with parity in core navigation and tasks.
- Component library versioning and distribution documented for all consuming teams.
- Unified state management passes integration tests in single-host profile (AGENTOS-2).
- Usage telemetry spans all surfaces and reports into the observability stack.

**Meta**
- Owner: codex
- Priority: P1
- Status: Processing
- Depends on: AGENTOS-2

<a id="task-adaptive-runtime"></a>
### AGENTOS-4 — Implement adaptive runtime orchestration
**Description:** Extend runtime controllers to detect host capabilities, adjust workload placement, and surface portability guidance tied to the kernel graph.

**Checklist**
- [ ] Build capability detectors leveraging hardware, OS, and workload telemetry inputs.
- [ ] Map detection outputs to scheduling policies aligned with kernel-first dependencies.
- [ ] Implement fallback strategies for unsupported or degraded environments.
- [ ] Add simulation tests covering heterogeneous hosts and failure cases.
- [ ] Publish operator documentation for tuning adaptive policies.

**Acceptance criteria**
- Runtime automatically selects compatible workloads per host profile with auditable decisions.
- Portability guidance updates documentation and CLI feedback in sync.
- Simulation suite covers at least three heterogeneous scenarios and passes in CI.
- Adaptive controls remain configurable via the single-host profile without manual patching.

**Meta**
- Owner: codex
- Priority: P1
- Status: Processing
- Depends on: AGENTOS-1, AGENTOS-2

<a id="task-gateway-observability"></a>
### AGENTOS-5 — Develop advanced gateway with observability
**Description:** Modernize the gateway to deliver policy enforcement, traffic shaping, and deep observability linked to platform telemetry expectations.

**Checklist**
- [ ] Define ingress, policy, and observability requirements aligned with kernel-first services.
- [ ] Implement gateway routing, authentication, and rate-limiting features with tests.
- [ ] Integrate telemetry export (metrics, traces, logs) into the observability pipeline.
- [ ] Provide dashboards and alert templates for gateway health.
- [ ] Document operator runbooks for incident response scenarios.

**Acceptance criteria**
- Gateway enforces policy and routing rules validated by automated integration tests.
- Observability data streams into the standard telemetry stack with dashboards for latency, errors, and throughput.
- Runbooks detail detection and mitigation steps for at least three incident classes.
- Gateway configuration references AGENTOS-2 baseline without duplicating definitions.

**Meta**
- Owner: codex
- Priority: P1
- Status: Processing
- Depends on: AGENTOS-2

<a id="task-value-ecosystem"></a>
### AGENTOS-6 — Launch value-add ecosystem features
**Description:** Bundle advanced services, curated workflows, and partner integrations that sit atop the hardened platform and gateway foundation.

**Checklist**
- [ ] Define ecosystem packaging that pulls from marketplace, workflow, and analytics assets.
- [ ] Implement enablement toggles and licensing gates respecting dependency readiness.
- [ ] Produce onboarding flows linking unified UI shell with new services.
- [ ] Add success metrics dashboards to verify adoption and ROI.
- [ ] Publish partner integration guidelines leveraging gateway and runtime capabilities.

**Acceptance criteria**
- Ecosystem bundle installs via documented steps and validates dependencies on AGENTOS-3, AGENTOS-4, and AGENTOS-5.
- Feature toggles correctly gate availability based on licensing and readiness signals.
- Onboarding flows verified end-to-end across all supported UI surfaces.
- Adoption dashboards expose tracked metrics sourced from the observability stack.

**Meta**
- Owner: codex
- Priority: P1
- Status: Processing
- Depends on: AGENTOS-3, AGENTOS-4, AGENTOS-5

Roadmap alignment: This sequence locks platform control through the kernel-first graph, delivers a resilient single-host deployment, unifies user experience, adds intelligent runtime and gateway ingress, and culminates in differentiated services layered on the hardened foundation for consistent portability.

<!-- END: GAP_REMEDIATION_TASKS -->

