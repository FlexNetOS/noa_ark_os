# AgentAsKit Integration Plan

**Source**: `crc/drop-in/incoming/stale/agentaskit`  
**Drop ID**: drop-92a147c5  
**Confidence**: 87%  
**Target Sandbox**: Model B (Bug Fixes)  
**Status**: Awaiting Manual Integration

---

## 📊 Overview

### Source Analysis
- **Repository**: https://github.com/FlexNetOS/agentaskit.git
- **Branch**: main
- **Language**: Rust (Multi-crate workspace)
- **Total Files**: 2,299
- **Total Size**: 46.2 MB
- **Structure**: Production-ready multi-agent system

### Key Components
```
agentaskit-production/
├── core/                    - Core agent functionality
├── flexnetos/
│   └── execution/
│       ├── core/           - Execution engine
│       └── wasm_host/      - WASM runtime host
├── shared/                 - Shared types and utilities
├── tests/                  - Integration tests
├── tools/                  - Development tools
├── configs/                - Configuration files
└── docs/                   - Comprehensive documentation
```

---

## 🎯 Integration Strategy

### Phase Approach (4 Phases)

Given the size and complexity, we'll integrate in phases rather than all at once.

#### **Phase 1: Core Types & Shared Utilities** ⭐ START HERE
**Priority**: Critical  
**Effort**: 2-4 hours  
**Risk**: Low

**What to Integrate**:
- `shared/src/types.rs` → `agents/src/agentaskit/types.rs`
- `shared/src/protocols.rs` → `agents/src/agentaskit/protocols.rs`
- `shared/src/errors.rs` → Merge with `agents/src/error.rs`

**Why**: Foundation for everything else, minimal dependencies

**Adaptations Needed**:
- Rename `agentaskit_shared::` → `noa_agents::agentaskit::`
- Update imports to use NOA workspace conventions
- Add `noa_` prefix to conflicting type names

**Files to Extract** (~20 files):
```rust
// Priority files from shared/src/
- types.rs          // Core types
- protocols.rs      // Communication protocols
- task.rs          // Task definitions
- agent.rs         // Agent traits
- errors.rs        // Error types
- utils.rs         // Utilities
```

#### **Phase 2: Execution Core** 
**Priority**: High  
**Effort**: 4-8 hours  
**Risk**: Medium

**What to Integrate**:
- `flexnetos/execution/core/` → `agents/src/execution/`
- Task execution engine
- State management
- Resource management

**Why**: Enables actual agent execution capabilities

**Adaptations Needed**:
- Replace external async runtime with NOA's tokio setup
- Integrate with NOA's resource management
- Update security model to match NOA standards

**Dependencies**:
- Phase 1 must be complete
- Requires NOA execution environment setup

#### **Phase 3: WASM Runtime** (OPTIONAL)
**Priority**: Medium  
**Effort**: 8-16 hours  
**Risk**: High

**What to Integrate**:
- `flexnetos/execution/wasm_host/` → `agents/src/wasm/`
- WASM module loading
- WASM execution sandbox
- WASI integration

**Why**: Enables sandboxed agent execution

**Adaptations Needed**:
- Wasmtime integration (13.0)
- Security policies alignment
- Resource quota enforcement

**Alternative**: May skip if NOA doesn't need WASM agents initially

#### **Phase 4: Advanced Features** (OPTIONAL)
**Priority**: Low  
**Effort**: 16+ hours  
**Risk**: High

**What to Integrate**:
- ML inference components (candle, burn)
- Vector storage (qdrant-client)
- Desktop UI (tauri) - likely SKIP
- Advanced orchestration

**Why**: Power user features, not essential for initial integration

**Recommendation**: Defer to Phase 2 of overall NOA ARK OS roadmap

---

## 📋 Detailed Phase 1 Plan

### Step 1: Prepare Destination Structure

```bash
# Create directory structure
mkdir -p agents/src/agentaskit
mkdir -p agents/src/agentaskit/protocols
mkdir -p agents/src/agentaskit/execution
```

### Step 2: Extract Core Files

**Files to Copy** (from `agentaskit-production/shared/src/`):

1. **types.rs** (~500 lines)
   - Core type definitions
   - Agent metadata
   - Task structures
   - Status enums

2. **protocols.rs** (~300 lines)
   - Communication protocols
   - Message formats
   - Agent coordination

3. **task.rs** (~400 lines)
   - Task definitions
   - Task lifecycle
   - Task orchestration

4. **agent.rs** (~300 lines)
   - Agent traits
   - Agent capabilities
   - Agent lifecycle

5. **errors.rs** (~200 lines)
   - Error types
   - Result types
   - Error handling

### Step 3: Adaptation Rules

**Find & Replace Patterns**:
```yaml
patterns:
  - from: "agentaskit_shared::"
    to: "noa_agents::agentaskit::"
  
  - from: "agentaskit_production::"
    to: "noa_agents::"
  
  - from: "pub struct Agent"
    to: "pub struct AgentAskitAgent"  # Avoid name collision
  
  - from: "pub struct Task"
    to: "pub struct AgentAskitTask"   # Avoid name collision
```

