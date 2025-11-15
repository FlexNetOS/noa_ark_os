# Workspace Organization and Management

## Overview

NOA ARK OS maintains a clean, organized workspace with clear structure, no redundant copies, and comprehensive management systems.

## Core Principles

1. **Single Source of Truth (SOT)**: One canonical version of every file
2. **No Live Duplicates**: All backups and versions are compressed and archived
3. **Clear Organization**: Intuitive file and folder structure
4. **Comprehensive Tracking**: Hashes, indexes, and registries for all assets
5. **Standard Operating Procedures (SOPs)**: Documented processes for all operations

## Directory Structure

```
noa_ark_os/
├── .workspace/                    # Workspace management (NEW!)
│   ├── registry/                  # Central registry
│   │   ├── files.json            # File hash registry
│   │   ├── versions.json         # Version tracking
│   │   ├── dependencies.json     # Dependency graph
│   │   └── assets.json           # Asset catalog
│   ├── backups/                   # Compressed backups
│   │   ├── daily/                # Daily snapshots
│   │   ├── weekly/               # Weekly archives
│   │   └── monthly/              # Monthly archives
│   ├── indexes/                   # Search indexes
│   │   ├── code.idx              # Code index
│   │   ├── docs.idx              # Documentation index
│   │   └── config.idx            # Configuration index
│   ├── todo/                      # Task management
│   │   ├── backlog.md            # Backlog items
│   │   ├── current-sprint.md     # Current tasks
│   │   └── completed.md          # Completed tasks
│   ├── sop/                       # Standard Operating Procedures
│   │   ├── development.md        # Development SOPs
│   │   ├── deployment.md         # Deployment SOPs
│   │   ├── backup.md             # Backup SOPs
│   │   └── recovery.md           # Recovery SOPs
│   └── config.yaml                # Workspace configuration
│
├── core/                          # Core OS (organized)
├── crc/                           # CRC with sandbox models
│   ├── sandboxes/                # Sandbox environments (NEW!)
│   │   ├── model-a/              # Sandbox Model A
│   │   ├── model-b/              # Sandbox Model B
│   │   ├── model-c/              # Sandbox Model C
│   │   ├── model-d/              # Integration Model D
│   │   └── merge/                # Merge operations
│   ├── drop-in/                  # Code drop-in
│   ├── archive/                  # Compressed archives
│   └── config/                   # CRC configuration
│
├── .graphs/                       # Visual graphs (NEW!)
│   ├── architecture/             # Architecture diagrams
│   ├── dependencies/             # Dependency graphs
│   ├── workflows/                # Workflow visualizations
│   └── metrics/                  # Metrics dashboards
│
├── .self-hosted/                  # Self-hosted apps (NEW!)
│   ├── owned/                    # NOA ARK owned apps
│   ├── external/                 # External apps (switchable)
│   └── registry.json             # App registry
│
└── [other directories...]        # Existing structure
```

## File Organization Rules

### 1. Single Source of Truth (SOT)

**Rule**: Only one canonical version of each file exists in the active workspace.

**Implementation**:
- Active files: Main workspace directories
- Versions: Compressed in `.workspace/backups/`
- Archives: Compressed in component-specific archives

**Example**:
```
# CORRECT:
core/src/lib.rs                              # Active version
.workspace/backups/daily/core-src-lib.rs.gz  # Yesterday's backup

# INCORRECT:
core/src/lib.rs                              # Active
core/src/lib.rs.backup                       # ❌ No live backups!
core/src/lib.rs.old                          # ❌ No live copies!
```

### 2. No Live Backups

**Rule**: All backups, old versions, and copies must be compressed.

**Compression**:
- Algorithm: zstd (level 3)
- Extension: `.zst`
- Metadata: Included in registry

**Automatic Cleanup**:
```yaml
cleanup:
  patterns:
    - "*.backup"
    - "*.old"
    - "*.bak"
    - "*~"
    - "*.orig"
  action: "compress_and_move"
  destination: ".workspace/backups/"
```

