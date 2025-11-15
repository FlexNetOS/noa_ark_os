#!/usr/bin/env bash
# NOA ARK OS - Activate Cargo environment (portable or system)
# Usage: source ./server/tools/activate-cargo.sh

set -euo pipefail

SILENT_MODE="${NOA_CARGO_ACTIVATE_SILENT:-0}"

log() {
    if [[ "$SILENT_MODE" != "1" ]]; then
        echo -e "$1"
    fi
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
DEV_SHELL_STATE_DIR="$WORKSPACE_ROOT/tools/devshell/state"
mkdir -p "$DEV_SHELL_STATE_DIR"

PORTABLE_CARGO_HOME="$WORKSPACE_ROOT/server/tools/cargo-portable"
PORTABLE_RUSTUP_HOME="$WORKSPACE_ROOT/server/tools/rustup-portable"
PORTABLE_CARGO_BIN_UNIX="$PORTABLE_CARGO_HOME/bin/cargo"
PORTABLE_CARGO_BIN_WIN="$PORTABLE_CARGO_HOME/bin/cargo.exe"

PORTABLE_DETECTED="false"
if [[ -f "$PORTABLE_CARGO_BIN_UNIX" || -f "$PORTABLE_CARGO_BIN_WIN" ]]; then
    PORTABLE_DETECTED="true"
fi

SYSTEM_RUSTUP_AVAILABLE="false"
if command -v rustup >/dev/null 2>&1; then
    SYSTEM_RUSTUP_AVAILABLE="true"
fi

MODE=""
PATH_MODIFIED="false"

log "\n🔧 Activating Cargo environment..."
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [[ "$PORTABLE_DETECTED" == "true" ]]; then
    MODE="portable"
    CARGO_HOME="$PORTABLE_CARGO_HOME"
    RUSTUP_HOME="$PORTABLE_RUSTUP_HOME"
    PATH_CANDIDATE="$CARGO_HOME/bin"
    export CARGO_HOME RUSTUP_HOME
    if [[ ":$PATH:" != *":$PATH_CANDIDATE:"* ]]; then
        export PATH="$PATH_CANDIDATE:$PATH"
        PATH_MODIFIED="true"
    fi
else
    MODE="system"
    if [[ -z "${CARGO_HOME:-}" ]]; then
        CARGO_HOME="$HOME/.cargo"
    fi
    if [[ -z "${RUSTUP_HOME:-}" ]]; then
        if [[ "$SYSTEM_RUSTUP_AVAILABLE" == "true" ]]; then
            if RUSTUP_QUERY_HOME=$(rustup show home 2>/dev/null); then
                RUSTUP_HOME="$RUSTUP_QUERY_HOME"
            else
                RUSTUP_HOME="$HOME/.rustup"
            fi
        else
            RUSTUP_HOME="$HOME/.rustup"
        fi
    fi
    export CARGO_HOME RUSTUP_HOME
    PATH_CANDIDATE="$CARGO_HOME/bin"
    if [[ ":$PATH:" != *":$PATH_CANDIDATE:"* ]]; then
        export PATH="$PATH_CANDIDATE:$PATH"
        PATH_MODIFIED="true"
    fi
fi

if ! command -v cargo >/dev/null 2>&1; then
    log "\n❌ ERROR: Cargo could not be found in the configured environment."
    if [[ "$MODE" == "portable" ]]; then
        log "\nExpected location: $CARGO_HOME/bin/cargo"
        log "\nPlease run setup first:"
        log "  ./server/tools/setup-portable-cargo.ps1"
    else
        log "\nInstall Rustup with:"
        log "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
    fi
    return 1 2>/dev/null || exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
    echo "Error: python3 is required but was not found in your PATH." >&2
    echo "Please install Python 3 and ensure it is available as 'python3'." >&2
    return 1 2>/dev/null || exit 1
fi

TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
STATUS_JSON_FILE="$DEV_SHELL_STATE_DIR/cargo-env.json"
STATUS_YAML_FILE="$DEV_SHELL_STATE_DIR/cargo-env.yaml"

python3 - <<'PY' "$STATUS_JSON_FILE" "$STATUS_YAML_FILE" "$TIMESTAMP" "$MODE" "$WORKSPACE_ROOT" "$CARGO_HOME" "$RUSTUP_HOME" "$PATH_MODIFIED" "$PORTABLE_DETECTED" "$SYSTEM_RUSTUP_AVAILABLE"
import json
import sys
from pathlib import Path

json_path, yaml_path, timestamp, mode, workspace_root, cargo_home, rustup_home, path_modified, portable_detected, system_rustup_available = sys.argv[1:]

def to_bool(value: str) -> bool:
    return value.lower() in {"1", "true", "yes", "on"}

status = {
    "timestamp": timestamp,
    "mode": mode,
    "workspace_root": workspace_root,
    "cargo_home": cargo_home,
    "rustup_home": rustup_home,
    "path_modified": to_bool(path_modified),
    "portable_detected": to_bool(portable_detected),
    "system_rustup_available": to_bool(system_rustup_available),
}

Path(json_path).write_text(json.dumps(status, indent=2) + "\n", encoding="utf-8")

def yaml_value(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, str):
        return "'" + value.replace("'", "''") + "'"
    return json.dumps(value)

with open(yaml_path, "w", encoding="utf-8") as fh:
    for key in ["timestamp", "mode", "workspace_root", "cargo_home", "rustup_home", "path_modified", "portable_detected", "system_rustup_available"]:
        fh.write(f"{key}: {yaml_value(status[key])}\n")
PY

if [[ "$SILENT_MODE" != "1" ]]; then
    log "\n✅ Cargo environment ready (${MODE})"
    log "\nEnvironment:"
    echo "  CARGO_HOME   = $CARGO_HOME"
    echo "  RUSTUP_HOME  = $RUSTUP_HOME"
    if [[ "$PATH_MODIFIED" == "true" ]]; then
        echo "  PATH         = [$PATH_CANDIDATE prepended]"
    else
        echo "  PATH         = [unchanged]"
    fi

    log "\nVersions:"
    cargo --version
    if command -v rustc >/dev/null 2>&1; then
        rustc --version
    else
        log "rustc not found in PATH"
    fi

    log "\n💡 Tips:"
    echo "  • Run 'cargo build' to build projects"
    echo "  • Run 'cargo run' to run projects"
    echo "  • Run 'cargo test' to run tests"
    echo "  • This activation is for the current shell session only"
    echo
fi