**Dependency Adaptations**:
```toml
# Keep these (already in NOA workspace):
tokio = "1.47"
serde = "1.0"
serde_json = "1.0"
uuid = "1.18"
anyhow = "1.0"
thiserror = "1.0"

# Remove these (external):
reqwest = "..."      # Use noa_http instead
tonic = "..."        # Defer to Phase 2
prost = "..."        # Defer to Phase 2
```

### Step 4: Integration Testing

**Create Test File**: `agents/tests/agentaskit_integration_test.rs`

```rust
// Test basic AgentAsKit types work
#[test]
fn test_agentaskit_types_available() {
    use noa_agents::agentaskit::types::*;
    // Verify types compile and work
}
```

### Step 5: Update Exports

**agents/src/lib.rs**:
```rust
// Add AgentAsKit module
pub mod agentaskit {
    pub mod types;
    pub mod protocols;
    pub mod task;
    pub mod agent;
}

// Re-export commonly used items
pub use agentaskit::types::*;
```

---

## 🚫 What NOT to Integrate (Yet)

### Skip These for Phase 1:

1. **Desktop UI (Tauri)**
   - Reason: NOA has its own UI system
   - Size: Large dependency tree
   - Recommendation: Skip entirely or integrate in Phase 4

2. **ML Inference (candle, burn)**
   - Reason: Heavy dependencies, separate concern
   - Recommendation: Integrate when NOA adds ML capabilities

3. **Vector DB (qdrant-client)**
   - Reason: External service dependency
   - Recommendation: Replace with NOA's storage layer

4. **gRPC Stack (tonic, prost)**
   - Reason: Complex, network-dependent
   - Recommendation: Phase 2 or use NOA's IPC instead

5. **Build Scripts and Tools**
   - Reason: AgentAsKit-specific
   - Recommendation: Adapt to NOA's build system

6. **Documentation Files**
   - Reason: Reference only
   - Recommendation: Extract useful info, don't copy verbatim

---

## 🔧 Dependencies Analysis

### Already in NOA Workspace ✅
```toml
tokio = "1.47"           # Async runtime
serde = "1.0"            # Serialization
serde_json = "1.0"       # JSON
uuid = "1.18"            # UUIDs
anyhow = "1.0"           # Error handling
thiserror = "1.0"        # Error macros
chrono = "0.4"           # Date/time
tracing = "0.1"          # Logging
```

### Need to Add to NOA ⚠️
```toml
# Phase 1 (optional):
async-trait = "0.1"      # Async traits
futures = "0.3"          # Futures utilities

# Phase 2 (execution):
parking_lot = "0.12"     # Better locks
crossbeam = "0.8"        # Concurrency
dashmap = "5.0"          # Concurrent HashMap

# Phase 3 (WASM):
wasmtime = "13.0"        # WASM runtime
wasi-common = "13.0"     # WASI support
```

### Should NOT Add (External) ❌
```toml
reqwest = "..."          # HTTP client - use noa_http
tonic = "..."            # gRPC - use noa_ipc
tauri = "..."            # Desktop UI - use noa_ui
qdrant-client = "..."    # Vector DB - use noa_storage
```

---

## 📝 Manual Steps Required

### Before Integration

1. **Review agentaskit documentation**
   - Read `PRODUCTION_READINESS_ANALYSIS.md`
   - Read `SEVEN_PHASE_IMPLEMENTATION_COMPLETE.md`
   - Understand the architecture

2. **Identify must-have features**
   - What does NOA actually need from AgentAsKit?
   - What can we build ourselves better?
   - What's just nice-to-have?

3. **Check for conflicts**
   - Name collisions with existing NOA types
   - Architectural mismatches
   - Design philosophy differences

### During Integration

1. **Copy files one at a time**
   - Don't copy entire directories
   - Test compilation after each file
   - Fix errors immediately

2. **Adapt as you go**
   - Rename conflicting types
   - Update imports
   - Replace external dependencies

3. **Document decisions**
   - Why did you skip X?
   - Why did you adapt Y this way?
   - What needs future work?

### After Integration

1. **Write integration tests**
   - Verify types work
   - Test core functionality
   - Ensure no regressions

2. **Update documentation**
   - Add AgentAsKit section to agents/README.md
   - Document new types and traits
   - Add usage examples

3. **Archive original**
   - Compress agentaskit-production to crc/archive/stale/
   - Remove from incoming/stale/
   - Keep manifest.json for reference

---

## 🎯 Success Criteria

### Phase 1 Complete When:
- [ ] Core types available in `noa_agents::agentaskit::types`
- [ ] Communication protocols in `noa_agents::agentaskit::protocols`
- [ ] Task definitions in `noa_agents::agentaskit::task`
- [ ] Agent traits in `noa_agents::agentaskit::agent`
- [ ] All tests pass: `cargo test -p noa_agents`
- [ ] Example compiles: `cargo run --example agentaskit_demo`
- [ ] Documentation updated

