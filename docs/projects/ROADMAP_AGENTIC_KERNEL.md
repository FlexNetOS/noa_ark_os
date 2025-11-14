# NOA ARK OS – Comprehensive Agentic Kernel Roadmap

**Version:** 1.1.0 (2025-11-14)  
**Status:** Active  
**Purpose:** Define the evolution path from current OS foundation to a fully autonomous, AI-first, kernel-sovereign operating system with humanless capability.

**Changelog v1.1.0:**
* ⭐ Added Goal 18: CLI-First Interface
* ⭐ Added Phase 0.5: CLI-First Foundation (CRITICAL PATH)
* ⭐ Expanded Phase 4: CLI Expansion & Registry Completion (40+ commands)
* ⭐ Documented IDE extension deprecation strategy
* ⭐ CLI-First Philosophy section added

---

## Top-Level Goals (North Star)

1. **AI-First Operating Core** – Kernel designed for AI agents, not humans
2. **Autonomous Self-Improvement** – System evolves itself safely
3. **Single North Star** – Integrity, Reversibility, Capability Gain as one measure
4. **Kernel Sovereignty** – Kernel controls all resources, policies, and trust
5. **AI-Native (not human-centric)** – Optimized for machine reasoning and execution
6. **Unified Machine-Readable Registry** – Single source of truth for all capabilities
7. **Kill Context Rot & Token Waste** – Eliminate redundancy and inefficiency
8. **Dynamic Trust & Adaptive Access** – Permissions adjust based on behavior
9. **Full Reversibility & Integrity** – Every change can be rolled back
10. **Policies as Code** – All rules machine-readable and enforceable
11. **Total Observability & Auditability** – Complete system transparency
12. **System-Health Rewarding** – Incentivize stability and efficiency
13. **Multi-Layer World Model** – Comprehensive system understanding
14. **Environment Portability** – Deploy anywhere (single host, edge, air-gapped, cloud)
15. **Supply-Chain Security & Provenance** – Verifiable builds and dependencies
16. **Proactive (not reactive)** – Anticipate and prevent issues
17. **Humanless Autonomy** – Capable of self-operation
18. **CLI-First Interface** – ⭐ **NEW** All capabilities accessible via unified CLI for both humans and AI agents; IDE extensions deprecated

---

## CLI-First Philosophy ⭐

### Hermetic Execution Targets (NEW)

To keep the CLI transformation self-contained, the dedicated checklist in [`docs/projects/HERMETIC_TARGETS.md`](HERMETIC_TARGETS.md) tracks the required portable toolchains, offline pipeline guarantees, gateway/profile enforcement, and Truth Gate evidence artifacts. Phase owners should update that catalog whenever Phase 0.5–P10 work lands so hermetic behavior stays measurable.

**Core Principle:** The `noa` CLI is the **primary interface** for all system operations, designed for both human operators and AI agents.

### Why CLI-First?

1. **Universal Access** – Works in any terminal, any environment
2. **AI-Native** – Machine-parsable output (JSON/YAML) by default
3. **IDE-Agnostic** – No editor lock-in or plugin dependencies
4. **Automation-Friendly** – Easy to script and integrate
5. **Faster Iteration** – No IDE restart required
6. **Lower Barriers** – Just a terminal, nothing else needed
7. **CI/CD Native** – Perfect for pipelines and automation
8. **Consistent Interface** – Same commands everywhere

### Interface Hierarchy

```
┌──────────────────────────────────────────┐
│  PRIMARY: noa CLI                        │ ← Humans & AI Agents
│  • All features available                │
│  • Machine-readable output               │
│  • Interactive TUI modes                 │
└──────────────────────────────────────────┘
                 ↓
┌──────────────────────────────────────────┐
│  SECONDARY: HTTP/gRPC API                │ ← Remote Services
│  • Mirrors CLI commands                  │
│  • OpenAPI/proto specs                   │
└──────────────────────────────────────────┘
                 ↓
┌──────────────────────────────────────────┐
│  LEGACY: IDE Extensions (Deprecated)     │ ← Thin wrappers only
│  • Wraps noa CLI commands                │
│  • No unique features                    │
│  • Migration path documented             │
└──────────────────────────────────────────┘
```

### Design Principles

* **JSON by Default** – All output structured for parsing
* **Human-Friendly Fallback** – Add `--tui` for interactive mode
* **Streaming Support** – Real-time updates with `--watch`
* **Composability** – Commands pipe together (Unix philosophy)
* **Self-Documenting** – `--help` and `--explain` on everything
* **Token-Efficient** – Minimal output with `--minimal` flag

