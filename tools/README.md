# Tools# Tools



Development tools and system utilities for NOA ARK OS workspace management.

## 🧹 Maintenance Scripts (`maintenance/`)

The CRC `scripts-1` drop has been adapted into curated maintenance helpers:

- `daily_maintenance.sh` – rotates logs, quarantines stray build artefacts, and captures metrics under `.workspace/metrics/`.
- `workspace_optimization.sh` – relocates bulky `target/` trees into `.workspace/quarantine/` and tidies docs.
- `generate_workspace_report.sh` – emits a snapshot report to `.workspace/reports/`.
- `pre_commit_check.sh` – optional pre-commit hygiene (fmt, clippy, large file scan, metrics).

Run via WSL/Git Bash, e.g. `bash tools/maintenance/daily_maintenance.sh`.


## 📦 Portable Distribution Builder

- `portable_builder.py` – assembles kernel capabilities, runtimes, AI assets, and
  application packages into reproducible bundles. The script reads the kernel
  manifest (`core/config/default_manifest.yaml`) and emits bundle metadata,
  deterministic lock hashes, and target descriptors for OCI/WASI archives.

**Usage**:

```bash
python tools/portable_builder.py --output build/portable
```

CI automation can run `scripts/ci/validate_portable_bundle.sh` to build and
validate the output using the `noa_cicd` validation utilities.



## 📁 Directory Structure## Development Tools



```- Compiler toolchain

tools/- Debugger

├── github/         # GitHub automation and integration- Profiler

│   ├── github_coding_agent.sh    # Automated GitHub coding workflows- Code editor integration

│   └── setup_github_agent.sh     # GitHub agent configuration

├── ai/             # AI model management## System Utilities

│   └── ollama_manager.sh         # Ollama model lifecycle management

├── build/          # Build and analysis utilities- System diagnostics

│   ├── build_code_deps.py        # Dependency analysis and building- Performance monitoring

│   └── build_code_index.py       # Codebase indexing- Log viewer

├── backup/         # Backup and archival tools- Configuration tools

│   └── rotate_backups.sh         # Backup rotation utility

└── README.md       # This file## Structure

```

```

## 🤖 GitHub Automation (`github/`)tools/

├── dev/            # Development tools

### `github_coding_agent.sh`├── utils/          # System utilities

Automated GitHub integration for coding workflows. Provides automated commit, push, PR creation, and issue management.├── build/          # Build system

└── diagnostics/    # Diagnostic tools

**Usage** (via WSL):```

```bash

wsl bash tools/github/github_coding_agent.sh [options]## Self-Contained Build System

```

The build system is itself written to run on NOA ARK OS, making the system self-hosting.

### `setup_github_agent.sh`
Setup and configuration utility for GitHub agent integration.

**Usage** (via WSL):
```bash
wsl bash tools/github/setup_github_agent.sh
```

## 🧠 AI Model Management (`ai/`)

### `ollama_manager.sh`
Comprehensive Ollama AI model management tool. Download, run, monitor, and manage local AI models.

**Features**:
- Model download and installation
- Model lifecycle management
- Performance monitoring
- Resource usage tracking

**Usage** (via WSL):
```bash
wsl bash tools/ai/ollama_manager.sh [command] [model]
```

## 🔨 Build & Analysis (`build/`)

### `build_code_deps.py`
Python utility for analyzing and building code dependencies across the workspace.

**Usage**:
```bash
python tools/build/build_code_deps.py [options]
```

**Features**:
- Dependency graph generation
- Build order calculation
- Circular dependency detection

### `build_code_index.py`
Creates searchable index of codebase for quick navigation and analysis.

**Usage**:
```bash
python tools/build/build_code_index.py [path]
```

## 💾 Backup Management (`backup/`)

### `rotate_backups.sh`
Automated backup rotation utility with configurable retention policies.

**Usage** (via WSL):
```bash
wsl bash tools/backup/rotate_backups.sh [backup_dir] [days]
```

## 🔧 Windows Compatibility Notes

Most tools are bash scripts designed for Linux/WSL environments. To use on Windows:

1. **Via WSL** (Recommended):
   ```bash
   wsl bash tools/[category]/[script].sh
   ```

2. **Via Git Bash**:
   ```bash
   bash tools/[category]/[script].sh
   ```

3. **PowerShell Ports** (Future):
   - Consider porting critical scripts to PowerShell for native Windows execution

## 📦 Source

These tools were migrated from `/home/deflex/workspace/tools/` (WSL) on October 8, 2025.

See `crc/drop-in/incoming/stale/tools/MANIFEST.md` for complete drop analysis and integration details.

## 🔮 Future Enhancements

- [ ] PowerShell ports for native Windows execution
- [ ] Additional build automation tools
- [ ] Workspace health monitoring
- [ ] Automated testing utilities
- [ ] CI/CD integration helpers

## Self-Contained Build System

The build system is designed to run on NOA ARK OS, making the system self-hosting.
