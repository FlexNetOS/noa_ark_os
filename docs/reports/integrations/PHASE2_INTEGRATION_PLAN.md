# 🔄 MANUAL INTEGRATION PLAN - Phase 2

## 📊 Drop Structure Analysis

### agentaskit Drop Contents

**Total:** 2,299 files, ~44 MB
**Language:** Primarily Rust with Python orchestration
**Quality:** 87% AI confidence (Model B - Bug Fixes track)

#### Key Directories Found:
```
agentaskit-production/
├── core/
│   └── src/
│       ├── agents/           ← Agent implementations
│       │   ├── board/        ← Board-level agents
│       │   ├── executive/    ← Executive agents
│       │   └── specialized/  ← Specialist agents
│       ├── orchestration/    ← Orchestration system
│       ├── workflows/        ← Workflow management
│       └── ui/              ← UI components
├── archive/                  ← Legacy versions
└── unified_agents/          ← Unified agent system
```

---

## 🎯 Integration Strategy

### Phase 2A: Core Agent Implementations
**Goal:** Integrate production-ready agent code

**Steps:**
1. ✅ Map agent implementations to registry entries
2. ✅ Create `agents/src/implementations/` module structure
3. ✅ Copy and adapt core agent files
4. ✅ Link to registry system

### Phase 2B: Orchestration System
**Goal:** Add orchestration capabilities

**Steps:**
1. ✅ Extract orchestration engine
2. ✅ Integrate with existing `agents/` crate
3. ✅ Connect to CRC system

### Phase 2C: Workflow Integration
**Goal:** Add workflow management

**Steps:**
1. ✅ Extract workflow components
2. ✅ Integrate with `workflow/` crate
3. ✅ Connect to agent system

---

## 📋 Manual Integration Checklist

### Step 1: Explore and Catalog ✅
- [x] Scan agentaskit structure
- [x] Identify key components
- [x] Map to existing workspace

### Step 2: Create Implementation Structure
- [ ] Create `agents/src/implementations/` directory
- [ ] Create subdirectories: `board/`, `executive/`, `specialist/`, `micro/`
- [ ] Add module declarations

### Step 3: Extract Core Agents (High Priority)
**Board Agents:**
- [ ] `digest_agent.rs`
- [ ] `finance_board_agent.rs`
- [ ] Others from board directory

**Executive Agents:**
- [ ] `emergency_responder.rs`
- [ ] Others from executive directory

**Specialized Agents:**
- [ ] `code_generation_agent.rs`
- [ ] `data_analytics_agent.rs`
- [ ] `deployment_agent.rs`
- [ ] Others from specialized directory

### Step 4: Adapt and Integrate
For each agent file:
1. [ ] Copy to appropriate `implementations/` subdirectory
2. [ ] Update imports to use NOA workspace types
3. [ ] Connect to registry
4. [ ] Add to module tree
5. [ ] Test compilation

### Step 5: Orchestration Integration
- [ ] Extract `orchestration/` components
- [ ] Integrate `agent.rs`, `engine.rs`, `executor.rs`
- [ ] Connect to existing agent factory
- [ ] Test basic orchestration

### Step 6: Workflow Integration
- [ ] Extract workflow components
- [ ] Integrate with `workflow/` crate
- [ ] Connect to CRC for drop processing

### Step 7: Testing & Validation
- [ ] Compile all integrated code
- [ ] Run unit tests
- [ ] Test agent spawning
- [ ] Validate registry connections

---

## 🚀 Quick Start Commands

### Scan Drop Contents
```powershell
# List agent implementations
Get-ChildItem -Path "crc\drop-in\incoming\stale\agentaskit\agentaskit-production\core\src\agents" -Recurse -Filter "*.rs" | Select-Object FullName

# Count by type
Get-ChildItem -Path "crc\drop-in\incoming\stale\agentaskit\agentaskit-production\core\src\agents" -Recurse -Filter "*.rs" | Group-Object Directory | Select-Object Count, Name
```

### Create Integration Structure
```powershell
# Create implementation directories
New-Item -ItemType Directory -Force -Path "agents\src\implementations\board"
New-Item -ItemType Directory -Force -Path "agents\src\implementations\executive"
New-Item -ItemType Directory -Force -Path "agents\src\implementations\specialist"
New-Item -ItemType Directory -Force -Path "agents\src\implementations\micro"
```

---

## 📝 Integration Template

### For Each Agent File

**1. Copy to workspace:**
```powershell
Copy-Item `
  "crc\drop-in\incoming\stale\agentaskit\...\agent_name.rs" `
  "agents\src\implementations\category\agent_name.rs"
```

**2. Adapt imports:**
```rust
// OLD (from drop)
use crate::types::*;
use crate::core::*;

// NEW (NOA workspace)
use noa_agents::{AgentMetadata, Result};
use noa_core::prelude::*;
```

**3. Add to module tree:**
```rust
// agents/src/implementations/mod.rs
pub mod board;
pub mod executive;
pub mod specialist;
pub mod micro;
```

**4. Register in factory:**
```rust
// Link implementation to registry entry
impl AgentFactory {
    pub fn spawn_from_registry(&self, agent_id: &str) -> Result<AgentId> {
        let metadata = registry.get(agent_id)?;
        // Spawn appropriate implementation
    }
}
```

---

## 🎯 Priority Agents to Integrate First

### High Priority (Core Functionality)
1. **ExecutiveCommanderChiefAgent** - Top-level orchestrator
2. **CodeGenAgent** - Code generation
3. **DigestAgent** - Information processing
4. **BackupRestoreAgent** - Data management
5. **MonitoringAlertingAgent** - System health

### Medium Priority (Board Level)
6. **FinanceBoardAgent**
7. **TechnologyBoardAgent**
8. **SecurityBoardAgent**
9. **OperationsBoardAgent**

### Lower Priority (Specialist)
10. **DataAnalyticsAgent**
11. **DeploymentAgent**
12. **TestCaseGeneratorAgent**

---

## 📈 Progress Tracking

### Agents Integrated: 0 / 928

**By Layer:**
- Board: 0 / ~15
- Executive: 0 / ~5
- StackChief: 0 / ~20
- Specialist: 0 / ~100
- Micro: 0 / ~788

**Status:** Ready to begin manual integration

---

## 🔄 Automation Goal

As we integrate manually, we'll identify patterns and create automation:

### Future Automation
1. **Drop Scanner** - Auto-detect agent files
2. **Import Adapter** - Auto-fix imports
3. **Registry Linker** - Auto-connect to metadata
4. **Test Generator** - Auto-create tests
5. **Full Pipeline** - Eventual zero-touch integration

---

## 💡 Next Action

**Ready to start integration!**

Choose one:
1. **Quick Start** - Integrate 5 priority agents now
2. **Full Scan** - Detailed analysis of all agent files
3. **Custom** - Tell me which agent to integrate first

---

**Last Updated:** 2024-01-15
**Status:** ⏸️ Ready for Phase 2A
**Next:** Manual integration of first agent
