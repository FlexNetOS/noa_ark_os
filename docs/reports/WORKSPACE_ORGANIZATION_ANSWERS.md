# Workspace Organization - Questions Answered

**Date**: Current  
**Purpose**: Answer organization questions and provide execution plan  

---

## ✅ **Your Questions Answered**

### **1. Clean Up Root - Add Sub-folders to Docs**

**Answer**: ✅ **DONE** - Created comprehensive structure

#### **Docs Sub-folders Created**:
- ✅ `docs/notes/` - Development notes and design decisions
- ✅ `docs/audits/` - Security, performance, dependency audits
- ✅ `docs/reports/` - Session, build, integration reports
- ✅ `docs/links/` - External references and resources
- ✅ `docs/projects/` - Project planning, roadmap, milestones
- ✅ `docs/references/` - Technical references (Rust, Python, Go, architecture)
- ✅ `docs/tests/` - Test plans, results, coverage reports
- ✅ `docs/guides/` - User and developer guides
- ✅ `docs/archive/` - Old/outdated documentation

#### **Additional Sub-folders You Might Have Missed**:
- ✅ `docs/audits/security/` - Security audit reports
- ✅ `docs/audits/performance/` - Performance analysis
- ✅ `docs/audits/dependency/` - Dependency audits
- ✅ `docs/projects/milestones/` - Milestone tracking
- ✅ `docs/projects/sprints/` - Sprint planning
- ✅ `docs/references/architecture/` - Architecture diagrams and docs
- ✅ `docs/tests/test-plans/` - Test plan documents
- ✅ `docs/tests/test-results/` - Test execution results
- ✅ `docs/tests/coverage-reports/` - Code coverage reports

---

### **2. Best Practices for Scripts**

**Answer**: ✅ **YES**, scripts should be in a scripts directory

#### **Why Use Scripts Directory?**

**✅ Benefits**:
1. **Clear Separation**: Scripts separated from source code
2. **Easy Discovery**: All automation in one place
3. **Better Permissions**: Can set different security policies
4. **Easier Maintenance**: Centralized script management
5. **Standard Practice**: Industry standard for large projects
6. **Better Documentation**: Single README for all scripts
7. **CI/CD Integration**: Easy to reference in pipelines

**❌ Problems with Root Scripts**:
1. Clutters root directory
2. Mixed with source code and configs
3. Hard to find specific scripts
4. No clear organization
5. Difficult to secure separately

#### **Recommended Structure**:

```
scripts/
├── README.md              # Documentation for all scripts
├── build/                 # Build automation
│   ├── build.ps1
│   ├── build.sh
│   ├── build-crc.ps1
│   └── clean.ps1
├── dev/                   # Development helpers
│   ├── quickstart.ps1
│   ├── quickstart.sh
│   └── setup-env.ps1
├── integration/           # Integration automation
│   ├── integrate-all-agents.ps1
│   ├── fix-agent-imports.ps1
│   └── merge-old-work.ps1
├── maintenance/           # Cleanup and backup
│   ├── organize-workspace.ps1
│   ├── cleanup.ps1
│   └── backup.ps1
├── tools/                 # Tool configuration
│   ├── update-copilot-settings.ps1
│   └── configure-git.ps1
└── testing/               # Test runners
    ├── run-tests.ps1
    └── run-benchmarks.ps1
```

#### **Exception: Component-Specific Scripts**

Some scripts stay with their component if **tightly coupled**:

```
crc/
├── src/
├── detect-forks.ps1       # ✅ CRC-specific, stays here
└── README.md
```

**Rule**: If script is only useful within its component context, keep it there.

#### **Naming Conventions**:

**✅ Good Names**:
- `build-workspace.ps1` - Clear purpose
- `run-tests.ps1` - Action-based
- `deploy-production.sh` - Specific action
- `cleanup-logs.ps1` - Clear what it does

