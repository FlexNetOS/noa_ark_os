
# FlexNetOS Migration Skeleton (UPGRADED3)

**Timestamp:** 20251005-011446

Adds:
- **Rust core load-time verification** of `artifacts/MANIFEST.sha256` against local files, plus optional **minisign** check if `FLEX_MINISIGN_PUB` and `minisign` are available.
- **Cap'n Proto UDS server + client** in one crate (bins: `flex-core`, `flex-client`).
- **Make targets:** `run-core` (NUMA-pinned if helper present) and `smoke-client` (starts server, hits it, shuts down).
- **NUMA/Hugepages helpers** (`tools/numa_pin.sh`, `tools/hugepages.sh`).
- **Contract test** compiles `contracts/inference.capnp` when `capnp` exists (`CAPNP_STRICT=1` to enforce).

## Quick path
```bash
cd flexnetos_migration_skeleton_UPGRADED3_20251005-011446
git init
make hooks-install

make init
make gen-sbom && make sign && make verify
CAPNP_STRICT=1 make contract-test

make build-core
make run-core          # run server (press Ctrl+C to stop)
# in another shell:
make smoke-client      # or run client directly

make tri-run && make merge
make anchor
make promote
```