---

## PHASE 0 — Repo Foundation & Dead Code Quarantine

**Serves Goals:** 1, 5, 7, 9, 10, 11, 16  
**Objective:** Zero zombie code; instant reversibility.  
**Status:** ✅ Complete (accepted)

### Tasks

* **P0-T1:** Create `/archive/quarantine/<component>@<commit>/` with `README.md` + `status.yaml` (owner, intended interface, reintegration criteria)
* **P0-T2:** CI fails if quarantined code is referenced by build or tests
* **P0-T3:** 90-day auto-archive job to cold storage
* **P0-T4:** Add `make snapshot` / `make rollback` (pre/post hooks)
* **P0-T5:** Extend `AGENT.md` with "Dead Code Handling" + "Reintegration Procedure"

### Acceptance Criteria

- [x] Quarantine directory structure exists with documented format
- [x] CI enforces no-reference policy for quarantined code
- [x] Auto-archive automation runs successfully on schedule
- [x] Snapshot/rollback commands work end-to-end
- [x] AGENT.md updated with quarantine procedures

---

## PHASE 0.5 — CLI-First Foundation ⭐ **PRIORITY 0**

**Serves Goals:** 1, 5, 6, 7, 10, 11, 14, 18  
**Objective:** Unified CLI as primary interface for humans and AI agents; deprecate IDE extensions.  
**Status:** 🚀 **CRITICAL PATH**

### Philosophy

**CLI > IDE Extensions > GUI**

* **Primary Interface:** `noa` CLI provides all capabilities
* **Machine-Readable:** JSON/YAML output for agent consumption
* **Human-Friendly:** Interactive TUI and streaming modes
* **IDE-Agnostic:** Works in any terminal, no editor lock-in
* **AI-Native:** Designed for agent invocation first, human second

### Tasks

* **P0.5-T1:** Create `cli/noa` core binary with subcommand architecture
  * `noa kernel` – Kernel operations (start, stop, status, logs)
  * `noa world` – World model operations (verify, fix, graph, diff)
  * `noa registry` – Registry operations (list, describe, search, validate)
  * `noa trust` – Trust operations (score, audit, history, thresholds)
  * `noa snapshot` – Snapshot operations (create, list, rollback, verify)
  * `noa agent` – Agent operations (spawn, list, kill, logs, metrics)
  * `noa policy` – Policy operations (validate, apply, test, dry-run)
  * `noa sbom` – SBOM operations (generate, verify, sign, audit)
  * `noa pipeline` – Pipeline operations (run, status, logs, artifacts)
  * `noa profile` – Profile operations (switch, list, validate, diff)

* **P0.5-T2:** Implement dual-mode output
  * `--json` – Machine-parsable JSON (default for scripts)
  * `--yaml` – Human-readable YAML
  * `--tui` – Interactive terminal UI (with streaming)
  * `--watch` – Real-time updates (for monitoring)
  * `--format=table|tree|json|yaml` – Flexible formatting

* **P0.5-T3:** Add streaming and interactive capabilities
  * `noa agent spawn --watch` – Stream agent output live
  * `noa pipeline run --interactive` – Prompt for confirmations
  * `noa trust score --explain` – Show reasoning
  * `noa world graph --interactive` – Navigate graph in TUI

* **P0.5-T4:** CLI-based development workflow
  * `noa dev init` – Initialize workspace
  * `noa dev test` – Run tests with live results
  * `noa dev build --watch` – Continuous build
  * `noa dev deploy --dry-run` – Preview deployment
  * `noa dev debug <agent>` – Attach debugger to agent

* **P0.5-T5:** AI agent integration
  * `noa agent invoke <tool> --args <json>` – Direct tool invocation
  * `noa query <natural-language>` – Natural language to CLI translation
  * `noa plan <task>` – Generate execution plan
  * `noa exec <plan-id>` – Execute plan with approval gates
  * Output always includes `exit_code`, `result`, `metadata`

* **P0.5-T6:** Deprecate IDE extensions
  * Document migration path from VS Code/JetBrains extensions to CLI
  * Create `noa ide migrate` command to help users transition
  * Mark IDE extensions as "legacy" in documentation
  * Focus all future development on CLI capabilities
  * IDE extensions become thin wrappers around `noa` CLI

* **P0.5-T7:** Shell completion and scripting
  * Bash, Zsh, Fish, PowerShell completion
  * `noa completion install` – Auto-install completions
  * `noa script generate <workflow>` – Generate bash/python scripts
  * Environment variable configuration (`NOA_*` vars)

