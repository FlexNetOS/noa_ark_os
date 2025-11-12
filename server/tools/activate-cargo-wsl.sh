#!/usr/bin/env bash
# NOA ARK OS - Activate Cargo (WSL/Linux)
# Usage: source ./server/tools/activate-cargo-wsl.sh

set -euo pipefail

log() {
    echo -e "$1"
}

log "\n🔧 Activating Cargo for WSL/Linux..."
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "$HOME/.cargo/env" ]; then
    # Load Cargo environment
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"

    log "\n✅ Cargo Activated Successfully!"
    log "\nEnvironment:"
    echo "  CARGO_HOME   = ${CARGO_HOME:-<not set>}"
    echo "  RUSTUP_HOME  = ${RUSTUP_HOME:-<not set>}"

    log "\nVersions:"
    cargo --version
    rustc --version

    log "\n💡 Tips:"
    echo "  • Run 'cargo build' to build projects"
    echo "  • Run 'cargo run' to run projects"
    echo "  • Run 'cargo test' to run tests"
    echo ""
else
    log "\n⚠️  Rust not found in WSL!"
    log "\nTo install Rust in WSL, run:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    log "\nThen reload this script:"
    echo "  source ./server/tools/activate-cargo-wsl.sh"
    echo ""

    WIN_CARGO="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable/bin/cargo.exe"
    if [ -f "$WIN_CARGO" ]; then
        log "ℹ️  Windows portable Cargo detected at:"
        echo "  $WIN_CARGO"
        log "\nYou can use Windows Cargo from WSL (slower):"
        echo "  export CARGO_HOME=/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable"
        echo "  export RUSTUP_HOME=/mnt/d/dev/workspaces/noa_ark_os/server/tools/rustup-portable"
        echo "  export PATH=\"/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable/bin:\$PATH\""
        echo ""

        read -r -p "Would you like to use Windows Cargo from WSL? (y/N): " reply
        echo
        if [[ $reply =~ ^[Yy]$ ]]; then
            export CARGO_HOME="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable"
            export RUSTUP_HOME="/mnt/d/dev/workspaces/noa_ark_os/server/tools/rustup-portable"
            export PATH="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable/bin:$PATH"

            log "\n✅ Windows Cargo activated in WSL!"
            cargo.exe --version
            rustc.exe --version
            echo ""
        fi
    fi
fi
