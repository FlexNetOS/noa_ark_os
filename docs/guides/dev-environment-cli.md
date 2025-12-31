# CLI Developer Environment Bootstrap

This guide describes how to provision the NOA ARK OS development container without relying on VS Code's Dev Containers extension. The CLI workflow mirrors the tooling, editor extensions, and post-create behavior defined in `.devcontainer/devcontainer.json` so any editor can share the same environment.

## Manifest-driven configuration

The manifest at `tools/devshell/dev-env.manifest.toml` is the single source of truth for the container image:

```toml
[image]
name = "mcr.microsoft.com/devcontainers/base:ubuntu-24.04"

[user]
name = "vscode"
uid = 1000
gid = 1000

[node]
version = "lts"
enable_pnpm = true

[rust]
profile = "minimal"
toolchain = "stable"

[packages]
apt = [
  "build-essential",
  "ca-certificates",
  "curl",
  "git"
]

[editor]
npm_package_manager = "pnpm"
terminal_profile = "bash"
extensions = [
  "rust-lang.rust-analyzer",
  "tamasfe.even-better-toml",
  "esbenp.prettier-vscode",
  "dbaeumer.vscode-eslint"
]

[rust_analyzer]
check_on_save = true
check_command = "check"
all_features = true

[post_create]
script = ".devcontainer/post-create.sh"
```

Editors can read this TOML file to configure extensions, default npm tooling, and language-server options. The CLI scripts convert the manifest into a Docker/Podman compatible build recipe.

## Shell workflow (Linux/macOS)

Use `scripts/dev-env.sh` to build and run the container:

```bash
# Build or rebuild the dev image
scripts/dev-env.sh build

# Launch an interactive shell in the container
scripts/dev-env.sh run

# Run a one-off command inside the container
scripts/dev-env.sh run pnpm install

# Validate tooling availability (node, pnpm, cargo, rustc)
scripts/dev-env.sh smoke
```

Set `CONTAINER_RUNTIME=podman` (or `docker`) if auto-detection picks the wrong runtime. Override the image tag with `DEV_ENV_IMAGE_TAG` when testing alternate versions.
The container entrypoint executes `.devcontainer/post-create.sh` on startup to mirror the devcontainer lifecycle; skip it by exporting `DEV_ENV_SKIP_POST_CREATE=1` when you need faster, dependency-free runs.

## PowerShell workflow (Windows)

For Windows hosts the PowerShell script provides the same commands:

```powershell
# Build the dev image
scripts/dev-env.ps1 build

# Open an interactive shell in the container
scripts/dev-env.ps1 run

# Execute a single command
scripts/dev-env.ps1 run cargo check

# Smoke test container tooling
scripts/dev-env.ps1 smoke
```

The PowerShell script uses the same manifest and runtime detection logic. When running on Windows Subsystem for Linux (WSL), prefer the Bash script.
Set `$env:DEV_ENV_SKIP_POST_CREATE = "1"` before calling `scripts/dev-env.ps1 run` if you need to bypass the post-create bootstrap.

## Visual Studio workflow without `.sln`

Visual Studio users can connect to the container through the CLI workflow:

1. Build and run the container with `scripts/dev-env.ps1 run`. This opens a shell with Rust, Node.js LTS, pnpm, and git pre-installed.
2. Launch Visual Studio with the "Open Folder" workflow and target the repository directory mounted at `/workspace`. Visual Studio's `devcontainers` support is not required; the environment is already provisioned.
3. Use the integrated terminal to run Cargo and pnpm commands directly: `cargo check`, `cargo test`, `pnpm install`, and `pnpm test` all operate inside the container.
4. When debugging native services, attach to processes running inside the container via Visual Studio's remote debugging support (SSH target `localhost` on the forwarded port).

The legacy `NoaArkOS.sln` remains available for historical reference, but the CLI tooling replaces its build orchestration. Once critical project types (Rust crates, pnpm workspaces, and Dockerized services) demonstrate parity through the CLI workflow and automated smoke tests, we will archive the solution file and document the retirement in the Evidence Ledger.

## Preventing environment drift

The repository provides a smoke test wrapper that CI and local contributors can call:

```bash
cicd/scripts/dev-env-smoke.sh
```

The script reuses the CLI bootstrap to ensure the container builds and that `node`, `pnpm`, `rustc`, and `cargo` are present. CI runners without a container runtime exit early with a warning, preventing false failures while still highlighting missing prerequisites locally.