### Acceptance Criteria

- [ ] `noa` CLI compiles and runs on Linux, macOS, Windows
- [ ] All 10+ subcommands implemented with help text
- [ ] JSON output parsable by jq and automation tools
- [ ] TUI mode works in any terminal (including SSH)
- [ ] Shell completions install successfully
- [ ] AI agents can invoke all commands programmatically
- [ ] Migration guide from IDE extensions published
- [ ] Zero features exclusive to IDE extensions

### Value Increase

**Before (IDE-centric):**
* Developers locked into specific editors
* AI agents can't easily access tools
* Inconsistent interfaces across platforms
* Limited scripting/automation capabilities

**After (CLI-first):**
* Universal access from any terminal
* AI agents have first-class CLI access
* Consistent interface everywhere
* Full automation via scripts
* Faster iteration (no IDE restart)
* Lower barrier to entry (just terminal)
* Better CI/CD integration

---

## PHASE 1 — Kernel-First Baseline

**Serves Goals:** 1, 3, 4, 5, 9, 10, 11, 14  
**Objective:** Kernel as sovereign controller.  
**Status:** 🔨 In Progress

### Tasks

* **P1-T1:** Create `kernel/` bundle: policy engine, scope/token service, snapshot/rollback, registry loader, event bus
* **P1-T2:** Add `make kernel` + `make image` separation
* **P1-T3:** Kernel host-control APIs (env takeover primitives)
* **P1-T4:** SBOM split preparation (see Phase 8)

### Acceptance Criteria

- [ ] Kernel bundle compiles and initializes independently
- [ ] Token service issues and validates capability tokens
- [ ] Host-control APIs demonstrated in isolation
- [ ] SBOM hooks ready for Phase 8 integration

### Current Progress

- ✅ Core OS skeleton established (Rust)
- ✅ Basic IPC mechanism implemented
- ✅ Process management foundation
- 🔨 Policy engine integration in progress
- 📋 Token service design pending

---

## PHASE 2 — North Star Implementation

**Serves Goals:** 3, 4, 8, 9, 10, 11, 12, 16, 17  
**Objective:** Internal compass + dynamic access.  
**Status:** 📋 Planned

### Tasks

* **P2-T1:** Add `north_star.deflex.json` (metrics, thresholds, escalation/de-escalation)
* **P2-T2:** Implement `kernel/scorekeeper/` to compute Integrity/Reversibility/Capability and expose `/v1/trust`
* **P2-T3:** Orchestrator gates scopes by trust; violations shrink scopes + trigger repair

### Acceptance Criteria

- [ ] north_star.deflex.json schema defined and validated
- [ ] Scorekeeper computes trust scores with test coverage
- [ ] Trust-based scope gating demonstrated in integration tests
- [ ] Auto-repair triggers on trust violations

---

## PHASE 3 — World Model & Contract Tests

**Serves Goals:** 1, 3, 4, 5, 7, 10, 11, 13, 16  
**Objective:** Machine-readable source of truth.  
**Status:** 📋 Planned

### Tasks

* **P3-T1:** Define `world.graph.schema.json` + seed `world.graph.json`
* **P3-T2:** `kernel/reconciler/` detects drift; opens PRs or blocks merges
* **P3-T3:** Contract-test-first rule for all integrations
* **P3-T4:** `make world-verify` / `make world-fix`

### Acceptance Criteria

- [ ] `kernel/world/world.graph.schema.json` and seed graph validate against repository state
- [ ] Reconciler detects and reports drift accurately with remediation plans
- [ ] Contract tests cover clean + drift scenarios using fixtures in `tests/world_model/`
- [ ] `make world-verify` / `make world-fix` wrap the reconciler and fail CI on drift
- [ ] CLI `noa world verify` / `noa world fix` documented for Phase 0.5 rollout

---

## PHASE 4 — CLI Expansion & Registry Completion

**Serves Goals:** 5, 6, 7, 8, 9, 10, 11, 14, 18  
**Objective:** Complete CLI suite with full registry integration and advanced capabilities.  
**Status:** 📋 Planned  
**Priority:** ⭐ High (builds on Phase 0.5)

### Tasks

* **P4-T1:** Complete `registry/tools.registry.json` with full metadata
  * Tool name, version, params/types
  * Side-effects, safety level, reversibility flag
  * Token/resource budgets, rate limits
  * Dependencies, prerequisites, conflicts
  * CLI command mappings

