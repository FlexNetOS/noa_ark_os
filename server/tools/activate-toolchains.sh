#!/usr/bin/env bash
# NOA ARK OS - Activate all portable toolchains (PowerShell, Cargo, Node)
# Usage: source ./server/tools/activate-toolchains.sh

__NOA_PREV_TOOLCHAIN_OPTS="$(set +o)"
set -euo pipefail

if [[ "${NOA_TOOLCHAINS_ACTIVATED:-0}" == "1" && "${NOA_TOOLCHAINS_FORCE:-0}" != "1" ]]; then
  if [[ "${NOA_ACTIVATE_SILENT:-0}" != "1" ]]; then
    echo "Toolchains already active (set NOA_TOOLCHAINS_FORCE=1 to re-run)."
  fi
  eval "$__NOA_PREV_TOOLCHAIN_OPTS"
  unset __NOA_PREV_TOOLCHAIN_OPTS
  return 0 2>/dev/null || exit 0
fi

if [[ "${NOA_TOOLCHAINS_CHAINING:+set}" == "set" ]]; then
  __NOA_PREV_TOOLCHAIN_CHAINING_SET=1
  __NOA_PREV_TOOLCHAIN_CHAINING="$NOA_TOOLCHAINS_CHAINING"
else
  __NOA_PREV_TOOLCHAIN_CHAINING_SET=0
  __NOA_PREV_TOOLCHAIN_CHAINING=""
fi
export NOA_TOOLCHAINS_CHAINING=1

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"

if [[ -f "$SCRIPT_DIR/activate-pwsh.sh" ]]; then
  # shellcheck source=/dev/null
  source "$SCRIPT_DIR/activate-pwsh.sh"
fi

if [[ -f "$SCRIPT_DIR/activate-cargo-wsl.sh" ]]; then
  __NOA_PREV_SKIP_PWSH="${NOA_SKIP_AUTO_PWSH:-0}"
  NOA_SKIP_AUTO_PWSH=1
  # shellcheck source=/dev/null
  source "$SCRIPT_DIR/activate-cargo-wsl.sh"
  NOA_SKIP_AUTO_PWSH="$__NOA_PREV_SKIP_PWSH"
  unset __NOA_PREV_SKIP_PWSH
fi

if [[ -f "$SCRIPT_DIR/activate-node.sh" ]]; then
  __NOA_PREV_SKIP_PWSH="${NOA_SKIP_AUTO_PWSH:-0}"
  NOA_SKIP_AUTO_PWSH=1
  # shellcheck source=/dev/null
  source "$SCRIPT_DIR/activate-node.sh"
  NOA_SKIP_AUTO_PWSH="$__NOA_PREV_SKIP_PWSH"
  unset __NOA_PREV_SKIP_PWSH
fi

export NOA_TOOLCHAINS_ACTIVATED=1

if [[ "${NOA_ACTIVATE_SILENT:-0}" != "1" ]]; then
  echo "All toolchains activated (PowerShell/Cargo/Node)."
fi

if [[ "$__NOA_PREV_TOOLCHAIN_CHAINING_SET" == "1" ]]; then
  export NOA_TOOLCHAINS_CHAINING="$__NOA_PREV_TOOLCHAIN_CHAINING"
else
  unset NOA_TOOLCHAINS_CHAINING
fi
unset __NOA_PREV_TOOLCHAIN_CHAINING_SET
unset __NOA_PREV_TOOLCHAIN_CHAINING

eval "$__NOA_PREV_TOOLCHAIN_OPTS"
unset __NOA_PREV_TOOLCHAIN_OPTS
return 0 2>/dev/null || exit 0
