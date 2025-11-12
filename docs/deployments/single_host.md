# Single-host deployment guide

The single-host topology runs the API gateway, MCP server, workflow orchestrator, and sandbox controllers on a single machine. Kernel-managed quotas (cgroup v2) cap resource usage while `server/src/adaptive_scaling.rs` makes dynamic adjustments using telemetry from `core::metrics`.

## When to choose this profile

- Developer workstations and lab servers without a separate control plane.
- Acceptance environments where CI/CD must validate the full control plane quickly.
- Edge or air-gapped deployments that prioritize determinism over horizontal scaling.

## Hardware sizing

| Tier | CPU | Memory | Storage | Notes |
| ---- | --- | ------ | ------- | ----- |
| Minimum | 8 vCPU | 32 GiB | NVMe SSD (0.5 TB) | Suitable for functional testing with lightweight inference. |
| Recommended | 16 vCPU | 64 GiB | NVMe SSD (1 TB) | Supports hybrid inference mode and concurrent sandboxes. |
| Performance | 24+ vCPU | 128 GiB | NVMe SSD (2 TB) + optional GPU | Allows full agent concurrency and heavyweight models. |

Network: 10 GbE preferred; enforce QoS for external control channels.

## Bootstrap procedure

1. Review and, if needed, edit `server/profiles/single_host/profile.toml`. Kernel quotas and telemetry thresholds feed the adaptive scaling controller.
2. Copy the profile and scripts to the host (default paths used by the assets assume `/opt/noa`).
3. Enable systemd target:
   ```bash
   sudo cp -r services/single-host /opt/noa/services/single-host
   sudo systemctl daemon-reload
   sudo systemctl enable noa-single-host.target
   sudo systemctl start noa-single-host.target
   ```
   Systemd units call `services/single-host/init/noa-single-host.sh`, which bootstraps each component while honouring kernel quotas.
4. Container-based deployments should use `services/single-host/container/entrypoint.sh`; set `NOA_SKIP_SYSTEMD=1` to run the init script directly within minimal images.

## Adaptive scaling

`server/src/adaptive_scaling.rs` interprets aggregated telemetry (`core::metrics`) and emits scaling decisions:

- **Idle**: full concurrency and heavyweight inference.
- **Steady**: baseline concurrency, default inference.
- **Elevated**: reduced concurrency with hybrid inference and small sandbox delay.
- **Saturated**: minimum concurrency and lightweight inference until pressure subsides.

The thresholds in `profile.toml` map directly to these load levels.

## CI/CD integration

The CRC → CI/CD automation includes a `single_host_acceptance` stage (`cicd/pipelines/crc-auto.yaml`). The Rust helper in `cicd/src/lib.rs` validates the manifest path (`server/profiles/single_host/profile.toml`) before deployments proceed. Ensure your pipelines either keep the default location or call `CICDSystem::configure_single_host_profile` with a custom path.

## Observability

- Metrics endpoint: `http://127.0.0.1:9310/metrics` (configurable in the profile).
- Control socket: `/var/run/noa/single-host.sock` for runtime coordination.
- Logs: `/var/log/noa` (default). Adjust via `NOA_LOG_DIR` before invoking the init script.

## Troubleshooting checklist

- `systemctl status noa-single-host@*.service` shows individual component health.
- Run `services/single-host/init/noa-single-host.sh status` for process inspection when systemd is unavailable.
- Ensure the kernel exposes cgroup v2 controllers (`mount | grep cgroup2`).
- Verify telemetry ingestion by calling `noa_core::metrics::current_load_level()` in diagnostics or checking the Prometheus scrape endpoint.
