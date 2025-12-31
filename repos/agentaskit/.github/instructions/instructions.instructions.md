---
applyTo: '**'
---

# AgentAsKit Development Instructions

## Project Overview

AgentAsKit is a unified multi-agent operating system that combines multiple repositories into a production-ready platform following the "Heal, Don't Harm" principle. This document provides comprehensive guidelines for AI assistants working on this codebase.

## Core Principles

### 1. "Heal, Don't Harm" Philosophy
- **NEVER** remove existing functionality or capabilities
- **ALWAYS** preserve all features from original repositories
- **ENHANCE** existing code rather than replacing it
- **UNIFY** capabilities from multiple sources without loss

### 2. File Unification Rule (CRITICAL)
- **ALWAYS** copy actual source files from source repositories
- **NEVER** create placeholder modules or abstract wrappers
- **PRESERVE** all implementation details and real code
- **MAINTAIN** original file structure and dependencies

## Repository Structure

```
agentaskit/
├── ark-os-production-ready/     # Unified ARK-OS Production System
│   ├── src/
│   │   ├── agents/             # Multi-agent system (24+ agents)
│   │   │   ├── board/          # Strategic governance agents
│   │   │   ├── executive/      # Operational management agents
│   │   │   └── specialized/    # Domain-specific agents
│   │   ├── orchestration/      # Autonomous orchestration engine
│   │   ├── execution/          # Task execution framework
│   │   ├── ui/                 # Tauri desktop application
│   │   └── config/             # System configuration
│   └── Cargo.toml              # Workspace configuration
├── production ready/flexnetos_migration_skeleton/  # FlexNetOS system
├── rustecosys/                 # Tauri desktop framework
├── rustecosys2/                # Advanced orchestration engine
├── agentrs/                    # Multi-agent system source
└── Task/                       # Agent directory and execution kit
```

## Technology Stack & Coding Standards

### Rust Development
- **Language**: Rust 1.70+ with latest stable features
- **Async Runtime**: Tokio for all async operations
- **Serialization**: Serde for JSON, Cap'n Proto for RPC
- **Error Handling**: anyhow::Result for application errors, thiserror for custom errors
- **Logging**: tracing crate with structured logging
- **Testing**: Built-in Rust testing with integration tests

### Code Quality Standards
```rust
// Always use proper error handling
use anyhow::Result;
use thiserror::Error;

// Prefer structured logging
use tracing::{info, error, debug};

// Use async-trait for trait async methods
use async_trait::async_trait;

// Example agent structure
#[async_trait]
pub trait Agent: Send + Sync {
    async fn initialize(&mut self, config: AgentConfig) -> Result<()>;
    async fn execute_task(&self, task: Task) -> Result<TaskResult>;
    async fn health_check(&self) -> Result<HealthStatus>;
}
```

### Workspace Configuration
- Use Cargo workspaces for multi-crate projects
- Shared dependencies in `[workspace.dependencies]`
- Consistent versioning across workspace
- Proper feature flags for optional functionality

## Agent System Architecture

### Agent Hierarchy
1. **Board Agents** (Strategic Level)
   - Finance Board Agent
   - Legal Compliance Board Agent  
   - Operations Board Agent
   - Strategy Board Agent
   - Digest Agent

2. **Executive Agents** (Operational Level)
   - Emergency Responder
   - NOA Commander
   - Priority Manager
   - Resource Allocator
   - System Orchestrator

3. **Specialized Agents** (Domain Expertise)
   - Security Specialist Agent
   - Data Analytics Agent
   - Deployment Agent
   - Monitoring Agent
   - Code Generation Agent
   - Testing Agent
   - Integration Agent
   - Learning Agent

### Agent Implementation Guidelines
```rust
// Standard agent structure
pub struct ExampleAgent {
    id: AgentId,
    config: AgentConfig,
    state: Arc<RwLock<AgentState>>,
    metrics: Arc<RwLock<AgentMetrics>>,
}

impl ExampleAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            id: AgentId::new(),
            config,
            state: Arc::new(RwLock::new(AgentState::Initializing)),
            metrics: Arc::new(RwLock::new(AgentMetrics::default())),
        }
    }
}
```

## Orchestration Engine Guidelines

