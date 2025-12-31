#!/usr/bin/env bash
set -euo pipefail
op="${1:-}"; target="${2:-}"
case "$op" in
  verity-enable)
    if ! command -v fsverity >/dev/null 2>&1; then echo "[fs] fsverity tool not found"; exit 1; fi
    sudo fsverity enable "$target" --hash-alg sha256 || { echo "[fs] fsverity enable failed"; exit 1; }
    echo "[fs] fs-verity enabled on $target"
    ;;
  verity-status)
    if command -v fsverity >/dev/null 2>&1; then fsverity measure "$target" || true; else echo "[fs] fsverity tool not found"; fi
    if command -v lsattr >/dev/null 2>&1; then lsattr "$target" || true; fi
    ;;
  verity-sign)
    if ! command -v fsverity >/dev/null 2>&1; then echo "[fs] fsverity tool not found"; exit 1; fi
    key="${3:-}"; cert="${4:-}"
    if [[ -z "$key" || -z "$cert" ]]; then echo "Usage: $0 verity-sign <file> <key.pem> <cert.pem>"; exit 2; fi
    fsverity sign "$target" --key "$key" --cert "$cert" --hash-alg sha256 > "${target}.p7sig"
    echo "[fs] fs-verity signature -> ${target}.p7sig"
    ;;
  *)
    echo "Usage: $0 <verity-enable|verity-status|verity-sign> <file> [key cert]"; exit 2;;
esac
