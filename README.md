# noa_ark_os - Unified FlexNetOS Mono-Repository

**Local first. Self hosted. Full-Stack. Mono-Repo. End-to-End Auto. AgenticAI. Hive Mind. Agent Swarm. Disposable MicroAgents**

## 🎯 Overview

This is the unified mono-repository consolidating all FlexNetOS projects into a single, cohesive development environment. All previously separate repositories have been merged as subtrees under the `/repos` directory, enabling streamlined development, continuous integration, and unified deployment workflows.

## 📁 Repository Structure

```
noa_ark_os/
├── repos/
│   ├── ark-os-noa/          # AgenticAI with Hive Mind running AI Agent Swarm via MicroAgentStack
│   ├── ARK-OS/              # Core OS architecture, system maps, and task graphs
│   ├── agentaskit/          # Task execution and code migration using multi-agent framework
│   ├── deflexnet-app/       # Python-based digest pipeline application
│   ├── deflex-ai-os/        # Rust-based AgenticAI File Management and Operations
│   └── MicroAgentStack/     # Disposable microagent orchestration and management
├── docs/                    # Unified documentation
├── scripts/                 # Cross-repo automation scripts
└── .github/                 # CI/CD workflows and actions
```

## 🔧 Consolidated Components

### 1. **ark-os-noa**
- **Language**: Python, HTML
- **Purpose**: Local-first Agentic OS with Hive Mind architecture
- **Key Features**: 
  - Agent pack distribution
  - Docker compose orchestration
  - Pipeline execution
  - Documentation and setup guides

### 2. **ARK-OS**
- **Language**: Mermaid, Python, JSON
- **Purpose**: System architecture and autonomous system mapping
- **Key Features**:
  - Autonomous system maps
  - Task execution graphs
  - Universal task execution policy
  - Conversation data and planning artifacts

### 3. **agentaskit**
- **Language**: Rust
- **Purpose**: Multi-agent task execution framework
- **Key Features**:
  - Code migration tools
  - System unification utilities
  - Production-ready agent toolkit
  - Comprehensive documentation

### 4. **deflexnet-app**
- **Language**: Python
- **Purpose**: Digest pipeline processing
- **Key Features**:
  - Pipeline orchestration
  - Test suite
  - Modular digest processing

### 5. **deflex-ai-os**
- **Language**: Rust
- **Purpose**: AgenticAI file management and operations
- **Key Features**:
  - Cargo-based Rust project
  - Service architecture
  - Docker compose setup
  - CI/CD workflows

### 6. **MicroAgentStack**
- **Language**: Python
- **Purpose**: Disposable microagent stack for distributed AI operations
- **Key Features**:
  - Agent manifest management
  - Dynamic agent generation
  - Docker orchestration
  - OpenAPI specification generation
  - Full automation scripts

## 🚀 Getting Started

### Prerequisites
- Git (for repository management)
- Docker & Docker Compose (for containerized services)
- Python 3.8+ (for Python-based components)
- Rust & Cargo (for Rust-based components)
- Node.js (if needed for any frontend components)

### Quick Start

1. **Clone the repository**
   ```bash
   git clone https://github.com/FlexNetOS/noa_ark_os.git
   cd noa_ark_os
   ```

2. **Explore individual components**
   ```bash
   # Navigate to any component
   cd repos/MicroAgentStack
   
   # Follow component-specific README
   cat README.md
   ```

3. **Run unified services**
  ```bash
  # Launch UI API + Vibe Kanban UI with health checks
  ./scripts/dev/full_system_launch.sh
  ```
  If you prefer manual steps instead of the launcher:

  ```bash
  corepack prepare pnpm@8.15.4 --activate
  corepack pnpm install --frozen-lockfile
  corepack pnpm --filter vibe-kanban dev
  ```

  > The workspace enforces pnpm@8.15.4 through `.pnpmfile.cjs`. If you see "Detected agent: unknown", export `npm_config_user_agent="pnpm/8.15.4"` before running the commands above.

## 🔄 Development Workflow

### CRC/CI/CD - Continue ReCode / Continuous Integration / Continuous Development

This unified repository implements a full auto AI operation model:

1. **Continue ReCode (CRC)**: Automated code refinement and optimization
2. **Continuous Integration (CI)**: Automated testing and validation across all components
3. **Continuous Development (CD)**: Automated deployment and system evolution

### Working with Subtrees

Each component under `/repos` is maintained as a git subtree, allowing:

- **Independent updates**: Update individual components from their source
- **Unified commits**: Make cross-component changes in single commits
- **Preserved history**: Maintain full git history from original repos

#### Update a component from upstream:
```bash
git subtree pull --prefix=repos/MicroAgentStack MicroAgentStack main --squash
```

#### Push changes back to component:
```bash
git subtree push --prefix=repos/MicroAgentStack MicroAgentStack main
```

## 🏗️ Architecture

The unified system follows a microservices architecture with:

- **Agent Orchestration**: Centralized through MicroAgentStack
- **Service Mesh**: Docker Compose based service discovery
- **Event-Driven**: Asynchronous agent communication
- **Scalable**: Horizontal scaling through disposable agents
- **Resilient**: Self-healing through agent lifecycle management

## 📚 Documentation

- [Architecture Overview](./docs/ARCHITECTURE.md)
- [Development Guide](./docs/DEVELOPMENT.md)
- [API Reference](./docs/API.md)
- [Deployment Guide](./docs/DEPLOYMENT.md)

## 🤝 Contributing

Each component maintains its own contribution guidelines. See individual component READMEs for details:

- [agentaskit/CONTRIBUTING.md](./repos/agentaskit/CONTRIBUTING.md)
- [MicroAgentStack/SECURITY.md](./repos/MicroAgentStack/SECURITY.md)

## 📄 License

Components maintain their individual licenses. See LICENSE files in respective directories.

## 🔗 Links

