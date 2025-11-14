# NOA ARK OS – Hermetic Execution Targets

**Version:** 0.1.0 (2025-11-14)

## Purpose

Translate the policy stack (AGENT.md), goal inventory, and the Agentic Kernel Roadmap into a concrete set of hermetic-behavior targets that keep every CLI, pipeline, and agent workflow self-contained. This file is the living checklist for sealing the toolchain, enforcing gateway-only dependencies, and proving offline-first execution.

## Instruction Inputs

| Source | Key Signals |
| --- | --- |
| `AGENT.md` | Heal-don't-harm, offline-by-default, gateway-managed dependencies/secrets, archival + evidence requirements |
| `docs/projects/ROADMAP_AGENTIC_KERNEL.md` | Phase 0.5 CLI-first mandate, pipeline blueprint, end-cap automation tasks |
| `docs/projects/roadmap_noa_ark_os.json` | Machine-readable goals (1–18), phase dependencies, progress metrics |
| `docs/ROADMAP_UPGRADE_SUMMARY.md` | Integration touchpoints (Makefile, pipeline.yml, documentation hierarchy) |

## Hermetic Objectives

1. **Portable Toolchains Only** – Rust, Node, pnpm, and auxiliary build tools must ship inside `server/tools/` or equivalent portable bundles with documented activation scripts.
2. **CLI-First Envelope** – The `noa` CLI (Phase 0.5) must encapsulate every capability with machine-readable output and never require IDE extensions or external services.
3. **Offline Pipeline Authority** – `make pipeline.local` remains the single source of truth; all stages consume local caches, world graphs, and manifests.
4. **Gateway-Governed Configuration** – Every environment knob, credential, or profile routes through gateway manifests and the kernel-issued capability tokens (Phases 5 & 9).
5. **Evidence & Reversibility** – Each hermetic operation records snapshots, SBOMs, trust scores, and ledger entries so the system can be rolled back or replayed deterministically.

## Target Catalog

| ID | Focus | Description | Dependencies | Acceptance Criteria |
| --- | --- | --- | --- | --- |
| **HT-01** | Portable Toolchains | Mirror the Node 20 + pnpm stack into `server/tools/` (next to portable Cargo) and expose a single activation script. Remove reliance on host-level installations. | Makefile `setup`, Phase 0.5 tooling prerequisites | `server/tools/activate-dev.ps1`/`.sh` spins up Rust+Node+pnpm; pipeline/local runs reference only portable paths; evidence ledger notes tool versions. |
| **HT-02** | CLI Build Capsule | Implement the `noa` CLI binary with 10+ subcommands plus `--json/--yaml/--tui/--watch` modes and ensure it bootstraps without IDE hooks. | Phase 0.5 tasks T1–T7 | CLI compiles via portable toolchain, emits reproducible JSON, and exposes shell completions + agent invocation endpoints. |
| **HT-03** | Pipeline Hermeticity | Seal `make pipeline.local` so each stage (world-verify → sign) pulls inputs from workspace manifests, cached registries, or snapshots, never from the public network unless `ONLINE_*` flags are set. | Phases 0, 3, 8, 10 | Air-gapped dry-run passes, SBOM artifacts exist for kernel/extensions, audit bundle lists zero external fetches, ONLINE flag logs when enabled. |
| **HT-04** | Gateway & Profiles | Define capability profiles (`/server/profiles/`) and gateway manifests describing allowed tools, budgets, and network scopes; enforce via kernel-issued tokens. | Phases 5, 9 | Profiles checked into repo, gateway validates tokens before executing CLI/pipeline commands, and profile diffs recorded in Evidence Ledger. |
| **HT-05** | Evidence + Observability | Extend the Truth Gate artifacts (claims table, evidence ledger, checklist) so every hermetic run outputs hashes, snapshot IDs, and rollback plans. | Phases 7, 10, 11 | `claims.table.md`, `evidence.ledger.json`, and `truth_gate.checklist.md` generated per pipeline run; reward metrics penalize non-hermetic behavior. |

### HT-01 Snapshot (2025-11-14)

| Tool | Version | Portable Path | SHA-256 | Evidence |
| --- | --- | --- | --- | --- |
| Node.js | 20.19.5 | `server/tools/node-portable/current/bin/node` | `8d01d4c50e7a9047d70196f8fc6f1b165069065b44bd9402491544bd70586c7d` | `server/tools/node-portable.manifest.json` |
| pnpm | 8.15.4 | `server/tools/node-portable/current/bin/pnpm` | `7d26cc57186850a2d71ab77da7cf52ff0eeabf680ac446c8da2324aa63808aac` | `docs/evidence_ledger.md` entry 2025-11-14 |

## Action Sequencing

1. **Immediate (Now → Phase 0.5)**
   - Replicate host-level Node/pnpm installs into `server/tools/` (HT-01).
   - Add Makefile verification that `server/tools` activation is sourced before CLI or pipeline targets.
   - Draft CLI scaffolding to reserve subcommands + output modes (HT-02).

2. **Short Term (Phase 1–3)**
   - Embed world graph schemas + reconciler outputs into pipeline caches (HT-03).
   - Define capability token formats for gateway-enforced CLI invocations (HT-04 dependency on kernel token service).

3. **Mid Term (Phase 4–9)**
   - Complete CLI expansion, plugin system, and registry coverage to remove remaining IDE reliance (HT-02 extension).
   - Finalize deployment profiles and integrate them with gateway manifests + CLI `noa profile` commands (HT-04).

4. **Long Term (Phase 10–13)**
   - Automate Truth Gate artifact generation per run (HT-05) and feed reward history.
   - Verify fully autonomous operation with hermetic audits in fire-drill scenarios.

## Verification Playbook

| Step | Check |
| --- | --- |
| 1 | Run `source server/tools/activate-cargo-wsl.sh && server/tools/activate-node.sh` (to be created) and ensure PATH contains only portable binaries. |
| 2 | Execute `make pipeline.local` with network disabled; confirm caches satisfy all dependencies. |
| 3 | Capture snapshot + SBOM hashes and log them in `audit/` + Evidence Ledger. |
| 4 | Inspect CLI output via `noa kernel status --json` (once implemented) to confirm machine-readable contract. |
| 5 | Review gateway logs to ensure every command carries a capability token tied to an active profile. |
| 6 | Apply Truth Gate checklist before claiming success; attach logs to `docs/reports/` as needed. |

## Documentation & Reporting Hooks

- Link this file from `ROADMAP_AGENTIC_KERNEL.md` in the CLI-First section.
- When HT items progress, update both this catalog and the machine-readable roadmap (`roadmap_noa_ark_os.json`).
- Record environment/toolchain updates in `docs/evidence_ledger.md` (or the structured ledger JSON when available).

## Next Steps Snapshot

1. Add portable Node/pnpm bundle instructions to the Makefile + server tools (HT-01).
2. Bootstrap the `noa` CLI crate/binary with stub subcommands and JSON output scaffolding (HT-02).
3. Extend pipeline targets with offline verification hooks and Truth Gate artifact emission (HT-03 & HT-05).
4. Start the capability profile schema in `/server/profiles/` and tie it to gateway manifests (HT-04).

Tracking progress here keeps the CLI-first transformation aligned with the hermetic policy wall and provides a single reference for future automation agents.