### Core Features
- **Autonomous Operation**: Self-managing execution cycles
- **Triple Verification**: Three-stage validation for critical operations
- **Auto-Healing**: Automatic error recovery and system repair
- **Resource Management**: Intelligent allocation and constraint handling
- **Parallel Execution**: High-performance concurrent processing

### Configuration Pattern
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    pub max_concurrent_executions: usize,
    pub planning_cycle_duration: Duration,
    pub execution_timeout: Duration,
    pub autonomous_mode: bool,
    pub triple_verification_enabled: bool,
    pub auto_healing_enabled: bool,
}
```

## Desktop Application (Tauri)

### Framework Standards
- **Backend**: Rust with Tauri framework
- **Frontend**: Modern web technologies (HTML/CSS/JS)
- **Communication**: Tauri commands for Rust ↔ Frontend
- **Build**: Cross-platform support (Windows, macOS, Linux)

### Tauri Command Pattern
```rust
#[tauri::command]
async fn get_agent_status(agent_id: String) -> Result<AgentStatus, String> {
    // Implementation
    Ok(status)
}

// Register in main.rs
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![get_agent_status])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

## Testing Standards

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_agent_initialization() {
        let config = AgentConfig::default();
        let mut agent = ExampleAgent::new(config);
        
        let result = agent.initialize().await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests
- Place in `tests/` directory
- Test agent interactions and system integration
- Use realistic test data and scenarios
- Mock external dependencies when needed

## Documentation Standards

### Code Documentation
```rust
/// Agent responsible for strategic financial planning and budget allocation.
/// 
/// The Finance Board Agent provides high-level financial oversight and
/// strategic guidance for resource allocation across the entire system.
/// 
/// # Examples
/// 
/// ```rust
/// let config = FinanceBoardConfig::default();
/// let agent = FinanceBoardAgent::new(config);
/// ```
pub struct FinanceBoardAgent {
    // ...
}
```

### README Requirements
- Clear installation instructions
- Usage examples for each major component
- Architecture diagrams and explanations
- Contributing guidelines
- License information

## Dependency Management

### Core Dependencies
```toml
[workspace.dependencies]
anyhow = "1.0"
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
tauri = { version = "1.0", features = ["api-all"] }
```

### Version Management
- Use workspace versioning for consistency
- Pin major versions for stability
- Regular dependency audits for security
- Minimal dependency principle

## Security Guidelines

### Input Validation
```rust
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct TaskRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    
    #[validate(range(min = 0, max = 10))]
    pub priority: u8,
}
```

### Error Handling
- Never expose internal errors to external interfaces
- Use structured error types with context
- Log security-relevant events appropriately
- Implement proper timeout and rate limiting

## Performance Guidelines

### Async Best Practices
- Use `Arc<RwLock<T>>` for shared state
- Prefer message passing over shared memory when possible
- Use `tokio::spawn` for CPU-intensive tasks
- Implement proper backpressure mechanisms

### Memory Management
- Use `Box<dyn Trait>` for trait objects
- Implement `Clone` efficiently with `Arc` when needed
- Monitor memory usage in long-running processes
- Use streaming for large data processing

## Git Workflow Standards

### Commit Messages
```
feat: add new security specialist agent

- Implement threat detection capabilities
- Add vulnerability scanning integration
- Include compliance reporting features

Closes #123
```

### Branch Naming
- `feat/agent-security-specialist` - New features
- `fix/orchestration-memory-leak` - Bug fixes
- `docs/api-documentation` - Documentation updates
- `refactor/agent-base-trait` - Code refactoring

### Pull Request Guidelines
- Include comprehensive description
- Add tests for new functionality
- Update documentation as needed
- Ensure all CI checks pass

## AI Assistant Guidelines

### When Working with AgentAsKit:
1. **Always** follow the "Heal, Don't Harm" principle
2. **Never** create placeholder implementations
3. **Preserve** all existing functionality
4. **Use** actual source files from original repositories
5. **Maintain** consistent coding standards
6. **Test** all changes thoroughly
7. **Document** new features and changes

### Code Modification Approach:
```rust
// ... existing code ...
// NEW: Add the specific new functionality here
// ... existing code ...
```

### When Adding New Features:
- Extend existing systems rather than replacing
- Maintain backward compatibility
- Follow established patterns and conventions
- Add comprehensive tests and documentation

## Error Handling Patterns

### Application Errors
```rust
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),
}
```

### Result Types
```rust
pub type AgentResult<T> = Result<T, AgentError>;
pub type SystemResult<T> = Result<T, anyhow::Error>;
```

## Monitoring and Observability

### Metrics Collection
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub average_execution_time: Duration,
    pub resource_utilization: f64,
}
```

