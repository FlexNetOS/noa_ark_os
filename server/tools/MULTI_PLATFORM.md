# Multi-Platform Cargo & Node Setup Guide

## 🌐 Overview

This workspace supports development on **three platforms**:

1. **Windows (PowerShell)** – Portable Cargo lives in `server/tools/cargo-portable/` and portable Node/pnpm in `server/tools/node-portable/`.
2. **WSL (Windows Subsystem for Linux)** – Can use native Linux Rust, but must still source the hermetic Node bundle for pnpm operations.
3. **Ubuntu/Linux** – Typically uses system Rust plus the shared Node bundle to keep builds offline.

**Rule:** Activate both cargo *and* node scripts before running `make`, `pnpm`, or any CI pipelines so PATH resolves to the hermetic tool bundles and HT‑01 stays satisfied. The exact hashes live in `server/tools/node-portable.manifest.json` for auditability.

All platforms now share a unified command entrypoint via `tools/devshell/portable-cargo.{sh,ps1}`. The wrapper sources the appropriate activator, sets `CARGO_HOME`/`RUSTUP_HOME`, and records the environment status in `tools/devshell/state/cargo-env.{json,yaml}` so other tooling can reuse the configuration without re-running detection.

---

## 🚀 Unified CLI Usage

Use the devshell wrapper instead of invoking `cargo` directly:

```bash
# Linux / WSL
./tools/devshell/portable-cargo.sh <cargo-args>
```

```powershell
# Windows PowerShell
./tools/devshell/portable-cargo.ps1 <cargo-args>
```

The wrapper auto-detects whether the portable toolchain is installed or if the system Rustup should be used, ensuring a consistent experience between terminal sessions and automation (Makefile, CI emulation, etc.).

---

## 🪟 Windows (PowerShell) Setup

### First Time Setup

```powershell
# Run setup script (one-time)
./server/tools/setup-portable-cargo.ps1
```
````powershell
# Run once per machine
./server/tools/setup-portable-cargo.ps1
./server/tools/setup-portable-node.ps1
````

### Every Session

```powershell
# Activate portable Cargo interactively when needed
./server/tools/activate-cargo.ps1

# Preferred: run commands through the wrapper
./tools/devshell/portable-cargo.ps1 --version
./tools/devshell/portable-cargo.ps1 build
./tools/devshell/portable-cargo.ps1 test
```
# Activate portable Cargo
python server/tools/dev_env_cli.py activate --platform windows
.\server\tools\activate-cargo.ps1
````powershell
./server/tools/activate-cargo.ps1
./server/tools/activate-node.ps1

cargo --version
pnpm -v
node -v
````

### Portable Locations
- **Cargo**: `D:\dev\workspaces\noa_ark_os\server\tools\cargo-portable\`
- **Rustup**: `D:\dev\workspaces\noa_ark_os\server\tools\rustup-portable\`
- **Node/pnpm**: `D:\dev\workspaces\noa_ark_os\server\tools\node-portable\`

---

## 🐧 WSL / Ubuntu Setup

### Option 1 – Native Linux Rust (Recommended)

````bash
# One-time install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Activate for current session
source "$HOME/.cargo/env"

# Verify through the wrapper
./tools/devshell/portable-cargo.sh --version
```
# Each session
source "$HOME/.cargo/env"
source ./server/tools/activate-node.sh

cargo --version
pnpm -v
````

### Option 2 – Use Windows Portable Cargo from WSL

```bash
# Optional interactive activation
# Activate Cargo
python server/tools/dev_env_cli.py activate --platform linux
source ./server/tools/activate-cargo.sh

# Preferred: run commands through the wrapper
./tools/devshell/portable-cargo.sh build
./tools/devshell/portable-cargo.sh run
```

### Option 2: Use Windows Portable Cargo from WSL

**Setup:**

