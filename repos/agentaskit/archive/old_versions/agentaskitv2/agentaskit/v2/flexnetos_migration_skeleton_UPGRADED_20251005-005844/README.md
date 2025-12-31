
# FlexNetOS Migration Skeleton (UPGRADED)

**Timestamp:** 20251005-005844

Three-plane architecture:
- **Execution Plane:** single-binary core + WASM components + eBPF guards.
- **Sandbox Plane:** tri-sandbox (A/B/C) parallel runs → evolutionary merge → **Model D**.
- **Orchestrator Plane:** contracts, policies, capability schema, keys, strict push gate, Progress Tokens.

## Quick start
```bash
cd flexnetos_migration_skeleton_UPGRADED_20251005-005844
make init
make gen-sbom && make sign && make verify
make contract-test
make tri-run && make merge
make anchor
make promote        # optional
make hooks-install  # install strict pre-push gate
make orchestrator-sim
```
Offline-first. No Docker. Python 3 only; Cargo optional for core.
