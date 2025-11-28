# NOA ARK OS – CLI-First Roadmap Update

**Date:** 2025-11-14  
**Version:** 1.1.0  
**Update Type:** Major Enhancement  
**Status:** ✅ Complete

---

## Executive Summary

Transformed NOA ARK OS roadmap to prioritize **CLI-first architecture**, making the `noa` CLI the **primary interface** for both humans and AI agents. IDE extensions are now deprecated in favor of universal terminal access with machine-parsable output.

### Key Changes

1. **New Goal 18:** CLI-First Interface
2. **New Phase 0.5:** CLI-First Foundation (Critical Path)
3. **Expanded Phase 4:** CLI Expansion with 40+ commands
4. **IDE Deprecation:** Extensions become thin wrappers around CLI
5. **AI Optimization:** Token-efficient, batch operations, persistent sessions

---

## What Changed

### 1. Added Goal 18: CLI-First Interface

**Description:** All capabilities accessible via unified CLI for both humans and AI agents; IDE extensions deprecated.

**Impact:**
* CLI becomes first-class citizen
* Consistent interface across all platforms
* AI agents get direct, efficient access
* No more editor lock-in

### 2. New Phase 0.5: CLI-First Foundation ⭐ CRITICAL PATH

**Objective:** Unified CLI as primary interface for humans and AI agents.

**Priority:** Critical (blocks Phase 4 and influences all other phases)

**Core Subcommands (10+):**
* `noa kernel` – Kernel operations
* `noa world` – World model operations
* `noa registry` – Registry operations
* `noa trust` – Trust operations
* `noa snapshot` – Snapshot operations
* `noa agent` – Agent operations
* `noa policy` – Policy operations
* `noa sbom` – SBOM operations
* `noa pipeline` – Pipeline operations
* `noa profile` – Profile operations

**Output Modes:**
* `--json` – Machine-parsable (default for agents)
* `--yaml` – Human-readable
* `--tui` – Interactive terminal UI
* `--watch` – Real-time updates
* `--format=table|tree|json|yaml` – Flexible formatting

**AI Agent Integration:**
* `noa agent invoke <tool> --args <json>` – Direct tool invocation
* `noa query <natural-language>` – NL to CLI translation
* `noa plan <task>` – Generate execution plan
* `noa exec <plan-id>` – Execute with approval gates

**IDE Deprecation:**
* Document migration path
* `noa ide migrate` command
* Mark extensions as "legacy"
* Focus development on CLI
* Extensions become thin wrappers

### 3. Expanded Phase 4: CLI Expansion & Registry Completion

**Objective:** Complete CLI suite with full registry integration and advanced capabilities.

**Priority:** High (builds on Phase 0.5)

**New Command Categories (40+ total):**

**Observability:**
* `noa observe metrics` – Live system metrics
* `noa observe traces` – Distributed tracing
* `noa observe logs --follow` – Live log streaming
* `noa observe health` – Health check dashboard

**Automation:**
* `noa auto schedule <task> <cron>` – Schedule tasks
* `noa auto trigger <event> <action>` – Event-driven
* `noa auto repair --enable` – Enable auto-repair
* `noa auto budget set <limits>` – Set resource budgets

**Collaboration:**
* `noa share snapshot <id>` – Share with team
* `noa import snapshot <url>` – Import shared
* `noa review plan <id>` – Review execution plan
* `noa approve plan <id>` – Approve for execution

**Analysis:**
* `noa analyze performance` – Performance analysis
* `noa analyze security` – Security audit
* `noa analyze dependencies` – Dependency analysis
* `noa analyze drift` – Configuration drift

**Development:**
* `noa dev init` – Initialize workspace
* `noa dev test --watch` – Continuous testing
* `noa dev build --watch` – Continuous build
* `noa dev deploy --dry-run` – Preview deployment
* `noa dev debug <agent>` – Attach debugger

**Query Languages:**
* `noa query --sql` – SQL-like queries
* `noa query --jsonpath` – JSONPath queries
* `noa query --graphql` – GraphQL endpoint

