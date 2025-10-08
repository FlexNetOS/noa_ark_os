# Workspace Organization Plan

**Date**: Current  
**Purpose**: Clean up root directory and establish best practices  
**Status**: Ready to Execute  

---

## 🎯 **Current Problem**

### **Root Directory Chaos**
- **70+ files** in root directory
- **50+ Markdown docs** (session reports, integration docs, build status)
- **10+ PowerShell scripts** (build, integration, fix scripts)
- **Mix of**: Documentation, scripts, build files, config files

### **Issues**
- ❌ Hard to find specific documentation
- ❌ Root directory cluttered
- ❌ No clear organization
- ❌ Scripts scattered
- ❌ Session reports mixed with project docs

---

## 📂 **Proposed Structure**

### **1. Documentation Organization** (`docs/`)

```
docs/
├── README.md                     # Documentation index
├── notes/                        # Development notes
│   ├── README.md
│   ├── design-decisions.md
│   └── meeting-notes/
├── audits/                       # Security & code audits
│   ├── README.md
│   ├── security/
│   ├── performance/
│   └── dependency/
├── reports/                      # Session & integration reports
│   ├── README.md
│   ├── sessions/                 # SESSION_COMPLETE.md files
│   ├── builds/                   # BUILD_STATUS_*.md files
│   ├── integrations/             # INTEGRATION_*.md files
│   └── analysis/                 # Analysis reports
├── links/                        # External references
│   ├── README.md
│   ├── tools.md
│   ├── libraries.md
│   └── documentation.md
├── projects/                     # Project planning
│   ├── README.md
│   ├── roadmap.md
│   ├── milestones/
│   └── sprints/
├── references/                   # Technical references
│   ├── README.md
│   ├── rust/
│   ├── python/
│   ├── go/
│   └── architecture/
├── tests/                        # Test documentation
│   ├── README.md
│   ├── test-plans/
│   ├── test-results/
│   └── coverage-reports/
├── guides/                       # User & developer guides
│   ├── README.md
│   ├── getting-started.md
│   ├── contributing.md
│   └── troubleshooting.md
└── archive/                      # Old documentation
    ├── README.md
    └── by-date/
```

### **2. Scripts Organization** (`scripts/`)

```
scripts/
├── README.md                     # Scripts index
├── build/                        # Build scripts
│   ├── build.ps1
│   ├── build.sh
│   ├── build-crc.ps1
│   └── clean.ps1
├── dev/                          # Development scripts
│   ├── quickstart.ps1
│   ├── quickstart.sh
│   └── setup-env.ps1
├── integration/                  # Integration scripts
│   ├── integrate-all-agents.ps1
│   ├── fix-agent-imports.ps1
│   └── merge-old-work.ps1
├── maintenance/                  # Maintenance scripts
│   ├── cleanup.ps1
│   ├── backup.ps1
│   └── update-deps.ps1
├── crc/                          # CRC-specific scripts
│   ├── simulate-crc-flow.ps1
│   └── detect-forks.ps1          # Or keep in crc/
├── tools/                        # Tool helpers
│   ├── update-copilot-settings.ps1
│   └── configure-git.ps1
└── testing/                      # Test runners
    ├── run-tests.ps1
    └── run-benchmarks.ps1
```

### **3. Logs Organization** (`logs/`)

```
logs/
├── README.md                     # Logging documentation
├── .gitkeep                      # Keep directory in git
├── app/                          # Application logs (gitignored)
│   ├── server/
│   ├── agents/
│   └── crc/
├── build/                        # Build logs (gitignored)
│   ├── success/
│   └── errors/
├── test/                         # Test logs (gitignored)
└── system/                       # System logs (gitignored)
```

**Note**: Actual log files are `.gitignore`d, only structure is tracked.

### **4. Root Directory (Clean)**

