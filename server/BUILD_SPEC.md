# NOA Unified Server - Build Specification

## Build Specification Overview

```json
{
  "build_spec": {
    "plan_checklist": [
      "✅ Align spec to Rust-first monorepo",
      "✅ Define unified server components",
      "✅ Select protocols, storage, observability",
      "✅ Harden security and config strategy",
      "✅ Specify build/packaging pipeline",
      "✅ Document deploy and health endpoints",
      "✅ Add Caddy reverse proxy integration",
      "✅ Add external tool integrations"
    ],
    "overview": {
      "name": "noa-unified-server",
      "purpose": "Single, integration-ready application server unifying API gateway, orchestration, inference, retrieval, and plugin execution for NOA Ark OS.",
      "goals": [
        "One binary with modular features",
        "Consistent API surface (HTTP+gRPC+WS)",
        "First-class observability and health",
        "Secure-by-default (TLS/JWT/capability tokens)",
        "Deterministic, reproducible builds and releases",
        "Caddy reverse proxy for automatic HTTPS",
        "Seamless external tool integration"
      ]
    }
  }
}
```

## Components

### 1. Gateway

**Technology Stack**:
- Rust (stable)
- axum (web framework)
- hyper (HTTP implementation)
- tower (middleware)
- serde_json (serialization)

**Interfaces**:
- HTTP/1.1 REST API
- HTTP/2 gRPC (tonic)
- WebSocket/SSE streaming

**Endpoints**:
```
GET  /health         - Liveness probe
GET  /ready          - Readiness probe
GET  /metrics        - Prometheus metrics
POST /v1/*           - REST API v1
gRPC services        - Same port (ALPN)
```

**Implementation**:
```rust
// server/src/gateway/mod.rs
use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/metrics", get(metrics_handler))
        .nest("/v1", api_v1_router())
        .layer(TraceLayer::new_for_http())
}
```

---

### 2. Core Orchestration

**Technology Stack**:
- tokio (async runtime)
- async-trait (trait support)
- thiserror (error handling)

**Capabilities**:
- Task scheduling
- Workflow execution
- Parallel execution
- Backpressure management

**Implementation**:
```rust
// server/src/orchestration/mod.rs
use tokio::task::JoinSet;
use async_trait::async_trait;

#[async_trait]
pub trait Orchestrator {
    async fn schedule(&self, task: Task) -> Result<TaskId>;
    async fn execute_workflow(&self, workflow: Workflow) -> Result<WorkflowResult>;
    async fn parallel_execute(&self, tasks: Vec<Task>) -> Result<Vec<TaskResult>>;
}
```

---

### 3. Inference Runtime

**Technology Stack**:
- candle (ML framework)
- safetensors (model format)

**Targets**:
- CPU (primary)
- CUDA (optional, if available)

**Endpoints**:
```
POST /v1/infer              - REST inference
gRPC InferenceService       - gRPC inference
```

**Configuration**:
```toml
[inference]
model_id = "model-name"
model_path = "/models/model.safetensors"
device = "auto"  # cpu | cuda | auto
max_batch_size = 32
timeout_ms = 5000
```

**Implementation**:
```rust
// server/src/inference/mod.rs
use candle_core::{Device, Tensor};
use safetensors::SafeTensors;

pub struct InferenceEngine {
    device: Device,
    model: Model,
    config: InferenceConfig,
}

impl InferenceEngine {
    pub async fn infer(&self, input: Tensor) -> Result<Tensor> {
        // Implementation
    }
}
```

---

### 4. Retrieval

**Technology Stack**:
- Qdrant client (vector database)
- fastembed-rs (embeddings)

**Endpoints**:
```
POST /v1/embed              - Generate embeddings
POST /v1/search             - Vector search
```

**Strategy**:
- Alias-based collections for zero-downtime migrations

