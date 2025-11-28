# Gateway Operations Runbook

This guide captures day-2 procedures for the programmable gateway introduced in `server/gateway`.

## Components

- **Programmable Router** – GraphQL federation, gRPC proxying, and WebSocket multiplexing handled by the `ProgrammableRouter`
  in `server/gateway/src/router.rs`.
- **Policy Enforcement** – `PolicyEnforcer` delegates to `core::security` so agent and account permissions remain the source of truth.
- **Unified AuthN/Z** – `UnifiedAuthenticator` validates mTLS fingerprints, OpenID Connect tokens, and API keys with per-agent
  fallbacks.
- **Agent-aware Rate Limiting** – The `RateLimiter` draws layer-aware limits from the hive mind metadata shipped in the
  `agents` crate.
- **Telemetry Export** – The `TelemetrySink` emits OpenTelemetry-compatible spans and metrics in `storage/telemetry`.

## Daily Checks

1. `cargo test -p noa_gateway` – Ensures routing, policy, and auth flows compile and execute with the embedded agent registry.
2. Inspect `storage/telemetry/gateway_metrics.json` for volume spikes and verify the UI dashboard renders counts.
3. Review recent `gateway_events.log` entries to confirm spans contain routing targets and agent identities.

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
| Missing telemetry files | The gateway creates `storage/telemetry/` on demand. Ensure the filesystem is writable and that the service account has permission. |
| Dashboard empty | Open the browser console. If the fetch path is incorrect adjust the static server root so the dashboard can read `storage/telemetry/gateway_metrics.json`. |

## Escalation

If policy enforcement or registry data appears corrupted, re-run `cargo test` to reproduce locally and escalate to the
orchestrator oversight agent listed in `core/README.md`.
