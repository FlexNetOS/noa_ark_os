#!/usr/bin/env bash
# NOA ARK OS - Activate all portable toolchains (PowerShell, Cargo, Node)
# Usage: source ./server/tools/activate-toolchains.sh

__NOA_PREV_TOOLCHAIN_OPTS="$(set +o)"
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"

if [[ -f "$SCRIPT_DIR/activate-pwsh.sh" ]]; then
  # shellcheck source=/dev/null
  source "$SCRIPT_DIR/activate-pwsh.sh"
fi

if [[ -f "$SCRIPT_DIR/activate-cargo-wsl.sh" ]]; then
  # shellcheck source=/dev/null
  source "$SCRIPT_DIR/activate-cargo-wsl.sh"
fi

if [[ -f "$SCRIPT_DIR/activate-node.sh" ]]; then
  # shellcheck source=/dev/null
  source "$SCRIPT_DIR/activate-node.sh"
fi

if [[ "${NOA_ACTIVATE_SILENT:-0}" != "1" ]]; then
  echo "All toolchains activated (PowerShell/Cargo/Node)."
fi

eval "$__NOA_PREV_TOOLCHAIN_OPTS"
unset __NOA_PREV_TOOLCHAIN_OPTS
return 0 2>/dev/null || exit 0
