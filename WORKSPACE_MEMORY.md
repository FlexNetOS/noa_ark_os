# NOA ARK OS - Workspace Memory Document

**Generated**: 2024-10-08  
**Purpose**: Complete workspace knowledge base for AI assistant memory

---

## 🎯 Project Overview

**NOA ARK OS** is a self-contained operating system platform that unifies applications, AI models, and services with:
- Zero external dependencies
- Multi-language support (Rust, Python, Go, C#)
- Full automation from code drop to production
- Multi-platform UI/UX (Server, Mobile, Desktop, Web, AR, XR)
- AI-supervised code adaptation (CRC)
- Maximum continuous delivery focus

**Core Philosophy**: "Every commit is potentially deployable"

---

### Task 1 – Documentation staging cleanup (2025-11-19)
- Keep only the curated CLI/doc files staged for merge: `docs/guides/dev-environment-cli.md`, `docs/plans/roadmap.md`, `docs/plans/roadmaps/cli_tool_roadmap.{md,todo.xml}`, `docs/roadmap/cli_config_migration_checklist.md`.
- Treat `docs/api/search.index/**` as generated Docusaurus artifacts; they remain locally generated but are not staged/committed (they already sit behind `.gitignore`).
- Treat `.venv-notebooks/` as disposable notebook tooling; added explicit ignore to keep the virtualenv strictly local.
- Treat security scan exports under `tools/security/shim/.workspace/indexes/**` as local cache; ignore and regenerate on demand instead of committing.
- Next steps after this cleanup: finish notebook conflict resolution, then unblock Go/Cargo builds for launch.

## 📂 Workspace Structure

### Location
```
D:\dev\workspaces\noa_ark_os\
```

### Primary Components

1. **`/core`** - OS kernel and core services (Rust)
2. **`/crc`** - Continuous ReCode system (AI-supervised adaptation)
3. **`/agents`** - Agent Factory (hive mind, swarm deployment)
4. **`/workflow`** - Unified workflow orchestration
5. **`/sandbox`** - Multi-branch sandbox (A,B,C→D)
6. **`/cicd`** - CI/CD pipeline (maximum CD focus)
7. **`/server`** - Unified application server
8. **`/ai`** - AI models and inference (Llama.cpp)
9. **`/runtime`** - Multi-language runtimes
10. **`/ui`** - Dynamic multi-platform UI/UX
11. **`/storage`** - Storage layer (VFS, database)
12. **`/apps`** - Built-in applications
13. **`/tools`** - Development utilities
14. **`/build`** - Build system

---

## 🔧 Development Environment

### Multi-Platform Support

**Three Platforms Configured:**

1. **Windows (PowerShell)** - DEFAULT
   - Portable Cargo: `server/tools/cargo-portable/`
   - Activation: `.\server\tools\activate-cargo.ps1`
   - Rust version: 1.90.0
   - Self-contained, workspace-local

2. **WSL (Ubuntu)** - OPTIONAL
   - Native Rust: `~/.cargo/`
   - Activation: `source ./server/tools/activate-cargo.sh`
   - Can also use Windows portable (slower)

3. **Linux/Ubuntu** - OPTIONAL
   - System Rust installation
   - Standard cargo/rustc

### VS Code Configuration

**Settings Location**: `.vscode/settings.json`

**Key Settings**:
- Default terminal: PowerShell (Windows)
- Portable Cargo paths configured
- Rust-analyzer: Points to portable installation
- Multi-platform terminal profiles
- Environment variables auto-configured

**Available Terminal Profiles**:
- PowerShell (default)
- WSL (Ubuntu)
- Command Prompt
- Git Bash

**Tasks Configured**: `.vscode/tasks.json`
- Cargo Build (Portable)
- Cargo Run (Portable)
- Cargo Test (Portable)
- Cargo Check (Portable)
- Activate Portable Cargo

---

## 🚀 Quick Commands

### Build & Run

```powershell
# Activate Cargo (Windows)
.\server\tools\activate-cargo.ps1

# Build workspace
cargo build --workspace --release

# Run tests
cargo test --workspace

# Run full system demo
cargo run --example full_system_demo

# Run CRC/CI/CD demo
cargo run --example crc_cicd_demo

# Run kernel
cargo run --bin noa_kernel
```

### Development Workflow

```powershell
# 1. Open PowerShell terminal in VS Code
# 2. Activate portable Cargo
.\server\tools\activate-cargo.ps1

# 3. Navigate to component
cd crc

# 4. Build
cargo build

# 5. Test
cargo test

# 6. Run
cargo run
```

---

## 🧠 Memory Indexes & Retrieval Performance

- **Hierarchical Stores:** Long-term context is persisted to `.workspace/memory/long_term.cbor` while active sessions stream to `session_<id>.cbor` using compact CBOR encoding.
- **Indexer Output:** Every core bootstrap now regenerates `ast_graph.json`, `ownership_graph.json`, and `config_graph.json` under `.workspace/indexes/`, keeping dependency views fresh for planners.
- **Retrieval SLA:** Incremental fetch APIs guarantee <100 ms p95 retrieval across combined session + long-term windows (validated in automated tests).
- **Scorekeeper Hooks:** Workflow instrumentation records context byte usage and retrieval latency, penalising workflows that exceed the 16 KiB default budget. Penalty and p95 stats surface in goal metric snapshots for downstream automation.

---

## 🏗️ Architecture Components

### 1. CRC - Continuous ReCode

**Purpose**: AI-supervised code adaptation system

**Flow**:
```
External Code → Drop-in → AI Analysis → Auto-Adapt → 
  Archive Original → CI Validation → Deploy
```

**Features**:
- Drop-in folder for external code
- AI analyzes and adapts to workspace conventions
- Auto-approve (95% confidence threshold)
- Compress and archive originals
- Zero stale code in workspace
- Full automation

**Locations**:
- Config: `crc/config/`
- Drop-in: `crc/drop-in/`
- Sandboxes: `crc/sandboxes/`
- Archives: `crc/archive/`

### 2. Agent Factory

**Purpose**: Multi-layered AI agents with collective intelligence

**Agent Types**:
- **Master Agent**: Orchestration
- **Worker Agents**: Task execution
- **Sub-Agents**: Single-purpose, disposable
- **Swarm Agents**: Coordinated groups

**Features**:
- Hive mind (shared knowledge)
- Swarm deployment (parallel execution)
- Multi-language (Python, Rust, Go)
- Auto-cleanup
- Full automation mode

### 3. Unified Workflow

**Purpose**: Single workflow system orchestrates everything

**Stage Types**:
- Sequential
- Parallel
- Conditional
- Loop

**Features**:
- Stage dependencies
- Cross-language coordination
- Real-time monitoring
- Error handling with retry
- Full automation mode

### 4. Sandbox System (A, B, C → D)

**Purpose**: Isolated development with merge-to-integration

**Sandboxes**:
- **A**: Feature development
- **B**: Bug fixes
- **C**: Experimental features
- **D**: Integration (ready to deploy)

**Definition of "Ready"** (for D):
- ✓ All tests passing
- ✓ Code coverage > 80%
- ✓ Security scan passed
- ✓ Performance benchmarks met
- ✓ Code review approved (2+ reviewers)
- ✓ Documentation updated
- ✓ Backward compatible

### 5. CI/CD Pipeline

**Philosophy**: Maximum CD focus, every commit potentially deployable

**Speed Targets**:
- CI (Validate + Build + Test): < 15 minutes
- CD (Deploy to Production): < 5 minutes
- Total (Commit to Production): < 20 minutes

**Deployment Strategies**:
1. Blue-Green (zero-downtime)
2. Canary (5% → 10% → 25% → 50% → 100%)
3. Rolling Update
4. Feature Flags

**Auto-Rollback**:
- Error rate > 5% → Rollback < 30 seconds
- Response time 2x → Rollback < 30 seconds
- Health checks fail → Rollback < 30 seconds

### 6. Server Infrastructure

**Purpose**: Unified application server

**Components** (planned):
- Unified application server
- MCP (Model Context Protocol) server
- Service orchestration
- API gateway
- Load balancing

**Build Spec**: `server/BUILD_SPEC.md`

**Tech Stack**:
- Rust (axum, tonic, tokio)
- PostgreSQL (primary DB)
- Redis (cache)
- Qdrant (vector DB)
- Caddy (reverse proxy) - planned

**External Integrations** (client libraries only):
- GitHub API (octocrab)
- Docker (bollard)
- Kubernetes (kube)
- AWS SDK
- Azure SDK
- Cloudflare
- Llama.cpp (planned)

### 7. Dynamic UI/UX

**Purpose**: Adaptive interfaces for all platforms

**Platforms**:
- 🖥️ Server (REST, GraphQL, WebSocket, gRPC)
- 📱 Mobile (iOS, Android, PWA)
- 💻 Desktop (Windows, Mac, Linux, Tauri)
- 🌐 Web (modern browsers)
- 👓 AR Glasses (overlay, gesture, voice)
- 🥽 XR Headset (VR, MR, spatial)

---

## 📚 Documentation Map

### Core Documentation
- `README.md` - Main overview
- `OVERVIEW.md` - System overview
- `HIERARCHY.md` - Complete file structure
- `SETUP_COMPLETE.md` - Setup summary

### Server Documentation
- `server/README.md` - Server overview
- `server/BUILD_SPEC.md` - Complete build specification
- `server/TOOLS_AUDIT.md` - Portable tools audit

### Tool Documentation
- `server/tools/README.md` - Tools overview
- `server/tools/MULTI_PLATFORM.md` - Multi-platform guide
- `server/tools/QUICK_START.md` - Quick reference
- `server/tools/SETTINGS_SUMMARY.md` - Configuration summary

### Component Documentation
- `crc/README.md` - CRC system
- `crc/SANDBOX_MODELS.md` - Sandbox guide
- `agents/README.md` - Agent factory
- `workflow/README.md` - Workflow system
- `cicd/README.md` - CI/CD pipeline
- `cicd/CRC_CI_CD.md` - CRC/CI/CD integration

### Architecture Documentation
- `docs/ARCHITECTURE.md` - Architecture details
- `docs/GETTING_STARTED.md` - Getting started
- `docs/INTEGRATION.md` - Component integration
- `docs/ROADMAP.md` - Development roadmap

---

## ⚙️ Configuration Files

### Root Level
- `Cargo.toml` - Workspace manifest
- `.gitignore` - Git exclusions
- `NoaArkOS.sln` - Solution file

### VS Code
- `.vscode/settings.json` - Multi-platform settings
- `.vscode/tasks.json` - Build tasks

### Server
- `server/config/default.toml` - Default config
- `server/config/dev.toml` - Development
- `server/config/staging.toml` - Staging
- `server/config/prod.toml` - Production

### CRC
- `crc/config/rules.yaml` - Adaptation rules
- `crc/config/patterns.yaml` - Code patterns
- `crc/config/standards.yaml` - Code standards
- `crc/config/sandboxes.yaml` - Sandbox config

### CI/CD
- `cicd/config/pipeline.yaml` - Pipeline config
- `cicd/config/deployment.yaml` - Deployment config

---

## 🔒 Git Configuration

### Excluded from Git (gitignored)

**Build Artifacts**:
- `target/`
- `bin/`
- `obj/`
- `dist/`

**Portable Tools**:
- `server/tools/cargo-portable/`
- `server/tools/rustup-portable/`

**Runtime Data**:
- `server/data/`
- `crc/temp/`
- `crc/archive/`
- `crc/sandboxes/*/active/`
- `crc/drop-in/incoming/`

**Models & Large Files**:
- `*.bin`
- `*.gguf`
- `*.safetensors`

**Secrets**:
- `.env*` (except `.env.example`)
- `*.key`
- `*.pem`
- `*.crt`

### Included in Git

**Documentation**: All `README.md` and `*.md` files
**Scripts**: All setup/activation scripts
**Configuration**: All `*.yaml`, `*.toml` files
**Source Code**: All `*.rs`, `*.py`, `*.go` files
**Structure**: `.gitkeep` files maintain empty directories

---

## 🎯 Development Status

### ✅ Completed (Phase 1)
- Core OS skeleton (Rust)
- Agent Factory with hive mind and swarm
- Unified Workflow engine
- Sandbox system (A,B,C→D)
- CI/CD pipeline framework
- Dynamic UI/UX design
- CRC - Continuous ReCode
- AI auto-approval system
- Drop-in folder architecture
- Archive and compression
- Multi-platform development setup
- Portable Cargo configuration

### 🔨 In Progress (Phase 2)
- Runtime environment integration
- Python runtime
- Go runtime
- .NET runtime

### 📋 Planned (Future Phases)
- AI engine implementation (Phase 3)
- Server infrastructure (Phase 4)
- Storage layer (Phase 5)
- Build system (Phase 6)
- Application framework
- Full self-hosting capability
- Production deployment

---

## 🚨 Important Notes

### Portable Cargo Installation

**CRITICAL**: The workspace uses a **Windows portable Cargo installation**.

**Must Use**:
- PowerShell (NOT WSL/bash)
- Activation script before each session
- Windows paths (backslashes)

**Activation Required**:
```powershell
.\server\tools\activate-cargo.ps1
```

**Verification**:
```powershell
cargo --version
# Should show: cargo 1.90.0 (...)
```

### Common Issues

1. **"cargo: command not found"**
   - Cause: Forgot to activate or using WSL
   - Fix: Run `.\server\tools\activate-cargo.ps1` in PowerShell

2. **Rust-analyzer errors**
   - Cause: Wrong paths in settings
   - Fix: Check `.vscode/settings.json`, reload window

3. **WSL vs PowerShell confusion**
   - Check prompt: `PS D:\...` = PowerShell, `deflex@...` = WSL
   - Use PowerShell for portable Cargo

### Best Practices

1. **Always activate Cargo** at start of session
2. **Use PowerShell** for development
3. **Run tasks via VS Code** (Ctrl+Shift+P → Tasks)
4. **Keep documentation updated** when making changes
5. **Follow sandbox workflow** for features/bugs
6. **Let CI/CD handle deployment** (automation)

---

## 📊 Key Metrics

### Performance Targets
- Build time: < 5 minutes (full workspace)
- Test time: < 10 minutes (all tests)
- Deploy time: < 5 minutes (to production)
- Response time: < 100ms (p95)
- Throughput: > 10,000 req/s

### Quality Targets
- Test coverage: > 80%
- Change failure rate: < 5%
- Mean time to recovery: < 5 minutes
- Uptime: 99.9%

### Automation
- Manual steps: 0 (fully automated)
- Deployment frequency: Multiple per day
- Lead time: < 1 hour (commit to production)

---

## 🔗 Quick Links

### Local Paths
- Workspace: `D:\dev\workspaces\noa_ark_os\`
- Portable Cargo: `D:\dev\workspaces\noa_ark_os\server\tools\cargo-portable\`
- VS Code Settings: `D:\dev\workspaces\noa_ark_os\.vscode\`

### Documentation Shortcuts
- Main README: `/README.md`
- Server Build Spec: `/server/BUILD_SPEC.md`
- Multi-Platform Guide: `/server/tools/MULTI_PLATFORM.md`
- Architecture: `/docs/ARCHITECTURE.md`

### Commands Reference
- Activate Cargo: `.\server\tools\activate-cargo.ps1`
- Build: `cargo build --workspace`
- Test: `cargo test --workspace`
- Demo: `cargo run --example full_system_demo`

---

## 🎓 Learning Resources

### For New Developers
1. Read `README.md` - Overview
2. Read `docs/GETTING_STARTED.md` - Quick start
3. Read `server/tools/MULTI_PLATFORM.md` - Setup guide
4. Run `cargo run --example full_system_demo` - See it work

### For Contributors
1. Read `docs/ARCHITECTURE.md` - System design
2. Read `docs/INTEGRATION.md` - Component integration
3. Read `crc/SANDBOX_MODELS.md` - Development workflow
4. Read `cicd/CRC_CI_CD.md` - Deployment process

### For DevOps
1. Read `server/BUILD_SPEC.md` - Build specification
2. Read `cicd/README.md` - CI/CD setup
3. Read `server/TOOLS_AUDIT.md` - Tools inventory
4. Read deployment configs in `server/config/`

---

## ✅ Memory Checklist

This workspace is:
- ✅ Multi-platform (Windows, WSL, Linux)
- ✅ Self-contained (portable Cargo)
- ✅ Fully automated (CRC → CI/CD → Deploy)
- ✅ Well-documented (comprehensive guides)
- ✅ Production-ready architecture
- ✅ Agent-driven (hive mind, swarms)
- ✅ Sandbox-isolated (A,B,C→D)
- ✅ Maximum CD focus (< 20 min to production)

**Current Focus**: Phase 2 - Runtime environment integration

**Primary Platform**: Windows with PowerShell and portable Cargo

**Last Updated**: 2024-10-08

---

**This document contains all essential workspace knowledge for AI assistant memory.**