**Plugin System:**
* `noa plugin install <name>` – Install plugins
* `noa plugin develop` – Dev workflow
* Plugin SDK (Rust/Python/Go)
* Sandboxed execution

**AI Agent Optimization:**
* `noa agent context --minimal` – Minimal context (80% token savings)
* `noa agent batch <commands.json>` – Batch operations
* `noa agent session start` – Persistent sessions
* `noa agent cache warm` – Pre-cache queries

**Registry Alignment (2025-03 update):**

- Canonical metadata now lives in `registry/tools.registry.json`, enabling
  automation, CLI, and plugin experiences to stay synchronized.
- Observability, automation, analysis, collaboration, and plugin commands read
  directly from the registry to emit machine-readable JSON that mirrors the
  OpenAPI (`docs/api/noa-tools.openapi.yaml`) and gRPC (`server/protos/noa_tools.proto`)
  definitions.
- Plugin SDK scaffolding (`plugins/sdk/`) exposes helper utilities so extension
  authors can adopt the registry schema without duplicating logic.

### 4. CLI-First Philosophy Section

Added comprehensive philosophy explaining:
* Why CLI-first matters
* Interface hierarchy (CLI → API → IDE)
* Design principles
* Universal access benefits

### 5. CLI Command Reference

Added complete reference with 50+ commands:
* Core operations (Phase 0.5)
* Advanced operations (Phase 4)
* Output modes and flags
* AI agent interface examples

---

## Value Increases

### Before (IDE-Centric)

**Problems:**
* ❌ Developer locked into specific editors
* ❌ AI agents can't easily access tools
* ❌ Inconsistent interfaces across platforms
* ❌ Limited scripting/automation capabilities
* ❌ Requires IDE restart for changes
* ❌ High barrier to entry (install IDE + extensions)
* ❌ Poor CI/CD integration

### After (CLI-First)

**Benefits:**
* ✅ Universal access from any terminal
* ✅ AI agents have first-class CLI access
* ✅ Consistent interface everywhere
* ✅ Full automation via scripts
* ✅ Faster iteration (no IDE restart)
* ✅ Lower barrier to entry (just terminal)
* ✅ Better CI/CD integration
* ✅ 50+ commands covering all operations
* ✅ Token-efficient for AI agents (--minimal saves 80%)
* ✅ Batch operations reduce round-trips
* ✅ Persistent sessions improve performance
* ✅ Query optimization for faster results

### Specific Improvements

**For Humans:**
* Work in any terminal (SSH, local, cloud shell)
* Switch between JSON (scripting) and TUI (interactive)
* Real-time updates with `--watch`
* Self-documenting with `--help` and `--explain`

**For AI Agents:**
* Structured JSON output by default
* Minimal context mode saves tokens
* Batch commands reduce API calls
* Direct tool invocation
* Natural language translation (`noa query`)
* Plan generation and execution

**For DevOps:**
* Easy CI/CD integration
* Scriptable workflows
* Consistent across environments
* Full offline capability

---

## Updated Roadmap Structure

### Phase Sequence

```
Phase 0   - Repo Foundation & Dead Code Quarantine
Phase 0.5 - CLI-First Foundation ⭐ CRITICAL PATH (NEW)
Phase 1   - Kernel-First Baseline
Phase 2   - North Star Implementation
Phase 3   - World Model & Contract Tests
Phase 4   - CLI Expansion & Registry Completion ⭐ HIGH PRIORITY (EXPANDED)
Phase 5   - Gateway Slimming & Capability Tokens
Phase 6   - Performance & Retrieval Discipline
Phase 7   - Reward System (System-Health First)
Phase 8   - SBOM Split & Supply-Chain Integrity
Phase 9   - Deployment Profiles (Drop-In Everywhere)
Phase 10  - Tests, CI/CD, and Audit Trail
Phase 11  - Documentation & Agent Policy
Phase 12  - Intelligent Growth & Self-Maintenance
Phase 13  - Humanless Autonomy
```

