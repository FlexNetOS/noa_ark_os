#!/usr/bin/env bash
set -euo pipefail

LAUNCH_MODE="full"
RUN_TESTS=1
SKIP_NOTEBOOK=0
SKIP_DOCKER=0
SKIP_MAKE=0
MASTER_CONTROLLER_DRY_RUN=1
MASTER_CONTROLLER_INTERVAL=60
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
DOCKER_COMPOSE_FILE=""
DOCKER_SERVICES="postgres redis qdrant"
DOCKER_STACK_STARTED=0
declare -a DOCKER_CMD
MAKE_TARGETS=""
POWERSHELL_BIN="${POWERSHELL_BIN:-pwsh}"
POWERSHELL_AVAILABLE=0
PW_SH_HINT_EMITTED=0
LLAMA_MODEL_SIZE="3b"
LLAMA_BUILD_FROM_SOURCE=0
LLAMA_SKIP_DOWNLOAD=0
CUDA_RUN_DOWNLOAD=0
CUDA_RUN_INSTALL=0
CUDA_RUN_VERIFY=1
declare -a PIPELINE_COMMANDS
declare -a LLAMA_SERVER_CMD

# Phase orchestration
KERNEL_MODE="auto"
CUDA_MODE="auto"
LLAMA_MODE="auto"
MASTER_CONTROLLER_MODE="auto"
PIPELINE_MODE="auto"
declare -A PHASE_STATE
declare -A PHASE_REASON
PHASE_ORDER=("powershell_activation" "kernel" "cuda" "llama_server" "master_controller" "pipeline_evidence")
PHASE_REASON_TMP=""
KERNEL_SENTINEL=""
CUDA_SENTINEL=""
LLAMA_SENTINEL=""
PIPELINE_SENTINEL=""
PWSH_SENTINEL=""
HOST_OS=""
IS_WINDOWS=0

usage() {
  cat <<'EOF'
Usage: scripts/full_stack_launch.sh [options]

Options:
  --prepare-only     Run dependency installs and build/test steps, then exit.
  --skip-tests       Skip the long-running pnpm/cargo test suites during prepare.
  --skip-notebook    Do not start the Jupyter notebook server.
  --skip-docker      Skip docker compose dependencies (Postgres/Redis/Qdrant).
  --skip-make        Skip Makefile targets after the prepare phase.
  --make-targets "t"  Override Makefile targets to run (space-delimited).
  --docker-compose-file <path>
                      Override docker compose file (default server/docker-compose.yml).
  --kernel-mode <auto|force|skip>
                      Control kernel hardening (make kernel && make image).
  --cuda-mode <auto|force|skip>
                      Control CUDA toolkit provisioning (PowerShell-only helper).
  --llama-mode <auto|force|skip>
                      Control llama.cpp provisioning and inference launch.
  --master-controller-mode <auto|force|skip>
                      Control scripts/autonomous/master-controller.ps1 orchestration.
  --pipeline-mode <auto|force|skip>
                      Control scripts/pipeline/record_local_pipeline.sh evidence capture.
  --skip-cuda-setup  Skip CUDA verification and installer checks. (legacy; maps to --cuda-mode skip)
  --cuda-download    Allow CUDA setup script to download installers.
  --cuda-install     Allow CUDA setup script to run installers.
  --skip-llama-setup Skip llama.cpp provisioning script. (legacy; maps to --llama-mode skip)
  --llama-build      Build llama.cpp from source instead of downloading.
  --llama-skip-download
                      Do not download GGUF models (assume already present).
  --llama-model-size <size>
                      GGUF size for setup script (3b/7b/8b).
  --skip-llama-start Skip launching the llama.cpp inference server. (legacy; maps to --llama-mode skip)
  --run-master-controller
                      Legacy flag equal to --master-controller-mode force (dry-run by default).
  --master-controller-live
                      Disable dry-run for the master controller (dangerous).
  --master-controller-interval <seconds>
                      Loop interval for master controller automation.
  --skip-pipeline-record
                      Do not run scripts/pipeline/record_local_pipeline.sh. (legacy; maps to --pipeline-mode skip)
  --help             Show this message and exit.

With no flags the script prepares the workspace and launches the full stack.
EOF
}

