# AGENT.md

**AGENT_POLICY_VERSION:** 1.0.0
**Status:** Active
**Scope:** Entire NOA ARK OS repository (agents, providers, tools, skills, automation, runtime, CI, UI).
**Authoritative Source:** This file. All provider instruction files must defer to this document.

> **Policy Markers**
> `@-- BEGIN:AGENT_POLICY --` … `@-- END:AGENT_POLICY --`
> Automation may only replace content between these markers; all other sections are immutable.

@-- BEGIN:AGENT_POLICY --

## 0) Purpose

Establish one strict operating policy for every agent-driven action so each task **heals** the codebase, **upgrades** capability, and **never downgrades nor deletes**. The policy blocks content rot, enforces cross-platform offline-first autonomy, and preserves architectural clarity across NOA ARK OS services, CRC, UI, workflows, storage, and tooling.

---

## 1) Non-Negotiable Rules

### Heal, Do Not Harm (Top Priority)

* **Upgrade, Enhance, Never Downgrade or Delete.**
* When a replacement is unavoidable, **compress + archive** the prior asset first; never remove history.
* Changes must remain **surgical**, **reversible**, and **traceable** via the archival ledger.

### Anti-Rot

* Every task must improve tests, coverage, documentation linkage, or automation quality.
* Stale references, dead endpoints, or orphaned files must be flagged and queued with remediation tasks—no silent decay.

### Architectural Clarity

* Maintain a canonical map of where code belongs (agents, ai, apps, core, crc, runtime, server, storage, ui, workflow, etc.).
* New components must declare their layer, dependencies, owners, and archival impact.

### Offline-First Ownership (Online via Feature Flag)

* **Default mode:** offline/local execution.
* **Online operations:** allowed only when `ONLINE_*` feature flags are explicitly enabled and logged.

### No Duplication, No Deletion

* Never duplicate instructions or logic across providers; use **gateways** and shared modules.
* Never delete files; if superseding content, compress+archive the prior version and document the swap.
* Preserve or wrap dead-code helpers and stubs so legacy integrations can be restored quickly.

### Contained Environment & Dependency Governance

* Treat NOA ARK OS as a **self-contained system**—all runtimes, libraries, assets, and secrets must live inside or be routed through the repository's managed gateways.
* Provision dependencies via gateway-managed manifests (see `services/gateway/service.py`, `server/gateway/`, `.workspace/config/`, `.workspace/registry/*.json`, `tools/portable_builder.py`) so nothing bypasses archival or verification controls.
* Route every environment variable, credential, configuration knob, dependent runtime, and secret through gateway shims and adapters maintained in `server/gateway/src/*.rs`, `services/gateway/`, `agents/src/implementations/generated/infrastructure/*gateway*`, and related tooling; no ad-hoc `.env` or external service dependencies.
* Record environmental assumptions and manifest updates in the Evidence Ledger and mirror them in `.workspace/registry/` (or corresponding gateway config files) whenever a task introduces or changes gateway-managed configuration.
* Provision dependencies via gateway-managed manifests (package registries, build kits, toolchains) so nothing bypasses archival or verification controls.
* Route every environment variable, credential, configuration knob, and secret through gateway shims (`agents/gateways/`, `tools/`, `.workspace/registry/`) with auditable definitions and fallbacks; no ad-hoc `.env` or external service dependencies.
* Record environmental assumptions in the Evidence Ledger whenever a task introduces or changes gateway-managed configuration.

**Archival Procedure (mandatory):**

* Archive path: `archive/YYYY/MM/<relative-path>.tar.zst`
* Ledger entry: original path, reason, timestamp, SHA-256, replacement path
* Commit archive + ledger with the replacement change.

---

## 2) System Goals & Operational Guardrails

