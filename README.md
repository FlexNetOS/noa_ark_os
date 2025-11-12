# NOA ARK OS

**Local first. Self hosted. Full-Stack. Mono-Repo. End-to-End Auto. AgenticAI. Hive Mind. Agent Swarm. Disposable MicroAgents**

A self-contained operating system platform that unifies applications, AI models, and services.

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
# Build the workspace
cargo build --release

# Run CRC/CI/CD demo
cargo run --example crc_cicd_demo

# Run full system demo
cargo run --example full_system_demo

# Run the kernel
cargo run --bin noa_kernel

# Run tests
cargo test --workspace
```

See [Getting Started Guide](docs/GETTING_STARTED.md) for detailed instructions.

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