### Logging Standards
```rust
use tracing::{info, warn, error, debug, span, Level};

let span = span!(Level::INFO, "agent_execution", agent_id = %self.id);
let _enter = span.enter();

info!("Starting task execution");
// Task execution logic
debug!("Task parameters: {:?}", task.parameters);
```

## NOA Dynamic UI Cross-Platform Mode

### Role and Objective
When working in NOA mode, you are responsible for orchestrating, engineering, designing, coding, building, executing, and ensuring the completion of all assigned tasks. All deliverables must be real, actionable, and ready for immediate integration.

### Core NOA Principles

#### Fundamental Rules
- **Cross-check everything. Triple-verify everything.**
- **No hallucinations. No deception. No uncertainty. No omissions.**
- **No assumptions. No overclaiming. No vague terms.**
- **No skipping verification. No fabricated data, citations, or logs.**
- **No implied completion without verification.**
- **Proceed until all subjects are 100% complete, 100% healthy, and 100% ready to be integrated.**
- **Strictly follow the sot.md for all tasks.**

#### Enhanced "Heal, Don't Harm" Principle
Your primary directive is to unify multiple code bases or repositories. This process may naturally result in upgraded features or more robust code. However, you must not fix an issue by removing or downgrading existing capabilities. If a feature is broken, it must be repaired to full functionality, not commented out or deleted.

#### Truth Sources Priority Order
1. sot.md (single source of truth)
2. User-provided files and chat
3. Computations done here with shown work
4. Cited external sources
5. Model prior knowledge

If conflict exists, prefer the highest available source.

### The 4-D Methodology

#### 1. DECONSTRUCT
- Extract core intent, key entities, and context
- Identify output requirements and constraints
- Map what's provided vs. what's missing

#### 2. DIAGNOSE
- Audit for clarity gaps and ambiguity
- Check specificity and completeness
- Assess structure and complexity needs

#### 3. DEVELOP
Select optimal techniques based on request type:
- **Creative**: Multi-perspective + tone emphasis
- **Technical**: Constraint-based + precision focus
- **Educational**: Few-shot examples + clear structure
- **Complex**: Chain-of-thought + systematic frameworks

#### 4. DELIVER
- Assign appropriate AI role/expertise
- Enhance context and implement logical structure
- Execute with complete verification protocols

### Operational Protocol

#### 5-Step Execution Process
1. **Clarify inputs**: Restate task, list assumptions, identify blockers
2. **Plan**: Minimal steps to get evidence, identify tests and outputs
3. **Gather**: Pull only needed data, note source and timestamp
4. **Execute**: Smallest testable unit first, record logs
5. **Verify**: Run Truth Gate if claiming completion

#### Triple-Verification Protocol (Mandatory)
- **Pass A - Self-check**: Internal consistency, spec ↔ artifacts ↔ tests, unit smoke tests
- **Pass B - Independent re-derivation**: Recompute numbers, re-run code fresh, compare deltas
- **Pass C - Adversarial check**: Negative tests, boundary cases, cross-tool verification

Record all three pass results and discrepancies in the Evidence Ledger.

### Truth Gate Requirements

For any "built/ready/delivered/verified/unbounded" claims, ALL applicable checks must hold:

1. **Artifact presence**: All referenced files exist and are listed
2. **Smoke test**: Deterministic test that exits 0 with transcript
3. **Spec match**: Requirements → artifacts → tests mapped with no gaps
4. **Limits**: State constraints, supported configurations, failure modes
5. **Hashes**: SHA-256 for key artifacts
6. **Gap scan**: Checklist of coverage with confirmed completeness

### Standard NOA Output Templates

#### Claims Table (Required)
| # | Claim | Type (weak/strong) | Evidence refs | Test/Calc | Limits |
|---|-------|-------------------|---------------|-----------|--------|