### Updated Dependencies

```
P0 (Foundation)
  │
  ├─→ P0.5 (CLI-First) ⭐ CRITICAL PATH
  │     │
  │     ├─→ P1 (Kernel) → P2 (North Star) → P7 (Rewards) → P12 → P13
  │     │                ├─→ P3 (World Model) → P4 (CLI Expansion) → P6
  │     │                └─→ P8 (SBOM)
  │     │
  │     └─→ P5 (Gateway) → P9 (Profiles)
  │
  └─→ P10 (CI/CD) → P11 (Docs)
```

**Key:** P0.5 blocks P4 and influences all phases

### Progress Update

| Phase | Status | Priority |
|-------|--------|----------|
| P0.5 | 📋 Planned | **CRITICAL** |
| P4 | 📋 Planned | **HIGH** |
| P1 | 🔨 In Progress (40%) | Medium |
| P10 | 🔨 In Progress (30%) | Medium |
| P11 | 🔨 In Progress (60%) | Medium |
| Others | 📋 Planned | Normal |

**Current Focus:** P0.5 (CLI-First), P1 (Kernel), P10 (CI/CD)

---

## Files Modified

### 1. ROADMAP_AGENTIC_KERNEL.md

**Before:** 603 lines  
**After:** 991 lines  
**Added:** +388 lines

**Changes:**
* Added Goal 18 (CLI-First Interface)
* Added CLI-First Philosophy section
* Added Phase 0.5 (CLI-First Foundation)
* Expanded Phase 4 (CLI Expansion)
* Added CLI Command Reference (50+ commands)
* Updated dependencies and progress tracking
* Updated version to 1.1.0

### 2. roadmap_noa_ark_os.json

**Before:** 598 lines  
**After:** 665 lines  
**Added:** +67 lines

**Changes:**
* Added Goal 18 object
* Added Phase 0.5 object with 7 tasks
* Expanded Phase 4 with 7 detailed tasks
* Added priority markers ("critical", "high")
* Updated progress tracking
* Updated current_focus array
* Updated version to 1.1.0
* Added changelog field

**Validation:** ✅ Valid JSON syntax

---

## Implementation Guidance

### Phase 0.5 Implementation Priority

**Week 1-2: Core CLI Binary**
1. Set up Rust CLI project with clap/structopt
2. Implement subcommand architecture
3. Add JSON/YAML output support
4. Create help system

**Week 3-4: Core Subcommands**
1. Implement `noa kernel` operations
2. Implement `noa world` operations
3. Implement `noa registry` operations
4. Add basic error handling

**Week 5-6: Advanced Features**
1. Add TUI mode with ratatui/crossterm
2. Implement `--watch` streaming
3. Add shell completion generation
4. Implement AI agent interface

**Week 7-8: Polish & Documentation**
1. Write comprehensive help text
2. Create examples and tutorials
3. Test on Linux/macOS/Windows
4. Document migration from IDE extensions

### Phase 4 Implementation Priority

**Follow Phase 0.5 completion**

**Tier 1 (Critical):**
* Observability commands (metrics, traces, logs)
* Analysis commands (performance, security, drift)
* Dev workflow commands (test, build, deploy)

**Tier 2 (High):**
* Automation commands (schedule, trigger, repair)
* Query languages (SQL, JSONPath, GraphQL)
* AI agent optimization (minimal, batch, sessions)

**Tier 3 (Nice-to-have):**
* Collaboration commands (share, import, review)
* Plugin system
* Data export/import

---

## Migration Guide: IDE Extensions → CLI

### For Users

**Step 1: Install CLI**
```bash
# Install noa CLI
curl -sSL https://get.noa-ark-os.dev | sh

# Verify installation
noa --version
```

**Step 2: Learn Core Commands**
```bash
# Get help
noa --help
noa <subcommand> --help

# Try interactive mode
noa world verify --tui
```

