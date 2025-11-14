# Portable Cargo - Quick Reference

## ⚡ Unified Quick Start

Use the devshell wrapper so Cargo is initialised the same way for terminals, scripts, and CI:

```powershell
# Windows PowerShell
./tools/devshell/portable-cargo.ps1 build
./tools/devshell/portable-cargo.ps1 test
```

```bash
# Linux / WSL
./tools/devshell/portable-cargo.sh build
./tools/devshell/portable-cargo.sh test
```

The wrapper sources the appropriate activator script, sets `CARGO_HOME`/`RUSTUP_HOME`, and records the current environment in `tools/devshell/state/cargo-env.{json,yaml}` for other commands to reuse.

## 🛠️ Initial Setup

```powershell
# Windows portable toolchain
./server/tools/setup-portable-cargo.ps1
```

```bash
# Native Linux toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, run the wrapper once to generate the status snapshot:

```powershell
./tools/devshell/portable-cargo.ps1 --version
```

```bash
./tools/devshell/portable-cargo.sh --version
```

## 🧭 Shell Guidance

- **PowerShell** users can still run `./server/tools/activate-cargo.ps1` for interactive sessions, but the wrapper is preferred for scripted commands.
- **Bash/WSL** users may source `./server/tools/activate-cargo.sh` for long-lived shells, then rely on the wrapper for builds/tests.
- `tools/devshell/state/cargo-env.json` reflects the last activation mode (`portable` vs `system`).

## 📋 Common Workflows

### Building

```powershell
./tools/devshell/portable-cargo.ps1 build
```

```bash
./tools/devshell/portable-cargo.sh build
```

### Testing

```powershell
./tools/devshell/portable-cargo.ps1 test
```

```bash
./tools/devshell/portable-cargo.sh test
```

### Checking / Linting

```powershell
./tools/devshell/portable-cargo.ps1 check
./tools/devshell/portable-cargo.ps1 fmt
```

```bash
./tools/devshell/portable-cargo.sh check
./tools/devshell/portable-cargo.sh fmt
```

## 🔄 Terminal Setup

### If You See WSL/bash Prompt

If you see:
```
deflex@FlexNetOS-1001:/mnt/d/...
```

You're in WSL. Use the bash wrapper:

```bash
./tools/devshell/portable-cargo.sh --version
```

### Correct PowerShell Prompt

You should see:
```
PS D:\dev\workspaces\noa_ark_os>
```

Then run:

```powershell
./tools/devshell/portable-cargo.ps1 --version
```

## 📁 Installation Locations

```
server/tools/
├── cargo-portable/       # Cargo installation (portable)
├── rustup-portable/      # Rustup data (portable)
├── activate-cargo.ps1    # Activation script (Windows)
├── activate-cargo.sh     # Activation script (WSL/Linux)
└── setup-portable-cargo.ps1

tools/devshell/
├── portable-cargo.ps1    # PowerShell wrapper
├── portable-cargo.sh     # Bash wrapper
└── state/                # Auto-generated status snapshots
```

## 🆘 Troubleshooting

### "cargo: command not found"

- Run the wrapper with `--version` to refresh the environment snapshot.
- Windows: ensure the portable toolchain exists (`./server/tools/setup-portable-cargo.ps1`).
- Linux/WSL: install Rust via `rustup` if the system toolchain is missing.

### Need to reinstall (Windows portable)

```powershell
Remove-Item -Recurse -Force server\tools\cargo-portable
Remove-Item -Recurse -Force server\tools\rustup-portable
./server/tools/setup-portable-cargo.ps1
```

## 💾 VS Code Configuration

The workspace still includes PowerShell-centric settings today, but these will be phased out as the devshell wrapper becomes the default entrypoint for tooling. Tasks can be updated to call `tools/devshell/portable-cargo.ps1` for parity with the CLI flow.

## 📚 Additional Resources

- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [Rust Book](https://doc.rust-lang.org/book/)
- Full setup guide: `server/tools/MULTI_PLATFORM.md`
- Audit report: `server/TOOLS_AUDIT.md`