* **P4-T2:** Expand `noa` CLI with advanced commands
  * **Observability:**
    * `noa observe metrics` – Live system metrics
    * `noa observe traces` – Distributed tracing
    * `noa observe logs --follow` – Live log streaming
    * `noa observe health` – Health check dashboard
  
  * **Automation:**
    * `noa auto schedule <task> <cron>` – Schedule tasks
    * `noa auto trigger <event> <action>` – Event-driven automation
    * `noa auto repair --enable` – Enable auto-repair
    * `noa auto budget set <limits>` – Set resource budgets
  
  * **Collaboration:**
    * `noa share snapshot <id>` – Share snapshot with team
    * `noa import snapshot <url>` – Import shared snapshot
    * `noa review plan <id>` – Review execution plan
    * `noa approve plan <id>` – Approve for execution
  
  * **Analysis:**
    * `noa analyze performance` – Performance analysis
    * `noa analyze security` – Security audit
    * `noa analyze dependencies` – Dependency analysis
    * `noa analyze drift` – Configuration drift detection

* **P4-T3:** HTTP/gRPC API with OpenAPI/proto specs
  * Mirror all CLI commands as HTTP endpoints
  * Generate OpenAPI 3.0 spec from CLI definitions
  * gRPC services with `.proto` files
  * WebSocket support for streaming operations
  * Server-sent events for real-time updates

* **P4-T4:** CLI plugin system
  * `noa plugin install <name>` – Install community plugins
  * `noa plugin develop` – Plugin development workflow
  * Plugin SDK with Rust/Python/Go support
  * Plugin registry and discovery
  * Sandboxed execution with capability tokens

* **P4-T5:** Enhanced machine interfaces
  * `noa query --sql` – SQL-like querying of system state
  * `noa query --jsonpath` – JSONPath queries
  * `noa query --graphql` – GraphQL endpoint
  * `noa export --format=csv|parquet` – Data export
  * `noa import --format=json|yaml` – Bulk data import

* **P4-T6:** AI agent optimization
  * `noa agent context --minimal` – Minimal context for token efficiency
  * `noa agent batch <commands.json>` – Batch command execution
  * `noa agent session start` – Start persistent session
  * `noa agent cache warm` – Pre-cache frequent queries
  * Token usage tracking and optimization hints

* **P4-T7:** Wrap legacy MCP adapter under kernel tokens
  * MCP protocol support via `noa mcp` subcommand
  * Token-based authentication for all MCP operations
  * Registry-only execution (no arbitrary commands)

### Acceptance Criteria

- [ ] Registry contains 100% of available tools/capabilities
- [ ] All CLI commands produce consistent JSON output
- [ ] HTTP/gRPC endpoints match OpenAPI/proto specs
- [ ] Plugin system allows third-party extensions
- [ ] SQL/JSONPath/GraphQL queries work correctly
- [ ] AI agents achieve <50ms average command latency
- [ ] MCP adapter uses kernel token authentication
- [ ] CLI covers 100% of IDE extension features (proving deprecation)

### Value Increase

**Expanded Capabilities:**
* 40+ CLI commands (up from 10)
* Plugin ecosystem for community contributions
* Multiple query languages (SQL, JSONPath, GraphQL)
* Real-time streaming and monitoring
* Advanced automation and scheduling
* Comprehensive observability

**AI Agent Benefits:**
* Minimal context mode saves 80% tokens
* Batch operations reduce round-trips
* Persistent sessions improve performance
* Query optimization for faster results

---

## PHASE 5 — Gateway Slimming & Capability Tokens

**Serves Goals:** 1, 4, 6, 8, 9, 11, 14  
**Objective:** Gateway = router; kernel = authority.  
**Status:** 📋 Planned

### Tasks

* **P5-T1:** Gateway reads kernel-issued capability tokens (FS scopes, net egress, rate limits)
* **P5-T2:** Hot-swappable adapters in `extensions/` with declared capability claims
* **P5-T3:** Update `AGENT.md`: registry-only, snapshot-before-danger, tokenized scopes

### Acceptance Criteria

- [ ] Gateway validates capability tokens for all requests
- [ ] Extension hot-swap works without restart
- [ ] AGENT.md documents token usage patterns
- [ ] Integration tests verify token enforcement

---

## PHASE 6 — Performance & Retrieval Discipline

**Serves Goals:** 1, 5, 7, 8, 11, 12, 16  
**Objective:** Low latency; minimal tokens.  
**Status:** 📋 Planned

### Tasks

* **P6-T1:** `kernel/indexer/` builds code+config graph (AST/ownership/dep)
* **P6-T2:** Hierarchical memory: `memory/long_term/*.json`, `memory/session/<id>.json`
* **P6-T3:** Incremental tests + streaming diffs; CBOR/MessagePack internally; JSON at rest
* **P6-T4:** Penalize large/bloated contexts via Scorekeeper

