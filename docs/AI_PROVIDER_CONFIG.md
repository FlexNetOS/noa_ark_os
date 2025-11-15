# AI Provider Configuration

This guide describes how to enable the Kanban AI Assist endpoint and switch between supported model providers.

## Environment variables

Set the following variables before running `corepack pnpm dev` or `corepack pnpm start` from `ui/vibe-kanban/`:

| Variable | Description |
| --- | --- |
| `AI_PROVIDER` | Provider selector. Accepted values: `llama.cpp`, `openai`, `anthropic`. Leave unset to skip remote calls. |
| `AI_PROVIDER_CHAIN` | Optional comma-separated fallback order (for example `openai,anthropic,llama.cpp`). The router will fail over in this order. |
| `LLAMA_CPP_ENDPOINT` | Base URL for a local llama.cpp server when `AI_PROVIDER=llama.cpp`. |
| `LLAMA_CPP_MODEL` | Friendly model name reported in telemetry when llama.cpp is active. |
| `OPENAI_API_KEY` | API key for OpenAI models. |
| `OPENAI_BASE_URL` | Optional override for hosted gateways or proxies. Defaults to `https://api.openai.com`. |
| `OPENAI_MODEL` | Optional override for the deployed model identifier. |
| `OPENAI_ORG` | Optional OpenAI organisation identifier forwarded in requests. |
| `OPENAI_TIMEOUT_SECS` | Request timeout budget applied to the OpenAI client (default `30`). |
| `ANTHROPIC_API_KEY` | API key for Anthropic models. |
| `ANTHROPIC_BASE_URL` | Optional override for gateways wrapping the Anthropic API. Defaults to `https://api.anthropic.com`. |
| `ANTHROPIC_MODEL` | Optional model identifier override. |
| `ANTHROPIC_VERSION` | Anthropic API version header value (default `2023-06-01`). |
| `ANTHROPIC_TIMEOUT_SECS` | Request timeout budget applied to the Anthropic client (default `30`). |

> **Tip:** The AI Assist feature always returns the rendered engineer prompt. When a provider is configured, the response also includes the provider completion payload.

## Quick start: llama.cpp

1. Download a llama.cpp binary for your platform (GPU builds recommended). Extract it into a local folder.
2. Launch the server with an OpenAI-compatible HTTP surface, for example:
   ```bash
   ./llama-server -m /path/to/model.gguf -c 4096 --host 0.0.0.0 --port 8080 --api-server
   ```
3. Set the endpoint and provider variables before starting Next.js:
   ```bash
   export AI_PROVIDER=llama.cpp
   export LLAMA_CPP_ENDPOINT=http://127.0.0.1:8080/v1
   corepack pnpm --filter vibe-kanban dev
   ```
4. Open the Kanban UI, click the **AI** button on any card, and copy the generated prompt into your IDE or provider client.

## Observability & logging

- Each request is stored in `ui/vibe-kanban/.data/ai_assist.sqlite` with latency, provider, and status metadata.
- Use the exported `listRecentAiRequests` helper from `ui/vibe-kanban/server/ai-database.ts` to build custom admin dashboards or CLI summaries.
- Rate limits default to five requests per minute per client identity. Adjust by editing `ui/vibe-kanban/server/rate-limiter.ts`.
- CLI operators can stream completions via `noa agent invoke --stream` or run quick checks with `noa query`. Both commands emit
  `InferenceMetric` entries so that CLI usage contributes to the Evidence Ledger alongside service traffic.