1. **Always Improve Quality:** Fix known errors in touched areas, close gaps (tests/types/docs), and increase style/safety/performance.
2. **Fully Autonomous from UI:** UI actions flow through orchestrator → planner → sub-agents → execution → verification → report with logged decisions.
3. **Cross-Platform Delivery:** Support Linux, macOS, Windows and PC/server/web/mobile/glasses/headset targets; ensure CI gates cover OS families (Linux required, others via matrix when feasible).
4. **4-D Method on Every Task:** Apply Deconstruct → Diagnose → Develop → Deliver with a recorded 3–7 bullet high-level plan.
5. **Sub-Agents & Helpers:** Partition large work among dedicated sub-agents (ingest, analysis, refactor, verification, packaging) to manage repo scale.
6. **Memory + DB + Neural Agents:** Maintain long-lived repository memory (indexes, symbol graphs) plus per-task memory; leverage `.workspace/`, `.graphs/`, and registry assets.
7. **Strict Verification Protocol:** Use Triple-Verification and the Truth Gate before completion claims.
8. **Organization Management & Gateway Reuse:** Enforce single sources of truth, re-use assets via gateways, and keep duplicate-check CI passing.
9. **Provider Instruction Uniformity:** Provider files remain empty pointers that direct readers back to this policy.

### Phase 0.5–10 Operator Patterns

Detailed execution guidance for the active roadmap phases lives in
`docs/guides/AGENTIC_OS_GUIDE.md` (sourced from
`docs/tasks/ROADMAP_AGENTIC_KERNEL_TASK_LINKS.md`). Operators must:

- **Phase 0.5 (CLI-First Foundation):** Drive all actions through CLI targets (`Makefile`, `pnpm`, `cargo`) and record evidence snapshots before state changes.
- **Phase 1 (Kernel Baseline):** Keep kernel manifests authoritative and trigger `make snapshot` prior to structural migrations.
- **Phase 2–4 (North Star, Contract Tests, CLI Expansion):** Extend capability registries instead of importing subsystems directly and publish machine-readable evidence for automation replay.
- **Phase 5 (Gateway Tokens):** Enforce registry-only execution; capability tokens issued via `services/gateway/` gate all runtime launches.
- **Phase 6–7 (Retrieval Discipline, Reward System):** Update metrics and analytics in lock-step with capability changes to preserve reproducibility.
- **Phase 8–10 (SBOM Split, Deployment Profiles, Machine-First Pipelines):** Generate SBOM placeholders, promote deployments through gateway-controlled profiles, and document machine-first behaviors in Truth Gate artifacts.

---

## 3) Repository Structure Map (Where Things Belong)

Grounded in the current repository layout:

```
agents/              # Orchestrators, planners, worker runtimes, generated gateway agents
ai/                  # AI engine integrations, llama.cpp bindings, model control
apps/                # System applications and UI frameworks
crc/archive/         # Compressed superseded assets + ledger
cicd/                # CI/CD orchestration, pipelines, deployment assets
core/                # Kernel/core OS services (process, memory, IPC, security)
crc/                 # Continuous ReCode (ingest, DAGs, transforms, sandboxes)
crc-adapter-sdk/     # SDK for CRC integrations
docs/                # Architecture, plans, SOPs, roadmaps, onboarding
runtime/             # Multi-language runtime environments
server/              # Unified application/MCP server, transport routers, gateway policy (see `server/gateway/`)
sandbox/             # Sandbox system libraries and isolation layers
storage/             # File system, databases, configuration engines
services/gateway/    # Python service wrapper for manifest/policy enforcement and telemetry export
tools/               # Build kits, utilities, safety tools, developer helpers
ui/                  # Multi-surface UI shells (web/desktop/mobile/AR/XR)
workflow/            # Workflow automation and coordination assets
.github/workflows/   # CI jobs (lint, tests, duplicate checks, platform matrix)
policies/            # Additional guardrail configs (must defer to AGENT.md)
.graphs/             # Architecture diagrams and dependency visuals
.workspace/          # Managed workspaces, registry, SOPs, indexes
.self-hosted/        # Self-hosted app registry and deployment configs
.cargo/              # Cargo configuration overlays
```

Reference documents: `HIERARCHY.md`, `WORKSPACE_ORGANIZATION_PLAN.md`, `docs/architecture/`, `.workspace/registry/`, `.graphs/`.

---

## 4) The 4-D Method (Required on Every Task)

1. **DECONSTRUCT:** Restate intent, inputs, constraints, and missing data.
2. **DIAGNOSE:** Surface ambiguities, risks, gaps, and architecture impact.
3. **DEVELOP:** Produce creative, technical, educational, and complex reasoning tracks with cross-platform considerations.
4. **DELIVER:** Assign roles, execute with verification, document archival/logging decisions.

