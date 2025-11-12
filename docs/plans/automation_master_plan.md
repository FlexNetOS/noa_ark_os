<!-- BEGIN: AUTOMATION_MASTER_PLAN -->
# AgenticOS Automation Master Plan

## Inventory
| Path | Purpose | Manual Steps | Current Trigger | Observable Outputs |
| --- | --- | --- | --- | --- |
| `.github/workflows/ci.yml` | GitHub Actions pipeline for pnpm build/lint/test/export. | Requires cloud GitHub runner; offline developers rerun tasks manually when disconnected; no auto retry or summary. | `push` to `main` and `release/**`; PR open/update. | Workflow logs, `build_kits/pm_roadmap.*` artifact. |
| `scripts/dev/quickstart.sh` | End-to-end bootstrap (cargo build/test/demo). | `read -p` prompt blocks automation; human inspects console for failures. | Developer invokes manually. | Console log only. |
| `package.json` scripts (`pnpm build/lint/typecheck/test/export:roadmap`) | Frontend build/test for `ui/vibe-kanban`. | No orchestration; humans chain commands and collect artifacts. | Manual `pnpm <script>` or CI job. | CLI exit status, generated Next.js `.next/`, lint reports. |
| `Cargo.toml` workspace (Rust crates) | Builds core services, agents, CLI, runtime. | Relies on manual feature selection and sequential runs; TODO placeholders require human triage. | Manual `cargo build/test`, or invoked via quickstart. | Target binaries, test reports. |
| `cicd/pipelines/crc-auto.yaml` | CRC automation pipeline for drop-in integrations. | `quality_gate` falls back to `manual_review`; humans inspect logs and move drops. | Files placed in `crc/drop-in/ready`. | Moved artifacts, log transcripts under `cicd/logs`. |
| `workflow/flows/merge/README.md` + `merge_consolidate.sh` | Runbook for merge/polish flows. | Manual conflict resolution instructions when simulation fails. | Operator runs merge script. | Merged tree, manual notes. |
| `apps/cli/src/main.rs` (`noa-cli`) | Relocation daemon CLI. | `approve` & `manual_override` subcommands require human decision. | Operators run CLI. | JSON approval report, stdout. |
| `tools/ai/ollama_manager.sh` | Local model lifecycle manager. | When `yq` missing, registry updates require manual edits. | Manual CLI invocation. | Console output, modified `models.yaml`. |
| `scripts/install_gh_cli.sh` | GitHub CLI installer. | Non-Debian platforms instructed to install manually via docs. | Manual script run. | CLI install or error message. |
| `docs/plans/gap_remediation_tasks.md` | Release planning checklist. | Release notes drafting and announcement coordination handled manually. | During release preparation. | Markdown updates; no generated artifacts. |

## HITL → Automation Map (overview)
- Seven distinct human-in-the-loop (HITL) touchpoints identified (see `docs/plans/hitl_elimination_map.md`).
- Each HITL step is mapped to an offline-first automation: local runners, autonomous policy engines, or AI co-pilots with llama.cpp default models.
- Online GitHub remains available behind `ONLINE_GITHUB_MODE=true`, syncing offline artifacts (PR reviews, release notes, approvals) only after offline gates succeed.

## Offline GitHub Mirror
1. **Local PR queue**
   - Directory: `tools/offline_pr_queue/` with subfolders `requests/`, `reviews/`, `merges/`, `artifacts/`.
   - Commands (via new `runa` CLI wrapper):
     - `runa pr create <branch> [--summary-file]` → snapshots diff, generates `request.json`, stores AI risk report.
     - `runa pr review <request-id> [--auto-apply]` → runs lint/test matrix offline, invokes AI reviewer via llama.cpp, records feedback under `reviews/<id>.json`.
     - `runa pr merge <request-id>` → enforces policy gates, performs merge locally (using Git worktree), writes ledger entry in `merges/ledger.jsonl`.
   - Ledger schema includes diff hash, test digest, approvals, and policy flags for reproducibility.
2. **Offline Actions runner**
   - Use `act` with `--container-architecture` matching target; provide `tools/offline_pr_queue/fixtures/github-secrets.env` for stubbed secrets.
   - Default `OFFLINE_FIRST=true` blocks external network calls; workflows requiring network must declare `requires_network=true` to opt-in.