```bash
# Set environment variables
python server/tools/dev_env_cli.py activate --platform wsl
````bash
export CARGO_HOME="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable"
export RUSTUP_HOME="/mnt/d/dev/workspaces/noa_ark_os/server/tools/rustup-portable"
export PATH="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable/bin:$PATH"

# Use the wrapper so commands forward to cargo.exe automatically
./tools/devshell/portable-cargo.sh --version
```
source ./server/tools/activate-node.sh
cargo.exe --version
````

**Reminder:** Windows binaries via WSL interop are slower; prefer native Rust where possible, but always reuse the portable Node bundle.

---

## 🧭 CLI Reference

- `python server/tools/dev_env_cli.py summary` – Display workspace paths and profiles.
- `python server/tools/dev_env_cli.py activate --platform <windows|wsl|linux>` – Print activation steps.
- `python server/tools/dev_env_cli.py doctor` – Check required scripts and directories.
- `python server/tools/dev_env_cli.py diagnostics` – Rust-analyzer troubleshooting guidance.
- **PowerShell** – Default for portable toolchains on Windows
- **WSL** – Ubuntu shell
- **Git Bash / CMD** – Optional

Switch via the terminal dropdown or `Ctrl+Shift+P → Terminal: Select Default Profile`.

---

## 📋 Quick Reference by Platform

### PowerShell (Windows)
```powershell
# Quick wrapper usage
./tools/devshell/portable-cargo.ps1 build
./tools/devshell/portable-cargo.ps1 run
```

### Bash (WSL/Ubuntu with native Rust)
```bash
# Quick wrapper usage
./tools/devshell/portable-cargo.sh build
./tools/devshell/portable-cargo.sh run
```

### Bash (WSL using Windows Cargo)
```bash
# Wrapper forwards to cargo.exe automatically
./tools/devshell/portable-cargo.sh build
./tools/devshell/portable-cargo.sh run
```
## 📋 Quick Reference

| Platform | Activation | Common Build Loop |
| --- | --- | --- |
| PowerShell (Windows) | `./server/tools/activate-cargo.ps1`<br>`./server/tools/activate-node.ps1` | `pnpm install --frozen-lockfile`<br>`cargo build` |
| Bash (WSL/Linux native Rust) | `source ./server/tools/activate-cargo.sh`<br>`source ./server/tools/activate-node.sh` | `pnpm install --frozen-lockfile`<br>`cargo build` |
| Bash (WSL using Windows Cargo) | `source ./server/tools/activate-cargo.sh` (choose Windows portable)<br>`source ./server/tools/activate-node.sh` | `pnpm install --frozen-lockfile`<br>`cargo.exe build` |

---

## 🧪 CLI Workflows

- `cargo build` / `cargo build --release` – Compile projects after activation.
- `cargo test` – Run the test suite.
- `cargo run --bin <target>` – Execute binaries.
- `cargo check` – Fast validation without producing binaries.
- `python server/tools/dev_env_cli.py doctor` – Confirm scripts before running automation or CI jobs.
## 🔧 VS Code Tasks

Press `Ctrl+Shift+P` → `Tasks: Run Task`:
- **Activate Portable Cargo** - Sets up environment
- **Cargo Build (Portable)** - Builds in PowerShell
- **Cargo Run (Portable)** - Runs in PowerShell
- **Cargo Test (Portable)** - Tests in PowerShell
- **Cargo Check (Portable)** - Checks in PowerShell

For WSL/Linux, use the devshell wrapper directly from your terminal.
Tasks now assume both activators are available:
- **Activate Portable Cargo** – Preps `CARGO_HOME` / `RUSTUP_HOME`
- **Activate Portable Node** – Preps `NOA_NODE_HOME`, `COREPACK_HOME`, PATH
- **Cargo Build/Test/Run (Portable)** – Chains the activators before executing

Trigger via `Ctrl+Shift+P → Tasks: Run Task` or bind them to shortcuts.

---

## 📊 Comparison Matrix

