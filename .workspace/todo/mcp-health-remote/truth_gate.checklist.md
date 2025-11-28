# Truth Gate Checklist — Fork MCP Health Guard

- [x] Artifacts hashed: `.workspace/todo/mcp-health-remote/evidence.ledger.json` lists SHA-256 values for every code/documentation/log file touched.
- [x] Smoke tests executed: `scripts/codex-bootstrap.sh`, local MCP health run, and remote-only MCP health run all exited 0 (see logs under `logs/codex-mcp-health/`).
- [x] Requirements ↔ artifacts mapping: captured in `claims.table.md`.
- [x] Limits and platform caveats noted per-claim (e.g., remote TLS/auth still out-of-scope, GitHub workflow pending CI observation).
- [x] Evidence ledger updated with command metadata and artifact hashes.
- [x] Gap scan: documented remaining risks around air-gapped hosts and third-party remote gateways in the claims table limits.
- [x] Triple-Verification: Pass A (diff + self-review), Pass B (reran bootstrap + local guard), Pass C (remote-only guard with independently launched noa_tools_agent).
