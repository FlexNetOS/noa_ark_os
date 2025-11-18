# NOA Core OS Layer (`noa_core`)

The `noa_core` crate is the foundational operating system layer for NOA ARK OS.
It is implemented in Rust for safety, determinism, and observability, and is
designed to be driven by autonomous agents as much as by humans.

## Responsibilities

- Kernel capability graph and profile management
- Process and thread lifecycle management
- Memory and filesystem initialization
- Inter-process communication (IPC) primitives
- Security, token issuance, and audit trail
- Gateway integration and core telemetry plumbing
- Workspace indexing (AST, config, ownership graphs)
- World model reconciliation and host control coordination

## Layout

At a high level:

```
core/
├── Cargo.toml              # noa_core crate manifest
├── README.md               # This document
├── config/                 # Kernel profiles and manifests
│   ├── default_manifest.yaml
│   └── profiles/
├── kernel/                 # Kernel runtime assets (manifest, security tooling)
├── proptest-regressions/   # Property test regression cases
├── src/
│   ├── bin/                # Kernel/host_control/world binaries
│   ├── capabilities/       # Kernel capability registry and KernelHandle
│   ├── config/             # Manifest/profile parsing
│   ├── hardware/           # Hardware profile and detection
│   ├── indexer/            # AST/config/ownership index generation
│   ├── legacy/             # Wrapped legacy components kept for compatibility
│   ├── scorekeeper/        # Trust/reward computation engine
│   ├── world/              # World model reconciliation helpers
│   ├── fs.rs               # Filesystem bootstrap
│   ├── gateway.rs          # Core gateway integration shim
│   ├── host_control.rs     # Host control API surface
│   ├── ipc.rs              # IPC primitives
│   ├── kernel.rs           # Kernel control loop + capability bootstrap
│   ├── memory.rs           # Memory subsystem wiring
│   ├── metrics.rs          # Aggregated telemetry
│   ├── process.rs          # Process/thread management facade
│   ├── runtime.rs          # Adaptive runtime controller and policy plumbing
│   ├── security.rs         # Security operations, tokens, and audit trail
│   ├── symbols.rs          # Stable symbol IDs for indexing
│   ├── time.rs             # Time utilities
│   ├── token.rs            # Capability tokens
│   ├── utils.rs            # Core utilities
│   └── lib.rs              # Crate entry point and init() wiring
└── tests/                  # Integration and regression tests
```

The public API of `noa_core` is anchored in `src/lib.rs`, which exposes:

- `init()` – initializes the kernel, core subsystems, and workspace indexer,
  returning a `KernelHandle`.
- Module namespaces for capabilities, kernel, runtime, gateway, memory, process,
  security, scorekeeper, world, and more.

## Building and Running

From the workspace root (after activating the portable toolchains so the
workspace-local Cargo and Node distributions are on `PATH`):

- Build the core crate for quick iteration (or run `make kernel` for the
  same cargo invocation):

  ```bash
  source ./server/tools/activate-cargo.sh
  cargo build -p noa_core
  ```

- Run the core test suite on the same activated shell:

  ```bash
  source ./server/tools/activate-cargo.sh
  cargo test -p noa_core
  ```

- Produce the hardened kernel bundle (release `noa_kernel` +
  `noa_host_control`, default manifest, README, and aggregated test log)
  under `dist/kernel/`:

  ```bash
  source ./server/tools/activate-cargo.sh
  source ./server/tools/activate-node.sh
  make image
  ```

  The image target leaves fresh artifacts in:

  - `dist/kernel/noa_kernel`
  - `dist/kernel/noa_host_control`
  - `dist/kernel/manifest.yaml` (copied from `core/config/default_manifest.yaml`)
  - `dist/kernel/README.md`
  - `dist/kernel/test-results.log` (stdout from `cargo test -p noa_core --tests -- --nocapture`)

  Keep these paths handy when handing the bundle to downstream packaging or
  when attaching evidence to the Truth Gate ledger.

The `kernel.rs` module is also used by the higher-level `noa_kernel` binary
(`core/src/bin/kernel.rs`) to expose an HTTP API for the scorekeeper and
kernel control surface.

### Service descriptors must shadow the manifest

`scripts/ci/check_kernel_manifest.py` enforces a one-to-one mapping between
`core/kernel/manifest/kernel_graph.json` and lightweight service descriptors
stored at `services/<service-id>/service.json`. Whenever you add or rename a
service inside the manifest, immediately drop a matching descriptor (at minimum
`id`, `version`, and `interfaces`) so the manifest check and
`tests/python/test_kernel_manifest.py` stay green. This also keeps gateway tests
and the Truth Gate evidence chain aligned because every service referenced in
`tests/python/test_kernel_manifest.py` is verified against its descriptor.

### Workspace indexer scope

The workspace indexer omits generated artifacts and vendored tool caches so
malformed upstream fixtures never block kernel boots. In addition to skipping
`archive/`, `build_output/`, `repos/`, and `.workspace/indexes/`, it now ignores
`server/tools/cargo-portable/registry/**`, which mirrors crates.io (including
`tests/ui` samples that intentionally fail to parse). This keeps the AST/config
graphs focused on first-party code while preserving Cargo’s offline cache.

## AI Control Loop Interfaces

The kernel exposes a machine-first control surface for orchestrators and
runtime schedulers:

- `kernel::AiControlLoop` – implemented by `KernelHandle`, emitting an
  `AgentHealthSnapshot` that blends aggregated kernel metrics with recent
  security operations.
- `kernel::MachineRemediationDirective` – policy bundle describing whether
  remediations should be executed by machines, including a confidence score,
  rationale, load level, telemetry echo, and outstanding security follow-ups.
- `runtime::AdaptiveRuntimeController` – consumes the kernel directive and
  hardware profile to compute a `RuntimePlan` for backends and a
  `MachineExecutionPolicy` that downstream schedulers use to prioritise
  machine-led recovery steps.

Downstream agents obtain the directive through `kernel::handle()` and avoid
human-in-the-loop fallbacks whenever the directive marks `prefer_machine = true`.
