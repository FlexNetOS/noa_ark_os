# Gateway Operations Runbook

This guide captures day-2 procedures for the programmable gateway introduced in `server/gateway`.

## Components

- **Programmable Router** – GraphQL federation, gRPC proxying, and WebSocket multiplexing handled by the `ProgrammableRouter`
  in `server/gateway/src/router.rs`.
- **Policy Enforcement** – `PolicyEnforcer` delegates to `core::security` so agent and account permissions remain the source of truth.
- **Unified AuthN/Z** – `UnifiedAuthenticator` validates mTLS fingerprints, OpenID Connect tokens, and API keys with per-agent
  fallbacks.
- **Agent-aware Rate Limiting** – The `RateLimiter` draws layer-aware limits from the hive mind metadata shipped in the
  `agents` crate and persists quota state in `storage/rate_limits/gateway_rate.db` (override with `GATEWAY_RATE_DB`).
- **Persistent Quota Store** – Distributed nodes share the same SQLite-backed rate store, so exhaust/restore events survive
  restarts and horizontal scaling. Replicate the `.db` file via the standard storage sync job.
- **Telemetry Export** – The `TelemetrySink` emits OpenTelemetry-compatible spans and metrics in `storage/telemetry`.

## Daily Checks

1. `cargo test -p noa_gateway` – Ensures routing, policy, and auth flows compile and execute with the embedded agent registry.
2. Inspect `storage/telemetry/gateway_metrics.json` for volume spikes, including the new `rate_limit` section that tracks
  total checks/denies per layer.
3. Review recent `gateway_events.log` entries to confirm spans contain routing targets and agent identities.
4. Validate that `storage/rate_limits/gateway_rate.db` is writable and replicating downstream; stale timestamps indicate the
  rate limiter has fallen back to in-memory mode.

## Agent Role Assignments

| Agent Role | Responsibilities |
| --- | --- |
| **Orchestrator** | Schedules daily checks, reviews escalation outcomes, and coordinates remediation windows. |
| **Planner** | Updates routing manifests, policy inputs, and dependency graphs when configuration changes are required. |
| **Worker** | Executes gateway commands (`cargo test`, log inspection, telemetry collection) and applies corrective scripts. |
| **Verifier** | Confirms telemetry integrity, validates test results, and records evidence in the operations ledger. |

## Troubleshooting

| Symptom | Action |
| --- | --- |
| `authentication failed` errors | Confirm the caller passes at least one credential type. API keys must match the trusted list in `UnifiedAuthenticator::default` or be accompanied by a valid mTLS fingerprint. |
| `rate limit exceeded` errors | Validate the agent is registered in the hive mind. Agents missing from the registry default to conservative limits; load the CSV via `AgentRegistry::load_from_csv` if testing custom agents. |
| Quotas reset after restart | Check that `storage/rate_limits/gateway_rate.db` exists and is writable. Override the location via `GATEWAY_RATE_DB=/path/to/db` if the default storage volume is ephemeral. |
| Telemetry missing `rate_limit` section | Ensure `TelemetrySink::record_rate_limits` is invoked by confirming the gateway binary is at least version `>=2025.11`. Delete `gateway_metrics.json` to allow the sink to rewrite it if the structure is stale. |
| Missing telemetry files | The gateway creates `storage/telemetry/` on demand. Ensure the filesystem is writable and that the service account has permission. |
| Dashboard empty | Open the browser console. If the fetch path is incorrect adjust the static server root so the dashboard can read `storage/telemetry/gateway_metrics.json`. |

## Escalation

If policy enforcement or registry data appears corrupted, re-run `cargo test` to reproduce locally and escalate to the
orchestrator oversight agent listed in `core/README.md`.
