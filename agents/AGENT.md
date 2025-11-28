# AGENT.md

**AGENT_POLICY_VERSION:** 1.0.0
**Status:** Active
**Scope:** Entire RTT-GATEWAY repository (agents, providers, tools, skills, automation, runtime, CI, UI).
**Authoritative Source:** This file. All provider instruction files must defer to this document.
**GOAL:** Gateway designed to drop into any OS or REPO. Independent, kernel first. Central smart Gateway for routing and connecting everythings, All paths. Autonoumously detects paths/ connections then reroutes through gateway with real CLI code read/write/edit. CLI first. Dynamic UI/UX connected by user login. Set-up as client then inverts to HOST. Adaptive, Full-end-to-end ANGENTICOS automation. Replaces human in the loop with AI AGENTS. Offline first, online only with feature flag designed as optional but non restrictive. Heal codebase, never downgrade or delete. Archive everything with ledger. Cross-platform (Linux/Mac/Win). Triple-Verification and Truth Gate on every task. Sub-agents for partitioned work. Evidence Ledger for traceability. Consolidation protocol for merging duplicate files without capability loss.

**Repository Context (RTT):** This copy of the AGENT policy lives in the `rtt/` drop-in and governs all
agent- and provider-driven activity inside this repository. It inherits the global
"heal, do not harm" and consolidation rules but specializes examples and directory
maps to RTT. When later sections mention NOA ARK OS–specific paths such as
`services/gateway/`, `.workspace/registry/`, or `server/gateway/`, treat those as
upstream host-environment patterns. For RTT work, prefer the layout and contracts
described in `rtt-prd.md` and `docs/ARCHITECTURE.md`, and use this file as the
single agent/provider policy layer that must remain consistent with them.

> **Policy Markers**
> `@-- BEGIN:AGENT_POLICY --` … `@-- END:AGENT_POLICY --`
> Automation may only replace content between these markers; all other sections are immutable.

@-- BEGIN:AGENT_POLICY --

## 0) Purpose

Establish one strict operating policy for every agent-driven action so each task **heals** the codebase, **upgrades** capability, and **never downgrades nor deletes**. The policy blocks content rot, enforces cross-platform offline-first autonomy, and preserves architectural clarity across NOA ARK OS services, CRC, UI, workflows, storage, and tooling.

---

## 1) Non-Negotiable Rules

### Heal, Do Not Harm (Top Priority)

- **Upgrade, Enhance, Never Downgrade or Delete.**
- When a replacement is unavoidable, **compress + archive** the prior asset first; never remove history.
- Changes must remain **surgical**, **reversible**, and **traceable** via the archival ledger.

### Anti-Rot

- Every task must improve tests, coverage, documentation linkage, or automation quality.
- Stale references, dead endpoints, or orphaned files must be flagged and queued with remediation tasks—no silent decay.

### Architectural Clarity

- Maintain a canonical map of where code belongs (agents, ai, apps, core, crc, runtime, server, storage, ui, workflow, etc.).
- New components must declare their layer, dependencies, owners, and archival impact.

### Offline-First Ownership (Online via Feature Flag)

- **Default mode:** offline/local execution.
- **Online operations:** allowed only when `ONLINE_*` feature flags are explicitly enabled and logged.

### No Duplication, No Deletion

- Never duplicate instructions or logic across providers; use **gateways** and shared modules.
- Never delete files; if superseding content, compress+archive the prior version and document the swap.
- Preserve or wrap dead-code helpers and stubs so legacy integrations can be restored quickly.

### Contained Environment & Dependency Governance

