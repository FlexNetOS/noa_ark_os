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

### 1. Gateway (`api/`)
- HTTP/1.1 REST API
- HTTP/2 gRPC (tonic)
- WebSocket/SSE streaming
- Single port via ALPN
- Health and metrics endpoints

### 2. Core Orchestration (`core/`)
- Task scheduling
- Workflow execution
- Parallel processing
- Backpressure management

### 3. Inference Runtime (`inference/`)
- Candle ML framework
- SafeTensors model format
- CPU and CUDA support
- Batch processing

### 4. Retrieval Engine (`retrieval/`)
- Vector embeddings (fastembed-rs)
- Qdrant vector database
- Semantic search
- RAG capabilities

### 5. Plugin Runtime (`plugins/`)
- WASM (wasmtime)
- Capability-based security
- Sandboxed execution
- Native plugins (feature-gated)

### 6. Observability (`observability/`)
- Structured logging (tracing)
- Prometheus metrics
- OpenTelemetry traces
- Health checks

### 7. CLI (`cli/`)
- Server management
- Configuration
- Migrations
- Admin tools

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

# Build specific component
cargo build -p noa-server-api
```

### Run

```bash
# Run server (development)
cargo run --bin noa-unified-server -- --http-addr 0.0.0.0:8787 --grpc-addr 0.0.0.0:50051

# Run with custom config
cargo run --bin noa-unified-server -- --config config/dev.toml --grpc-addr 0.0.0.0:50051

# Run release binary
./target/release/noa-unified-server --http-addr 0.0.0.0:8787 --grpc-addr 0.0.0.0:50051
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

### Operational readiness validation

The unified server ships with a Docker Compose + load test harness located in `server/tests/`.
It provisions PostgreSQL, Redis, and the UI API binary, waits for `/healthz` and `/readyz`,
executes HTTP/gRPC smoke calls, records metrics, and enforces the `p95 < 100ms` and
`error rate < 0.1%` thresholds before running the bundled k6 script.

```bash
# Run the stack validation once (requires Docker + python3)
python3 server/tests/run_suite.py \
  --compose-file server/tests/docker-compose.test.yml \
  --metrics-output server/tests/.last-run.json \
  --k6-script server/tests/k6/ui_workflow.js

# Full CI-friendly target (installs Python deps, runs the suite, Criterion bench + k6)
make server.test-all
```

`server/tests/run_suite.py` prints the sampled metrics as JSON so operators can capture a
pre-activation evidence trail. A recent run looked like:

```json
{
  "timestamp": "2024-06-01T12:00:00Z",
  "http": {
    "name": "http",
    "samples": 20,
    "duration_sec": 0.62,
    "p95_ms": 7.41,
    "throughput_rps": 31.6,
    "error_rate": 0.0
  },
  "grpc": {
    "name": "grpc",
    "samples": 20,
    "duration_sec": 0.71,
    "p95_ms": 8.02,
    "throughput_rps": 28.1,
    "error_rate": 0.0
  }
}
```

Use the generated `server/tests/.last-run.json` file when handing over an environment; a
passed run indicates the REST/gRPC surfaces are stable, the Compose stack is healthy, the
Criterion benchmark completed, and the k6 script met the latency/error thresholds.

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
GET  /health          - Legacy liveness probe (always 200 if alive)
GET  /healthz         - HTTP liveness probe for the UI API stack
GET  /ready           - Legacy readiness probe (checks dependencies)
GET  /readyz          - Aggregated readiness (drop root + streaming session)
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
