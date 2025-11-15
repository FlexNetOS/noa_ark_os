# Getting Started with NOA ARK OS

## Quick Start

### Prerequisites

For the bootstrap build (first time only):
- Rust toolchain (latest stable)
- Git

### Clone and Build

```bash
# Clone the repository
git clone <repository-url>
cd noa_ark_os

# Build the entire workspace through the portable shim
make cargo-build ARGS="--release"

# Run the kernel via the same shim
make cargo-run ARGS="--bin noa_kernel"
```

> 💡 Need UI assets? Use `make ui-build`, `make ui-dev`, or `make ui-test` to invoke the `pnpm` scripts through discoverable make targets.

## Project Structure

```
noa_ark_os/
├── core/              # OS core (Rust)
├── agents/            # Agent Factory system
├── workflow/          # Unified workflow engine
├── sandbox/           # Multi-branch sandbox system
├── cicd/              # CI/CD pipeline (CD-focused)
├── ui/                # Dynamic UI/UX (multi-platform)
├── server/            # Server infrastructure
├── ai/                # AI engine and llama.cpp
├── runtime/           # Multi-language runtimes
├── apps/              # Applications
├── tools/             # Development tools
├── storage/           # Storage layer
├── build/             # Build system
└── docs/              # Documentation
```

## Development Workflow

### 1. Create a Sandbox

```bash
# Create your development sandbox
sandbox create --name my-feature --type feature
```

### 2. Develop and Test

```bash
# Activate sandbox
sandbox activate my-feature

# Make changes
# Test locally (workspace default)
make cargo-test

# Validate
sandbox validate
```

### 3. Merge to Integration

```bash
# Merge to integration (D)
sandbox merge my-feature --target D --auto
```

### 4. Deploy

```bash
# CI/CD automatically deploys if all checks pass
# Or manually trigger:
cicd deploy --env production --strategy canary
```

## Key Features

### Agent Factory

Create and manage AI agents:

```rust
use noa_agents::AgentFactory;

let factory = AgentFactory::new();
let agent_id = factory.create_agent(
    "worker_1".to_string(),
    AgentType::Worker,
    AgentLanguage::Rust,
    false,
)?;
```

### Unified Workflow

Define and execute workflows:

```yaml
workflow:
  name: "my-workflow"
  stages:
    - name: "process"
      type: "parallel"
      tasks:
        - agent: "python_agent"
          action: "data_processing"
```

### Sandbox System

Work in isolated environments:

```rust
use noa_sandbox::SandboxManager;

let manager = SandboxManager::new();
manager.create_sandbox("feature_a".to_string(), SandboxType::Feature, "main".to_string())?;
manager.validate("feature_a")?;
manager.merge_to_integration(vec!["feature_a".to_string()])?;
```

### CI/CD Pipeline

Continuous delivery focused:

```rust
use noa_cicd::CICDSystem;

let cicd = CICDSystem::new();
let pipeline_id = cicd.trigger_pipeline("build".to_string(), "abc123".to_string())?;
cicd.execute_pipeline(&pipeline_id)?;
```

## Configuration

Configuration files are located in each component directory. See individual README files for component-specific configuration.

## Capability registry and UI gating

The Kanban workspace reads feature capabilities from the repository-wide registry to decide which controls are available offline.

1. **Register a capability.** Add or update entries in `registry/capabilities.json`. Each capability can expose multiple tokens through its `provides` array (for example `kanban.manageColumns` or `kanban.assist`).
2. **Server-side cache.** The module `ui/vibe-kanban/server/capabilities.ts` loads the registry at startup and caches the parsed structure. Extend `ui/vibe-kanban/shared/capabilities.ts` when you introduce new feature gates so the cache can evaluate them consistently.
3. **API exposure.** The Next.js route at `ui/vibe-kanban/app/api/capabilities/route.ts` serves the cached registry with offline-friendly headers. Front-end components consume this endpoint via the `useBoardState` hook, which now returns a `capabilities` bag containing loading state, feature evaluations, and a `has()` helper.
4. **Gate UI controls.** Use the evaluated gates to toggle controls (see `BoardHeader`, `AssistPanel`, and `BoardShell` for examples). Surface the summary in the UI so operators can immediately see why a control is disabled.
5. **Test the change.** Add or update Vitest suites under `ui/vibe-kanban/app/**/__tests__` to cover both the API and the gated UI states (`pnpm --filter vibe-kanban test`).

Following this workflow keeps the capability list authoritative, documents the intended behavior, and guarantees that offline users receive deterministic UI feedback when a feature is unavailable.

## Testing

```bash
# Run all tests
make cargo-test

# Run specific component tests
make cargo-test ARGS="-p noa_core"
make cargo-test ARGS="-p noa_agents"
make cargo-test ARGS="-p noa_workflow"

# Run integration tests
make cargo-test ARGS="--test integration_test"
```

## Building for Production

```bash
# Build optimized release
make cargo-build ARGS="--release --workspace"

# The binaries will be in target/release/
```

## Troubleshooting

### Build Errors

If you encounter build errors, ensure:
1. Rust toolchain is up to date: `rustup update`
2. All dependencies are available: `cargo check`

### Runtime Errors

Check logs in:
- `logs/kernel.log` - Kernel logs
- `logs/agents.log` - Agent system logs
- `logs/cicd.log` - CI/CD logs

## Next Steps

1. Read the [Architecture documentation](docs/ARCHITECTURE.md)
2. Review the [Roadmap](docs/ROADMAP.md)
3. Explore individual component READMEs
4. Join development discussions

## Contributing

This is a self-contained operating system project. See CONTRIBUTING.md for guidelines.

## License

MIT License - See LICENSE file for details
