# NOA Ark OS Unified CLI Tooling Roadmap

This roadmap describes how to evolve the NOA Ark OS tooling into a single, MCP-backed CLI stack that serves every provider workflow (Codex, Copilot, Claude, etc.) with feature parity to VS Code and its most-used extensions, without relying on fragile symlinks. It consolidates the existing surfaces in `apps/cli`, `server/tools_agent`, `tools/commit_copilot`, `server/tools`, `server/mcp`, and the agents that depend on them.

## 1. Objectives & Success Criteria
- **Provider ubiquity** – every provider (Codex, Copilot, Claude, others) runs the same tool catalog via MCP + `noa` CLI without IDE-specific dependencies.
- **VS Code parity** – cover the core feature buckets exposed by VS Code + key extensions: workspace navigation, editing, SCM, diagnostics/tests, debugging, terminals/tasks, AI assistance, collaboration, observability.
- **Better-than-symlink distribution** – publish tooling as crates/binaries/npm packages so each consumer depends on versioned artifacts instead of local filesystem linking.
- **Server/mcp as hub** – `server/mcp` becomes the single transport that routes CLI functionality into IDEs, terminals, and agents.
- **Tool value amplification** – every tool call is auditable (Truth Gate), composable (ToolRegistry), and works offline-first with opt-in online extensions.

## 2. Current Tooling Surface (gap analysis)
| Surface | Location | Capabilities today | Gaps vs goal |
| --- | --- | --- | --- |
| Commit Copilot CLI | `tools/commit_copilot/cli.ts` | Conventional commit enforcement & suggestions for Git history | Not published to shared registry, Codex can’t call it via MCP, confined to Node runtime |
| NOA CLI | `apps/cli/src/main.rs` | Unified `noa` binary (kernel/world/registry/trust/snapshot/policy/sbom/profile/agent/pipeline, evidence ledger, registry queries) | Lacks ergonomic per-provider commands, no surfacing of workspace ops (read/edit/search), not exposed over MCP |
| Agent Tools | `server/tools_agent/src/bin/agent_tools.rs` + HTTP server | File IO, glob/grep/semantic search, todo mgmt, archival, consolidation, command execution | Separate binary/API from `noa`, duplicate capability schemas, providers must know multiple entrypoints |
| Portable env scripts | `server/tools/*.sh|ps1` | Bootstraps cargo/node/caddy/rustup portable installs | Not orchestrated via CLI, no status reporting, no Windows/macOS parity plan |
| MCP transport | `server/mcp/src/main.rs` | Minimal JSON-RPC handler serving `noa.*` tools through `ToolClient` | Tool catalog hardcoded, no discovery from registry, doesn’t multiplex multiple binaries, not yet linked to agents |
| Specialist agents | `agents/src/implementations/specialist/code_generation.rs` (rep of others) | Agents expect tool metadata + capabilities but don’t receive CLI feature set programmatically | No shared ToolRegistry ingestion, limited template/tool awareness |

## 3. Provider & VS Code Feature Coverage Targets
1. **Workspace Awareness** – list/search/glob/semantic, symbol graph, file tree (VS Code explorer parity).
2. **Editing & Refactoring** – read/apply patch/edit, formatting, rename, code actions.
3. **Diagnostics & Testing** – run tests, lint, coverage, problem matcher output.
4. **Execution & Debug** – task runner, command palette equivalents, debug session attach (headless).
5. **SCM & Release** – git status/stage/commit (leveraging commit-copilot), changelog automation.
6. **Environments** – portable toolchain activation, dependency bootstrap, remote workspace sync.
7. **AI & Agents** – MCP tool registry, prompt templates, code generation hooks (specialist agents).
8. **Collaboration & Observability** – evidence ledger, telemetry, workflow approvals, todo/state sync.

## 4. Target Architecture
### 4.1 Layered components
1. **Tool Implementations** (Rust/Node/Python):
   - Keep existing binaries (`agent-tools`, `commit-copilot`, `noa`) but refactor shared logic into libraries (`noa-tools-core`, `commit-lint-kit`) so all consumers link dependency versions instead of symlinks.
2. **Tool Registry & Packaging**:
   - Extend `noa_plugin_sdk::ToolRegistry` to auto-ingest descriptors from each binary package.
   - Publish crates (Rust), npm packages (Node), and PyPI modules (portable env orchestrators) with semantic versions.
3. **MCP Hub (`server/mcp`)**:
   - Dynamically load tool descriptors from registry files (`registry/tools.registry.json`) or `apps/cli`.
   - Proxy commands to the right runtime (Rust CLI via `noa` binary, Node CLI via `node-portable`, shell scripts) while enforcing capability tokens.