validate_mode() {
  local value="$1"
  case "$value" in
    auto|force|skip)
      return 0
      ;;
    *)
      fail "Invalid mode '$value'. Allowed values: auto, force, skip."
      ;;
  esac
}

set_mode_var() {
  local var_name="$1"
  local new_value="$2"
  validate_mode "$new_value"
  printf -v "$var_name" '%s' "$new_value"
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
      --skip-docker)
        SKIP_DOCKER=1
        shift
        ;;
      --skip-make)
        SKIP_MAKE=1
        shift
        ;;
      --make-targets)
        if [[ $# -lt 2 ]]; then
          echo "--make-targets requires a value" >&2
          exit 2
        fi
        MAKE_TARGETS="$2"
        shift 2
        ;;
      --docker-compose-file)
        if [[ $# -lt 2 ]]; then
          echo "--docker-compose-file requires a path" >&2
          exit 2
        fi
        DOCKER_COMPOSE_FILE="$2"
        shift 2
        ;;
      --kernel-mode)
        if [[ $# -lt 2 ]]; then
          echo "--kernel-mode requires a value" >&2
          exit 2
        fi
        set_mode_var KERNEL_MODE "$2"
        shift 2
        ;;
      --cuda-mode)
        if [[ $# -lt 2 ]]; then
          echo "--cuda-mode requires a value" >&2
          exit 2
        fi
        set_mode_var CUDA_MODE "$2"
        shift 2
        ;;
      --llama-mode)
        if [[ $# -lt 2 ]]; then
          echo "--llama-mode requires a value" >&2
          exit 2
        fi
        set_mode_var LLAMA_MODE "$2"
        shift 2
        ;;
      --master-controller-mode)
        if [[ $# -lt 2 ]]; then
          echo "--master-controller-mode requires a value" >&2
          exit 2
        fi
        set_mode_var MASTER_CONTROLLER_MODE "$2"
        shift 2
        ;;
      --pipeline-mode)
        if [[ $# -lt 2 ]]; then
          echo "--pipeline-mode requires a value" >&2
          exit 2
        fi
        set_mode_var PIPELINE_MODE "$2"
        shift 2
        ;;
      --skip-cuda-setup)
        set_mode_var CUDA_MODE skip
        shift
        ;;
      --cuda-download)
        CUDA_RUN_DOWNLOAD=1
        shift
        ;;
      --cuda-install)
        CUDA_RUN_INSTALL=1
        shift
        ;;
      --skip-llama-setup)
        set_mode_var LLAMA_MODE skip
        shift
        ;;
      --llama-build)
        LLAMA_BUILD_FROM_SOURCE=1
        shift
        ;;
      --llama-skip-download)
        LLAMA_SKIP_DOWNLOAD=1
        shift
        ;;
      --llama-model-size)
        if [[ $# -lt 2 ]]; then
          echo "--llama-model-size requires a value" >&2
          exit 2
        fi
        LLAMA_MODEL_SIZE="$2"
        shift 2
        ;;
      --skip-llama-start)
        set_mode_var LLAMA_MODE skip
        shift
        ;;
      --run-master-controller)
        set_mode_var MASTER_CONTROLLER_MODE force
        shift
        ;;
      --master-controller-live)
        MASTER_CONTROLLER_DRY_RUN=0
        shift
        ;;
      --master-controller-interval)
        if [[ $# -lt 2 ]]; then
          echo "--master-controller-interval requires seconds" >&2
          exit 2
        fi
        MASTER_CONTROLLER_INTERVAL="$2"
        shift 2
        ;;
      --skip-pipeline-record)
        set_mode_var PIPELINE_MODE skip
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

set_phase_status() {
  local phase="$1"
  local state="$2"
  local reason="$3"
  PHASE_STATE["$phase"]="$state"
  PHASE_REASON["$phase"]="$reason"
}

init_phase_statuses() {
  for phase in "${PHASE_ORDER[@]}"; do
    set_phase_status "$phase" "pending" "waiting for evaluation"
  done
}

print_phase_summary() {
  printf '\n=== Launch Phase Summary ===\n'
  printf '%-24s %-10s %s\n' "Phase" "State" "Details"
  for phase in "${PHASE_ORDER[@]}"; do
    printf '%-24s %-10s %s\n' "$phase" "${PHASE_STATE[$phase]}" "${PHASE_REASON[$phase]}"
  done
  printf '\n'
}

detect_host_platform() {
  HOST_OS=$(uname -s 2>/dev/null || echo "unknown")
  IS_WINDOWS=0
  if [[ "${OS:-}" == "Windows_NT" ]]; then
    IS_WINDOWS=1
  elif [[ "$HOST_OS" =~ MINGW|MSYS|CYGWIN ]]; then
    IS_WINDOWS=1
  fi
  if [[ -n "${WSL_DISTRO_NAME:-}" ]]; then
    # Treat WSL as Linux to avoid invoking Windows-only installers
    IS_WINDOWS=0
  fi
}

initialize_launch_paths() {
  KERNEL_SENTINEL="$LOG_DIR/kernel-status.json"
  CUDA_SENTINEL="$LOG_DIR/cuda-status.json"
  LLAMA_SENTINEL="$LOG_DIR/llama-status.json"
  PIPELINE_SENTINEL="$LOG_DIR/pipeline-status.json"
  PWSH_SENTINEL="$LOG_DIR/pwsh-activation.json"
}

announce_missing_powershell() {
  local phases="$*"
  if [[ $PW_SH_HINT_EMITTED -eq 1 ]]; then
    return
  fi
  PW_SH_HINT_EMITTED=1
  cat <<EOF
[INFO] PowerShell prerequisite missing for: $phases
       Install PowerShell 7+: https://learn.microsoft.com/powershell/scripting/install/installing-powershell
       Once available, rerun scripts/full_stack_launch.sh or set POWERSHELL_BIN to the executable path.
EOF
}

check_pwsh_prereqs() {
  local needs=()
  [[ "$CUDA_MODE" != "skip" ]] && needs+=("CUDA setup")
  [[ "$LLAMA_MODE" != "skip" ]] && needs+=("llama server")
  [[ "$MASTER_CONTROLLER_MODE" != "skip" ]] && needs+=("master controller")
  if (( ${#needs[@]} == 0 )); then
    return
  fi
  if ensure_powershell; then
    return
  fi
  announce_missing_powershell "${needs[*]}"
}

info() {
  printf '[%s] %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*"
}

fail() {
  printf '\n[ERROR] %s\n' "$1" >&2
  exit "${2:-1}"
}

json_escape() {
  local text="$1"
  text="${text//\\/\\\\}"
  text="${text//\"/\\\"}"
  text="${text//$'\n'/\\n}"
  printf '%s' "$text"
}

compute_sha256() {
  local target="$1"
  if [[ -z "$target" || ! -f "$target" ]]; then
    printf 'unavailable'
    return
  fi
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$target" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$target" | awk '{print $1}'
  else
    printf 'unavailable'
  fi
}

record_pwsh_activation() {
  local status="$1"
  local reason="$2"
  local bin="$3"
  local manifest="$4"
  local manifest_sha
  manifest_sha=$(compute_sha256 "$manifest")
  if [[ -z "$PWSH_SENTINEL" ]]; then
    return
  fi
  cat >"$PWSH_SENTINEL" <<EOF
{
  "status": "$(json_escape "$status")",
  "reason": "$(json_escape "$reason")",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "binary": "$(json_escape "${bin:-}")",
  "manifest": "$(json_escape "${manifest:-}")",
  "manifest_sha256": "$(json_escape "$manifest_sha")"
}
EOF
}

activate_portable_pwsh() {
  local activator="$ROOT/server/tools/activate-pwsh.sh"
  local manifest="$ROOT/server/tools/pwsh-portable.manifest.json"
  local reason=""
  local previous_silent="${NOA_ACTIVATE_SILENT:-0}"
  if [[ -n "${NOA_PWSH_ENV:-}" ]]; then
    reason="NOA_PWSH_ENV preset ($POWERSHELL_BIN)"
    record_pwsh_activation "preactivated" "$reason" "$POWERSHELL_BIN" "$manifest"
    set_phase_status "powershell_activation" "skipped" "$reason"
    return 0
  fi
  if [[ ! -f "$activator" ]]; then
    reason="missing $activator"
    record_pwsh_activation "missing_activator" "$reason" "" "$manifest"
    set_phase_status "powershell_activation" "skipped" "no activator; run setup-portable-pwsh.sh"
    return 1
  fi
  if NOA_ACTIVATE_SILENT=1 source "$activator"; then
    reason="portable bundle activated"
    record_pwsh_activation "activated" "$reason" "$POWERSHELL_BIN" "$manifest"
    set_phase_status "powershell_activation" "ran" "$reason"
    NOA_ACTIVATE_SILENT="$previous_silent"
    return 0
  else
    local rc=$?
    reason="activator exit $rc"
    record_pwsh_activation "failed" "$reason" "$POWERSHELL_BIN" "$manifest"
    set_phase_status "powershell_activation" "failed" "activator failed; run setup-portable-pwsh.sh"
    NOA_ACTIVATE_SILENT="$previous_silent"
    return 1
  fi
}

ensure_root() {
  if ! ROOT=$(git rev-parse --show-toplevel 2>/dev/null); then
    fail "Run this script inside the noa_ark_os repository"
  fi
  cd "$ROOT"
  LOG_DIR="$ROOT/build_output/system-launch"
  PY_ENV_DIR="$ROOT/noa_ark_os_env"
  mkdir -p "$LOG_DIR"
  if [[ -z "$DOCKER_COMPOSE_FILE" ]]; then
    DOCKER_COMPOSE_FILE="$ROOT/server/docker-compose.yml"
  fi
}

append_pipeline_cmd() {
  local cmd="$1"
  PIPELINE_COMMANDS+=("$cmd")
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
  [[ -z "$version" || "$version" == "None" ]] && version="9.11.0"
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
  local pnpm_cmd="${PNPM_BIN[*]:-pnpm}"
  env npm_config_user_agent="$PNPM_AGENT" "${PNPM_BIN[@]}" install --frozen-lockfile
  append_pipeline_cmd "$pnpm_cmd install --frozen-lockfile"

  info "Running pnpm lint/build/typecheck"
  env npm_config_user_agent="$PNPM_AGENT" "${PNPM_BIN[@]}" lint
  append_pipeline_cmd "$pnpm_cmd lint"
  env npm_config_user_agent="$PNPM_AGENT" "${PNPM_BIN[@]}" build
  append_pipeline_cmd "$pnpm_cmd build"
  env npm_config_user_agent="$PNPM_AGENT" "${PNPM_BIN[@]}" typecheck
  append_pipeline_cmd "$pnpm_cmd typecheck"

  if [[ $RUN_TESTS -eq 1 ]]; then
    info "Running pnpm test suite"
    env npm_config_user_agent="$PNPM_AGENT" "${PNPM_BIN[@]}" test
    append_pipeline_cmd "$pnpm_cmd test"
  else
    info "Skipping pnpm tests"
  fi

  info "Building Rust workspace"
  "$CARGO_BIN" build --workspace
  append_pipeline_cmd "$CARGO_BIN build --workspace"

  info "Formatting Rust workspace"
  "$CARGO_BIN" fmt --all
  append_pipeline_cmd "$CARGO_BIN fmt --all"

  info "Running cargo clippy"
  "$CARGO_BIN" clippy --workspace -- -D warnings
  append_pipeline_cmd "$CARGO_BIN clippy --workspace -- -D warnings"

  if [[ $RUN_TESTS -eq 1 ]]; then
    info "Running cargo test suite"
    "$CARGO_BIN" test --workspace
    append_pipeline_cmd "$CARGO_BIN test --workspace"
  else
    info "Skipping cargo tests"
  fi

  info "Notebook sync"
  "$CARGO_BIN" run -p noa_symbol_graph --bin notebook_watcher -- --once .
  append_pipeline_cmd "$CARGO_BIN run -p noa_symbol_graph --bin notebook_watcher -- --once ."
  env npm_config_user_agent="$PNPM_AGENT" "${PNPM_BIN[@]}" notebooks:sync
  append_pipeline_cmd "$pnpm_cmd notebooks:sync"
}

configure_make_targets() {
  if [[ $SKIP_MAKE -eq 1 ]]; then
    MAKE_TARGETS=""
    return
  fi
  if [[ -n "$MAKE_TARGETS" ]]; then
    return
  fi
  if [[ $RUN_TESTS -eq 1 ]]; then
    MAKE_TARGETS="pipeline.local"
  else
    MAKE_TARGETS="deps lint typecheck format"
  fi
}

run_duplicate_task_guard() {
  local guard_script="$ROOT/scripts/lib/ensure_no_duplicate_tasks.sh"
  if [[ -x "$guard_script" ]]; then
    "$guard_script"
  else
    info "Duplicate-task guard helper missing or not executable at $guard_script"
    info "Skipping AGENT.md enforcement for duplicate tasks"
  fi
}

run_make_phase() {
  if [[ $SKIP_MAKE -eq 1 ]]; then
    info "Make phase skipped by flag"
    return
  fi
  if [[ -z "$MAKE_TARGETS" ]]; then
    info "No Makefile targets configured; skipping make phase"
    return
  fi
  info "Running Makefile targets: $MAKE_TARGETS"
  for target in $MAKE_TARGETS; do
    info "make $target"
    make "$target"
    append_pipeline_cmd "make $target"
  done
}

ensure_powershell() {
  if [[ $POWERSHELL_AVAILABLE -eq 1 ]]; then
    return 0
  fi
  if [[ -n "${NOA_PWSH_ENV:-}" && -n "${POWERSHELL_BIN:-}" && -x "$POWERSHELL_BIN" ]]; then
    POWERSHELL_AVAILABLE=1
    return 0
  fi
  if command -v "$POWERSHELL_BIN" >/dev/null 2>&1; then
    POWERSHELL_AVAILABLE=1
    return 0
  fi
  if command -v pwsh >/dev/null 2>/dev/null; then
    POWERSHELL_BIN="pwsh"
    POWERSHELL_AVAILABLE=1
    return 0
  fi
  if command -v powershell >/dev/null 2>/dev/null; then
    POWERSHELL_BIN="powershell"
    POWERSHELL_AVAILABLE=1
    return 0
  fi
  info "PowerShell executable not found; skipping PowerShell-based integrations"
  return 1
}

run_pwsh_script() {
  local script="$1"
  shift || true
  if [[ ! -f "$script" ]]; then
    info "PowerShell script missing: $script"
    return 1
  fi
  if ! ensure_powershell; then
    return 1
  fi
  info "Running PowerShell script: $script $*"
  "$POWERSHELL_BIN" -NoProfile -ExecutionPolicy Bypass -File "$script" "$@"
}

needs_kernel_build() {
  PHASE_REASON_TMP=""
  if [[ ! -d "$ROOT/dist/kernel" ]]; then
    PHASE_REASON_TMP="dist/kernel missing"
    return 0
  fi
  if [[ ! -f "$KERNEL_SENTINEL" ]]; then
    PHASE_REASON_TMP="kernel sentinel missing"
    return 0
  fi
  local recorded_sha
  recorded_sha=$(awk -F'"' '/git_sha/{print $4}' "$KERNEL_SENTINEL" 2>/dev/null || echo "")
  local current_sha
  current_sha=$(git rev-parse HEAD 2>/dev/null || echo "")
  if [[ -z "$recorded_sha" || "$recorded_sha" != "$current_sha" ]]; then
    PHASE_REASON_TMP="workspace HEAD changed (was ${recorded_sha:-unknown})"
    return 0
  fi
  PHASE_REASON_TMP="artifacts align with commit $current_sha"
  return 1
}

record_kernel_state() {
  local current_sha
  current_sha=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
  cat >"$KERNEL_SENTINEL" <<EOF
{
  "git_sha": "$current_sha",
  "recorded_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
}

run_kernel_build() {
  info "Hardening kernel (make kernel && make image)"
  make kernel
  append_pipeline_cmd "make kernel"
  make image
  append_pipeline_cmd "make image"
  record_kernel_state
}

maybe_run_kernel_build() {
  local reason=""
  case "$KERNEL_MODE" in
    skip)
      set_phase_status "kernel" "skipped" "mode=skip"
      return
      ;;
    force)
      reason="forced"
      ;;
    auto)
      if needs_kernel_build; then
        reason="$PHASE_REASON_TMP"
      else
        set_phase_status "kernel" "skipped" "$PHASE_REASON_TMP"
        return
      fi
      ;;
  esac
  run_kernel_build
  set_phase_status "kernel" "ran" "${reason:-manual trigger}"
}

needs_cuda_setup() {
  PHASE_REASON_TMP=""
  if [[ ! -f "$CUDA_SENTINEL" ]]; then
    PHASE_REASON_TMP="cuda sentinel missing"
    return 0
  fi
  local recorded_at
  recorded_at=$(awk -F'"' '/recorded_at/{print $4}' "$CUDA_SENTINEL" 2>/dev/null || echo "previous run")
  PHASE_REASON_TMP="CUDA verified at $recorded_at"
  return 1
}

record_cuda_state() {
  cat >"$CUDA_SENTINEL" <<EOF
{
  "recorded_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "host": "$HOST_OS"
}
EOF
}

maybe_run_cuda_setup() {
  local reason=""
  case "$CUDA_MODE" in
    skip)
      set_phase_status "cuda" "skipped" "mode=skip"
      return
      ;;
    force)
      reason="forced"
      ;;
    auto)
      if needs_cuda_setup; then
        reason="$PHASE_REASON_TMP"
      else
        set_phase_status "cuda" "skipped" "$PHASE_REASON_TMP"
        return
      fi
      ;;
  esac
  if [[ $IS_WINDOWS -ne 1 ]]; then
    set_phase_status "cuda" "skipped" "requires Windows PowerShell"
    return
  fi
  if ! ensure_powershell; then
    set_phase_status "cuda" "skipped" "PowerShell unavailable"
    return
  fi
  run_cuda_setup
  record_cuda_state
  set_phase_status "cuda" "ran" "${reason:-manual trigger}"
}

