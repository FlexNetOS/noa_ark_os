# llama.cpp Integration (staging)

This integration wires llama.cpp into AgentAsKit behind an optional feature flag and a selector bridge.

Contents:
- `fetch.sh` — clones llama.cpp at a pinned commit
- `build.sh` — example build (Linux) for CPU; extend for CUDA/Metal as needed
- `../../core/src/ai/model_selector_bridge.rs` — selector bridge skeleton (feature-gated)

Usage:
```
./fetch.sh
./build.sh
```

Upstream: https://github.com/ggerganov/llama.cpp

Notes:
- Do not commit large binaries; use build artifacts locally or CI cache.
- Default workspace build remains unaffected; enable via a feature flag in code when available.
