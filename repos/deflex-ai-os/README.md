# FlexNetOS Star Mono

A unified monorepo combining the best open-source AI/ML tools and frameworks.

## 🏗️ Architecture

```
star-mono/
├── apps/          # User-facing applications
├── services/      # Microservices
├── packages/      # Shared libraries
├── trainer/       # ML training pipelines
├── vendor/        # Git subtree imports
├── external/      # Git submodule imports
└── notebooks/     # Jupyter notebooks
```

## 🚀 Quick Start

```bash
# Clone the repository
git clone --recursive https://github.com/FlexNetOS/star-mono.git
cd star-mono

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build all services
cargo build --workspace

# Run the API gateway
cargo run --bin api-gateway
```

## 🔧 Services

- **API Gateway** - Central API orchestration (Axum)
- **Inference** - ML model serving (Candle/llama.cpp)
- **Agents** - Autonomous agent coordination (Goose/Letta)
- **Retrieval** - RAG and vector search (Qdrant/Chroma)
- **Data** - Data processing pipelines (DataFusion/DuckDB)

## 📦 Included Projects

### Core Infrastructure
- Pingora - Cloudflare's proxy framework
- Axum - Web framework
- Tokio - Async runtime

### ML/AI
- Burn - Rust ML framework
- Candle - Rust tensor operations
- llama.cpp - LLM inference

### Agents
- Letta - Memory-enhanced agents
- OpenHands - Code generation
- Browser-use - Web automation

## 🔄 Repository Management

Repositories are managed via:
- **Git Subtrees** - For vendored dependencies we modify
- **Git Submodules** - For large external projects

Update all dependencies:
```bash
# Update subtrees
./scripts/sync-subtrees.sh

# Update submodules
git submodule update --remote --recursive
```

## 📊 Observability

Built-in observability with:
- OpenTelemetry tracing
- Structured logging
- Prometheus metrics (coming soon)

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test --workspace`
5. Submit a pull request

## 📄 License

This project is dual-licensed under MIT and Apache-2.0.
Individual vendored/external projects retain their original licenses.