# NOA Agent Tool API

The NOA Agent Tool API standardizes every workspace mutation or inspection so
all human and automated operators use the exact same sandbox. The API is
transport-agnostic (HTTP JSON today, MCP bridge ready) and enforces the
repository guardrails from `AGENT.md`.

## Core Principles

1. **Workspace containment** – every path is resolved relative to the NOA Ark OS
   repository root. Requests that escape the workspace are rejected.
2. **Deletion requires archival** – tools never delete files. Replacements must
   copy the previous version into `archive/YYYY/MM/<path>.tar.zst` before the new
   content lands.
3. **Command allowlist** – only commands defined in `tools/allowed_commands.toml`
   may be executed.
4. **Evidence logging** – each request is appended to
   `docs/evidence_ledger.tools.log` so every action is auditable.

## Request/Response Schemas

All requests use UTF-8 JSON payloads. Responses include `success` plus
operation-specific data. Errors return `success: false` with a `message`.

### `run_command`

Execute an allowlisted shell command inside the workspace.

**Request**

```json
{
  "command": "cargo",
  "args": ["build", "--workspace"],
  "environment": {
    "RUST_LOG": "info"
  },
  "timeout_seconds": 600
}
```

**Response**

```json
{
  "success": true,
  "exit_code": 0,
  "stdout": "…",
  "stderr": "…"
}
```

Constraints: `command` must be a bare executable (no slashes). The concatenated
`command + args` must match a record in `tools/allowed_commands.toml`. Commands
run from the repository root and inherit a minimal environment.

### `edit_file`

Replace the entire contents of a file.

**Request**

```json
{
  "path": "docs/example.md",
  "contents": "# Updated contents\n",
  "create_if_missing": true
}
```

**Response**

```json
{ "success": true, "bytes_written": 23 }
```

Constraints: `path` must stay inside the workspace. When overwriting an existing
file, callers must archive the previous version before committing per
`AGENT.md`.

### `apply_patch`

Apply structured, line-oriented hunks to a file without rewriting the entire
asset.

**Request**

```json
{
  "path": "src/lib.rs",
  "hunks": [
    {
      "start_line": 10,
      "end_line": 12,
      "replacement": "fn updated() {\n    println!(\"ready\");\n}\n"
    }
  ]
}
```

*`start_line` and `end_line` are 1-indexed inclusive ranges. Hunks must be
ordered and non-overlapping.*

**Response**

```json
{ "success": true, "hunks_applied": 1 }
```

### `list_files`

Enumerate entries under a directory.

**Request**

```json
{ "path": "server/tools_agent" }
```

**Response**

```json
{
  "success": true,
  "entries": [
    { "path": "server/tools_agent/Cargo.toml", "kind": "file", "size": 512 },
    { "path": "server/tools_agent/src", "kind": "dir", "size": 0 }
  ]
}
```

### `read_file`

Read a UTF-8 file from the workspace.

**Request**

```json
{ "path": "README.md" }
```

**Response**

```json
{ "success": true, "path": "README.md", "contents": "…" }
```

### `run_tests`

Shortcut for the canonical workspace test command
(`cargo test --workspace`).

**Request**

```json
{}
```

**Response**

Same shape as `run_command`.

### `build_workspace`

Shortcut for `cargo build --workspace`.

**Request/Response** – identical to `run_tests`.

## Transport Bindings

- **HTTP** – `POST /run_command`, `/edit_file`, `/apply_patch`, `/list_files`,
  `/read_file`, `/run_tests`, `/build_workspace`. Bodies contain the JSON
  payloads defined above.
- **MCP** – `server/mcp` exposes the same tools over a newline-delimited
  JSON-RPC bridge (see `server/mcp/README.md`).

## Logging

Each request appends a JSON line to `docs/evidence_ledger.tools.log`:

```json
{"timestamp":"2024-05-19T00:00:00Z","action":"run_command","path":"","details":{"command":"cargo","args":["build","--workspace"],"status":0}}
```

Consumers must keep this ledger in source control to preserve provenance.
