#!/usr/bin/env bash
# Unified entrypoint for Cargo with portable/system auto-detection

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ACTIVATOR="$WORKSPACE_ROOT/server/tools/activate-cargo.sh"

if [[ ! -f "$ACTIVATOR" ]]; then
    echo "❌ Unable to locate Cargo activator at $ACTIVATOR" >&2
    exit 1
fi

export NOA_CARGO_ACTIVATE_SILENT=1
# shellcheck disable=SC1090
source "$ACTIVATOR"
unset NOA_CARGO_ACTIVATE_SILENT

if [[ $# -eq 0 ]]; then
    exec cargo
else
    exec cargo "$@"
fi