- **Organization**: [FlexNetOS on GitHub](https://github.com/FlexNetOS)
- **Individual Repositories** (now consolidated):
  - [ark-os-noa](https://github.com/FlexNetOS/ark-os-noa)
  - [ARK-OS](https://github.com/FlexNetOS/ARK-OS)
  - [agentaskit](https://github.com/FlexNetOS/agentaskit)
  - [deflexnet-app](https://github.com/FlexNetOS/deflexnet-app)
  - [deflex-ai-os](https://github.com/FlexNetOS/deflex-ai-os)
  - [MicroAgentStack](https://github.com/FlexNetOS/MicroAgentStack)

## 🎯 Roadmap

- [x] Consolidate all FlexNetOS repositories into mono-repo
- [ ] Create unified CI/CD pipeline
- [ ] Implement cross-component integration tests
- [ ] Develop unified API gateway
- [ ] Create comprehensive documentation site
- [ ] Implement automated deployment orchestration
- [ ] Build unified monitoring and observability stack

---

**Status**: Active Development | **Version**: 1.0.0-unified | **Last Updated**: 2025-10-08
# NOA ARK OS

**Local first. Self hosted. Full-Stack. Mono-Repo. End-to-End Auto. AgenticAI. Hive Mind. Agent Swarm. Disposable MicroAgents**

A self-contained operating system platform that unifies applications, AI models, and services.

> **Provider Instruction Policy:** All provider-specific guidance defers to [AGENT.md](./AGENT.md), which is the single source of truth for execution rules and policies.

## Architecture Overview

NOA ARK OS is designed as a fully self-contained operating system with:
- **No external dependencies**
- Multi-language support (Rust, Python, Go, C#)
- Unified server infrastructure
- Built-in AI models and MCP server
- Integrated llama.cpp for LLM inference
- Comprehensive tooling ecosystem
- Dynamic multi-platform UI/UX
- Multi-layered AI agent system
- Unified workflow orchestration
- Advanced sandbox system
- **CRC/CI/CD with maximum automation and AI supervision**

## Core Components

### 1. Kernel & Core Services (`/core`)
- System initialization
- Process management
- Resource allocation
- Inter-process communication

### 2. CRC - Continuous ReCode (`/crc`) 🆕
- **AI-supervised code adaptation**
- **Drop-in folder for external code**
- **Auto-approve high-confidence changes**
- **Compress and archive originals**
- **Zero stale code in workspace**
- **Complete automation from drop to deploy**

### 3. Dynamic UI/UX (`/ui`)
- Server API interface
- Mobile (iOS/Android)
- Desktop (Win/Mac/Linux)
- Web applications
- AR Glasses support
- XR Headset integration

### 4. Agent Factory (`/agents`)
- Multi-layered AI agents
- Hive mind coordination
- Swarm deployments
- Disposable sub-agents
- Parallel execution (Python/Rust/Go)
- End-to-end automation

### 5. Unified Workflow (`/workflow`)
- Single workflow system
- Cross-language coordination
- Pipeline orchestration
- Full auto mode
- Real-time monitoring

### 6. Sandbox System (`/sandbox`)
- A, B, C branch isolation
- Merge to D (integration)
- Validation pipelines
- Definition of "ready"
- Conflict resolution

### 7. CI/CD Pipeline (`/cicd`)
- **CRC integration**
- **AI auto-approve**
- Maximum CD focus
- Automated deployments
- Blue-green/Canary strategies
- Auto-rollback
- Zero-downtime deployments

### 8. Server Infrastructure (`/server`)
- Unified application server
- MCP (Model Context Protocol) server
- Service orchestration
- API gateway

### 9. AI Engine (`/ai`)
- Model management
- Llama.cpp integration
- Inference engine
- Model serving

### 10. Runtime Environments (`/runtime`)
- Rust runtime
- Python interpreter (embedded)
- Go runtime
- .NET runtime

### 11. Applications (`/apps`)
- Built-in system applications
- Application framework
- UI layer

### 12. Tools (`/tools`)
- Development tools
- System utilities
- Build system

### 13. Storage (`/storage`)
- File system
- Database engine
- Configuration management

## CRC/CI/CD Pipeline 🔥

### Complete Automation Flow

```
External Code → CRC Drop-In → AI Analysis → Auto-Adapt -->
  → Archive Original → CI Validation → CD Deploy -->
  → Production (Zero Human Touch)
```

### Key Features

#### CRC - Continuous ReCode
- 📂 **Drop-in folder**: Just drop code, system handles rest
- 🤖 **AI supervision**: Understands and adapts code
- ✅ **Auto-approve**: 95% confidence → automatic deployment
- 🗜️ **Auto-archive**: Compress originals, no stale code
- 📊 **Cross-reference**: Fast lookups without extraction
- 🔄 **Full automation**: Zero human intervention

#### Enhanced CI/CD
- 🎯 **CRC integration**: Triggered by adapted code
- 🤖 **AI confidence**: Auto-approve based on confidence
- ⚡ **< 15 min**: Commit to production
- 🔵 **Blue-Green**: Zero-downtime staging
- 🕯️ **Canary**: Gradual production rollout
- 🔄 **Auto-rollback**: < 30 seconds on failure

## Quick Start

```bash
# Build the workspace with the portable-aware shim
make cargo-build ARGS="--release"

# Run CRC/CI/CD demo through the shim
make cargo-run ARGS="--example crc_cicd_demo"

# Run full system demo
make cargo-run ARGS="--example full_system_demo"

# Run the kernel
make cargo-run ARGS="--bin noa_kernel"

# Run tests (workspace-wide)
make cargo-test ARGS="--workspace"

# Build the primary web client
make ui-build
```

All `make cargo-*` targets wrap the portable Cargo activator when available and fall back to the host toolchain otherwise. The `make ui-*` helpers surface the most common `pnpm` flows without requiring VS Code task bindings.

See [Getting Started Guide](docs/GETTING_STARTED.md) for detailed instructions.

## Developer Environment (CLI-first)

- Build and run the development container with the cross-platform scripts in [`scripts/dev-env.sh`](scripts/dev-env.sh) or [`scripts/dev-env.ps1`](scripts/dev-env.ps1).
- Configuration details (base image, language runtimes, editor extensions) live in [`tools/devshell/dev-env.manifest.toml`](tools/devshell/dev-env.manifest.toml).
- Usage guides, Visual Studio replacement workflow, and environment smoke tests are documented in [`docs/guides/dev-environment-cli.md`](docs/guides/dev-environment-cli.md).
## Tool Registry & CLI Extensions

- Discover cross-cutting tooling via `registry/tools.registry.json`, which
  enumerates observability, automation, analysis, collaboration, and plugin
  capabilities together with budgets, parameters, side effects, and CLI
  mappings.
- The `noa` CLI now surfaces structured output for these categories, for
  example:

  ```bash
  noa observability metrics --target kernel --window 120
  noa automation run --plan-id <uuid>
  noa analysis security --scope services --since HEAD~3
  noa collaboration review --workflow-id <id>
  noa plugin describe --surface cli
  ```

- Plugin developers can import `noa-plugin-sdk` (see `plugins/sdk/`) to parse
  the registry and bootstrap integrations.
- REST and gRPC consumers can target `docs/api/noa-tools.openapi.yaml` and
  `server/protos/noa_tools.proto`, which mirror the CLI signatures for remote
  orchestration.
## Local-First Merge Gate

- Activate the portable toolchains each session (`source server/tools/activate-cargo.sh` and `source server/tools/activate-node.sh`).
- Run `make pipeline.local` (or the VS Code task **Pipeline Local (Portable)**) before committing changes; this run is the source of truth.
- The pipeline automatically refreshes `audit/local_pipeline_status.json`, which includes commit SHA, tool versions, and hashes of `build_output.txt`/`test_output.txt`; commit this file with your changes.
- Configure git once with `git config core.hooksPath tools/git-hooks`; the bundled `pre-push` hook blocks pushes until the evidence file matches `HEAD`.
- GitHub workflows now call `tools/ci/require_local_pipeline.py`, so remote CI simply confirms the recorded local run instead of replacing it.

## AI Assist for Kanban

- The Vibe Kanban app now includes an **AI** button on every card that assembles an engineer-ready implementation prompt.
- Configure providers with `AI_PROVIDER` and related environment variables as documented in [`docs/AI_PROVIDER_CONFIG.md`](docs/AI_PROVIDER_CONFIG.md).
- The button posts to `/api/ai/prompt`, logs each request in SQLite, and surfaces copy/preview actions inside the card.
- Regenerate roadmap build kits via `pnpm export:roadmap` (see [`docs/README_BUILD_KITS.md`](docs/README_BUILD_KITS.md)).

## Project Structure

```
noa_ark_os/
├── core/           # OS core and kernel (Rust)
├── crc/            # Continuous ReCode (AI-supervised) 🆕
│   ├── drop-in/    # Drop code here 🆕
│   ├── archive/    # Compressed archives 🆕
│   ├── temp/       # Temporary (no live code) 🆕
│   └── config/     # Adaptation rules 🆕
├── ui/             # Dynamic multi-platform UI/UX
├── agents/         # Agent Factory (hive, swarm, multi-layered)
├── workflow/       # Unified workflow engine
├── sandbox/        # Multi-branch sandbox system (A,B,C→D)
├── cicd/           # CI/CD pipeline (CRC-integrated) 🆕
├── server/         # Server infrastructure
├── ai/             # AI models and inference
├── runtime/        # Language runtimes
├── apps/           # Applications
├── tools/          # Utilities and tools
├── storage/        # Storage layer
├── build/          # Build system
├── docs/           # Documentation
└── examples/       # Integration examples
    ├── full_system_demo.rs
    └── crc_cicd_demo.rs 🆕
```

## Key Features

### 🤖 CRC - Continuous ReCode (NEW!)
- Drop external code into folder
- AI analyzes and adapts automatically
- Fits code to workspace conventions
- Archives originals (compressed)
- No stale live code
- Auto-triggers CI/CD

### 🔄 Enhanced CI/CD
- CRC integration
- AI auto-approval
- Full automation mode
- Multiple deployment strategies
- Health monitoring
- Automatic rollback

### 🤖 Agent Factory
- Create disposable AI agents
- Hive mind collective intelligence
- Swarm-based parallel execution
- Multi-language agents (Python, Rust, Go)
- Fully automated workflows

### 🔄 Unified Workflow
- Single workflow orchestrates everything
- Parallel and sequential stages
- Cross-component integration
- Full automation mode
- Real-time monitoring

### 🧪 Sandbox System
- Isolated development (A, B, C)
- Merge to integration (D)
- Automated validation
- Ready definition enforcement
- Conflict resolution

### 🖥️ Dynamic UI/UX
- Server, Mobile, Desktop, Web
- AR Glasses, XR Headset
- Adaptive interfaces
- Real-time state sync
- Multi-platform support

## Design Principles

1. **Self-Contained**: No external dependencies
2. **Unified**: Single platform for all services
3. **Performant**: Optimized for speed and efficiency
4. **Extensible**: Modular architecture
5. **Secure**: Security built-in from the ground up
6. **Automated**: Full automation support (NEW!)
7. **Parallel**: Multi-language, multi-agent concurrency
8. **AI-Supervised**: AI model supervision for code adaptation (NEW!)

## Documentation

- [Getting Started](docs/GETTING_STARTED.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Component Integration](docs/INTEGRATION.md)
- [Roadmap](docs/ROADMAP.md)
- [CRC/CI/CD Guide](cicd/CRC_CI_CD.md) 🆕
- [CRC System](crc/README.md) 🆕

## Examples

### CRC/CI/CD Automation 🆕
```bash
cargo run --example crc_cicd_demo
```

Demonstrates:
- Dropping external code
- AI analysis and adaptation
- Auto-approval process
- CI/CD automation
- Archive and cleanup
- Zero human intervention

### Full System Integration
```bash
cargo run --example full_system_demo
```

## Developer Tooling Updates

- **Unified Makefile** – Use the new root `Makefile` for common tasks:
  - `make build` runs the workspace `pnpm build`
  - `make test` executes both the UI/Vitest suite and `cargo test -p noa_crc`
  - `make digest` triggers `cargo run -p noa_crc -- ingest` to exercise the CRC pipeline locally
  - `make run` starts the UI (`corepack pnpm --filter vibe-kanban dev`) and UI API (`cargo run -p noa_ui_api`) side-by-side with automatic teardown
  - `make ci:local` aggregates linting, type checking, formatting, and the test matrix used in CI so contributors can reproduce gates offline.
- **Environment bootstrap** – Copy `.env.example` to `.env` (or `.env.local`) to apply safe defaults for `OFFLINE_FIRST`, `AI_PROVIDER`, `LLAMA_CPP_ENDPOINT`, `NOA_UI_DROP_ROOT`, `UPLOAD_TMP`, and UI bridge URLs. These values mirror the defaults expected by both the Node services and the Rust CRC/UI API crates.
- **Structured logging** – UI handlers and Rust services now emit JSON logs with `trace_id`, `component`, and `outcome` fields. TypeScript routes use the shared helper in `@noa-ark/shared-ui/logging`, while Rust components register a `tracing_subscriber` JSON layer through `noa_crc::telemetry`. Update custom scripts to rely on these structured payloads rather than string parsing.

Demonstrates:
- All components working together
- Agent Factory with hive mind
- Unified workflow
- Sandbox system
- CI/CD pipeline
- Complete deployment cycle

## Status

🚧 **Under Active Development** 🚧

### Completed
- ✅ Core OS skeleton (Rust)
- ✅ Agent Factory system (928 agents cataloged)
- ✅ Unified Workflow engine
- ✅ Sandbox system architecture
- ✅ CI/CD pipeline framework
- ✅ Dynamic UI/UX design
- ✅ **CRC - Continuous ReCode** 🆕
- ✅ **AI auto-approval system** 🆕
- ✅ **Drop-in folder architecture** 🆕
- ✅ **Archive and compression** 🆕
- ✅ **Fork processing system** 🆕

### In Progress
- 🔨 Runtime environment integration
- 🔨 AI engine implementation
- 🔨 Server infrastructure
- 🔨 Build system

### Planned
- 📋 Application framework
- 📋 Full self-hosting capability
- 📋 Production deployment

## License

MIT License - See [LICENSE](LICENSE) file for details.

## Contributing

This project aims to be a self-contained operating system. Contributions should maintain the zero-dependency philosophy and support full automation.