```
noa_ark_os/
├── .github/                      # GitHub config
├── .vscode/                      # VS Code config
├── agents/                       # Source code
├── crc/                          # Source code
├── cicd/                         # Source code
├── core/                         # Source code
├── docs/                         # ✨ All documentation
├── examples/                     # Example code
├── logs/                         # ✨ Log files (gitignored)
├── notebooks/                    # Jupyter notebooks
├── sandbox/                      # Source code
├── scripts/                      # ✨ All scripts
├── server/                       # Source code
├── services/                     # Service code
├── tests/                        # Integration tests
├── tools/                        # Development tools
├── ui/                           # Source code
├── workflow/                     # Source code
├── .gitignore                    # Git config
├── Cargo.toml                    # Rust workspace
├── Cargo.lock                    # Rust dependencies
├── CONTRIBUTING.md               # How to contribute
├── LICENSE                       # License file
├── NoaArkOS.sln                  # Visual Studio solution
├── README.md                     # Main readme
└── WORKSPACE_MEMORY.md           # AI assistant memory
```

**Total Root Files**: ~15 (down from 70+!)

---

## 🔧 **Best Practices**

### **Scripts Organization**

#### **✅ DO: Use Scripts Directory**

**Why?**
1. ✅ Clear separation from source code
2. ✅ Easy to find all automation
3. ✅ Can set different permissions
4. ✅ Easier to document and maintain
5. ✅ Standard practice in large projects

**Structure**:
```
scripts/
├── README.md              # What each script does
├── build/                 # Build automation
├── dev/                   # Developer helpers
├── integration/           # Integration scripts
├── maintenance/           # Cleanup, backup
└── testing/               # Test runners
```

#### **❌ DON'T: Keep Scripts in Root**

**Problems**:
- ❌ Clutters root directory
- ❌ Mixed with source code
- ❌ Hard to find specific scripts
- ❌ No clear organization

#### **Exception: Keep in Component**

Some scripts stay with their component:
```
crc/
├── src/
├── detect-forks.ps1       # ✅ CRC-specific, stays here
└── README.md
```

**Rule**: If script is **tightly coupled** to component, keep it there.

---

### **Logging Best Practices**

#### **Where Logs Are Hosted**

**Development**:
```
logs/                      # Local development logs
├── app/
│   ├── server/            # Server logs
│   │   ├── access.log
│   │   └── error.log
│   ├── agents/            # Agent execution logs
│   └── crc/               # CRC processing logs
├── build/                 # Build logs
└── test/                  # Test logs
```

**Production** (Server):
```
server/data/logs/          # Production logs
├── app/
│   ├── server/
│   ├── agents/
│   └── crc/
├── system/                # System logs
├── security/              # Security logs
└── audit/                 # Audit logs
```

**Configuration**:
```rust
// In server config
[logging]
level = "info"
output = "file"
path = "server/data/logs"
rotation = "daily"
retention_days = 30
```

#### **Logging Strategy**

**Application Logs**:
- **Location**: `logs/app/` (dev) or `server/data/logs/` (prod)
- **Format**: JSON or structured text
- **Rotation**: Daily or by size
- **Retention**: 30 days (configurable)

**Build Logs**:
- **Location**: `logs/build/`
- **Ephemeral**: Can be deleted after successful build
- **CI/CD**: Stored by CI system (GitHub Actions, etc.)

**Test Logs**:
- **Location**: `logs/test/`
- **Ephemeral**: Can be deleted after test run
- **Coverage**: Stored separately

**System Logs**:
- **Production**: `server/data/logs/system/`
- **Includes**: OS logs, performance metrics, health checks

---

## 🚀 **Migration Plan**

### **Phase 1: Create Directory Structure** (5 min)