llama_binary_path() {
  printf '%s' "$ROOT/server/ai/llama-cpp/bin/llama-server.exe"
}

llama_assets_ready() {
  PHASE_REASON_TMP=""
  local binary
  binary=$(llama_binary_path)
  if [[ ! -f "$binary" ]]; then
    PHASE_REASON_TMP="llama binary missing"
    return 1
  fi
  local models_dir="$ROOT/server/ai/llama-cpp/models"
  if [[ ! -d "$models_dir" ]]; then
    PHASE_REASON_TMP="models directory missing"
    return 1
  fi
  if ! find "$models_dir" -maxdepth 1 -name '*.gguf' -print -quit >/dev/null 2>&1; then
    PHASE_REASON_TMP="no GGUF models in $models_dir"
    return 1
  fi
  PHASE_REASON_TMP="binary + models present"
  return 0
}

record_llama_state() {
  cat >"$LLAMA_SENTINEL" <<EOF
{
  "recorded_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "model_size": "$LLAMA_MODEL_SIZE"
}
EOF
}

maybe_prepare_llama_server() {
  LLAMA_SERVER_CMD=()
  local reason=""
  case "$LLAMA_MODE" in
    skip)
      set_phase_status "llama_server" "skipped" "mode=skip"
      return
      ;;
    force)
      reason="forced"
      ;;
    auto)
      reason="auto"
      ;;
  esac
  if [[ $IS_WINDOWS -ne 1 ]]; then
    set_phase_status "llama_server" "skipped" "requires Windows PowerShell"
    return
  fi
  if ! ensure_powershell; then
    set_phase_status "llama_server" "skipped" "PowerShell unavailable"
    return
  fi
  if llama_assets_ready; then
    reason="$PHASE_REASON_TMP"
  else
    info "Llama assets missing ($PHASE_REASON_TMP); attempting setup"
    run_llama_setup
    record_llama_state
    if ! llama_assets_ready; then
      set_phase_status "llama_server" "skipped" "assets unavailable after setup"
      return
    fi
    reason="assets provisioned"
  fi
  local script="$ROOT/scripts/dev/start-llama-server.ps1"
  if [[ ! -f "$script" ]]; then
    set_phase_status "llama_server" "skipped" "missing $script"
    return
  fi
  LLAMA_SERVER_CMD=("$POWERSHELL_BIN" -NoProfile -ExecutionPolicy Bypass -File "$script")
  LLAMA_SERVER_READY=1
  set_phase_status "llama_server" "planned" "${reason:-prereqs satisfied}"
}

