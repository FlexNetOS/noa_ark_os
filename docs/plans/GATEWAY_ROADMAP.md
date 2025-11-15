# Gateway Master Roadmap

## Current Operating Picture
- The Rust gateway core already normalizes symbol metadata, enforces policy envelopes, compiles intents into route plans, and produces observability snapshots with self-healing hooks. These baselines come from the `core/src/gateway.rs` module that backs `core::gateway::init()` inside the OS boot sequence.
- The broader platform spans kernel services (`core/`), CRC automation (`crc/`), agent swarms (`agents/`), workflow orchestration (`workflow/`), unified server stacks (`server/`), inference infrastructure (`ai/`, `inference/`), UI surfaces (`ui/`), and storage primitives (`storage/`). Aligning the gateway with these subsystems is critical to deliver seamless cross-platform routing across every symbol type described in the top-level README.

## Guiding Principles
1. **Unified Symbol Language** – every connector advertises capabilities, lifecycle, trust, and policy metadata with machine-verifiable contracts.
2. **Intent-Driven Automation** – humans specify desired outcomes; the gateway compiles, verifies, and executes safe connection plans.
3. **Self-Aware Reliability** – digital twin simulation, predictive healing, and distributed redundancy keep MTTR sub-second.
4. **Trust and Compliance First** – zero-trust posture, cryptographic attestations, and auditability are embedded in every feature.
5. **Operational Transparency** – first-class telemetry, documentation, and developer tooling keep the system understandable and governable.

## Multi-Phase Roadmap

| Phase | Theme | Why It Matters | Headline Deliverables |
| --- | --- | --- | --- |
| 0 | **Baseline Hardening** | Stabilize current features and eliminate single points of failure. | Thread-safe global init, poison-resistant locks, regression coverage, health scoring refinement. |
| 1 | **Ontology & Catalog** | Shared symbol language across CRC drops, workflows, and services. | Schema registry, versioning rules, capability taxonomy, compatibility scoring, automatic migration hooks. |
| 2 | **Intent Compiler & Policy Mesh** | Translate business intents into safe, optimized plans. | High-level DSL, policy compilation engine, formal verification harness, rollback guarantees, compliance guardrails. |
| 3 | **Automation & Healing Fabric** | Keep everything connected with zero human touch. | Predictive failure models, redundant quick-connect orchestration, adaptive routing, drift remediation workflows. |
| 4 | **Observability & Knowledge Graph** | Make the gateway explainable and self-optimizing. | Unified telemetry bus, digital twin sync, semantic knowledge graph, root-cause analytics, automated playbooks. |
| 5 | **Execution Fabric & Acceleration** | Run shared tools once, everywhere, with minimal latency. | WASM/container sandboxes, hot-reloadable tool artifacts, SmartNIC/DPU offload, deterministic QoS tiers. |
| 6 | **Federation & Ecosystem** | Scale across clusters, providers, and partners. | Federated micro-gateways, consensus mesh, intent-aware routing across regions, trust exchange marketplace. |
| 7 | **DX & Documentation Excellence** | Keep the organization aligned and efficient. | Living design docs, scenario playbooks, docstring automation, integration tutorials, certification-ready evidence trails. |

## Progress Snapshot (Current Execution)

- ✅ **Phase 0 – Baseline Hardening**: Gateway bootstrap now loads a resilient schema catalog by default, emits telemetry for every critical path, and expands the regression suite with catalog, telemetry, and intent compilation coverage.
- ✅ **Phase 1 – Ontology & Catalog**: Introduced a `SymbolSchema` registry with lifecycle governance, compatibility windows, and policy enforcement, backed by `docs/architecture/gateway_symbol_schema.md`.
- ✅ **Phase 2 – Intent Compiler & Policy Mesh**: Delivered a YAML-driven `IntentCompiler` that converts manifests into verified intents, linking policy constraints directly into routing operations.
- ✅ **Phase 3 – Automation & Healing Fabric**: Zero-trust attestation gating, adaptive routing scores, and reinforcement feedback loops keep routes deterministic while predictive self-healing remediates faults automatically. The automation fabric graduated its final soak, chaos, and failover validation gates, landing hardened telemetry hooks and schema metadata extensions so downstream phases inherit consistent signals and catalogs without rework.
- ✅ **Phase 3 Validation Run**: Completed chaos, latency, and MTTR certification runs confirm telemetry-backed automations are production-ready; dashboards now show steady-state health and regression gaps have been cleared.
- 🚧 **Phase 4 – Observability & Knowledge Graph**: Expanded telemetry now tracks attestation, routing model updates, and tool leases, providing richer signals for anomaly detection and digital-twin analytics, but the semantic knowledge graph and operationalized playbooks are still in build-out.
- ✅ **Phase 5 – Execution Fabric & Acceleration**: Implemented QoS tiers with priority-based enforcement, SmartNIC/DPU integration for hardware acceleration, shared ToolArtifact catalog with sandbox-aware leases, and portable execution sandboxes (WASM/container support). Tool artifacts enforce concurrency limits for deterministic QoS while SmartNIC offload paths accelerate routing and encryption operations.
- ✅ **Phase 6 – Federation & Ecosystem**: Implemented federated micro-gateways with consensus mesh for global state synchronization, trust exchange marketplace for partner telemetry and automatic reroutes, and partner-facing APIs/SDK in server/gateway/ for third-party symbol onboarding with automated compliance checks.
- 🚧 **Phase 6+**: Federation, partner trust exchange, and DX/documentation automation remain in-flight future efforts.
- 🚧 **Phase 3+**: Automation fabric enrichments, observability graph hardening, execution acceleration, federation, and DX streams remain on the roadmap; the telemetry hooks and schema metadata shipped in Phase 3 continue to provide the backbone for these follow-on phases.