**❌ Bad Names**:
- `script1.ps1` - No context
- `temp.ps1` - Unclear purpose
- `fix.ps1` - Too generic
- `run.sh` - What does it run?

#### **Documentation**:

Every script should have:
```powershell
<#
.SYNOPSIS
    Brief description

.DESCRIPTION
    Detailed description of what script does

.PARAMETER ParameterName
    Description of parameter

.EXAMPLE
    PS> .\script-name.ps1 -Parameter value
    
.NOTES
    Author: Team Name
    Version: 1.0
    Last Modified: Date
#>
```

---

### **3. Where Are Logs Hosted?**

**Answer**: ✅ **It depends on environment**

#### **Development Logs** (Local):

**Location**: `logs/` (in workspace root)

```
logs/
├── README.md
├── app/                   # Application logs
│   ├── server/            # Server logs
│   │   ├── access.log
│   │   ├── error.log
│   │   └── debug.log
│   ├── agents/            # Agent execution logs
│   │   ├── agent-*.log
│   │   └── hive.log
│   └── crc/               # CRC processing logs
│       ├── fork-*.log
│       └── processing.log
├── build/                 # Build logs
│   ├── build-*.log
│   └── errors.log
├── test/                  # Test logs
│   ├── test-*.log
│   └── coverage.log
└── system/                # System logs
    ├── performance.log
    └── health.log
```

**Characteristics**:
- ✅ Local to your machine
- ✅ Not committed to Git (.gitignored)
- ✅ Useful for debugging
- ✅ Can be deleted anytime
- ✅ Rotation: Manual or by size

#### **Production Logs** (Server):

**Location**: `server/data/logs/` (on production server)

```
server/data/logs/
├── app/                   # Application logs
│   ├── server/
│   ├── agents/
│   └── crc/
├── system/                # System logs
│   ├── performance/
│   ├── health/
│   └── metrics/
├── security/              # Security logs
│   ├── access/
│   ├── auth/
│   └── audit/
└── audit/                 # Audit trail
    ├── changes/
    ├── deployments/
    └── admin/
```

**Characteristics**:
- ✅ Persisted on server
- ✅ Backed up regularly
- ✅ Rotation: Daily or by size (100MB)
- ✅ Retention: 30 days (configurable)
- ✅ Monitored by log aggregation tools

#### **Configuration**:

**Rust (using tracing)**:
```rust
// In server/src/main.rs or config
use tracing_subscriber;

fn setup_logging() {
    let log_dir = if cfg!(debug_assertions) {
        "logs/app/server"  // Development
    } else {
        "server/data/logs/app/server"  // Production
    };
    
    let file_appender = tracing_appender::rolling::daily(log_dir, "server.log");
    
    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .json()  // Structured logging
        .init();
}
```

**Configuration File** (`server/config/logging.toml`):
```toml
[logging]
# Log level: trace, debug, info, warn, error
level = "info"

# Output: stdout, file, both
output = "both"

# File path (relative to workspace root)
path = "logs/app/server"

# Production path
production_path = "server/data/logs/app/server"

# Rotation strategy: daily, hourly, size
rotation = "daily"

# Max file size before rotation (MB)
max_size = 100

# Retention (days)
retention_days = 30

# Format: json, text
format = "json"

# Include timestamps
timestamps = true

# Include source location
source_location = false
```

#### **Log Levels**:

**Development**:
- Use: `DEBUG` or `TRACE`
- Keep everything for debugging
- Can be noisy

**Production**:
- Use: `INFO` or `WARN`
- Only important events
- Performance-conscious

**Levels Explained**:
```rust
// TRACE - Very detailed, every step
tracing::trace!("Entering function with param: {}", param);

// DEBUG - Debugging information
tracing::debug!("Processing item {}", id);

// INFO - Important events
tracing::info!("Server started on port {}", port);

// WARN - Warning conditions
tracing::warn!("Deprecated API called");

// ERROR - Error conditions
tracing::error!("Failed to process request: {}", err);
```

#### **Log Rotation**:

**Strategy**:
- **Daily**: New file each day (`server-2024-10-08.log`)
- **Size-based**: New file when > 100MB
- **Combined**: Daily or when > 100MB

**Cleanup**:
```powershell
# In scripts/maintenance/cleanup-logs.ps1
# Delete logs older than 7 days (development)
Get-ChildItem -Path "logs" -Include "*.log" -Recurse -File | 
    Where-Object {$_.LastWriteTime -lt (Get-Date).AddDays(-7)} | 
    Remove-Item

# Production: 30 days retention (handled by log rotation)
```

#### **Monitoring & Aggregation** (Production):

**Tools**:
- **Loki** (Grafana stack) - Log aggregation
- **ELK** (Elasticsearch, Logstash, Kibana) - Full-text search
- **CloudWatch** (AWS) - Cloud-native
- **Azure Monitor** (Azure) - Cloud-native

**Example Setup**:
```
Logs → Loki → Grafana → Alerts
```

---

## 📊 **Summary**

### **Your Questions**:

1. ✅ **Root Cleanup**: 
   - Added 9 sub-folders to docs/
   - Plus sub-sub-folders for organization
   - Covered: notes, audits, reports, links, projects, references, tests, guides, archive

2. ✅ **Scripts Best Practices**:
   - YES, use `scripts/` directory
   - Organize by purpose (build, dev, integration, maintenance, tools, testing)
   - Exception: Keep component-specific scripts with component
   - Document all scripts

3. ✅ **Logs Location**:
   - **Development**: `logs/` (local, gitignored)
   - **Production**: `server/data/logs/` (persisted, backed up)
   - **Rotation**: Daily or by size
   - **Retention**: 7 days (dev), 30 days (prod)

---

## 🚀 **Execute Organization**

### **Option 1: Automated** (Recommended)

```powershell
cd D:\dev\workspaces\noa_ark_os

# Dry run first (see what would happen)
.\organize-workspace.ps1 -WhatIf

# Execute
.\organize-workspace.ps1

# Commit
git add .
git commit -m "refactor: Organize workspace - docs, scripts, logs structure"
git push origin main
```

### **Option 2: Manual**

Follow commands in `WORKSPACE_ORGANIZATION_PLAN.md`

---

## ✅ **After Organization**

### **Root Directory Will Have**:
```
noa_ark_os/
├── .github/
├── tools/devshell/
├── agents/               # Source
├── crc/                  # Source
├── cicd/                 # Source
├── core/                 # Source
├── docs/                 # ✨ ALL documentation
├── examples/             # Source
├── logs/                 # ✨ Log files (gitignored)
├── notebooks/            # Source
├── sandbox/              # Source
├── scripts/              # ✨ ALL scripts
├── server/               # Source
├── services/             # Source
├── tests/                # Tests
├── tools/                # Tools
├── .vscode/              # Optional editor hints (mirrors devshell)
├── ui/                   # Source
├── workflow/             # Source
├── .gitignore
├── Cargo.toml
├── Cargo.lock
├── CONTRIBUTING.md
├── LICENSE
├── NoaArkOS.sln
├── README.md
└── WORKSPACE_MEMORY.md
```

**Total Root Files**: ~15 (down from 70+!)

---

## 📝 **Quick Reference**

### **Documentation**
- Session reports: `docs/reports/sessions/`
- Build reports: `docs/reports/builds/`
- Integration reports: `docs/reports/integrations/`
- Guides: `docs/guides/`
- References: `docs/references/`

### **Scripts**
- Build: `scripts/build/`
- Development: `scripts/dev/`
- Integration: `scripts/integration/`
- Maintenance: `scripts/maintenance/`

### **Logs**
- Development: `logs/app/`, `logs/build/`, `logs/test/`
- Production: `server/data/logs/`

---

**Status**: ✅ All questions answered, plan ready to execute!

**Next**: Run `.\organize-workspace.ps1 -WhatIf` to preview changes, then execute!