run_cuda_setup() {
  local script="$ROOT/scripts/setup/setup-cuda.ps1"
  if [[ ! -f "$script" ]]; then
    info "CUDA setup script not found (expected $script)"
    return
  fi
  local args=("-WorkspaceRoot" "$ROOT")
  if [[ $CUDA_RUN_DOWNLOAD -eq 1 ]]; then
    args+=("-Download")
  fi
  if [[ $CUDA_RUN_INSTALL -eq 1 ]]; then
    args+=("-Install")
  fi
  if [[ $CUDA_RUN_VERIFY -eq 1 ]]; then
    args+=("-Verify")
  fi
  run_pwsh_script "$script" "${args[@]}"
}

run_llama_setup() {
  local script="$ROOT/scripts/dev/setup-llama-cpp.ps1"
  if [[ ! -f "$script" ]]; then
    info "Llama setup script not found (expected $script)"
    return
  fi
  local args=("-WorkspaceRoot" "$ROOT" "-ModelSize" "$LLAMA_MODEL_SIZE")
  if [[ $LLAMA_BUILD_FROM_SOURCE -eq 1 ]]; then
    args+=("-BuildFromSource")
  fi
  if [[ $LLAMA_SKIP_DOWNLOAD -eq 1 ]]; then
    args+=("-SkipModelDownload")
  fi
  run_pwsh_script "$script" "${args[@]}"
}