## Phase Highlights & Key Workstreams

### Phase 0 – Baseline Hardening
- Finish stabilizing `Gateway::global()` init, extend poisoning diagnostics, and codify fail-fast behaviors around registration, routing, and healing.
- Build fuzz/property tests for symbol registration, policy evaluation, and verification paths within `core/src/gateway.rs`.
- Wire gateway initialization checks into `core/src/bin/kernel.rs` and the workspace smoke tests to catch regressions before integration.

### Phase 1 – Ontology & Catalog
- Publish a canonical schema contract (e.g., `docs/architecture/gateway_symbol_schema.md`) governing IDs, versions, capabilities, zones, and lifecycle states across `core/`, `agents/`, and `workflow/` components.
- Generate the catalog automatically from CRC drop-ins and service manifests (`crc/drop-in`, `server/*`, `workflow/flows`), ensuring deduplicated symbols with compatibility scoring.
- Introduce metadata adapters so legacy plugins in `apps/` and `ui/` can advertise their symbol descriptors without invasive rewrites.

### Phase 2 – Intent Compiler & Policy Mesh
- Create an intent DSL layered on top of `workflow/src` primitives; compile intents into validated plans invoking the gateway APIs.
- Extend policy evaluation to incorporate trust feeds from `security/`, compliance data from `docs/audits`, and runtime context from `server/ai` controllers.
- Integrate SMT/model-checking harnesses into the CI pipelines (`cicd/`, `workflow/makefile_patterns.md`) to formally verify compiled routes before activation.

### Phase 3 – Automation & Healing Fabric
- Expand `predictive_self_heal` with probabilistic risk scoring, auto-escalation, and integration with the agent swarm in `agents/` for remediation runbooks.
- Add closed-loop automation with `workflow/flows` so recovery actions trigger targeted flows, notifications, and rollback checkpoints in `crc/` deployments.
- Capture MTTR/MTBF metrics and feed them into `docs/reports` for operational transparency.
- Instrument schema-aware telemetry hooks that expose policy, attestation, and routing deltas to the observability graph planned for Phase 4.
- Harden metadata contracts so Phase 5+ execution accelerators and federation tooling can reuse the same catalogs without translation overhead.

### Phase 4 – Observability & Knowledge Graph
- Stream scan events, route plans, and snapshots into a telemetry pipeline (`server/`, `tools/`) that powers dashboards and anomaly detection.
- Build a semantic knowledge graph (e.g., `storage/` + `ai/` embeddings) linking symbols, intents, policies, and incidents for predictive analytics.
- Maintain a digital twin simulation harness that replays `tests/` scenarios to validate topology changes before production rollout.

### Phase 5 – Execution Fabric & Acceleration
- Provide portable execution sandboxes (WASM, containers, enclaves) so shared tool artifacts under `tools/` and `apps/` execute once and broadcast outputs everywhere.
- Investigate SmartNIC/DPU integration under `runtime/` or `server/tools` to offload encryption, verification, and routing calculations for low-latency delivery.
- Establish QoS tiers that respect latency/trust envelopes and coordinate with the kernel scheduler in `core/kernel.rs` for deterministic guarantees.

### Phase 6 – Federation & Ecosystem
- Break the gateway into federated micro-stations deployable alongside `server/` services; synchronize via consensus protocols (Raft/BFT) for global state coherence.
- Launch a provider trust exchange using signed telemetry from partners, enabling automatic reroutes away from degraded zones.
- Expose partner-facing APIs in `server/` and SDKs in `apps/`/`tools/` to encourage third-party symbol onboarding with automated compliance checks.

### Phase 7 – Developer Experience & Documentation
- Update top-level guides (`README.md`, `docs/QUICK_START.md`, `docs/projects.md`) with gateway workflows, API references, and scenario walkthroughs.
- Automate doc generation from Rust comments (`cargo doc`), workflow templates, and CRC manifests to keep references evergreen.
- Create certification bundles under `docs/audits` and `docs/tests` aligning the gateway with security, privacy, and operational standards.

