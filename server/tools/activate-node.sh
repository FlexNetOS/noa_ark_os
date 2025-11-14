#!/usr/bin/env bash
# NOA ARK OS - Activate Portable Node + pnpm (WSL/Linux/macOS)
# Usage: source ./server/tools/activate-node.sh

__NOA_PREV_SHELL_OPTS="$(set +o)"
set -euo pipefail

NOA_ACTIVATE_SILENT="${NOA_ACTIVATE_SILENT:-0}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
NODE_HOME_ROOT="$SCRIPT_DIR/node-portable"
CURRENT_LINK="$NODE_HOME_ROOT/current"
COREPACK_DIR="$NODE_HOME_ROOT/corepack"

if [[ ! -d "$NODE_HOME_ROOT" ]]; then
    echo "❌ Portable Node directory not found at $NODE_HOME_ROOT" >&2
    echo "Run ./server/tools/setup-portable-node.sh first." >&2
    eval "$__NOA_PREV_SHELL_OPTS"
    unset __NOA_PREV_SHELL_OPTS
    return 1 2>/dev/null || exit 1
fi

if [[ ! -d "$CURRENT_LINK/bin" ]]; then
    echo "❌ Portable Node missing. Expected resolved path at $CURRENT_LINK/bin" >&2
    echo "Run ./server/tools/setup-portable-node.sh to download Node v20.19.5." >&2
    eval "$__NOA_PREV_SHELL_OPTS"
    unset __NOA_PREV_SHELL_OPTS
    return 1 2>/dev/null || exit 1
fi

export NOA_NODE_HOME="$CURRENT_LINK"
export NOA_NODE_PORTABLE_ROOT="$NODE_HOME_ROOT"
export COREPACK_HOME="$COREPACK_DIR"
export PATH="$NOA_NODE_HOME/bin:$PATH"
export NOA_NODE_ENV=1
export NOA_DEV_ENV=1

if command -v hash >/dev/null 2>&1; then
    hash -r 2>/dev/null || true
fi

if [[ "$NOA_ACTIVATE_SILENT" != "1" ]]; then
    NODE_VERSION_STR="$($NOA_NODE_HOME/bin/node -v 2>/dev/null || echo "node unavailable")"
    PNPM_VERSION_STR="$($NOA_NODE_HOME/bin/pnpm -v 2>/dev/null || echo "pnpm unavailable")"
    echo
    echo "🟢 Portable Node activated"
    echo "   NODE_HOME  = $NOA_NODE_HOME"
    echo "   COREPACK   = $COREPACK_HOME"
    echo "   PATH head  = $NOA_NODE_HOME/bin"
    echo "   node       = $NODE_VERSION_STR"
    echo "   pnpm       = $PNPM_VERSION_STR"
    echo
fi

eval "$__NOA_PREV_SHELL_OPTS"
unset __NOA_PREV_SHELL_OPTS
return 0 2>/dev/null || exit 0