**Per-Task Planning Checklist (3–7 bullets, conceptual only)**

* [ ] Inputs & constraints acknowledged
* [ ] Interfaces/boundaries named
* [ ] Risks & mitigations identified
* [ ] Minimal viable increment defined
* [ ] Verification criteria declared
* [ ] Rollback + archival impact considered
* [ ] Sub-agent responsibilities enumerated

---

## 5) Operational Protocol

### 5-Step Execution Process

1. **Clarify inputs:** Restate the task, assumptions, blockers.
2. **Plan:** Outline minimal evidence steps; choose tests/checks.
3. **Gather:** Collect only necessary data; log sources and timestamps.
4. **Execute:** Change the smallest testable unit first; capture logs.
5. **Verify:** Apply the Truth Gate before any completion claim.

**Truth Sources Priority**

1. User-provided instructions/files
2. Computations performed here with visible work
3. Referenced external sources (cited)
4. Model prior knowledge

Resolve conflicts in favor of the highest-priority source.

---

## 6) Verification & Truth Gate

### Triple-Verification Protocol

* **Pass A — Self-check:** Ensure internal consistency, spec ↔ artifacts alignment, smoke tests.
* **Pass B — Independent re-derivation:** Recompute/re-run fresh and compare deltas.
* **Pass C — Adversarial:** Negative/boundary tests and cross-tool verification.

### Truth Gate Checklist

* [ ] All referenced artifacts exist with SHA-256 hashes recorded
* [ ] Smoke tests executed with logs and exit codes
* [ ] Requirements ↔ artifacts ↔ tests fully mapped
* [ ] Limits, supported configs, and failure modes documented
* [ ] Evidence ledger updated (files, hashes, sources)
* [ ] Gap scan completed (coverage, docs, dependencies)
* [ ] Triple-Verification passes logged (A/B/C)

**Standard Output Artifacts**

* **Claims Table:** `claims.table.md` with claim strength, evidence, tests, limits
* **Evidence Ledger:** `evidence.ledger.json` capturing files, hashes, data sources, formulas, tests
* **Truth Gate Checklist:** `truth_gate.checklist.md`
* **Result Block:**
  ```
  RESULT: PASS | PARTIAL | FAIL
  WHY: <one line>
  EVIDENCE: <references>
  NEXT: <smallest verifiable follow-up>
  VERIFIED_BY: <Pass A/B/C status>
  ```

---

## 7) Provider & Agent Instruction Files

Provider instruction files must remain empty pointers back to this policy.

* `.copilot`
  ```
  # Defer to AGENT.md
  Use AGENT.md at repo root as the sole policy and instruction source. Do not duplicate logic here.
  ```
* `CLAUDE.md`
  ```
  This file intentionally contains no instructions. All policies and execution rules are defined in AGENT.md.
  ```
* `.github/AGENT_POINTER.txt`
  ```
  Provider/agent instruction policy lives in AGENT.md. Do not place instructions elsewhere.
  ```

Set environment variables when supported:

* `AGENT_POLICY_PATH=AGENT.md`
* `AGENT_POLICY_VERSION=1.0.0`

---

## 8) Execution Guardrails

* **Non-destructive editing:** No deletions or history-dropping renames; archive superseded assets first.
* **Gateways, not copies:** Provider runtimes integrate through the shared gateway stack (`server/gateway/`, `services/gateway/`, generated gateway agents under `agents/src/implementations/generated/infrastructure/`) to avoid duplication, and **all connections must be routed through the gateway** (paths, routing, hooks, shims, auth, CAS, agents, providers, front-end, back-end, etc.).
* **Gateway-managed environments:** Configure dependencies, secrets, environment variables, and external integrations exclusively via gateway adapters and repository-contained manifests (`services/gateway/service.py`, `.workspace/config/`, `.workspace/registry/*.json`, `tools/portable_builder.py`); document updates in the Evidence Ledger and workspace registry checkpoints.
* **Conventional Commits:** Use `feat`, `fix`, `refactor`, `docs`, `test`, `perf`, `chore`; include archival notes and feature-flag status in commit messages.
* **CI Acceptance:** Lint, type checks, unit tests, duplicate detectors, and offline jobs must pass. Linux job mandatory; macOS/Windows matrix best-effort.
* **Dead-code-aware stubs:** Wrap inactive pathways and document them for quick reactivation of archived features.

