# Multi-Platform Cargo & Node Setup Guide

## 🌐 Overview

This workspace supports development on **three platforms**:

1. **Windows (PowerShell)** – Portable Cargo lives in `server/tools/cargo-portable/` and portable Node/pnpm in `server/tools/node-portable/`.
2. **WSL (Windows Subsystem for Linux)** – Can use native Linux Rust, but must still source the hermetic Node bundle for pnpm operations.
3. **Ubuntu/Linux** – Typically uses system Rust plus the shared Node bundle to keep builds offline.

**Rule:** Activate both cargo *and* node scripts before running `make`, `pnpm`, or any CI pipelines.

---

## 🪟 Windows (PowerShell) Setup

### First Time Setup

````powershell
# Run once per machine
./server/tools/setup-portable-cargo.ps1
./server/tools/setup-portable-node.ps1
````

### Every Session

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

# Each session
source "$HOME/.cargo/env"
source ./server/tools/activate-node.sh

cargo --version
pnpm -v
````

### Option 2 – Use Windows Portable Cargo from WSL

````bash
export CARGO_HOME="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable"
export RUSTUP_HOME="/mnt/d/dev/workspaces/noa_ark_os/server/tools/rustup-portable"
export PATH="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable/bin:$PATH"

source ./server/tools/activate-node.sh
cargo.exe --version
````

**Reminder:** Windows binaries via WSL interop are slower; prefer native Rust where possible, but always reuse the portable Node bundle.

---

## 🎯 VS Code Terminal Profiles

- **PowerShell** – Default for portable toolchains on Windows
- **WSL** – Ubuntu shell
- **Git Bash / CMD** – Optional

Switch via the terminal dropdown or `Ctrl+Shift+P → Terminal: Select Default Profile`.

---

## 📋 Quick Reference

| Platform | Activation | Common Build Loop |
| --- | --- | --- |
| PowerShell (Windows) | `./server/tools/activate-cargo.ps1`<br>`./server/tools/activate-node.ps1` | `pnpm install --frozen-lockfile`<br>`cargo build` |
| Bash (WSL/Linux native Rust) | `source ./server/tools/activate-cargo.sh`<br>`source ./server/tools/activate-node.sh` | `pnpm install --frozen-lockfile`<br>`cargo build` |
| Bash (WSL using Windows Cargo) | `source ./server/tools/activate-cargo.sh` (choose Windows portable)<br>`source ./server/tools/activate-node.sh` | `pnpm install --frozen-lockfile`<br>`cargo.exe build` |

---

## 🔧 VS Code Tasks

Tasks now assume both activators are available:
- **Activate Portable Cargo** – Preps `CARGO_HOME` / `RUSTUP_HOME`
- **Activate Portable Node** – Preps `NOA_NODE_HOME`, `COREPACK_HOME`, PATH
- **Cargo Build/Test/Run (Portable)** – Chains the activators before executing

Trigger via `Ctrl+Shift+P → Tasks: Run Task` or bind them to shortcuts.

---

## 📊 Comparison Matrix

| Feature | PowerShell | WSL Native | WSL w/ Windows Cargo |
| --- | --- | --- | --- |
| Rust Source | Portable (`server/tools/cargo-portable`) | `~/.cargo` | Portable via `/mnt/d/...` |
| Node Source | Portable (`server/tools/node-portable`) | Same portable bundle | Same portable bundle |
| Performance | ⚡ Fast | ⚡ Fast | 🐌 Slower |
| Isolation | ✅ Workspace-local | ⚠️ Rust global, Node portable | ✅ Workspace-local |
| Recommended Use | Primary Windows dev | Cross-platform / Linux targets | Quick verification only |

---

## 🔍 Verification Commands

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

1. Run both setup scripts for your platform.
2. Activate cargo + node in every new terminal.
3. Use `pnpm` and `cargo` confidently knowing they originate from `server/tools/` and satisfy HT‑01.

**Keep the environment portable, offline-ready, and policy-compliant.**
