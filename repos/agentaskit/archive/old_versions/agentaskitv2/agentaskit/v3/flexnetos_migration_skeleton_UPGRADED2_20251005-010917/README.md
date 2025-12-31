
# FlexNetOS Migration Skeleton (UPGRADED2)

**Timestamp:** 20251005-010917

This build adds:
- **Cap'n Proto UDS server** wired in Rust core (`execution/core/`).
- **NUMA/Hugepages helpers** (`tools/numa_pin.sh`, `tools/hugepages.sh`).
- **Stronger contract test** (invokes `capnp compile` when present; strict mode via `CAPNP_STRICT=1`).
- **Cross-check report** (`orchestrator/crosscheck_report.md`).

Quick start:
```bash
cd flexnetos_migration_skeleton_UPGRADED2_20251005-010917
git init
make hooks-install
make init
make gen-sbom && make sign && make verify
make contract-test
make tri-run && make merge
make anchor
make promote
# Build and run the UDS server if cargo+capnp installed:
make build-core
# (Run in another shell): execution/core/target/release/flex-core
```
