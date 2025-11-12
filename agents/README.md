# Agent Factory

Multi-layered AI agent system with hive mind coordination, swarm deployments, and disposable sub-agents.

## Architecture

### 5-Layer Agent Hierarchy

NOA ARK OS uses a hierarchical 5-layer architecture for organizing agents, from strategic governance to infrastructure tasks:

| Layer | Alternative Name | Description | Examples |
|-------|-----------------|-------------|----------|
| **L1Autonomy** | Executive | Root CECCA, Constitutional authority | CECCA, Constitutional Oversight |
| **L2Reasoning** | Board | Board & Executive decision-making | Board agents, Strategic Planning |
| **L3Orchestration** | Stack-Chief | Chief Commanders, Orchestrators | Stack Chiefs, Workflow Coordinators |
| **L4Operations** | Specialist | Domain specialists, Workers | Code agents, Security specialists |
| **L5Infrastructure** | Micro | Micro agents, Infrastructure tasks | Utility agents, Monitoring |

**Escalation Flow**: Agents escalate decisions upward (L5 → L4 → L3 → L2 → L1) when tasks exceed their authority level or require higher-level coordination.

**Historical Note**: The L1-L5 naming provides a clearer numerical hierarchy while maintaining backward compatibility with the original organizational names (Executive, Board, Stack-Chief, Specialist, Micro).

### Component Structure

```
agents/
├── factory/           # Agent creation and management
├── hive/              # Hive mind coordination
├── swarm/             # Swarm deployment system
├── layers/            # Multi-layered agent hierarchy
├── runtime/           # Cross-language runtime (Python/Rust/Go)
├── workflows/         # End-to-end process automation
├── disposable/        # Ephemeral sub-agent system
├── communication/     # Real-time messaging hub adapted from CRC drop
├── implementations/   # Concrete agent implementations (board, specialist, orchestrator)
└── agentaskit/        # Adapted AgentAsKit types and configuration
```

### AgentAsKit Integration

- Phase 1 brings across the shared configuration/error model from the `agentaskit` drop (`agents/src/agentaskit`).
- The `agent-src` drop seeds a task orchestrator (`implementations/orchestrator`) that schedules work against the embedded registry.
- The `agent-communication` drop is expressed as a Tokio-based hub (`communication/mod.rs`) that replaces Redis/WebSocket transport with in-memory broadcast channels for testing.
- Executive agent implementations remain feature-gated behind `agentaskit-executive` while we progressively port the runtime dependencies.
- The legacy drop is archived under `crc/archive/stale/agentaskit` once integration is complete.

## Agent Types

### 1. Master Agent
- Top-level orchestration
- Strategic decision making
- Resource allocation
- Hive mind coordinator

### 2. Worker Agents
- Task execution
- Parallel processing
- Specialized capabilities
- Report to master

### 3. Sub-Agents (Disposable)
- Single-purpose execution
- Short-lived
- Minimal resources
- Auto-cleanup

### 4. Swarm Agents
- Coordinated group behavior
- Distributed processing
- Self-organizing
- Fault tolerant

## Multi-Language Support

### Python Agents
- Rapid prototyping
- ML/AI libraries
- Data processing
- Scripting tasks

### Rust Agents
- High-performance tasks
- Systems programming
- Memory-safe operations
- Zero-cost abstractions

### Go Agents
- Concurrent services
- Network operations
- Microservices
- Goroutine-based parallelism

## Parallel Execution

- **Process-level**: Multiple OS processes
- **Thread-level**: Shared memory threading
- **Async-level**: Async/await concurrency
- **Distributed**: Multi-node execution

## Hive Mind

Collective intelligence system:
- Shared knowledge base
- Consensus decision making
- Load balancing
- Failure recovery

## Workflow Automation

### End-to-End Process
1. Task definition
2. Agent allocation
3. Parallel execution
4. Result aggregation
5. Quality verification
6. Cleanup

### Full Auto Mode
- Zero human intervention
- Self-healing
- Auto-scaling
- Continuous optimization

## Agent Lifecycle

```
Create → Initialize → Execute → Monitor → Terminate → Cleanup
   ↑                                                      |
   └──────────────────────────────────────────────────────┘
                    (Disposable agents)
```

## Communication

- Inter-agent messaging (IPC)
- Pub/sub event system
- Shared state management
- Real-time synchronization

## Capabilities

- Dynamic agent creation
- Hot-swapping agents
- Agent cloning
- Resource isolation
- Security sandboxing
