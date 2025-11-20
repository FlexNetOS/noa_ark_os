#!/usr/bin/env bash
set -euo pipefail

LAUNCH_MODE="full"
RUN_TESTS=1
SKIP_NOTEBOOK=0
ROOT=""
LOG_DIR=""
PY_ENV_DIR=""
PYTHON_BIN="python3"
# pnpm/corepack are configured below after activation
PNPM_REQUIRED_VERSION=""
PNPM_AGENT=""
declare -a PNPM_BIN
CARGO_BIN="cargo"
MAKE_LAUNCH_NOTE=""

usage() {
  cat <<'EOF'
Usage: scripts/full_stack_launch.sh [options]

Options:
  --prepare-only     Run dependency installs and build/test steps, then exit.
  --skip-tests       Skip the long-running pnpm/cargo test suites during prepare.
  --skip-notebook    Do not start the Jupyter notebook server.
  --help             Show this message and exit.

With no flags the script prepares the workspace and launches the full stack.
EOF
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --prepare-only)
        LAUNCH_MODE="prepare"
        shift
        ;;
      --skip-tests)
        RUN_TESTS=0
        shift
        ;;
      --skip-notebook)
        SKIP_NOTEBOOK=1
        shift
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      *)
        echo "Unknown option: $1" >&2
        usage
        exit 2
        ;;
    esac
  done
}

info() {
  printf '[%s] %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*"
}

fail() {
  printf '\n[ERROR] %s\n' "$1" >&2
  exit "${2:-1}"
}

ensure_root() {
  if ! ROOT=$(git rev-parse --show-toplevel 2>/dev/null); then
    fail "Run this script inside the noa_ark_os repository"
  fi
  cd "$ROOT"
  LOG_DIR="$ROOT/build_output/system-launch"
  PY_ENV_DIR="$ROOT/noa_ark_os_env"
  mkdir -p "$LOG_DIR"
}

activate_toolchains() {
  if [[ -z "${NOA_CARGO_ENV:-}" ]]; then
    if [[ -f "$ROOT/server/tools/activate-cargo-wsl.sh" ]]; then
      info "Activating portable Cargo toolchain (WSL/Linux)"
      # shellcheck source=/dev/null
      source "$ROOT/server/tools/activate-cargo-wsl.sh"
    elif [[ -f "$ROOT/server/tools/activate-cargo.sh" ]]; then
      info "Activating portable Cargo toolchain"
      # shellcheck source=/dev/null
      source "$ROOT/server/tools/activate-cargo.sh"
    else
      info "Cargo activator not found; assuming cargo is on PATH"
    fi
  fi
  if [[ -z "${NOA_NODE_ENV:-}" ]]; then
    if [[ -f "$ROOT/server/tools/activate-node.sh" ]]; then
      info "Activating portable Node toolchain"
      # shellcheck source=/dev/null
      source "$ROOT/server/tools/activate-node.sh"
    else
      info "Node activator not found; assuming node/pnpm are on PATH"
    fi
  fi
}

