# Gateway Telemetry Storage

This directory holds OpenTelemetry-compatible artefacts emitted by the programmable gateway. The `noa_gateway` crate exports structured metrics under `gateway_metrics.json` and appends span-aligned events to `gateway_events.log` so downstream collectors can ingest them.

Notebook automation runs now emit dedicated telemetry in `notebook_gateway_automation.jsonl`. Each line captures:

- `access_scope.fs` / `access_scope.network` to trace the capability token scopes exercised.
- `token_rate_limit` to monitor rate consumption relative to policy.
- `trigger_source` (`scheduler`, `user-api`, etc.) for governance attribution.
- `status` and optional `throttle_reason` when rate limits engage.

Files are generated automatically at runtime. It is safe to clean the directory between test runs.

For offline development and tests, synthetic examples are checked into
`gateway_metrics.json` and `gateway_events.log`. The
`server.python.autonomy.telemetry_loader` helpers load these samples so gateway
traffic can be replayed deterministically during unit tests and local demos.
