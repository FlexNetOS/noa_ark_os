#!/usr/bin/env bash
# HEALED: This script was completely missing from v7 - restored from v5
set -euo pipefail
op="${1:-}"; target="${2:-}"
if [[ -z "$op" || -z "$target" ]]; then
  echo "Usage: $0 <seal-immutable|unseal-immutable|verity-enable|verity-status|verity-sign> <file> [key] [cert]"; exit 2
fi
case "$op" in
  seal-immutable)
    if ! command -v chattr >/dev/null 2>&1; then echo "[fs] chattr not found"; exit 1; fi
    sudo chattr +i "$target"
    echo "[fs] immutable set on $target"
    ;;
  unseal-immutable)
    if ! command -v chattr >/dev/null 2>&1; then echo "[fs] chattr not found"; exit 1; fi
    sudo chattr -i "$target"
    echo "[fs] immutable cleared on $target"
    ;;
  verity-enable)
    if ! command -v fsverity >/dev/null 2>&1; then echo "[fs] fsverity tool not found"; exit 1; fi
    sudo fsverity enable "$target" --hash-alg sha256 || { echo "[fs] fsverity enable failed"; exit 1; }
    echo "[fs] fs-verity enabled on $target"
    ;;
  verity-status)
    if command -v fsverity >/dev/null 2>&1; then
      fsverity measure "$target" || true
    else
      echo "[fs] fsverity tool not found"
    fi
    if command -v lsattr >/dev/null 2>&1; then
      lsattr "$target" || true
    fi
    ;;
  verity-sign)
    key="${3:-${FSV_KEY:-/path/key.pem}}"
    cert="${4:-${FSV_CERT:-/path/cert.pem}}"
    if ! command -v fsverity >/dev/null 2>&1; then echo "[fs] fsverity tool not found"; exit 1; fi
    if [[ ! -f "$key" || ! -f "$cert" ]]; then echo "[fs] key or cert file not found: $key, $cert"; exit 1; fi
    fsverity sign "$target" "$key" "$cert" || { echo "[fs] fsverity sign failed"; exit 1; }
    echo "[fs] fs-verity signed $target"
    ;;
  *)
    echo "Unknown op: $op"; exit 2;;
esac