| Feature | PowerShell (Windows) | WSL (Native Rust) | WSL (Windows Cargo) |
|---------|---------------------|-------------------|---------------------|
| **Speed** | ⚡ Fast | ⚡ Fast | 🐌 Slower |
| **Setup** | Run setup script | Install Rust | Use Windows install |
| **Isolation** | ✅ Workspace-local | ❌ System-wide | ✅ Workspace-local |
| **Command** | `./tools/devshell/portable-cargo.ps1` | `./tools/devshell/portable-cargo.sh` | `./tools/devshell/portable-cargo.sh` |
| **Best For** | Windows dev | Linux-native dev | Quick testing |

---

## 🎓 Recommendations

### When to Use Each Platform:

**Use PowerShell (Windows) when:**
- ✅ Primary Windows development
- ✅ Need workspace-isolated tooling
- ✅ Want portable, self-contained setup
- ✅ Building for Windows targets

**Use WSL (Native Rust) when:**
- ✅ Cross-platform development
- ✅ Building for Linux targets
- ✅ Using Linux-specific tools
- ✅ Prefer Linux development workflow

**Use WSL (Windows Cargo) when:**
- ⚠️ Quick testing only
- ⚠️ Don't want to install Rust in WSL
- ⚠️ Performance is not critical
| Feature | PowerShell | WSL Native | WSL w/ Windows Cargo |
| --- | --- | --- | --- |
| Rust Source | Portable (`server/tools/cargo-portable`) | `~/.cargo` | Portable via `/mnt/d/...` |
| Node Source | Portable (`server/tools/node-portable`) | Same portable bundle | Same portable bundle |
| Performance | ⚡ Fast | ⚡ Fast | 🐌 Slower |
| Isolation | ✅ Workspace-local | ⚠️ Rust global, Node portable | ✅ Workspace-local |
| Recommended Use | Primary Windows dev | Cross-platform / Linux targets | Quick verification only |

---

## 🔍 Verification Commands

### Check Active Environment

**PowerShell:**
```powershell
./tools/devshell/portable-cargo.ps1 --version
Get-Content tools/devshell/state/cargo-env.json
```

**Bash (WSL/Ubuntu):**
```bash
./tools/devshell/portable-cargo.sh --version
cat tools/devshell/state/cargo-env.yaml
```
**PowerShell**
````powershell
$env:CARGO_HOME
$env:NOA_NODE_HOME
where cargo
where pnpm
cargo --version
pnpm -v
node -v
````

**Bash**
````bash
echo $CARGO_HOME
echo $NOA_NODE_HOME
which cargo
which pnpm
cargo --version
pnpm -v
node -v
````

Outputs should always reference `server/tools/...` once activated.

---

## 🆘 Troubleshooting

### "Command not found" in PowerShell
- **Solution**: Run `./tools/devshell/portable-cargo.ps1 --version`
- If the portable toolchain is missing, install via `./server/tools/setup-portable-cargo.ps1`

### "Command not found" in WSL
- **Solution**: Run `./tools/devshell/portable-cargo.sh --version`
- Or install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Rust-analyzer not working
- **Windows**: Check `.vscode/settings.json` has correct paths (until IDE settings are migrated to the wrapper flow)
- **WSL**: Ensure native Rust is installed
- **Reload**: `Ctrl+Shift+P` → "Developer: Reload Window"
- **Run**: `python server/tools/dev_env_cli.py diagnostics`
- **Windows**: Ensure `.\server\tools\activate-cargo.ps1` was executed in the current session
- **WSL**: Ensure native Rust is installed or select the portable toolchain during activation

### Wrong Cargo version
- **Check**: Inspect `tools/devshell/state/cargo-env.json`
- **Fix**: Ensure the wrapper ran successfully (`./tools/devshell/portable-cargo.* --version`)

