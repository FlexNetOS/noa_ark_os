# NOA ARK OS - Portable Development Tools

## 🧭 CLI Overview

The workspace ships with `server/tools/dev_env_cli.py`, a cross-platform utility that replaces
editor-specific `.vscode` profiles. Run it from any terminal to inspect and activate the correct
Rust toolchain configuration:

```bash
python server/tools/dev_env_cli.py summary
```

Use `--format json` when scripting or integrating with CI.

Need to temporarily restore archived IDE assets? Use the CLI `archives`
commands to list bundles and rebuild the legacy `.devcontainer/` or `.vscode/`
packages:

```bash
python server/tools/dev_env_cli.py archives list
python server/tools/dev_env_cli.py archives restore --bundle devcontainer --mode extract --output .devcontainer
python server/tools/dev_env_cli.py archives restore --bundle vscode --mode extract --output .vscode
```

The restore commands write the files to `./.devcontainer` or `./.vscode` (or
any custom path) so you can opt-in to the legacy experience before removing it
again.

## 🌐 Multi-Platform Support

This workspace supports development on:
- **Windows (PowerShell)** - Portable Cargo installation
- **WSL (Ubuntu)** - Native Linux Rust or Windows portable
- **Linux** - System Rust installation

**See [MULTI_PLATFORM.md](MULTI_PLATFORM.md) for detailed multi-platform setup guide.**

---

## 🚀 Quick Start (Windows PowerShell)

### First Time Setup

```powershell
# Run setup script (one-time)
.\server\tools\setup-portable-cargo.ps1
```

This will:
- Download Rust toolchain (~150 MB)
- Install to `server/tools/cargo-portable/`
- Create activation script

### Every Time You Develop

**IMPORTANT**: Use PowerShell (not WSL/bash) for the portable Cargo installation.

```powershell
# 1. Review activation steps (optional)
python server/tools/dev_env_cli.py activate --platform windows

# 2. Navigate to workspace
cd D:\dev\workspaces\noa_ark_os

# 3. Activate portable cargo
.\server\tools\activate-cargo.ps1

# 4. Navigate to project
cd crc

# 5. Build
cargo build

# 6. Run
cargo run --bin crc-server
```

---

## 🐧 Quick Start (WSL/Ubuntu)

### Option 1: Native Linux Rust (Recommended)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Activate
source $HOME/.cargo/env

# Verify
cargo --version
```

### Option 2: Use Windows Portable Cargo

```bash
# Inspect CLI guidance (optional)
python server/tools/dev_env_cli.py activate --platform wsl

# Run activation script
source ./server/tools/activate-cargo.sh

# Follow prompts to use Windows Cargo (slower)
```

**See [MULTI_PLATFORM.md](MULTI_PLATFORM.md) for detailed WSL/Ubuntu setup.**

---

## 📂 Directory Structure

```
server/tools/
├── cargo-portable/          # Portable Cargo installation
│   ├── bin/
│   │   ├── cargo.exe
│   │   ├── rustc.exe
│   │   └── rustfmt.exe
│   └── lib/
├── rustup-portable/         # Rustup data
├── node-portable/           # Portable Node 20 + pnpm 8 toolchain (gitignored)
│   ├── artifacts/           # Downloaded archives
│   ├── current/             # Symlink to extracted Node release
│   └── corepack/            # pnpm asset cache
├── node-portable.manifest.json # Tracked manifest copy (versions + hashes)
├── setup-portable-cargo.ps1 # One-time setup script
├── setup-portable-node.ps1  # One-time Node setup (Windows)
├── setup-portable-node.sh   # One-time Node setup (Linux/WSL/macOS)
└── activate-cargo.ps1       # Activation script (run every session)
	activate-node.ps1        # Node activation (powershell)
	activate-node.sh         # Node activation (bash/zsh)
