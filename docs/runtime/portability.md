# Runtime Portability & Target Profiles

The runtime manager inspects the hardware profile exposed by `noa_core::hardware` to dynamically select execution backends for language runtimes and model pipelines. This document captures the target profiles, optimization surfaces, and fallback behaviour that higher layers can rely on.

## Hardware Profiles

Hardware discovery ships with the following guarantees:

- **CPU profile** – vendor string, brand, logical/physical core counts, and base frequency in MHz.
- **Memory profile** – total and currently available memory (bytes) plus convenience helpers for GiB conversions.
- **GPU profile** – detected vendors (NVIDIA, AMD, Intel, Apple), optional memory totals, and driver metadata.
- **Accelerator profile** – GPU entries plus optional TPU/NPU flags surfaced via environment variables.

The resulting `HardwareProfile` struct is serialisable and safe to forward to orchestration or remote policy engines.

## Target Profiles

CRC automation and the runtime manager reason about two coarse target categories:

| Profile | Description | Primary Backends |
|---------|-------------|------------------|
| `edge`  | Thermally constrained systems such as Jetson-class boards or small form-factor PCs. | `llama.cpp` CPU, lightweight Python runtime |
| `server`| High-throughput servers with ample memory, GPUs, or custom accelerators. | `llama.cpp` GPU (when policy allows), full CPython runtime |

Each profile can be expanded with additional metadata in the generated manifests located under `storage/artifacts/<profile>/<drop-id>/`.

## Optimization Flags

| Profile | Example Flags | Feature Gates |
|---------|----------------|----------------|
| `edge`  | `-C opt-level=z`, `-C lto=fat`, `-C codegen-units=1` | `low-power`, `quantized-kernels` |
| `server`| `-C opt-level=3`, `-C lto=thin` | `throughput`, `gpu-offload` |

The CRC build automation writes these settings to `flags.txt` and `manifest.yaml` for each drop to keep CI/CD pipelines deterministic.

## Backend Selection Policy

The runtime manager reads `RuntimePolicy` definitions (defaults shown below) to produce a `RuntimePlan`:

```yaml
prefer_gpu: true
min_gpu_memory_gb: 8.0
prefer_lightweight_python_on_low_memory: true
lightweight_memory_threshold_gb: 6.0
allow_accelerator_experiments: true
```

Selection behaviour:

1. **llama.cpp backend** – chooses GPU execution when a GPU meets the minimum memory threshold. Otherwise, falls back to CPU builds (which are always listed as a fallback option in the plan).
2. **Python runtime** – selects the lightweight runtime when total or available memory drops below the configured threshold, otherwise defaults to full CPython.
3. **Accelerator orchestration** – if `allow_accelerator_experiments` is enabled and TPUs/NPUs are detected, the plan adds an experimental offload entry while still providing CPU/GPU fallbacks.

## Fallback Semantics

Generated runtime plans always include conservative fallbacks:

- `llama.cpp` CPU builds remain available even when GPU acceleration is selected.
- The lightweight Python runtime is always listed as a fallback when CPython is chosen.
- Accelerator experiments never remove the standard GPU/CPU options.

These guarantees ensure deployments can degrade gracefully when hardware availability changes between planning and execution.
