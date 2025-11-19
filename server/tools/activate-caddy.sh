#!/usr/bin/env bash
# Activate portable Caddy binary for the current shell session.
# NOTE: Do not use 'set -euo pipefail' in activate scripts intended to be sourced,
# as it can cause the parent shell to exit on error.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
CADDY_HOME="$SCRIPT_DIR/caddy-portable"
CURRENT_BIN="$CADDY_HOME/current"

if [[ -x "$CURRENT_BIN/caddy" ]]; then
    CADDY_BIN="$CURRENT_BIN/caddy"
elif [[ -x "$CURRENT_BIN/caddy.exe" ]]; then
    CADDY_BIN="$CURRENT_BIN/caddy.exe"
else
    echo "❌ Portable Caddy not found. Run ./server/tools/setup-portable-caddy.sh first." >&2
    exit 1
fi

export NOA_CADDY_HOME="$CURRENT_BIN"
export PATH="$CURRENT_BIN:$PATH"

cat <<INFO
✅ Caddy activated from $CURRENT_BIN
   Binary : $CADDY_BIN
   Admin  : ${NOA_CADDY_ADMIN_ENDPOINT:-http://127.0.0.1:2019}
INFO
