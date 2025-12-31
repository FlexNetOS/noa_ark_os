# Quarantined Agent Implementations (Board & Executive)

**Source path:** `agents/src/implementations/{board, executive}`  
**Archived:** 2025-11-14  
**Reason:** Cargo test runs surfaced dozens of `unused` warnings across these modules (unused imports, private fields, unreachable helpers). Per [docs/projects/ROADMAP_AGENTIC_KERNEL.md](../../../docs/projects/ROADMAP_AGENTIC_KERNEL.md) (Phase 0 – Dead Code Quarantine), noisy, inactive components must be moved out of the active build so the kernel stays warning-free and reversible.

## Status

The code in this directory is intact and can be reactivated once the board/executive agent suite is wired into the registry and runtime. Reintegration should:

1. Reintroduce the modules under `agents/src/implementations/` (or behind a feature gate) only after their APIs are exercised by tests.
2. Add coverage for each agent role (board finance/legal/strategy, executive emergency/priority/resources/etc.).
3. Ensure `cargo test -p noa_agents` runs without warnings (set `RUSTFLAGS='-D warnings'` locally to verify) before moving back.
4. Update this README + `status.yaml` with the new commit hash when reintegrated.

While quarantined, these files are intentionally excluded from the crate to keep CI green and to respect the “Heal, Don’t Harm” principle.