maybe_launch_master_controller() {
  local reason=""
  case "$MASTER_CONTROLLER_MODE" in
    skip)
      set_phase_status "master_controller" "skipped" "mode=skip"
      return
      ;;
    force)
      reason="forced"
      ;;
    auto)
      reason="auto"
      ;;
  esac
  local script="$ROOT/scripts/autonomous/master-controller.ps1"
  if [[ ! -f "$script" ]]; then
    set_phase_status "master_controller" "skipped" "missing $script"
    return
  fi
  if ! ensure_powershell; then
    set_phase_status "master_controller" "skipped" "PowerShell unavailable"
    return
  fi
  local args=(-NoProfile -ExecutionPolicy Bypass -File "$script" -EnableAutoLaunch -LoopInterval "$MASTER_CONTROLLER_INTERVAL")
  if [[ $MASTER_CONTROLLER_DRY_RUN -eq 1 ]]; then
    args+=(-DryRun)
  fi
  start_service "master_controller" "$POWERSHELL_BIN" "${args[@]}"
  set_phase_status "master_controller" "running" "${reason:-started}"
}

maybe_record_pipeline_evidence() {
  case "$PIPELINE_MODE" in
    skip)
      set_phase_status "pipeline_evidence" "skipped" "mode=skip"
      return
      ;;
  esac
  if [[ ${#PIPELINE_COMMANDS[@]} -eq 0 ]]; then
    set_phase_status "pipeline_evidence" "skipped" "no commands recorded"
    return
  fi
  local recorder="$ROOT/scripts/pipeline/record_local_pipeline.sh"
  if [[ ! -f "$recorder" ]]; then
    set_phase_status "pipeline_evidence" "skipped" "missing $recorder"
    return
  fi
  info "Recording pipeline evidence via $recorder"
  if (cd "$ROOT" && bash "$recorder" "${PIPELINE_COMMANDS[@]}"); then
    set_phase_status "pipeline_evidence" "ran" "captured ${#PIPELINE_COMMANDS[@]} commands"
  else
    set_phase_status "pipeline_evidence" "skipped" "recorder exited non-zero"
  fi
}
detect_docker_compose() {
  if [[ $SKIP_DOCKER -eq 1 ]]; then
    return
  fi
  if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    DOCKER_CMD=(docker compose)
  elif command -v docker-compose >/dev/null 2>&1; then
    DOCKER_CMD=(docker-compose)
  else
    fail "Docker compose command not found. Install docker compose or pass --skip-docker."
  fi
}

wait_for_container_health() {
  local service="$1"
  local timeout="${DOCKER_HEALTH_TIMEOUT:-180}"
  local step=3
  local elapsed=0
  info "Waiting for docker service $service (timeout=${timeout}s)"
  while (( elapsed < timeout )); do
    local container_id
    container_id=$("${DOCKER_CMD[@]}" -f "$DOCKER_COMPOSE_FILE" ps -q "$service" 2>/dev/null || true)
    if [[ -n "$container_id" ]]; then
      local status
      status=$(docker inspect --format='{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}' "$container_id" 2>/dev/null || echo "unknown")
      if [[ "$status" == "healthy" || "$status" == "running" ]]; then
        info "Docker service $service ready (status=$status)"
        return 0
      fi
    fi
    sleep "$step"
    elapsed=$(( elapsed + step ))
  done
  fail "Docker service $service failed health check"
}

start_docker_stack() {
  if [[ $SKIP_DOCKER -eq 1 ]]; then
    info "Docker stack launch skipped by flag"
    return
  fi
  detect_docker_compose
  if [[ ! -f "$DOCKER_COMPOSE_FILE" ]]; then
    fail "Docker compose file $DOCKER_COMPOSE_FILE not found"
  fi
  info "Launching docker stack via ${DOCKER_CMD[*]} -f $DOCKER_COMPOSE_FILE up -d $DOCKER_SERVICES"
  "${DOCKER_CMD[@]}" -f "$DOCKER_COMPOSE_FILE" up -d $DOCKER_SERVICES
  append_pipeline_cmd "${DOCKER_CMD[*]} -f $DOCKER_COMPOSE_FILE up -d $DOCKER_SERVICES"
  DOCKER_STACK_STARTED=1
  for service in $DOCKER_SERVICES; do
    wait_for_container_health "$service"
  done
}

stop_docker_stack() {
  if [[ $DOCKER_STACK_STARTED -eq 1 ]]; then
    info "Stopping docker stack"
    "${DOCKER_CMD[@]}" -f "$DOCKER_COMPOSE_FILE" down || true
    append_pipeline_cmd "${DOCKER_CMD[*]} -f $DOCKER_COMPOSE_FILE down"
    DOCKER_STACK_STARTED=0
  fi
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
  if [[ ${#LLAMA_SERVER_CMD[@]} -gt 0 ]]; then
    start_service "inference" "${LLAMA_SERVER_CMD[@]}"
    set_phase_status "llama_server" "running" "PowerShell launch"
  elif [[ -n "${LLAMA_SERVER_COMMAND:-}" ]]; then
    start_service "inference" bash -lc "${LLAMA_SERVER_COMMAND}"
    set_phase_status "llama_server" "running" "LLAMA_SERVER_COMMAND override"
  else
    if [[ "${PHASE_STATE[llama_server]}" == "pending" ]]; then
      set_phase_status "llama_server" "skipped" "no launch command configured"
    fi
    info "Inference server not started (set LLAMA_SERVER_COMMAND or keep llama start enabled)"
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
  stop_docker_stack
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
  detect_host_platform
  ensure_root
  init_phase_statuses
  initialize_launch_paths
  activate_portable_pwsh || true
  configure_make_targets
  run_duplicate_task_guard
  trap cleanup EXIT INT TERM
  check_pwsh_prereqs
  activate_toolchains
  prepare_node_pnpm
  setup_python_env
  maybe_run_kernel_build
  maybe_run_cuda_setup
  maybe_prepare_llama_server
  run_prepare_phase
  run_make_phase
  maybe_record_pipeline_evidence
  if [[ "$LAUNCH_MODE" == "prepare" ]]; then
    info "Prepare phase complete. Exiting due to --prepare-only."
    if [[ "${PHASE_STATE[master_controller]}" == "pending" ]]; then
      if [[ "$MASTER_CONTROLLER_MODE" == "skip" ]]; then
        set_phase_status "master_controller" "skipped" "mode=skip"
      else
        set_phase_status "master_controller" "skipped" "prepare-only mode"
      fi
    fi
    print_phase_summary
    return
  fi
  start_docker_stack
  start_services
  maybe_launch_master_controller
  print_phase_summary
  monitor_services
}

main "$@"
