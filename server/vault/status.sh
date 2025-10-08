#!/usr/bin/env bash
set -euo pipefail
export VAULT_ADDR=${VAULT_ADDR:-http://127.0.0.1:8200}

if ! command -v vault >/dev/null 2>&1; then
  echo "[vault-status] 'vault' binary not found in PATH." >&2
  exit 1
fi

vault status || true

