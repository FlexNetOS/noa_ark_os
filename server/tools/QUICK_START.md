# Portable Cargo - Quick Reference

## ⚡ Quick Start (PowerShell Only!)

```powershell
# Activate Cargo
python server/tools/dev_env_cli.py activate --platform windows
.\server\tools\activate-cargo.ps1

# Verify
cargo --version
```

## 🧭 CLI Helpers

Use the workspace CLI to inspect and validate your environment before running commands:

- `python server/tools/dev_env_cli.py summary` – Show portable toolchain locations.
- `python server/tools/dev_env_cli.py activate --platform windows` – Review activation steps.
- `python server/tools/dev_env_cli.py doctor` – Confirm activation scripts and directories exist.
- `python server/tools/dev_env_cli.py diagnostics` – Rust-analyzer guidance without opening an IDE.

## 🧪 Common CLI Commands

- `cargo build` – Compile the current project after activation.
- `cargo test` – Run the full test suite.
- `cargo run --bin <name>` – Execute a binary target.
- `cargo check` – Fast verification without producing binaries.

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

## 🔄 Terminal Check

- Run `python server/tools/dev_env_cli.py doctor` to confirm the activation scripts are present.
- Ensure your prompt begins with `PS` (PowerShell) before invoking `.\server\tools\activate-cargo.ps1`.
- If you see a WSL prompt such as `deflex@FlexNetOS-1001:/mnt/d/...`, switch to Windows PowerShell first.

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
1. Run `python server/tools/dev_env_cli.py doctor` to confirm scripts are present.
2. Make sure you're in PowerShell (not WSL).
3. Run activation script: `.\server\tools\activate-cargo.ps1`.

### "Cannot find path" errors

**Cause**: Using WSL/bash instead of PowerShell

**Solution**: Switch to PowerShell terminal and re-run `python server/tools/dev_env_cli.py activate --platform windows` for guidance.

### Need to reinstall

```powershell
# Remove old installation
Remove-Item -Recurse -Force server\tools\cargo-portable
Remove-Item -Recurse -Force server\tools\rustup-portable

# Reinstall
.\server\tools\setup-portable-cargo.ps1

# Verify
python server/tools/dev_env_cli.py doctor
```

## 🗂️ CLI Assets

- `server/tools/dev_env_cli.py` – Workspace configuration helper.
- `server/tools/activate-cargo.ps1` – Windows activation script.
- `server/tools/activate-cargo.sh` – WSL/Linux activation script.
- `server/tools/setup-portable-cargo.ps1` – One-time bootstrapper.

## 📚 Additional Resources

- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [Rust Book](https://doc.rust-lang.org/book/)
- Full setup guide: `server/tools/README.md`
- Audit report: `server/TOOLS_AUDIT.md`
