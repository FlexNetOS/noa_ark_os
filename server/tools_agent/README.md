# NOA Tools Agent

Offline-first tool dispatcher and HTTP server providing workspace operations without relying on VS Code. All policy and guardrails defer to `AGENT.md`.

## Capabilities

- File read/write with automatic archival before modification.
- Patch application (line-based hunks, overlap detection).
- Grep regex search (`/grep_search` and CLI `agent-tools grep`).
- Glob file search (`/glob_search`).
- Naive semantic search (substring scoring) (`/semantic_search`).
- Todo list management persisted in `.workspace/todo/todo_list.json`.
- Command execution with allowlist and timeout.
- Manual archival endpoint `/archive_file` and CLI `agent-tools archive`.
- Workspace build/test endpoints.

## Policy Pointer

> Single authoritative source: `AGENT.md` (heal-not-harm, archive-first, offline-first, gateway-managed).

## HTTP Endpoints

| Path | Description |
|------|-------------|
| POST /run_command | Execute allowlisted command |
| POST /edit_file | Write file (archival if exists) |
| POST /apply_patch | Apply hunks (archival) |
| POST /list_files | List directory entries |
| POST /read_file | Read file contents |
| POST /build_workspace | `cargo build --workspace` |
| POST /run_tests | `cargo test --workspace` |
| POST /grep_search | Regex search across files |
| POST /glob_search | Glob match enumeration |
| POST /semantic_search | Naive substring search with scoring |
| POST /todo | Todo operations (list/add/update/remove) |
| POST /archive_file | Archive a file manually |

## CLI Usage

Activate portable toolchains first.

```bash
cd /home/noa/dev/workspace/noa_ark_os/noa_ark_os
source ./server/tools/activate-cargo-wsl.sh
cargo run -p noa-tools-agent --bin agent-tools -- read --path server/gateway/src/lib.rs
cargo run -p noa-tools-agent --bin agent-tools -- grep --pattern 'RoutingError' --path server/gateway/src
cargo run -p noa-tools-agent --bin agent-tools -- glob --pattern 'server/**/*.rs'
cargo run -p noa-tools-agent --bin agent-tools -- semantic --query 'Protocol::Grpc' --path server
cargo run -p noa-tools-agent --bin agent-tools -- todo --op add --title "Investigate redis upgrade" --status not-started
cargo run -p noa-tools-agent --bin agent-tools -- archive --path server/gateway/src/lib.rs
```

## Archival Format

- Path: `archive/YYYY/MM/<relative_path with slashes replaced>.tar.zst`
- Includes original file content compressed with zstd.
- Ledger entries appended to `docs/evidence_ledger.tools.log`.

## Notes

- Online-only features (e.g., GitHub API) intentionally omitted until `ONLINE_*` flags are defined.
- Semantic search is naive; replace with vector index when retrieval subsystem is restored.
- All modifications captured in ledger for audit and Truth Gate verification.

## Future Work

- GitHub integration behind capability tokens.
- Structured diff summarization endpoint.
- Incremental symbol graph updates.
- Rich semantic search with embeddings.
