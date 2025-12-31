# NOA Ark OS Integration Ledger

_Last updated: 2025-09-30 (Workspace sync)_

## Mission & Guardrails
## 1) Task Ledger

### 1.1 Executed Tasks
- [x] 2025-09-30 23:08 UTC — Phase 1 inventory baseline captured — Artifacts: `docs/integration/phase1_inventory.md`, `cargo metadata` log (terminal) — Notes: catalogued workspace crates/services and identified missing Rust ecosystem components; TODO updated.

### 1.2 Backlog Excerpts
- (See `TODO.md` for full phase checklist).

- Unify the Ark-derived monorepo (`mono/`) with supporting operator assets while preserving local-first execution.
- Keep this file as the authoritative tracker for executed work, evidence bundles, and outstanding actions.
- Map every change to tangible artifacts under version control, following the "Heal, Don't Harm" directive.

## Hosted Content Inventory (All-depths)
- **Root evidence dossiers** — `ARCHITECTURE_COMPLETE.md`, `BACKEND_MULTIPLEXING_COMPLETE.md`, `FASTEMBED_NATIVE_COMPLETE.md`, `STREAMING_INTEGRATION_COMPLETE.md`, `PROJECT_SUMMARY.md`, `*_COMPLETE.md` bundles documenting prior Ark milestones.
- **Data & analytics** — Tabular inventories `noaOS.csv`, `noaOS.xlsx` plus `data/` staging area for future exports and snapshots.
- **Ark monorepo (`mono/`)**
	- Apps: `apps/desktop`, `apps/web` (Leptos 0.7 target with recent compile fixes).
	- Services: `services/{agent,analytics,api,inference,retrieval}` (agent compiling with transport refactor; inference/retrieval currently red).
	- Crates: `crates/{config,globals,obs,pybridge,rustpyshim,shared,wasm-infer}` plus shared tooling in `mono/tools/`.
	- Operations & docs: `mono/docs/{README.md,architecture/,operations/,playbooks/,references/}` with runbooks and topology overlays.
	- Tests: `mono/tests/{conformance,integration,performance,smoke,unit}` scaffolding prepared for future harnesses.
	- Build + vendor metadata: `Makefile`, `rust-toolchain.toml`, `Cargo.lock`, `.cargo/config.toml`, `vendor/`.
- **Unified JS workspace (`unified/`)** — `pnpm-workspace.yaml`, shared `scripts/boot.ts`, agent digest packages, kernel/shared/shims packages, Johnson interop contracts, and TS configs aligned for DOM + Node.
- **Tooling binaries** — `tools/aos-ctl`, `tools/aos-watchdog`, `tools/aos-installer` Rust CLIs aligned with operator workflows.
- **Packaging & systemd assets** — `packaging/` and `mk/` targets for deployment orchestration (systemd manifests, Makefiles).
- **Agenticos bundles** — `agenticos-otel-services.zip`, `agenticos-watchdog-otel.zip`, `agenticos-rollback-watchdog.zip` pending review and extraction.
- **Config & telemetry staging** — `config/`, `logs/`, `tests/`, `repos/` placeholders to capture future integration artefacts.

## Target Operating Structure
```
noa_ark_os/
├── sot.md          # <-- this ledger (authoritative task list)
├── mono/
│   ├── apps/{desktop,web}
│   ├── services/{agent,analytics,api,inference,retrieval}
│   ├── crates/{config,globals,obs,pybridge,rustpyshim,shared,wasm-infer}
│   ├── data/notebooks
│   ├── db/migrations
│   ├── docs/
│   │   ├── README.md
│   │   ├── architecture/
│   │   ├── operations/
│   │   ├── playbooks/
│   │   └── references/
│   ├── scripts/
│   ├── tests/
│   │   ├── README.md
│   │   ├── conformance/
│   │   ├── integration/
│   │   ├── performance/
│   │   ├── smoke/
│   │   └── unit/
│   ├── tools/
│   └── trainer/burn/
├── mk/
├── packaging/
├── repos/
├── tools/
└── unified/
```

