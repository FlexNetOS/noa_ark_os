# Task Execution Kit Rust Upgrade Architecture

## Design Philosophy

The upgrade follows the **"Heal, Don't Harm"** principle, preserving the Rust implementation's strengths while integrating the Python version's proven functionality. The result will be a hybrid architecture that combines:

- **Rust's type safety and performance** with **Python's proven task execution patterns**
- **Effect-based capability security** with **multi-agent orchestration**
- **Zero-dependency philosophy** with **comprehensive feature set**

## Target Architecture

### Enhanced Workspace Structure

```
task_exec_kit_rust/
├── Cargo.toml                    # Enhanced workspace
├── crates/
│   ├── abi/                      # [EXISTING] Data structures
│   │   ├── src/lib.rs           # Enhanced with task/agent types
│   │   └── Cargo.toml
│   ├── kernel/                   # [ENHANCED] Core orchestration
│   │   ├── src/
│   │   │   ├── lib.rs           # Unified exports
│   │   │   ├── effect.rs        # [EXISTING] Effect trait
│   │   │   ├── broker.rs        # [EXISTING] Resource management
│   │   │   ├── ledger.rs        # [EXISTING] State tracking
│   │   │   ├── agent.rs         # [NEW] Agent trait and registry
│   │   │   ├── executor.rs      # [NEW] Parallel execution engine
│   │   │   └── permit_check.rs  # [EXISTING] Security
│   │   └── Cargo.toml
│   ├── planner/                  # [NEW] SoT parsing and planning
│   │   ├── src/
│   │   │   ├── lib.rs           # Public API
│   │   │   ├── parser.rs        # SoT markdown parsing
│   │   │   ├── queue.rs         # Queue extraction
│   │   │   └── task.rs          # Task identification
│   │   └── Cargo.toml
│   ├── agents/                   # [NEW] Agent implementations
│   │   ├── src/
│   │   │   ├── lib.rs           # Agent registry
│   │   │   ├── noop.rs          # Evidence stub agent
│   │   │   ├── shell.rs         # Command execution agent
│   │   │   └── effect.rs        # Effect-based agent adapter
│   │   └── Cargo.toml
│   ├── cli/                      # [NEW] Command-line interface
│   │   ├── src/
│   │   │   ├── main.rs          # CLI entry point
│   │   │   ├── commands/        # Subcommands
│   │   │   │   ├── mod.rs
│   │   │   │   ├── plan.rs      # Plan generation
│   │   │   │   └── run.rs       # Plan execution
│   │   │   └── config.rs        # Configuration loading
│   │   └── Cargo.toml
│   ├── effects-file/             # [EXISTING] File operations
│   ├── effects-net/              # [EXISTING] Network operations
│   ├── effects-process/          # [ENHANCED] Process execution
│   └── runner/                   # [ENHANCED] Main orchestrator
├── config/                       # [NEW] Configuration files
│   ├── manifest.json            # Agent and layer configuration
│   ├── hooks.json               # Task-to-command mappings
│   └── examples/                # Example configurations
├── schemas/                      # [EXISTING] JSON schemas
└── docs/                        # [NEW] Documentation
    ├── ARCHITECTURE.md
    ├── MIGRATION.md
    └── API.md
```

## Core Components Design

### 1. Enhanced ABI (abi/)

**Purpose**: Extend existing data structures with task execution types

```rust
// Enhanced abi/src/lib.rs
pub mod budget;     // [EXISTING] Resource tracking
pub mod permit;     // [EXISTING] Security model
pub mod envelope;   // [EXISTING] Effect envelope
pub mod task;       // [NEW] Task and queue types
pub mod agent;      // [NEW] Agent metadata
pub mod plan;       // [NEW] Execution plan structures

// New task types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub layer: String,
    pub queue: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Queue {
    pub name: String,
    pub tasks: Vec<Task>,
    pub hooks: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub version: u32,
    pub source: String,
    pub generated_at: String,
    pub queues: Vec<Queue>,
}
```

### 2. Enhanced Kernel (kernel/)

**Purpose**: Unified orchestration combining effects and agents

```rust
// kernel/src/agent.rs - New agent system
pub trait Agent: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn execute(
        &self,
        task: &Task,
        queue: &Queue,
        hooks: &serde_json::Value,
        context: &ExecutionContext,
    ) -> Result<AgentResult>;
}

pub struct AgentRegistry {
    agents: HashMap<String, Box<dyn Agent>>,
}

// kernel/src/executor.rs - New parallel execution
pub struct ParallelExecutor {
    registry: AgentRegistry,
    broker: Box<dyn Broker>,
    ledger: Box<dyn Ledger>,
    max_workers: usize,
}

impl ParallelExecutor {
    pub async fn execute_plan(&self, plan: ExecutionPlan) -> Result<ExecutionResult> {
        // Parallel task execution with resource management
    }
}
```

### 3. SoT Planner (planner/)

**Purpose**: Parse sot.md files and generate execution plans

```rust
// planner/src/parser.rs
pub struct SotParser {
    queue_regex: Regex,
    task_regex: Regex,
}

impl SotParser {
    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<ExecutionPlan> {
        // Port Python planner logic to Rust
    }
    
    fn extract_queues(&self, content: &str) -> Result<Vec<Queue>> {
        // Queue extraction with regex
    }
    
    fn extract_tasks(&self, line: &str) -> Vec<Task> {
        // Task ID extraction
    }
}
```

### 4. Agent Implementations (agents/)

**Purpose**: Concrete agent implementations