**Step 3: Migrate Workflows**
```bash
# Old: Click button in VS Code extension
# New: Run command
noa kernel start

# Old: View in IDE panel
# New: Stream in terminal
noa agent spawn worker --watch
```

**Step 4: Set Up Aliases (Optional)**
```bash
# Add to .bashrc/.zshrc
alias nk='noa kernel'
alias nw='noa world'
alias na='noa agent'
```

### For Extension Developers

**Step 1: Wrap CLI Commands**
```typescript
// Old: Direct implementation
async function startKernel() {
  // ... complex logic ...
}

// New: Wrap CLI
async function startKernel() {
  const result = await exec('noa kernel start --json');
  return JSON.parse(result.stdout);
}
```

**Step 2: Remove Duplicate Features**
* Delete custom UI for operations
* Delete duplicate logic
* Keep only thin wrappers

**Step 3: Document CLI Usage**
* Update extension README
* Point users to CLI docs
* Mark extension as "legacy wrapper"

---

## Success Metrics

### Phase 0.5 Success Criteria

- [ ] `noa` CLI runs on Linux, macOS, Windows
- [ ] 10+ subcommands fully implemented
- [ ] JSON output parsable by jq
- [ ] TUI mode works in any terminal
- [ ] Shell completions install successfully
- [ ] AI agents can invoke all commands
- [ ] Migration guide published
- [ ] Zero IDE-exclusive features

### Phase 4 Success Criteria

- [ ] 40+ commands implemented
- [ ] Plugin system working
- [ ] Query languages functional
- [ ] AI agents achieve <50ms latency
- [ ] 100% feature parity with deprecated IDE extensions
- [ ] Token savings >80% with --minimal

### Overall Value Metrics

**Developer Experience:**
* Time to first operation: <5 minutes (vs 30 min IDE setup)
* Context switch time: <1 second (vs IDE restart)
* Learning curve: 1 day (vs 1 week for IDE + extensions)

**AI Agent Efficiency:**
* Token usage: -80% with --minimal flag
* Latency: <50ms average
* Batch efficiency: 10x fewer round-trips

**System Quality:**
* Test coverage: >80%
* Cross-platform support: 100%
* Offline capability: 100%

---

## Next Steps

### Immediate (This Week)
1. ✅ Update roadmap documentation (DONE)
2. ✅ Update JSON specification (DONE)
3. ✅ Document CLI-first philosophy (DONE)
4. Create Phase 0.5 implementation ticket
5. Set up Rust CLI project structure

### Short Term (This Month)
1. Implement core `noa` binary
2. Add first 5 subcommands (kernel, world, registry, trust, snapshot)
3. JSON/YAML output support
4. Basic TUI mode

### Medium Term (Next Quarter)
1. Complete all 10 Phase 0.5 subcommands
2. Shell completion support
3. AI agent integration
4. Migration guide and examples

### Long Term (Next Year)
1. Complete Phase 4 expansion (40+ commands)
2. Plugin system
3. Query languages
4. Full IDE extension deprecation

---

## Related Documentation

* **[ROADMAP_AGENTIC_KERNEL.md](docs/projects/ROADMAP_AGENTIC_KERNEL.md)** – Full roadmap (v1.1.0)
* **[roadmap_noa_ark_os.json](docs/projects/roadmap_noa_ark_os.json)** – Machine spec (v1.1.0)
* **[ROADMAP_UPGRADE_SUMMARY.md](docs/ROADMAP_UPGRADE_SUMMARY.md)** – Original upgrade summary
* **[Makefile](Makefile)** – Build targets

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2025-11-14 | 1.1.0 | Added Goal 18, Phase 0.5, expanded Phase 4, CLI-first transformation |
| 2025-11-14 | 1.0.0 | Initial comprehensive agentic kernel roadmap |

---

**Status:** ✅ CLI-First Transformation Complete  
**Next:** Begin Phase 0.5 Implementation  
**Version:** 1.1.0  
**Maintained By:** NOA ARK OS Core Team

---

*"CLI-first is AI-first. Universal terminal access is universal agent access."*
