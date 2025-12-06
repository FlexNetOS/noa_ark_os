# NOA Ark OS Integration Plan

## Phase 1 – Baseline Inventory & Repo Sync
- [x] Catalogue existing crates/services; identify missing Rust ecosystem components. (See docs/integration/phase1_inventory.md)
- [ ] Mirror required upstream repositories into `repos/` with commit references.
- [ ] Update Cargo workspace manifests; verify with `cargo metadata`.
- [ ] Refresh `.env` / `.env.example` with core environment variables.
- [ ] Document outcomes/gaps in project SoT and audit notes.

## Phase 2 – Core Runtime & Bindings
- [ ] Integrate RustPython + PyO3 scaffolding with smoke tests.
- [ ] Extend env loader (APP_ENV_FILE) to runtime crate and update docs.
- [ ] Record configuration/test artefacts in SoT and audit tree.

## Phase 3 – Platform Frameworks
- [ ] Add Tauri desktop/mobile shells (hello world) under workspace.
- [ ] Introduce Axum service skeleton using env loaders.
- [ ] Capture build/test logs and update `.env` + SoT.

## Phase 4 – ML Stack
- [ ] Stage Candle inference module with placeholder model.
- [ ] Stage Burn training pipeline stub; add feature flags.
- [ ] Document hardware requirements and test results.

## Phase 5 – Agent & Data Services
- [ ] Integrate Rig orchestrator sample flow.
- [ ] Configure Qdrant client, rust-postgres, redis-rs modules with env settings.
- [ ] Verify connectivity (mock/integration tests) and log outputs.

## Phase 6 – Documentation & Audit Refresh
- [ ] Update SoT executed/backlog sections per phase.
- [ ] Refresh audit bundles and hash manifests.
- [ ] Ensure `.env`/`.env.example` mirror final configuration and overlays.

