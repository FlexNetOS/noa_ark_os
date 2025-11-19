#!/usr/bin/env bash
# NOA ARK OS - Setup Portable Cargo (One-Time)
# Usage: bash ./server/tools/setup-portable-cargo.sh [--workspace <path>] [--force]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFAULT_WORKSPACE="$(cd "$SCRIPT_DIR/../.." && pwd)"
WORKSPACE_ROOT="$DEFAULT_WORKSPACE"
FORCE_REINSTALL="0"
PROFILE="${NOA_RUST_PROFILE:-minimal}"
TOOLCHAIN="${NOA_RUST_TOOLCHAIN:-stable}"
RUSTUP_INIT_URL="${NOA_RUSTUP_INIT_URL:-https://sh.rustup.rs}"

usage() {
    cat <<'USAGE'
NOA ARK OS - Portable Cargo Setup (POSIX)

Options:
  --workspace <path>   Override workspace root detection
  --force              Reinstall even if cargo-portable already exists
  -h, --help           Show this help message

Environment overrides:
  NOA_RUST_PROFILE, NOA_RUST_TOOLCHAIN, NOA_RUSTUP_INIT_URL,
  NOA_CARGO_HOME, NOA_RUSTUP_HOME
USAGE
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --workspace)
            if [[ ! -d "$2" ]]; then
                echo "Workspace directory does not exist: $2" >&2
                exit 1
            fi
            WORKSPACE_ROOT="$(cd "$2" && pwd)"
            shift 2
            ;;
        --force)
            FORCE_REINSTALL="1"
            shift 1
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

CARGO_HOME="${NOA_CARGO_HOME:-$WORKSPACE_ROOT/server/tools/cargo-portable}"
RUSTUP_HOME="${NOA_RUSTUP_HOME:-$WORKSPACE_ROOT/server/tools/rustup-portable}"
CARGO_BIN_UNIX="$CARGO_HOME/bin/cargo"
CARGO_BIN_WIN="$CARGO_HOME/bin/cargo.exe"

info() { printf '%b\n' "${1}"; }
success() { info "✅ ${1}"; }
warn() { info "⚠️  ${1}"; }
error() { info "❌ ${1}" >&2; }

info "\n🚀 NOA ARK OS - Portable Cargo Setup (POSIX)"
info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
info "Workspace: $WORKSPACE_ROOT"
info "CARGO_HOME: $CARGO_HOME"
info "RUSTUP_HOME: $RUSTUP_HOME"

mkdir -p "$CARGO_HOME/bin" "$RUSTUP_HOME"

if [[ -x "$CARGO_BIN_UNIX" || -x "$CARGO_BIN_WIN" ]]; then
    if [[ "$FORCE_REINSTALL" != "1" ]]; then
        read -r -p "Portable Cargo already exists. Reinstall? [y/N] " response || true
        case "${response:-}" in
            [yY][eE][sS]|[yY])
                ;;
            *)
                warn "Keeping existing installation."
                exit 0
                ;;
        esac
    else
        warn "--force supplied. Reinstalling portable Cargo."
    fi
fi

if ! command -v curl >/dev/null 2>&1; then
    error "curl is required to download rustup-init"
    exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

# SECURITY NOTE:
# This script downloads the official rustup-init script from $RUSTUP_INIT_URL (default: https://sh.rustup.rs).
# - The integrity of the Rust toolchain is checked by rustup-init itself after download.
# - For air-gapped or high-security environments, you may override NOA_RUSTUP_INIT_URL to use a locally verified copy.
#   Example: export NOA_RUSTUP_INIT_URL="file:///path/to/verified/rustup-init.sh"
info "\n📥 Downloading rustup-init script..."
curl -fsSL "$RUSTUP_INIT_URL" -o "$TMP_DIR/rustup-init.sh"
chmod +x "$TMP_DIR/rustup-init.sh"

info "\n🔧 Installing Rust toolchain (profile=$PROFILE, toolchain=$TOOLCHAIN)..."
CARGO_HOME="$CARGO_HOME" \
RUSTUP_HOME="$RUSTUP_HOME" \
"$TMP_DIR/rustup-init.sh" -y --profile "$PROFILE" --default-toolchain "$TOOLCHAIN" --no-modify-path

success "Rustup completed"

PORTABLE_CARGO_BIN=""
if [[ -x "$CARGO_BIN_UNIX" ]]; then
    PORTABLE_CARGO_BIN="$CARGO_BIN_UNIX"
elif [[ -x "$CARGO_BIN_WIN" ]]; then
    PORTABLE_CARGO_BIN="$CARGO_BIN_WIN"
fi

if [[ -z "$PORTABLE_CARGO_BIN" ]]; then
    error "Cargo binary not found after installation"
    exit 1
fi

info "\n🔍 Verifying portable toolchain..."
PATH="$CARGO_HOME/bin:$PATH" "$PORTABLE_CARGO_BIN" --version
if [[ -x "$CARGO_HOME/bin/rustc" ]]; then
    PATH="$CARGO_HOME/bin:$PATH" "$CARGO_HOME/bin/rustc" --version
fi

cat <<'OUTRO'

✅ Portable Cargo ready!
Next steps:
  1. Activate in each shell session:
     source ./server/tools/activate-cargo.sh
  2. Build from workspace root:
     cargo build --workspace

Tip: Use NOA_CARGO_ACTIVATE_SILENT=1 to suppress activation logs.
OUTRO
