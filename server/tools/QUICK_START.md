# Portable Cargo - Quick Reference

## ⚡ Quick Start (PowerShell Only!)

```powershell
# Activate Cargo
.\server\tools\activate-cargo.ps1

# Verify
cargo --version
```

## 🔧 VS Code Settings Updated

The workspace now has:
- ✅ PowerShell as default terminal
- ✅ Portable Cargo environment variables
- ✅ Rust-analyzer configured for portable installation
- ✅ Tasks for common Cargo commands

## 📋 Available VS Code Tasks

Press `Ctrl+Shift+P` → `Tasks: Run Task`:
- **Activate Portable Cargo** - Sets up environment
- **Cargo Build (Portable)** - Builds the project
- **Cargo Run (Portable)** - Runs the project
- **Cargo Test (Portable)** - Runs tests
- **Cargo Check (Portable)** - Checks for errors

## ⚠️ Important Notes

### USE POWERSHELL ONLY

**DO NOT** use WSL/bash with the portable Cargo installation!

❌ **Wrong**:
```bash
# This will NOT work (WSL/bash)
./server/tools/activate-cargo.ps1
cargo build
```

✅ **Correct**:
```powershell
# This WILL work (PowerShell)
.\server\tools\activate-cargo.ps1
cargo build
```

### Why PowerShell Only?

The portable Cargo installation contains Windows executables (`.exe` files) that are designed to run on Windows. While they *can* technically run through WSL interop, the environment variables and paths don't translate correctly.

## 🚀 Common Workflows

### Building a Project

```powershell
# Activate Cargo
.\server\tools\activate-cargo.ps1

# Navigate to project
cd crc

# Build
cargo build

# Build release
cargo build --release
```

### Running Tests

```powershell
# Activate Cargo
.\server\tools\activate-cargo.ps1

# Run all tests
cargo test

# Run specific test
cargo test test_name
```

### Checking Code

```powershell
# Activate Cargo
.\server\tools\activate-cargo.ps1

# Quick check (faster than build)
cargo check

# Lint with Clippy
cargo clippy
```

## 🔄 Terminal Setup

### If You See WSL/bash Prompt

If you see:
```
deflex@FlexNetOS-1001:/mnt/d/...
```

You're in WSL. You need PowerShell instead:

1. In VS Code: Click the terminal dropdown (v icon) → Select "PowerShell"
2. Or press `Ctrl+Shift+P` → "Terminal: Select Default Profile" → Choose "PowerShell"
3. Open new terminal: `Ctrl+Shift+`` (backtick)

### Correct PowerShell Prompt

You should see:
```
PS D:\dev\workspaces\noa_ark_os>
```

## 📁 Installation Locations

```
server/tools/
├── cargo-portable/       # Cargo installation
│   └── bin/
│       ├── cargo.exe     # ← Windows executable
│       ├── rustc.exe     # ← Windows executable
│       └── rustfmt.exe   # ← Windows executable
├── rustup-portable/      # Rustup data
└── activate-cargo.ps1    # Activation script
```

## 🆘 Troubleshooting

### "cargo: command not found"

**Cause**: Cargo not activated or using wrong shell

**Solution**:
1. Make sure you're in PowerShell (not WSL)
2. Run activation script: `.\server\tools\activate-cargo.ps1`

### "Cannot find path" errors

**Cause**: Using WSL/bash instead of PowerShell

**Solution**: Switch to PowerShell terminal

### Need to reinstall

```powershell
# Remove old installation
Remove-Item -Recurse -Force server\tools\cargo-portable
Remove-Item -Recurse -Force server\tools\rustup-portable

# Reinstall
.\server\tools\setup-portable-cargo.ps1
```

## 💾 VS Code Configuration

The workspace now includes:

### `.vscode/settings.json`
- Sets PowerShell as default terminal
- Configures Rust-analyzer for portable Cargo
- Sets environment variables automatically

### `.vscode/tasks.json`
- Pre-configured build tasks
- Activates Cargo automatically
- Run with `Ctrl+Shift+P` → "Tasks: Run Task"

## 📚 Additional Resources

- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [Rust Book](https://doc.rust-lang.org/book/)
- Full setup guide: `server/tools/README.md`
- Audit report: `server/TOOLS_AUDIT.md`
