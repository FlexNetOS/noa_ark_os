#!/usr/bin/env bash
# Launches the full NOA Ark OS developer surface (UI API + Vibe Kanban UI)
# and runs the required build/test checks beforehand.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

MODE="launch"
SKIP_TESTS=0
SKIP_BUILD=0
DRY_RUN=0
UI_HOST="${UI_HOST:-127.0.0.1}"
UI_PORT="${UI_PORT:-3030}"
UI_API_BIND="${NOA_UI_API_ADDR:-127.0.0.1:8787}"

usage() {
  cat <<'EOF'
Usage: scripts/dev/full_system_launch.sh [options]

Options:
  --verify        Run dependency install/build/test checks and exit.
  --dry-run       Print the steps that would be executed without running them.
  --skip-tests    Skip pnpm/cargo test suites (use with caution).
  --skip-build    Skip pnpm/cargo build steps (assumes artifacts already exist).
  -h, --help      Show this help text.

Environment overrides:
  UI_HOST             Hostname for the Next.js dev server (default: 127.0.0.1)
  UI_PORT             Port for the Next.js dev server (default: 3030)
  NOA_UI_API_ADDR     Host:port binding for the Rust UI API (default: 127.0.0.1:8787)
  NOA_UI_DROP_ROOT    Drop-in root directory for CRC uploads
  CRC_CAS_DIR         Storage location for CAS objects
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --verify)
      MODE="verify"
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --skip-tests)
      SKIP_TESTS=1
      shift
      ;;
    --skip-build)
      SKIP_BUILD=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

LOG_DIR="$ROOT_DIR/build_output/system-launch"
mkdir -p "$LOG_DIR"
API_LOG="$LOG_DIR/ui-api.log"
UI_LOG="$LOG_DIR/ui-dev.log"

log() {
  printf '[%s] %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*"
}

run_step() {
  local description="$1"
  shift
  log "▶ $description"
  "$@"
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

activate_toolchains() {
  if [[ -z "${NOA_CARGO_ENV:-}" && -f "$ROOT_DIR/server/tools/activate-cargo.sh" ]]; then
    # shellcheck disable=SC1091
    source "$ROOT_DIR/server/tools/activate-cargo.sh"
  fi
  if [[ -z "${NOA_NODE_ENV:-}" && -f "$ROOT_DIR/server/tools/activate-node.sh" ]]; then
    # shellcheck disable=SC1091
    source "$ROOT_DIR/server/tools/activate-node.sh"
  fi
}

prepare_env() {
  activate_toolchains
  require_cmd pnpm
  require_cmd cargo

  if [[ "$UI_API_BIND" == http://* || "$UI_API_BIND" == https://* ]]; then
    UI_API_URL="$UI_API_BIND"
    UI_API_BIND="${UI_API_BIND#http://}"
    UI_API_BIND="${UI_API_BIND#https://}"
  else
    UI_API_URL="http://$UI_API_BIND"
  fi

  export NOA_UI_DROP_ROOT="${NOA_UI_DROP_ROOT:-$ROOT_DIR/crc/drop-in/incoming}"
  export CRC_CAS_DIR="${CRC_CAS_DIR:-$ROOT_DIR/storage/cas}"
  export NEXT_PUBLIC_UI_API="${NEXT_PUBLIC_UI_API:-$UI_API_URL}"
  export UPLOAD_BRIDGE_URL="${UPLOAD_BRIDGE_URL:-$UI_API_URL}"
  mkdir -p "$NOA_UI_DROP_ROOT" "$CRC_CAS_DIR"
  mkdir -p "$NOA_UI_DROP_ROOT"/repos "$NOA_UI_DROP_ROOT"/forks "$NOA_UI_DROP_ROOT"/mirrors "$NOA_UI_DROP_ROOT"/stale
}

install_deps() {
  run_step "Installing pnpm workspace dependencies" pnpm install --frozen-lockfile
}

build_workspace() {
  if [[ "$SKIP_BUILD" -eq 1 ]]; then
    log "⚠ Skipping build phase per flag"
    return
  fi
  run_step "Building Rust workspace" cargo build --workspace
  run_step "Building Vibe Kanban UI" pnpm build
}

run_tests() {
  if [[ "$SKIP_TESTS" -eq 1 ]]; then
    log "⚠ Skipping tests per flag"
    return
  fi
  run_step "Running pnpm test suite" pnpm test
  run_step "Running cargo test suite" cargo test --workspace
}

API_PID=""
UI_PID=""

stop_process() {
  local pid_var="$1"
  local pid_value="${!pid_var:-}"
  if [[ -n "$pid_value" ]] && kill -0 "$pid_value" >/dev/null 2>&1; then
    kill "$pid_value" >/dev/null 2>&1 || true
    wait "$pid_value" 2>/dev/null || true
  fi
}

cleanup() {
  local status=$?
  trap - EXIT INT TERM
  stop_process API_PID
  stop_process UI_PID
  exit "$status"
}

start_services() {
  trap cleanup EXIT INT TERM

  log "▶ Launching NOA UI API on $UI_API_BIND"
  (
    set -euo pipefail
    cargo run -p noa_ui_api -- "$UI_API_BIND"
  ) |& tee "$API_LOG" &
  API_PID=$!

  log "▶ Launching Vibe Kanban UI on http://$UI_HOST:$UI_PORT"
  (
    set -euo pipefail
    HOST="$UI_HOST" PORT="$UI_PORT" pnpm --filter vibe-kanban dev
  ) |& tee "$UI_LOG" &
  UI_PID=$!

  log "System launch ready"
  log "• UI:        http://$UI_HOST:$UI_PORT"
  log "• UI API:    ${UI_API_URL:-http://$UI_API_BIND}"
  log "• Drop root: $NOA_UI_DROP_ROOT"
  log "Use Ctrl+C to stop both processes. Logs in $LOG_DIR"

  if (( BASH_VERSINFO[0] > 4 || (BASH_VERSINFO[0] == 4 && BASH_VERSINFO[1] >= 3) )); then
    wait -n || true
  else
    wait "$API_PID" || true
    wait "$UI_PID" || true
  fi
}

if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "[dry-run] Would prepare toolchains, install deps, build, test, and start services."
  exit 0
fi

prepare_env
install_deps
build_workspace
run_tests

if [[ "$MODE" == "verify" ]]; then
  log "Verification complete."
  exit 0
fi

start_services
