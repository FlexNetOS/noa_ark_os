# Agent Registry Service

Production-ready agent lifecycle management and discovery service for NOA ARK OS.

**Two Implementations Available**:
- **Go** (Primary, feature-complete) - `go/`
- **Rust** (Secondary, minimal) - `rust/`

---

## 📋 Overview

The Agent Registry Service provides centralized registration, discovery, health monitoring, and lifecycle management for agents running in the NOA ARK OS ecosystem.

### Key Features

✅ **Agent Registration** - Dynamic agent enrollment at runtime  
✅ **Health Monitoring** - CPU, memory, connection tracking  
✅ **Heartbeat System** - Automatic failure detection  
✅ **Discovery** - Find agents by type, capability, or status  
✅ **Constitutional Governance** - Integration with Trifecta Court  
✅ **Persistence** - PostgreSQL + Redis caching  
✅ **Observability** - Prometheus metrics, structured logging  
✅ **REST API** - Full CRUD operations  
✅ **Containerized** - Docker-ready deployment  

---

## 🏗️ Architecture

### Go Implementation (Primary)

```
Agent Registry Service
├── HTTP Server (Gin)           # RESTful API
├── PostgreSQL Database         # Persistent storage
├── Redis Cache                 # Fast lookups
├── Lifecycle Manager           # Agent state machine
├── Trifecta Court Client       # Constitutional validation
├── Prometheus Exporter         # Metrics
└── Graceful Shutdown           # Clean termination
```

**Tech Stack**:
- Framework: Gin (Go web framework)
- Database: PostgreSQL
- Cache: Redis
- Metrics: Prometheus
- Logging: Logrus
- Containerization: Docker

### Rust Implementation (Secondary)

```
Agent Registry Service (Rust)
├── HTTP Server (Warp)          # Async RESTful API
├── In-Memory Storage           # Arc<RwLock<HashMap>>
└── Constitutional Client       # Validation integration
```

**Tech Stack**:
- Framework: Warp (async web framework)
- Runtime: Tokio
- Storage: In-memory (no persistence)
- Serialization: Serde + JSON

---

## 🚀 Quick Start

### Go Implementation (Recommended)

#### Prerequisites
- Go 1.21+
- PostgreSQL 13+
- Redis 6+

#### 1. Set Up Database

```sql
-- Create database
CREATE DATABASE agent_registry;

-- Create agents table
CREATE TABLE agents (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(100) NOT NULL,
    category VARCHAR(100),
    version VARCHAR(50),
    capabilities JSONB,
    endpoints JSONB,
    status VARCHAR(50) DEFAULT 'active',
    metadata JSONB,
    registered_at TIMESTAMP DEFAULT NOW(),
    last_seen TIMESTAMP DEFAULT NOW(),
    health JSONB,
    heartbeat JSONB
);

-- Create indices
CREATE INDEX idx_agents_type ON agents(type);
CREATE INDEX idx_agents_status ON agents(status);
CREATE INDEX idx_agents_last_seen ON agents(last_seen);
```

#### 2. Configure Environment

```bash
# .env file or environment variables
export DATABASE_URL="postgres://user:password@localhost:5432/agent_registry?sslmode=disable"
export REDIS_URL="redis://localhost:6379"
export SERVER_PORT="8080"
export LOG_LEVEL="info"
export TRIFECTA_COURT_URL="http://localhost:9000"
export HEARTBEAT_TIMEOUT_SECONDS="30"
```

#### 3. Install Dependencies

```bash
cd services/agent-registry/go
go mod download
```

#### 4. Run Service

```bash
go run main.go
```

Service will start on `http://localhost:8080`

#### 5. Verify Health

```bash
curl http://localhost:8080/health
# Expected: {"status": "healthy"}
```

---

### Rust Implementation

#### Prerequisites
- Rust 1.70+

#### 1. Install Dependencies

```bash
cd services/agent-registry/rust
cargo build --release
```

#### 2. Run Service

```bash
cargo run --release
```

Service will start on `http://0.0.0.0:3003`

---

## 🐳 Docker Deployment

### Build Image

