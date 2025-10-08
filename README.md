# noa_ark_os - Unified FlexNetOS Mono-Repository

**Local first. Self hosted. Full-Stack. Mono-Repo. End-to-End Auto. AgenticAI. Hive Mind. Agent Swarm. Disposable MicroAgents**

## 🎯 Overview

This is the unified mono-repository consolidating all FlexNetOS projects into a single, cohesive development environment. All previously separate repositories have been merged as subtrees under the `/repos` directory, enabling streamlined development, continuous integration, and unified deployment workflows.

## 📁 Repository Structure

```
noa_ark_os/
├── repos/
│   ├── ark-os-noa/          # AgenticAI with Hive Mind running AI Agent Swarm via MicroAgentStack
│   ├── ARK-OS/              # Core OS architecture, system maps, and task graphs
│   ├── agentaskit/          # Task execution and code migration using multi-agent framework
│   ├── deflexnet-app/       # Python-based digest pipeline application
│   ├── deflex-ai-os/        # Rust-based AgenticAI File Management and Operations
│   └── MicroAgentStack/     # Disposable microagent orchestration and management
├── docs/                    # Unified documentation
├── scripts/                 # Cross-repo automation scripts
└── .github/                 # CI/CD workflows and actions
```

## 🔧 Consolidated Components

### 1. **ark-os-noa**
- **Language**: Python, HTML
- **Purpose**: Local-first Agentic OS with Hive Mind architecture
- **Key Features**: 
  - Agent pack distribution
  - Docker compose orchestration
  - Pipeline execution
  - Documentation and setup guides

### 2. **ARK-OS**
- **Language**: Mermaid, Python, JSON
- **Purpose**: System architecture and autonomous system mapping
- **Key Features**:
  - Autonomous system maps
  - Task execution graphs
  - Universal task execution policy
  - Conversation data and planning artifacts

### 3. **agentaskit**
- **Language**: Rust
- **Purpose**: Multi-agent task execution framework
- **Key Features**:
  - Code migration tools
  - System unification utilities
  - Production-ready agent toolkit
  - Comprehensive documentation

### 4. **deflexnet-app**
- **Language**: Python
- **Purpose**: Digest pipeline processing
- **Key Features**:
  - Pipeline orchestration
  - Test suite
  - Modular digest processing

### 5. **deflex-ai-os**
- **Language**: Rust
- **Purpose**: AgenticAI file management and operations
- **Key Features**:
  - Cargo-based Rust project
  - Service architecture
  - Docker compose setup
  - CI/CD workflows

### 6. **MicroAgentStack**
- **Language**: Python
- **Purpose**: Disposable microagent stack for distributed AI operations
- **Key Features**:
  - Agent manifest management
  - Dynamic agent generation
  - Docker orchestration
  - OpenAPI specification generation
  - Full automation scripts

## 🚀 Getting Started

### Prerequisites
- Git (for repository management)
- Docker & Docker Compose (for containerized services)
- Python 3.8+ (for Python-based components)
- Rust & Cargo (for Rust-based components)
- Node.js (if needed for any frontend components)

### Quick Start

1. **Clone the repository**
   ```bash
   git clone https://github.com/FlexNetOS/noa_ark_os.git
   cd noa_ark_os
   ```

2. **Explore individual components**
   ```bash
   # Navigate to any component
   cd repos/MicroAgentStack
   
   # Follow component-specific README
   cat README.md
   ```

3. **Run unified services**
   ```bash
   # Use the main orchestration script (to be created)
   ./scripts/start-all-services.sh
   ```

## 🔄 Development Workflow

### CRC/CI/CD - Continue ReCode / Continuous Integration / Continuous Development

This unified repository implements a full auto AI operation model:

1. **Continue ReCode (CRC)**: Automated code refinement and optimization
2. **Continuous Integration (CI)**: Automated testing and validation across all components
3. **Continuous Development (CD)**: Automated deployment and system evolution

### Working with Subtrees

Each component under `/repos` is maintained as a git subtree, allowing:

- **Independent updates**: Update individual components from their source
- **Unified commits**: Make cross-component changes in single commits
- **Preserved history**: Maintain full git history from original repos

#### Update a component from upstream:
```bash
git subtree pull --prefix=repos/MicroAgentStack MicroAgentStack main --squash
```

#### Push changes back to component:
```bash
git subtree push --prefix=repos/MicroAgentStack MicroAgentStack main
```

## 🏗️ Architecture

The unified system follows a microservices architecture with:

- **Agent Orchestration**: Centralized through MicroAgentStack
- **Service Mesh**: Docker Compose based service discovery
- **Event-Driven**: Asynchronous agent communication
- **Scalable**: Horizontal scaling through disposable agents
- **Resilient**: Self-healing through agent lifecycle management

## 📚 Documentation

- [Architecture Overview](./docs/ARCHITECTURE.md)
- [Development Guide](./docs/DEVELOPMENT.md)
- [API Reference](./docs/API.md)
- [Deployment Guide](./docs/DEPLOYMENT.md)

## 🤝 Contributing

Each component maintains its own contribution guidelines. See individual component READMEs for details:

- [agentaskit/CONTRIBUTING.md](./repos/agentaskit/CONTRIBUTING.md)
- [MicroAgentStack/SECURITY.md](./repos/MicroAgentStack/SECURITY.md)

## 📄 License

Components maintain their individual licenses. See LICENSE files in respective directories.

## 🔗 Links

- **Organization**: [FlexNetOS on GitHub](https://github.com/FlexNetOS)
- **Individual Repositories** (now consolidated):
  - [ark-os-noa](https://github.com/FlexNetOS/ark-os-noa)
  - [ARK-OS](https://github.com/FlexNetOS/ARK-OS)
  - [agentaskit](https://github.com/FlexNetOS/agentaskit)
  - [deflexnet-app](https://github.com/FlexNetOS/deflexnet-app)
  - [deflex-ai-os](https://github.com/FlexNetOS/deflex-ai-os)
  - [MicroAgentStack](https://github.com/FlexNetOS/MicroAgentStack)

## 🎯 Roadmap

- [x] Consolidate all FlexNetOS repositories into mono-repo
- [ ] Create unified CI/CD pipeline
- [ ] Implement cross-component integration tests
- [ ] Develop unified API gateway
- [ ] Create comprehensive documentation site
- [ ] Implement automated deployment orchestration
- [ ] Build unified monitoring and observability stack

---

**Status**: Active Development | **Version**: 1.0.0-unified | **Last Updated**: 2025-10-08
