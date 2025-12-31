# Profile validation and health probes

Each server profile shipped in `server/profiles/` must prove that its capability constraints map cleanly onto the kernel before
shipping. The workflow below combines manual checks with automated CI gates so that the single-host, air-gapped, devcontainer,
and edge_lite profiles all deliver the expected storage roots, tool allow lists, and network policies.

## Local validation commands

1. **Generate capability tokens**
   ```bash
   cargo test -p noa_core profile_switching -- --nocapture
   ```
   The test loads every profile in sequence and asserts that each capability token includes the declared tools, egress mode,
   budgets, and storage roots. It also verifies the air-gapped profile reports `egress_mode = denied`.

2. **Dry-run bootstrap scripts** (for profiles with init helpers)
   ```bash
   NOA_DRY_RUN=1 \
   NOA_PROFILE=server/profiles/single_host/profile.toml \
     services/single-host/init/noa-single-host.sh start all || true
   ```
   Adjust `NOA_PROFILE` for alternate profiles if they provide dedicated init flows.

## Continuous integration coverage

`cicd/pipelines/crc-auto.yaml` adds a `profile_health_matrix` step that executes the same capability-token regression under CI.
The step iterates over:

- `server/profiles/single_host/profile.toml`
- `server/profiles/air_gapped/profile.toml`
- `server/profiles/devcontainer/profile.toml`
- `server/profiles/edge_lite/profile.toml`

For each manifest the job sets `NOA_PROFILE` and runs `cargo test -p noa_core profile_switching --release`. The pipeline fails if
any profile omits required health data (for example, storage roots or the air-gapped egress denial).

Together these checks guarantee that the kernel emits capability tokens aligned with every supported deployment topology, and
that isolated environments remain sealed even when exercised automatically in CI.
