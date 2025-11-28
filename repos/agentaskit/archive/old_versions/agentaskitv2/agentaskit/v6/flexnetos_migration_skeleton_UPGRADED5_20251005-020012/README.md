# FlexNetOS Migration Skeleton (UPGRADED5)

**Timestamp:** 20251005-020012

Adds:
- **WASM connectors (capability tokens, WASI sandbox)**: host at `execution/wasm_host/` with `wasmtime` + `wasi`, demo connectors in `execution/connectors/`.
- **fs-verity signing** helper and Make targets.
- **Mount policy enforcement**: server refuses to start if `artifacts/` is not read-only when `FLEX_ENFORCE_MOUNT_RO=1`.
- **Imported artifacts from your files** under `docs/imported/`, linked in `docs/agent_map_crosswalk.md`.
