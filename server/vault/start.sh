#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
DATA_DIR="$SCRIPT_DIR/data"
LOG_DIR="$SCRIPT_DIR/logs"
CONF_FILE="$SCRIPT_DIR/vault.hcl"

mkdir -p "$DATA_DIR" "$LOG_DIR"

if ! command -v vault >/dev/null 2>&1; then
  echo "[vault-start] 'vault' binary not found in PATH. Please install to workspace/tools/bin." >&2
  exit 1
fi

export VAULT_ADDR="http://127.0.0.1:8200"

echo "[vault-start] Starting Vault with config: $CONF_FILE"
exec vault server -config "$CONF_FILE" 2>&1 | tee -a "$LOG_DIR/server.log"

