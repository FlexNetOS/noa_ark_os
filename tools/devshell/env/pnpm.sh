#!/usr/bin/env bash
set -euo pipefail

DEV_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPLIER="${DEV_DIR}/apply-config.mjs"

if command -v node >/dev/null 2>&1 && [ -f "${APPLIER}" ]; then
  if ! command -v jq >/dev/null 2>&1; then
    echo "❌ 'jq' is required to parse environment configuration. Please install 'jq'." >&2
    exit 1
  fi
  node "${APPLIER}" posix | jq -r 'to_entries[] | "export \(.key)=\(.value|@sh)"' | while read -r line; do
    eval "$line"
  done
  eval "$(node "${APPLIER}" posix)"
else
  echo "⚠️  Node.js is required to hydrate NOA Ark OS devshell environment" >&2
fi

if [[ -n "${PNPM_HOME:-}" ]]; then
  mkdir -p "${PNPM_HOME}"
  export PATH="${PNPM_HOME}:$PATH"
fi

if [[ -n "${NOA_PNPM_REQUIRED:-}" ]] && command -v corepack >/dev/null 2>&1; then
  corepack prepare "pnpm@${NOA_PNPM_REQUIRED}" --activate >/dev/null 2>&1 || true
fi

export PNPM=${PNPM:-pnpm}
