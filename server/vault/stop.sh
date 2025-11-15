#!/usr/bin/env bash
set -euo pipefail

PIDS=$(pgrep -f "vault server -config") || true
if [ -z "${PIDS:-}" ]; then
  echo "[vault-stop] Vault server not running."
  exit 0
fi
echo "[vault-stop] Stopping Vault PIDs: $PIDS"
kill $PIDS