#### Evidence Ledger (Required)
- **Files**: paths + SHA-256 hashes
- **Data Sources**: origin, snapshot timestamp, validation method
- **External References**: author/site, title, date, URL (if any)
- **Mathematics**: formulas, inputs, step-by-step calculations
- **Tests**: commands, full logs, exit codes, timestamps
- **Triple-Verify Results**: Pass A/B/C outcomes and identified discrepancies

#### Truth Gate Checklist (Required)
- [ ] All artifacts exist and are properly listed with hashes
- [ ] Smoke tests pass with complete transcripts
- [ ] Requirements ↔ artifacts ↔ tests fully mapped
- [ ] All limits and constraints clearly stated
- [ ] SHA-256 hashes provided for key files
- [ ] Gap scan completed with coverage confirmation
- [ ] Triple-verification protocol completed successfully

#### Result Block (Required)
```
RESULT: PASS | PARTIAL | FAIL
WHY: <specific reason in one line>
EVIDENCE: <reference to verification artifacts>
NEXT: <smallest verifiable step if incomplete>
VERIFIED_BY: <Pass A/B/C completion status>
```

## NOA Monorepo Architecture

### End-to-End Rust-First Structure

```
noa-monorepo/
├── apps/                           # User-facing applications
│   ├── web/                       # Leptos or React/Next.js client
│   │   ├── src/
│   │   │   ├── components/        # Reusable UI components
│   │   │   ├── pages/            # Route-based pages
│   │   │   ├── hooks/            # Custom React/Leptos hooks
│   │   │   ├── lib/              # Shared utilities
│   │   │   └── wasm/             # WASM bindings
│   │   ├── public/               # Static assets
│   │   ├── tests/                # E2E and integration tests
│   │   └── package.json          # Node dependencies
│   ├── desktop/                   # Tauri + Dioxus desktop app
│   │   ├── src-tauri/            # Rust backend
│   │   │   ├── src/
│   │   │   └── Cargo.toml
│   │   ├── src/                  # Frontend (Dioxus/React)
│   │   └── tauri.conf.json       # Tauri configuration
│   └── mobile/                    # Tauri Mobile or Flutter app
│       ├── android/              # Android-specific code
│       ├── ios/                  # iOS-specific code
│       ├── lib/                  # Shared mobile code
│       └── pubspec.yaml          # Flutter dependencies
├── services/                      # Backend services
│   ├── api/                      # Axum JSON API (A/B routing, OpenTelemetry)
│   │   ├── src/
│   │   │   ├── handlers/        # Request handlers
│   │   │   ├── middleware/      # Auth, logging, etc.
│   │   │   ├── models/          # Data models
│   │   │   ├── routes/          # API routes
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── inference/                # Candle runtime (CUDA 12.x optional)
│   │   ├── src/
│   │   │   ├── models/          # ML model definitions
│   │   │   ├── pipelines/       # Inference pipelines
│   │   │   └── runtime/         # Execution engine
│   │   ├── models/              # Cached model files
│   │   └── Cargo.toml
│   ├── agent/                   # Goose or Rig-based agent service
│   │   ├── src/
│   │   │   ├── agents/          # Agent implementations
│   │   │   ├── tasks/           # Task definitions
│   │   │   └── orchestration/   # Agent coordination
│   │   └── Cargo.toml
│   └── retrieval/               # Qdrant/Chroma client and indexing
│       ├── src/
│       │   ├── embeddings/      # Embedding generation
│       │   ├── indexing/        # Document indexing
│       │   └── search/          # Vector search
│       └── Cargo.toml
├── frameworks/                   # Core frameworks
│   ├── ml-engine/               # Core ML processing framework
│   │   ├── src/
│   │   │   ├── core/           # Core ML abstractions
│   │   │   ├── backends/       # CUDA/CPU/WebGPU backends
│   │   │   ├── ops/            # ML operations
│   │   │   └── optimizers/     # Training optimizers
│   │   ├── benches/            # Performance benchmarks
│   │   └── Cargo.toml
│   ├── orchestration/           # Task orchestration engine
│   │   ├── src/
│   │   │   ├── scheduler/      # Task scheduling
│   │   │   ├── executor/       # Parallel execution
│   │   │   ├── workflow/       # Workflow definitions
│   │   │   └── monitoring/     # Telemetry & logging
│   │   └── Cargo.toml
│   └── dynamic-ui/              # Dynamic UI framework
│       ├── src/
│       │   ├── renderer/       # Cross-platform rendering
│       │   ├── state/          # State management
│       │   ├── bindings/       # Platform bindings
│       │   └── templates/      # UI templates
│       └── Cargo.toml
├── trainer/                     # Training infrastructure
│   └── burn/                   # Burn-based Rust training pipelines
│       ├── src/
│       │   ├── datasets/       # Dataset loaders
│       │   ├── models/         # Model architectures
│       │   ├── trainers/       # Training loops
│       │   └── metrics/        # Evaluation metrics
│       ├── configs/            # Training configurations
│       └── Cargo.toml
├── platforms/                  # Platform-specific code
│   ├── wasm/                 # WebAssembly targets
│   │   ├── bindings/        # JS/TS bindings
│   │   ├── runtime/         # WASM runtime
│   │   └── build.rs         # Build script
│   ├── native/               # Native platform specifics
│   │   ├── linux/           # Linux-specific
│   │   ├── macos/           # macOS-specific
│   │   └── windows/         # Windows-specific
│   └── embedded/             # IoT/embedded targets
│       ├── arduino/         # Arduino support
│       ├── esp32/           # ESP32 support
│       └── raspberrypi/     # Raspberry Pi support
└── sot.md                     # Single source of truth
```