## Updating tooling

When you change dependencies (Rust toolchain, Node.js version, VS Code extensions, etc.), edit `tools/devshell/dev-env.manifest.toml`. Re-run `scripts/dev-env.sh build` (or the PowerShell equivalent) to regenerate the image, and add the change to PRs so downstream users keep pace with manifest updates.

## Codex CLI + MCP bootstrap

Codex ships with the container so every terminal session can launch the MCP-enabled workflow without manual npm globals:

- `.devcontainer/post-create.sh` now runs `scripts/codex-bootstrap.sh`, which downloads the latest Codex binary for the host platform, installs `noa-mcp-server` via `cargo install`, and appends a `[mcp_servers.noa]` entry to `${CODEX_HOME:-~/.codex}/config.toml` pointing at the repository root.
- Run the script manually when working outside the container:

  ```bash
  scripts/codex-bootstrap.sh
  ```

  This ensures `codex`, `noa-mcp-server`, and the MCP config exist locally, and that Codex knows to launch the server from `/home/<user>/.local/bin/noa-mcp-server` with `cwd` set to your repo root.

Use `codex mcp list` to confirm the `noa` entry is registered, then start `noa_tools_agent` (for example with `cargo run -p noa_tools_agent`) before running `codex` so the MCP bridge can forward tool invocations.

### Host-specific tips

- **macOS**: Run the script from Terminal/iTerm after installing the Xcode Command Line Tools (`xcode-select --install`). The script auto-detects the proper Codex artifact and uses the system `python3`. When launching from zsh, make sure `$HOME/.local/bin` is on the PATH (`echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc`).
- **Windows**: Execute the script from Git Bash (ships with Git for Windows) or from WSL. The bootstrapper now auto-detects `python3` vs `python` via the `PYTHON_BIN` override, so a standard `winget install Python.Python.3.11` is sufficient. Add `%USERPROFILE%\.local\bin` to your PATH (`setx PATH "%USERPROFILE%\.local\bin;%PATH%"`) so both `codex` and `noa-mcp-server` are discoverable in PowerShell sessions.
- **Remote tool targets**: When you need Codex to talk to a shared `noa_tools_agent`, export `NOA_TOOLS_SERVER_URL=https://<remote-host>:<port>/` before running the bootstrap script so the `[mcp_servers.noa]` stanza records the remote gateway URL instead of `http://127.0.0.1:8910/`.

## MCP health check

CI and local preflight runs can validate that Codex still reaches `noa_tools_agent` through the MCP shim:

```bash
cicd/scripts/codex-mcp-health.sh
```

The script performs three steps:

1. Runs `codex mcp list --json` and asserts the `noa` entry exists in `~/.codex/config.toml`.
2. Boots `noa_tools_agent` (bound to `127.0.0.1:8910`) and verifies the HTTP surface responds to a `list_files` request.
3. Launches the stdio-based `noa-mcp-server`, sends `initialize`, `list_tools`, and `call_tool` (for `noa.list_files`) requests over JSON-RPC, and fails fast if the result payload changes.

Wire this script into CI or local commit hooks to detect regressions in `noa_tools_agent` or the MCP adapter before Codex users encounter them.

### Remote + multi-OS validation

- **Remote-only probes**: Set `NOA_MCP_REMOTE_ONLY=1` and `NOA_TOOLS_SERVER_URL` to a reachable agent endpoint when you want to keep the health script from spawning a local `noa_tools_agent`. In that mode you are responsible for starting the remote agent (for local simulations run `NOA_WORKSPACE_ROOT=$PWD NOA_TOOLS_ADDRESS=127.0.0.1:8910 cargo run -p noa_tools_agent --bin noa_tools_agent --quiet &` before invoking the script).
- **Log capture**: Both local and remote runs can tee their output into `logs/codex-mcp-health/*.log` for evidence. The script already waits for `/list_files` to respond before exercising the MCP RPC surface, which makes it safe to run on CI runners where startup latency varies.
- **Fork-only GitHub Actions guard**: The `Fork MCP Health Guard` workflow (`.github/workflows/noa-mcp-health.yml`) runs `scripts/codex-bootstrap.sh` and `cicd/scripts/codex-mcp-health.sh` on `ubuntu-latest`, `macos-latest`, and `windows-latest` for every forked PR. It boots a background `noa_tools_agent`, forces remote-only mode, and surfaces regressions before the branch merges upstream.
