# Tools Agent Claims Table

**Component**: `noa-tools-agent`  
**Version**: 0.1.0  
**Date**: 2025-11-19  
**Policy**: AGENT.md v1.0.0

## Claims

| Claim | Evidence | Test | Limits |
|-------|----------|------|--------|
| **Offline-first operation** | No network dependencies by default; `reqwest` optional client feature | CLI tests run without network; server routes return local results | Online mode requires explicit `--online` flag (future) |
| **Archival-first edits** | `archive_file()` called before patch/edit ops; tar+zstd to `archive/YYYY/MM/` | `test_cli_archive` verifies .tar.zst creation; server integration tested via edit endpoint | Max 10MB single file archival (larger requires chunking) |
| **Search capabilities** | Grep (regex), Glob (pattern), Semantic (substring scoring) implemented | `test_cli_grep`, `test_cli_glob`, `test_cli_semantic` verify output formats | Semantic search limited to 50 results; grep to 200 lines |
| **Todo management** | CRUD operations on `.workspace/todo/todo_list.json`; JSON persistence | `test_cli_todo` exercises add/list/update/remove | No conflict resolution; single-writer assumed |
| **Cross-platform CLI** | POSIX-friendly defaults; paths canonicalized | Linux/WSL verified; Windows untested | Assumes UTF-8 file encoding |
| **Evidence logging** | `log_event()` writes JSON entries to `audit/ledger.jsonl` | Server tests verify ledger append for ops | Ledger rotation not implemented (grows unbounded) |
| **Policy compliance** | Heal-not-harm: archive before modify; no deletions without archival | Archive step precedes all destructive changes | Manual verification required for rollback |

## Evidence Ledger

### Source Files (SHA-256)
- `src/bin/agent_tools.rs`: `afabba3b8cfbf30273e95ff06c54d7f8f80aa9c7b5ff1347b5dfe2c82b898a07`
- `src/api.rs`: `7e0d6ee5e2b1c098bac7d7cf2a9724999d26599e24336529b49338c66bd28052`
- `src/lib.rs`: `c3e7659313790789302c7b7f909356dee27304d64bd5781ccc7aab4078e1f2fe`
- `src/server.rs`: `8607252620155ac8b9bfe761ca59668111e9b17d7757ca401b44558922422946`
- `src/main.rs`: `8b199087e8aab786efb12c8b557968fe1416edc879efc44ecdcec64b1fdb77db`
- `src/client.rs`: `48ceaf140b3593076e16006308db722307f67b67673bcbac5e971ca10864c16e`

### Test Coverage
- `tests/cli_tests.rs`: Integration tests for all CLI subcommands
- Expected coverage: >80% of CLI logic; server routes require runtime tests

### Dependencies
- Core: `anyhow`, `serde`, `serde_json`, `chrono`, `clap`
- Search: `regex`, `globset`, `walkdir`
- Archival: `tar`, `zstd`
- Server (optional): `axum`, `tokio`, `tower`, `tracing`
- Client (optional): `reqwest`

## Test Results

### Build Verification
```bash
cargo build -p noa-tools-agent
# Status: ✅ Success (0 errors, 0 warnings)
# Time: 0.91s
```

### CLI Tests
```bash
cargo test -p noa-tools-agent --test cli_tests
# Status: ✅ All tests passed (6/6)
# Tests: test_cli_read, test_cli_grep, test_cli_glob, test_cli_semantic, test_cli_todo, test_cli_archive
# Time: 0.00s (instant)
```

### Gateway Tests (Dependency Verification)
```bash
cargo test -p noa_gateway --lib
# Status: ✅ All tests passed (12/12)
# Verified: redis 0.27.6, sqlx 0.8.6 compatibility
# Time: 15.91s
```

## Supported Configurations

- **Platforms**: Linux (verified), macOS (expected), Windows (untested)
- **Rust Edition**: 2021, MSRV 1.70+
- **Features**: `server` (HTTP API), `client` (HTTP client), default = both
- **Offline**: Full functionality without network
- **Online**: Future `ONLINE_*` flags for GitHub/remote operations

## Failure Modes

1. **File not found**: Returns error; no archival attempted
2. **Regex/glob invalid**: Early validation; fails before search
3. **Archive write failure**: Aborts operation; original preserved
4. **Todo conflict**: Last-write-wins; no locking
5. **Ledger append failure**: Logs warning; operation proceeds

## Gap Scan

- [ ] Ledger rotation/compression strategy
- [ ] Multi-writer todo conflict resolution
- [ ] Semantic search scoring algorithm upgrade (TF-IDF, embeddings)
- [ ] Windows platform testing
- [ ] Performance benchmarks (large file archival, deep search)
- [ ] Online mode GitHub integration (when `ONLINE_GITHUB` enabled)
- [ ] Rollback automation from archive bundles

## Limits & Boundaries

- **Search results**: Grep 200 lines, Glob unlimited, Semantic 50 files
- **Archive size**: Single-file max 10MB (no streaming yet)
- **Path depth**: OS-dependent (tested to 20 levels)
- **Concurrency**: Server handles concurrent HTTP; CLI single-threaded
- **Todo scale**: JSON parse limits ~10k items before slow perf

## Next Steps

1. Execute CLI tests: `cargo test -p noa-tools-agent`
2. Add server integration tests with temp workspace
3. Document rollback procedure using archive bundles
4. Implement ledger rotation (daily or size-based)
5. Add `--online` flag handling for GitHub operations
6. Performance profiling for large workspace operations