3. **Synchronization when online**
   - Background task `runa pr sync` (triggered when `ONLINE_GITHUB_MODE=true`) mirrors queued PRs to GitHub, posts AI review summaries, attaches artifacts, and reconciles merge status.
   - Conflict policy: offline queue remains source of truth; GitHub merges allowed only if ledger entry exists.

## AI Co-Pilot Roles & Prompts
- **Provider abstraction**: Implement `server/ai/providers/` interface (`mod.rs` defining `AiProvider` trait) with adapters:
  - `llama_cpp` default using `LLAMA_CPP_ENDPOINT`.
  - `openai` adapter honoring `OPENAI_MODEL` (default `gpt-5`, fallback `gpt-4.1`).
  - `anthropic` adapter using `ANTHROPIC_MODEL`.
- **Roles**
  - *Code reviewer*: inspects diffs, emits blocking/advisory/nit suggestions, runnable offline.
  - *Test writer*: expands coverage by analyzing changed files and generating tests.
  - *Doc drafter*: summarizes changes into release notes/docs.
  - *Triage agent*: diagnoses failed pipelines/logs.
  - *Commit co-pilot*: suggests Conventional Commit messages per module impact.
- **Prompt packs** stored under `server/ai/prompts/<role>.md` with deterministic instructions (temperature ≤ 0.2, max tokens 1024).
- **Caching**: local SQLite cache `runtime/cache/ai_responses.sqlite` keyed by hash of prompt+context.
- **Safety**: scrub secrets via regex before outbound calls; offline mode bypasses remote providers entirely.

## Pipelines
### Pull request creation & triage
- Trigger: `runa pr create` or offline queue entry.
- Steps: generate diff summary → run static analyzers (lint/type/test) → AI risk scoring (code reviewer role) → label via `labels.json`.
- Artifacts: `tools/offline_pr_queue/requests/<id>/summary.md`, risk JSON, coverage delta report.
- Acceptance: queue entry valid when analyzers succeed and AI review stored; failure escalates to `triage` role.

### Code review & suggestion application
- Trigger: `runa pr review` or CI completion.
- Steps: AI reviewer produces suggestions; auto-apply diffs via `git apply` sandbox; re-run targeted tests; update ledger.
- Offline default applies suggestions when severity ≤ advisory.
- Online sync posts review via GitHub API when `ONLINE_GITHUB_MODE=true`.
- Acceptance: mergeable only when no blocking findings remain, advisory suggestions applied or explicitly waived.

### Failed runs auto-triage
- Trigger: workflow exit code ≠ 0 (from `act`, cargo, pnpm, go tests).
- Steps: parse logs with deterministic regex → AI triage summarises root cause & fix → create local ticket in `tools/offline_pr_queue/incidents/<timestamp>.json`.
- Online sync optionally opens GitHub issue with label `auto-triaged`.
- Acceptance: each failure yields triage artifact with reproduction command and proposed fix snippet.

### Commits & messages (co-pilot)
- Trigger: `runa commit` wrapper.
- Steps: analyze staged diff, map to modules via `WORKSPACE_ORGANIZATION_PLAN.md`, enforce Conventional Commit scope; AI suggests subject/body; user can accept or regenerate.
- Offline ensures no network; online optionally validates via OpenAI when flagged.
- Acceptance: commit blocked unless `pnpm commitlint`/Rust fmt/tests pass and message conforms to spec.

### Auto-merge policy
- Trigger: `runa pr merge`.
- Gates: lint/type/test matrix, coverage ≥ baseline, SBOM scan clean, license scan clean, secret scan clean, AI review = advisory/nit only, policy YAML under `.workspace/policy/merge.yaml` satisfied.
- Artifacts: `merges/<id>/compliance_report.json` and signed summary (age/sigstore optional).
- Acceptance: merge occurs only when all gates true; else PR flagged for manual review.

### Release notes & versioning
- Trigger: `runa release prepare` (per tag branch) or nightly automation.
- Steps: collect merged PR ledger entries → AI doc drafter composes notes → bump version via `workspace/version.toml` (forward-only) → update changelog & docs.
- Offline stores notes under `docs/releases/drafts/<version>.md`; online sync publishes GitHub release when flagged.
- Acceptance: release blocked unless changelog diff, version bump, SBOM, and release notes draft exist.

