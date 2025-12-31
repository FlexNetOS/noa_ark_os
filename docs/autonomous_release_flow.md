# Autonomous Release Flow

The autonomous release loop now enforces verifiable trust before code reaches the default
branch. The flow is composed of four cooperating capabilities:

1. **Trigger trust gate** – `cicd/src/trigger.rs` reads trust metrics from `audit/ledger.jsonl`
   and only performs git merge previews when the in-flight drop meets both the AI
   confidence threshold and the historical trust average. Each preview produces a
   reproducible diff artifact in `audit/merges/` and persists an approval entry to the
   ledger.
2. **Rollback drills** – _(Planned: will use)_ `cargo run --manifest-path cicd/Cargo.toml --bin rollback_simulation`
   _(Note: The `rollback_simulation` binary is not yet implemented. This command will fail until it is added to the codebase.)_
   This step will perform a scheduled checkout of the previous commit in an isolated worktree. Results are
   stored under `audit/rollbacks/` and the ledger captures the simulation outcome to support
   audits.
3. **Audit bundles** – `make publish-audit` runs the `publish_audit` binary, which collects a
   signed SBOM, trust summary, and release metadata for the current workspace. Each bundle is
   saved under `audit/bundle-*` with a deterministic checksum signature and Python/Shell
   verification helpers.
4. **CI coverage** – `.github/workflows/autonomous-release.yml` runs the ledger-aware tests,
   executes rollback simulations, publishes an audit bundle, and verifies the emitted
   artifacts so regressions fail fast.

## Operating the system

- Run `make rollback-sim` locally to exercise the rollback drill. The command appends a
  `rollback_simulation` event to the ledger and fails fast if git worktrees cannot be
  prepared.
- Run `make publish-audit` after approving a drop. The target emits the signed bundle and
  automatically validates it through `audit/verify.sh`.
- Keep `audit/ledger.jsonl` under version control; it is the evidence chain tying trust
  decisions, rollbacks, and bundle publication together.

## CI integration

The `autonomous-release` workflow ensures the following invariants:

- Ledger-powered trust tests must pass (`cargo test -p noa_cicd`).
- Rollback simulation must complete successfully on the repository state checked out by CI.
- Audit bundles must verify using the shipped scripts, preventing checksum drift between
  environments.

This contract allows the automation to make forward progress without human intervention while
preserving an auditable record of every approval and recovery drill.
