# Single Host Server Profile

The **single_host** profile co-locates the API gateway, MCP server, workflow orchestrator, and sandbox controllers on a single machine. Kernel-managed quotas (cgroup v2) ensure that each service respects CPU, memory, IO, and network limits without requiring an external control plane.

## Key Characteristics

- **Co-location**: All critical control plane components share a host, reducing cross-node latency and simplifying developer or lab deployments.
- **Kernel quotas**: CPU, memory, IO, and network budgets are enforced through kernel controllers; the runtime surfaces telemetry for adaptive scaling policies.
- **Adaptive friendly**: Threshold values under `[telemetry.*]` mirror the defaults consumed by `server/src/adaptive_scaling.rs`, enabling the server to throttle concurrency and inference intensity before saturation.
- **Boot orchestration**: The `boot.steps` section mirrors the order executed by the systemd target and container entrypoint scripts under `services/single-host/`.

## Usage

1. Point `NOA_PROFILE` to `server/profiles/single_host/profile.toml`.
2. Start services via `services/single-host/init/noa-single-host.sh start all` or by enabling the `noa-single-host.target` systemd unit.
3. Observe telemetry on the endpoint defined at `[profile.runtime.metrics_endpoint]` to drive adaptive scaling decisions.

Consult `docs/deployments/single_host.md` for end-to-end deployment guidance, hardware sizing, and CI/CD expectations.