### Acceptance Criteria

- [ ] Indexer produces accurate dependency graphs
- [ ] Memory hierarchy tested with realistic workloads
- [ ] Binary protocols reduce bandwidth by >50%
- [ ] Scorekeeper penalizes context bloat measurably

---

## PHASE 7 — Reward System (System-Health First)

**Serves Goals:** 3, 8, 9, 11, 12, 16, 17  
**Objective:** Incentives aligned to stability & growth.  
**Status:** 📋 Planned

### Tasks

* **P7-T1:** Scorekeeper produces weighted rewards on: coverage↑, flake↓, tokens↓, latency↓, successful rollbacks
* **P7-T2:** Penalize: missing snapshots, budget overruns, duplicate retrieval
* **P7-T3:** Persist deltas in `/metrics/reward_history.json`

### Acceptance Criteria

- [ ] Reward calculations documented and tested
- [ ] Penalties reduce problematic behaviors in simulation
- [ ] Reward history tracked over time with trends
- [ ] Dashboard visualizes reward metrics

---

## PHASE 8 — SBOM Split & Supply-Chain Integrity

**Serves Goals:** 1, 9, 10, 11, 14, 15  
**Objective:** Verifiable kernel + reproducible extensions.  
**Status:** 📋 Planned

### Tasks

* **P8-T1:** Generate `SBOM.kernel.cdx.json` and `SBOM.extensions.cdx.json` + `SBOM.manifest.json`
* **P8-T2:** Sign artifacts; `make verify` reproduces kernel build and compares hashes
* **P8-T3:** SLSA-style attestations

### Acceptance Criteria

- [ ] SBOM files generated automatically on build
- [ ] Signature verification passes in CI
- [ ] Reproducible builds verified on clean systems
- [ ] SLSA provenance documents complete

---

## PHASE 9 — Deployment Profiles (Drop-In Everywhere)

**Serves Goals:** 1, 4, 8, 11, 14, 16  
**Objective:** Behavior via profiles, not forks.  
**Status:** 📋 Planned

### Tasks

* **P9-T1:** `/server/profiles/`: `single_host`, `air_gapped`, `devcontainer`, `edge_lite`
* **P9-T2:** Each profile declares tools allowed, net egress, budgets, storage roots
* **P9-T3:** Kernel policy loads profile and issues matching tokens

### Acceptance Criteria

- [ ] All four profiles defined with schemas
- [ ] Profile switching works without code changes
- [ ] Token issuance respects profile constraints
- [ ] Air-gapped profile tested in isolated environment

---

## PHASE 10 — Tests, CI/CD, and Audit Trail (Machine-First)

**Serves Goals:** 1, 5, 8, 9, 10, 11, 14, 16, 17  
**Objective:** CI/CD runs locally first; GitHub is just a remote mirror.  
**Status:** 🔨 In Progress

### Tasks

* **P10-T1:** Replace web-centric CI with **machine-first pipeline**:
  * `make pipeline.local` (runs full chain locally or on self-hosted runner): world-verify → build → sbom → tests → scorekeeper → package → sign
  * GitHub Actions minimal wrapper that shells into the same pipeline; if self-hosted runner present, run there; otherwise containerized local emulation
* **P10-T2:** Mandatory checks: contract tests, policy tests, rollback drill, performance budget, SBOM verify
* **P10-T3:** Publish audit bundle (snapshot ID, SBOM hashes, diff summary, trust score)

### Acceptance Criteria

- [ ] make pipeline.local completes successfully offline
- [ ] GitHub Actions workflow uses thin shim pattern
- [ ] All mandatory checks pass in CI
- [ ] Audit bundles published for every merge

### Current Progress

- ✅ CI workflow exists with basic checks
- ✅ Makefile targets established
- 🔨 Pipeline.local implementation in progress
- 📋 Audit bundle format pending

---

## PHASE 11 — Documentation & Agent Policy

**Serves Goals:** 3, 4, 5, 8, 9, 10, 11, 14  
**Objective:** One set of rules for humans and agents—machine readable first.  
**Status:** 🔨 In Progress

### Tasks

* **P11-T1:** Update `AGENT.md`: Heal-Don't-Harm; registry-only; snapshots; tokens; examples
* **P11-T2:** `docs/guides/AGENTIC_OS_GUIDE.md` for operators
* **P11-T3:** Architecture diagrams updated for kernel sovereignty and tokenized gateway