```bash
cd services/agent-registry
docker build -t noa-agent-registry:latest .
```

### Run Container

```bash
docker run -d \
  --name agent-registry \
  -p 8080:8080 \
  -e DATABASE_URL="postgres://user:password@db:5432/agent_registry" \
  -e REDIS_URL="redis://redis:6379" \
  -e SERVER_PORT="8080" \
  -e LOG_LEVEL="info" \
  noa-agent-registry:latest
```

### Docker Compose (Recommended)

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: agent_registry
      POSTGRES_USER: registry
      POSTGRES_PASSWORD: changeme
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  agent-registry:
    build: .
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: "postgres://registry:changeme@postgres:5432/agent_registry?sslmode=disable"
      REDIS_URL: "redis://redis:6379"
      SERVER_PORT: "8080"
      LOG_LEVEL: "info"
      TRIFECTA_COURT_URL: "http://trifecta-court:9000"
      HEARTBEAT_TIMEOUT_SECONDS: "30"
    depends_on:
      - postgres
      - redis

volumes:
  postgres_data:
  redis_data:
```

Run:
```bash
docker-compose up -d
```

---

## 📡 API Reference

### Endpoints

#### Health Check
```http
GET /health
```

**Response**:
```json
{"status": "healthy"}
```

#### Register Agent
```http
POST /agents/register
Content-Type: application/json
```

**Request**:
```json
{
  "name": "DataProcessor",
  "type": "worker",
  "category": "processing",
  "version": "1.0.0",
  "capabilities": ["data-transform", "validation"],
  "endpoints": {
    "api": "http://localhost:5000",
    "metrics": "http://localhost:5000/metrics"
  },
  "metadata": {
    "team": "data-team",
    "environment": "production"
  },
  "heartbeat": {
    "interval_seconds": 10,
    "enabled": true
  }
}
```

**Response**:
```json
{
  "id": "agent-12345-uuid",
  "name": "DataProcessor",
  "status": "active",
  "registered_at": "2025-10-08T15:30:00Z"
}
```

#### List Agents
```http
GET /agents
GET /agents?type=worker
GET /agents?status=active
```

**Response**:
```json
{
  "agents": [
    {
      "id": "agent-12345-uuid",
      "name": "DataProcessor",
      "type": "worker",
      "status": "active",
      "last_seen": "2025-10-08T15:32:00Z"
    }
  ],
  "total": 1
}
```

#### Get Agent Details
```http
GET /agents/:id
```

**Response**:
```json
{
  "id": "agent-12345-uuid",
  "name": "DataProcessor",
  "type": "worker",
  "capabilities": ["data-transform", "validation"],
  "status": "active",
  "health": {
    "status": "healthy",
    "cpu_percent": 15.5,
    "memory_percent": 32.1,
    "uptime_seconds": 3600
  },
  "heartbeat": {
    "interval_seconds": 10,
    "last_beat": "2025-10-08T15:32:00Z",
    "missed_beats": 0
  }
}
```

#### Update Heartbeat
```http
POST /agents/:id/heartbeat
Content-Type: application/json
```

**Request**:
```json
{
  "health": {
    "cpu_percent": 15.5,
    "memory_percent": 32.1,
    "active_connections": 5,
    "error_count": 0
  }
}
```

**Response**:
```json
{
  "id": "agent-12345-uuid",
  "last_beat": "2025-10-08T15:32:10Z",
  "status": "active"
}
```

#### Deregister Agent
```http
DELETE /agents/:id
```

**Response**:
```json
{
  "id": "agent-12345-uuid",
  "status": "deregistered"
}
```

#### Prometheus Metrics
```http
GET /metrics
```

**Response**: Prometheus text format
```
# HELP agent_registrations_total Total number of agent registrations
# TYPE agent_registrations_total counter
agent_registrations_total 42

