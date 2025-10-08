# NOA ARK OS - Portable Development Tools

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
# 1. Open PowerShell terminal in VS Code or Windows

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

**VS Code Integration**: You can also use Tasks (Ctrl+Shift+P → "Tasks: Run Task" → "Cargo Build (Portable)")

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
├── setup-portable-cargo.ps1 # One-time setup script
└── activate-cargo.ps1       # Activation script (run every session)
```

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

```powershell
# After activation, verify:
cargo --version
# Should show: cargo 1.75.0 (D:\dev\workspaces\noa_ark_os\server\tools\cargo-portable\bin\cargo.exe)

rustc --version
# Should show: rustc 1.75.0

# Check location
where.exe cargo
# Should show: D:\dev\workspaces\noa_ark_os\server\tools\cargo-portable\bin\cargo.exe
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
```

---

## ❓ Troubleshooting

### "Cargo not found"

1. Make sure you ran `activate-cargo.ps1` in current session
2. Check if `cargo.exe` exists at: `server\tools\cargo-portable\bin\cargo.exe`
3. If missing, run `setup-portable-cargo.ps1` again

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
