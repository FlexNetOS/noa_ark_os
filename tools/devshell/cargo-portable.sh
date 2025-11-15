#!/usr/bin/env bash
# Portable Cargo shim that activates the repository-provided toolchain
# before running the requested Cargo subcommand when the portable
# toolchain is available. Falls back to system Cargo otherwise.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ACTIVATE_SCRIPT="$WORKSPACE_ROOT/server/tools/activate-cargo.sh"
PORTABLE_CARGO="$WORKSPACE_ROOT/server/tools/cargo-portable/bin/cargo.exe"

if [[ -f "$ACTIVATE_SCRIPT" && -f "$PORTABLE_CARGO" ]]; then
    # shellcheck disable=SC1090
    source "$ACTIVATE_SCRIPT"
elif [[ -n "${NOA_FORCE_PORTABLE:-}" ]]; then
    echo "NOA_FORCE_PORTABLE is set but portable toolchain was not found." >&2
    echo "Expected: $PORTABLE_CARGO" >&2
    exit 1
else
    if command -v cargo >/dev/null 2>&1; then
        echo "ℹ️  Portable Cargo bundle not detected; using system cargo on PATH." >&2
    else
        echo "Cargo executable not found. Install Rust or provision the portable toolchain." >&2
        exit 1
    fi
fi

if [[ $# -eq 0 ]]; then
    echo "Usage: $0 <cargo-args>" >&2
    exit 2
fi

cargo "$@"