4. **Unified CLI (`apps/cli`)**:
   - Acts as the local authority: exposes subcommands, but also runs as headless service for providers that can’t run MCP (e.g., GitHub Copilot CLI contexts).
5. **Provider Adapters**:
   - Thin wrappers for Codex (Codex CLI harness), Copilot (GH CLI plugin), VS Code extension, terminal experience—each only needs MCP endpoint + optional `noa` binary.

### 4.2 Distribution (no symlinks)
- **Crate workspace** – publish `noa-cli`, `noa-tools-agent`, `noa-mcp` to an internal registry; providers pin versions via `Cargo.lock`.
- **Binary release pipeline** – reuse `server/tools` portable installers to ship signed binaries (`noa`, `agent-tools`, `commit-copilot`) for Linux/macOS/Windows.
- **Package indexes** – `commit-copilot` on npm, `dev_env_cli.py` packaged via pipx; reference from `apps/cli` help output.
- **Tool catalog manifest** – single `registry/tooling.catalog.json` lists every tool, version, binary path, MCP name, capabilities. `server/mcp` consumes manifest; IDE extensions fetch the same manifest over HTTP.

## 5. Roadmap & Milestones
### Phase 0 – Inventory & alignment (1 sprint)
- Normalize documentation: add CLI inventory to `server/tools_agent/README.md`, `apps/cli` docs, `CODEX.md`.
- Extract tool descriptors from `agent-tools` and `commit-copilot` into `noa_plugin_sdk::ToolDescriptor`.
- Ensure `apps/cli` can call `agent-tools` subcommands (bridge command).

### Phase 1 – Unified MCP backbone (2 sprints)
- Enhance `server/mcp`:
  - Load tool list from manifest generated during build.
  - Forward `call_tool` to corresponding binary via gRPC/CLI spawn.
  - Add auth/capability metadata for each provider persona.
- Embed MCP client inside `apps/cli` so `noa agent` subcommands can instrument tools consistently.
- Provide provider-specific startup scripts that launch `server/mcp` + register with Codex/Copilot adapters.

### Phase 2 – Feature parity & packaging (3–4 sprints)
- **Workspace ops**: upgrade `agent-tools` to support structured AST queries, symbol graph, batched edits; expose via MCP.
- **Diagnostics/tests**: wrap `cargo test`, `npm test`, custom scripts inside `noa pipeline` commands with problem matcher output for IDE surfaces.
- **SCM**: integrate `commit-copilot` as `noa scm commit --assist`, expose JSON API for MCP tool `noa.commit`.
- **Env bootstrap**: turn `server/tools/*.sh` into `noa env install/activate` commands with status checks.
- **VS Code parity**: implement debugging hooks (DAP over CLI), task runner mapping, extension-equivalent commands (e.g., todo tree, outline).

### Phase 3 – Provider experience & value (ongoing)
- Telemetry + evidence ledger streaming through MCP so providers see the same workflow events as `noa`.
- Tool marketplace: allow providers to register custom tools via `registry` commands, optionally signed.
- Offline/online policy gating (Truth Gate) integrated so each tool exposes capability flags; agents choose allowed set automatically.
- Performance work: caching for search/semantic calls, incremental state watchers, streaming outputs for long runs.

## 6. Implementation Considerations
- **Multi-runtime orchestration** – standardize on JSON-over-stdio adapters so Rust/Node/Python tools can all be proxied uniformly.
- **Security** – extend `noa_tools_agent` archival & audit logic to every write-capable tool invoked through MCP; integrate into `AGENT.md` guardrails.
- **Testing** – add cross-tool integration tests under `apps/cli/tests` that start `server/mcp`, invoke representative commands, and validate that both Codex and Copilot adapters can call them.
- **Agent collaboration** – expose tool metadata to specialist agents (e.g., `code_generation.rs`) so they request capabilities programmatically rather than through hardcoded lists.
- **Documentation** – update `Makefile` targets to build all CLIs, and add a “Tooling quick start” pointing users to a single activation command.

## 7. Immediate Next Steps
1. Author `registry/tooling.catalog.json` schema + generator (probably in `apps/cli/build.rs`).
2. Modify `server/mcp` to load catalog and proxy `noa.*` calls dynamically.
3. Wrap `agent-tools` commands as hidden subcommands inside `apps/cli` to provide one-binary UX for Codex.
4. Publish `commit-copilot` CLI to npm (internal) and document invocation via MCP tool `noa.commit`.
5. Draft provider adapter specifications so Copilot/Codex/Claude plugins share the same bootstrap instructions.