### 3. Clear Naming

**Conventions**:
```
# Files
lowercase-with-dashes.ext        # General files
snake_case.rs                    # Rust files
PascalCase.cs                    # C# files

# Directories
lowercase-with-dashes/           # General dirs
snake_case/                      # Module dirs
PascalCase/                      # Namespace dirs

# Archives
YYYY-MM-DD_component_hash.tar.zst
```

## Registry System

### File Hash Registry

**`.workspace/registry/files.json`**:
```json
{
  "version": "1.0",
  "updated": "2024-01-15T10:30:00Z",
  "files": {
    "core/src/lib.rs": {
      "hash": "sha256:abc123...",
      "size": 12345,
      "modified": "2024-01-15T09:00:00Z",
      "backed_up": "2024-01-15T10:00:00Z",
      "backup_location": ".workspace/backups/daily/2024-01-15_core-src-lib.rs.zst"
    }
  }
}
```

### Version Tracking

**`.workspace/registry/versions.json`**:
```json
{
  "core/src/lib.rs": {
    "current": {
      "version": "1.2.3",
      "hash": "sha256:abc123...",
      "date": "2024-01-15T09:00:00Z"
    },
    "history": [
      {
        "version": "1.2.2",
        "hash": "sha256:def456...",
        "date": "2024-01-14T15:00:00Z",
        "archive": ".workspace/backups/daily/2024-01-14_core-src-lib.rs.zst"
      }
    ]
  }
}
```

### Dependency Graph

**`.workspace/registry/dependencies.json`**:
```json
{
  "graph": {
    "core": {
      "depends_on": [],
      "used_by": ["agents", "workflow", "sandbox", "cicd", "crc"]
    },
    "crc": {
      "depends_on": ["core"],
      "used_by": ["cicd"]
    }
  }
}
```

### Asset Catalog

**`.workspace/registry/assets.json`**:
```json
{
  "models": [
    {
      "name": "code-adapter-v1",
      "type": "ai_model",
      "location": "ai/models/code-adapter-v1.gguf",
      "hash": "sha256:...",
      "size": 7340032000,
      "compressed": true
    }
  ],
  "configs": [...],
  "docs": [...]
}
```

## TODO Management

### Structure

```
.workspace/todo/
├── backlog.md           # All future tasks
├── current-sprint.md    # Active tasks (2-week sprint)
├── completed.md         # Done tasks (for reference)
└── templates/
    ├── task.md          # Task template
    └── epic.md          # Epic template
```

### Task Format

```markdown
## [PRIORITY] Task Title

**Status**: Not Started | In Progress | Blocked | Done
**Assignee**: System | Human | Agent
**Sprint**: 2024-W03
**Estimate**: 2 hours
**Dependencies**: [Other tasks]

### Description
Clear description of the task.

### Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2

### Notes
Additional context.
```

## Standard Operating Procedures (SOPs)

### Development SOP

**`.workspace/sop/development.md`**:
```markdown
# Development Standard Operating Procedure

## 1. Starting New Work
1. Check TODO list
2. Create branch (if needed)
3. Update task status to "In Progress"

## 2. Making Changes
1. Edit files (SOT - no duplicates)
2. Run tests locally
3. Commit with conventional format

## 3. Completing Work
1. Run full test suite
2. Update documentation
3. Mark task as "Done"
4. Trigger CI/CD
```

### Backup SOP

**`.workspace/sop/backup.md`**:
```markdown
# Backup Standard Operating Procedure

## Automated Backups

### Daily (00:00 UTC)
- Compress changed files
- Store in .workspace/backups/daily/
- Retention: 7 days

### Weekly (Sunday 00:00 UTC)
- Compress full workspace
- Store in .workspace/backups/weekly/
- Retention: 4 weeks

### Monthly (1st of month 00:00 UTC)
- Compress full workspace
- Store in .workspace/backups/monthly/
- Retention: 12 months

## Manual Backup
```bash
workspace backup --type manual --note "Before major refactor"
```

## Restoration
```bash
workspace restore --date 2024-01-15
```
```