**Implementation**:
```rust
// server/src/retrieval/mod.rs
use qdrant_client::Qdrant;
use fastembed::TextEmbedding;

pub struct RetrievalEngine {
    qdrant: Qdrant,
    embedding: TextEmbedding,
}

impl RetrievalEngine {
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.embedding.embed(text)
    }
    
    pub async fn search(&self, query: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>> {
        self.qdrant.search(query, limit).await
    }
}
```

---

### 5. Persistence

**Primary Storage**: PostgreSQL via sqlx
```toml
[database]
url = "postgresql://user:pass@localhost:5432/noa"
max_connections = 20
min_connections = 5
acquire_timeout_ms = 5000
idle_timeout_ms = 600000
max_lifetime_ms = 1800000
```

**Cache**: Redis
```toml
[cache]
url = "redis://localhost:6379"
pool_size = 10
timeout_ms = 1000
```

**Migrations**: sqlx migrate
```bash
# Run migrations
sqlx migrate run --database-url $DATABASE_URL
```

**Implementation**:
```rust
// server/src/persistence/mod.rs
use sqlx::{PgPool, Pool, Postgres};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = PgPool::connect(url).await?;
        Ok(Self { pool })
    }
    
    pub async fn execute(&self, query: &str) -> Result<u64> {
        sqlx::query(query).execute(&self.pool).await
    }
}
```

---

### 6. Messaging (Optional)

**Options**:
- NATS (preferred for simplicity)
- Kafka (for high-throughput)

**Use Cases**:
- Audit events
- Async tasks
- Bulk ingestion

**Configuration**:
```toml
[messaging]
enabled = true
backend = "nats"  # nats | kafka
url = "nats://localhost:4222"
```

---

### 7. Observability

**Logging**: tracing + tracing-subscriber (JSON)
```rust
use tracing::{info, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}
```

**Metrics**: Prometheus exporter + OpenTelemetry
```rust
use metrics_exporter_prometheus::PrometheusBuilder;

fn init_metrics() -> Result<()> {
    PrometheusBuilder::new()
        .install()
        .map_err(|e| anyhow!("Failed to install metrics: {}", e))
}
```

**Traces**: opentelemetry-otlp (OTLP/gRPC)
```rust
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;

fn init_tracing() -> Result<()> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317")
        )
        .install_batch(opentelemetry::runtime::Tokio)?;
    
    global::set_tracer_provider(tracer);
    Ok(())
}
```

**Health Endpoints**:
```
GET /health      - Liveness (always returns 200 if process alive)
GET /ready       - Readiness (checks dependencies)
GET /metrics     - Prometheus metrics
```

---

### 8. Security

**Transport**: TLS via rustls
```rust
use axum_server::tls_rustls::RustlsConfig;

let tls_config = RustlsConfig::from_pem_file(
    "certs/cert.pem",
    "certs/key.pem"
).await?;
```

**Authentication**: JWT (jsonwebtoken)
```rust
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

pub struct JwtAuth {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtAuth {
    pub fn verify(&self, token: &str) -> Result<Claims> {
        decode(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
    }
}
```

**Authorization**: Capability tokens (HMAC HS256)
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn verify_capability_token(token: &[u8], secret: &[u8]) -> bool {
    let mut mac = HmacSha256::new_from_slice(secret).unwrap();
    mac.verify_slice(token).is_ok()
}
```

**Supply Chain**:
- SBOM generation with syft
- Signatures with cosign/minisign
- Dependency audit with cargo-deny
- FS integrity with fs-verity (when enabled)

---

### 9. Configuration

**Technology**: config crate + dotenvy

**Sources**:
1. TOML/YAML files
2. Environment variables
3. CLI flags (override)

**Profiles**: dev, staging, prod

**Configuration File**:
```toml
# server/config/default.toml
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
trace_endpoint = "http://localhost:4317"

[security]
tls_enabled = true
jwt_secret = "${JWT_SECRET}"
```

**Implementation**:
```rust
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub inference: InferenceConfig,
}