# HELP agent_health Agent health status (1=healthy, 0=unhealthy)
# TYPE agent_health gauge
agent_health{agent_id="agent-123",name="DataProcessor"} 1
```

---

## 🔧 Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | (required) | PostgreSQL connection string |
| `REDIS_URL` | (required) | Redis connection string |
| `SERVER_PORT` | `8080` | HTTP server port |
| `LOG_LEVEL` | `info` | Log level (debug, info, warn, error) |
| `TRIFECTA_COURT_URL` | `""` | Constitutional governance service URL |
| `HEARTBEAT_TIMEOUT_SECONDS` | `30` | Missed heartbeat threshold |

### Database Schema

See **Quick Start → Set Up Database** for PostgreSQL schema.

---

## 🔗 Integration with NOA ARK OS

### Relationship to Existing Agent System

**Existing Agent System** (`/agents`):
- Static agent definitions (CSV, 302 agents)
- Agent blueprints and types
- In-process execution (Hive, Swarm, Factory)

**Agent Registry Service** (this service):
- Runtime agent tracking
- Live instance registration
- Health monitoring
- Dynamic discovery

### Integration Architecture

```
┌─────────────────────┐
│   Agent CSV Data    │  Static definitions (302 agents)
│   (/agents/data/)   │
└──────────┬──────────┘
           │ Bootstrap
           ▼
┌─────────────────────┐
│  Agent Registry     │  Runtime tracking
│  Service            │  (this service)
│  (/services/)       │
└──────────┬──────────┘
           │ Register/Heartbeat
           ▼
┌─────────────────────┐
│  Live Agent         │  Running instances
│  Instances          │
│  (/agents/src/)     │
└─────────────────────┘
```

### Creating a Rust Client

To integrate with the existing Rust agent system, create a client in `/agents/src/registry_client.rs`:

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct RegistryClient {
    base_url: String,
    client: Client,
}

impl RegistryClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    pub async fn register_agent(&self, req: RegisterRequest) -> Result<AgentInfo> {
        let url = format!("{}/agents/register", self.base_url);
        let resp = self.client.post(&url).json(&req).send().await?;
        Ok(resp.json().await?)
    }

    pub async fn heartbeat(&self, agent_id: &str, health: HealthUpdate) -> Result<()> {
        let url = format!("{}/agents/{}/heartbeat", self.base_url, agent_id);
        self.client.post(&url).json(&health).send().await?;
        Ok(())
    }
}
```

---

## 📊 Monitoring & Observability

### Prometheus Metrics

Exposed at `/metrics` endpoint:

- `agent_registrations_total` - Total agent registrations
- `agent_health{agent_id,name}` - Agent health status gauge
- `http_request_duration_seconds` - HTTP request latency histogram
- `active_agents{type,status}` - Current active agents by type

### Logging

Structured JSON logging (logrus):

```json
{
  "time": "2025-10-08T15:30:00Z",
  "level": "info",
  "msg": "Agent registered successfully",
  "agent_id": "agent-12345-uuid",
  "agent_name": "DataProcessor",
  "agent_type": "worker"
}
```

### Health Checks

- `/health` - Service health (returns 200 OK)
- Database connectivity check
- Redis connectivity check

---

## 🛡️ Constitutional Governance

The registry integrates with the **Trifecta Court** constitutional governance system to validate agent actions.

### Validation Flow

```
Agent Action Request
        ↓
Agent Registry
        ↓
Trifecta Court Validation
        ↓
Allow / Deny based on:
- Scripture (spiritual law)
- Geometry (mathematical law)
- Law (system law)
```

### Configuration

Set `TRIFECTA_COURT_URL` to enable constitutional validation:

```bash
export TRIFECTA_COURT_URL="http://trifecta-court:9000"
```

---

## 🔮 Future Enhancements

- [ ] GraphQL API in addition to REST
- [ ] Agent capability matching/discovery
- [ ] Agent orchestration (start/stop/restart)
- [ ] Multi-region registry federation
- [ ] Event-driven agent notifications
- [ ] Advanced query DSL
- [ ] Web UI dashboard
- [ ] Agent dependency graphs

---

## 📦 Source

Migrated from `/home/deflex/workspace/agents/agent-registry/` (WSL) on October 8, 2025.

See `crc/drop-in/incoming/repos/agent-registry/MANIFEST.md` for complete analysis.

---

## 📄 License

Part of NOA ARK OS. See workspace root for license details.