## Workspace Commands

### CLI Tool

```bash
# Workspace management
workspace init                    # Initialize workspace
workspace status                  # Show workspace status
workspace clean                   # Remove duplicates, compress old files
workspace check                   # Verify integrity

# Registry
workspace registry update         # Update all registries
workspace registry verify         # Verify hashes
workspace hash <file>             # Calculate file hash

# Backups
workspace backup                  # Manual backup
workspace restore <date>          # Restore from backup
workspace list-backups            # List available backups

# TODO
workspace todo list               # List tasks
workspace todo add "Task name"    # Add task
workspace todo start <id>         # Start task
workspace todo done <id>          # Complete task

# SOPs
workspace sop list                # List SOPs
workspace sop show <name>         # Show SOP
```

## Cleanup Automation

### Daily Cleanup (Automatic)

```yaml
daily_cleanup:
  schedule: "00:00 UTC"
  tasks:
    - find_duplicates:
        action: "remove_newer"
        keep: "oldest"
    
    - find_backups:
        patterns: ["*.backup", "*.old", "*.bak"]
        action: "compress_and_archive"
    
    - compress_logs:
        older_than: "1 day"
        destination: ".workspace/backups/logs/"
    
    - cleanup_temp:
        directories: ["tmp/", "temp/", "crc/temp/"]
        older_than: "1 day"
    
    - update_registries:
        - files
        - versions
        - dependencies
```

## Monitoring

### Dashboard

```
NOA ARK OS Workspace Status
═══════════════════════════════════════════════

Storage:
  Active Files:     12,345 files (2.3 GB)
  Compressed:       45,678 files (8.9 GB compressed, 156 GB uncompressed)
  Compression:      94.3% reduction

Integrity:
  Hash Mismatches:  0
  Missing Files:    0
  Orphaned Backups: 0

Backups:
  Daily:    7 snapshots (last: 2024-01-15 00:00)
  Weekly:   4 archives (last: 2024-01-14 00:00)
  Monthly:  3 archives (last: 2024-01-01 00:00)

TODO:
  Backlog:          45 tasks
  Current Sprint:   12 tasks (8 in progress, 4 not started)
  Completed:        234 tasks

Last Cleanup:       2024-01-15 00:00:15
Next Cleanup:       2024-01-16 00:00:00
```

## Best Practices

### DO:
✅ Keep only one version of each file
✅ Compress all backups and old versions
✅ Update registries after changes
✅ Follow naming conventions
✅ Document in SOPs
✅ Use TODO system
✅ Run cleanup regularly

### DON'T:
❌ Create `.backup` or `.old` files
❌ Keep uncompressed archives
❌ Modify files in backup directories
❌ Skip registry updates
❌ Ignore TODO items
❌ Work without SOPs

## Migration Guide

### Converting Existing Workspace

```bash
# 1. Initialize workspace management
workspace init

# 2. Find and compress duplicates
workspace clean --dry-run         # Preview
workspace clean --execute         # Execute

# 3. Build registries
workspace registry build

# 4. Set up automation
workspace setup-automation

# 5. Verify
workspace check
```

## Integration

### With CRC
```yaml
crc:
  on_complete:
    - update_registry
    - compress_original
    - update_dependencies
```

### With CI/CD
```yaml
cicd:
  before_build:
    - workspace check
  after_deploy:
    - workspace backup
    - update_registry
```

## Metrics

Track workspace health:
- File count (active vs. archived)
- Storage usage (active vs. compressed)
- Compression ratio
- Duplicate detection rate
- Registry update frequency
- Backup success rate
- TODO completion rate