### Phase 2 Complete When:
- [ ] Execution engine integrated
- [ ] Agents can execute tasks
- [ ] Resource management works
- [ ] State persistence works
- [ ] Integration tests pass

### Phase 3 Complete When:
- [ ] WASM modules can be loaded
- [ ] WASM agents can execute
- [ ] Sandbox security enforced
- [ ] Performance acceptable

---

## ⚠️ Risk Assessment

### High Risk Areas

1. **Type Conflicts**
   - AgentAsKit has `Agent`, `Task` types
   - NOA already has `Agent` type
   - **Mitigation**: Rename to `AgentAskitAgent`, etc.

2. **External Dependencies**
   - AgentAsKit uses many external crates
   - Some conflict with NOA's self-contained philosophy
   - **Mitigation**: Replace with NOA equivalents or inline critical code

3. **Architectural Mismatch**
   - AgentAsKit may have different design patterns
   - Integration may require significant refactoring
   - **Mitigation**: Start small (Phase 1), evaluate, then proceed

### Medium Risk Areas

1. **WASM Runtime**
   - Complex, large dependency
   - May have performance implications
   - **Mitigation**: Make it optional, gate behind feature flag

2. **ML Inference**
   - Heavy dependencies (candle, burn)
   - May not be needed initially
   - **Mitigation**: Defer to Phase 4 or skip

### Low Risk Areas

1. **Core Types**
   - Simple structs and enums
   - Minimal dependencies
   - **Mitigation**: Start here (Phase 1)

2. **Communication Protocols**
   - Well-defined interfaces
   - Can be adapted easily
   - **Mitigation**: Include in Phase 1

---

## 💡 Recommendations

### Recommendation 1: Start with Phase 1 Only
**Rationale**: Validate integration approach with minimal risk  
**Effort**: 2-4 hours  
**Value**: Immediate access to AgentAsKit type system

### Recommendation 2: Evaluate After Phase 1
**Rationale**: Decide if Phase 2+ is worth the effort  
**Questions to Answer**:
- Do we really need AgentAsKit's execution engine?
- Can we build something simpler/better ourselves?
- Is the integration cost justified by the value?

### Recommendation 3: Consider Alternatives
**Rationale**: AgentAsKit might not be the best fit  
**Alternatives**:
- Build agent execution from scratch (simpler, tailored to NOA)
- Use only the type definitions (Phase 1 only)
- Cherry-pick specific features, not wholesale integration

### Recommendation 4: Feature Flag Everything
**Rationale**: Make AgentAsKit integration optional  
**Implementation**:
```toml
[features]
default = []
agentaskit = ["wasmtime", "async-trait"]
agentaskit-wasm = ["agentaskit", "wasmtime", "wasi-common"]
```

---

## 🚀 Next Steps

### Immediate (Today)
1. ✅ Read this integration plan
2. ✅ Review agentaskit documentation in drop-in folder
3. ✅ Decide: Is Phase 1 integration worth it?
4. ⬜ If yes: Create `agents/src/agentaskit/` structure
5. ⬜ If no: Document decision and skip

### This Week
1. ⬜ Execute Phase 1 integration (if approved)
2. ⬜ Write integration tests
3. ⬜ Update documentation
4. ⬜ Archive original agentaskit code

### This Month
1. ⬜ Evaluate Phase 1 results
2. ⬜ Decide on Phase 2 (execution engine)
3. ⬜ Plan Phase 3 (WASM) if needed

---

## 📚 Reference Files

### Key Files to Review

**Documentation**:
- `agentaskit-production/README.md` - Main overview
- `agentaskit-production/PRODUCTION_READINESS_ANALYSIS.md` - Readiness status
- `agentaskit-production/SEVEN_PHASE_IMPLEMENTATION_COMPLETE.md` - Implementation details

**Code to Study**:
- `shared/src/types.rs` - Core types
- `shared/src/protocols.rs` - Communication protocols
- `core/src/agent.rs` - Agent implementation
- `flexnetos/execution/core/src/executor.rs` - Execution engine

**Tests to Learn From**:
- `tests/workflows/integration_test.rs` - Integration patterns
- `core/tests/agent_test.rs` - Agent testing

---

## 📞 Questions to Answer

Before proceeding, answer these:

1. **Do we need AgentAsKit at all?**
   - What specific features do we need?
   - Can we build them ourselves faster/better?

2. **What's the minimum viable integration?**
   - Just types? (Phase 1)
   - Execution engine too? (Phase 2)
   - Full stack? (All phases)

3. **What's our timeline?**
   - Need it working this week? → Phase 1 only
   - Have a month? → Consider Phase 2
   - Long-term project? → All phases possible

4. **What's our maintenance commitment?**
   - Keep in sync with upstream? (No, it's stale code)
   - Fork and maintain ourselves? (Yes, if we integrate)
   - One-time extraction? (Probably yes)

---

**Integration Plan Complete**  
**Status**: Awaiting User Decision on Phase 1  
**Recommendation**: Start with Phase 1, evaluate, then decide on Phase 2+
