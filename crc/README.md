# CRC – Continuous ReCode System (`noa_crc`)

CRC is the Continuous ReCode System for NOA ARK OS. It ingests external or
stale code, normalizes it into a safe workspace shape, and prepares artifacts
for downstream CI/CD and relocation flows.

It is implemented as a Rust crate (`noa_crc`) with a CLI (`crc`) plus a set of
supporting directories under the workspace `crc/` root.

## Current Status

CRC is no longer purely “manual integration only”. The codebase now provides:

**Implemented**
- ✅ Content-addressed storage (`crc::cas`) with a CLI subcommand (`crc cas …`)
- ✅ File watcher for drop-in folders (`crc::watcher`) with source-type detection
- ✅ Drop processing pipeline (`crc::processor::DropProcessor`) that:
  - Analyzes drops (file/line counts, languages, dependency scan, basic patterns)
  - Adapts them to workspace conventions (stubbed but wired)
  - Validates structure (stubbed cargo check, required files, basic heuristics)
  - Assigns sandbox models based on `SourceType` and confidence
  - Produces profile-specific builds under `storage/artifacts/*`
  - Archives drops via `ArchiveManager` with per-source retention policies
- ✅ Parallel queueing skeleton (`crc::parallel::ParallelDropProcessor`) for staged
  analysis → adaptation → validation with synthetic timing
- ✅ CRC graph + engine (`crc::graph`, `crc::engine`) and an orchestrator for
  queueing graph-based jobs
- ✅ CLI commands (`crc/src/main.rs`) for:
  - `serve` – run watcher + parallel processor together
  - `run` – execute a CRC graph plan once to a checkpoint
  - `ingest` – run digestors and emit an ingest report
  - `cas` – interact with the content addressed store
  - `migrate` – apply FileReplacePlan migrations
  - `graph` – inspect CRC graph nodes and traces
- ✅ Telemetry integration with structured events across CAS, ingest, service,
  and migration paths
- ✅ Integration tests for CAS, archive ingestion, digestors, DAG seed, and basic
  orchestrator/parallel processing flows (`crc/tests/*.rs`)

**Simulated / Incomplete**
- ⚠️ `CRCSystem::scan_incoming` is a stub (returns an empty list).
- ⚠️ `DropProcessor`’s AI-related behavior (patterns, test generation, cargo
  checks) is simulated or no-op:
  - `generate_missing_tests`, `apply_conventions`, `run_cargo_check`, and
    `find_issues` are intentionally shallow placeholders.
- ⚠️ `ParallelDropProcessor` uses synthetic timers and does not yet call the
  real `DropProcessor` / archive pipeline.
- ⚠️ The watcher does not directly trigger the processor; a proper queue or
  channel wiring point is left as a TODO.
- ⚠️ YAML configs in `crc/config/` (rules/patterns/standards/sources) describe
  desired behavior but are not yet consumed by the Rust pipeline.

**Aspirational (docs mention, code not fully there yet)**
- 🤖 True AI-backed analysis/adaptation instead of heuristics.
- 🔄 End-to-end “drop → adapt → verify → ready → CI/CD” with no manual hand-offs.
- 📊 Agent registry integration (“agent registry loaded from drops”) – this is
  not present in the current crate and should be treated as future work.

For the operator-facing workflow, see:
- `DROP_IN_QUICKSTART.md` – drop-in directory walkthrough
- `QUICK_REFERENCE.md` – quick-start card
- `SANDBOX_MODELS.md` – sandbox model details
- `USER_WORKFLOW.md` – user-oriented CRC flow

## Directory Layout (runtime)

The `crc` crate assumes the following workspace-relative layout (created on
startup by `verify_directory_structure()` when missing):

```text
crc/
├── config/                         # YAML configuration, not yet fully wired
│   ├── rules.yaml                  # Adaptation rules (naming, deps, quality)
│   ├── patterns.yaml               # Pattern detection hints
│   ├── sources.yaml                # Source-type configuration
│   └── standards.yaml              # Coding / architecture standards
├── drop-in/
│   ├── incoming/
│   │   ├── repos/                  # Active / maintained repositories
│   │   ├── forks/                  # Developer-maintained forks
│   │   ├── mirrors/                # Mirrored/backup repos
│   │   └── stale/                  # Old/unmaintained code drops
│   ├── processing/
│   │   ├── analysis/               # Analysis workspaces
│   │   ├── adaptation/             # Adaptation workspaces
│   │   └── validation/             # Validation workspaces
│   └── ready/
│       ├── model-a-queue/          # Feature sandbox queue
│       ├── model-b-queue/          # Bugfix sandbox queue
│       ├── model-c-queue/          # Experimental sandbox queue
│       └── model-d-queue/          # Integration sandbox queue
├── archive/
│   ├── stale/                      # Archived stale codebases
│   ├── repos/                      # Archived external repos
│   ├── forks/                      # Archived forks
│   ├── mirrors/                    # Archived mirrors
│   └── internal/                   # Archived internal drops
├── temp/
│   ├── analysis-cache/             # Cached analysis outputs
│   ├── extracts/                   # Temporary extraction roots
│   └── logs/                       # CRC logs and traces
├── storage/
│   └── artifacts/
│       ├── edge/                   # Edge/agent-friendly build artifacts
│       └── server/                 # Server-side build artifacts
└── src/, tests/, sandboxes/, out/  # Crate sources, tests, and examples
```

