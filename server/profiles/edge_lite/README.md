# Edge Lite Server Profile

The **edge_lite** profile targets constrained edge appliances that must operate with minimal compute and telemetry-only egress.
It keeps the runtime footprint small while preserving auditability and remote observability.

## Tooling policy
- **Allowed**: `bash`, `busybox`, `python3`, `rsync`
- **Blocked**: `docker`, `git`, `node`

## Network egress
- Mode: **allow list** (telemetry only)
- Domains: `telemetry.noa-ark.local`
- Notes: The upstream control plane must initiate any command-and-control traffic.

## Resource budgets
- CPU: reserve 1 core (max 2) for real-time scheduling
- Memory: soft cap 4 GiB, hard cap 6 GiB
- Network: 50 Mbps sustained, 100 Mbps burst to drain telemetry buffers

## Storage roots
- `/opt/noa/workspaces` (read-write, 4 GiB) — lightweight execution workspace
- `/opt/noa/telemetry` (read-write, 2 GiB) — buffered metrics awaiting uplink
- `/opt/noa/artifacts` (read-only, 2 GiB) — signed binaries and model bundles

## Validation
- Manual: `cargo test -p noa_core profile_switching -- --nocapture`
- CI: the `profile_health_matrix` step in `cicd/pipelines/crc-auto.yaml` exercises the profile with
  `NOA_PROFILE=server/profiles/edge_lite/profile.toml`.