### Mixing Windows and WSL
- **Issue**: Using Windows paths in WSL or vice versa
- **Fix**: Use appropriate activation script or wrapper for your terminal
- **Tip**: Check prompt - `PS D:\...` = PowerShell, `deflex@...` = WSL
| Issue | Resolution |
| --- | --- |
| `pnpm`/`node` missing | Run `source ./server/tools/activate-node.sh` or `./server/tools/activate-node.ps1`. |
| Host pnpm still used | Ensure `$NOA_NODE_HOME/bin` sits at the front of `PATH`. Re-run the activation scripts. |
| Make target fails early | Confirm the shell already sourced both activators; `make` now expects the env to be ready. |
| Wrong cargo version | Run `where cargo` / `which cargo` and re-source the corresponding activation script. |

---

## 📁 Portable Tool Layout

```
server/tools/
├── cargo-portable/             # Windows portable Cargo (gitignored)
├── rustup-portable/            # Windows Rustup data (gitignored)
├── setup-portable-cargo.ps1    # Windows setup (committed)
├── activate-cargo.ps1          # Windows activation (committed)
├── activate-cargo.sh           # Linux/WSL activation (committed)
├── README.md                   # Detailed documentation
├── QUICK_START.md              # Quick reference
└── MULTI_PLATFORM.md           # This file

tools/devshell/
├── portable-cargo.sh           # Unified CLI wrapper (bash)
├── portable-cargo.ps1          # Unified CLI wrapper (PowerShell)
└── state/
    └── cargo-env.{json,yaml}   # Auto-generated status snapshots
```

**WSL/Linux Rust** (if installed):
```
~/.cargo/        # Cargo installation
~/.rustup/       # Rustup data
├── cargo-portable/          # Portable Cargo + rustc (gitignored)
├── rustup-portable/         # Rustup data (gitignored)
├── node-portable/           # Node 20 + pnpm 8 bundle (gitignored)
│   └── manifest.json        # Version + hash manifest (tracked)
├── setup-portable-cargo.ps1
├── setup-portable-node.ps1
├── setup-portable-node.sh
├── activate-cargo.ps1 / activate-cargo.sh / activate-cargo-wsl.sh
├── activate-node.ps1 / activate-node.sh
└── README / QUICK_START / MULTI_PLATFORM docs
```

---

## 🚀 Getting Started

### First Time (Choose Your Platform)

**Windows Developer:**
```powershell
# PowerShell
./server/tools/setup-portable-cargo.ps1
./tools/devshell/portable-cargo.ps1 --version
```

**Linux/WSL Developer:**
```bash
# WSL/Ubuntu
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
./tools/devshell/portable-cargo.sh --version
```

**Multi-Platform Developer:**
- Do both! Keep them separate:
  - Windows portable in `server/tools/`
  - Linux system-wide in `~/.cargo/`

---

## 💡 Tips

1. **Prefer the CLI wrapper** (`tools/devshell/portable-cargo.*`) to keep environment state and automation aligned.
2. **Rust-analyzer and VS Code tasks** still target the portable toolchain today, but IDE-specific settings will be phased out in favour of the unified wrapper flow.
3. **Tasks** use PowerShell today—run them from PowerShell or update them to call the wrapper for parity.
4. **Add to shell profile** for automatic activation if you prefer interactive usage:
   - PowerShell: Add `./server/tools/activate-cargo.ps1`
   - Bash: Add `source ./server/tools/activate-cargo.sh`
1. **Run the CLI summary** before switching platforms to confirm paths.
2. **Use `python server/tools/dev_env_cli.py doctor`** after upgrades or reinstalls.
3. **Keep Windows and WSL toolchains separate** to avoid path collisions.
4. **Add to shell profile** for automatic activation:
   - PowerShell: Add `.\server\tools\activate-cargo.ps1` to `$PROFILE`
   - Bash: Add `source ./server/tools/activate-cargo.sh` to `~/.bashrc`

---
1. Run both setup scripts for your platform.
2. Activate cargo + node in every new terminal.
3. Use `pnpm` and `cargo` confidently knowing they originate from `server/tools/` and satisfy HT‑01.

**Keep the environment portable, offline-ready, and policy-compliant.**