- Treat NOA ARK OS as a **self-contained system**—all runtimes, libraries, assets, and secrets must live inside or be routed through the repository's managed gateways.
- **User-dynamic paths**: All path references must be relative to the user's installation location (never hardcoded to specific root paths). Users can place `noa_ark_os` anywhere on their system; agents must resolve paths dynamically from the repository root or user workspace.
- Provision dependencies via gateway-managed manifests (see `services/gateway/service.py`, `server/gateway/`, `.workspace/config/`, `.workspace/registry/*.json`, `tools/portable_builder.py`) so nothing bypasses archival or verification controls.
- Route every environment variable, credential, configuration knob, dependent runtime, and secret through gateway shims and adapters maintained in `server/gateway/src/*.rs`, `services/gateway/`, `agents/src/implementations/generated/infrastructure/*gateway*`, and related tooling; no ad-hoc `.env` or external service dependencies.
- Record environmental assumptions and manifest updates in the Evidence Ledger and mirror them in `.workspace/registry/` (or corresponding gateway config files) whenever a task introduces or changes gateway-managed configuration.

**Archival Procedure (mandatory):**

- Archive path: `archive/YYYY/MM/<relative-path>.tar.zst`
- Ledger entry: original path, reason, timestamp, SHA-256, replacement path
- Commit archive + ledger with the replacement change.

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

## 3) **Sample** Repository Structure Map (Where Things Belong)

This section specializes the global AGENT policy for the `rtt/` drop-in.

For authoritative, always-up-to-date details, defer to `docs/ARCHITECTURE.md`,
`docs/RSD-PLAN.md`, and `docs/PHASE-GUIDE.md`. The summary here is a quick
orientation for agents and operators.

```text
.rtt/                  # Control-plane state: CAS registry, manifests, panel, policy (see docs/ARCHITECTURE.md)
agents/                # Shared agent definitions and common agents used by RTT
providers/             # Provider overlays and provider-specific views
connector-mcp/         # MCP ingestion bridge and tooling
fabric/                # Lane implementations (SHM/UDS/TCP) for the RTT fabric
planner/               # Planner implementation and related tools
auto/                  # Automation pipeline stages (00-bootstrap → 50-apply_plan)
telemetry/             # Flight recorder and observability components
docs/                  # Architecture, PRD elaborations, operations, phase guides, acceptance criteria
tests/                 # Unit/integration/validation tests
tools/                 # Developer tools and helpers (CAS, MCP, plan, invariants, etc.)
systemd/               # Service units for running RTT as a daemon
scripts/               # Operational scripts and wrappers
```

When the AGENT policy references broader NOA ARK OS concepts (for example,
`.workspace/registry/` or `services/gateway/`), interpret them as **host
environment** patterns. In this `rtt/` repo:

- RTT’s own spec and delivery plan live in `rtt-prd.md`.
- Architecture and component locations are described in `docs/ARCHITECTURE.md`.
- Agent orchestration flows and background agents are described in
  `docs/AGENT-COORDINATION.md`.

Agents must use those files as the single source of truth for RTT-specific
structure before falling back to any upstream OS or organization-wide docs.

---

## 4) The 4-D Method (Required on Every Task)

1. **DECONSTRUCT:** Restate intent, inputs, constraints, and missing data.
2. **DIAGNOSE:** Surface ambiguities, risks, gaps, and architecture impact.
3. **DEVELOP:** Produce creative, technical, educational, and complex reasoning tracks with cross-platform considerations.
4. **DELIVER:** Assign roles, execute with verification, document archival/logging decisions.

### Per-Task Planning Checklist (3–7 bullets, conceptual only)

- [ ] Inputs & constraints acknowledged
- [ ] Interfaces/boundaries named
- [ ] Risks & mitigations identified
- [ ] Minimal viable increment defined
- [ ] Verification criteria declared
- [ ] Rollback + archival impact considered
- [ ] Sub-agent responsibilities enumerated

---

## 5) Operational Protocol

### 5-Step Execution Process

1. **Clarify inputs:** Restate the task, assumptions, blockers.
2. **Plan:** Outline minimal evidence steps; choose tests/checks.
3. **Gather:** Collect only necessary data; log sources and timestamps.
4. **Execute:** Change the smallest testable unit first; capture logs.
5. **Verify:** Apply the Truth Gate before any completion claim.

