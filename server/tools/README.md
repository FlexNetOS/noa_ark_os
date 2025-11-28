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
- **Windows (PowerShell)** - Native portable workflow (`setup-portable-cargo.ps1` / `activate-cargo.ps1`)
- **Linux + macOS shells** - POSIX portable workflow (`setup-portable-cargo.sh` / `activate-cargo.sh`)
- **WSL** - Use the Linux/macOS scripts directly so the toolchain lives inside the repo

Every path stays inside `server/tools/` so builds remain reproducible regardless of the host platform.
**See [MULTI_PLATFORM.md](MULTI_PLATFORM.md) for deeper platform notes.**

---

## 🚀 Portable Cargo Quick Start (All Platforms)

### 1. One-Time Setup

| Platform | Command |
| --- | --- |
| Windows PowerShell | `powershell -ExecutionPolicy Bypass -File .\server\tools\setup-portable-cargo.ps1` |
| Linux / macOS shells | `bash ./server/tools/setup-portable-cargo.sh` |

The setup script:
- Downloads the latest stable Rust toolchain (~150–200 MB)
- Installs it into `server/tools/cargo-portable/`
- Stores rustup metadata under `server/tools/rustup-portable/`

### 2. Activate Per-Shell Session

| Platform | Command |
| --- | --- |
| Windows PowerShell | `.\server\tools\activate-cargo.ps1` |
| Linux / macOS shells | `source ./server/tools/activate-cargo.sh` |

Activation exports `CARGO_HOME`, `RUSTUP_HOME`, and updates `PATH` for the current shell. Re-run it for every new terminal session or integrate it with your shell profile if desired.

### 3. Build/Run

```bash
# From workspace root after activation
cargo build --workspace
cargo test --workspace
```

> 💡 Use `python server/tools/dev_env_cli.py activate --platform <windows|linux|macos>` to review which script to call plus environment tips.

---

## 🐧 Linux + macOS Notes

- Required packages: `curl`, `python3`, `pkg-config`, `libssl-dev`/`openssl` headers, a C toolchain (`build-essential` or Xcode CLT), and `xz`/`tar` for unpacking.
- Always `source` the activation script (`source ./server/tools/activate-cargo.sh`) so it can export environment variables into your current shell. Running it with `bash ./activate-cargo.sh` will spawn a subshell and the variables will be lost.
- The POSIX setup script accepts `--workspace <path>` and `--force` for custom layouts and rebuilds.
- If you keep multiple worktrees, set `NOA_CARGO_HOME` / `NOA_RUSTUP_HOME` before running the setup script to pin the installation to a shared location.

---

## 💻 Windows Tips

- Stay inside PowerShell when using the portable installation. Git Bash or WSL shells cannot invoke the Windows binaries.
- Set `NOA_CARGO_ACTIVATE_SILENT=1` before calling `activate-cargo.ps1` to suppress logs in automated scripts.
- Run `python server/tools/dev_env_cli.py doctor` if activation does not update the PATH; it validates the stored state JSON/YAML in `tools/devshell/state/`.

---

## 🔁 WSL Interop

When using WSL, prefer the Linux/macOS scripts so the toolchain lives directly under the Linux filesystem (`/home/<user>/.../server/tools`). This avoids Windows path translation penalties and ensures symbolic links behave correctly.

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

### "Cargo not found" (Linux/macOS)

1. Confirm you **sourced** the activation script: `source ./server/tools/activate-cargo.sh`.
2. Inspect `echo $CARGO_HOME`—it should point at `.../server/tools/cargo-portable`.
3. If the path is blank, rerun `python server/tools/dev_env_cli.py doctor` to see which script to call for your platform.
4. Reinstall with `bash ./server/tools/setup-portable-cargo.sh --force` if `cargo-portable/bin/cargo` is missing.

### "Cargo not found" (Windows)

1. Run `python server/tools/dev_env_cli.py doctor` to confirm the activation scripts exist.
2. Make sure you ran `.\server\tools\activate-cargo.ps1` in the current session.
3. Check `server\tools\cargo-portable\bin\cargo.exe`.
4. Re-run `.\server\tools\setup-portable-cargo.ps1` if the binary is missing.

### "PATH didn't update" on macOS/Linux

Some shells (e.g., `zsh`) require the script to be sourced **after** custom PATH logic in `.zshrc`. Move `source ./server/tools/activate-cargo.sh` to the bottom of your profile or export `NOA_CARGO_ACTIVATE_SILENT=1` and let the CLI add it when running `python server/tools/dev_env_cli.py activate --auto`.

### Windows execution policy errors

```powershell
Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned
```

### Need to reinstall from scratch

```powershell
# Delete directories (Windows)
Remove-Item -Recurse -Force server\tools\cargo-portable
Remove-Item -Recurse -Force server\tools\rustup-portable
.\server\tools\setup-portable-cargo.ps1

```

```bash
# Delete directories (Linux/macOS)
rm -rf server/tools/cargo-portable server/tools/rustup-portable
bash ./server/tools/setup-portable-cargo.sh --force
```

After reinstalling, run `python server/tools/dev_env_cli.py doctor` for a sanity check.

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
