# Multi-Platform Cargo Setup Guide

## 🌐 Overview

This workspace supports development on **three platforms**:

1. **Windows (PowerShell)** - Uses portable Cargo in `server/tools/cargo-portable/`
2. **WSL (Windows Subsystem for Linux)** - Can use native Linux Rust or Windows portable
3. **Ubuntu/Linux** - Uses system Rust installation

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

### Every Session

```powershell
# Activate portable Cargo interactively when needed
./server/tools/activate-cargo.ps1

# Preferred: run commands through the wrapper
./tools/devshell/portable-cargo.ps1 --version
./tools/devshell/portable-cargo.ps1 build
./tools/devshell/portable-cargo.ps1 test
```

### Location
- **Cargo**: `D:\dev\workspaces\noa_ark_os\server\tools\cargo-portable\`
- **Rustup**: `D:\dev\workspaces\noa_ark_os\server\tools\rustup-portable\`

---

## 🐧 WSL/Ubuntu Setup

You have **two options** for using Rust in WSL:

### Option 1: Native Linux Rust (Recommended)

**Install Rust in WSL:**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Activate for current session
source "$HOME/.cargo/env"

# Verify through the wrapper
./tools/devshell/portable-cargo.sh --version
```

**Every Session:**

```bash
# Optional interactive activation
source ./server/tools/activate-cargo.sh

# Preferred: run commands through the wrapper
./tools/devshell/portable-cargo.sh build
./tools/devshell/portable-cargo.sh run
```

### Option 2: Use Windows Portable Cargo from WSL

**Setup:**

```bash
# Set environment variables
export CARGO_HOME="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable"
export RUSTUP_HOME="/mnt/d/dev/workspaces/noa_ark_os/server/tools/rustup-portable"
export PATH="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable/bin:$PATH"

# Use the wrapper so commands forward to cargo.exe automatically
./tools/devshell/portable-cargo.sh --version
```

**Note**: This option is **slower** because it runs Windows executables through WSL interop.

---

## 🎯 VS Code Terminal Profiles

The workspace is configured with multiple terminal profiles:

### Windows Terminals:
- **PowerShell** (Default for Windows) - Use for portable Cargo
- **Command Prompt** - Available if needed
- **WSL** - Ubuntu subsystem
- **Git Bash** - If installed

### Switch Terminals:
1. Click the dropdown arrow (v) in terminal panel
2. Select desired profile
3. Or press `Ctrl+Shift+P` → "Terminal: Select Default Profile"

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

---

## 🔧 VS Code Tasks

Pre-configured tasks work with **PowerShell** (portable Cargo):

Press `Ctrl+Shift+P` → `Tasks: Run Task`:
- **Activate Portable Cargo** - Sets up environment
- **Cargo Build (Portable)** - Builds in PowerShell
- **Cargo Run (Portable)** - Runs in PowerShell
- **Cargo Test (Portable)** - Tests in PowerShell
- **Cargo Check (Portable)** - Checks in PowerShell

For WSL/Linux, use the devshell wrapper directly from your terminal.

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

### Expected Output

**PowerShell (Portable):**
```
CARGO_HOME: D:\dev\workspaces\noa_ark_os\server\tools\cargo-portable
RUSTUP_HOME: D:\dev\workspaces\noa_ark_os\server\tools\rustup-portable
cargo 1.90.0 (...)
```

**WSL (Native):**
```
CARGO_HOME: /home/username/.cargo
RUSTUP_HOME: /home/username/.rustup
cargo 1.90.0 (...)
```

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

### Wrong Cargo version
- **Check**: Inspect `tools/devshell/state/cargo-env.json`
- **Fix**: Ensure the wrapper ran successfully (`./tools/devshell/portable-cargo.* --version`)

### Mixing Windows and WSL
- **Issue**: Using Windows paths in WSL or vice versa
- **Fix**: Use appropriate activation script or wrapper for your terminal
- **Tip**: Check prompt - `PS D:\...` = PowerShell, `deflex@...` = WSL

---

## 📁 File Structure

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

**Choose the setup that fits your workflow best!** 🎯
