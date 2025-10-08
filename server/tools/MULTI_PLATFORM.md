# Multi-Platform Cargo Setup Guide

## 🌐 Overview

This workspace supports development on **three platforms**:

1. **Windows (PowerShell)** - Uses portable Cargo in `server/tools/cargo-portable/`
2. **WSL (Windows Subsystem for Linux)** - Can use native Linux Rust or Windows portable
3. **Ubuntu/Linux** - Uses system Rust installation

---

## 🪟 Windows (PowerShell) Setup

### First Time Setup

```powershell
# Run setup script (one-time)
.\server\tools\setup-portable-cargo.ps1
```

### Every Session

```powershell
# Activate portable Cargo
.\server\tools\activate-cargo.ps1

# Verify
cargo --version
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
source $HOME/.cargo/env

# Verify
cargo --version
```

**Every Session:**

```bash
# Activate Cargo
source ./server/tools/activate-cargo.sh

# Or add to ~/.bashrc for automatic loading:
echo 'source $HOME/.cargo/env' >> ~/.bashrc
```

### Option 2: Use Windows Portable Cargo from WSL

**Setup:**

```bash
# Set environment variables
export CARGO_HOME="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable"
export RUSTUP_HOME="/mnt/d/dev/workspaces/noa_ark_os/server/tools/rustup-portable"
export PATH="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable/bin:$PATH"

# Use .exe extension
cargo.exe --version
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
# Activate
.\server\tools\activate-cargo.ps1

# Build
cargo build

# Run
cargo run
```

### Bash (WSL/Ubuntu with native Rust)
```bash
# Activate
source ./server/tools/activate-cargo.sh

# Build
cargo build

# Run
cargo run
```

### Bash (WSL using Windows Cargo)
```bash
# Activate
source ./server/tools/activate-cargo.sh
# (Choose option to use Windows Cargo)

# Build (use .exe)
cargo.exe build

# Run (use .exe)
cargo.exe run
```

---

## 🔧 VS Code Tasks

Pre-configured tasks work with **PowerShell** (portable Cargo):

Press `Ctrl+Shift+P` → `Tasks: Run Task`:
- **Cargo Build (Portable)** - Builds in PowerShell
- **Cargo Run (Portable)** - Runs in PowerShell
- **Cargo Test (Portable)** - Tests in PowerShell
- **Cargo Check (Portable)** - Checks in PowerShell

For WSL/Linux, use the terminal directly.

---

## 📊 Comparison Matrix

| Feature | PowerShell (Windows) | WSL (Native Rust) | WSL (Windows Cargo) |
|---------|---------------------|-------------------|---------------------|
| **Speed** | ⚡ Fast | ⚡ Fast | 🐌 Slower |
| **Setup** | Run setup script | Install Rust | Use Windows install |
| **Isolation** | ✅ Workspace-local | ❌ System-wide | ✅ Workspace-local |
| **Command** | `cargo` | `cargo` | `cargo.exe` |
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
$env:CARGO_HOME
$env:RUSTUP_HOME
where.exe cargo
cargo --version
```

**Bash (WSL/Ubuntu):**
```bash
echo $CARGO_HOME
echo $RUSTUP_HOME
which cargo
cargo --version
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
- **Solution**: Run `.\server\tools\activate-cargo.ps1`

### "Command not found" in WSL
- **Solution**: Run `source ./server/tools/activate-cargo.sh`
- Or install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Rust-analyzer not working
- **Windows**: Check `.vscode/settings.json` has correct paths
- **WSL**: Ensure native Rust is installed
- **Reload**: `Ctrl+Shift+P` → "Developer: Reload Window"

### Wrong Cargo version
- **Check**: Run `where.exe cargo` (Windows) or `which cargo` (Linux)
- **Fix**: Ensure activation script ran successfully

### Mixing Windows and WSL
- **Issue**: Using Windows paths in WSL or vice versa
- **Fix**: Use appropriate activation script for your terminal
- **Tip**: Check prompt - `PS D:\...` = PowerShell, `deflex@...` = WSL

---

## 📁 File Structure

```
server/tools/
├── cargo-portable/         # Windows portable Cargo (gitignored)
├── rustup-portable/        # Windows Rustup data (gitignored)
├── setup-portable-cargo.ps1   # Windows setup (committed)
├── activate-cargo.ps1      # Windows activation (committed)
├── activate-cargo.sh       # Linux/WSL activation (committed)
├── README.md              # Detailed documentation
├── QUICK_START.md         # Quick reference
└── MULTI_PLATFORM.md      # This file
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
.\server\tools\setup-portable-cargo.ps1
.\server\tools\activate-cargo.ps1
```

**Linux/WSL Developer:**
```bash
# WSL/Ubuntu
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Multi-Platform Developer:**
- Do both! Keep them separate:
  - Windows portable in `server/tools/`
  - Linux system-wide in `~/.cargo/`

---

## 💡 Tips

1. **VS Code will auto-detect** your active terminal and use appropriate commands
2. **Rust-analyzer** is configured for Windows portable by default
3. **Tasks** use PowerShell, run them from PowerShell terminal
4. **Add to shell profile** for automatic activation:
   - PowerShell: Add to `$PROFILE`
   - Bash: Add to `~/.bashrc`

---

**Choose the setup that fits your workflow best!** 🎯
