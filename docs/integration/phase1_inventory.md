# Phase 1 Inventory — NOA Ark OS (Rust Ecosystem)

## Workspace Snapshot
- Cargo workspace file: `mono/Cargo.toml`
- Current members (abridged): crates (`shared`, `globals`, `obs`, `config`, `pybridge`, `rustpyshim`, `wasm-infer`), services (`api`, `agent`, `inference`, `retrieval`, `analytics`), apps (`web`, `desktop`), tools (`sbom`, `aos-bundler`, `aos-installer`, `aos-ctl`, `aos-watchdog`), trainer (`burn`).
- Global dependencies already declared: `axum`, `tokio`, `serde`, `candle-*`, `pyo3`, `qdrant-client`, `opentelemetry`, `hf-hub`, `tokenizers`, etc.

## Coverage vs Target Stack
| Component | Status | Notes |
|-----------|--------|-------|
| Cargo workspace | ✅ | Workspace members defined under `mono/Cargo.toml`; `cargo metadata` runs successfully. |
| RustPython | ⚠️ Present via `crates/rustpyshim` & `crates/pybridge`, further validation needed. |
| PyO3 | ⚠️ Declared in workspace dependencies but not enabled in crates. |
| Tauri | ⚠️ `apps/desktop` exists but integration level TBD; requires verification. |
| Axum | ✅ Used by services (api, packaging). |
| Candle | ⚠️ `candle-core` patched vendor exists; need inference crate wiring/tests. |
| Burn | ⚠️ Trainer crate exists; Burn deps commented out. |
| Rig | ❌ Not present; requires upstream repo. |
| Qdrant client | ⚠️ Declared dependency; check usage in retrieval service. |
| rust-postgres | ❌ Not present (using other DB?). |
| redis-rs | ❌ Not present. |

## Repositories Mirror
- `/home/deflex/workspace/.projects/noa_ark_os/repos` — currently empty.
- TODO: mirror upstream repos (Rig, Qdrant samples, redis connectors, etc.) with commit hashes.

## Environment Files
- Workspace `.env` targets not yet aligned with `mono/` env loaders; need to propagate APP_ENV_FILE pattern across services.

## Commands Executed
```
cargo metadata --format-version 1 --no-deps  # at mono/
```

Outputs logged via terminal history.

## Next Steps
1. Mirror upstream repositories into `repos/` with lockfiles/commit refs.
2. Update workspace manifests if new crates are introduced.
3. Extend `.env` / `.env.example` with placeholders for DBs (Postgres, Redis, Qdrant, etc.).
4. Plan feature-specific integrations in subsequent phases.

