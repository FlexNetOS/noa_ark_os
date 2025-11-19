# NOA ARK OS - Python Services

Python infrastructure services for the NOA ARK OS agent system.

## 📦 Packages

### `agents/` (17 files)
Python agent implementations that complement the Rust core agents.

**Key Components**:
- `noa_commander.py` - Python NOA Commander implementation
- Agent communication and coordination systems

### `infrastructure/` (7 files + 34 other)
Core infrastructure services for the agent ecosystem.

**Key Components**:
- API Gateway services
- Service mesh integration
- Load balancing
- Service discovery

### `constitutional/` (6 files)
Governance and constitutional validation systems.

**Key Components**:
- CQRS system
- Distributed cache
- Message queue
- Validators and compliance systems

### `autonomy/` (5 files)
Self-* systems for autonomous operation.

**Key Components**:
- `self_debugging_repair_system.py` - Self-debugging
- `self_modification_system.py` - Self-modification
- `self_monitoring_system.py` - Self-monitoring
- `self_testing_framework.py` - Self-testing
- `proactive_autonomy_system.py` - Proactive autonomy

### `devops/` (16 files)
DevOps and operational tooling.

**Key Components**:
- Canary testing service
- Drift detection
- Metrics collection
- Rollback systems
- Promotion pipelines

### `optimization/` (14 files)
Performance optimization systems.

**Key Components**:
- Resource allocators
- Network optimizers
- Storage optimization
- Hardware tuning

## 🚀 Installation

```bash
cd server/python
pip install -r requirements.txt
```

## 🔧 Usage

### Running Individual Services

```bash
# Example: Start API Gateway
python infrastructure/unified_api_gateway.py

# Example: Start monitoring
python autonomy/self_monitoring_system.py
```

### Integration with Rust Agents

Python services can communicate with Rust agents via:
- HTTP/REST APIs
- gRPC interfaces
- Message queues
- Shared configuration

## 📊 Architecture

```
Python Services
├── agents/          - Agent implementations
├── infrastructure/  - Core services
├── constitutional/  - Governance
├── autonomy/       - Self-* systems
├── devops/         - Operations
└── optimization/   - Performance
        ↕
    HTTP/gRPC
        ↕
Rust Agent System
├── executive/      - L1/L2 agents
├── board/         - L2 agents
└── specialist/    - L4 agents
```

## 🔗 Related Systems

- **Rust Agents**: `../../agents/` - Core agent system
- **Go Services**: `../go/` - Performance services
- **Documentation**: `../../docs/` - Complete docs

## 📝 Notes

- All Python services are designed for independent operation
- Can integrate with Rust agents via APIs when needed
- Follows microservices architecture patterns
- Each service is self-contained and deployable
- Configure workspace paths via `NOA_WORKSPACE_PATH` (gateway-managed); the
  shared helper in `server.python.common.workspace` propagates the value so no
  manual edits are required inside the individual services.

## 🧪 Testing

```bash
pytest tests/
```

## 📚 Documentation

See `../../docs/architecture/` for complete system architecture.
