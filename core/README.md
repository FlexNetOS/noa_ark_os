# Core OS Module

The core operating system layer built primarily in Rust for safety and performance.

## Responsibilities

- Process and thread management
- Memory allocation and management
- Inter-process communication (IPC)
- System call interface
- Security and permissions
- Resource scheduling

## Structure

```
core/
├── kernel/         # Kernel components
├── fs/             # File system
├── net/            # Network stack
├── security/       # Security subsystem
└── ipc/            # Inter-process communication
```

## Building

This will be a Rust-based core. Build instructions coming soon.

## AI Control Loop Interfaces

The kernel now exposes a machine-first control surface for orchestrators and
runtime schedulers:

- `kernel::AiControlLoop` – implemented by `KernelHandle`, emitting an
  `AgentHealthSnapshot` that blends aggregated metrics with recent security
  operations.
- `kernel::MachineRemediationDirective` – policy bundle describing whether
  remediations should be executed by machines, including a confidence score,
  rationale, load level, telemetry echo, and outstanding security follow-ups.
- `runtime::RuntimeControlLoop` – implemented by `RuntimeManager` to provide a
  `MachineExecutionPolicy` wrapping the kernel directive alongside active
  runtimes for schedulers to prioritise machine-led recovery steps.

Downstream agents obtain the directive through `kernel::handle()` and avoid
human-in-the-loop fallbacks whenever the directive marks `prefer_machine = true`.