impl Settings {
    pub fn new(profile: &str) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", profile)).required(false))
            .add_source(Environment::with_prefix("NOA"))
            .build()?;
        
        config.try_deserialize()
    }
}
```

---

### 10. Plugins

**Runtime**:
- WASM (wasmtime, wasi-cap-std)
- Native (feature-gated crates)

**Policy**:
- Capability-scoped access
- Preopened directories (no ambient filesystem)

**Implementation**:
```rust
use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;

pub struct PluginRuntime {
    engine: Engine,
    linker: Linker<WasiCtx>,
}

impl PluginRuntime {
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
        
        Ok(Self { engine, linker })
    }
    
    pub fn load_plugin(&self, wasm_bytes: &[u8]) -> Result<Plugin> {
        let module = Module::new(&self.engine, wasm_bytes)?;
        // Create sandboxed context
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .preopened_dir(Dir::open_ambient_dir("/plugins", ambient_authority())?, "/")?
            .build();
        
        Ok(Plugin { module, wasi })
    }
}
```

---

### 11. Caddy Reverse Proxy Integration

**Purpose**: Automatic HTTPS, reverse proxy, and load balancing

**Technology**:
- Caddy v2 (written in Go)
- Automatic certificate management (Let's Encrypt)
- HTTP/3 support
- Admin API for dynamic configuration

**Configuration** (`server/caddy/Caddyfile`):
```
{
    # Global options
    admin 0.0.0.0:2019
    auto_https on
    email admin@noa-ark-os.com
}

# Main application
noa-ark-os.com {
    # Reverse proxy to NOA server
    reverse_proxy localhost:8080 {
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-For {remote_host}
        header_up X-Forwarded-Proto {scheme}
        
        # Load balancing
        lb_policy round_robin
        lb_try_duration 5s
        lb_try_interval 250ms
        
        # Health checks
        health_uri /health
        health_interval 10s
        health_timeout 5s
    }
    
    # WebSocket support
    @websocket {
        header Connection *Upgrade*
        header Upgrade websocket
    }
    reverse_proxy @websocket localhost:8080
    
    # Enable compression
    encode gzip zstd
    
    # Rate limiting
    rate_limit {
        zone dynamic {
            key {remote_host}
            events 100
            window 1m
        }
    }
    
    # Security headers
    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
        X-Content-Type-Options "nosniff"
        X-Frame-Options "DENY"
        X-XSS-Protection "1; mode=block"
        Content-Security-Policy "default-src 'self'"
        Referrer-Policy "strict-origin-when-cross-origin"
    }
    
    # Logging
    log {
        output file /var/log/caddy/access.log
        format json
    }
}

# Metrics endpoint (internal only)
:9090 {
    reverse_proxy localhost:9090
    
    # IP whitelist for metrics
    @metrics {
        remote_ip 127.0.0.1 10.0.0.0/8
    }
    handle @metrics {
        reverse_proxy localhost:9090
    }
    handle {
        respond "Forbidden" 403
    }
}

# API subdomain
api.noa-ark-os.com {
    reverse_proxy localhost:8080 {
        header_up Host {host}
    }
}
```

**Caddy Integration in Server**:
```rust
// server/src/caddy/mod.rs
use serde_json::json;
use reqwest::Client;

pub struct CaddyManager {
    admin_endpoint: String,
    client: Client,
}

impl CaddyManager {
    pub fn new(admin_endpoint: String) -> Self {
        Self {
            admin_endpoint,
            client: Client::new(),
        }
    }
    
    pub async fn add_route(&self, domain: &str, upstream: &str) -> Result<(), Error> {
        let config = json!({
            "match": [{"host": [domain]}],
            "handle": [{
                "handler": "reverse_proxy",
                "upstreams": [{"dial": upstream}]
            }]
        });
        
        self.client
            .post(format!("{}/config/apps/http/servers/srv0/routes", self.admin_endpoint))
            .json(&config)
            .send()
            .await?;
        
        Ok(())
    }
    
