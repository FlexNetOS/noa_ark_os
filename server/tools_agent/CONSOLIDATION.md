# Consolidation Tool Documentation

## Overview

The consolidation tool implements the File Consolidation Protocol defined in `AGENT.md` Section 12. It merges multiple source files into a single canonical file while preserving all functionality, maintaining complete version history, and ensuring zero capability loss.

## Usage

### Basic Command

```bash
cargo run --bin agent_tools -- consolidate \
  --sources path/to/file1.rs,path/to/file2.rs \
  --target path/to/canonical.rs \
  --verify \
  --archive \
  --reason "Merging duplicate implementations"
```

### Options

- `--sources <SOURCES>` (required): Comma-separated list of source files to merge
- `--target <TARGET>` (required): Target canonical file (will be created if doesn't exist)
- `--verify`: Run verification checks including capability matrix comparison (default: false)
- `--archive`: Archive source files with versioned naming (default: true)
- `--reason <REASON>`: Consolidation reason/notes for the ledger (default: "Manual consolidation")
- `--root <ROOT>`: Repository root directory (default: current directory)

## Features Implemented

### 1. AST-Based Capability Extraction ✅

The tool uses the `syn` crate to parse Rust source files and extract:
- Public functions
- Public structs
- Public enums
- Public traits
- Public constants
- Public type aliases
- Macros

**Location**: `server/tools_agent/src/consolidation.rs::extract_capabilities()`

**Example**:
```rust
pub fn extract_capabilities(file_path: &Path) -> Result<Vec<Capability>>
```

### 2. Intelligent Merging ✅

Merges content from multiple source files into a canonical target:
- Preserves existing target content
- Adds consolidation metadata comments with timestamps
- Includes source attribution with SHA-256 hashes
- Documents original file paths and capability counts

**Output Format**:
```rust
// ========================================
// Consolidation performed: 2025-11-19T12:00:00Z
// Reason: Manual consolidation
// ========================================

// Consolidated from: old/auth_v1.rs (capabilities: 12)
// SHA-256: abc123...
<original content>
```

### 3. Capability Preservation Verification ✅

When `--verify` flag is used:
- Re-extracts capabilities from merged file
- Compares with capabilities from all source files
- Calculates preservation percentage
- Warns if any capabilities are lost
- Displays detailed capability matrix

**Example Output**:
```
✅ Verification Phase
  Merged file capabilities: 25
  Capability preservation: 25/25 (100.0%)
```

### 4. Version-Tracked Archival ✅

Each consolidation creates:
- **Compressed archives**: `archive/consolidation/YYYY/MM/<path>/filename_v<N>.tar.zst`
- **Version ledger**: `versions.json` with metadata for each version
- **Consolidation index**: `.workspace/registry/consolidation_index.json`

**Version Ledger Schema**:
```json
{
  "canonical_file": "server/gateway/src/auth.rs",
  "versions": [
    {
      "version": "v1",
      "source_path": "old/auth_v1.rs",
      "timestamp": "2025-11-19T12:00:00Z",
      "sha256": "abc123...",
      "consolidation_reason": "Manual consolidation",
      "preserved_capabilities": ["authenticate", "authorize"],
      "archived_capabilities": [],
      "merged_by": "agent_tools_cli"
    }
  ]
}
```

### 5. Consolidation Reports ✅

Generates markdown reports with:
- Consolidation metadata (date, canonical file, sources merged)
- Capability comparison table
- Test status (placeholder for future integration)
- Preservation statistics

**Report Location**: `consolidation_report_YYYYMMDD_HHMMSS.md`

**Example Report**:
```markdown
# Consolidation Report

**Date**: 2025-11-19T12:00:00Z
**Canonical File**: `server/gateway/src/auth.rs`
**Sources Merged**: 3

## Capability Comparison

| Source File | Functions | Preserved | Notes |
|-------------|-----------|-----------|-------|
| auth_v1.rs | 12 | 12 | All migrated |
| auth_v2.rs | 8 | 8 | All migrated |

## Tests

- All existing tests pass: ❌ (not run)
- Total capabilities preserved: 20
- Total capabilities archived: 0
```

### 6. Master Consolidation Index ✅

Maintains a repository-wide index at `.workspace/registry/consolidation_index.json`:

```json
{
  "last_updated": "2025-11-19T12:00:00Z",
  "total_consolidations": 1,
  "entries": [
    {
      "canonical_file": "server/gateway/src/auth.rs",
      "version_count": 1,
      "last_consolidation": "2025-11-19T12:00:00Z",
      "ledger_path": "archive/consolidation/2025/11/.../versions.json"
    }
  ]
}
```

## Workflow Example

### Scenario: Consolidating Duplicate Auth Implementations

1. **Identify duplicates**:
   ```bash
   # Find similar files
   cargo run --bin agent_tools -- grep "authenticate" --path server/
   ```

2. **Run consolidation**:
   ```bash
   cargo run --bin agent_tools -- consolidate \
     --sources server/old/auth_v1.rs,server/experimental/auth_v2.rs \
     --target server/gateway/src/auth.rs \
     --verify \
     --archive \
     --reason "Consolidating authentication implementations"
   ```

3. **Review output**:
   - Merged content in `server/gateway/src/auth.rs`
   - Archives in `archive/consolidation/2025/11/...`
   - Version ledger: `archive/consolidation/2025/11/.../versions.json`
   - Report: `consolidation_report_20251119_120000.md`

4. **Manual verification** (REQUIRED):
   - Review merged content for true duplicates
   - Remove exact duplicate functions
   - Resolve import conflicts
   - Run tests: `cargo test`
   - Format code: `cargo fmt`
   - Review diff: `git diff server/gateway/src/auth.rs`

5. **Commit**:
   ```bash
   git add server/gateway/src/auth.rs archive/ .workspace/
   git commit -m "feat(gateway): consolidate auth implementations

   - Merged auth_v1.rs and auth_v2.rs into canonical auth.rs
   - Preserved all 20 capabilities
   - Archived v1 with SHA-256 verification
   - See consolidation_report_20251119_120000.md"
   ```

## Limitations & Future Work

### Current Limitations

1. **No automatic deduplication**: Merging is simple concatenation; duplicate detection is manual
2. **No conflict resolution**: Import conflicts must be resolved manually
3. **No test execution**: Verification doesn't run `cargo test` (capability checking only)
4. **Rust-only**: Currently supports Rust files only (uses `syn` parser)

### Planned Enhancements

1. **Smart Deduplication Engine**:
   - Semantic similarity detection (AST embeddings, cosine similarity)
   - Automatic detection of functionally equivalent code
   - ML model for pattern recognition

2. **Conflict Resolution UI**:
   - Interactive side-by-side diff viewer
   - Guided merge decisions
   - Rollback of individual resolutions

3. **Test Integration**:
   - Automatic `cargo test` execution
   - Test result inclusion in reports
   - Regression detection

4. **Multi-Language Support**:
   - TypeScript/JavaScript (via `swc` or `babel` AST)
   - Python (via `ast` module)
   - Generic text consolidation fallback

5. **Batch Compression**:
   - Automatic triggering at 100+ files threshold
   - Scheduled compression via CI
   - Integrity verification after compression

6. **Consolidation Dashboard**:
   - UI for tracking metrics
   - Visualization of consolidation graph
   - Alert system for failed consolidations

## Integration with Existing Systems

### Evidence Ledger

All consolidations are logged to `audit/ledger.jsonl`:

```json
{
  "event": "consolidation",
  "timestamp": "2025-11-19T12:00:00Z",
  "canonical": "server/gateway/src/auth.rs",
  "sources": ["old/auth_v1.rs", "experimental/auth_v2.rs"],
  "version": "v1",
  "sha256": "abc123...",
  "capabilities_preserved": 20,
  "capabilities_archived": 0
}
```

### Truth Gate Compliance

Consolidation reports satisfy Truth Gate requirements:
- ✅ All artifacts recorded with SHA-256 hashes
- ✅ Capability verification documented
- ✅ Version ledger maintained
- ✅ Rollback procedure defined
- ✅ Evidence ledger updated

### CRC Integration

Fork consolidation flows through:
1. CRC drop-in (`crc/drop-in/incoming/forks/`)
2. Consolidation tool (this)
3. Production (`server/`, `core/`, etc.)

See `AGENT.md` Section 12 for CRC-specific handling.

## Troubleshooting

### "Failed to parse Rust file"

**Cause**: Source file has syntax errors
**Fix**: Run `cargo check` on source file first

### "Capability preservation < 100%"

**Cause**: Merged file missing some functions from sources
**Fix**: Manual review required; check for:
- Namespace conflicts (same function name, different signature)
- Private functions (not extracted by AST visitor)
- Conditional compilation (`#[cfg(...)]`)

### "Archive directory permission denied"

**Cause**: Insufficient permissions to create `archive/` directory
**Fix**: Ensure write permissions: `chmod +w archive/`

## Related Documentation

- **Policy**: `AGENT.md` Section 12 (File Consolidation Protocol)
- **API Reference**: `server/tools_agent/src/consolidation.rs`
- **CLI Binary**: `server/tools_agent/src/bin/agent_tools.rs`
- **Index Schema**: `.workspace/registry/consolidation_index.json`

## Testing

```bash
# Build
cargo build -p noa-tools-agent

# Run tests (when implemented)
cargo test -p noa-tools-agent consolidation

# Lint
cargo clippy -p noa-tools-agent

# Format
cargo fmt -p noa-tools-agent
```

## Support

For issues or questions:
1. Check `AGENT.md` Section 12 for policy guidance
2. Review consolidation reports for error details
3. Check `.workspace/registry/consolidation_index.json` for historical data
4. Open issue with consolidation report attached

---

**Version**: 1.0.0  
**Last Updated**: 2025-11-19  
**Status**: Production Ready (manual review required)