### Truth Sources Priority

1. User-provided instructions/files
2. Computations performed here with visible work
3. Referenced external sources (cited)
4. Model prior knowledge

Resolve conflicts in favor of the highest-priority source.

---

## 6) Verification & Truth Gate

### Triple-Verification Protocol

- **Pass A — Self-check:** Ensure internal consistency, spec ↔ artifacts alignment, smoke tests.
- **Pass B — Independent re-derivation:** Recompute/re-run fresh and compare deltas.
- **Pass C — Adversarial:** Negative/boundary tests and cross-tool verification.

### Truth Gate Checklist

- [ ] All referenced artifacts exist with SHA-256 hashes recorded
- [ ] Smoke tests executed with logs and exit codes
- [ ] Requirements ↔ artifacts ↔ tests fully mapped
- [ ] Limits, supported configs, and failure modes documented
- [ ] Evidence ledger updated (files, hashes, sources)
- [ ] Gap scan completed (coverage, docs, dependencies)
- [ ] Triple-Verification passes logged (A/B/C)

### Standard Output Artifacts

- **Claims Table:** `claims.table.md` with claim strength, evidence, tests, limits
- **Evidence Ledger:** `evidence.ledger.json` capturing files, hashes, data sources, formulas, tests
- **Truth Gate Checklist:** `truth_gate.checklist.md`
- **Result Block:**

  ```text
  RESULT: PASS | PARTIAL | FAIL
  WHY: <one line>
  EVIDENCE: <references>
  NEXT: <smallest verifiable follow-up>
  VERIFIED_BY: <Pass A/B/C status>
  ```

---

## 7) Provider & Agent Instruction Files

Provider instruction files must remain empty pointers back to this policy.

- `.copilot`

  ```text
  # Defer to AGENT.md
  Use AGENT.md at repo root as the sole policy and instruction source. Do not duplicate logic here.
  ```

- `CLAUDE.md`

  ```text
  This file intentionally contains no instructions. All policies and execution rules are defined in AGENT.md.
  ```

- `.github/AGENT_POINTER.txt`

  ```text
  Provider/agent instruction policy lives in AGENT.md. Do not place instructions elsewhere.
  ```

Set environment variables when supported:

- `AGENT_POLICY_PATH=AGENT.md`
- `AGENT_POLICY_VERSION=1.0.0`

---

## 8) Execution Guardrails

- **Non-destructive editing:** No deletions or history-dropping renames; archive superseded assets first.
- **Gateways, not copies:** Provider runtimes integrate through the shared gateway stack (`server/gateway/`, `services/gateway/`, generated gateway agents under `agents/src/implementations/generated/infrastructure/`) to avoid duplication, and **all connections must be routed through the gateway** (paths, routing, hooks, shims, auth, CAS, agents, providers, front-end, back-end, etc.).
- **Gateway-managed environments:** Configure dependencies, secrets, environment variables, and external integrations exclusively via gateway adapters and repository-contained manifests (`services/gateway/service.py`, `.workspace/config/`, `.workspace/registry/*.json`, `tools/portable_builder.py`); document updates in the Evidence Ledger and workspace registry checkpoints.
- **Conventional Commits:** Use `feat`, `fix`, `refactor`, `docs`, `test`, `perf`, `chore`; include archival notes and feature-flag status in commit messages.
- **CI Acceptance:** Lint, type checks, unit tests, duplicate detectors, and offline jobs must pass. Linux job mandatory; macOS/Windows matrix best-effort.
- **Dead-code-aware stubs:** Wrap inactive pathways and document them for quick reactivation of archived features.

### Capability Token Flow (Phase 5 Rollout)