### Capability Token Flow (Phase 5 Rollout)

1. **Kernel issuance:** Use `core/kernel/security/issue_capability_token` for controlled tooling/tests. Each token carries `fs`, `network`, and `rate_limit_per_minute` claims signed with the kernel secret defined in `core/kernel/security/tokens.py`.
2. **Gateway enforcement:** `services/gateway/service.py` must call `verify_capability_token` before routing. Reject when client IDs mismatch, required fs/network scopes are absent, or token rates fall below `PolicyRule.rate_limit_per_minute`.
3. **Effective throttling:** Honor the stricter of gateway policy vs. token allowance to prevent clients from exceeding kernel quotas while keeping telemetry coherent.
4. **Extension loading:** Route every adapter load through `extensions.ExtensionRegistry`. Declarative manifests (`extensions/*/manifest.json`) declare scope requirements; registry refreshes allow hot-swaps without process restarts while guarding adapters with the same capability token checks.
5. **Telemetry + auditing:** Persist gateway telemetry with scope metadata (`fs_scope`, `network_scope`, `token_rate_limit`) so audits can trace which claim combinations were exercised per request.
### Dead Code Handling
* Quarantine any superseded component by copying it into `archive/quarantine/<component>@<commit>/` and filling in both `README.md` and `status.yaml` (owner, contact, reintegration gates, hash ledger).
* Capture a fresh repository snapshot with `make snapshot` immediately after quarantining files so the ledger records a verified rollback point.
* Reference archived assets via the ledger only—never link to the relocated files directly from live code.

### Reintegration Procedure
1. Review the quarantine bundle’s `status.yaml` gates and confirm every condition has been satisfied (tests, reviews, telemetry).
2. Run `cargo run -p quarantine_guard --bin quarantine_guard -- <paths>` on the candidate changes; the guard must report zero quarantined references.
3. Restore the bundle through `make rollback BUNDLE=archive/YYYY/MM/snapshots/<snapshot>.tar.zst` if provenance files are required for comparison, then delete the restored copy once verification is complete.
4. When bundles age past 90 days, execute `cargo run -p quarantine_guard --bin quarantine_rotate` (or allow the scheduled workflow) to relocate them into `archive/YYYY/MM/quarantine/` and append a rotation entry to the monthly ledger.

---

## 9) Autonomy & Sub-Agent Pattern

* **Agent-0 (Orchestrator):** Route tasks, resolve dependencies, partition work.
* **Planner:** Produce execution DAG aligned with the 4-D plan.
* **Workers:** Handle ingestion, analysis, refactor, verification, packaging; respect archival rules.
* **Verifier:** Execute Triple-Verification, maintain the Evidence Ledger, close the Truth Gate checklist.
* **Reporter:** Emit results, logs, metrics, and UI-facing summaries.
* **Project Management Integration:** Log every request in workspace/project tooling (`.workspace/todo/`, registry updates) for traceability.

Each sub-agent must save `claims.table.md`, `evidence.ledger.json`, and `truth_gate.checklist.md` in its task workspace.

---

## 10) Cross-Platform Standards

* Favor POSIX-friendly defaults; avoid case sensitivity issues.
* Provide PowerShell equivalents for shell commands when scripting.
* Package artifacts for Linux/macOS/Windows portability.
* UI components must render on web by default; gate native desktop/mobile/AR/XR surfaces behind feature flags.
* Feature flags must document offline vs. online behavior explicitly.

---

## 11) Completion Gate

Before marking a task complete:

* [ ] Archival ledger updated and committed with new assets
* [ ] Duplicate-check CI job passes (or logged reason if pending)
* [ ] Provider stubs verified to point back to AGENT.md
* [ ] Offline mode validated; online usage documented via feature flags
* [ ] Repository memory/indexes updated if affected (e.g., `.workspace/indexes/`)
* [ ] Evidence Ledger and Truth Gate artifacts attached to the task record
* [ ] Sub-agent logs archived for audit

@-- END:AGENT_POLICY --

---

## Appendices

### A) Orientation Aids