    pub async fn reload(&self) -> Result<(), Error> {
        self.client
            .post(format!("{}/load", self.admin_endpoint))
            .send()
            .await?;
        
        Ok(())
    }
}
```

### 12. External Tool Integrations

#### GitHub Integration

**Purpose**: Code drop source, CI/CD integration

**Configuration**:
```toml
[integrations.github]
enabled = true
api_url = "https://api.github.com"
token = "${GITHUB_TOKEN}"
organizations = ["your-org"]
webhooks = ["push", "pull_request"]
```

**Implementation**:
```rust
// server/src/integrations/github.rs
use octocrab::Octocrab;

pub struct GitHubIntegration {
    client: Octocrab,
}

impl GitHubIntegration {
    pub async fn new(token: String) -> Result<Self, Error> {
        let client = Octocrab::builder()
            .personal_token(token)
            .build()?;
        
        Ok(Self { client })
    }
    
    pub async fn fetch_repo(&self, owner: &str, repo: &str) -> Result<Repository, Error> {
        let repo = self.client
            .repos(owner, repo)
            .get()
            .await?;
        
        Ok(repo)
    }
    
    pub async fn create_webhook(&self, owner: &str, repo: &str, url: &str) -> Result<(), Error> {
        self.client
            .repos(owner, repo)
            .create_hook()
            .url(url)
            .events(vec!["push", "pull_request"])
            .send()
            .await?;
        
        Ok(())
    }
}
```

#### VS Code Extension Integration

**Purpose**: Development environment integration

**Configuration**:
```toml
[integrations.vscode]
enabled = true
extension_port = 9000
language_server = true
```

**Language Server Protocol**:
```rust
// server/src/integrations/vscode.rs
use tower_lsp::{LspService, Server};

pub async fn start_language_server(port: u16) -> Result<(), Error> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let (service, socket) = LspService::new(|client| NoaLanguageServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
    
    Ok(())
}
```

#### Llama.cpp Integration

**Purpose**: Local AI inference

**Configuration**:
```toml
[integrations.llama_cpp]
enabled = true
model_path = "./ai/models"
context_size = 4096
gpu_layers = 35
threads = 8
```

**Implementation**:
```rust
// server/src/integrations/llama_cpp.rs
use llama_cpp_rs::{LlamaModel, LlamaContext, LlamaParams};

pub struct LlamaCppIntegration {
    context: LlamaContext,
}

impl LlamaCppIntegration {
    pub fn new(model_path: &str) -> Result<Self, Error> {
        let params = LlamaParams::default()
            .with_n_ctx(4096)
            .with_n_gpu_layers(35);
        
        let model = LlamaModel::load_from_file(model_path, params)?;
        let context = model.new_context()?;
        
        Ok(Self { context })
    }
    
    pub fn infer(&self, prompt: &str) -> Result<String, Error> {
        let response = self.context.infer(prompt)?;
        Ok(response)
    }
}
```

#### AWS SDK Integration

**Purpose**: Cloud services integration

**Configuration**:
```toml
[integrations.aws]
enabled = true
region = "us-east-1"
services = ["s3", "dynamodb", "secretsmanager"]
```

**Implementation**:
```rust
// server/src/integrations/aws.rs
use aws_sdk_s3 as s3;
use aws_sdk_dynamodb as dynamodb;

pub struct AWSIntegration {
    s3_client: s3::Client,
    dynamodb_client: dynamodb::Client,
}

impl AWSIntegration {
    pub async fn new() -> Result<Self, Error> {
        let config = aws_config::load_from_env().await;
        
        Ok(Self {
            s3_client: s3::Client::new(&config),
            dynamodb_client: dynamodb::Client::new(&config),
        })
    }
}
```

#### Cargo Integration

**Purpose**: Rust package management

**Configuration**:
```toml
[integrations.cargo]
enabled = true
registry = "https://crates.io"
```

#### Dart Integration

**Purpose**: Flutter/Dart support

**Configuration**:
```toml
[integrations.dart]
enabled = true
sdk_path = "/usr/lib/dart"
pub_cache = "~/.pub-cache"
```

#### Docker Integration

**Purpose**: Container management

**Configuration**:
```toml
[integrations.docker]
enabled = true
socket = "/var/run/docker.sock"
registry = "docker.io"
```

**Implementation**:
```rust
// server/src/integrations/docker.rs
use bollard::Docker;