1. **Kernel issuance:** Use `core/kernel/security/issue_capability_token` for controlled tooling/tests. Each token carries `fs`, `network`, and `rate_limit_per_minute` claims signed with the kernel secret defined in `core/kernel/security/tokens.py`.
2. **Gateway enforcement:** `services/gateway/service.py` must call `verify_capability_token` before routing. Reject when client IDs mismatch, required fs/network scopes are absent, or token rates fall below `PolicyRule.rate_limit_per_minute`.
3. **Effective throttling:** Honor the stricter of gateway policy vs. token allowance to prevent clients from exceeding kernel quotas while keeping telemetry coherent.
4. **Extension loading:** Route every adapter load through `extensions.ExtensionRegistry`. Declarative manifests (`extensions/*/manifest.json`) declare scope requirements; registry refreshes allow hot-swaps without process restarts while guarding adapters with the same capability token checks.
5. **Telemetry + auditing:** Persist gateway telemetry with scope metadata (`fs_scope`, `network_scope`, `token_rate_limit`) so audits can trace which claim combinations were exercised per request.

### Dead Code Handling

- Quarantine any superseded component by copying it into `archive/quarantine/<component>@<commit>/` and filling in both `README.md` and `status.yaml` (owner, contact, reintegration gates, hash ledger).
- Capture a fresh repository snapshot with `make snapshot` immediately after quarantining files so the ledger records a verified rollback point.
- Reference archived assets via the ledger only—never link to the relocated files directly from live code.

### Reintegration Procedure

1. Review the quarantine bundle’s `status.yaml` gates and confirm every condition has been satisfied (tests, reviews, telemetry).
2. Run `cargo run -p quarantine_guard --bin quarantine_guard -- <paths>` on the candidate changes; the guard must report zero quarantined references.
3. Restore the bundle through `make rollback BUNDLE=archive/YYYY/MM/snapshots/<snapshot>.tar.zst` if provenance files are required for comparison, then delete the restored copy once verification is complete.
4. When bundles age past 90 days, execute `cargo run -p quarantine_guard --bin quarantine_rotate` (or allow the scheduled workflow) to relocate them into `archive/YYYY/MM/quarantine/` and append a rotation entry to the monthly ledger.

---

## 9) Autonomy & Sub-Agent Pattern

- **Agent-0 (Orchestrator):** Route tasks, resolve dependencies, partition work.
- **Planner:** Produce execution DAG aligned with the 4-D plan.
- **Workers:** Handle ingestion, analysis, refactor, verification, packaging; respect archival rules.
- **Verifier:** Execute Triple-Verification, maintain the Evidence Ledger, close the Truth Gate checklist.
- **Reporter:** Emit results, logs, metrics, and UI-facing summaries.
- **Project Management Integration:** Log every request in workspace/project tooling (`.workspace/todo/`, registry updates) for traceability.

Each sub-agent must save `claims.table.md`, `evidence.ledger.json`, and `truth_gate.checklist.md` in its task workspace.

---

## 10) Cross-Platform Standards

- Favor POSIX-friendly defaults; avoid case sensitivity issues.
- Provide PowerShell equivalents for shell commands when scripting.
- Package artifacts for Linux/macOS/Windows portability.
- UI components must render on web by default; gate native desktop/mobile/AR/XR surfaces behind feature flags.
- Feature flags must document offline vs. online behavior explicitly.

---

## 11) Completion Gate

Before marking a task complete:

- [ ] Archival ledger updated and committed with new assets
- [ ] Duplicate-check CI job passes (or logged reason if pending)
- [ ] Provider stubs verified to point back to AGENT.md
- [ ] Offline mode validated; online usage documented via feature flags
- [ ] Repository memory/indexes updated if affected (e.g., `.workspace/indexes/`)
- [ ] Evidence Ledger and Truth Gate artifacts attached to the task record
- [ ] Sub-agent logs archived for audit

---

## 12) File Consolidation Protocol

