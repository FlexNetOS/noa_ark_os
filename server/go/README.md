# NOA ARK OS - Go Services

High-performance Go services for the NOA ARK OS agent system.

## 📦 Services

### `registry/` (2 files)
Agent registry implementations in Go.

**Components**:
- `registry.go` - Main agent registry (22.1 KB)
- `registry_complete.go` - Complete registry implementation (15.8 KB)

### `memory/` (4 files)
3-plane memory system for agent coordination.

**Components**:
- `coordinator/` - Coordinator cluster
  - `capability_registry.go` (50.6 KB)
  - `main.go` (66.8 KB)
  - `promotion_controller.go` (68.0 KB)
- `sandbox/` - Sandbox cluster
  - `capability_ingestor.go` (33.3 KB)

### `agents/` (4 files)
Go agent implementations.

**Components**:
- `api/` - API services
  - `api.go` (30.0 KB)
  - `cors.go` (0.7 KB)
- `digest/` - Digest agent (15.4 KB)
- `security/` - Security scanner (15.9 KB)

### `hierarchy/` (4 files)
Agent hierarchy management systems.

**Components**:
- `board/board.go` - Board agents (18.6 KB)
- `capsule/orchestrator.go` - Capsule orchestrator (32.5 KB)
- `microagent/stacks.go` - Microagent stacks (27.9 KB)
- `model_selector/selector.go` - Model selector (20.6 KB)

### `core/` (1 file)
NOA core implementation in Go.

**Components**:
- `noa.go` - NOA core (8.8 KB)

## 🚀 Installation

```bash
cd server/go
go mod download
go build ./...
```

## 🔧 Usage

### Running Services

```bash
# Agent Registry
go run registry/registry.go

# Memory Coordinator
go run memory/coordinator/*.go

# API Server
go run agents/api/*.go
```

### Building Binaries

```bash
# Build all services
go build -o bin/ ./...

# Build specific service
go build -o bin/registry registry/registry.go
```

## 📊 Architecture

```
Go Services
├── registry/        - Agent registry
├── memory/          - 3-plane memory
│   ├── coordinator/ - Coordination
│   └── sandbox/     - Sandbox
├── agents/          - Agent implementations
├── hierarchy/       - Hierarchy management
└── core/           - NOA core
        ↕
    gRPC/HTTP
        ↕
Rust Agent System
```

## 🔗 Integration

Go services integrate with:
- **Rust Agents**: Via gRPC and HTTP APIs
- **Python Services**: Via REST and message queues
- **Database**: Shared data stores
- **Message Queue**: Event-driven communication

## ⚡ Performance

Go services are optimized for:
- High concurrency (goroutines)
- Low latency (compiled, not interpreted)
- Memory efficiency
- Network performance

## 🧪 Testing

```bash
go test ./...
go test -v ./memory/...
```

## 📝 Development

```bash
# Format code
go fmt ./...

# Lint
go vet ./...

# Run with race detector
go run -race ./...
```

## 🛠️ Tools

- Go 1.21+
- Protocol Buffers (for gRPC)
- Docker (for containerization)

## 📚 Documentation

See `../../docs/architecture/` for complete system architecture.

## 🔐 Security

All Go services include:
- TLS/SSL support
- Authentication/authorization
- Rate limiting
- Input validation