pub struct DockerIntegration {
    docker: Docker,
}

impl DockerIntegration {
    pub fn new() -> Result<Self, Error> {
        let docker = Docker::connect_with_socket_defaults()?;
        Ok(Self { docker })
    }
    
    pub async fn list_containers(&self) -> Result<Vec<Container>, Error> {
        let containers = self.docker.list_containers(None).await?;
        Ok(containers)
    }
}
```

#### Kubernetes Integration

**Purpose**: Container orchestration

**Configuration**:
```toml
[integrations.kubernetes]
enabled = true
context = "default"
namespace = "noa-ark-os"
```

**Implementation**:
```rust
// server/src/integrations/k8s.rs
use kube::{Client, Api};
use k8s_openapi::api::core::v1::Pod;

pub struct K8sIntegration {
    client: Client,
}

impl K8sIntegration {
    pub async fn new() -> Result<Self, Error> {
        let client = Client::try_default().await?;
        Ok(Self { client })
    }
    
    pub async fn list_pods(&self, namespace: &str) -> Result<Vec<Pod>, Error> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let pod_list = pods.list(&Default::default()).await?;
        Ok(pod_list.items)
    }
}
```

#### NPM Integration

**Purpose**: Node.js package management

**Configuration**:
```toml
[integrations.npm]
enabled = true
registry = "https://registry.npmjs.org"
```

#### Cloudflare Integration

**Purpose**: CDN and tunnel services

**Configuration**:
```toml
[integrations.cloudflare]
enabled = true
zone_id = "${CLOUDFLARE_ZONE_ID}"
tunnel_token = "${CLOUDFLARE_TUNNEL_TOKEN}"
```

**Implementation**:
```rust
// server/src/integrations/cloudflare.rs
use cloudflare::framework::{auth::Credentials, Environment};

pub struct CloudflareIntegration {
    client: cloudflare::framework::Client,
}

impl CloudflareIntegration {
    pub fn new(api_token: String) -> Result<Self, Error> {
        let credentials = Credentials::UserAuthToken { token: api_token };
        let client = cloudflare::framework::Client::new(
            credentials,
            Default::default(),
            Environment::Production,
        )?;
        
        Ok(Self { client })
    }
}
```

#### Azure SDK Integration

**Purpose**: Azure cloud services

**Configuration**:
```toml
[integrations.azure]
enabled = true
tenant_id = "${AZURE_TENANT_ID}"
client_id = "${AZURE_CLIENT_ID}"
subscription_id = "${AZURE_SUBSCRIPTION_ID}"
```

### Integration Manager

**Centralized integration management**:

```rust
// server/src/integrations/mod.rs
pub struct IntegrationManager {
    github: Option<GitHubIntegration>,
    vscode: Option<VSCodeIntegration>,
    llama_cpp: Option<LlamaCppIntegration>,
    aws: Option<AWSIntegration>,
    docker: Option<DockerIntegration>,
    k8s: Option<K8sIntegration>,
    npm: Option<NPMIntegration>,
    cloudflare: Option<CloudflareIntegration>,
    azure: Option<AzureIntegration>,
    cargo: Option<CargoIntegration>,
    dart: Option<DartIntegration>,
    caddy: Option<CaddyManager>,
}