detect_pnpm_version() {
  local config_json="$ROOT/tools/devshell/config.json"
  local version=""
  if command -v node >/dev/null 2>&1 && [[ -f "$config_json" ]]; then
    version=$(node -e '
      const fs=require("fs");
      const p=process.argv[1];
      try{const j=JSON.parse(fs.readFileSync(p,"utf8"));
        process.stdout.write((j?.pnpm?.requiredVersion)||"");
      }catch{process.stdout.write("");}
    ' "$config_json" 2>/dev/null || true)
  fi
  [[ -z "$version" || "$version" == "None" ]] && version="8.15.4"
  printf '%s' "$version"
}

prepare_node_pnpm() {
  if command -v corepack >/dev/null 2>&1; then
    PNPM_REQUIRED_VERSION=$(detect_pnpm_version)
    PNPM_AGENT="pnpm/${PNPM_REQUIRED_VERSION}"
    corepack prepare "pnpm@${PNPM_REQUIRED_VERSION}" --activate >/dev/null 2>&1 || true
    export npm_config_user_agent="$PNPM_AGENT"
    PNPM_BIN=(corepack pnpm)
  else
    # Fallback: assume pnpm on PATH; still set agent to reduce warnings
    PNPM_REQUIRED_VERSION=$(detect_pnpm_version)
    PNPM_AGENT="pnpm/${PNPM_REQUIRED_VERSION}"
    export npm_config_user_agent="$PNPM_AGENT"
    PNPM_BIN=(pnpm)
  fi
}

setup_python_env() {
  if [[ ! -d "$PY_ENV_DIR" ]]; then
    info "Creating Python virtual environment"
    "$PYTHON_BIN" -m venv "$PY_ENV_DIR"
  fi
  # shellcheck disable=SC1090
  source "$PY_ENV_DIR/bin/activate"
  info "Installing notebook dependencies"
  pip install --upgrade pip >/dev/null
  pip install -r "$ROOT/notebooks/requirements.txt" >/dev/null
  pip install jupyter >/dev/null
  deactivate
}

run_prepare_phase() {
  info "Installing Node dependencies"
  env npm_config_user_agent="$PNPM_AGENT" "${PNPM_BIN[@]}" install --frozen-lockfile

  info "Running pnpm lint/build/typecheck"
  env npm_config_user_agent="$PNPM_AGENT" "${PNPM_BIN[@]}" lint
  env npm_config_user_agent="$PNPM_AGENT" "${PNPM_BIN[@]}" build
  env npm_config_user_agent="$PNPM_AGENT" "${PNPM_BIN[@]}" typecheck

  if [[ $RUN_TESTS -eq 1 ]]; then
    info "Running pnpm test suite"
    env npm_config_user_agent="$PNPM_AGENT" "${PNPM_BIN[@]}" test
  else
    info "Skipping pnpm tests"
  fi

  info "Building Rust workspace"
  "$CARGO_BIN" build --workspace

  info "Formatting Rust workspace"
  "$CARGO_BIN" fmt --all

  info "Running cargo clippy"
  "$CARGO_BIN" clippy --workspace -- -D warnings

  if [[ $RUN_TESTS -eq 1 ]]; then
    info "Running cargo test suite"
    "$CARGO_BIN" test --workspace
  else
    info "Skipping cargo tests"
  fi

  info "Notebook sync"
  "$CARGO_BIN" run -p noa_symbol_graph --bin notebook_watcher -- --once .
  env npm_config_user_agent="$PNPM_AGENT" "${PNPM_BIN[@]}" notebooks:sync
}

declare -A SERVICE_PIDS
SERVICE_ORDER=()

start_service() {
  local name="$1"
  shift
  local log_file="$LOG_DIR/${name}.log"
  : >"$log_file"
  info "Launching $name (log: $log_file)"
  (set -euo pipefail; "$@") >>"$log_file" 2>&1 &
  local pid=$!
  SERVICE_PIDS["$name"]=$pid
  SERVICE_ORDER+=("$name")
}

start_services() {
  start_service "gateway" "$CARGO_BIN" run -p noa_gateway --bin gateway
  start_service "ui_api" "$CARGO_BIN" run -p noa_ui_api --bin noa_ui_api
  start_service "kernel" "$CARGO_BIN" run -p noa_core --bin noa_kernel
  start_service "web_ui" env npm_config_user_agent="$PNPM_AGENT" "${PNPM_BIN[@]}" --filter vibe-kanban dev
  if [[ -n "${LLAMA_SERVER_COMMAND:-}" ]]; then
    start_service "inference" bash -lc "${LLAMA_SERVER_COMMAND}"
  else
    info "Inference server not started (set LLAMA_SERVER_COMMAND to auto-launch llama.cpp)"
  fi
  if [[ $SKIP_NOTEBOOK -eq 0 ]]; then
    start_service "notebook" bash -lc "source '$PY_ENV_DIR/bin/activate' && exec jupyter notebook notebooks/noa_ark_os_unified_interface.ipynb --no-browser --NotebookApp.token='' --NotebookApp.password='' --allow-root"
  else
    info "Notebook launch skipped by flag"
  fi
}

cleanup() {
  local exit_code=$?
  if [[ ${#SERVICE_ORDER[@]} -gt 0 ]]; then
    info "Shutting down services"
    for name in "${SERVICE_ORDER[@]}"; do
      local pid=${SERVICE_PIDS[$name]:-}
      if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
        info "Stopping $name (pid $pid)"
        kill "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
      fi
    done
  fi
  exit "$exit_code"
}

monitor_services() {
  if [[ ${#SERVICE_ORDER[@]} -eq 0 ]]; then
    return
  fi
  info "Services running. Press Ctrl+C to stop."
  while true; do
    sleep 2
    for name in "${SERVICE_ORDER[@]}"; do
      local pid=${SERVICE_PIDS[$name]:-}
      if [[ -z "$pid" ]]; then
        continue
      fi
      if ! kill -0 "$pid" 2>/dev/null; then
        wait "$pid" 2>/dev/null || true
        fail "$name exited unexpectedly. Check $LOG_DIR/${name}.log for details."
      fi
    done
  done
}

main() {
  parse_args "$@"
  ensure_root
  trap cleanup EXIT INT TERM
  activate_toolchains
  prepare_node_pnpm
  setup_python_env
  run_prepare_phase
  if [[ "$LAUNCH_MODE" == "prepare" ]]; then
    info "Prepare phase complete. Exiting due to --prepare-only."
    return
  fi
  start_services
  monitor_services
}

main "$@"