```powershell
cd D:\dev\workspaces\noa_ark_os

# Create documentation structure
New-Item -ItemType Directory -Force -Path docs/notes
New-Item -ItemType Directory -Force -Path docs/audits/security
New-Item -ItemType Directory -Force -Path docs/audits/performance
New-Item -ItemType Directory -Force -Path docs/audits/dependency
New-Item -ItemType Directory -Force -Path docs/reports/sessions
New-Item -ItemType Directory -Force -Path docs/reports/builds
New-Item -ItemType Directory -Force -Path docs/reports/integrations
New-Item -ItemType Directory -Force -Path docs/reports/analysis
New-Item -ItemType Directory -Force -Path docs/links
New-Item -ItemType Directory -Force -Path docs/projects/milestones
New-Item -ItemType Directory -Force -Path docs/projects/sprints
New-Item -ItemType Directory -Force -Path docs/references/rust
New-Item -ItemType Directory -Force -Path docs/references/python
New-Item -ItemType Directory -Force -Path docs/references/go
New-Item -ItemType Directory -Force -Path docs/references/architecture
New-Item -ItemType Directory -Force -Path docs/tests/test-plans
New-Item -ItemType Directory -Force -Path docs/tests/test-results
New-Item -ItemType Directory -Force -Path docs/tests/coverage-reports
New-Item -ItemType Directory -Force -Path docs/guides
New-Item -ItemType Directory -Force -Path docs/archive

# Create scripts structure
New-Item -ItemType Directory -Force -Path scripts/build
New-Item -ItemType Directory -Force -Path scripts/dev
New-Item -ItemType Directory -Force -Path scripts/integration
New-Item -ItemType Directory -Force -Path scripts/maintenance
New-Item -ItemType Directory -Force -Path scripts/tools
New-Item -ItemType Directory -Force -Path scripts/testing

# Create logs structure
New-Item -ItemType Directory -Force -Path logs/app/server
New-Item -ItemType Directory -Force -Path logs/app/agents
New-Item -ItemType Directory -Force -Path logs/app/crc
New-Item -ItemType Directory -Force -Path logs/build
New-Item -ItemType Directory -Force -Path logs/test
New-Item -ItemType Directory -Force -Path logs/system
```

### **Phase 2: Move Documentation** (10 min)

```powershell
# Move session reports
Move-Item SESSION_COMPLETE.md docs/reports/sessions/
Move-Item MISSION_ACCOMPLISHED.md docs/reports/sessions/
Move-Item GITHUB_SYNC_COMPLETE.md docs/reports/sessions/
Move-Item FINAL_SYNC_STATUS.md docs/reports/sessions/
Move-Item ALL_ISSUES_RESOLVED.md docs/reports/sessions/

# Move build reports
Move-Item BUILD_SUCCESS_STATUS.md docs/reports/builds/
Move-Item BUILD_STATUS_FINAL.md docs/reports/builds/
Move-Item BUILD_AFTER_INTEGRATION.md docs/reports/builds/
Move-Item BUILD_ERROR_FIXED.md docs/reports/builds/
Move-Item BUILD_FAILURE_ANALYSIS.md docs/reports/builds/
Move-Item BUILD_FIX_APPLIED.md docs/reports/builds/
Move-Item BUILD_FIX_STATUS.md docs/reports/builds/
Move-Item BUILD_READY.md docs/reports/builds/
Move-Item ALL_BUILD_ERRORS_FIXED.md docs/reports/builds/
Move-Item REMAINING_BUILD_FIXES.md docs/reports/builds/

# Move integration reports
Move-Item AGENT_INTEGRATION_PHASE1_COMPLETE.md docs/reports/integrations/
Move-Item AGENT_INTEGRATION_SUMMARY.md docs/reports/integrations/
Move-Item AGENT_REGISTRY_DROP_INTEGRATION_COMPLETE.md docs/reports/integrations/
Move-Item COMPREHENSIVE_INTEGRATION_PLAN.md docs/reports/integrations/
Move-Item INTEGRATION_READY.md docs/reports/integrations/
Move-Item INTEGRATION_SUCCESS_26_AGENTS.md docs/reports/integrations/
Move-Item PHASE2_INTEGRATION_PLAN.md docs/reports/integrations/
Move-Item QUICK_START_INTEGRATION.md docs/reports/integrations/

# Move analysis reports
Move-Item OLD_WORK_ANALYSIS.md docs/reports/analysis/
Move-Item MERGE_STRATEGY_GUIDE.md docs/reports/analysis/

# Move CRC reports
Move-Item CRC_AUTOMATION_COMPLETE.md docs/reports/integrations/
Move-Item CRC_CI_CD_COMPLETE.md docs/reports/integrations/
Move-Item CRC_DROP_IN_VERIFICATION_REPORT.md docs/reports/integrations/
Move-Item CRC_IMPROVEMENTS_COMPLETE.md docs/reports/integrations/
Move-Item CRC_TEST_STATUS.md docs/reports/integrations/

# Move workspace reports
Move-Item WORKSPACE_COMPLETE.md docs/reports/sessions/
Move-Item WORKSPACE_FLOW.md docs/reports/sessions/
Move-Item WORKSPACE_ORGANIZATION_COMPLETE.md docs/reports/sessions/
Move-Item WORKSPACE_PHASE3_COMPLETE.md docs/reports/sessions/
Move-Item WORKSPACE_PHASE4_COMPLETE.md docs/reports/sessions/

# Move guides to proper location
Move-Item CONTRIBUTING.md docs/guides/
Move-Item WSL_CODE_DROP_GUIDE.md docs/guides/
Move-Item MANUAL_CRC_FLOW.md docs/guides/

# Move existing docs content
Move-Item docs/ARCHITECTURE.md docs/references/architecture/
Move-Item docs/GETTING_STARTED.md docs/guides/
Move-Item docs/INTEGRATION.md docs/references/architecture/
Move-Item docs/ROADMAP.md docs/projects/

# Archive old/outdated docs
Move-Item AGENTASKIT_INTEGRATION_PLAN.md docs/archive/
Move-Item COMPLETE_SYSTEM_DEMO_CREATED.md docs/archive/
Move-Item SERVER_BUILD_SPEC_COMPLETE.md docs/archive/
Move-Item SETUP_COMPLETE.md docs/archive/
Move-Item OPTION_3_READY.md docs/archive/
Move-Item OPTION_A_COMPLETE.md docs/archive/
Move-Item OPTION3_5_CYCLES_COMPLETE.md docs/archive/
Move-Item NOA_ARK_OS_SELECTIVE_DROP_INTEGRATION_COMPLETE.md docs/archive/
Move-Item SERVER_WSL_DROP_INTEGRATION_COMPLETE.md docs/archive/
Move-Item TASK_EXEC_KIT_DROP_INTEGRATION_COMPLETE.md docs/archive/
Move-Item TOOLS_DROP_INTEGRATION_COMPLETE.md docs/archive/
Move-Item SECRETS_MANAGEMENT.md docs/archive/
```