## Current State Snapshot (2025-09-30)
| Domain | State | Evidence |
|--------|-------|----------|
| Monorepo documentation hub | ✅ Sustained | `mono/docs/README.md`, `architecture/`, `operations/`, `playbooks/`, `references/` synced with Ark dossiers |
| Rust apps (desktop/web) | ✅ Building | `cargo check -p web` transcript (2025-09-29) post-Leptos 0.7 remediation |
| Rust services (agent/inference/retrieval) | ⚠️ Partial | `agent` builds with native transport (2025-09-30); `inference` + `retrieval` blocked on Candle/Qdrant API drift |
| Tests scaffolding | ✅ Organized | `mono/tests/` structure + README placeholders; smoke harness pending execution wiring |
| Unified JS workspace | ✅ Present | `unified/package.json`, `pnpm-workspace.yaml`, `agents/digest`, `packages/{kernel,shared,shims}`, `scripts/boot.ts` |
| Tooling CLIs | ✅ Available | `tools/{aos-ctl,aos-installer,aos-watchdog}` Cargo manifests and binaries |
| Agenticos archives | 🛈 Staged | `agenticos-*.zip` stored at repo root; extraction plan pending backlog item |
| Truth Gate bundle | ⚠️ Needs refresh | Root `FASTEMBED_NATIVE_COMPLETE.md` et al + `~/docs/` bundle awaiting new hashes/tests |

## Implementation Log — Completed
- [x] **2025-09-28** — Audited existing Ark OS assets and confirmed baseline alignment with NOA monorepo blueprint.
- [x] **2025-09-28** — Established `mono/docs/` hub with architecture, operations, playbooks, and references tracks.
- [x] **2025-09-28** — Expanded `mono/tests/` to include unit, integration, performance, and smoke suites alongside existing conformance checks.
- [x] **2025-09-28** — Authored this `sot.md` to govern subsequent workstreams.
- [x] **2025-09-29** — Repaired Leptos 0.7 compile breakages in `mono/apps/web/src/app.rs`; verified clean build with `cargo check -p web`.
- [x] **2025-09-30** — Replaced rmcp `process-wrap` dependency with native Tokio child transport, refreshed vendor checksums, and recorded `cargo check -p agent` failure surface beyond dependency resolution.

## Backlog — Active Workstreams
- [ ] **Smoke harness activation** — Implement `mono/tests/smoke/` runner covering `cargo check`, database migrations, and service boot.
  - Blockers: `services/inference` (Candle API drift) and `services/retrieval` (Qdrant client breakage) currently prevent full workspace builds.
  - Progress (2025-09-30): `services/agent` now compiles with native Tokio transport; outstanding handler dedup + missing deps to resolve post-harness draft.
- [x] 2025-10-01 04:21 UTC — **Service crate remediation** — Align `services/inference` and `services/retrieval` with upstream API changes, add regression coverage, and capture transcripts.
  - [x] 2025-10-01 04:08 UTC — Restored `services/agent` build (`cargo check -p agent`) by refreshing vendored dependencies, updating queue/MCP integrations, and reconciling ingest/crawler utilities. Evidence: `logs/cargo_check_agent_20251001_040843Z.log`, source updates in `services/agent/{src,Cargo.toml}`.
  - [x] 2025-10-01 04:20 UTC — `services/inference` builds cleanly (`cargo check -p inference`), confirming Candle/Tokio/Qdrant integration baseline. Evidence: `logs/cargo_check_inference_20251001_042059Z.log`.
  - [x] 2025-10-01 04:21 UTC — `services/retrieval` updated for modern Qdrant + FastEmbed APIs (alias cutover via `create_alias`/`delete_alias`, embedding engine modernization). Evidence: `logs/cargo_check_retrieval_20251001_042108Z.log`, `services/retrieval/src/{main.rs,embed.rs,migrate.rs}`.
- [ ] **Crate test seeds** — Introduce unit/integration suites for `crates/shared/` and `services/api/` controllers; document commands in `mono/docs/operations/`.
- [ ] **Agenticos package analysis** — Unpack `agenticos-*.zip`, inventory modules, and map adoption plan into `mono/docs/references/` with hash attestations.
- [ ] **Truth Gate refresh** — Regenerate `~/docs/{HASHES.txt,COVERAGE.md,FINAL_REPORT.md,REPRO.md,TEST/}` with Ark updates, including new build/test transcripts.
- [ ] **Unified workspace health check** — Run `pnpm install && pnpm lint` (or project-specific scripts) in `unified/`, document results, and sync TS configs with mono tooling where necessary.
- [ ] **Cache hygiene & diagnostics** — Clear stale build artifacts (`target/`, `.pnpm-store/`), verify Leptos assets rebuild cleanly, and note remediation steps.

## Verification Expectations
1. Every backlog item must reference produced artifacts and update this ledger upon completion.
2. Changes impacting verification assets must also refresh the Truth Gate bundle (`~/docs/{HASHES.txt,COVERAGE.md,FINAL_REPORT.md,REPRO.md,TEST/}`) with recorded Pass A/B/C outcomes.
3. When new tests or docs land, include reproducible command snippets in `mono/docs/operations/` or corresponding README files.

Maintain this ledger as the first stop for planning and audit trails. No additional workstreams should be undertaken unless listed above.