* `README.md`, `OVERVIEW.md`, `CONTRIBUTING.md`
* `HIERARCHY.md`, `WORKSPACE_ORGANIZATION_PLAN.md`, `WORKSPACE_MEMORY.md`
* `docs/architecture/`, `docs/plans/`, `docs/roadmap/`
* `.workspace/registry/` inventories and `.graphs/` diagrams
* `docs/guides/AGENTIC_OS_GUIDE.md` for end-to-end operator workflows, kernel sovereignty diagrams, and Phase 0.5–10 patterns
* `.github/workflows/` CI/CD enforcement points

### B) Provider Pointer Snippets

(See Section 7 for canonical content.)

### C) Tooling Quickstart

* Archive superseded files: `tar --zstd -cf archive/YYYY/MM/<relpath>.tar.zst <relpath>`
* Record SHA-256 hashes: `sha256sum <file>` (capture in Evidence Ledger)
* Run duplicate check before commit: mirror CI job locally where available

### D) Environment & Workflow Quick Reference

* **Primary execution context:** Windows 11 + PowerShell 7 with portable Cargo/Node under `server/tools/`; WSL/Linux acceptable for parity as long as you do not mix environments mid-session. Always run `./server/tools/activate-cargo.ps1` (or `source server/tools/activate-cargo-wsl.sh`) before any Rust command and `./server/tools/activate-node.ps1|.sh` before pnpm.
* **Build/Test cadence:**
  * `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all`
  * pnpm targets: `pnpm install --frozen-lockfile`, `pnpm build`, `pnpm lint`, `pnpm typecheck`, `pnpm test`
* **Feature workflow:** create `feature/<name>` branch → edit → run full build/test matrix above → commit using Conventional Commits → push. Archive any replaced assets under `archive/YYYY/MM/` with ledger updates before merging.
* **Fork processing (CRC):** place incoming fork under `crc/drop-in/incoming/forks/<name>/`, run `crc/detect-forks.ps1 -Mode process -ForkName "<name>"`, review generated branch `fork/<name>`, integrate, then archive originals when done.
* **Agent restoration pattern:** inspect backup in `agents/src/implementations/_backup/`, bring implementation back to `agents/src/implementations/`, register via `agents/src/registry.rs` and factory, add tests (`cargo test -p noa_agents`), update docs.
* **Security checklist:** no secrets in repo, prefer async I/O, no `unwrap()` in production paths, enforce gateway-managed env vars, scan forks before execution, honor duplicate/deletion/report guardrails.
* **Support & troubleshooting:** if Cargo/Node not found, re-run activation scripts; for CI parity use `make pipeline.local` and capture evidence artifacts under `audit/`. Use documentation files listed in Appendix A whenever deeper architectural guidance is required.

### E) Component Snapshot & Status

* **Agents (`agents/`):** 928 agents cataloged in `agents/src/registry.rs`; 26 placeholder integrations live, 902 pending restoration. Factory lives in `agents/src/factory.rs`; original implementations preserved under `agents/src/implementations/_backup/` and must be archived before replacement.
* **CRC (`crc/`):** Continuous ReCode system managing fork ingestion via `crc/drop-in/incoming/forks/` and archival via `crc/archive/forks/`. `crc/detect-forks.ps1` plus `crc/FORK_PROCESSING_SYSTEM.md` detail automation.
* **Core (`core/`):** Kernel + core services in Rust; treat manifests as authoritative for Phase 1 gating.
* **CI/CD (`cicd/`):** Pipeline assets tuned for <20 min commit→prod loops; wired into CRC for fork validation.
* **Workflow (`workflow/`), Sandbox (`sandbox/`), UI (`ui/`), Server (`server/`):** Multi-language orchestration, integration sandboxes (Pattern A/B/C→D), multi-surface shell targets, and the unified MCP/API server with portable tools under `server/tools/`.
* **Phase tracking:** Phase 1 complete (registry, fork infra, build verification); Phase 2 in progress (fork testing, CRC AI design, agent trait, DigestAgent); Phase 3 planned (runtime integration, AI engine, server hardening, mass agent restoration).

### F) Command & Toolchain Reference

