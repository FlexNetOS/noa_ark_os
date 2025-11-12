# Documentation Refresh Workflow

## Trigger
- Merge to `main`
- Manual rerun of documentation pipeline

## Steps
1. Run CI job `docs-refresh` to generate diff summary and approvals.
2. Execute documentation agent via `cargo run -p noa_agents --bin documentation_sync`.
3. Update Markdown artifacts (`docs/documentation`, `docs/wiki`, `.workspace/sop`).
4. Publish runbook updates under `docs/runbook/`.
5. Notify release management channel with summary and links.

## Verification
- `docs/wiki/index.md` exists and is up to date.
- `.workspace/sop/automation-status.md` reflects latest SOP state.
- `docs/runbook/index.md` includes verification matrix entry for each SOP.

## Rollback
- Re-run documentation agent with previous pipeline payload.
- Restore wiki and SOP artifacts from git history.