```

---

## 🟢 Portable Node + pnpm (HT-01)

Node.js v20.19.5 and pnpm v8.15.4 are mirrored locally so `pnpm`, `node`, and
`npm` executions never depend on host-level installations.

### Install / Refresh

```bash
# Linux/WSL/macOS
./server/tools/setup-portable-node.sh
```

```powershell
# Windows PowerShell
./server/tools/setup-portable-node.ps1
```

Both scripts download the official Node + pnpm release artifacts, place them in
`server/tools/node-portable/`, and generate `manifest.json` with SHA-256 hashes
recorded for evidence tracking.

### Activate per Shell Session

```bash
source ./server/tools/activate-node.sh
```

```powershell
./server/tools/activate-node.ps1
```

Activation prepends `server/tools/node-portable/current/bin` to `PATH` and
exports `NOA_NODE_HOME` / `COREPACK_HOME` so all Make targets resolve tooling
from the hermetic bundle. Combine this with the cargo activation scripts before
invoking `make`, `pnpm`, or the pipeline tasks.

---

## 🌐 Portable Caddy reverse proxy

The reverse proxy follows the same hermetic toolchain approach. The scripts
under `server/tools/` download the official Caddy release for your platform,
extract it into `server/tools/caddy-portable/`, and refresh
`caddy-portable.manifest.json`.

```bash
# Linux/macOS/WSL
./server/tools/setup-portable-caddy.sh
source ./server/tools/activate-caddy.sh

# Windows PowerShell
./server/tools/setup-portable-caddy.ps1
./server/tools/activate-caddy.ps1
```

Activation exports `NOA_CADDY_HOME` and prepends the portable binary to `PATH`
so `caddy validate`, `caddy run`, and `noa caddy ...` always reference the same
audited artifact across CI and developer hosts.

---

## 🔧 Manual Installation Steps

If you prefer manual setup:

### 1. Set Variables
```powershell
$CARGO_HOME = "D:\dev\workspaces\noa_ark_os\server\tools\cargo-portable"
$RUSTUP_HOME = "D:\dev\workspaces\noa_ark_os\server\tools\rustup-portable"
$env:CARGO_HOME = $CARGO_HOME
$env:RUSTUP_HOME = $RUSTUP_HOME
```

### 2. Download Rustup
```powershell
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
```

### 3. Install
```powershell
.\rustup-init.exe --default-toolchain stable --profile minimal --no-modify-path -y
```

### 4. Cleanup
```powershell
Remove-Item rustup-init.exe
```

---

## ✅ Verification

Use the CLI doctor command for a quick sanity check before building:

```bash
python server/tools/dev_env_cli.py doctor
```

You can still call the binaries directly:

```powershell
cargo --version
rustc --version
where.exe cargo
```

---

## 📦 Size

- **Minimal profile**: ~150-200 MB
- **Full profile** (with docs): ~500+ MB

We use minimal profile to save space.

---

## 🔄 Updating Rust

```powershell
# Activate cargo first
.\server\tools\activate-cargo.ps1

# Then update
rustup update stable

# Refresh CLI summary (optional)
python server/tools/dev_env_cli.py summary
```

---

## ❓ Troubleshooting

### "Cargo not found"

1. Run `python server/tools/dev_env_cli.py doctor` to confirm the activation scripts exist.
2. Make sure you ran `activate-cargo.ps1` in current session.
3. Check if `cargo.exe` exists at: `server\tools\cargo-portable\bin\cargo.exe`.
4. If missing, run `setup-portable-cargo.ps1` again.

### "Permission denied"

Run PowerShell as Administrator or:
```powershell
Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned
```

### Need to reinstall

```powershell
# Delete directories
Remove-Item -Recurse -Force server\tools\cargo-portable
Remove-Item -Recurse -Force server\tools\rustup-portable

# Run setup again
.\server\tools\setup-portable-cargo.ps1

# Verify with the CLI
python server/tools/dev_env_cli.py doctor
```

---

## 🎯 Why Portable?

1. ✅ **Self-contained** - Everything in workspace
2. ✅ **No system PATH pollution** - Clean environment
3. ✅ **Version locked** - Consistent across team
4. ✅ **Easy cleanup** - Just delete folder
5. ✅ **Multiple workspaces** - Different versions possible

---

## 📝 Notes

- Installation location is in `.gitignore` (won't be committed)
- Scripts ARE committed for easy setup by other developers
- Toolchain version: Stable channel (auto-updates to latest stable)

---

**Last Updated**: 2024-01-15
**Rust Version**: 1.75+ (stable)