### Acceptance Criteria

- [ ] AGENT.md includes all Phase 1-10 patterns
- [ ] Operator guide covers common scenarios
- [ ] Architecture diagrams reflect current state
- [ ] Documentation CI validates completeness

### Current Progress

- ✅ AGENT.md v1.0.0 active
- ✅ Basic architecture documented
- 🔨 Integration examples in progress
- 📋 Operator guide pending

---

## PHASE 12 — Intelligent Growth & Self-Maintenance

**Serves Goals:** 2, 3, 8, 9, 11, 12, 16, 17  
**Objective:** Self-diagnosis, self-refactor, self-repair.  
**Status:** 📋 Planned

### Tasks

* **P12-T1:** Self-repair agent proposes/lands patches gated by trust
* **P12-T2:** Reflection API `/v1/self/status` surfaces drift, hot paths, budget offenders
* **P12-T3:** Periodic plan: "shorten feedback cycles; reduce hot-spot latency; consolidate duplicated logic"

### Acceptance Criteria

- [ ] Self-repair agent demonstrates successful patch flow
- [ ] Reflection API returns actionable insights
- [ ] Improvement plans generated automatically
- [ ] Trust gates prevent harmful changes

---

## PHASE 13 — Humanless Autonomy

**Serves Goals:** 2, 3, 8, 9, 11, 12, 16, 17  
**Objective:** Show it can run itself safely.  
**Status:** 📋 Planned

### Tasks

* **P13-T1:** Auto-PR + auto-merge when trust ≥ threshold and policy suite is green
* **P13-T2:** Scheduled rollback simulations (fire drills)
* **P13-T3:** Auto-sign, publish, and verify release bundle

### Acceptance Criteria

- [ ] Auto-merge completes end-to-end without intervention
- [ ] Fire drills demonstrate successful recovery
- [ ] Release automation produces verifiable artifacts
- [ ] System operates autonomously for 72+ hours

---

## End-Cap Automation Tasks

### Error Auto-Triage

**Purpose:** Classify and prepare fixes for common errors automatically.

**Implementation:**
* Event rules → classify errors
* Open issues with minimal repro
* Attach patches if confidence ≥ threshold

**Status:** 📋 Planned

### Auto-Fixer Agents

**Purpose:** Handle routine maintenance automatically.

**Categories:**
* Lint issues
* Type errors
* Import optimization
* Flaky test stabilization
* Small refactors

**Gating:** snapshot + incremental tests + policy

**Status:** 📋 Planned

### Budget Guardians

**Purpose:** Prevent resource waste.

**Actions:**
* Kill token explosions
* Penalize latency spikes
* Rewrite plans with stricter bounds

**Status:** 📋 Planned

### Local-First CI Harness

**Purpose:** Make CI independent of cloud services.

**Features:**
* `pipeline.local` authority
* GitHub Action = thin shim
* Can be removed if not needed

**Status:** 🔨 In Progress

### Mandatory Offline Mode

**Purpose:** Support air-gapped deployments.

**Implementation:**
* Network-blocked profile
* Cached indices and mirrors
* Reproducible results

**Status:** 📋 Planned

---

## Machine-First CI/CD Blueprint

### Philosophy

**Local First:** All CI/CD logic runs locally or on self-hosted infrastructure. GitHub Actions is merely a thin wrapper that invokes local tooling.

**Offline Capable:** System can build, test, and deploy without internet access using cached dependencies and local indices.

**Reproducible:** Same inputs produce identical outputs regardless of environment.

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                 make pipeline.local                     │
│  (Authoritative pipeline implementation)                │
└────────────────────────┬────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
    ┌────▼─────┐    ┌───▼────┐    ┌────▼─────┐
    │  Local   │    │  Self  │    │  GitHub  │
    │  Dev     │    │ Hosted │    │  Actions │
    │  Machine │    │ Runner │    │ (Shim)   │
    └──────────┘    └────────┘    └──────────┘
```

### Pipeline Stages

1. **world-verify** – Validate world model consistency
2. **build** – Compile all workspace components
3. **sbom** – Generate software bill of materials
4. **tests** – Run full test suite
5. **scorekeeper** – Calculate trust scores
6. **package** – Create deployment artifacts
7. **sign** – Cryptographically sign artifacts

### Makefile Targets

```makefile
# Run complete pipeline locally
make pipeline.local

# Individual stages
make world-verify
make kernel
make sbom
make tests
make scorekeeper
make package
make sign