## Quality/Security Upgrades
- **Lint/type/test matrices**
  - Rust: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-features`.
  - Go: `go fmt ./...`, `go test ./...`, `golangci-lint run` (vendor offline). 
  - Node: `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm format` (check mode).
- **Pre-commit**: configure `pre-commit` hooks wrapping above commands plus Conventional Commit lint via `tools/git-hooks/pre-commit` script.
- **Deterministic builds**: lock tool versions (`rust-toolchain.toml`, `pnpm-lock.yaml`, `go.sum`) and vendor dependencies into `vendor/` for offline usage.
- **SBOM & license**: run `syft packages . -o json` (offline) and `licensee detect . --json` within sandbox; store under `build_artifacts/sbom/`.
- **Secret scanning**: offline regex-based scanner `tools/security/secrets_scan.py` invoked pre-merge.
- **A11y checks**: `pnpm --filter vibe-kanban test:a11y` (new script using `axe-core`) executed in pipeline.

## Feature Flags & Env
```
OFFLINE_FIRST=true
ONLINE_GITHUB_MODE=false
AI_PROVIDER=llama.cpp  # options: llama.cpp|openai|anthropic
OPENAI_MODEL=gpt-5     # fallback to gpt-4.1 without failing
LLAMA_CPP_ENDPOINT=http://127.0.0.1:8080/v1
ANTHROPIC_MODEL=claude-3-opus
CRC_AUTO_MODE=offline
RELOCATION_POLICY_AUTO_APPROVE=true
```
- Flags stored in `.env.local.example` with documentation in `docs/AI_PROVIDER_CONFIG.md` and new `docs/runtime/offline_flags.md`.

## Risk/Rollback (upgrade-only guardrails)
- Maintain shims: existing scripts keep current behavior when `OFFLINE_FIRST` unset; automation adds opt-in flags first.
- All new commands log to `tools/offline_pr_queue/logs/` with replayable steps for rollback.
- Auto-generated changes require signed ledger entry; revert path defined by `runa pr revert <id>`.
- Online sync uses dry-run mode before mutating GitHub to prevent accidental downgrades.

## Acceptance Criteria & KPIs
- 100% of identified HITL steps have automated pathways with binary checks (see CSV/JSON build kits).
- ≥90% PRs merged via offline queue without human review once policy gates pass; remaining ≤10% flagged for assisted workflow.
- Mean time to green ≤30 minutes for standard PRs using local runners.
- Release cadence: automated notes + version bump produced within 15 minutes of tagging.
- Offline parity: lint/test/SBOM/license/AI review identical outputs offline vs online (hash comparison tolerance ±1%).

## Suggested Tasks & Links
- **Suggested task:** [View task](#task-offline-pr-cli)
- **Suggested task:** [View task](#task-ai-provider-abstraction)
- **Suggested task:** [View task](#task-crc-automation)
- **Suggested task:** [View task](#task-relocation-policy-engine)
- **Suggested task:** [View task](#task-release-ledger-notes)

<a id="task-offline-pr-cli"></a>
### AUTO-1 — Stand up offline PR queue & `runa` CLI
**Description:** Build the offline-first pull request queue, ledger, and CLI (`runa`) commands that anchor automation, including create/review/merge flows and artifact persistence.

**Checklist**
- [ ] Scaffold `tools/offline_pr_queue/` directories with schema-validated JSON contracts for requests, reviews, merges, and incidents.
- [ ] Implement `runa pr create`, `runa pr review`, and `runa pr merge` commands with configurable policy gates and dry-run mode.
- [ ] Integrate local execution of lint/test matrices and attach result digests to ledger entries.

**Acceptance criteria**
- Creating a PR offline produces ledger artifacts with deterministic hashes and policy evaluation results.
- Reviewing a PR runs the offline CI matrix and stores AI summaries without reaching the network when `OFFLINE_FIRST=true`.
- Merge command blocks until all acceptance gates succeed and writes a signed compliance report.

**Meta**
- Owner: TBA
- Priority: P0
- Status: Proposed
- Depends on: None

<a id="task-ai-provider-abstraction"></a>
### AUTO-2 — Implement AI provider abstraction & caching
**Description:** Provide a unified provider interface under `server/ai/providers/` with adapters for llama.cpp (default), OpenAI, and Anthropic plus deterministic caching and safety rails.

**Checklist**
- [ ] Define `AiProvider` trait/module with shared request/response schema, safety hooks, and instrumentation.
- [ ] Ship adapters for llama.cpp, OpenAI, and Anthropic honoring feature flags and offline fallbacks.
- [ ] Build prompt cache backed by SQLite with eviction policy and hash-based lookups.

**Acceptance criteria**
- Switching `AI_PROVIDER` between `llama.cpp`, `openai`, and `anthropic` routes requests without code changes or restarts.
- Offline mode (`OFFLINE_FIRST=true`) prevents outbound network calls and reuses cached completions when available.
- Secret redaction and token limits enforced consistently across providers with unit tests covering edge cases.

**Meta**
- Owner: TBA
- Priority: P0
- Status: Proposed
- Depends on: AUTO-1

<a id="task-crc-automation"></a>
### AUTO-3 — Automate CRC quality gate adjudication
**Description:** Replace manual CRC `quality_gate` reviews with an offline policy engine and AI-assisted triage that records confidence scores and escalation artifacts.

**Checklist**
- [ ] Implement metrics policy evaluation module for `cicd/pipelines/crc-auto.yaml` with configurable thresholds and fixtures.
- [ ] Integrate AI triage summaries stored under `crc/logs/quality_gate.jsonl` when confidence ≥ 0.9.
- [ ] Provide escalation workflow that queues manual review tasks when automation declines to approve.

**Acceptance criteria**
- CRC pipeline runs offline without manual intervention for nominal data sets and records automated pass/fail rationale.
- Confidence scoring below threshold produces structured escalation entries for human review.
- Historical replay of CRC runs yields identical decisions when rerun with the same inputs.

**Meta**
- Owner: TBA
- Priority: P1
- Status: Proposed
- Depends on: AUTO-1, AUTO-2

<a id="task-relocation-policy-engine"></a>
### AUTO-4 — Automate relocation approvals with policy engine
**Description:** Introduce a rule + AI-driven evaluator for the relocation daemon (`apps/cli`) that automates safe approvals and logs rationale for auditability.

**Checklist**
- [ ] Encode relocation safety policies and overrides in machine-readable configuration with unit tests.
- [ ] Extend CLI to run `runa relocation auto-approve` applying policies and invoking AI triage when anomalies detected.
- [ ] Persist approval/denial logs and rollback metadata under `.workspace/relocation/logs/`.

**Acceptance criteria**
- Auto-approval executes without human input for routine relocations and stores decision artifacts with AI confidence.
- Policy violations or ambiguous cases trigger manual queue entries instead of silent approval.
- Auditors can replay an approval decision using captured context and configuration snapshots.

**Meta**
- Owner: TBA
- Priority: P1
- Status: Proposed
- Depends on: AUTO-2

<a id="task-release-ledger-notes"></a>
### AUTO-5 — Generate release notes from ledger artifacts
**Description:** Automate release preparation by synthesizing changelog entries and notes from the offline PR ledger and AI doc drafter role.

**Checklist**
- [ ] Aggregate merged PR ledger entries into release candidate bundles with semantic grouping.
- [ ] Invoke doc-drafter prompts to produce release drafts stored under `docs/releases/drafts/`.
- [ ] Enforce version bump guardrails and changelog updates via `runa release prepare` command.

**Acceptance criteria**
- Running `runa release prepare` offline generates version bump proposal, changelog diff, and release notes draft without manual editing.
- Generated notes capture risk/impact sections with traceability back to ledger entries.
- Online sync (when enabled) publishes notes to GitHub releases only after offline artifacts pass policy checks.

**Meta**
- Owner: TBA
- Priority: P2
- Status: Proposed
- Depends on: AUTO-1, AUTO-2, AUTO-3

## Execution Summary
- **Total HITL steps found:** 7
- **Automation coverage:** 6 targeted for full removal, 1 (CRC quality gate) converted to AI-assisted with manual fallback when confidence <0.9.
- **Offline parity:** PR queue, CI matrix, release generation, relocation approvals, and model registry tasks fully mirrored offline.
- **Online pathways:** `ONLINE_GITHUB_MODE=true` syncs PRs, reviews, releases, and approvals to GitHub without bypassing offline ledger.
- **Next tasks:**
  1. Owner TBA — Priority P0 — Implement `runa` CLI scaffolding for offline PR queue (`tools/offline_pr_queue/`).
  2. Owner TBA — Priority P0 — Build provider abstraction in `server/ai/providers/` with llama.cpp adapter.
  3. Owner TBA — Priority P1 — Wire CRC pipeline automation to replace `manual_review` gate with AI triage.
  4. Owner TBA — Priority P1 — Automate relocation approvals using policy engine + AI triage in `apps/cli`.
  5. Owner TBA — Priority P2 — Generate release notes automation leveraging ledger & doc drafter role.
<!-- END: AUTOMATION_MASTER_PLAN -->