### **Phase 3: Move Scripts** (5 min)

```powershell
# Move build scripts
Move-Item build.ps1 scripts/build/
Move-Item build.sh scripts/build/
Move-Item build-crc.ps1 scripts/build/

# Move dev scripts
Move-Item quickstart.ps1 scripts/dev/
Move-Item quickstart.sh scripts/dev/

# Move integration scripts
Move-Item integrate-all-agents.ps1 scripts/integration/
Move-Item fix-agent-imports.ps1 scripts/integration/
Move-Item merge-old-work.ps1 scripts/integration/
Move-Item apply-quick-fix.ps1 scripts/integration/

# Move maintenance scripts
Move-Item simulate-crc-flow.ps1 scripts/maintenance/

# Move tools scripts
Move-Item update-copilot-settings.ps1 scripts/tools/
```

### **Phase 4: Create README Files** (10 min)

Will create README files for each directory to explain purpose and contents.

### **Phase 5: Update .gitignore** (2 min)

Add logs directory to gitignore:
```
# Logs (keep structure, ignore files)
logs/**/*.log
logs/**/*.txt
logs/**/!(README.md)
```

### **Phase 6: Update References** (5 min)

Update any documentation that references moved files.

---

## 📝 **Automation Script**

Create `scripts/maintenance/organize-workspace.ps1`:

```powershell
# Will contain all the move commands above
# Plus validation and rollback capability
```

---

## ✅ **Success Criteria**

### **After Migration**

**Root Directory**:
- ✅ < 20 files (down from 70+)
- ✅ Only essential config and main docs
- ✅ Clear and organized

**Scripts**:
- ✅ All in `scripts/` directory
- ✅ Organized by purpose
- ✅ Documented in README

**Documentation**:
- ✅ All in `docs/` directory
- ✅ Organized by type
- ✅ Easy to find

**Logs**:
- ✅ Clear location (`logs/` dev, `server/data/logs/` prod)
- ✅ Properly gitignored
- ✅ Documented strategy

---

## 🎯 **Quick Start**

```powershell
# Run the organization script
cd D:\dev\workspaces\noa_ark_os
.\scripts\maintenance\organize-workspace.ps1

# Or do it manually with the commands above
```

---

**Status**: Plan ready for execution  
**Impact**: Major cleanup, improved organization  
**Time**: ~30 minutes  