**Definition**: Consolidation is the process of merging multiple files with overlapping purpose into a single canonical file while preserving all functionality, maintaining complete version history, and ensuring zero capability loss.

### When to Consolidate

- Duplicate-check CI identifies similar files with overlapping implementations
- Manual consolidation request via task workflow or operator directive
- Refactoring phase discovers redundant implementations across modules
- Fork integration requires merging external implementations with internal equivalents

### Consolidation Workflow

#### 1. Comparison Phase

- Generate side-by-side diff of source files using standard diff tools
- Extract function/class/capability inventory from each file (via AST parsing or manual review)
- Identify true duplicates (exact match) vs. variants (different behavior/signatures)
- Document behavioral differences in comparison report

#### 2. Merge Phase (Additive Strategy)

- **Bring ALL content** from incoming file(s) into the canonical target file
- Preserve every function, class, constant, type definition, and logic block
- Remove only exact duplicates:
  - Identical function signatures AND implementations
  - Semantically equivalent code (after normalization)
- Keep variants if behavior differs, even slightly
- Reformat merged content per project style guide (rustfmt, prettier, etc.)
- Add source attribution comments:

  ```rust
  // Consolidated from: old/path/auth_v2.rs (v2, 2025-11-19)
  // Original SHA-256: abc123...
  ```

- Update imports and resolve namespace conflicts
- Add docstring tags: `@consolidated_from <source_path> <version>`

#### 3. Verification Phase

- **Capability Matrix Check**: Verify all functions from source files present in consolidated output
  - Before consolidation: Extract list of all public functions, types, constants
  - After consolidation: Verify 100% presence in merged file
  - Document any intentional omissions (dead code, deprecated functions)
- Run full test suite (must pass without modification)
- Check that all imports/dependencies resolve correctly
- Verify no compilation errors or warnings introduced
- Generate `consolidation_report.md`:

  ```markdown
  # Consolidation Report

  **Date**: 2025-11-19
  **Canonical File**: server/gateway/src/auth.rs
  **Sources Merged**: 3 files

  ## Capability Comparison

  | Source File               | Functions | Preserved | Notes                     |
  | ------------------------- | --------- | --------- | ------------------------- |
  | old/auth_v1.rs            | 12        | 12        | All migrated              |
  | experimental/auth_v2.rs   | 8         | 8         | Variants kept             |
  | deprecated/auth_legacy.rs | 5         | 3         | 2 dead functions archived |

  ## Tests

  - All existing tests pass: ✅
  - New integration tests added: 2
  ```

#### 4. Archival Phase

- Archive each incoming file with versioned naming:
  - Path: `archive/consolidation/YYYY/MM/<relative-path>/v<N>.tar.zst`
  - Example: `archive/consolidation/2025/11/server/gateway/src/auth_v2/v1.tar.zst`
- Create or update version ledger: `archive/consolidation/YYYY/MM/<relative-path>/versions.json`

  ```json
  {
    "canonical_file": "server/gateway/src/auth.rs",
    "versions": [
      {
        "version": "v1",
        "source_path": "old/auth_v1.rs",
        "timestamp": "2025-11-19T12:00:00Z",
        "sha256": "abc123...",
        "consolidation_reason": "Duplicate implementation discovered",
        "preserved_capabilities": [
          "authenticate",
          "authorize",
          "validate_token"
        ],
        "archived_capabilities": ["legacy_auth_v1", "deprecated_check"],
        "merged_by": "agent-consolidation-v1"
      },
      {
        "version": "v2",
        "source_path": "experimental/auth_v2.rs",
        "timestamp": "2025-11-19T12:30:00Z",
        "sha256": "def456...",
        "consolidation_reason": "Experimental features promoted to production",
        "preserved_capabilities": ["oauth2_flow", "jwt_refresh"],
        "merged_by": "agent-consolidation-v1"
      }
    ]
  }
  ```