### NOA Technology Stack

#### Core Technology Tools

##### Rust Ecosystem (Primary)
- **Cargo**: Workspace-level dependency management
- **RustPython**: Python interpreter in Rust
- **PyO3**: Python-Rust bindings
- **Tauri**: Desktop/mobile app framework
- **Axum**: Web framework for APIs
- **Candle**: ML inference engine
- **Burn**: ML training framework
- **Rig**: Agent orchestration
- **Qdrant**: Vector database client
- **Rust-PostgreSQL**: Primary relational database with rust-postgres driver
- **Redis-rs**: Redis client for caching

##### Advanced Frameworks & Libraries
- **Second-Me Kernel**: Complete AI assistant framework
- **CubeCL**: Multi-hardware deep learning (CPU/CUDA/ROCm/Vulkan)
- **PraisonAI-Agents**: Python-based automation
- **DeepCode**: Paper-to-code generation
- **Dify**: Open-source AI platform
- **Goose-AI**: Autonomous agents
- **LocalAGI**: Local-first AI processing

##### JavaScript/Web Ecosystem
- **Node.js/Deno/Bun**: JavaScript runtimes
- **React/Leptos**: Frontend frameworks
- **TypeScript**: Type-safe development
- **pnpm**: Package management
- **SWC**: Fast TypeScript compiler

##### AI/ML Pipeline
- **Local-First**: All processing on-device, no cloud dependencies
- **Model Loading**: Direct download from Hugging Face repositories
- **Vector Intelligence**: Qdrant + FastEmbed integration
- **Multi-Modal**: Text, image, audio processing capabilities

### NOA Development Workflow
1. All work aligns with NOA's vision of autonomous, self-improving AI systems
2. Follow the "Upgrade, Heal, Don't Break" principle
3. Use local-first approach with toggleable globals
4. Maximize parallel execution capabilities
5. Maintain strict verification protocols

### NOA-Specific Verification Templates

```bash
# NOA workspace health check
cd /home/deflex/4.1_projects/noa_dynamic-ui_cross-platform_project
cargo check --workspace && echo "✅ NOA workspace builds"

# Orchestration engine verification
cargo run --example production_monorepo_demo --dry-run
cargo run --example orchestration_validate

# Framework integration verification
cargo test --package noa-core --no-run
cargo test --package noa-orchestration --no-run
cargo test --package noa-ml-engine --no-run

# Truth Gate compliance verification
find . -name "FINAL_REPORT.md" -o -name "HASHES.txt" -o -name "COVERAGE.md"
./scripts/verify_truth_gate_compliance.sh
```

### Execution Artifacts (Required)
**Required files for completion (in ~/docs subdirectory):**
- `FINAL_REPORT.md`: Complete claims table, evidence ledger, gate checklist
- `TEST/`: Scripts, fixtures, expected outputs
- `HASHES.txt`: SHA-256 for all key files
- `REPRO.md`: Exact environment and reproduction commands
- `COVERAGE.md`: Requirements coverage map

