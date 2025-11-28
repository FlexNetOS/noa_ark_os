# Changelog

## [Unreleased]

### Added

- `docs/guides/AGENTIC_OS_GUIDE.md` outlining operator workflows, policy
  enforcement, and CLI-first execution patterns.
- Kernel sovereignty and tokenized gateway diagrams under `.graphs/workflows/`
  with embedded copies in the new guide.
- Repository-wide markdown linting via `pnpm docs:lint` and `make docs:check`.

### Updated

- `AGENT.md` cross-links to the new guide and codifies Phase 0.5–10 patterns
  (snapshots, capability tokens, registry-only execution).
- `docs/guides/README.md` index now references the Agentic OS Operator Guide.
- Portable toolchain activators now redirect through `server/tools/activate-toolchains.sh`,
  so calling `activate-{cargo-wsl,node}.sh` brings up PowerShell/Cargo/Node together, the
  checker reports both binary/current link paths, and the launcher logs the resulting
  PowerShell activation evidence (manifest hash, binary path, env flags) inside the phase
  summary output.
