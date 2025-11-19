# Server Infrastructure

NOA ARK OS unified application server - a single, integration-ready server unifying API gateway, orchestration, inference, retrieval, and plugin execution.

## Overview

The NOA Unified Server is a Rust-first monolithic application server designed to provide:
- **One binary** with modular features
- **Consistent API** surface (HTTP+gRPC+WebSocket)
- **First-class observability** with OpenTelemetry
- **Secure-by-default** with TLS, JWT, and capability tokens
- **Deterministic builds** and releases

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    NOA UNIFIED SERVER                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Gateway  │  │   Core   │  │Inference │  │Retrieval │  │
│  │  (API)   │  │Orchestr. │  │ Engine   │  │  (RAG)   │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
│       │             │              │              │         │
│  ┌────┴─────────────┴──────────────┴──────────────┴─────┐  │
│  │         Observability (Tracing/Metrics/Logs)         │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │PostgreSQL│  │  Redis   │  │  Qdrant  │  │  WASM    │  │
│  │   (DB)   │  │ (Cache)  │  │(Vectors) │  │(Plugins) │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Components

The workspace now describes the crates that actually live in `server/` instead
of pointing at placeholder directories. Each crate can be built and tested in
isolation, but they are also wired together through the workspace manifest.

| Crate | Path | Kind | Depends On | Purpose |
| --- | --- | --- | --- | --- |
| `noa_orchestrator` | `server/` | library | `noa_core`, `tracing` | Adaptive scaling policies and orchestration utilities that inspect telemetry and coordinate workloads. |
| `noa_gateway` | `server/gateway` | library + bin | `noa_core`, `noa_agents`, security + auth deps | Programmable multi-protocol entrypoint that exposes HTTP/gRPC/WebSocket surfaces with auth, policy, and rate-limiting. |
| `noa_inference` | `server/ai/inference` | library | async + HTTP tooling | Client for inference backends, model streaming helpers, and test shims for AI integrations. |
| `noa_ui_api` | `server/ui_api` | library | `noa_workflow`, `noa_crc` | Server-driven UI orchestration layer that exposes workflow metadata and streaming UI events. |
| `relocation-server` | `server/relocation` | library + bin | `relocation-daemon`, `hyper` | HTTP control plane for the relocation daemon, used to bootstrap agents across hosts. |
| `noa-unified-server` | `server/bins/noa-unified-server` | binary | `noa_orchestrator`, `noa_gateway` | Thin binary that initialises the orchestrator and gateway so the unified server can be launched via `cargo run --bin noa-unified-server`. |

The `noa-unified-server` binary currently verifies that the orchestrator and
gateway bootstrap paths succeed and emits telemetry about the scaling decision
it calculated. Upcoming features can add the HTTP runtime, plugin loading, and
workflow orchestration on top of this foundation.

## Technology Stack

**Language**: Rust (stable)

**Key Dependencies**:
- `axum` + `hyper` + `tower` - Web framework
- `tonic` - gRPC framework
- `tokio` - Async runtime
- `tracing` + `opentelemetry` - Observability
- `sqlx` - PostgreSQL client
- `redis` - Redis client
- `jsonwebtoken` + `rustls` - Security
- `candle-core` + `safetensors` - ML inference
- `wasmtime` - WASM runtime

## Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install system dependencies
sudo apt install libssl-dev pkg-config postgresql-client redis-tools
```

### Build

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Build the workspace binary that wires the orchestrator + gateway
cargo build --bin noa-unified-server
```

### Run

```bash
# Run server (development)
cargo run --bin noa-unified-server

# Run with custom config
cargo run --bin noa-unified-server -- --config config/dev.toml

# Run release binary
./target/release/noa-unified-server
```

### Test

```bash
# Unit tests
cargo test --all

# Integration tests
cargo test --test integration

# With coverage
cargo tarpaulin --out Html
```

## Configuration

Configuration is loaded from (in order of precedence):
1. CLI flags
2. Environment variables (prefix: `NOA_`)
3. Config files (`config/default.toml`, `config/{profile}.toml`)

**Example** (`config/default.toml`):
```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[database]
url = "postgresql://localhost:5432/noa"
max_connections = 20

[cache]
url = "redis://localhost:6379"

[inference]
device = "auto"
model_path = "/models"

[observability]
log_level = "info"
metrics_port = 9090
```

**Environment Variables**:
```bash
export NOA_SERVER__HOST=0.0.0.0
export NOA_SERVER__PORT=8080
export NOA_DATABASE__URL=postgresql://localhost:5432/noa
export RUST_LOG=info
```

## API Endpoints

### Health & Metrics

```
GET  /health          - Liveness probe (always 200 if alive)
GET  /ready           - Readiness probe (checks dependencies)
GET  /metrics         - Prometheus metrics
```

