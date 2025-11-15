# Portable Cargo - Quick Reference

## ⚡ Unified Quick Start

Use the devshell wrapper so Cargo is initialised the same way for terminals, scripts, and CI:
```powershell
# Activate Cargo
python server/tools/dev_env_cli.py activate --platform windows
.\server\tools\activate-cargo.ps1

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
## 🔄 Terminal Check

- Run `python server/tools/dev_env_cli.py doctor` to confirm the activation scripts are present.
- Ensure your prompt begins with `PS` (PowerShell) before invoking `.\server\tools\activate-cargo.ps1`.
- If you see a WSL prompt such as `deflex@FlexNetOS-1001:/mnt/d/...`, switch to Windows PowerShell first.

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
Remove-Item -Recurse -Force server\tools\cargo-portable
Remove-Item -Recurse -Force server\tools\rustup-portable
./server/tools/setup-portable-cargo.ps1
```

## 💾 VS Code Configuration

The workspace still includes PowerShell-centric settings today, but these will be phased out as the devshell wrapper becomes the default entrypoint for tooling. Tasks can be updated to call `tools/devshell/portable-cargo.ps1` for parity with the CLI flow.

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
- Full setup guide: `server/tools/MULTI_PLATFORM.md`
- Audit report: `server/TOOLS_AUDIT.md`
