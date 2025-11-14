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
scripts/dev-env.ps1 run "cargo check"

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
