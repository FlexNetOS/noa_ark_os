# Gateway incident response runbook

## Policy rejections
1. Inspect `services/gateway/telemetry/gateway-metrics.json` for the `rejected_policy` counter.
2. Compare affected client IDs to the manifest using `core.kernel.manifest.load_manifest()`.
3. Update `PolicyRule.allowed_paths` or rotate credentials if a misconfiguration is detected.

## Latency
1. Review `p95_latency_ms` from the telemetry export.
2. Validate runtime allocation via `runtime.manager.AdaptiveRuntimeController` to ensure accelerated backends are available.
3. If latency persists, throttle ingress using the rate limiter configuration and notify the runtime team.

## Rate limit saturation
1. Confirm rate limit thresholds align with the single-host profile envelope.
2. Capture a snapshot using `server.deploy.single_host.SingleHostOrchestrator.snapshot()` for auditing.
3. Coordinate with ecosystem owners when partner workloads cause sustained saturation.