- Record consolidation event in Evidence Ledger (`audit/ledger.jsonl`):

  ```json
  {
    "event": "consolidation",
    "timestamp": "2025-11-19T12:00:00Z",
    "canonical": "server/gateway/src/auth.rs",
    "sources": ["old/auth_v1.rs", "experimental/auth_v2.rs"],
    "version": "v1",
    "sha256": "abc123...",
    "capabilities_preserved": 15,
    "capabilities_archived": 2
  }
  ```

- Batch Compression (≥100 Archived Versions)
  - **Trigger**: When a consolidation directory accumulates 100+ versioned archives
  - Group by month and directory: `archive/consolidation/YYYY/MM/batch_NNN.tar.zst`
  - Include batch manifest: `archive/consolidation/YYYY/MM/batch_NNN_manifest.json`

    ```json
    {
      "batch_id": "batch_001",
      "compression_date": "2025-11-19T15:00:00Z",
      "file_count": 127,
      "total_size_mb": 45.2,
      "compression_ratio": 8.3,
      "files": [
        {
          "archived_path": "server/gateway/src/auth_v2/v1.tar.zst",
          "sha256": "abc123...",
          "original_path": "old/auth_v1.rs",
          "size_bytes": 8192
        }
      ],
      "notes": "Monthly batch compression of November 2025 consolidations"
    }
    ```

  - Update master consolidation index: `.workspace/registry/consolidation_index.json`

    ```json
    {
      "last_updated": "2025-11-19T15:00:00Z",
      "total_consolidations": 342,
      "batches": [
        {
          "batch_id": "batch_001",
          "path": "archive/consolidation/2025/11/batch_001.tar.zst",
          "file_count": 127,
          "date": "2025-11-19"
        }
      ]
    }
    ```

  - Delete individual `.tar.zst` files after successful batch compression
  - Verify batch integrity: extract and compare SHA-256 hashes against manifest

### Production File Naming Rules

- **Canonical file retains original production name** throughout consolidation process
  - Example: `server/gateway/src/auth.rs` stays `auth.rs` even after consolidating 5 variants
  - Never rename production files based on consolidation activity
- Archive versioning is independent of production naming conventions
- Consolidation attribution embedded as inline comments and docstrings, not reflected in filename
- If production file requires renaming for architectural reasons, treat as separate refactoring task (not part of consolidation)

### Rollback Procedure

1. **Locate version** in consolidation ledger:
   - Check `.workspace/registry/consolidation_index.json` for batch location
   - Find version entry in `archive/consolidation/YYYY/MM/<path>/versions.json`
2. **Extract archived version**:

   ```bash
   # Individual version
   tar -xf archive/consolidation/2025/11/server/gateway/src/auth_v2/v1.tar.zst

   # From batch
   tar -xf archive/consolidation/2025/11/batch_001.tar.zst server/gateway/src/auth_v2/v1.tar.zst
   tar -xf server/gateway/src/auth_v2/v1.tar.zst
   ```

3. **Review pre-consolidation snapshot**:

```bash
make rollback BUNDLE=audit/bundle-2025-11-19T12-00-00.tar.zst
```

1. **Restore if needed**: Copy extracted version to production path, run tests, commit with rollback note
2. **Document rollback**: Update rollback ledger (`audit/rollbacks/`) with reason and outcome

### Consolidation Guardrails (Truth Gate)

- [ ] Pre-merge capability inventory captured (all functions/types/constants listed)
- [ ] All source functions present in consolidated output (capability matrix = 100%)
- [ ] Tests pass post-consolidation without modification
- [ ] Version ledger updated with SHA-256 hashes and timestamps
- [ ] Consolidation report generated with before/after comparison
- [ ] Production file name unchanged
- [ ] Archive paths follow standard structure (`archive/consolidation/YYYY/MM/...`)
- [ ] Batch compression triggered and verified at 100+ file threshold
- [ ] Evidence Ledger entry created for consolidation event
- [ ] Source attribution comments added to merged code
- [ ] Imports/dependencies verified and resolved

