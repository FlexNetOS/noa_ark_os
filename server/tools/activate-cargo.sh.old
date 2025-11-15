#!/usr/bin/env bash
# NOA ARK OS - Activate Portable Cargo (Windows)
# Usage: source ./server/tools/activate-cargo.sh

set -euo pipefail

NOA_ACTIVATE_SILENT="${NOA_ACTIVATE_SILENT:-0}"

if [[ "$NOA_ACTIVATE_SILENT" != "1" ]]; then
    echo -e "\n🔧 Activating Portable Cargo..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

CARGO_HOME="$WORKSPACE_ROOT/server/tools/cargo-portable"
RUSTUP_HOME="$WORKSPACE_ROOT/server/tools/rustup-portable"

if [ ! -f "$CARGO_HOME/bin/cargo.exe" ]; then
    echo -e "\n❌ ERROR: Portable Cargo not found!"
    echo -e "\nExpected location: $CARGO_HOME/bin/cargo.exe"
    echo -e "\nPlease run setup first:"
    echo "  ./server/tools/setup-portable-cargo.ps1"
    echo
    exit 1
fi

export CARGO_HOME="$CARGO_HOME"
export RUSTUP_HOME="$RUSTUP_HOME"

if [[ ":$PATH:" != *":$CARGO_HOME/bin:"* ]]; then
    export PATH="$CARGO_HOME/bin:$PATH"
fi

export NOA_CARGO_ENV=1

if [[ "$NOA_ACTIVATE_SILENT" != "1" ]]; then
    echo -e "\n✅ Portable Cargo Activated Successfully!"
    echo -e "\nEnvironment:"
    echo "  CARGO_HOME   = $CARGO_HOME"
    echo "  RUSTUP_HOME  = $RUSTUP_HOME"
    echo "  PATH         = [cargo-portable/bin prepended]"

    echo -e "\nVersions:"
    cargo --version
    rustc --version

    echo -e "\n💡 Tips:"
    echo "  • Run 'cargo build' to build projects"
    echo "  • Run 'cargo run' to run projects"
    echo "  • Run 'cargo test' to run tests"
    echo "  • This activation is for the current shell session only"
    echo
fi
