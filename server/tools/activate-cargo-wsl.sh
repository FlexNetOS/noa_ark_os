#!/usr/bin/env bash
# NOA ARK OS - Activate Cargo (WSL/Linux)
# Usage: source ./server/tools/activate-cargo-wsl.sh

__NOA_PREV_CARGO_OPTS="$(set +o)"
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
SILENT="${NOA_ACTIVATE_SILENT:-0}"

if [[ -z "${NOA_PWSH_ENV:-}" && "${NOA_SKIP_AUTO_PWSH:-0}" != "1" ]]; then
    if [[ -f "$SCRIPT_DIR/activate-pwsh.sh" ]]; then
        __NOA_PREV_ACTIVATE_SILENT="${NOA_ACTIVATE_SILENT:-0}"
        NOA_ACTIVATE_SILENT=1
        # shellcheck source=/dev/null
        source "$SCRIPT_DIR/activate-pwsh.sh" 2>/dev/null || true
        NOA_ACTIVATE_SILENT="$__NOA_PREV_ACTIVATE_SILENT"
        unset __NOA_PREV_ACTIVATE_SILENT
    fi
fi

log() {
    if [[ "$SILENT" == "1" ]]; then
        return
    fi
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
    export NOA_CARGO_ENV=1
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

        if [[ "$SILENT" == "1" ]]; then
            reply="n"
        else
            read -r -p "Would you like to use Windows Cargo from WSL? (y/N): " reply
            echo
        fi
        if [[ $reply =~ ^[Yy]$ ]]; then
            export CARGO_HOME="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable"
            export RUSTUP_HOME="/mnt/d/dev/workspaces/noa_ark_os/server/tools/rustup-portable"
            export PATH="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable/bin:$PATH"

            log "\n✅ Windows Cargo activated in WSL!"
            cargo.exe --version
            rustc.exe --version
            echo ""
            export NOA_CARGO_ENV=1
        fi
    fi
fi

eval "$__NOA_PREV_CARGO_OPTS"
unset __NOA_PREV_CARGO_OPTS
return 0 2>/dev/null || exit 0