### Consolidation Tooling Recommendations

#### Immediate Implementation

1. **CLI Command**: Extend `server/tools_agent` with consolidation subcommand:

   ```bash
   cargo run -p noa-tools-agent consolidate \
     --sources old/auth_v1.rs,experimental/auth_v2.rs \
     --target server/gateway/src/auth.rs \
     --verify \
     --archive
   ```

   - Parse AST to extract function signatures
   - Generate capability matrix automatically
   - Run tests before/after merge
   - Create version ledger entries
   - Archive source files with compression

2. **Automated Capability Checking**: Use `syn` crate (Rust) or language-specific AST parsers:

   ```rust
   pub fn extract_capabilities(file_path: &Path) -> Result<Vec<Capability>> {
       let content = fs::read_to_string(file_path)?;
       let syntax = syn::parse_file(&content)?;
       let mut capabilities = Vec::new();
       for item in syntax.items {
           match item {
               syn::Item::Fn(func) => capabilities.push(Capability::Function(func.sig.ident.to_string())),
               syn::Item::Struct(s) => capabilities.push(Capability::Type(s.ident.to_string())),
               // ... extract all public items
           }
       }
       Ok(capabilities)
   }
   ```

3. **Consolidation Report Generator**: Markdown template with:
   - Side-by-side diff visualization
   - Capability matrix table (preserved/archived/new)
   - Test results summary
   - File size comparison
   - Complexity metrics (cyclomatic, lines of code)

#### Future Enhancements

1. **Consolidation Dashboard** (`ui/core/src/components/ConsolidationDashboard.tsx`):
   - Track consolidation metrics: total files consolidated, archive growth rate, rollback frequency
   - Visualize consolidation graph: which files merged into which canonical files
   - Show capability preservation percentage over time
   - Alert on failed consolidations or capability loss

2. **Smart Deduplication Engine**:
   - Use semantic similarity (cosine similarity on AST embeddings) to detect functionally equivalent code
   - Threshold: >95% similarity = candidate for deduplication
   - Generate suggestions: "Function `auth_v1` and `auth_v2` are 98% similar. Consolidate?"
   - ML model trained on codebase to detect patterns (e.g., "this is a refactored version of that")

3. **Conflict Resolution UI** (`ui/core/src/components/ConflictResolver.tsx`):
   - When merge creates ambiguity (e.g., two functions with same name, different behavior):
     - Show side-by-side diff
     - Prompt: "Keep both as variants?" / "Merge with conditional logic?" / "Flag for manual review?"
   - Track resolution decisions in consolidation ledger
   - Allow rollback of individual conflict resolutions

4. **CRC Integration**: Treat fork consolidation as special case:
   - External fork code merges with internal equivalent via consolidation protocol
   - Fork attribution preserved: `// Consolidated from fork: <repo_url> @ <commit>`
   - Fork-specific tests maintained alongside internal tests
   - See `crc/FORK_PROCESSING_SYSTEM.md` for fork-to-consolidation pipeline

5. **Consolidation CI Job** (`.github/workflows/consolidation.yml`):
   - Detect consolidation candidates automatically (via duplicate-check or similarity scan)
   - Create consolidation PR with pre-filled report
   - Run capability checks and tests in CI
   - Auto-merge if all guardrails pass and approval granted

### Integration with Existing Systems

- **Evidence Ledger**: All consolidation events logged to `audit/ledger.jsonl`
- **Truth Gate**: Consolidation reports satisfy Truth Gate requirements (artifacts, tests, verification)
- **Archival System**: Consolidation archives follow standard `archive/YYYY/MM/` structure
- **CRC Pipeline**: Fork consolidation flows through CRC → consolidation → production
- **Workspace Registry**: Consolidation index maintained in `.workspace/registry/consolidation_index.json`