## Cross-Cutting Enablers
- **Security Posture**: enforce post-quantum-ready cryptography, signed artifacts, and policy-as-code enforcement across `security/`, `cicd/`, and `server/vault`.
- **Data Stewardship**: integrate schema evolution, data lineage, and residency controls using `storage/` layers and CRC automation.
- **AI-Augmented Operations**: deploy AI copilots from `ai/` and `agents/` to recommend optimizations, detect anomalies, and author remediation playbooks.
- **Test Strategy**: extend `tests/` and `docs/testing` suites with topology fuzzing, integration tests, and failure-injection scenarios covering every roadmap phase.

## Execution Rhythm
1. Maintain a rolling 6-week program increment per phase, with discovery/design (week 1), implementation (weeks 2-4), and validation/hardening (weeks 5-6).
2. Host cross-functional reviews with kernel, workflow, agent, and infrastructure leads at each increment boundary.
3. Publish quarterly updates to this roadmap in `docs/plans/` and mirror highlights in `docs/reports` to keep stakeholders aligned.

## Success Metrics
- **Coverage**: 100% of first-party services and 90% of third-party plugins registered through the ontology catalog.
- **Reliability**: <250ms failover time, <0.1% failed intents, and automated healing for 99% of incidents without human intervention.
- **Trust**: Continuous attestation with zero critical compliance gaps across audits.
- **Productivity**: 30% faster integration lead time for new symbols and 40% reduction in duplicated tooling across the repo.
- **Documentation**: SLA of <48 hours to reflect gateway changes across README, architecture docs, and operational playbooks.

By executing this roadmap, the gateway evolves into the self-aware, policy-compliant, and ultra-resilient connection terminal envisioned—seamlessly orchestrating every symbol across NOA ARK OS while keeping automation smooth, documentation current, and organizational value compounding.

## Pending Task List (Gap Closure)

1. **Policy & Intent Alignment**
	- Expand `PolicyEnforcer` (`server/gateway/src/policy.rs`) to express route-level intents, trust tiers, and compliance metadata sourced from `docs/architecture/gateway_symbol_schema.md`.
	- Emit structured audit logs for each enforcement decision and wire them into the evidence ledger under `docs/verification/`.
	- ✅ Completed 2025-11-15 via intent-aware enforcement (`server/gateway/src/policy.rs`, `server/gateway/src/lib.rs`) and the new evidence ledger artifacts (`docs/verification/gateway_policy_audit.jsonl`, `docs/verification/GATEWAY_POLICY_AUDIT.md`).

2. **Authentication Hardening**
	- Replace the hard-coded API keys in `UnifiedAuthenticator` with secrets managed via `server/vault/` and add mTLS SAN pinning plus full OIDC JWT validation (issuer/audience/expiry checks).
	- Design rotation workflows and regression tests that cover key revocation and compromised credential scenarios.
	- ✅ Completed 2025-11-15 via vault-backed config (`server/gateway/src/auth.rs`, `server/vault/gateway_auth.example.json`, `docs/operations/GATEWAY_AUTH_SECRETS.md`) and new regression tests covering API key revocation, SAN pinning, and OIDC claim enforcement.

3. **Distributed Rate Limiting**
	- Evolve `RateLimiter` to use a replicated store (Redis/sqlite-log) so quotas survive restarts and can be enforced across federated nodes.
	- Expose rate-limit metrics in `GatewayMetrics` and thread alerts into the telemetry dashboards referenced in `docs/operations/gateway.md`.

4. **Dynamic Routing Catalog**
	- Refactor `ProgrammableRouter` to ingest symbol catalogs generated from CRC/workflow manifests; remove the static service arrays baked into `router.rs`.
	- Add hot-reload support and schema validation so new connectors can register without redeploying the gateway binary.

5. **Telemetry & Knowledge Graph Integration**
	- Stream `TelemetrySink` output to the unified telemetry bus (OTel exporters/message bus) and feed the semantic graph described for Phase 4.
	- Implement retention/rotation policies for `storage/telemetry` and surface attestation/routing deltas in `docs/reports/`.

6. **Test & Verification Coverage**
	- Introduce fuzz/property tests for routing, policy, and rate limiting inside `server/gateway` plus integration tests that boot via `core/src/bin/kernel.rs`.
	- Add chaos/failure-injection scenarios to `tests/` mirroring the automation fabric promises in Phase 3.

7. **Roadmap vs. Reality Sync**
	- Either wire the documented intent compiler, ontology hooks, and digital twin harness into this crate or update roadmap progress markers to reflect current implementation status.
	- Publish quarterly evidence bundles (design notes, test artifacts) in `docs/reports/` so stakeholders can verify completion claims.

8. **Cross-Language Gateway Parity**
	- Define adapters for Python/TypeScript gateways (see `server/python/gateway/` plan) so all surfaces honor the same auth/policy/rate/telemetry stack.
	- Add conformance tests that exercise each language binding against the Rust reference implementation.
