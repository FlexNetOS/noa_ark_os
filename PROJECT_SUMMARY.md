# NOA-AgenticOS: Unified Multi-Agent Baseline

## 🎯 Project Overview

This repository contains a **unified multi-agent baseline** system with both **TypeScript** and **Rust** implementations, designed as a minimal, self-hostable foundation for multi-agent systems.

## 📁 Repository Structure

```
workspaces/
├── unified/                    # TypeScript multi-agent system
│   ├── packages/
│   │   ├── shared/            # Common message types
│   │   ├── shims/             # Platform adapters  
│   │   └── kernel/            # Agent registry & bus
│   ├── agents/
│   │   └── digest/            # Sample digest agent
│   ├── scripts/
│   │   └── boot.ts           # System bootstrap
│   ├── tools/
│   │   └── sbom.ts           # Self-healing SBOM
│   └── johnson/              # Future IDL contracts
└── mono/                      # Rust-first monorepo
    ├── crates/
    │   ├── shared/           # A/B events, telemetry, errors
    │   ├── globals/          # Toggleable hardware bridges
    │   └── obs/              # OpenTelemetry setup
    ├── services/
    │   ├── api/              # Axum JSON API + A/B middleware
    │   ├── inference/        # Candle runtime (CUDA optional)
    │   ├── agent/            # Rig/Goose-ready agent service
    │   └── retrieval/        # Qdrant/Chroma clients
    ├── trainer/burn/         # Burn ML pipelines
    ├── db/migrations/        # Neon/Postgres schema
    └── tools/sbom/           # Rust SBOM tool
```

## 🚀 Quick Start

### TypeScript System (unified/)

```bash
cd unified
pnpm install
pnpm dev           # Runs SBOM check + build + boot
```

### Rust System (mono/)

```bash
cd mono
cargo check --workspace
cargo run -p sbom-tool
cargo run -p api          # API service with A/B testing
cargo run -p inference    # Candle-based inference
```

## ✨ Key Features

### TypeScript Implementation
- **Multi-agent kernel** with message bus and registry
- **Self-healing SBOM** with dependency auto-fix
- **Workspace architecture** with proper TypeScript project references
- **Docker Compose** support for easy deployment
- **Johnson IDL** contracts for future transpilation

### Rust Implementation  
- **Workspace-based** monorepo with shared dependencies
- **A/B testing middleware** with automatic bucketing
- **OpenTelemetry integration** for observability
- **Toggleable globals** (PC/Mobile/AR) via environment
- **Modular services**: API, Inference, Agent, Retrieval, Training
- **Database migrations** for Neon/Postgres
- **CI/CD pipeline** with formatting, linting, and tests

## 🔧 Configuration

### Environment Variables (Rust)
```env
RUST_LOG=info
UNIFIED_ENABLE_GLOBAL_PC=true
UNIFIED_ENABLE_GLOBAL_MOBILE=false
UNIFIED_ENABLE_GLOBAL_AR=false
DATABASE_URL=postgres://user:pass@localhost/db
QDRANT_URL=http://localhost:6333
```

### Environment Variables (TypeScript)
```env
NODE_ENV=development
UNIFIED_LOG_LEVEL=info
UNIFIED_ENABLE_GLOBAL_PC=true
UNIFIED_ENABLE_GLOBAL_MOBILE=false
UNIFIED_ENABLE_GLOBAL_AR=false
```

## 🏗️ Architecture Highlights

### Shared Types (Rust)
- **AbEvent**: A/B testing events with user bucketing
- **TelemetryAttributes**: Structured observability metadata
- **SharedError**: Common error types across services

### Message System (TypeScript)
- **Envelope**: Universal message format with routing
- **AgentDescriptor**: Agent capability declarations
- **KernelAPI**: Pub/sub interface for agent communication

### Services Integration
- **API Service**: Axum-based REST API with A/B middleware
- **Inference Service**: Candle runtime for ML models
- **Agent Service**: Rig-compatible agent execution
- **Retrieval Service**: Vector database integration
- **Training Service**: Burn ML pipeline framework

## 📊 Database Schema

The system includes comprehensive database migrations for:
- **Experiments**: A/B test configuration and lifecycle
- **A/B Events**: User bucketing and route tracking  
- **Agent Runs**: Agent execution logs and results
- **Inference Logs**: Model inference metrics and costs

## 🔄 Development Workflow

1. **Start with TypeScript** for rapid prototyping
2. **Move to Rust services** for production performance
3. **Use SBOM tools** for dependency management
4. **Deploy via Docker Compose** or individual services
5. **Monitor via OpenTelemetry** traces and metrics

## 🎛️ Toggleable Features

Both implementations support runtime feature toggles:
- **Hardware Bridges**: PC, Mobile, AR platform support
- **Model Backends**: CPU vs CUDA acceleration
- **Database Integrations**: Postgres, Qdrant, etc.
- **Agent Capabilities**: Pluggable agent architecture

## 🔮 Future Extensions

- **Johnson Language**: Transpiler for cross-language agent definitions
- **Web Frontend**: Leptos-based admin interface
- **Desktop App**: Tauri + Dioxus native client
- **Python Bridge**: PyO3 integration for ML workflows
- **Advanced ML**: Unsloth training, evcxr Jupyter kernels

---

**Status**: ✅ Baseline complete, both systems operational
**Next**: Extend with domain-specific agents and production deployment