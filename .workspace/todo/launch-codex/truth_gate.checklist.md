# Truth Gate Checklist — Launch Codex

- [x] Artifacts hashed: see `.workspace/todo/launch-codex/evidence.ledger.json` for SHA-256 values of every modified file.
- [x] Smoke tests executed: `scripts/codex-bootstrap.sh` and `cicd/scripts/codex-mcp-health.sh` both exited 0 (logs captured in terminal transcript).
- [x] Requirements ↔ artifacts mapping: documented in `claims.table.md`.
- [x] Limits, supported configs, and caveats recorded alongside each claim.
- [x] Evidence ledger updated with file hashes and command metadata.
- [x] Gap scan: noted that Windows/macOS builds and remote NOA_TOOLS_SERVER_URL targets remain unvalidated.
- [x] Triple-Verification: Pass A (self-check) by reviewing diffs, Pass B (rerun bootstrap), Pass C (health guard exercising real RPC calls).