impl IntegrationManager {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        Ok(Self {
            github: if config.integrations.github.enabled {
                Some(GitHubIntegration::new(config.integrations.github.token.clone()).await?)
            } else { None },
            
            vscode: if config.integrations.vscode.enabled {
                Some(VSCodeIntegration::new().await?)
            } else { None },
            
            llama_cpp: if config.integrations.llama_cpp.enabled {
                Some(LlamaCppIntegration::new(&config.integrations.llama_cpp.model_path)?)
            } else { None },
            
            aws: if config.integrations.aws.enabled {
                Some(AWSIntegration::new().await?)
            } else { None },
            
            docker: if config.integrations.docker.enabled {
                Some(DockerIntegration::new()?)
            } else { None },
            
            k8s: if config.integrations.k8s.enabled {
                Some(K8sIntegration::new().await?)
            } else { None },
            
            npm: if config.integrations.npm.enabled {
                Some(NPMIntegration::new()?)
            } else { None },
            
            cloudflare: if config.integrations.cloudflare.enabled {
                Some(CloudflareIntegration::new(config.integrations.cloudflare.api_token.clone())?)
            } else { None },
            
            azure: if config.integrations.azure.enabled {
                Some(AzureIntegration::new().await?)
            } else { None },
            
            cargo: if config.integrations.cargo.enabled {
                Some(CargoIntegration::new()?)
            } else { None },
            
            dart: if config.integrations.dart.enabled {
                Some(DartIntegration::new()?)
            } else { None },
            
            caddy: if config.integrations.caddy.enabled {
                Some(CaddyManager::new(config.integrations.caddy.admin_endpoint.clone()))
            } else { None },
        })
    }
}
```

## Updated Tech Stack

### External Integration Crates

```toml
[dependencies]
# ...existing dependencies...

# External Integrations
octocrab = "0.32"                  # GitHub API
tower-lsp = "0.20"                 # Language Server Protocol
llama-cpp-rs = "0.1"               # Llama.cpp bindings
aws-config = "1.0"                 # AWS SDK
aws-sdk-s3 = "1.0"
aws-sdk-dynamodb = "1.0"
aws-sdk-secretsmanager = "1.0"
bollard = "0.15"                   # Docker API
kube = "0.87"                      # Kubernetes client
k8s-openapi = "0.20"
cloudflare = "0.10"                # Cloudflare API
azure_core = "0.17"                # Azure SDK
azure_identity = "0.17"
```

---

## Build Pipeline

### Workspace Structure

```
server/
├── Cargo.toml              # Workspace root
├── core/                   # Core functionality
│   └── Cargo.toml
├── api/                    # API layer
│   └── Cargo.toml
├── inference/              # Inference engine
│   └── Cargo.toml
├── retrieval/              # Retrieval/RAG
│   └── Cargo.toml
├── plugins/                # Plugin runtime
│   └── Cargo.toml
├── observability/          # Logging/metrics
│   └── Cargo.toml
└── cli/                    # CLI binary
    └── Cargo.toml
```

### Build Profiles

**Development**:
```toml
[profile.dev]
opt-level = 0
debug = true
```

**Release**:
```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
```

### Quality Checks

```bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --all --all-features -- -D warnings

# Tests
cargo test --all --all-features

# Security audit
cargo deny check
```

### Security

```bash
# Generate SBOM
syft scan . -o spdx-json > sbom.json

# Container scanning
trivy image noa-unified-server:latest

# Sign artifacts
cosign sign --key cosign.key noa-unified-server:latest
```

### Build Artifacts

```
dist/
├── noa-unified-server        # Binary
├── sbom.json                 # Software Bill of Materials
├── HASHES.txt                # SHA256 checksums
└── noa-unified-server.sig    # Signature
```

---

## Packaging

### Docker

**Multi-stage Dockerfile**:
```dockerfile
# Build stage
FROM rust:1.75-slim as builder
WORKDIR /build
COPY . .
RUN cargo build --release --bin noa-unified-server

