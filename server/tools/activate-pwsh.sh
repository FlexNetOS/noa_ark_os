#!/usr/bin/env bash
# NOA ARK OS - Activate Portable PowerShell runtime
# Usage: source ./server/tools/activate-pwsh.sh

__NOA_PREV_PWSH_OPTS="$(set +o)"
set -euo pipefail

NOA_ACTIVATE_SILENT="${NOA_ACTIVATE_SILENT:-0}"
PWSH_VERSION="${PWSH_VERSION:-7.4.5}"
REQUESTED_PLATFORM="${NOA_PWSH_PLATFORM:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PORTABLE_ROOT="$SCRIPT_DIR/pwsh-portable"
CURRENT_LINK="$PORTABLE_ROOT/current"
BIN_DIR="$PORTABLE_ROOT/bin"
MANIFEST_PATH="$SCRIPT_DIR/pwsh-portable.manifest.json"

log() {
    if [[ "$NOA_ACTIVATE_SILENT" == "1" ]]; then
        return
    fi
    printf '%s\n' "$1"
}

fail_missing_bundle() {
    log "❌ Portable PowerShell not provisioned under $PORTABLE_ROOT"
    log "   Run ./server/tools/setup-portable-pwsh.sh to download v${PWSH_VERSION}."
    eval "$__NOA_PREV_PWSH_OPTS"
    unset __NOA_PREV_PWSH_OPTS
    return 1 2>/dev/null || exit 1
}

if [[ -n "${NOA_PWSH_ENV:-}" ]]; then
    log "ℹ️  Portable PowerShell already active ($POWERSHELL_BIN)"
    eval "$__NOA_PREV_PWSH_OPTS"
    unset __NOA_PREV_PWSH_OPTS
    return 0 2>/dev/null || exit 0
fi

if [[ ! -d "$PORTABLE_ROOT" ]]; then
    fail_missing_bundle
fi

PWSH_CANDIDATES=(
    "$BIN_DIR/pwsh"
    "$BIN_DIR/pwsh.exe"
    "$CURRENT_LINK/pwsh"
    "$CURRENT_LINK/pwsh.exe"
)

if [[ -n "$REQUESTED_PLATFORM" ]]; then
    if [[ -f "$BIN_DIR/$REQUESTED_PLATFORM/pwsh" ]]; then
        PWSH_CANDIDATES=("$BIN_DIR/$REQUESTED_PLATFORM/pwsh" "${PWSH_CANDIDATES[@]}")
    fi
    if [[ -f "$BIN_DIR/$REQUESTED_PLATFORM/pwsh.exe" ]]; then
        PWSH_CANDIDATES=("$BIN_DIR/$REQUESTED_PLATFORM/pwsh.exe" "${PWSH_CANDIDATES[@]}")
    fi
fi

POWERSHELL_BIN=""
for cand in "${PWSH_CANDIDATES[@]}"; do
    if [[ -f "$cand" ]]; then
        POWERSHELL_BIN="$cand"
        break
    fi
done

if [[ -z "$POWERSHELL_BIN" ]]; then
    log "❌ No pwsh executable found in $CURRENT_LINK"
    fail_missing_bundle
fi

PWSH_DIR="$(cd "$(dirname "$POWERSHELL_BIN")" && pwd)"
resolve_manifest_platform() {
    if [[ ! -f "$MANIFEST_PATH" ]]; then
        return
    fi
    if ! command -v python3 >/dev/null 2>&1; then
        return
    fi
    python3 - <<'PY' "$MANIFEST_PATH" "$POWERSHELL_BIN"
import json, sys
from pathlib import Path
manifest = Path(sys.argv[1])
binary = Path(sys.argv[2]).resolve()
try:
    data = json.loads(manifest.read_text())
except Exception:
    raise SystemExit
platforms = data.get("platforms")
if isinstance(platforms, list):
    for entry in platforms:
        rel = entry.get("binary")
        if not rel:
            continue
        binary_path = (manifest.parent / rel).resolve()
        if binary_path == binary:
            print(entry.get("platform", ""))
            raise SystemExit
default = data.get("platform") or data.get("default_platform")
if default:
    print(default)
PY
}
RESOLVED_PLATFORM="$(resolve_manifest_platform || true)"
export POWERSHELL_BIN="$POWERSHELL_BIN"
export NOA_PWSH_ENV=1
export NOA_PWSH_PORTABLE_ROOT="$PORTABLE_ROOT"
export NOA_PWSH_MANIFEST="$MANIFEST_PATH"
export NOA_PWSH_PLATFORM_RESOLVED="${RESOLVED_PLATFORM:-}"
export PATH="$PWSH_DIR:$PATH"
if command -v hash >/dev/null 2>&1; then
    hash -r 2>/dev/null || true
fi

if [[ "$NOA_ACTIVATE_SILENT" != "1" ]]; then
    log ""
    log "🟢 Portable PowerShell activated"
    log "   POWERSHELL_BIN = $POWERSHELL_BIN"
    if [[ -f "$MANIFEST_PATH" ]]; then
        if command -v sha256sum >/dev/null 2>&1; then
            MANIFEST_SHA=$(sha256sum "$MANIFEST_PATH" 2>/dev/null | awk '{print $1}')
        elif command -v shasum >/dev/null 2>&1; then
            MANIFEST_SHA=$(shasum -a 256 "$MANIFEST_PATH" 2>/dev/null | awk '{print $1}')
        else
            MANIFEST_SHA="unavailable"
        fi
        log "   manifest      = $MANIFEST_PATH"
        log "   manifest sha  = ${MANIFEST_SHA:-unavailable}"
    fi
    if [[ -n "$RESOLVED_PLATFORM" ]]; then
        log "   platform     = $RESOLVED_PLATFORM"
    fi
    "$POWERSHELL_BIN" --version 2>/dev/null || true
    log ""
fi

eval "$__NOA_PREV_PWSH_OPTS"
unset __NOA_PREV_PWSH_OPTS
return 0 2>/dev/null || exit 0