The default configuration in `CRCConfig::default()` matches this layout.

## Core Components

CRC is organized into focused modules (all under `crc/src/`):

- `lib.rs` – type definitions (`CodeDrop`, `DropManifest`, `CRCSystem`,
  `CRCConfig`, `SandboxModel`, `CRCState`, archive index types) and public
  module exports.
- `main.rs` – `crc` CLI entrypoint (serve, run, ingest, cas, migrate, graph).
- `cas.rs` – content-addressed store (BLAKE3-based) for artifacts
  (`storage/cas` by default), with `Cas::put_bytes`, `Cas::get`, and `Cas::stat`.
- `archive.rs` – `ArchiveManager` providing compressed archives with retention
  policies, hash computation, cleanup, and metadata persistence.
- `processor.rs` – `DropProcessor` pipeline: analyze → adapt → validate →
  sandbox assignment → ready queue move → build generation → archive +
  cleanup. Many behaviors are intentionally simplified but fully wired.
- `watcher.rs` – filesystem watcher that:
  - Watches the `crc/drop-in/incoming/*` folders
  - Classifies paths into `SourceType`
  - Runs extraction and metadata capture
  - Generates a `DropManifest` and registers it in `CRCSystem`
- `parallel.rs` – `ParallelDropProcessor` for generic queue-based processing of
  drop IDs across Analysis/Adaptation/Validation stages (currently simulates
  work with sleeps and synthetic confidence scores).
- `graph.rs` / `engine.rs` / `orchestrator.rs` – graph-based pipeline
  execution, topological ordering, and queued job orchestration for more
  complex flows.
- `digestors/` – pluggable digestors for `git`, `config`, `API specs`, SBOMs,
  and binaries, used by the `crc ingest` command and `crc/tests/digest_smoke.rs`.
- `build.rs` – build manifest and artifact definitions plus helpers to produce
  profile-optimized builds into `storage/artifacts/*`.
- `telemetry.rs` – structured logging helpers (`info`, `warn`, `error`) used
  across CLI and services.
- `error.rs`, `types.rs`, `ir.rs` – shared error, type aliases, and pipeline
  IR (lanes, node states, etc.).

## CLI Overview (`crc` binary)

From the workspace root:

```bash
cargo run -p noa_crc -- --help
```

Key subcommands:

- `crc serve` – start the long-lived CRC service:
  - Ensures the directory layout exists (`verify_directory_structure`).
  - Instantiates `CRCSystem` + `CRCWatcher` + `ParallelDropProcessor`.
  - Runs watcher and processor concurrently until shutdown.

- `crc ingest --root <PATH> --report <FILE>` – digest a repo or artifact root
  with all digestors and emit a JSON ingest report (assets + coverage +
  trust averages).

- `crc cas put/get/stat` – store and retrieve CAS objects, or inspect their
  metadata (size, path, timestamps).

- `crc run --checkpoint <DIR>` – construct a simple CRC graph and run it once
  into a checkpoint directory for debugging or demos.

- `crc migrate --plan <PLAN.yaml|json> [...] --root <PATH>` – apply a sequence
  of file replacement plans and optionally roll them back, with telemetry
  events for traceability.

- `crc graph ls|show|trace` – inspect CRC graph nodes and topological traces;
  useful when debugging orchestrated flows.

The PowerShell `detect-forks.ps1` remains available as a Windows-centric helper
around the drop-in directories, but the canonical API is now the `crc` CLI.

## Local Tests and Smoke Checks

From the workspace root:

```bash
# CAS round-trip hash stability
cargo test -p noa_crc --test cas_smoke

# Digestors + ingest report generation
cargo test -p noa_crc --test digest_smoke

# DAG orchestration checkpoint verification
cargo test -p noa_crc --test dag_seed

# Archive ingestion and retention
cargo test -p noa_crc --test archive_ingestion
```

These tests exercise CAS, digestors, basic graph orchestration, and archive
handling to catch regressions early.

## Known Gaps & Conflicts (Code vs. Docs)

To align with the AGENT policy (“heal, do not harm”) the following gaps are
explicitly tracked rather than hidden:

- **Manual vs. automatic integration**
  - Earlier docs (and older README text) describe CRC as “manual integration
    only”. The current codebase clearly includes a watcher, processor, and
    parallel queues. However, the wiring between watcher → CRCSystem →
    DropProcessor → ParallelDropProcessor is not yet complete; processing is
    still orchestrated manually or via demos.

- **Agent registry integration**
  - The previous README claimed “Agent registry loaded from drops”. There is
    no agent registry logic in `noa_crc`; any such integration will likely live
    in agents/gateway layers and should be added explicitly when designed.

- **AI-supervised behavior**
  - The pipeline carries AI-related fields (`ai_confidence`, auto-approve
    flags), but uses deterministic heuristics and simulated sleeps rather than
    actual model calls. Any AI provider integration should be introduced via
    gateway-managed config (per AGENT.md) and documented as such.

- **Config usage**
  - `crc/config/*.yaml` defines rich adaptation/standards metadata that is not
    fully consumed by the current Rust code. Future work should:
    - Parse these files into typed structs.
    - Drive `DropProcessor` behavior directly from them.
    - Emit evidence into the workspace indexes and evidence ledger.

- **CI/CD triggers**
  - `CRCConfig::trigger_cicd` and related flags exist, but there is no concrete
    integration with the workspace CI/CD yet. When added, it should go through
    the kernel/gateway policy surface rather than ad-hoc shell commands.

As the implementation advances, this README should be kept in sync with actual
symbols and behavior in `crc/src/*.rs`, treating the crate as the single source
of truth and keeping aspiration clearly separated from shipped functionality.

### Code Patterns

`config/patterns.yaml`:

```yaml
patterns:
  # Replace external HTTP libraries
  - pattern: "requests.get(url)"
    replace: "noa_http::get(url)"
    reason: "Use internal HTTP client"
  
  # Replace external database
  - pattern: "import sqlite3"
    replace: "use noa_storage::db"
    reason: "Use embedded database"
  
  # Add error handling
  - pattern: "risky_function()"
    wrap: "match risky_function() { Ok(v) => v, Err(e) => handle_error(e) }"
    reason: "Ensure proper error handling"
  
  # Add logging
  - pattern: "def important_function"
    instrument: "log::info!(\"Called: {}\", function_name);"
    reason: "Add observability"
```

### Workspace Standards

`config/standards.yaml`:

```yaml
standards:
  # Language-specific
  rust:
    edition: "2021"
    lints:
      - clippy::all
      - clippy::pedantic
    format: rustfmt
  
  python:
    version: "3.11"
    lints:
      - pylint
      - mypy
    format: black
  
  go:
    version: "1.21"
    lints:
      - golangci-lint
    format: gofmt
  
  # Testing requirements
  testing:
    min_coverage: 80
    require_unit_tests: true
    require_integration_tests: true
  
  # Documentation
  documentation:
    require_readme: true
    require_docstrings: true
    require_examples: true
```

## AI Model

### Capabilities

- **Code Analysis**: Understand structure, patterns, dependencies
- **Semantic Understanding**: Comprehend intent and behavior
- **Pattern Matching**: Identify common patterns to replace
- **Dependency Resolution**: Find embedded alternatives
- **Test Generation**: Create comprehensive tests
- **Quality Assessment**: Evaluate code quality

### Confidence Scoring

```
95-100%: Auto-approve and deploy
80-95%:  Queue for human review
50-80%:  Reject with detailed feedback
0-50%:   Reject immediately
```

### Learning System

AI improves by:
- Learning from human reviews
- Tracking successful adaptations
- Identifying common patterns
- Building pattern library
- Improving confidence calibration

## Archive System

### Compression Strategy

```yaml
compression:
  algorithm: zstd        # Fast + good ratio
  level: 3               # Balanced
  
  triggers:
    - processed: true    # After CRC complete
    - age: 7d            # Or after 7 days
    - size: 100MB        # Large files immediately
  
  retention:
    stale: 90d           # Keep 90 days
    repos: 180d          # Keep 180 days
    forks: 90d           # Keep 90 days
    mirrors: 30d         # Keep 30 days
    temp: 1d             # Clean daily
```

### Archive Structure

```
archive/
├── stale/
│   └── 2024-01-15_old-project_abc123.tar.zst
├── repos/
│   └── github-com-user-repo_commit-hash.tar.zst
├── forks/
│   └── fork-original-name_2024-01-15.tar.zst
└── mirrors/
    └── mirror-gitlab-project_snapshot-123.tar.zst
```

### Cross-Reference Index

Instead of keeping live code, maintain compressed index:

```json
{
  "version": "1.0",
  "archives": {
    "project-abc": {
      "hash": "sha256:abc123...",
      "archive_path": "archive/repos/project-abc.tar.zst",
      "created": "2024-01-15T10:30:00Z",
      "size": 12500000,
      "index": {
        "files": [
          {"path": "src/main.rs", "hash": "sha256:..."},
          {"path": "src/lib.rs", "hash": "sha256:..."}
        ],
        "symbols": [
          {"name": "main", "file": "src/main.rs", "line": 10},
          {"name": "process", "file": "src/lib.rs", "line": 45}
        ],
        "dependencies": [
          {"name": "serde", "version": "1.0"},
          {"name": "tokio", "version": "1.35"}
        ]
      }
    }
  }
}
```

## Integration with CI/CD

### Automatic Trigger

When adapted code reaches `ready/`:
1. CRC notifies CI/CD system
2. CI pipeline starts automatically
3. Tests run on adapted code
4. If all pass, CD deploys
5. Full automation, zero human touch

### Pipeline Integration

```yaml
# Automatic pipeline trigger
on:
  crc_complete:
    - analyze: adapted code
    - validate: tests and quality
    - approve: if confidence > 95%
    - trigger: ci/cd pipeline
```

## Monitoring

### Real-Time Dashboard

```
CRC Dashboard
├── Queue: 3 projects in incoming/
├── Processing: 1 project adapting
├── Ready: 2 projects ready for CI
├── Archive: 45 GB compressed
└── Metrics:
    ├── Success Rate: 94%
    ├── Avg Processing: 8m 23s
    ├── Auto-Approve: 87%
    └── Archive Growth: +2GB/week
```

### Metrics Tracked

- **Adaptation Success Rate**: % successfully adapted
- **AI Confidence**: Average and distribution
- **Auto-Approve Rate**: % auto-approved
- **Processing Time**: Average and p95
- **Archive Size**: Total and growth rate
- **Cross-Ref Queries**: Speed and accuracy
- **CI/CD Success**: % adapted code that deploys

## Usage Examples

### Example 1: Adapt External Repo

```bash
# Drop repo
cd crc/drop-in/incoming/
git clone https://github.com/external/useful-lib.git

# CRC automatically processes
# Watch status
crc status useful-lib

# Output:
# Status: Processing
# Stage: Analyzing dependencies
# Progress: 45%
# AI Confidence: 92%
# ETA: 3m 15s
```

### Example 2: Adapt Stale Internal Code

```bash
# Drop old codebase
cp -r /old-projects/legacy-system crc/drop-in/incoming/

# Add manifest
cat > crc/drop-in/incoming/legacy-system/manifest.json <<EOF
{
  "source": "internal",
  "type": "stale",
  "original_date": "2020-06-15",
  "priority": "low"
}
EOF

# CRC processes and archives
```

### Example 3: Query Archived Code

```bash
# Search archived code without extracting
crc search "function_name" --in-archives

# Output:
# Found in: archive/repos/project-abc.tar.zst
# File: src/lib.rs
# Line: 145
# Context: pub fn function_name(param: i32) -> Result<String>
```

## Error Handling

### Adaptation Failures

If adaptation fails:
1. Log detailed error
2. Store in failed/ directory
3. Notify admin (if configured)
4. Keep original in incoming/
5. Provide feedback for improvement

### Low Confidence

If AI confidence < 80%:
1. Don't auto-approve
2. Create human review task
3. Provide detailed analysis
4. Show suggested changes
5. Wait for manual approval

## Best Practices

1. **Regular Drops**: Drop code regularly, don't batch
2. **Clear Manifests**: Include metadata in manifest.json
3. **Monitor Dashboard**: Check for issues daily
4. **Review Rejections**: Learn from low-confidence rejections
5. **Archive Cleanup**: Follow retention policies
6. **Index Maintenance**: Keep cross-ref index optimized

## Security

### Sandbox Execution

All adaptation happens in isolated sandbox:
- No network access
- Limited file system
- Resource quotas
- Timeout enforcement

### Code Scanning

Before adaptation:
- Security vulnerability scan
- Malware detection
- License compliance check
- Suspicious pattern detection

## Performance

### Optimization

- Parallel processing of multiple projects
- Incremental analysis for large codebases
- Cached pattern matching
- Compressed index for fast lookups
- Lazy archive extraction

### Benchmarks

- Small project (< 1000 lines): ~2 minutes
- Medium project (1000-10000 lines): ~8 minutes
- Large project (> 10000 lines): ~20 minutes
- Archive compression: ~30 seconds
- Cross-ref query: < 100ms

## Future Enhancements

- [ ] Multi-language AI models (specialized per language)
- [ ] Interactive adaptation mode
- [ ] Diff visualization
- [ ] Pattern learning from successful adaptations
- [ ] Distributed processing for large codebases
- [ ] Real-time collaboration on reviews
- [ ] API for external integrations
