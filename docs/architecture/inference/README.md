# Inference Pipeline Operations

The production inference stack now routes every completion request through the shared
`ProviderRouter` and exports observability signals into the workflow instrumentation ledger.
This document captures the critical behaviours for platform, CLI, and MLOps owners.

## Provider chaining and failover

- The router accepts a comma-delimited provider chain via `AI_PROVIDER_CHAIN` (for example
  `openai,anthropic,llama.cpp`).
- Health checks run before each invocation. Providers that respond with non-success status or
  time out are skipped automatically, with telemetry documenting the failure reason.
- Fallback providers receive the original `CompletionRequest` without modification, enabling
  model downgrades when a premium endpoint is unavailable.

## Streaming completions

- OpenAI, Anthropic, and llama.cpp backends expose streaming through `Provider::stream`. The
  router wraps each stream in an instrumented guard that records latency, tokens, and the final
  status once the stream completes.
- CLI users can access the stream through `noa agent invoke --stream` or
  `noa query --stream`. Output is flushed chunk-by-chunk to STDOUT for real-time feedback.

## Telemetry and the Evidence Ledger

- Every success or failure generates an `InferenceMetric` entry written to the
  `inference_metrics.log` channel managed by `PipelineInstrumentation`.
- Metrics record provider ID, model, request latency, prompt and completion token counts, and any
  surfaced error message. These logs back the Evidence Ledger under
  `storage/db/evidence/ledger.jsonl`.
- The CLI and gateway services attach telemetry through the `TelemetrySink` trait, ensuring that
  agent or workflow invocations automatically emit metrics without duplicating logging logic.

## CLI integration

- `noa agent invoke "<prompt>" [--stream]` issues prompts through the provider chain and prints
  the generated completion. The command relies on the router for failover and telemetry.
- `noa query "<question>" [--stream]` provides a lightweight natural-language query path that uses
  the same router instance. The output is prefixed with `Response:` to aid piping into tooling.
- Both commands initialise `PipelineInstrumentation` on demand so that agent operators inherit the
  same metrics exported by long-running services.

## MLOps responsibilities

- The MLOps crew curates gateway manifests and environment inventory entries for provider keys,
  timeout budgets, and organisational identifiers.
- Provider failures and degradations are surfaced through the telemetry log, enabling automated
  alerting and historic SLA analysis without bespoke instrumentation.
- Health checks should be exercised regularly using `noa agent invoke` with a low-cost prompt to
  confirm primary → fallback transitions.
