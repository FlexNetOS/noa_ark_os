# Dev Container Server Profile

The **devcontainer** profile powers IDE containers and ephemeral development pods. It exposes a broader toolchain for build and
test workflows while constraining network egress to audited registries.

## Tooling policy
- **Allowed**: `bash`, `cargo`, `docker`, `git`, `node`, `pnpm`, `python3`
- **Blocked**: `curl`, `pip`

## Network egress
- Mode: **allow list**
- Domains: `github.com`, `registry.npmjs.org`, `crates.io`, `index.docker.io`
- Notes: Connectivity beyond the allow list must be tunnelled through the gateway for auditing.

## Resource budgets
- CPU: reserve 4 cores (max 8) to keep build spikes predictable
- Memory: soft cap 16 GiB, hard cap 32 GiB for long-lived compilation tasks
- Network: 400 Mbps sustained, 800 Mbps burst to accommodate registry downloads

## Storage roots
- `/workspaces/noa` (read-write, 20 GiB) — active workspace volume mounted by IDEs
- `/workspaces/noa/toolchains` (read-only, 10 GiB) — curated compilers and SDKs
- `/workspaces/noa/cache` (read-write, 15 GiB) — build caches preserved between container restarts

## Validation
- Manual: `cargo test -p noa_core profile_switching -- --nocapture`
- CI: the `profile_health_matrix` step in `cicd/pipelines/crc-auto.yaml` validates capability tokens with
  `NOA_PROFILE=server/profiles/devcontainer/profile.toml`.
