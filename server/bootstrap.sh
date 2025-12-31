#!/usr/bin/env bash
# NOA ARK OS - Server bootstrap helper (Linux/WSL/macOS)
# One-time helper when the repo/server is moved to a new location.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SERVER_ROOT="$WORKSPACE_ROOT/server"
VAULT_DIR="$SERVER_ROOT/vault"
VAULT_HCL="$VAULT_DIR/vault.hcl"
VAULT_DATA_DIR="$VAULT_DIR/data"
GATEWAY_AUTH_CONFIG_SH="$VAULT_DIR/configure-gateway-auth-simple.sh"
GATEWAY_AUTH_JSON="$VAULT_DIR/runtime/gateway_auth.json"

PYTHON_BIN="$(command -v python3 2>/dev/null || true)"

COLOR_INFO="\033[1;34m"
COLOR_OK="\033[1;32m"
COLOR_WARN="\033[1;33m"
COLOR_ERR="\033[1;31m"
COLOR_RESET="\033[0m"

info()  { echo -e "${COLOR_INFO}[*]${COLOR_RESET} $*"; }
ok()    { echo -e "${COLOR_OK}[ok]${COLOR_RESET} $*"; }
warn()  { echo -e "${COLOR_WARN}[!!]${COLOR_RESET} $*"; }
error() { echo -e "${COLOR_ERR}[xx]${COLOR_RESET} $*" >&2; }

RUN_SERVER=1
BUILD_ONLY=0

for arg in "$@"; do
  case "$arg" in
    --no-run)
      RUN_SERVER=0
      ;;
    --build-only)
      RUN_SERVER=0
      BUILD_ONLY=1
      ;;
    --help|-h)
      cat <<EOF
NOA ARK OS - Server Bootstrap

Usage:
  bash server/bootstrap.sh            # Rebind paths, build, and run server
  bash server/bootstrap.sh --no-run   # Rebind paths and build only
  bash server/bootstrap.sh --build-only

This script:
  - Detects the current workspace root
  - Updates Vault config paths under server/vault
  - Builds (and optionally runs) the unified server binary
EOF
      exit 0
      ;;
    *)
      warn "Unknown argument: $arg (ignored)"
      ;;
  esac
done

echo
info "NOA ARK OS - Server bootstrap"
info "Workspace root: $WORKSPACE_ROOT"
echo

update_vault_hcl_path() {
  if [[ ! -f "$VAULT_HCL" ]]; then
    warn "Vault config not found at $VAULT_HCL (skipping Vault path rebinding)"
    return 0
  fi

  if [[ -z "$PYTHON_BIN" ]]; then
    warn "python3 not found; skipping Vault path rebinding (server build will continue)"
    return 0
  fi

  mkdir -p "$VAULT_DATA_DIR"

  local target_path="$VAULT_DATA_DIR"
  info "Rebinding Vault storage path to: $target_path"

  "$PYTHON_BIN" - "$VAULT_HCL" "$target_path" <<'PY'
import sys
from pathlib import Path

if len(sys.argv) != 3:
    print("usage: <script> VAULT_HCL TARGET_PATH", file=sys.stderr)
    sys.exit(1)

vault_hcl = Path(sys.argv[1])
target_path = sys.argv[2]

text = vault_hcl.read_text(encoding="utf-8").splitlines()
out = []
in_raft = False
updated = False

for line in text:
    stripped = line.strip()
    if stripped.startswith('storage "raft"'):
        in_raft = True
        out.append(line)
        continue
    if in_raft and stripped.startswith("path"):
        # Preserve indentation before "path"
        prefix = line.split("p", 1)[0]
        out.append(f'{prefix}path    = "{target_path}"')
        in_raft = False
        updated = True
        continue
    out.append(line)

if not updated:
    print("[bootstrap] Warning: did not find path line in storage \"raft\" block", file=sys.stderr)

vault_hcl.write_text("\n".join(out) + "\n", encoding="utf-8")
PY

  ok "Vault storage path updated in vault.hcl"
}

update_gateway_auth_config_path() {
  if [[ ! -f "$GATEWAY_AUTH_CONFIG_SH" ]]; then
    warn "Gateway auth helper not found at $GATEWAY_AUTH_CONFIG_SH (skipping)"
    return 0
  fi

  if [[ -z "$PYTHON_BIN" ]]; then
    warn "python3 not found; skipping gateway auth config path rebinding (server build will continue)"
    return 0
  fi

  local target_path="$GATEWAY_AUTH_JSON"
  info "Rebinding gateway auth JSON path to: $target_path"

  "$PYTHON_BIN" - "$GATEWAY_AUTH_CONFIG_SH" "$target_path" <<'PY'
import sys
from pathlib import Path

if len(sys.argv) != 3:
    print("usage: <script> CONFIG_SH TARGET_PATH", file=sys.stderr)
    sys.exit(1)

config_sh = Path(sys.argv[1])
target_path = sys.argv[2]

lines = config_sh.read_text(encoding="utf-8").splitlines()
out = []
updated = False

for line in lines:
    stripped = line.lstrip()
    if stripped.startswith("AUTH_CONFIG_FILE="):
        prefix = line.split("A", 1)[0]
        out.append(f'{prefix}AUTH_CONFIG_FILE="{target_path}"')
        updated = True
    else:
        out.append(line)

if not updated:
    print("[bootstrap] Warning: did not find AUTH_CONFIG_FILE line in configure-gateway-auth-simple.sh", file=sys.stderr)

config_sh.write_text("\n".join(out) + "\n", encoding="utf-8")
PY

  ok "Gateway auth config path updated in configure-gateway-auth-simple.sh"
}

choose_cargo() {
  local wrapper="$WORKSPACE_ROOT/tools/devshell/portable-cargo.sh"
  if [[ -x "$wrapper" ]]; then
    echo "$wrapper"
  else
    echo "cargo"
  fi
}

ensure_cargo_available() {
  local wrapper="$WORKSPACE_ROOT/tools/devshell/portable-cargo.sh"
  if [[ -x "$wrapper" ]]; then
    return 0
  fi
  if command -v cargo >/dev/null 2>&1; then
    return 0
  fi
  error "No Cargo runner found. Install Rust (rustup) or configure the portable toolchain (see server/tools/README.md)."
  exit 1
}

build_server() {
  local cargo_bin
  cargo_bin="$(choose_cargo)"

  info "Using Cargo runner: $cargo_bin"
  info "Building noa-unified-server (this may take a while on first run)..."

  (cd "$WORKSPACE_ROOT" && "$cargo_bin" build --manifest-path "$SERVER_ROOT/Cargo.toml" --bin noa-unified-server)

  ok "Build complete"
}

run_server() {
  local cargo_bin
  cargo_bin="$(choose_cargo)"

  info "Launching noa-unified-server..."
  echo
  (cd "$WORKSPACE_ROOT" && "$cargo_bin" run --manifest-path "$SERVER_ROOT/Cargo.toml" --bin noa-unified-server)
}

update_vault_hcl_path
update_gateway_auth_config_path

ensure_cargo_available

build_server

if [[ "$RUN_SERVER" -eq 1 && "$BUILD_ONLY" -eq 0 ]]; then
  run_server
else
  info "Skipping server run (per flags)."
fi

echo
ok "Server bootstrap complete."
echo "Workspace root: $WORKSPACE_ROOT"
echo "Server root:    $SERVER_ROOT"
echo