# Runtime stage
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /build/target/release/noa-unified-server /usr/local/bin/
USER nonroot:nonroot
EXPOSE 8080 9090
ENTRYPOINT ["/usr/local/bin/noa-unified-server"]
```

**Build & Push**:
```bash
# Build multi-arch
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t noa-unified-server:latest \
  --push .
```

**Environment**:
```bash
RUST_LOG=info
NOA_SERVER__HOST=0.0.0.0
NOA_SERVER__PORT=8080
```

### Binary Targets

```bash
# x86_64 Linux (GNU)
cargo build --release --target x86_64-unknown-linux-gnu

# ARM64 Linux (GNU)
cargo build --release --target aarch64-unknown-linux-gnu

# x86_64 Linux (MUSL) - static binary
cargo build --release --target x86_64-unknown-linux-musl
```

---

## Deployment

### Docker Compose

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: noa
      POSTGRES_USER: noa
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
  
  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"
  
  qdrant:
    image: qdrant/qdrant:latest
    volumes:
      - qdrant_data:/qdrant/storage
    ports:
      - "6333:6333"
  
  server:
    image: noa-unified-server:latest
    depends_on:
      - postgres
      - redis
      - qdrant
    environment:
      RUST_LOG: info
      NOA_DATABASE__URL: postgresql://noa:${POSTGRES_PASSWORD}@postgres:5432/noa
      NOA_CACHE__URL: redis://redis:6379
    ports:
      - "8080:8080"
      - "9090:9090"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  postgres_data:
  redis_data:
  qdrant_data:
```

### Kubernetes Helm Chart

```yaml
# server/helm/values.yaml
replicaCount: 3

image:
  repository: noa-unified-server
  tag: latest
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 8080
  metricsPort: 9090

resources:
  limits:
    cpu: 2000m
    memory: 4Gi
  requests:
    cpu: 500m
    memory: 1Gi

livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5

serviceMonitor:
  enabled: true
  interval: 30s
```

---

## Tests

### Unit Tests

```bash
cargo test --all --all-features
```

### Integration Tests

```bash
# Start ephemeral compose
docker-compose -f docker-compose.test.yml up -d

# Run integration tests
cargo test --test integration -- --test-threads=1

# Cleanup
docker-compose -f docker-compose.test.yml down -v
```

### Performance Tests

**Criterion benchmarks**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_inference(c: &mut Criterion) {
    let engine = InferenceEngine::new().unwrap();
    c.bench_function("inference", |b| {
        b.iter(|| engine.infer(black_box(input.clone())))
    });
}

criterion_group!(benches, bench_inference);
criterion_main!(benches);
```

**Load testing with k6**:
```javascript
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  vus: 100,
  duration: '30s',
};