```
# Rust workspace
cargo build --workspace
cargo build --workspace --release
cargo test --workspace
cargo check --workspace
cargo clean
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo fix --workspace --allow-dirty

# CRC fork automation
./crc/detect-forks.ps1 -Mode process -ForkName "fork-name"
./crc/detect-forks.ps1 -Mode list
./crc/detect-forks.ps1 -Mode watch -IntervalSeconds 60

# Examples
cargo run --example agent_registry_demo
cargo run --example full_system_demo
cargo run --example crc_cicd_demo
```

*When PowerShell is unavailable, call the paired `.sh` wrappers; always source the activation script beforehand.*

### G) Coding & Documentation Standards

* **Error handling:** Binaries favor `anyhow::{Result, Context}` with `.context()` on each fallible hop; libraries define typed errors via `thiserror::Error` to encode IO/operation failures.
* **Async model:** `tokio` runtime (`use tokio::{fs, time};`) is the default for I/O-heavy flows; avoid blocking calls inside async contexts.
* **Agent traits:** Agents implement metadata accessors plus async `initialize/execute_task/shutdown` operations returning structured `TaskResult` types; keep them `Send + Sync` ready for swarm orchestration.
* **Module hygiene:** Use `mod.rs` (or inline modules) for hierarchy clarity, keep public APIs documented with Rustdoc (`///` blocks describing args/returns/errors/examples).
* **Dependency governance:** Prefer workspace-level versions for `tokio`, `anyhow`, `thiserror`, `serde`, `serde_json`, `tracing`, and document new crates (purpose, license, verification run) inside the Evidence Ledger.
* **Documentation cadence:** Every new public API increments inline docs plus relevant `README`s; large features warrant entries in `docs/` (architecture/operations) and updates to `WORKSPACE_MEMORY.md`.

### H) Workflow Playbooks

* **Feature development:** `git checkout -b feature/<slug>` → implement → `cargo build/test/clippy` (and pnpm equivalents when touching JS) → Conventional Commit with archival notes → push branch.
* **Fork processing:** Stage fork under `crc/drop-in/incoming/forks/<name>/`, run `detect-forks.ps1 -Mode process`, review `fork/<name>` branch, run workspace build/tests, merge, then compress originals into `crc/archive/forks/` with ledger updates.
* **Agent restoration:** Inspect `_backup` implementation, port into live `agents/src/implementations/`, register modules + factory, run `cargo test -p noa_agents`, document behavior, and ensure Evidence Ledger references the restored agent.
* **AI assistant prompts:** When asked to build, test, process forks, add agents, or fix builds, follow the command snippets above and capture outputs for Truth Gate evidence.

### I) Security, Quality & Metrics

* **Always:** activate portable Cargo, run full workspace builds/tests pre-commit, document APIs, follow Rust 2021, prefer async I/O, add `.context()`, keep `cargo clippy` clean.
* **Never:** mix Windows/WSL paths in a single session, skip builds before commits, leave commented dead code, use `.unwrap()` in production, commit `target/`, retain live fork code post-archive, block async loops with sync I/O, ignore warnings.
* **Fork security:** scan for malware/secrets, verify licenses, audit dependencies, isolate tests, never auto-run external code, and compress/archive originals immediately after integration.
* **Code review checklist:** no secrets, validate inputs, no panics in prod, safe error propagation, guard against injection, trust-only dependencies, license compatibility.
* **Quality/perf targets:** coverage >80%, full build <5 min, tests <10 min, deploy <5 min, CI-to-prod <20 min, p95 response <100 ms, change failure rate <5%, MTTR <5 min.

### J) Troubleshooting & Support

* **Common fixes:**
  * `cargo: command not found` → re-run activation script.
  * Language tooling unavailable → reactivate portable Cargo and rely on built-in commands (no VS Code extensions such as rust-analyzer are used here).
  * Build/test failures → run `cargo build --workspace --verbose` or `cargo test --workspace -- --nocapture` to capture context.
  * Fork not detected → re-run `detect-forks.ps1 -Mode process -ForkName "name"` to initialize metadata.
* **Debug helpers:** `cargo check -p <crate> --verbose`, `cargo tree`, `cargo outdated`.
* **Support path:** consult this policy, `WORKSPACE_MEMORY.md`, component READMEs, documentation map, then escalate via task comments with exact command, error, cwd, tool versions, and expectation vs. outcome.


## Testing

⚠️ Tests not run (documentation-only change).