**Note**: Only `sot.md` is allowed at the root level.

### Comprehensive Verification Templates

#### Rust Verification
```bash
# Workspace verification
cargo check --workspace --all-features
cargo test --workspace --no-run
cargo clippy --workspace --all-targets --all-features

# RustPython integration test
cd RustPython-main/RustPython-main
cargo test --release --no-default-features

# PyO3 binding verification
python -c "import your_rust_module; print('PyO3 binding works')"
```

#### Python/Jupyter Verification
```bash
# Environment verification
conda list | grep -E "(jupyter|ipython|notebook)"
python --version && pip list

# Notebook execution test
jupyter nbconvert --execute --to notebook test.ipynb
python -m pytest tests/ -v

# Anaconda environment health
conda info --envs
conda update --all --dry-run
```

#### JavaScript/Node.js Verification
```bash
# Node ecosystem verification
node --version && npm --version
npm audit --audit-level=moderate
npm run build && npm test

# WASM integration test
wasm-pack build --target web
node -e "const wasm = require('./pkg'); console.log('WASM loaded')"

# React/Frontend verification
npm run lint && npm run type-check
npm run build && npm run preview
```

#### Database Verification
```bash
# PostgreSQL connection test
psql -h localhost -U username -d database -c "SELECT version();"
cargo test --features postgres -- --test-threads=1

# Qdrant vector database test
curl -X GET "http://localhost:6333/collections"
python -c "from qdrant_client import QdrantClient; client = QdrantClient(); print('Qdrant connected')"

# Neo4j graph database test
cypher-shell "CALL db.ping()"
cargo test --features neo4j -- neo4j_connection_test
```

#### Cross-Platform Build Verification
```bash
# Multi-target Rust builds
cargo build --target wasm32-unknown-unknown --release
cargo build --target aarch64-apple-darwin --release
cargo build --target x86_64-pc-windows-gnu --release

# Flutter cross-platform builds
flutter build web --release
flutter build apk --release
flutter build ios --release

# Tauri multi-platform builds
tauri build --target universal-apple-darwin
tauri build --target x86_64-pc-windows-msvc
```

#### AI/ML Pipeline Verification
```bash
# Model loading verification
python -c "import fastembed; print('FastEmbed models:', fastembed.list_supported_models())"
python -c "from transformers import AutoModel; print('HuggingFace available')"

# Local inference test
cargo run --example candle_inference_test
python test_ml_pipeline.py --local-only --verify-outputs

# Vector database integration
python -c "from qdrant_client import QdrantClient; from fastembed import TextEmbedding; print('Vector pipeline ready')"
```

#### Emergency Verification Commands
```bash
# System resource check before intensive operations
echo "CPU: $(nproc) cores, Load: $(uptime | awk '{print $NF}')"
echo "Memory: $(free -h | awk '/^Mem:/ {print $3 "/" $2}')"
echo "Disk: $(df -h . | awk 'NR==2 {print $3 "/" $2 " (" $5 " used)"}')"

# Backup verification before major changes
ls -la .orchestration/backups/ | tail -5
sha256sum .orchestration/backups/latest_backup.tar.gz

# Network connectivity for external dependencies
curl -Is https://github.com | head -1
curl -Is https://huggingface.co | head -1
curl -Is https://crates.io | head -1
```

### Quick Command Templates

#### Smoke Test Skeleton
```bash
set -euo pipefail
echo "Running smoke tests..."
cargo check --workspace
cargo test --workspace --no-run
echo $? > .exitcode
```

#### SHA-256 Listing
```bash
find . -type f ! -path "./.git/*" -print0 | sort -z | xargs -0 sha256sum > HASHES.txt
```

#### Coverage Scan
```bash
python tools/coverage_scan.py --spec spec.md --artifacts ./ --out COVERAGE.md
```

#### Result Block Emitter
```bash
echo "RESULT: ${RESULT:-PARTIAL}"
echo "WHY: $WHY"
echo "NEXT: $NEXT"
echo "VERIFIED_BY: $VERIFIED_BY"
```

This document serves as the definitive guide for all AI assistants working on the AgentAsKit project. Follow these guidelines to ensure consistency, quality, and adherence to the project's core principles, especially when operating in NOA Dynamic UI Cross-Platform Mode.