# Utilities
make snapshot
make rollback
make verify
make publish-audit
```

### GitHub Actions Integration

**File:** `.github/workflows/pipeline.yml`

**Strategy:** Minimal wrapper that delegates to `make pipeline.local`

**Fallback:** Container-based execution for standard hosted runners

See implementation in Phase 10 tasks.

---

## Progress Tracking

### Legend

* ✅ **Complete** – Fully implemented and tested
* 🔨 **In Progress** – Active development
* 📋 **Planned** – Designed but not started
* 🚫 **Blocked** – Waiting on dependencies

### Phase Status Summary

| Phase | Status | % Complete | Blockers |
|-------|--------|------------|----------|
| **P0.5 – CLI-First Foundation** ⭐ | 📋 Planned | 0% | **CRITICAL PATH** |
| P0 – Repo Foundation | 📋 Planned | 0% | None |
| P1 – Kernel Baseline | 🔨 In Progress | 40% | Token service design |
| P2 – North Star | 📋 Planned | 0% | P1 |
| P3 – World Model | 📋 Planned | 0% | P1 |
| **P4 – CLI Expansion** ⭐ | 📋 Planned | 0% | **P0.5** |
| P5 – Gateway Tokens | 📋 Planned | 0% | P1, P4 |
| P6 – Performance | 📋 Planned | 0% | P3, P4 |
| P7 – Reward System | 📋 Planned | 0% | P2, P6 |
| P8 – SBOM & Security | 📋 Planned | 0% | P1 |
| P9 – Profiles | 📋 Planned | 0% | P5, P8 |
| P10 – CI/CD & Audit | 🔨 In Progress | 30% | None |
| P11 – Documentation | 🔨 In Progress | 60% | P1-P10 |
| P12 – Self-Maintenance | 📋 Planned | 0% | P2, P7, P10 |
| P13 – Humanless Autonomy | 📋 Planned | 0% | P12 |

### Overall Progress: ~15%

**Current Focus:** **Phase 0.5 (CLI-First)**, Phase 1 (Kernel Baseline), Phase 10 (CI/CD)

---

## Dependencies & Prerequisites

### Phase Dependencies

```
P0 (Foundation)
  │
  ├─→ P0.5 (CLI-First) ⭐ CRITICAL PATH
  │     │
  │     ├─→ P1 (Kernel) ────────┬─→ P2 (North Star) ─→ P7 (Rewards) ─→ P12 (Self-Maintain) ─→ P13 (Autonomy)
  │     │                       │
  │     │                       ├─→ P3 (World Model) ─→ P4 (CLI Expansion) ─→ P6 (Performance)
  │     │                       │                           │
  │     │                       └─→ P8 (SBOM) ──────────────┤
  │     │                                                    │
  │     └────────────────────────────────────────────────────├─→ P5 (Gateway) ─→ P9 (Profiles)
  │                                                          │
  └─→ P10 (CI/CD) ───────────────────────────────────────────┘
       │
       └─→ P11 (Docs) ─────────────────────────────────────────────────────┘

KEY:
⭐ = Critical Path (P0.5 CLI-First blocks P4 and influences all phases)
─→ = Depends on
```

### External Dependencies (Minimal)

* **Rust toolchain** – 1.70+ (portable, bundled)
* **Development tools** – Make, Git (standard POSIX)
* **Optional:** Container runtime for CI emulation

**Note:** All runtime dependencies bundled in repository per self-contained principle.

---

## Success Metrics

### Phase 1-5 (Foundation)

* [ ] Kernel boots and manages all resources independently
* [ ] Trust scores computed and enforced
* [ ] World model validates repository state
* [ ] Registry complete with 100% coverage
* [ ] Gateway uses only kernel tokens

### Phase 6-10 (Performance & Operations)

* [ ] Context retrieval < 100ms (p95)
* [ ] SBOM generation < 30s
* [ ] CI pipeline < 15 min local
* [ ] 100% offline capable
* [ ] Audit trail complete and queryable

### Phase 11-13 (Autonomy)

* [ ] Self-repair handles 80% of issues
* [ ] Reflection API provides actionable insights
* [ ] Auto-merge success rate > 95%
* [ ] 72-hour autonomous operation proven
* [ ] Fire drills succeed 100%

---

## CLI Command Reference (Phase 0.5 & 4)

### Core Commands (Phase 0.5)

```bash
# Kernel Operations
noa kernel start                    # Start kernel
noa kernel stop                     # Stop kernel
noa kernel status --json            # Get status (JSON output)
noa kernel logs --follow            # Stream logs

