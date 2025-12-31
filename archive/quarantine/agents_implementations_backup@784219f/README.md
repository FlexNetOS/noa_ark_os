# Quarantine: Legacy Agent Implementations Backup

**Original path:** `agents/src/implementations/_backup/`
**Archived on:** 2025-11-15
**Reason:** Dead board/executive implementations from the agentaskit drop still triggered warnings and violated the "Heal, Don't Harm" policy. Per [docs/projects/ROADMAP_AGENTIC_KERNEL.md](../../../docs/projects/ROADMAP_AGENTIC_KERNEL.md) (Phase 0 – Dead Code Quarantine), unused modules must be moved under `archive/quarantine/` with provenance so future reintegration is deliberate.

## Reintegration Checklist
1. Reintroduce only the agents that will be exercised by tests (board and executive suites stay gated until runtime wiring exists).
2. Update `agents/src/implementations/mod.rs` to expose the revived modules.
3. Add integration or unit tests proving each agent interacts with the registry/factory without warnings (treat warnings as errors via `RUSTFLAGS='-D warnings'`).
4. Document the change and update this README + `status.yaml` with the new commit hash.

All files are preserved verbatim so restoration is a simple move back into `agents/src/implementations/`.
