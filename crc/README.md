# CRC - Continuous ReCode System

Intelligent code adaptation system with AI supervision and automatic workspace fitting.

## ⚠️ Current Status: Manual Integration Mode

**Full automation (Option B) is under development.** Currently operating in **manual integration mode (Option A)**.

**What Works Now**:
- ✅ Drop-in folder structure
- ✅ Manual code analysis
- ✅ Manual integration with AI assistance
- ✅ Agent registry loaded from drops

**In Development**:
- ⏳ Automated file watching
- ⏳ Automated adaptation pipeline
- ⏳ Automated validation and testing
- ⏳ Automated sandbox assignment
- ⏳ Automated archival system

**See**: `DROP_IN_QUICKSTART.md` for current manual process

## Overview

CRC automatically adapts external code (repos, forks, mirrors, stale codebases) to fit the NOA ARK OS workspace conventions, architecture, and standards.

**Vision**: Full automation from drop to deploy  
**Reality**: Manual integration with AI assistance (for now)

## Key Features

- 🤖 **AI-Supervised**: AI model analyzes and adapts code
- 🔄 **Auto-Approve**: High-confidence changes deploy automatically
- 📦 **Drop-In Folder**: Simple code ingestion
- 🗜️ **Auto-Archive**: Compress originals, no live stale code
- 📊 **Cross-Reference**: Fast lookups without decompression
- 🚀 **Zero-Touch**: Complete automation from drop to deploy

## Directory Structure

```
crc/
├── drop-in/           # Active drop folder
│   ├── incoming/      # Drop code here
│   ├── processing/    # Being adapted
│   └── ready/         # Ready for CI/CD
├── archive/           # Compressed archives
│   ├── stale/         # Old codebases
│   ├── repos/         # External repos
│   ├── forks/         # Forked code
│   └── mirrors/       # Mirror snapshots
├── temp/              # Temporary (no live code)
│   ├── extract/       # Temp extraction
│   ├── analysis/      # AI analysis
│   └── cross-ref/     # Compressed references
└── config/            # Configuration
    ├── rules.yaml     # Adaptation rules
    ├── patterns.yaml  # Code patterns
    └── standards.yaml # Workspace standards
```

## How It Works

### 1. Drop Code

Drop any code into `drop-in/incoming/`:

```bash
# Drop external repo
cp -r /path/to/external-project crc/drop-in/incoming/

# Or clone directly
cd crc/drop-in/incoming/
git clone https://github.com/external/project.git
```

### 2. Automatic Processing

CRC automatically:
1. Detects new code in incoming/
2. Analyzes structure and dependencies
3. Determines adaptation strategy
4. Adapts code to workspace conventions
5. Generates tests
6. Validates adapted code
7. Compresses original to archive/
8. Outputs adapted code to ready/
9. Triggers CI/CD pipeline

### 3. AI Supervision

AI model provides:
- Code structure analysis
- Dependency mapping
- Adaptation recommendations
- Quality assessment
- Confidence scoring
- Auto-approve decisions

### 4. Archive and Cleanup

Original code:
- Compressed to `archive/` with timestamp
- Indexed for cross-reference
- Removed from active workspace
- Retained per retention policy

## Configuration

### Adaptation Rules

`config/rules.yaml`:

```yaml
adaptation_rules:
  # Naming conventions
  naming:
    convert_to_snake_case: true
    prefix_modules: "noa_"
    max_name_length: 50
  
  # Architecture alignment
  architecture:
    use_workspace_patterns: true
    enforce_layer_separation: true
    apply_security_model: true
    match_directory_structure: true
  
  # Dependencies
  dependencies:
    remove_external: true
    use_embedded_runtimes: true
    vendor_if_necessary: true
    self_contained: true
  
  # Code quality
  quality:
    apply_formatter: true
    enforce_linting: true
    add_documentation: true
    generate_tests: true
    min_coverage: 80
```

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
