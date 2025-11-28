# Single-host bootstrap assets

This directory contains bootstrap assets for the `single_host` server profile:

- `systemd/` — unit files that co-locate the API gateway, MCP server, workflow orchestrator, and sandbox controllers under a shared `noa-single-host.target`.
- `init/noa-single-host.sh` — thin wrapper that respects kernel-enforced quotas and starts/stops the colocated services.
- `container/entrypoint.sh` — container-friendly entrypoint that either activates the systemd target or launches the services directly when `NOA_SKIP_SYSTEMD=1`.

All scripts expect the profile manifest at `server/profiles/single_host/profile.toml`. Override the default by exporting `NOA_PROFILE` before invoking the entrypoint or init script.