export default function() {
  let response = http.post('http://localhost:8080/v1/infer', JSON.stringify({
    input: 'test input'
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
  
  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
  });
}
```

### Smoke Tests

```bash
#!/bin/bash
# Start server
./noa-unified-server &
SERVER_PID=$!

# Wait for startup
sleep 5

# Ping health
curl -f http://localhost:8080/health || exit 1

# Cleanup
kill $SERVER_PID
exit 0
```

---

## Key Architectural Decisions

### 1. Rust-First Monolith
**Decision**: Single Rust binary with modular crates
**Rationale**: Reduces cross-service overhead, simplifies deployment, maintains coherence
**Trade-off**: Less flexibility for polyglot services

### 2. Single Port HTTP/1.1 + HTTP/2 (ALPN)
**Decision**: Combine axum + tonic on same port
**Rationale**: Simpler networking, single entry point
**Implementation**: ALPN negotiation for protocol selection

### 3. WASM Plugin Runtime
**Decision**: Use wasmtime with scoped capabilities
**Rationale**: Safe extensibility without compromising security
**Trade-off**: Some performance overhead vs native

### 4. PostgreSQL Primary + Redis Cache + Qdrant Vectors
**Decision**: Purpose-specific storage layers
**Rationale**: Right tool for each job
**Trade-off**: More operational complexity

### 5. OpenTelemetry Native
**Decision**: OTel for all observability
**Rationale**: Industry standard, vendor-neutral
**Trade-off**: Some initial complexity

### 6. Caddy for Reverse Proxy and HTTPS
**Decision**: Integrate Caddy as a reverse proxy
**Rationale**: Automatic HTTPS, advanced routing, and load balancing
**Trade-off**: Introduces Go dependency, additional configuration

### 7. Modular External Tool Integrations
**Decision**: Pluggable integrations for external tools and services
**Rationale**: Flexibility to support various development and deployment workflows
**Trade-off**: Increased complexity in configuration and initialization

---

## Build Commands

### Development

```bash
# Run server
cargo run --bin noa-unified-server

# Run with specific profile
cargo run --bin noa-unified-server -- --profile dev

# Watch for changes
cargo watch -x run
```

### Production Build

```bash
# Build release binary
cargo build --release --bin noa-unified-server

# Generate SBOM
syft scan target/release/noa-unified-server -o spdx-json > sbom.json

# Generate checksums
sha256sum target/release/noa-unified-server > HASHES.txt

# Sign
cosign sign-blob --key cosign.key target/release/noa-unified-server > noa-unified-server.sig
```

### Docker

```bash
# Build image
docker build -t noa-unified-server:latest .

# Run container
docker run -p 8080:8080 -p 9090:9090 \
  -e RUST_LOG=info \
  noa-unified-server:latest
```

---

## Health Endpoints

### Liveness Probe
```
GET /health
```

**Response**:
```json
{
  "status": "ok",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### Readiness Probe
```
GET /ready
```

**Response**:
```json
{
  "status": "ready",
  "checks": {
    "database": "ok",
    "cache": "ok",
    "inference": "ok"
  }
}
```

### Metrics
```
GET /metrics
```

**Response** (Prometheus format):
```
# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="GET",path="/v1/infer"} 1234

# HELP http_request_duration_seconds HTTP request duration
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{le="0.1"} 950
http_request_duration_seconds_bucket{le="0.5"} 1200
```

---

## Configuration Examples

### Development (dev.toml)
```toml
[server]
host = "127.0.0.1"
port = 8080
log_level = "debug"

[database]
url = "postgresql://localhost:5432/noa_dev"

[inference]
device = "cpu"
```

### Production (prod.toml)
```toml
[server]
host = "0.0.0.0"
port = 8080
log_level = "info"

[database]
url = "${DATABASE_URL}"
max_connections = 50

[inference]
device = "cuda"

[security]
tls_enabled = true
jwt_secret = "${JWT_SECRET}"
```

---

## Monitoring & Alerting

### Prometheus Rules
```yaml
groups:
  - name: noa_server
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        annotations:
          summary: "High error rate detected"
      
      - alert: HighLatency
        expr: histogram_quantile(0.95, http_request_duration_seconds) > 1
        annotations:
          summary: "High latency detected"
```

### Grafana Dashboard
- Request rate (req/s)
- Error rate (%)
- Latency (p50, p95, p99)
- Resource usage (CPU, memory)
- Active connections
- Database connection pool

---

## Security Checklist

- [ ] TLS enabled for all external traffic
- [ ] JWT tokens validated with RS256
- [ ] Capability tokens verified with HMAC
- [ ] All secrets in environment variables
- [ ] Non-root user in containers
- [ ] SBOM generated and published
- [ ] Container images scanned with Trivy
- [ ] Dependencies audited with cargo-deny
- [ ] Artifacts signed with cosign
- [ ] Rate limiting enabled
- [ ] Input validation on all endpoints
- [ ] SQL injection prevention (parameterized queries)

---

## Next Steps

1. Implement core server structure
2. Add gateway with health endpoints
3. Integrate observability stack
4. Add persistence layer
5. Implement inference engine
6. Add retrieval capabilities
7. Set up CI/CD pipeline
8. Write comprehensive tests
9. Create Helm charts
10. Document API endpoints