# World Model
noa world verify                    # Verify consistency
noa world fix --auto                # Auto-repair
noa world graph --interactive       # Navigate in TUI
noa world diff <snapshot>           # Compare states

# Registry
noa registry list                   # List all tools
noa registry describe <tool>        # Get tool details
noa registry search <query>         # Search registry
noa registry validate               # Validate registry

# Trust & Security
noa trust score                     # Get current score
noa trust audit --history           # Audit history
noa trust thresholds set <rules>    # Set thresholds

# Snapshots
noa snapshot create <name>          # Create snapshot
noa snapshot list                   # List snapshots
noa snapshot rollback <id>          # Rollback
noa snapshot verify <id>            # Verify integrity

# Agents
noa agent spawn <type> --watch      # Spawn with live output
noa agent list --json               # List agents
noa agent kill <id>                 # Kill agent
noa agent logs <id> --follow        # Stream agent logs
noa agent metrics <id>              # Get metrics

# Policy
noa policy validate <file>          # Validate policy
noa policy apply <file>             # Apply policy
noa policy test <file>              # Test policy
noa policy dry-run <file>           # Dry run

# SBOM
noa sbom generate                   # Generate SBOM
noa sbom verify                     # Verify signatures
noa sbom sign                       # Sign SBOM
noa sbom audit                      # Security audit

# Pipeline
noa pipeline run --interactive      # Run pipeline
noa pipeline status                 # Pipeline status
noa pipeline logs --follow          # Stream logs
noa pipeline artifacts              # List artifacts

# Profile
noa profile switch <name>           # Switch profile
noa profile list                    # List profiles
noa profile validate <name>         # Validate profile
noa profile diff <a> <b>            # Compare profiles
```

### Advanced Commands (Phase 4)

```bash
# Observability
noa observe metrics                 # Live metrics
noa observe traces                  # Distributed tracing
noa observe logs --follow           # Log streaming
noa observe health                  # Health dashboard

# Automation
noa auto schedule <task> <cron>     # Schedule task
noa auto trigger <event> <action>   # Event automation
noa auto repair --enable            # Enable auto-repair
noa auto budget set <limits>        # Set budgets

# Collaboration
noa share snapshot <id>             # Share snapshot
noa import snapshot <url>           # Import snapshot
noa review plan <id>                # Review plan
noa approve plan <id>               # Approve plan

# Analysis
noa analyze performance             # Performance analysis
noa analyze security                # Security audit
noa analyze dependencies            # Dependency analysis
noa analyze drift                   # Drift detection

# Development
noa dev init                        # Initialize workspace
noa dev test --watch                # Continuous testing
noa dev build --watch               # Continuous build
noa dev deploy --dry-run            # Preview deployment
noa dev debug <agent>               # Attach debugger

# AI Agent Interface
noa agent invoke <tool> --args <json>  # Invoke tool
noa query <natural-language>        # NL to CLI
noa plan <task>                     # Generate plan
noa exec <plan-id>                  # Execute plan

# Query Languages
noa query --sql "SELECT * FROM agents WHERE status='active'"
noa query --jsonpath "$.kernel.status"
noa query --graphql "{agents{id status}}"

# Data Export/Import
noa export --format=csv             # Export as CSV
noa export --format=parquet         # Export as Parquet
noa import --format=json <file>     # Import JSON

# Plugin System
noa plugin install <name>           # Install plugin
noa plugin develop                  # Dev workflow
noa plugin list                     # List plugins
```

### Output Modes

```bash
# Machine-readable (default for AI agents)
noa <command> --json                # JSON output
noa <command> --yaml                # YAML output
noa <command> --minimal             # Minimal context

# Human-friendly
noa <command> --tui                 # Interactive TUI
noa <command> --watch               # Live updates
noa <command> --format=table        # Table format

# Efficiency
noa <command> --explain             # Show reasoning
noa <command> --help                # Full help
noa <command> --dry-run             # Preview only
```

---

## Related Documentation

* **[AGENT.md](../../AGENT.md)** – Agent execution policy
* **[ROADMAP.md](ROADMAP.md)** – Traditional feature roadmap
* **[roadmap_noa_ark_os.json](roadmap_noa_ark_os.json)** – Machine-readable spec
* **[OVERVIEW.md](../../OVERVIEW.md)** – System architecture
* **[WORKSPACE_MEMORY.md](../../WORKSPACE_MEMORY.md)** – Project context

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2025-11-14 | 1.0.0 | Initial comprehensive agentic kernel roadmap created |

---

**Last Updated:** 2025-11-14  
**Maintained By:** NOA ARK OS Core Team  
**Status:** Active Development
