# Air-gapped Server Profile

The **air_gapped** profile runs the unified server inside strictly isolated labs. It forbids outbound network traffic, limits the
runtime to audited offline tooling, and constrains compute budgets so emergency shutdowns remain deterministic even without
external orchestration.

## Tooling policy
- **Allowed**: `bash`, `cargo`, `pytest`, `rustc`, `tar`
- **Blocked**: `curl`, `wget`, `git`

## Network egress
- Mode: **denied** (no outbound sockets)
- Exceptions: _none_
- Notes: Operators must sideload models, datasets, and updates through removable media audited by the evidence ledger.

## Resource budgets
- CPU: reserve 2 cores (max 4) for deterministic scheduling
- Memory: soft cap 8 GiB, hard cap 12 GiB
- Network: egress capped at 0 Mbps to enforce offline execution

## Storage roots
- `/srv/noa/workspaces` (read-write, 10 GiB) — transient workspaces for kernels and sandboxes
- `/srv/noa/artifacts` (read-only, 5 GiB) — vetted binaries and model snapshots
- `/srv/noa/evidence` (read-write, 2 GiB) — signed audit trails and ledger checkpoints

## Validation
- Manual: `cargo test -p noa_core profile_switching -- --nocapture`
- CI: the `profile_health_matrix` step in `cicd/pipelines/crc-auto.yaml` executes the same probe with
  `NOA_PROFILE=server/profiles/air_gapped/profile.toml` to verify health in isolation.