```rust
// agents/src/noop.rs
pub struct NoOpAgent;

impl Agent for NoOpAgent {
    fn name(&self) -> &'static str { "noop" }
    
    fn execute(&self, task: &Task, queue: &Queue, hooks: &serde_json::Value, context: &ExecutionContext) -> Result<AgentResult> {
        // Generate evidence stub
    }
}

// agents/src/shell.rs
pub struct ShellAgent {
    allow_execution: bool,
}

impl Agent for ShellAgent {
    fn execute(&self, task: &Task, queue: &Queue, hooks: &serde_json::Value, context: &ExecutionContext) -> Result<AgentResult> {
        // Execute shell commands with logging
    }
}

// agents/src/effect.rs - Bridge to effect system
pub struct EffectAgent<E: Effect> {
    effect: E,
}

impl<E: Effect> Agent for EffectAgent<E> {
    fn execute(&self, task: &Task, queue: &Queue, hooks: &serde_json::Value, context: &ExecutionContext) -> Result<AgentResult> {
        // Convert task to effect envelope and execute
    }
}
```

### 5. CLI Interface (cli/)

**Purpose**: Command-line interface matching Python functionality

```rust
// cli/src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "task-exec-kit")]
#[command(about = "NOA Task Execution Kit (Rust)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Plan {
        #[arg(long)]
        sot: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    Run {
        #[arg(long)]
        plan: PathBuf,
        #[arg(long, default_value = "4")]
        max_workers: usize,
        #[arg(long)]
        dry_run: bool,
    },
}
```

## Integration Strategy

### Phase 1: Foundation (Core Components)
1. **Enhance ABI**: Add task, queue, and plan types
2. **Create Planner**: Implement SoT parsing
3. **Basic Agents**: NoOp and Shell agents
4. **CLI Skeleton**: Basic plan/run commands

### Phase 2: Execution Engine
1. **Parallel Executor**: Multi-threaded task execution
2. **Agent Registry**: Dynamic agent management
3. **Configuration System**: JSON-based config loading
4. **Evidence Logging**: Structured output

### Phase 3: Effect Integration
1. **Effect-Agent Bridge**: Connect effect system to agents
2. **Security Integration**: Permit checking for agents
3. **Resource Management**: Budget tracking for tasks
4. **Advanced Features**: Error handling, recovery

### Phase 4: Advanced Features
1. **Enhanced CLI**: Full feature parity with Python
2. **Configuration Management**: Advanced config options
3. **Monitoring**: Progress tracking and metrics
4. **Documentation**: Comprehensive guides

## Compatibility Matrix

| Feature | Python Version | Rust Target | Integration Method |
|---------|---------------|-------------|-------------------|
| SoT Parsing | Regex-based | Regex + nom parser | Direct port + enhancement |
| Task Planning | JSON output | Typed structures | Serde serialization |
| Agent System | Duck typing | Trait-based | Type-safe implementation |
| Parallel Execution | ThreadPoolExecutor | Tokio/Rayon | Async/parallel hybrid |
| Shell Commands | subprocess | std::process | Enhanced with logging |
| Configuration | JSON files | Typed config | Serde deserialization |
| Evidence Logging | Text files | Structured logs | JSON + text output |
| CLI Interface | argparse | clap | Feature-complete port |

## Migration Path

### Backward Compatibility
- **Same CLI interface**: `plan` and `run` commands
- **Same configuration format**: JSON manifest and hooks
- **Same output format**: Compatible evidence files
- **Same SoT parsing**: Handles existing sot.md files

### Enhanced Features
- **Type safety**: Compile-time guarantees
- **Better performance**: Rust's zero-cost abstractions
- **Memory safety**: No segfaults or memory leaks
- **Capability security**: Effect-based permissions
- **Resource tracking**: Budget management
- **Better error handling**: Result types and error chains

## Success Criteria

### Functional Requirements
- [ ] Parse same sot.md files as Python version
- [ ] Generate compatible execution plans
- [ ] Execute tasks in parallel with same semantics
- [ ] Produce equivalent evidence outputs
- [ ] Support dry-run mode
- [ ] Handle shell command execution safely

### Non-Functional Requirements
- [ ] Faster execution than Python version
- [ ] Lower memory usage
- [ ] Type-safe operations
- [ ] Comprehensive error handling
- [ ] Maintainable codebase
- [ ] Extensive test coverage

### Integration Requirements
- [ ] Preserve existing effect system
- [ ] Maintain capability security model
- [ ] Support budget tracking
- [ ] Enable ledger verification
- [ ] Allow effect-agent bridging
- [ ] Provide migration tools

## Risk Mitigation

### Technical Risks
- **Regex complexity**: Use nom parser as fallback
- **Async complexity**: Hybrid sync/async design
- **Effect integration**: Gradual migration approach
- **Performance**: Benchmark against Python version

### Compatibility Risks
- **CLI changes**: Maintain exact interface
- **Output format**: Extensive testing
- **Configuration**: Validation against Python version
- **Behavior differences**: Comprehensive test suite

## Implementation Timeline

### Week 1: Foundation
- Enhanced ABI types
- Basic SoT parser
- CLI skeleton

### Week 2: Core Features
- Agent trait and registry
- NoOp and Shell agents
- Basic execution engine

### Week 3: Integration
- Parallel executor
- Configuration system
- Evidence logging

### Week 4: Polish
- Effect integration
- Error handling
- Documentation
- Testing

This architecture provides a clear path to upgrade the Rust implementation while preserving its strengths and adding the proven functionality from the Python version.