### REST API (v1)

```
POST /v1/infer        - Run inference
POST /v1/embed        - Generate embeddings
POST /v1/search       - Vector search
POST /v1/workflow     - Execute workflow
```

### gRPC Services

```
InferenceService      - ML inference
RetrievalService      - Vector search
OrchestrationService  - Task scheduling
```

### WebSocket

```
WS   /ws/stream       - Real-time streaming
WS   /ws/events       - Server-sent events
```

## Development

### Code Structure

```
server/
├── core/              # Core orchestration
├── api/               # REST/gRPC gateway
├── inference/         # ML inference engine
├── retrieval/         # Vector search/RAG
├── plugins/           # WASM plugin runtime
├── observability/     # Logging/metrics/traces
├── cli/               # CLI binary
├── config/            # Configuration files
├── migrations/        # Database migrations
└── tests/             # Integration tests
```

### Quality Checks

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all -- -D warnings

# Security audit
cargo deny check

# Generate documentation
cargo doc --no-deps --open
```

### Database Migrations

```bash
# Create migration
sqlx migrate add <migration_name>

# Run migrations
sqlx migrate run --database-url $DATABASE_URL

# Revert migration
sqlx migrate revert --database-url $DATABASE_URL
```

## Deployment

### Docker

```bash
# Build image
docker build -t noa-unified-server:latest .

# Run container
docker run -p 8080:8080 -p 9090:9090 \
  -e RUST_LOG=info \
  -e NOA_DATABASE__URL=postgresql://... \
  noa-unified-server:latest
```

### Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f server

# Stop services
docker-compose down
```

### Kubernetes

```bash
# Install with Helm
helm install noa-server ./helm \
  --set image.tag=latest \
  --set database.url=postgresql://...

# Upgrade
helm upgrade noa-server ./helm

# Uninstall
helm uninstall noa-server
```

## Observability

### Logs

Structured JSON logs to stdout:
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "INFO",
  "target": "noa_server",
  "message": "Request processed",
  "request_id": "abc-123",
  "duration_ms": 45
}
```

### Metrics

Prometheus metrics available at `/metrics`:
```
http_requests_total
http_request_duration_seconds
inference_requests_total
inference_duration_seconds
db_connections_active
cache_hits_total
```

### Traces

OpenTelemetry traces exported to OTLP collector:
- Request tracing
- Database queries
- Cache operations
- Inference calls

### Dashboards

Pre-built Grafana dashboards:
- Request throughput and latency
- Error rates
- Resource usage
- Database performance
- Inference metrics

## Security

### Transport Security
- TLS 1.3 via rustls
- Certificate validation
- Automatic cert rotation

### Authentication
- JWT tokens (RS256)
- Token expiration
- Refresh tokens

### Authorization
- Capability tokens (HMAC HS256)
- Scoped permissions
- Principle of least privilege

### Supply Chain
- SBOM generation (syft)
- Container scanning (trivy)
- Dependency audit (cargo-deny)
- Artifact signing (cosign)

## Performance

### Benchmarks

```bash
# Criterion benchmarks
cargo bench

# Load testing
k6 run tests/load/basic.js
```

**Targets**:
- p50 latency: < 50ms
- p95 latency: < 100ms
- p99 latency: < 200ms
- Throughput: > 10,000 req/s
- Error rate: < 0.1%

### Optimization

- LTO enabled in release
- Single codegen unit
- CPU-specific optimizations
- SIMD where applicable
- Zero-copy where possible

## Troubleshooting

### Server won't start

```bash
# Check logs
RUST_LOG=debug cargo run

# Verify config
cargo run -- --check-config

# Test database connection
psql $DATABASE_URL -c "SELECT 1"
```

### High latency

```bash
# Check metrics
curl http://localhost:9090/metrics

# Enable detailed tracing
RUST_LOG=trace cargo run

# Profile
cargo flamegraph --bin noa-unified-server
```

### Memory issues

```bash
# Check resource usage
docker stats noa-unified-server

# Heap profiling
cargo run --features dhat

# Memory leak detection
valgrind --leak-check=full ./target/debug/noa-unified-server
```

## Contributing

See [BUILD_SPEC.md](BUILD_SPEC.md) for detailed build specifications.

### Development Workflow

1. Create feature branch
2. Make changes
3. Run tests and quality checks
4. Submit PR
5. CI/CD validates
6. Deploy after approval

## Documentation

- [Build Specification](BUILD_SPEC.md) - Complete build spec
- [API Documentation](docs/api.md) - REST/gRPC API docs
- [Deployment Guide](docs/deployment.md) - Deploy instructions
- [Configuration Reference](docs/config.md) - Config options

## License

MIT License - See [LICENSE](../LICENSE) file for details.
